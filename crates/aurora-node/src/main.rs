//! Commandes du nœud local AURORA : regtest et testnet expérimental.
mod p2p;
use std::{
    error::Error,
    fs,
    path::PathBuf,
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use aurora_consensus::{
    account_block::AccountBlock, account_fork_chain::AccountForkChain,
    difficulty::DifficultyParams, proof_of_work::mine_nonce, reward::RewardParams,
};
use aurora_core::{accounts::AccountId, hash::Hash256, network::NetworkId, state::AccountLedger};
use aurora_storage::{load_latest_snapshot, save_snapshot, StorageError};
use p2p::{propagate_block, serve as serve_peers, sync_from_peer};

const TESTNET_INITIAL_BITS: u16 = 8;
const TESTNET_TREASURY: AccountId = AccountId([0xA0; 32]);
const TESTNET_DIFFICULTY: DifficultyParams = DifficultyParams {
    target_block_seconds: 60,
    adjustment_interval: 10,
    max_adjustment_bits: 2,
    min_bits: 4,
    max_bits: 20,
};

fn regtest_chain() -> Result<AccountForkChain, Box<dyn Error>> {
    let network = NetworkId::Regtest;
    let genesis = AccountBlock::build(network, Hash256::ZERO, 1, 0, Vec::new())?;
    Ok(AccountForkChain::new(
        genesis,
        0,
        AccountLedger::new(network),
    )?)
}

fn testnet_chain() -> Result<AccountForkChain, Box<dyn Error>> {
    let mut genesis = AccountBlock::build(NetworkId::Testnet, Hash256::ZERO, 1, 0, Vec::new())?;
    genesis.header.nonce = mine_nonce(&genesis.header, TESTNET_INITIAL_BITS, 0, 1_000_000)?;
    Ok(AccountForkChain::new_with_protocol_params(
        genesis,
        TESTNET_INITIAL_BITS,
        TESTNET_DIFFICULTY,
        RewardParams::experimental_testnet(TESTNET_TREASURY),
        AccountLedger::new(NetworkId::Testnet),
    )?)
}

fn load_testnet_chain(dir: &std::path::Path) -> Result<AccountForkChain, Box<dyn Error>> {
    fs::create_dir_all(dir)?;
    let chain = match load_latest_snapshot(dir, AccountLedger::new(NetworkId::Testnet)) {
        Ok(chain) => chain,
        Err(StorageError::NoSnapshot) => {
            let chain = testnet_chain()?;
            save_snapshot(dir, &chain)?;
            chain
        }
        Err(error) => return Err(error.into()),
    };
    if chain.initial_difficulty_bits() != TESTNET_INITIAL_BITS
        || chain.difficulty_params() != TESTNET_DIFFICULTY
        || chain.reward_params() != RewardParams::experimental_testnet(TESTNET_TREASURY)
    {
        return Err("stored chain uses different testnet consensus parameters".into());
    }
    Ok(chain)
}

fn run_testnet_peer(
    args: impl Iterator<Item = String>,
    server: bool,
) -> Result<(), Box<dyn Error>> {
    let mut args = args;
    let mut data_dir: Option<PathBuf> = None;
    let mut endpoint: Option<String> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir = Some(args.next().ok_or("--data-dir requires a path")?.into())
            }
            "--listen" if server => {
                endpoint = Some(args.next().ok_or("--listen requires IP:PORT")?)
            }
            "--peer" if !server => endpoint = Some(args.next().ok_or("--peer requires IP:PORT")?),
            _ => return Err(format!("unknown testnet option: {arg}").into()),
        }
    }
    let dir = data_dir.ok_or("testnet peer command requires --data-dir PATH")?;
    let endpoint: std::net::SocketAddr = endpoint
        .ok_or("missing --listen or --peer endpoint")?
        .parse()?;
    let mut chain = load_testnet_chain(&dir)?;
    if server {
        serve_peers(endpoint, &mut chain, &dir)?;
    } else {
        let accepted = sync_from_peer(endpoint, &mut chain, &dir)?;
        println!(
            "peer sync complete: accepted={accepted} height={} tip={}",
            chain.height(),
            chain.tip_hash()
        );
    }
    Ok(())
}

fn parse_account(value: &str) -> Result<AccountId, Box<dyn Error>> {
    if value.len() != 64 {
        return Err("--miner requires exactly 64 hexadecimal characters (32 bytes)".into());
    }
    let mut bytes = [0u8; 32];
    for (i, slot) in bytes.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16)
            .map_err(|_| "--miner contains a non-hexadecimal character")?;
    }
    Ok(AccountId(bytes))
}

fn mine_one(
    chain: &mut AccountForkChain,
    miner: AccountId,
    timestamp: u64,
) -> Result<AccountBlock, Box<dyn Error>> {
    let timestamp = timestamp.max(chain.median_time_past().saturating_add(1));
    let difficulty = chain.next_difficulty_bits()?;
    let mut block = AccountBlock::build_with_miner(
        NetworkId::Testnet,
        chain.tip_hash(),
        timestamp,
        0,
        Some(miner),
        Vec::new(),
    )?;
    block.header.nonce = mine_nonce(&block.header, difficulty, 0, u64::MAX)?;
    let update = chain.submit(block.clone())?;
    if !update.became_best {
        return Err("mined block did not become the best chain".into());
    }
    Ok(block)
}

fn run_testnet_miner(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let mut args = args;
    let mut data_dir: Option<PathBuf> = None;
    let mut miner: Option<AccountId> = None;
    let mut count = 1u64;
    let mut peers = Vec::<std::net::SocketAddr>::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir = Some(args.next().ok_or("--data-dir requires a path")?.into())
            }
            "--miner" => {
                miner = Some(parse_account(
                    &args.next().ok_or("--miner requires an account id")?,
                )?)
            }
            "--count" => {
                count = args
                    .next()
                    .ok_or("--count requires a positive integer")?
                    .parse()?
            }
            "--peer" => peers.push(args.next().ok_or("--peer requires IP:PORT")?.parse()?),
            _ => return Err(format!("unknown testnet option: {arg}").into()),
        }
    }
    if count == 0 || count > 10_000 {
        return Err("--count must be between 1 and 10000".into());
    }
    let miner = miner.ok_or("testnet mine requires --miner <64 hex characters>")?;
    let dir = data_dir.ok_or("testnet mine requires --data-dir <path>")?;
    let mut chain = load_testnet_chain(&dir)?;
    for _ in 0..count {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let block = mine_one(&mut chain, miner, now)?;
        save_snapshot(&dir, &chain)?;
        for peer in &peers {
            propagate_block(*peer, &chain, &block)?;
        }
        println!(
            "height={} tip={} difficulty_bits={} issued_units={} miner_balance_units={}",
            chain.height(),
            chain.tip_hash(),
            chain.difficulty_bits(),
            chain.state().total_issued().to_units(),
            chain.state().account(miner).balance.to_units()
        );
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("regtest") => {
            let mut data_dir: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                if arg == "--data-dir" {
                    data_dir = Some(args.next().ok_or("--data-dir requires a path")?.into());
                } else {
                    return Err(format!("unknown regtest option: {arg}").into());
                }
            }
            let chain = if let Some(dir) = &data_dir {
                fs::create_dir_all(dir)?;
                match load_latest_snapshot(dir, AccountLedger::new(NetworkId::Regtest)) {
                    Ok(chain) => chain,
                    Err(StorageError::NoSnapshot) => {
                        let chain = regtest_chain()?;
                        save_snapshot(dir, &chain)?;
                        chain
                    }
                    Err(error) => return Err(error.into()),
                }
            } else {
                regtest_chain()?
            };
            println!("AURORA local regtest (experimental)");
            println!(
                "network: regtest\ndifficulty_bits: {}\nheight: {}\ngenesis: {}",
                chain.difficulty_bits(),
                chain.height(),
                chain.tip_hash()
            );
            println!(
                "validated snapshot loaded from disk; {} accepted block(s)",
                chain.known_block_count()
            );
        }
        Some("testnet") => match args.next().as_deref() {
            Some("mine") => run_testnet_miner(args),
            Some("serve") => run_testnet_peer(args, true),
            Some("sync") => run_testnet_peer(args, false),
            _ => Err("Usage: aurora-node testnet mine|serve|sync ...".into()),
        }?,
        Some("--help") | Some("-h") => {
            println!("Usage:\n  aurora-node regtest [--data-dir PATH]\n  aurora-node testnet mine --data-dir PATH --miner <64 hex chars> [--count N] [--peer IP:PORT]\n  aurora-node testnet serve --data-dir PATH --listen IP:PORT\n  aurora-node testnet sync --data-dir PATH --peer IP:PORT");
            println!("Experimental testnet: TCP pull sync with bounded frames and consensus fingerprint; 8 initial leading-zero bits, 60-second target, 10-block retarget, 500M AUR cap.");
        }
        _ => return Err("Run aurora-node --help for available commands".into()),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("aurora-node: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn testnet_mining_reward_and_snapshot_restart_are_reproducible() {
        let dir = std::env::temp_dir().join(format!(
            "aurora-node-{}-{}",
            process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let miner = AccountId([0x42; 32]);
        let mut chain = testnet_chain().unwrap();
        let _mined = mine_one(&mut chain, miner, 2).unwrap();
        assert_eq!(chain.height(), 1);
        assert!(chain.state().total_issued().to_units() > 0);
        let expected_tip = chain.tip_hash();
        let expected_state = chain.state().clone();
        save_snapshot(&dir, &chain).unwrap();
        let restored = load_latest_snapshot(&dir, AccountLedger::new(NetworkId::Testnet)).unwrap();
        assert_eq!(restored.tip_hash(), expected_tip);
        assert_eq!(restored.state(), &expected_state);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn testnet_miner_identity_parser_is_strict() {
        assert_eq!(
            parse_account(&"ab".repeat(32)).unwrap(),
            AccountId([0xab; 32])
        );
        assert!(parse_account("abcd").is_err());
        assert!(parse_account(&"gg".repeat(32)).is_err());
    }
}

//! Commandes du nœud local AURORA : regtest et testnet expérimental.
use std::{
    collections::{HashSet, VecDeque},
    error::Error,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use aurora_consensus::{
    account_block::AccountBlock,
    account_fork_chain::AccountForkChain,
    proof_of_work::mine_nonce,
    reward::RewardParams,
    testnet::{testnet_genesis_block, TESTNET_DIFFICULTY, TESTNET_INITIAL_BITS, TESTNET_TREASURY},
};
use aurora_core::{
    accounts::{AccountId, AccountTransaction},
    hash::Hash256,
    network::NetworkId,
    state::AccountLedger,
};
use aurora_node::p2p;
use aurora_storage::{load_latest_snapshot, save_snapshot, StorageError};
use p2p::{
    discover_peers_with_retry, fetch_mempool, propagate_block_with_retry, serve as serve_peers,
    sync_from_peer_with_retry, MAX_DISCOVERED_PEERS,
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
    Ok(AccountForkChain::new_with_protocol_params(
        testnet_genesis_block(),
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

// Keep this file handle alive for the entire command. The OS releases the lock
// after a normal exit or a crash; a stale lock file needs no manual cleanup.
fn lock_data_dir(dir: &Path) -> Result<fs::File, Box<dyn Error>> {
    fs::create_dir_all(dir)?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join(".aurora.lock"))?;
    lock.try_lock().map_err(|error| {
        format!(
            "data directory {} is already in use or cannot be locked: {error}",
            dir.display()
        )
    })?;
    Ok(lock)
}

fn run_testnet_verify(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let mut args = args;
    let mut data_dir: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir = Some(args.next().ok_or("--data-dir requires a path")?.into())
            }
            _ => return Err(format!("unknown testnet verify option: {arg}").into()),
        }
    }
    let dir = data_dir.ok_or("testnet verify requires --data-dir PATH")?;
    let chain = load_latest_snapshot(&dir, AccountLedger::new(NetworkId::Testnet))?;
    let expected_genesis = testnet_genesis_block().header.hash()?;
    let stored_genesis = chain
        .blocks()
        .next()
        .ok_or("stored chain has no genesis")?
        .header
        .hash()?;
    if stored_genesis != expected_genesis
        || chain.initial_difficulty_bits() != TESTNET_INITIAL_BITS
        || chain.difficulty_params() != TESTNET_DIFFICULTY
        || chain.reward_params() != RewardParams::experimental_testnet(TESTNET_TREASURY)
    {
        return Err("stored chain uses different testnet consensus parameters".into());
    }
    println!(
        "verified testnet data: height={} tip={} known_blocks={} work={:?}",
        chain.height(),
        chain.tip_hash(),
        chain.known_block_count(),
        chain.chain_work()
    );
    Ok(())
}

fn run_testnet_peer(
    args: impl Iterator<Item = String>,
    server: bool,
) -> Result<(), Box<dyn Error>> {
    let mut args = args;
    let mut data_dir: Option<PathBuf> = None;
    let mut listen: Option<SocketAddr> = None;
    let mut peers = Vec::<SocketAddr>::new();
    let mut discover = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir = Some(args.next().ok_or("--data-dir requires a path")?.into())
            }
            "--listen" if server => {
                listen = Some(args.next().ok_or("--listen requires IP:PORT")?.parse()?);
            }
            "--peer" => peers.push(args.next().ok_or("--peer requires IP:PORT")?.parse()?),
            "--discover" if !server => discover = true,
            _ => return Err(format!("unknown testnet option: {arg}").into()),
        }
    }
    let dir = data_dir.ok_or("testnet peer command requires --data-dir PATH")?;
    let _data_dir_lock = lock_data_dir(&dir)?;
    if server {
        let listen = listen.ok_or("testnet serve requires --listen IP:PORT")?;
        if peers.len() > 32 {
            return Err("testnet serve accepts at most 32 advertised --peer addresses".into());
        }
        let mut chain = load_testnet_chain(&dir)?;
        serve_peers(listen, &mut chain, &dir, &peers)?;
        return Ok(());
    }
    if peers.is_empty() {
        return Err("testnet sync requires at least one --peer IP:PORT".into());
    }
    if peers.len() > MAX_DISCOVERED_PEERS {
        return Err("testnet sync accepts at most 64 initial peers".into());
    }

    let mut chain = load_testnet_chain(&dir)?;
    let mut queue = VecDeque::from(peers);
    let mut contacted = HashSet::new();
    let mut successful = 0usize;
    let mut last_error = None;
    while let Some(endpoint) = queue.pop_front() {
        if !contacted.insert(endpoint) {
            continue;
        }
        if contacted.len() > MAX_DISCOVERED_PEERS {
            break;
        }
        match sync_from_peer_with_retry(endpoint, &mut chain, &dir) {
            Ok(accepted) => {
                successful += 1;
                println!(
                    "peer sync complete: peer={endpoint} accepted={accepted} height={} tip={}",
                    chain.height(),
                    chain.tip_hash()
                );
                if discover {
                    match discover_peers_with_retry(endpoint, &chain) {
                        Ok(discovered) => {
                            for peer in discovered {
                                if !contacted.contains(&peer)
                                    && queue.len() + contacted.len() < MAX_DISCOVERED_PEERS
                                {
                                    queue.push_back(peer);
                                }
                            }
                        }
                        Err(error) => eprintln!("peer discovery failed for {endpoint}: {error}"),
                    }
                }
            }
            Err(error) => {
                eprintln!("peer sync failed for {endpoint}: {error}");
                last_error = Some(error.to_string());
            }
        }
    }
    if successful == 0 {
        if let Some(error) = last_error {
            return Err(format!("all configured testnet peers failed: {error}").into());
        }
        return Err("no testnet peer was contacted".into());
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
    transactions: Vec<AccountTransaction>,
) -> Result<AccountBlock, Box<dyn Error>> {
    let timestamp = timestamp
        .max(chain.median_time_past().saturating_add(1))
        .max(chain.tip_timestamp().saturating_add(1));
    let difficulty = chain.next_difficulty_bits(timestamp)?;
    let mut block = AccountBlock::build_with_miner(
        NetworkId::Testnet,
        chain.tip_hash(),
        timestamp,
        0,
        Some(miner),
        transactions,
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
    let _data_dir_lock = lock_data_dir(&dir)?;
    let mut chain = load_testnet_chain(&dir)?;
    for peer in &peers {
        sync_from_peer_with_retry(*peer, &mut chain, &dir)?;
    }
    for _ in 0..count {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let transactions = match peers.first() {
            Some(peer) => fetch_mempool(*peer, &chain)?,
            None => Vec::new(),
        };
        let block = mine_one(&mut chain, miner, now, transactions)?;
        save_snapshot(&dir, &chain)?;
        for peer in &peers {
            propagate_block_with_retry(*peer, &chain, &block)?;
        }
        println!(
            "height={} tip={} difficulty_bits={} transactions={} issued_units={} miner_balance_units={}",
            chain.height(),
            chain.tip_hash(),
            chain.difficulty_bits(),
            block.transactions.len(),
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
            Some("verify") => run_testnet_verify(args),
            _ => Err("Usage: aurora-node testnet mine|serve|sync|verify ...".into()),
        }?,
        Some("--help") | Some("-h") => {
            println!("Usage:\n  aurora-node regtest [--data-dir PATH]\n  aurora-node testnet mine --data-dir PATH --miner <64 hex chars> [--count N] [--peer IP:PORT]\n  aurora-node testnet serve --data-dir PATH --listen IP:PORT [--peer IP:PORT ...]\n  aurora-node testnet sync --data-dir PATH --peer IP:PORT [--peer IP:PORT ...] [--discover]\n  aurora-node testnet verify --data-dir PATH");
            println!("Experimental testnet: bounded TCP push/pull and opt-in one-hop peer discovery; 8 initial leading-zero bits, 60-second target, 10-block retarget, 500M AUR cap.");
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
        assert_eq!(chain.height(), 0);
        assert_eq!(chain.state().total_issued().to_units(), 0);
        let _mined = mine_one(&mut chain, miner, 2, Vec::new()).unwrap();
        assert_eq!(chain.height(), 1);
        assert_eq!(
            chain.state().total_issued().to_units(),
            500 * aurora_core::amount::UNITS_PER_COIN
        );
        assert_eq!(
            chain.state().account(miner).balance.to_units(),
            475 * aurora_core::amount::UNITS_PER_COIN
        );
        let expected_tip = chain.tip_hash();
        let expected_state = chain.state().clone();
        save_snapshot(&dir, &chain).unwrap();
        let restored = load_latest_snapshot(&dir, AccountLedger::new(NetworkId::Testnet)).unwrap();
        assert_eq!(restored.tip_hash(), expected_tip);
        assert_eq!(restored.state(), &expected_state);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn testnet_verify_rejects_missing_data_and_accepts_saved_chain() {
        let dir = std::env::temp_dir().join(format!(
            "aurora-node-verify-{}-{}",
            process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let args = || ["--data-dir".to_string(), dir.display().to_string()].into_iter();
        assert!(run_testnet_verify(args()).is_err());
        let chain = testnet_chain().unwrap();
        save_snapshot(&dir, &chain).unwrap();
        assert!(run_testnet_verify(args()).is_ok());
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

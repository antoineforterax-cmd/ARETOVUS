//! Commandes locales de diagnostic pour le prototype AURORA.
use std::{error::Error, process};

use aurora_consensus::{account_block::AccountBlock, account_fork_chain::AccountForkChain};
use aurora_core::{hash::Hash256, network::NetworkId, state::AccountLedger};
use aurora_storage::{load_latest_snapshot, save_snapshot, StorageError};
use std::path::PathBuf;

fn regtest_chain() -> Result<AccountForkChain, Box<dyn Error>> {
    let network = NetworkId::Regtest;
    let genesis = AccountBlock::build(network, Hash256::ZERO, 1, 0, Vec::new())?;
    Ok(AccountForkChain::new(
        genesis,
        0,
        AccountLedger::new(network),
    )?)
}

fn run() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1).as_deref() {
        Some("regtest") => {
            let mut args = std::env::args().skip(2);
            let mut data_dir: Option<PathBuf> = None;
            while let Some(arg) = args.next() {
                if arg == "--data-dir" {
                    data_dir = Some(args.next().ok_or("--data-dir requires a path")?.into());
                } else {
                    return Err(format!("unknown regtest option: {arg}").into());
                }
            }
            let chain = if let Some(dir) = &data_dir {
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
            println!("network: regtest");
            println!("difficulty_bits: {}", 0);
            println!("height: {}", chain.height());
            println!("genesis: {}", chain.tip_hash());
            if data_dir.is_some() {
                println!("validated snapshot loaded from disk; canonical blocks only");
            } else {
                println!("state is in memory and will be discarded at exit");
            }
            println!("no peer network is started");
        }
        Some("--help") | Some("-h") => {
            println!("Usage: aurora-node regtest [--data-dir PATH]");
            println!("Validate a deterministic local regtest chain. --data-dir stores versioned, checksummed canonical block snapshots.");
            println!("This command does not start a peer network or mine blocks.");
        }
        _ => {
            println!("AURORA experimental node; no network service is available.");
            println!("Run aurora-node --help for the local regtest smoke check.");
        }
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

    #[test]
    fn regtest_genesis_is_deterministic_and_empty() {
        let first = regtest_chain().unwrap();
        let second = regtest_chain().unwrap();
        assert_eq!(first.height(), 0);
        assert_eq!(first.tip_hash(), second.tip_hash());
        assert_eq!(first.state().total_issued().to_units(), 0);
    }
}

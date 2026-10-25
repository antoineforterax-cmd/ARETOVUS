//! Commandes locales de diagnostic pour le prototype AURORA.
use std::{error::Error, process};

use aurora_consensus::{account_block::AccountBlock, account_fork_chain::AccountForkChain};
use aurora_core::{hash::Hash256, network::NetworkId, state::AccountLedger};

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
            let chain = regtest_chain()?;
            println!("AURORA local regtest (experimental)");
            println!("network: regtest");
            println!("difficulty_bits: {}", 0);
            println!("height: {}", chain.height());
            println!("genesis: {}", chain.tip_hash());
            println!(
                "state is in memory and will be discarded at exit; no peer network is started"
            );
        }
        Some("--help") | Some("-h") => {
            println!("Usage: aurora-node regtest");
            println!("Build and validate the deterministic empty in-memory regtest genesis.");
            println!("This command does not start a peer network or write persistent data.");
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

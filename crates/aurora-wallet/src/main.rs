//! AURORA v3 testnet command-line wallet.
use std::{error::Error, net::SocketAddr, path::Path, thread, time::{Duration, Instant}};

use aurora_core::hash::Hash256;
use aurora_wallet::{
    account_id_hex, build_signed_transaction, parse_account_id, parse_amount, rpc, WalletKey,
};

fn option<'a>(args: &'a [String], name: &str) -> Result<&'a str, Box<dyn Error>> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
        .ok_or_else(|| format!("missing required option {name}").into())
}

fn node_address(args: &[String]) -> Result<SocketAddr, Box<dyn Error>> {
    Ok(option(args, "--node")?.parse()?)
}

fn txid(value: &str) -> Result<Hash256, Box<dyn Error>> {
    let account = parse_account_id(value)?;
    Ok(Hash256::from_bytes(account.0))
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or("Use aurora-wallet --help")?;
    let args = args.collect::<Vec<_>>();

    match command.as_str() {
        "keygen" => {
            let path = Path::new(option(&args, "--out")?);
            let key = WalletKey::generate()?;
            key.save_plain_testnet_file(path)?;
            println!("AURORA TESTNET ONLY: key saved unencrypted at {}", path.display());
            println!("address={}", account_id_hex(key.account_id()));
            println!("Keep this file private. It is not encrypted.");
        }
        "address" => {
            let key = WalletKey::load_plain_testnet_file(Path::new(option(&args, "--key-file")?))?;
            println!("{}", account_id_hex(key.account_id()));
        }
        "balance" => {
            let key = WalletKey::load_plain_testnet_file(Path::new(option(&args, "--key-file")?))?;
            let account = rpc::get_account(node_address(&args)?, key.account_id())?;
            println!(
                "address={} confirmed_balance={} next_nonce={}",
                account_id_hex(key.account_id()),
                account.balance,
                account.next_nonce
            );
        }
        "send" => {
            let key = WalletKey::load_plain_testnet_file(Path::new(option(&args, "--key-file")?))?;
            let recipient = parse_account_id(option(&args, "--to")?)?;
            let amount = parse_amount(option(&args, "--amount")?)?;
            let fee = parse_amount(option(&args, "--fee")?)?;
            let node = node_address(&args)?;
            let account = rpc::get_account(node, key.account_id())?;
            let debit = amount.checked_add(fee)?;
            if account.balance < debit {
                return Err(format!(
                    "insufficient confirmed balance: have {}, need {}",
                    account.balance, debit
                ).into());
            }
            let transaction = build_signed_transaction(
                &key,
                recipient,
                amount,
                fee,
                account.next_nonce,
            )?;
            let (id, pending) = rpc::submit_transaction(node, &transaction)?;
            println!("transaction_id={id}");
            println!("mempool=pending transactions={pending}");
            println!("Use aurora-wallet status --node {} --txid {id} to check confirmation.", node);
        }
        "status" => {
            let id = txid(option(&args, "--txid")?)?;
            let status = rpc::transaction_status(node_address(&args)?, id)?;
            println!("{status:?}");
        }
        "wait" => {
            let id = txid(option(&args, "--txid")?)?;
            let timeout = option(&args, "--timeout-seconds")?.parse::<u64>()?;
            let node = node_address(&args)?;
            let deadline = Instant::now() + Duration::from_secs(timeout);
            loop {
                let status = rpc::transaction_status(node, id)?;
                println!("{status:?}");
                if status == rpc::TransactionStatus::Confirmed {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err("timed out waiting for transaction confirmation".into());
                }
                thread::sleep(Duration::from_secs(2));
            }
        }
        "--help" | "-h" | "help" => {
            println!("AURORA ML-DSA-65 testnet wallet (experimental; plain key files)");
            println!("  aurora-wallet keygen --out FILE");
            println!("  aurora-wallet address --key-file FILE");
            println!("  aurora-wallet balance --key-file FILE --node IP:PORT");
            println!("  aurora-wallet send --key-file FILE --to ACCOUNT_HEX --amount AUR --fee AUR --node IP:PORT");
            println!("  aurora-wallet status --node IP:PORT --txid TXID_HEX");
            println!("  aurora-wallet wait --node IP:PORT --txid TXID_HEX --timeout-seconds N");
        }
        _ => return Err("unknown command; use aurora-wallet --help".into()),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("aurora-wallet: {error}");
        std::process::exit(1);
    }
}

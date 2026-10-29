//! AURORA v3 testnet command-line wallet.
use std::{
    error::Error,
    io::{self, Write},
    net::SocketAddr,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use aurora_core::hash::Hash256;
use aurora_wallet::{
    account_id_hex, build_signed_transaction, parse_account_id, parse_amount, rpc, WalletKey,
};
use zeroize::Zeroize;

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

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

fn load_key(args: &[String]) -> Result<WalletKey, Box<dyn Error>> {
    let path = Path::new(option(args, "--key-file")?);
    if WalletKey::file_is_encrypted(path)? {
        let mut password = rpassword::prompt_password("Wallet password: ")?;
        let result = WalletKey::load_encrypted_testnet_file(path, &password);
        password.zeroize();
        result
    } else {
        eprintln!("WARNING: reading unencrypted testnet key file");
        WalletKey::load_plain_testnet_file(path)
    }
}

fn copy_encrypted_key(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    let mut password = rpassword::prompt_password("Wallet password: ")?;
    let result = WalletKey::copy_encrypted_testnet_file(source, destination, &password);
    password.zeroize();
    let account = result?;
    println!(
        "encrypted_key={} address={}",
        destination.display(),
        account_id_hex(account)
    );
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or("Use aurora-wallet --help")?;
    let args = args.collect::<Vec<_>>();

    match command.as_str() {
        "keygen" => {
            let path = Path::new(option(&args, "--out")?);
            let key = WalletKey::generate()?;
            if flag(&args, "--plain-testnet") {
                key.save_plain_testnet_file(path)?;
                println!(
                    "AURORA TESTNET ONLY: unencrypted key saved at {}",
                    path.display()
                );
            } else {
                let mut password =
                    rpassword::prompt_password("New wallet password (at least 12 characters): ")?;
                let mut confirmation = rpassword::prompt_password("Confirm wallet password: ")?;
                if password != confirmation {
                    password.zeroize();
                    confirmation.zeroize();
                    return Err("passwords do not match".into());
                }
                confirmation.zeroize();
                let result = key.save_encrypted_testnet_file(path, &password);
                password.zeroize();
                result?;
                println!("Encrypted testnet key saved at {}", path.display());
            }
            println!("address={}", account_id_hex(key.account_id()));
            println!("Back up the key file and password separately.");
        }
        "backup" => {
            let source = Path::new(option(&args, "--key-file")?);
            let destination = Path::new(option(&args, "--out")?);
            copy_encrypted_key(source, destination)?;
        }
        "restore" => {
            let source = Path::new(option(&args, "--from")?);
            let destination = Path::new(option(&args, "--out")?);
            copy_encrypted_key(source, destination)?;
        }
        "verify-backup" => {
            let source = Path::new(option(&args, "--from")?);
            let expected = parse_account_id(option(&args, "--expected-account")?)?;
            let mut password = rpassword::prompt_password("Backup password: ")?;
            let result = WalletKey::verify_encrypted_backup(source, &password, expected);
            password.zeroize();
            result?;
            println!("backup_verified account={}", account_id_hex(expected));
        }
        "address" => {
            let key = load_key(&args)?;
            println!("{}", account_id_hex(key.account_id()));
        }
        "balance" => {
            let key = load_key(&args)?;
            let account = rpc::get_account(node_address(&args)?, key.account_id())?;
            println!(
                "address={} pending_adjusted_balance={} next_nonce={}",
                account_id_hex(key.account_id()),
                account.balance,
                account.next_nonce
            );
        }
        "send" => {
            let key = load_key(&args)?;
            let recipient = parse_account_id(option(&args, "--to")?)?;
            let amount = parse_amount(option(&args, "--amount")?)?;
            let fee = parse_amount(option(&args, "--fee")?)?;
            let node = node_address(&args)?;
            let account = rpc::get_account(node, key.account_id())?;
            use aurora_wallet::safety::{check_payment, read_confirmation, PaymentLimits};
            let limits = PaymentLimits {
                max_amount: if flag(&args, "--max-amount") {
                    Some(parse_amount(option(&args, "--max-amount")?)?)
                } else { None },
                max_fee: if flag(&args, "--max-fee") {
                    Some(parse_amount(option(&args, "--max-fee")?)?)
                } else { None },
            };
            let debit = check_payment(key.account_id(), recipient, amount, fee, account.balance, limits)?;
            if flag(&args, "--yes") && flag(&args, "--dry-run") {
                return Err("--yes and --dry-run cannot be combined".into());
            }
            let destination = account_id_hex(recipient);
            println!("Review payment on AURORA Testnet");
            println!("sender={}", account_id_hex(key.account_id()));
            println!("recipient={destination}");
            println!("amount={amount} fee={fee} total_debit={debit}");
            println!("node={node} nonce={} balance_source=pending-adjusted", account.next_nonce);
            if flag(&args, "--dry-run") {
                println!("preview_only: no transaction signed or submitted");
                return Ok(());
            }
            if !flag(&args, "--yes") {
                println!("A confirmed payment is irreversible. Check the full recipient address.");
                print!("Type SEND {destination} to authorize: ");
                io::stdout().flush()?;
                if !read_confirmation(&mut io::stdin().lock(), &destination)? {
                    return Err("payment cancelled before signing".into());
                }
            }
            let refreshed = rpc::get_account(node, key.account_id())?;
            if refreshed.next_nonce != account.next_nonce || refreshed.balance != account.balance {
                return Err("account changed during review; review the payment again".into());
            }
            let transaction =
                build_signed_transaction(&key, recipient, amount, fee, account.next_nonce)?;
            let (id, pending) = rpc::submit_transaction(node, &transaction)?;
            println!("transaction_id={id}");
            println!("mempool=pending transactions={pending}");
            println!(
                "Use aurora-wallet status --node {} --txid {id} to check confirmation.",
                node
            );
        }
        "verify" => {
            let id = txid(option(&args, "--txid")?)?;
            let confirmations = option(&args, "--confirmations")?.parse::<u64>()?;
            let result = aurora_wallet::spv::verify_transaction_inclusion(
                node_address(&args)?,
                id,
                confirmations,
            )?;
            println!(
                "SPV verified: txid={} block_height={} confirmations={} block={} tip_height={} tip={}",
                result.txid,
                result.block_height,
                result.confirmations,
                result.block_hash,
                result.tip_height,
                result.tip_hash
            );
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
            println!("AURORA ML-DSA-65 testnet wallet (experimental)");
            println!("  aurora-wallet keygen --out FILE [--plain-testnet]");
            println!("  aurora-wallet backup --key-file FILE --out BACKUP_FILE");
            println!("  aurora-wallet restore --from BACKUP_FILE --out NEW_FILE");
            println!("  aurora-wallet verify-backup --from FILE --expected-account ACCOUNT_HEX");
            println!("  aurora-wallet address --key-file FILE");
            println!("  aurora-wallet balance --key-file FILE --node IP:PORT");
            println!("    Balance and next_nonce include this node's pending transactions; they are not proof of confirmed state.");
            println!("  aurora-wallet send --key-file FILE --to ACCOUNT_HEX --amount AUR --fee AUR --node IP:PORT [--dry-run | --yes] [--max-amount AUR] [--max-fee AUR]");
            println!("  aurora-wallet verify --node IP:PORT --txid TXID_HEX --confirmations N");
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

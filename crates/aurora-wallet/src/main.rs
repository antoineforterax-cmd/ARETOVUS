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
use zeroize::Zeroizing;

fn option<'a>(args: &'a [String], name: &str) -> Result<&'a str, Box<dyn Error>> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
        .ok_or_else(|| format!("missing required option {name}").into())
}

fn validate_options(
    args: &[String],
    values: &[&str],
    flags: &[&str],
) -> Result<(), Box<dyn Error>> {
    let mut seen = std::collections::BTreeSet::new();
    let mut cursor = 0;
    while cursor < args.len() {
        let name = args[cursor].as_str();
        if !seen.insert(name) {
            return Err("duplicate command option".into());
        }
        if values.contains(&name) {
            cursor += 1;
            let value = args.get(cursor).ok_or("missing option value")?;
            if value.starts_with("--") {
                return Err("missing option value".into());
            }
        } else if !flags.contains(&name) {
            return Err("unsupported option for this command".into());
        }
        cursor += 1;
    }
    Ok(())
}

fn validate_command_options(command: &str, args: &[String]) -> Result<(), Box<dyn Error>> {
    let (values, flags): (&[&str], &[&str]) = match command {
        "keygen" => (&["--out"], &["--plain-testnet"]),
        "backup" => (&["--key-file", "--out"], &[]),
        "restore" => (&["--from", "--out"], &[]),
        "recovery-export" => (&["--key-file", "--out"], &["--acknowledge-secret"]),
        "recovery-verify" => (&["--from", "--expected-account"], &[]),
        "recovery-restore" => (
            &["--from", "--out", "--expected-account"],
            &["--plain-testnet"],
        ),
        "verify-backup" => (&["--from", "--expected-account"], &[]),
        "address" | "limits-show" => (&["--key-file"], &[]),
        "account" => (&["--address", "--node"], &[]),
        "balance" => (&["--key-file", "--node"], &[]),
        "limits-create" => (&["--key-file", "--max-amount", "--max-fee"], &[]),
        "send" => (
            &[
                "--key-file",
                "--to",
                "--amount",
                "--fee",
                "--node",
                "--max-amount",
                "--max-fee",
            ],
            &["--yes", "--dry-run"],
        ),
        "sign-offline" => (
            &[
                "--key-file",
                "--to",
                "--amount",
                "--fee",
                "--nonce",
                "--out",
                "--max-amount",
                "--max-fee",
            ],
            &["--yes", "--dry-run"],
        ),
        "inspect-signed" => (&["--from"], &[]),
        "broadcast" => (&["--from", "--node"], &["--yes"]),
        "verify" => (&["--node", "--txid", "--confirmations"], &[]),
        "status" => (&["--node", "--txid"], &[]),
        "wait" => (&["--node", "--txid", "--timeout-seconds"], &[]),
        "help" | "-h" | "--help" => (&[], &[]),
        _ => return Err("unknown command; use aurora-wallet --help".into()),
    };
    validate_options(args, values, flags)?;
    if flag(args, "--yes") && flag(args, "--dry-run") {
        return Err("--yes and --dry-run cannot be combined".into());
    }
    Ok(())
}

fn wait_deadline(seconds: u64) -> Result<Instant, Box<dyn Error>> {
    Instant::now()
        .checked_add(Duration::from_secs(seconds))
        .ok_or_else(|| "wait timeout is out of range".into())
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
        let password = Zeroizing::new(rpassword::prompt_password("Wallet password: ")?);
        WalletKey::load_encrypted_testnet_file(path, &password)
    } else {
        eprintln!("WARNING: reading unencrypted testnet key file");
        WalletKey::load_plain_testnet_file(path)
    }
}

fn load_recovery_key(args: &[String]) -> Result<WalletKey, Box<dyn Error>> {
    let expected = parse_account_id(option(args, "--expected-account")?)?;
    if flag(args, "--from") {
        WalletKey::load_recovery_file(Path::new(option(args, "--from")?), expected)
    } else {
        let code = Zeroizing::new(rpassword::prompt_password("Recovery code (hidden): ")?);
        WalletKey::from_recovery_code(&code, expected)
    }
}

fn copy_encrypted_key(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    let password = Zeroizing::new(rpassword::prompt_password("Wallet password: ")?);
    let result = WalletKey::copy_encrypted_testnet_file(source, destination, &password);
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

    validate_command_options(&command, &args)?;

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
                let password = Zeroizing::new(rpassword::prompt_password(
                    "New wallet password (at least 12 characters): ",
                )?);
                let confirmation =
                    Zeroizing::new(rpassword::prompt_password("Confirm wallet password: ")?);
                if password.as_str() != confirmation.as_str() {
                    return Err("passwords do not match".into());
                }
                let result = key.save_encrypted_testnet_file(path, &password);
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
        "recovery-export" => {
            if !flag(&args, "--acknowledge-secret") {
                return Err("recovery export is an unencrypted spending secret; add --acknowledge-secret and store it offline".into());
            }
            let destination = Path::new(option(&args, "--out")?);
            let key = load_key(&args)?;
            key.save_recovery_file(destination)?;
            println!(
                "recovery_file={} account={}",
                destination.display(),
                account_id_hex(key.account_id())
            );
            println!(
                "SECRET: anyone with this recovery code can spend. Store it separately offline."
            );
            println!("Verify the recovery copy before relying on it. No secret is printed here.");
        }
        "recovery-verify" => {
            let key = load_recovery_key(&args)?;
            println!(
                "recovery_verified account={}",
                account_id_hex(key.account_id())
            );
        }
        "recovery-restore" => {
            let destination = Path::new(option(&args, "--out")?);
            let key = load_recovery_key(&args)?;
            if flag(&args, "--plain-testnet") {
                key.save_plain_testnet_file(destination)?;
                println!("WARNING: restored an unencrypted testnet key");
            } else {
                let password = Zeroizing::new(rpassword::prompt_password(
                    "New wallet password (at least 12 characters): ",
                )?);
                let confirmation =
                    Zeroizing::new(rpassword::prompt_password("Confirm new wallet password: ")?);
                if *password != *confirmation {
                    return Err("passwords do not match".into());
                }
                key.save_encrypted_testnet_file(destination, &password)?;
            }
            println!(
                "recovered_key={} account={}",
                destination.display(),
                account_id_hex(key.account_id())
            );
            println!("The original signing key is restored, not revoked or rotated.");
        }
        "verify-backup" => {
            let source = Path::new(option(&args, "--from")?);
            let expected = parse_account_id(option(&args, "--expected-account")?)?;
            let password = Zeroizing::new(rpassword::prompt_password("Backup password: ")?);
            let result = WalletKey::verify_encrypted_backup(source, &password, expected);
            result?;
            println!("backup_verified account={}", account_id_hex(expected));
        }
        "address" => {
            let key = load_key(&args)?;
            println!("{}", account_id_hex(key.account_id()));
        }
        "account" => {
            let address = parse_account_id(option(&args, "--address")?)?;
            let account = rpc::get_account(node_address(&args)?, address)?;
            println!(
                "address={} pending_adjusted_balance={} next_nonce={}",
                account_id_hex(address),
                account.balance,
                account.next_nonce
            );
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
        "limits-create" => {
            let key = load_key(&args)?;
            let limits = aurora_wallet::safety::PaymentLimits {
                max_amount: Some(parse_amount(option(&args, "--max-amount")?)?),
                max_fee: Some(parse_amount(option(&args, "--max-fee")?)?),
            };
            let path = Path::new(option(&args, "--key-file")?);
            aurora_wallet::policy::create(path, key.account_id(), limits)?;
            println!(
                "limits_created={} account={}",
                aurora_wallet::policy::policy_path(path).display(),
                account_id_hex(key.account_id())
            );
        }
        "limits-show" => {
            let key = load_key(&args)?;
            let path = Path::new(option(&args, "--key-file")?);
            match aurora_wallet::policy::load(path, key.account_id())? {
                Some(limits) => {
                    println!("limits=active");
                    if let Some(maximum) = limits.max_amount {
                        println!("max_amount={maximum}");
                    }
                    if let Some(maximum) = limits.max_fee {
                        println!("max_fee={maximum}");
                    }
                }
                None => println!("limits=none"),
            }
        }
        "sign-offline" => {
            if flag(&args, "--yes") && flag(&args, "--dry-run") {
                return Err("--yes and --dry-run cannot be combined".into());
            }
            let destination_file = Path::new(option(&args, "--out")?);
            let key = load_key(&args)?;
            let recipient = parse_account_id(option(&args, "--to")?)?;
            let amount = parse_amount(option(&args, "--amount")?)?;
            let fee = parse_amount(option(&args, "--fee")?)?;
            let nonce = option(&args, "--nonce")?.parse::<u64>()?;
            if nonce == u64::MAX {
                return Err("nonce is exhausted".into());
            }
            let key_path = Path::new(option(&args, "--key-file")?);
            let stored = aurora_wallet::policy::load(key_path, key.account_id())?;
            use aurora_wallet::safety::{check_payment, PaymentLimits};
            let requested = PaymentLimits {
                max_amount: if flag(&args, "--max-amount") {
                    Some(parse_amount(option(&args, "--max-amount")?)?)
                } else {
                    None
                },
                max_fee: if flag(&args, "--max-fee") {
                    Some(parse_amount(option(&args, "--max-fee")?)?)
                } else {
                    None
                },
            };
            let limits = stored.unwrap_or_default().stricter(requested);
            let debit = check_payment(
                key.account_id(),
                recipient,
                amount,
                fee,
                aurora_core::amount::Amount::MAX,
                limits,
            )?;
            let destination = account_id_hex(recipient);
            println!("Review OFFLINE payment on AURORA Testnet");
            println!(
                "sender={} recipient={destination}",
                account_id_hex(key.account_id())
            );
            println!("amount={amount} fee={fee} total_debit={debit} nonce={nonce}");
            println!("balance=unverified nonce_source=manual network_requests=none");
            if let Some(maximum) = limits.max_amount {
                println!("effective_max_amount={maximum}");
            }
            if let Some(maximum) = limits.max_fee {
                println!("effective_max_fee={maximum}");
            }
            if flag(&args, "--dry-run") {
                println!("preview_only: no transaction signed or written");
                return Ok(());
            }
            println!("The signed file authorizes spending. Anyone holding it can broadcast; deleting it cannot revoke a copy.");
            if !flag(&args, "--yes") {
                let phrase = aurora_wallet::safety::payment_intent_confirmation_phrase(
                    &destination,
                    amount,
                    fee,
                    nonce,
                );
                print!("Type {phrase} to authorize: ");
                io::stdout().flush()?;
                if !aurora_wallet::safety::read_payment_intent_confirmation(
                    &mut io::stdin().lock(),
                    &destination,
                    amount,
                    fee,
                    nonce,
                )? {
                    return Err("offline payment cancelled before signing".into());
                }
            }
            if aurora_wallet::policy::load(key_path, key.account_id())? != stored {
                return Err("wallet limits changed during review; review the payment again".into());
            }
            let transaction = build_signed_transaction(&key, recipient, amount, fee, nonce)?;
            aurora_wallet::safety::verify_signed_payment_intent(
                &transaction,
                key.account_id(),
                recipient,
                amount,
                fee,
                nonce,
            )?;
            aurora_wallet::signed_payment::save(destination_file, &transaction)?;
            println!(
                "signed_file={} prepared_txid={}",
                destination_file.display(),
                transaction.txid()?
            );
            println!("not_broadcast: nonce and balance must be checked on the connected machine");
        }
        "inspect-signed" => {
            let transaction =
                aurora_wallet::signed_payment::load(Path::new(option(&args, "--from")?))?;
            // load validates canonical encoding, testnet domain and signature.
            println!("verified_signed_transaction=true");
            println!("network=AURORA-TESTNET version={}", transaction.version);
            println!("txid={}", transaction.txid()?);
            println!("sender={}", account_id_hex(transaction.sender));
            println!("recipient={}", account_id_hex(transaction.recipient));
            println!(
                "amount={} fee={} nonce={}",
                transaction.amount, transaction.fee, transaction.nonce
            );
            println!("read_only=true private_key_loaded=false network_requests=none");
            println!("Compare every field on a trusted independent device before broadcasting.");
        }
        "broadcast" => {
            let transaction =
                aurora_wallet::signed_payment::load(Path::new(option(&args, "--from")?))?;
            let node = node_address(&args)?;
            let id = transaction.txid()?;
            let destination = account_id_hex(transaction.recipient);
            println!("Review SIGNED payment on AURORA Testnet txid={id}");
            println!(
                "sender={} recipient={destination}",
                account_id_hex(transaction.sender)
            );
            println!(
                "amount={} fee={} nonce={} node={node}",
                transaction.amount, transaction.fee, transaction.nonce
            );
            if !flag(&args, "--yes") {
                let phrase = aurora_wallet::safety::payment_intent_confirmation_phrase(
                    &destination,
                    transaction.amount,
                    transaction.fee,
                    transaction.nonce,
                );
                print!("Type {phrase} to broadcast: ");
                io::stdout().flush()?;
                if !aurora_wallet::safety::read_payment_intent_confirmation(
                    &mut io::stdin().lock(),
                    &destination,
                    transaction.amount,
                    transaction.fee,
                    transaction.nonce,
                )? {
                    return Err("broadcast cancelled; the signed file remains spendable".into());
                }
            }
            let account = rpc::get_account(node, transaction.sender)?;
            if account.next_nonce != transaction.nonce {
                return Err("signed nonce differs from the node's next nonce; check transaction status before signing another payment".into());
            }
            aurora_wallet::safety::check_payment(
                transaction.sender,
                transaction.recipient,
                transaction.amount,
                transaction.fee,
                account.balance,
                aurora_wallet::safety::PaymentLimits::default(),
            )?;
            let (accepted, pending) = rpc::submit_transaction(node, &transaction)?;
            if accepted != id {
                return Err(
                    "node acknowledged a different transaction id; check the prepared txid status"
                        .into(),
                );
            }
            println!("transaction_id={id}");
            println!("mempool=pending transactions={pending}");
        }
        "send" => {
            let key = load_key(&args)?;
            let recipient = parse_account_id(option(&args, "--to")?)?;
            let amount = parse_amount(option(&args, "--amount")?)?;
            let fee = parse_amount(option(&args, "--fee")?)?;
            let node = node_address(&args)?;
            let key_path = Path::new(option(&args, "--key-file")?);
            let stored_limits = aurora_wallet::policy::load(key_path, key.account_id())?;
            let account = rpc::get_account(node, key.account_id())?;
            use aurora_wallet::safety::{check_payment, PaymentLimits};
            let limits = PaymentLimits {
                max_amount: if flag(&args, "--max-amount") {
                    Some(parse_amount(option(&args, "--max-amount")?)?)
                } else {
                    None
                },
                max_fee: if flag(&args, "--max-fee") {
                    Some(parse_amount(option(&args, "--max-fee")?)?)
                } else {
                    None
                },
            };
            let limits = stored_limits.unwrap_or_default().stricter(limits);
            let debit = check_payment(
                key.account_id(),
                recipient,
                amount,
                fee,
                account.balance,
                limits,
            )?;
            if flag(&args, "--yes") && flag(&args, "--dry-run") {
                return Err("--yes and --dry-run cannot be combined".into());
            }
            let destination = account_id_hex(recipient);
            println!("Review payment on AURORA Testnet");
            if let Some(maximum) = limits.max_amount {
                println!("effective_max_amount={maximum}");
            }
            if let Some(maximum) = limits.max_fee {
                println!("effective_max_fee={maximum}");
            }
            println!("sender={}", account_id_hex(key.account_id()));
            println!("recipient={destination}");
            println!("amount={amount} fee={fee} total_debit={debit}");
            println!(
                "node={node} nonce={} balance_source=pending-adjusted",
                account.next_nonce
            );
            if flag(&args, "--dry-run") {
                println!("preview_only: no transaction signed or submitted");
                return Ok(());
            }
            if !flag(&args, "--yes") {
                println!("A confirmed payment is irreversible. Check the full recipient address.");
                let phrase = aurora_wallet::safety::payment_intent_confirmation_phrase(
                    &destination,
                    amount,
                    fee,
                    account.next_nonce,
                );
                print!("Type {phrase} to authorize: ");
                io::stdout().flush()?;
                if !aurora_wallet::safety::read_payment_intent_confirmation(
                    &mut io::stdin().lock(),
                    &destination,
                    amount,
                    fee,
                    account.next_nonce,
                )? {
                    return Err("payment cancelled before signing".into());
                }
            }
            let refreshed = rpc::get_account(node, key.account_id())?;
            if refreshed.next_nonce != account.next_nonce || refreshed.balance != account.balance {
                return Err("account changed during review; review the payment again".into());
            }
            if aurora_wallet::policy::load(key_path, key.account_id())? != stored_limits {
                return Err("wallet limits changed during review; review the payment again".into());
            }
            let transaction =
                build_signed_transaction(&key, recipient, amount, fee, account.next_nonce)?;
            aurora_wallet::safety::verify_signed_payment_intent(
                &transaction,
                key.account_id(),
                recipient,
                amount,
                fee,
                account.next_nonce,
            )?;
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
            let deadline = wait_deadline(timeout)?;
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
            println!("  aurora-wallet recovery-export --key-file FILE --out SECRET_FILE --acknowledge-secret");
            println!("  aurora-wallet recovery-verify --expected-account ACCOUNT_HEX [--from SECRET_FILE]");
            println!("  aurora-wallet recovery-restore --out NEW_FILE --expected-account ACCOUNT_HEX [--from SECRET_FILE] [--plain-testnet]");
            println!("    Recovery requires a secret saved BEFORE loss. Without --from, the code is entered with hidden input.");
            println!("  aurora-wallet backup --key-file FILE --out BACKUP_FILE");
            println!("  aurora-wallet restore --from BACKUP_FILE --out NEW_FILE");
            println!("  aurora-wallet verify-backup --from FILE --expected-account ACCOUNT_HEX");
            println!("  aurora-wallet address --key-file FILE");
            println!("  aurora-wallet account --address ACCOUNT_HEX --node IP:PORT");
            println!("    Query the public balance and next nonce without loading a key file.");
            println!("  aurora-wallet balance --key-file FILE --node IP:PORT");
            println!("    Balance and next_nonce include this node's pending transactions; they are not proof of confirmed state.");
            println!(
                "  aurora-wallet limits-create --key-file FILE --max-amount AUR --max-fee AUR"
            );
            println!("  aurora-wallet limits-show --key-file FILE");
            println!("    A FILE.limits policy is applied automatically to send, including --yes; one-shot limits only tighten it.");
            println!("  aurora-wallet sign-offline --key-file FILE --to ACCOUNT_HEX --amount AUR --fee AUR --nonce N --out SIGNED_FILE [--dry-run | --yes] [--max-amount AUR] [--max-fee AUR]");
            println!("    No network requests; supplied nonce and balance remain unverified. Wallet limits apply.");
            println!("  aurora-wallet inspect-signed --from SIGNED_FILE");
            println!("    Offline, read-only signature and field inspection; compare independently before broadcast.");
            println!("  aurora-wallet broadcast --from SIGNED_FILE --node IP:PORT [--yes]");
            println!("    The connected machine needs no private key. A signed file is a spending authorization, with no automatic expiry.");
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

#[cfg(test)]
mod wait_deadline_tests {
    use super::*;

    #[test]
    fn zero_timeout_is_a_valid_immediate_deadline() {
        let before = Instant::now();
        let deadline = wait_deadline(0).unwrap();
        assert!(deadline >= before && deadline <= Instant::now());
    }

    #[test]
    fn overflowing_timeout_is_an_error_instead_of_a_panic() {
        assert!(wait_deadline(u64::MAX).is_err());
    }
}

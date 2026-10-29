//! Optional, local per-payment limits automatically associated with a wallet path.
use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use aurora_core::accounts::AccountId;

use crate::{account_id_hex, parse_amount, safety::PaymentLimits};

const HEADER: &str = "AURORA-TESTNET-LIMITS-v1";
const MAX_BYTES: u64 = 1024;

pub fn policy_path(key_file: &Path) -> PathBuf {
    let mut path = key_file.as_os_str().to_os_string();
    path.push(".limits");
    PathBuf::from(path)
}

pub fn create(
    key_file: &Path,
    account: AccountId,
    limits: PaymentLimits,
) -> Result<(), Box<dyn Error>> {
    let amount = limits.max_amount.ok_or("maximum amount is required")?;
    let fee = limits.max_fee.ok_or("maximum fee is required")?;
    if amount.to_units() == 0 {
        return Err("maximum payment amount must be positive".into());
    }
    let text = format!(
        "{HEADER}\naccount={}\nmax_amount={amount}\nmax_fee={fee}\n",
        account_id_hex(account)
    );
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(policy_path(key_file))?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

/// Only absence is optional; unreadable, dangling, oversized or invalid files fail closed.
pub fn load(key_file: &Path, expected: AccountId) -> Result<Option<PaymentLimits>, Box<dyn Error>> {
    let path = policy_path(key_file);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !metadata.file_type().is_file() {
        return Err("wallet limits must be a regular file, not a symlink or directory".into());
    }
    let file = File::open(path)?;
    let mut text = String::new();
    file.take(MAX_BYTES + 1).read_to_string(&mut text)?;
    if text.len() as u64 > MAX_BYTES {
        return Err("wallet limits file exceeds size limit".into());
    }
    Ok(Some(parse(&text, expected)?))
}

fn parse(text: &str, expected: AccountId) -> Result<PaymentLimits, Box<dyn Error>> {
    let lines = text.lines().collect::<Vec<_>>();
    if lines.len() != 4 || lines[0] != HEADER {
        return Err("invalid wallet limits version, network or fields".into());
    }
    if lines[1] != format!("account={}", account_id_hex(expected)) {
        return Err("wallet limits belong to a different account".into());
    }
    let amount = parse_amount(
        lines[2]
            .strip_prefix("max_amount=")
            .ok_or("missing maximum amount")?,
    )?;
    let fee = parse_amount(
        lines[3]
            .strip_prefix("max_fee=")
            .ok_or("missing maximum fee")?,
    )?;
    if amount.to_units() == 0 {
        return Err("maximum payment amount must be positive".into());
    }
    Ok(PaymentLimits {
        max_amount: Some(amount),
        max_fee: Some(fee),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "aurora-limits-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn limits() -> PaymentLimits {
        PaymentLimits {
            max_amount: Some(parse_amount("1").unwrap()),
            max_fee: Some(parse_amount("0.00000003").unwrap()),
        }
    }

    #[test]
    fn stored_limits_survive_reloads_and_never_overwrite() {
        let wallet = path();
        let a = AccountId([1; 32]);
        assert_eq!(load(&wallet, a).unwrap(), None);
        create(&wallet, a, limits()).unwrap();
        for _ in 0..2 {
            assert_eq!(load(&wallet, a).unwrap(), Some(limits()));
        }
        assert!(create(&wallet, a, PaymentLimits::default()).is_err());
        assert!(create(&wallet, a, limits()).is_err());
        assert!(load(&wallet, AccountId([2; 32])).is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(policy_path(&wallet))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        fs::remove_file(policy_path(&wallet)).unwrap();
    }

    #[test]
    fn malformed_or_oversized_policies_are_never_treated_as_absent() {
        let wallet = path();
        let a = AccountId([1; 32]);
        let valid = format!(
            "{HEADER}\naccount={}\nmax_amount=1\nmax_fee=0.00000003\n",
            account_id_hex(a)
        );
        for bad in [
            valid.replace(HEADER, "AURORA-MAINNET-LIMITS-v1"),
            valid.replace("max_fee=", "unknown="),
            valid.replace("max_amount=1", "max_amount=0"),
            valid.replace("max_fee=0.00000003", "max_fee=-1"),
            valid.replace("max_fee=0.00000003", "max_fee=0.000000001"),
            valid.replace("max_amount=1", "max_amount=500000001"),
            format!("{valid}max_fee=0\n"),
            String::new(),
            "x".repeat(1025),
        ] {
            fs::write(policy_path(&wallet), bad).unwrap();
            assert!(load(&wallet, a).is_err());
        }
        fs::write(policy_path(&wallet), [0xff]).unwrap();
        assert!(load(&wallet, a).is_err());
        fs::remove_file(policy_path(&wallet)).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn dangling_symlink_does_not_disable_limits() {
        use std::os::unix::fs::symlink;
        let wallet = path();
        symlink(path(), policy_path(&wallet)).unwrap();
        assert!(load(&wallet, AccountId([1; 32])).is_err());
        fs::remove_file(policy_path(&wallet)).unwrap();
    }
}

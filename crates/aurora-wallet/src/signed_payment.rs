//! Bounded transfer of a signed v3 testnet transaction between offline and online machines.
use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use aurora_core::{
    accounts::{AccountTransaction, POST_QUANTUM_ACCOUNT_TX_VERSION},
    codec::{CanonicalDecode, CanonicalEncode},
    network::NetworkId,
    signature::verify_account_transaction,
};

const MAGIC: &[u8; 8] = b"AURTX3\0\0";
const MAX_FILE_BYTES: u64 = 8192;

pub fn validate(transaction: &AccountTransaction) -> Result<(), Box<dyn Error>> {
    if transaction.version != POST_QUANTUM_ACCOUNT_TX_VERSION
        || transaction.network != NetworkId::Testnet
    {
        return Err("signed payment requires the v3 testnet profile".into());
    }
    transaction.check_stateless()?;
    verify_account_transaction(transaction)?;
    Ok(())
}

pub fn save(path: &Path, transaction: &AccountTransaction) -> Result<(), Box<dyn Error>> {
    validate(transaction)?;
    let bytes = transaction.to_canonical_bytes()?;
    if bytes.len() as u64 + MAGIC.len() as u64 > MAX_FILE_BYTES {
        return Err("signed payment exceeds file size limit".into());
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(MAGIC)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

pub fn load(path: &Path) -> Result<AccountTransaction, Box<dyn Error>> {
    if !fs::metadata(path)?.is_file() {
        return Err("signed payment must be a regular file".into());
    }
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("signed payment exceeds file size limit".into());
    }
    let encoded = bytes.strip_prefix(MAGIC).ok_or("invalid signed payment file header")?;
    let transaction = AccountTransaction::from_canonical_bytes(encoded)?;
    validate(&transaction)?;
    Ok(transaction)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::{accounts::AccountId, amount::Amount};
    use crate::{build_signed_transaction, WalletKey};
    use std::{path::PathBuf, sync::atomic::{AtomicU64, Ordering}};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "aurora-signed-payment-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    fn payment() -> AccountTransaction {
        build_signed_transaction(
            &WalletKey::from_seed_bytes([55; 32]), AccountId([22; 32]),
            Amount::from_units(100).unwrap(), Amount::from_units(3).unwrap(), 0,
        ).unwrap()
    }

    #[test]
    fn signed_file_preserves_exact_transaction_and_refuses_overwrite() {
        let path = path();
        let tx = payment();
        save(&path, &tx).unwrap();
        assert_eq!(load(&path).unwrap(), tx);
        assert_eq!(load(&path).unwrap().txid().unwrap(), tx.txid().unwrap());
        assert!(save(&path, &tx).is_err());
        assert_eq!(load(&path).unwrap(), tx);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        }
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_network_version_or_signed_fields_are_rejected() {
        let path = path();
        let tx = payment();
        for field in 0..7 {
            let mut changed = tx.clone();
            match field {
                0 => changed.network = NetworkId::Regtest,
                1 => changed.version = 1,
                2 => changed.nonce += 1,
                3 => changed.recipient = AccountId([23; 32]),
                4 => changed.amount = Amount::from_units(101).unwrap(),
                5 => changed.fee = Amount::from_units(4).unwrap(),
                _ => changed.auth_data[10] ^= 1,
            }
            assert!(save(&path, &changed).is_err());
            assert!(!path.exists());
            let mut bytes = MAGIC.to_vec();
            bytes.extend_from_slice(&changed.to_canonical_bytes().unwrap());
            fs::write(&path, bytes).unwrap();
            assert!(load(&path).is_err());
            fs::remove_file(&path).unwrap();
        }
    }

    #[test]
    fn truncated_oversized_and_trailing_data_are_rejected() {
        let path = path();
        let tx = payment();
        save(&path, &tx).unwrap();
        let valid = fs::read(&path).unwrap();
        for cut in [0, MAGIC.len() - 1, MAGIC.len(), valid.len() - 1] {
            fs::write(&path, &valid[..cut]).unwrap();
            assert!(load(&path).is_err());
        }
        let mut trailing = valid.clone();
        trailing.push(0);
        fs::write(&path, trailing).unwrap();
        assert!(load(&path).is_err());
        let mut bad_signature = valid;
        *bad_signature.last_mut().unwrap() ^= 1;
        fs::write(&path, bad_signature).unwrap();
        assert!(load(&path).is_err());
        fs::write(&path, vec![0; MAX_FILE_BYTES as usize + 1]).unwrap();
        assert!(load(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}

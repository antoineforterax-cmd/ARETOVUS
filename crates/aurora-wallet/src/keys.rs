//! ML-DSA-65 testnet key generation and local key-file handling.
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use aurora_core::{
    hash::sha256_plain,
    signature::post_quantum_account_id,
    accounts::AccountId,
};
use ml_dsa::{Keypair, MlDsa65, SigningKey};

const KEY_MAGIC: &[u8; 8] = b"AURKEY3\0";
const KEY_FILE_BYTES: usize = 8 + 32 + 32;

/// An unencrypted testnet signing seed and its derived public identity.
pub struct WalletKey {
    seed: [u8; 32],
    public_key: Vec<u8>,
    account_id: AccountId,
}

impl WalletKey {
    /// Generate a fresh seed using the operating system CSPRNG.
    pub fn generate() -> Result<Self, Box<dyn Error>> {
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed)?;
        Ok(Self::from_seed(seed))
    }

    fn from_seed(seed_bytes: [u8; 32]) -> Self {
        let mut seed = ml_dsa::Seed::default();
        seed.copy_from_slice(&seed_bytes);
        let signing_key = SigningKey::<MlDsa65>::from_seed(&seed);
        let public_key = signing_key.verifying_key().encode().as_slice().to_vec();
        let account_id = post_quantum_account_id(&public_key);
        Self {
            seed: seed_bytes,
            public_key,
            account_id,
        }
    }

    /// Restore a wallet key from its 32-byte FIPS 204 key-generation seed.
    pub fn from_seed_bytes(seed: [u8; 32]) -> Self {
        Self::from_seed(seed)
    }

    pub fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    pub(crate) fn signing_key(&self) -> SigningKey<MlDsa65> {
        let mut seed = ml_dsa::Seed::default();
        seed.copy_from_slice(&self.seed);
        SigningKey::<MlDsa65>::from_seed(&seed)
    }

    /// Save a plain testnet seed without replacing an existing file.
    ///
    /// The checksum detects accidental corruption; it does not encrypt the seed or
    /// authenticate it against deliberate modification.
    pub fn save_plain_testnet_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut payload = Vec::with_capacity(KEY_FILE_BYTES);
        payload.extend_from_slice(KEY_MAGIC);
        payload.extend_from_slice(&self.seed);
        let checksum = sha256_plain(&payload);
        payload.extend_from_slice(checksum.as_bytes());

        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(path)?;
        file.write_all(&payload)?;
        file.sync_all()?;
        Ok(())
    }

    /// Read and validate a plain testnet seed file.
    pub fn load_plain_testnet_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        if file.metadata()?.len() != KEY_FILE_BYTES as u64 {
            return Err("invalid AURKEY3 file length".into());
        }
        let mut payload = [0u8; KEY_FILE_BYTES];
        file.read_exact(&mut payload)?;
        if &payload[..8] != KEY_MAGIC {
            return Err("invalid AURKEY3 file header".into());
        }
        let expected = sha256_plain(&payload[..40]);
        if expected.as_bytes() != &payload[40..72] {
            return Err("AURKEY3 checksum mismatch".into());
        }
        let seed: [u8; 32] = payload[8..40].try_into()?;
        Ok(Self::from_seed(seed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, sync::atomic::{AtomicU64, Ordering}};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn key_file_roundtrip_is_stable_and_never_overwrites() {
        let path = std::env::temp_dir().join(format!(
            "aurora-wallet-key-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let key = WalletKey::from_seed_bytes([42; 32]);
        key.save_plain_testnet_file(&path).unwrap();
        let loaded = WalletKey::load_plain_testnet_file(&path).unwrap();
        assert_eq!(loaded.account_id(), key.account_id());
        assert_eq!(loaded.public_key(), key.public_key());
        assert!(loaded.save_plain_testnet_file(&path).is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn key_file_rejects_corruption() {
        let path = std::env::temp_dir().join(format!(
            "aurora-wallet-corrupt-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let key = WalletKey::from_seed_bytes([7; 32]);
        key.save_plain_testnet_file(&path).unwrap();
        let mut bytes = fs::read(&path).unwrap();
        bytes[12] ^= 1;
        fs::write(&path, bytes).unwrap();
        assert!(WalletKey::load_plain_testnet_file(&path).is_err());
        fs::remove_file(path).unwrap();
    }
}

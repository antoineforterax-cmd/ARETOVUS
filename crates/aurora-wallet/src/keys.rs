//! ML-DSA-65 testnet key generation and local key-file handling.
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use argon2::{Algorithm, Argon2, Params, Version};
use aurora_core::{accounts::AccountId, hash::sha256_plain, signature::post_quantum_account_id};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use ml_dsa::{Keypair, MlDsa65, SigningKey};
use zeroize::Zeroize;

const KEY_MAGIC: &[u8; 8] = b"AURKEY3\0";
const KEY_FILE_BYTES: usize = 8 + 32 + 32;
const ENCRYPTED_MAGIC: &[u8; 8] = b"AURKEY4\0";
const ARGON_MEMORY_KIB: u32 = 64 * 1024;
const ARGON_PASSES: u32 = 3;
const ARGON_LANES: u32 = 1;
const ENCRYPTED_HEADER_BYTES: usize = 8 + 12 + 16 + 24;
const ENCRYPTED_FILE_BYTES: usize = ENCRYPTED_HEADER_BYTES + 32 + 16;

/// Testnet signing seed and its derived public identity; the seed is wiped on drop.
pub struct WalletKey {
    seed: [u8; 32],
    public_key: Vec<u8>,
    account_id: AccountId,
}

impl WalletKey {
    /// Generate a fresh seed using the operating system CSPRNG.
    pub fn generate() -> Result<Self, Box<dyn Error>> {
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed).map_err(|error| format!("OS CSPRNG failed: {error}"))?;
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

    /// Whether a key file uses the authenticated encrypted AURKEY4 format.
    pub fn file_is_encrypted(path: &Path) -> Result<bool, Box<dyn Error>> {
        let mut file = File::open(path)?;
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)?;
        if &magic == ENCRYPTED_MAGIC {
            Ok(true)
        } else if &magic == KEY_MAGIC {
            Ok(false)
        } else {
            Err("unknown AURORA key format".into())
        }
    }

    /// Save a password-encrypted testnet seed with a fresh random salt and nonce.
    pub fn save_encrypted_testnet_file(
        &self,
        path: &Path,
        password: &str,
    ) -> Result<(), Box<dyn Error>> {
        if password.len() < 12 {
            return Err("wallet password must contain at least 12 characters".into());
        }
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 24];
        getrandom::getrandom(&mut salt).map_err(|error| format!("OS CSPRNG failed: {error}"))?;
        getrandom::getrandom(&mut nonce).map_err(|error| format!("OS CSPRNG failed: {error}"))?;
        let mut header = Vec::with_capacity(ENCRYPTED_HEADER_BYTES);
        header.extend_from_slice(ENCRYPTED_MAGIC);
        header.extend_from_slice(&ARGON_MEMORY_KIB.to_be_bytes());
        header.extend_from_slice(&ARGON_PASSES.to_be_bytes());
        header.extend_from_slice(&ARGON_LANES.to_be_bytes());
        header.extend_from_slice(&salt);
        header.extend_from_slice(&nonce);

        let mut key = derive_password_key(password, &salt)?;
        let cipher = XChaCha20Poly1305::new_from_slice(&key)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid derived key"))?;
        let nonce_ref = XNonce::try_from(nonce.as_slice())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid nonce"))?;
        let encrypted = cipher.encrypt(
            &nonce_ref,
            Payload {
                msg: &self.seed,
                aad: &header,
            },
        );
        key.zeroize();
        let encrypted = encrypted
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "wallet encryption failed"))?;
        let mut payload = header;
        payload.extend_from_slice(&encrypted);
        if payload.len() != ENCRYPTED_FILE_BYTES {
            return Err("unexpected encrypted key length".into());
        }
        write_new_private_file(path, &payload)?;
        Ok(())
    }

    /// Restore a password-encrypted seed after authenticating all header fields.
    pub fn load_encrypted_testnet_file(
        path: &Path,
        password: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(path)?;
        if file.metadata()?.len() != ENCRYPTED_FILE_BYTES as u64 {
            return Err("invalid AURKEY4 file length".into());
        }
        let mut payload = [0u8; ENCRYPTED_FILE_BYTES];
        file.read_exact(&mut payload)?;
        if &payload[..8] != ENCRYPTED_MAGIC {
            return Err("invalid AURKEY4 file header".into());
        }
        let memory = u32::from_be_bytes(payload[8..12].try_into()?);
        let passes = u32::from_be_bytes(payload[12..16].try_into()?);
        let lanes = u32::from_be_bytes(payload[16..20].try_into()?);
        if (memory, passes, lanes) != (ARGON_MEMORY_KIB, ARGON_PASSES, ARGON_LANES) {
            return Err("unsupported AURKEY4 derivation parameters".into());
        }
        let salt: [u8; 16] = payload[20..36].try_into()?;
        let nonce: [u8; 24] = payload[36..60].try_into()?;
        let mut key = derive_password_key(password, &salt)?;
        let cipher = XChaCha20Poly1305::new_from_slice(&key)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid derived key"))?;
        let nonce_ref = XNonce::try_from(nonce.as_slice())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid nonce"))?;
        let decrypted = cipher.decrypt(
            &nonce_ref,
            Payload {
                msg: &payload[ENCRYPTED_HEADER_BYTES..],
                aad: &payload[..ENCRYPTED_HEADER_BYTES],
            },
        );
        key.zeroize();
        let mut decrypted = decrypted.map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "wrong password or damaged key file",
            )
        })?;
        if decrypted.len() != 32 {
            decrypted.zeroize();
            return Err("invalid decrypted seed length".into());
        }
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&decrypted);
        decrypted.zeroize();
        Ok(Self::from_seed(seed))
    }

    /// Authenticate an encrypted backup and verify its expected account, without writing files.
    pub fn verify_encrypted_backup(
        path: &Path,
        password: &str,
        expected: AccountId,
    ) -> Result<(), Box<dyn Error>> {
        let restored = Self::load_encrypted_testnet_file(path, password)?;
        if restored.account_id() != expected {
            return Err("backup belongs to a different account".into());
        }
        Ok(())
    }

    /// Verify an encrypted key, then create an independent encrypted copy.
    ///
    /// The destination uses a fresh salt and nonce and is never overwritten.
    pub fn copy_encrypted_testnet_file(
        source: &Path,
        destination: &Path,
        password: &str,
    ) -> Result<AccountId, Box<dyn Error>> {
        if !Self::file_is_encrypted(source)? {
            return Err("backup and restore require an encrypted AURKEY4 source".into());
        }
        let key = Self::load_encrypted_testnet_file(source, password)?;
        key.save_encrypted_testnet_file(destination, password)?;
        Ok(key.account_id())
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

        write_new_private_file(path, &payload)?;
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

impl Drop for WalletKey {
    fn drop(&mut self) {
        self.seed.zeroize();
    }
}

fn derive_password_key(password: &str, salt: &[u8; 16]) -> Result<[u8; 32], Box<dyn Error>> {
    let params = Params::new(ARGON_MEMORY_KIB, ARGON_PASSES, ARGON_LANES, Some(32))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon.hash_password_into(password.as_bytes(), salt, &mut key)?;
    Ok(key)
}

fn write_new_private_file(path: &Path, payload: &[u8]) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(payload)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

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
    fn encrypted_key_roundtrip_rejects_wrong_password_and_tampering() {
        let path = std::env::temp_dir().join(format!(
            "aurora-wallet-encrypted-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let key = WalletKey::from_seed_bytes([43; 32]);
        key.save_encrypted_testnet_file(&path, "a long test password")
            .unwrap();
        assert!(WalletKey::file_is_encrypted(&path).unwrap());
        let loaded = WalletKey::load_encrypted_testnet_file(&path, "a long test password").unwrap();
        assert_eq!(loaded.account_id(), key.account_id());
        assert!(WalletKey::load_encrypted_testnet_file(&path, "wrong password").is_err());
        assert!(key
            .save_encrypted_testnet_file(&path, "a long test password")
            .is_err());
        let mut bytes = fs::read(&path).unwrap();
        bytes[ENCRYPTED_HEADER_BYTES] ^= 1;
        fs::write(&path, bytes).unwrap();
        assert!(WalletKey::load_encrypted_testnet_file(&path, "a long test password").is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn encrypted_backup_restores_identity_without_reusing_ciphertext() {
        let unique = NEXT.fetch_add(1, Ordering::Relaxed);
        let source = std::env::temp_dir().join(format!(
            "aurora-wallet-backup-source-{}-{unique}",
            std::process::id()
        ));
        let backup = std::env::temp_dir().join(format!(
            "aurora-wallet-backup-copy-{}-{unique}",
            std::process::id()
        ));
        let restored = std::env::temp_dir().join(format!(
            "aurora-wallet-backup-restored-{}-{unique}",
            std::process::id()
        ));
        let key = WalletKey::from_seed_bytes([44; 32]);
        key.save_encrypted_testnet_file(&source, "another long test password")
            .unwrap();
        WalletKey::verify_encrypted_backup(&source, "another long test password", key.account_id())
            .unwrap();
        assert!(WalletKey::verify_encrypted_backup(
            &source,
            "another long test password",
            AccountId([0; 32])
        )
        .is_err());
        assert!(
            WalletKey::verify_encrypted_backup(&source, "wrong password", key.account_id())
                .is_err()
        );
        let account =
            WalletKey::copy_encrypted_testnet_file(&source, &backup, "another long test password")
                .unwrap();
        assert_eq!(account, key.account_id());
        assert_ne!(fs::read(&source).unwrap(), fs::read(&backup).unwrap());
        assert!(WalletKey::copy_encrypted_testnet_file(
            &source,
            &backup,
            "another long test password"
        )
        .is_err());
        assert!(
            WalletKey::copy_encrypted_testnet_file(&backup, &restored, "wrong password").is_err()
        );
        assert!(!restored.exists());
        WalletKey::copy_encrypted_testnet_file(&backup, &restored, "another long test password")
            .unwrap();
        let loaded =
            WalletKey::load_encrypted_testnet_file(&restored, "another long test password")
                .unwrap();
        assert_eq!(loaded.account_id(), key.account_id());
        for path in [source, backup, restored] {
            fs::remove_file(path).unwrap();
        }
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

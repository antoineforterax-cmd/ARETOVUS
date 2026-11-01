//! ML-DSA-65 testnet key generation and local key-file handling.
use std::{
    error::Error,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use aretovus_core::{accounts::AccountId, hash::sha256_plain, signature::post_quantum_account_id};
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use ml_dsa::{Keypair, MlDsa65, SigningKey};
use zeroize::{Zeroize, Zeroizing};

const KEY_MAGIC: &[u8; 8] = b"ARETOVK3";
const KEY_FILE_BYTES: usize = 8 + 32 + 32;
const ENCRYPTED_MAGIC: &[u8; 8] = b"ARETOVK4";
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
        let mut seed = Zeroizing::new([0u8; 32]);
        getrandom::getrandom(&mut *seed).map_err(|error| format!("OS CSPRNG failed: {error}"))?;
        Ok(Self::from_seed(*seed))
    }

    fn from_seed(mut seed_bytes: [u8; 32]) -> Self {
        let mut seed = ml_dsa::Seed::default();
        seed.copy_from_slice(&seed_bytes);
        let signing_key = SigningKey::<MlDsa65>::from_seed(&seed);
        for byte in seed.iter_mut() {
            byte.zeroize();
        }
        let public_key = signing_key.verifying_key().encode().as_slice().to_vec();
        let account_id = post_quantum_account_id(&public_key);
        let key = Self {
            seed: seed_bytes,
            public_key,
            account_id,
        };
        seed_bytes.zeroize();
        key
    }

    /// Restore a wallet key from its 32-byte FIPS 204 key-generation seed.
    pub fn from_seed_bytes(mut seed: [u8; 32]) -> Self {
        let key = Self::from_seed(seed);
        seed.zeroize();
        key
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
        let key = SigningKey::<MlDsa65>::from_seed(&seed);
        for byte in seed.iter_mut() {
            byte.zeroize();
        }
        key
    }

    /// Whether a key file uses the authenticated encrypted ARETOVK4 format.
    pub fn file_is_encrypted(path: &Path) -> Result<bool, Box<dyn Error>> {
        let mut file = open_regular_wallet_file(path)?;
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)?;
        if &magic == ENCRYPTED_MAGIC {
            Ok(true)
        } else if &magic == KEY_MAGIC {
            Ok(false)
        } else {
            Err("unknown ARETOVUS key format".into())
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

        let mut key = Zeroizing::new(derive_password_key(password, &salt)?);
        let cipher = XChaCha20Poly1305::new_from_slice(&*key)
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
        let mut file = open_regular_wallet_file(path)?;
        if file.metadata()?.len() != ENCRYPTED_FILE_BYTES as u64 {
            return Err("invalid ARETOVK4 file length".into());
        }
        let mut payload = [0u8; ENCRYPTED_FILE_BYTES];
        file.read_exact(&mut payload)?;
        if &payload[..8] != ENCRYPTED_MAGIC {
            return Err("invalid ARETOVK4 file header".into());
        }
        let memory = u32::from_be_bytes(payload[8..12].try_into()?);
        let passes = u32::from_be_bytes(payload[12..16].try_into()?);
        let lanes = u32::from_be_bytes(payload[16..20].try_into()?);
        if (memory, passes, lanes) != (ARGON_MEMORY_KIB, ARGON_PASSES, ARGON_LANES) {
            return Err("unsupported ARETOVK4 derivation parameters".into());
        }
        let salt: [u8; 16] = payload[20..36].try_into()?;
        let nonce: [u8; 24] = payload[36..60].try_into()?;
        let mut key = Zeroizing::new(derive_password_key(password, &salt)?);
        let cipher = XChaCha20Poly1305::new_from_slice(&*key)
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
        let mut seed = Zeroizing::new([0u8; 32]);
        seed.copy_from_slice(&decrypted);
        decrypted.zeroize();
        Ok(Self::from_seed(*seed))
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
            return Err("backup and restore require an encrypted ARETOVK4 source".into());
        }
        let key = Self::load_encrypted_testnet_file(source, password)?;
        key.save_encrypted_testnet_file(destination, password)?;
        Ok(key.account_id())
    }

    /// Export an unencrypted recovery secret. Anyone holding it can spend this account.
    pub fn save_recovery_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut code = crate::recovery::encode_seed(&self.seed);
        code.push('\n');
        write_new_private_file(path, code.as_bytes())?;
        Ok(())
    }

    /// Recover the same v3 identity, without the original file or its password.
    pub fn from_recovery_code(code: &str, expected: AccountId) -> Result<Self, Box<dyn Error>> {
        let seed = crate::recovery::decode_seed(code)?;
        let key = Self::from_seed(*seed);
        if key.account_id() != expected {
            return Err("recovery code belongs to a different account".into());
        }
        Ok(key)
    }

    /// Read only a bounded recovery text file and validate its expected identity.
    pub fn load_recovery_file(path: &Path, expected: AccountId) -> Result<Self, Box<dyn Error>> {
        let file = open_regular_wallet_file(path)?;
        // Reserve beyond the read limit so secret bytes never move through reallocation.
        let mut code = Zeroizing::new(Vec::with_capacity(258));
        file.take(257).read_to_end(&mut code)?;
        if code.len() > 256 {
            return Err("recovery file exceeds size limit".into());
        }
        Self::from_recovery_code(std::str::from_utf8(&code)?, expected)
    }

    /// Save a plain testnet seed without replacing an existing file.
    ///
    /// The checksum detects accidental corruption; it does not encrypt the seed or
    /// authenticate it against deliberate modification.
    pub fn save_plain_testnet_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut payload = Zeroizing::new(Vec::with_capacity(KEY_FILE_BYTES));
        payload.extend_from_slice(KEY_MAGIC);
        payload.extend_from_slice(&self.seed);
        let checksum = sha256_plain(&payload);
        payload.extend_from_slice(checksum.as_bytes());

        write_new_private_file(path, &payload)?;
        Ok(())
    }

    /// Read and validate a plain testnet seed file.
    pub fn load_plain_testnet_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut file = open_regular_wallet_file(path)?;
        if file.metadata()?.len() != KEY_FILE_BYTES as u64 {
            return Err("invalid ARETOVK3 file length".into());
        }
        let mut payload = Zeroizing::new([0u8; KEY_FILE_BYTES]);
        file.read_exact(&mut *payload)?;
        if &payload[..8] != KEY_MAGIC {
            return Err("invalid ARETOVK3 file header".into());
        }
        let expected = sha256_plain(&payload[..40]);
        if expected.as_bytes() != &payload[40..72] {
            return Err("ARETOVK3 checksum mismatch".into());
        }
        let seed = Zeroizing::new(<[u8; 32]>::try_from(&payload[8..40])?);
        Ok(Self::from_seed(*seed))
    }
}

impl Drop for WalletKey {
    fn drop(&mut self) {
        self.seed.zeroize();
    }
}

/// Reject symlinks, directories and device files before reading secret key material.
/// On Unix, compare the opened file identity to the lstat result to reject a
/// final-component replacement between those operations.
pub(crate) fn open_regular_wallet_file(path: &Path) -> io::Result<File> {
    let expected = std::fs::symlink_metadata(path)?;
    if !expected.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "wallet secret must be a regular file, not a symlink",
        ));
    }
    open_verified_wallet_file(path, &expected)
}

/// Open a prechecked path without following a final-component symlink.
/// The Unix O_NOFOLLOW flag closes the preflight/open race, even if an
/// attacker substitutes a symlink pointing to the *same inode*.
/// Keep the original inode check for regular-file replacement.
fn open_verified_wallet_file(path: &Path, expected: &std::fs::Metadata) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Ask Windows to open the reparse point itself, not its target.
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        options.custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options.open(path)?;
    let opened = file.metadata()?;
    if !opened.file_type().is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "wallet secret is not a regular file",
        ));
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
        if opened.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "wallet secret must not be a reparse point",
            ));
        }
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if opened.dev() != expected.dev() || opened.ino() != expected.ino() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "wallet key path was replaced while opening",
            ));
        }
    }
    Ok(file)
}

fn derive_password_key(password: &str, salt: &[u8; 16]) -> Result<[u8; 32], Box<dyn Error>> {
    let params = Params::new(ARGON_MEMORY_KIB, ARGON_PASSES, ARGON_LANES, Some(32))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = Zeroizing::new([0u8; 32]);
    argon.hash_password_into(password.as_bytes(), salt, &mut *key)?;
    Ok(*key)
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
    fn historical_key_magic_is_rejected_for_plain_and_encrypted_files() {
        let root = std::env::temp_dir().join(format!(
            "aretovus-wallet-foreign-magic-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let key = WalletKey::from_seed_bytes([0x5c; 32]);
        let plain = root.join("plain.aretkey");
        let encrypted = root.join("encrypted.aretkey");
        let password = "test-only native wallet password";

        key.save_plain_testnet_file(&plain).unwrap();
        key.save_encrypted_testnet_file(&encrypted, password)
            .unwrap();
        assert!(!WalletKey::file_is_encrypted(&plain).unwrap());
        assert!(WalletKey::file_is_encrypted(&encrypted).unwrap());

        let native_plain = fs::read(&plain).unwrap();
        let native_encrypted = fs::read(&encrypted).unwrap();
        let mut old_plain = native_plain.clone();
        let mut old_encrypted = native_encrypted.clone();
        old_plain[..8].copy_from_slice(b"AURKEY3\0");
        old_encrypted[..8].copy_from_slice(b"AURKEY4\0");
        fs::write(&plain, &old_plain).unwrap();
        fs::write(&encrypted, &old_encrypted).unwrap();

        assert!(WalletKey::file_is_encrypted(&plain).is_err());
        assert!(WalletKey::load_plain_testnet_file(&plain).is_err());
        assert!(WalletKey::file_is_encrypted(&encrypted).is_err());
        assert!(WalletKey::load_encrypted_testnet_file(&encrypted, password).is_err());
        assert_eq!(fs::read(&plain).unwrap(), old_plain);
        assert_eq!(fs::read(&encrypted).unwrap(), old_encrypted);

        // Reverting only the two test fixtures re-enables native reads; no
        // migration or write to the rejected original files occurs.
        fs::write(&plain, native_plain).unwrap();
        fs::write(&encrypted, native_encrypted).unwrap();
        assert_eq!(
            WalletKey::load_plain_testnet_file(&plain)
                .unwrap()
                .account_id(),
            key.account_id()
        );
        assert_eq!(
            WalletKey::load_encrypted_testnet_file(&encrypted, password)
                .unwrap()
                .account_id(),
            key.account_id()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn key_file_roundtrip_is_stable_and_never_overwrites() {
        let path = std::env::temp_dir().join(format!(
            "aretovus-wallet-key-{}-{}",
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
            "aretovus-wallet-encrypted-{}-{}",
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
            "aretovus-wallet-backup-source-{}-{unique}",
            std::process::id()
        ));
        let backup = std::env::temp_dir().join(format!(
            "aretovus-wallet-backup-copy-{}-{unique}",
            std::process::id()
        ));
        let restored = std::env::temp_dir().join(format!(
            "aretovus-wallet-backup-restored-{}-{unique}",
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
    fn recovery_survives_original_file_and_password_loss_and_signs() {
        let base = std::env::temp_dir().join(format!(
            "aretovus-recovery-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&base).unwrap();
        let original = base.join("original.aretkey");
        let secret = base.join("recovery.txt");
        let replacement = base.join("replacement.aretkey");
        let key = WalletKey::from_seed_bytes([73; 32]);
        let expected = key.account_id();
        key.save_encrypted_testnet_file(&original, "old password forgotten later")
            .unwrap();
        key.save_recovery_file(&secret).unwrap();
        assert!(key.save_recovery_file(&secret).is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&secret).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        drop(key);
        fs::remove_file(&original).unwrap();
        assert!(!original.exists());
        assert!(WalletKey::load_recovery_file(&secret, AccountId([0; 32])).is_err());
        let recovered = WalletKey::load_recovery_file(&secret, expected).unwrap();
        recovered
            .save_encrypted_testnet_file(&replacement, "entirely new password")
            .unwrap();
        drop(recovered);
        let restored =
            WalletKey::load_encrypted_testnet_file(&replacement, "entirely new password").unwrap();
        assert_eq!(restored.account_id(), expected);
        let tx = crate::build_signed_transaction(
            &restored,
            AccountId([25; 32]),
            aretovus_core::amount::Amount::from_units(100).unwrap(),
            aretovus_core::amount::Amount::from_units(3).unwrap(),
            7,
        )
        .unwrap();
        aretovus_core::signature::verify_account_transaction(&tx).unwrap();
        assert!(restored
            .save_encrypted_testnet_file(&replacement, "entirely new password")
            .is_err());
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn recovery_file_rejects_corruption_oversize_and_foreign_identity() {
        let path = std::env::temp_dir().join(format!(
            "aretovus-recovery-bad-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let key = WalletKey::from_seed_bytes([74; 32]);
        key.save_recovery_file(&path).unwrap();
        let mut bytes = Zeroizing::new(fs::read(&path).unwrap());
        bytes[40] = if bytes[40] == b'0' { b'1' } else { b'0' };
        fs::write(&path, &*bytes).unwrap();
        assert!(WalletKey::load_recovery_file(&path, key.account_id()).is_err());
        fs::write(&path, [b'x'; 257]).unwrap();
        assert!(WalletKey::load_recovery_file(&path, key.account_id()).is_err());
        fs::write(&path, [0xff; 155]).unwrap();
        assert!(WalletKey::load_recovery_file(&path, key.account_id()).is_err());
        fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn wallet_secret_readers_reject_symlinks_but_accept_original_files() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "aretovus-wallet-links-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let key = WalletKey::from_seed_bytes([0x68; 32]);
        let encrypted = root.join("encrypted.aretkey");
        let plain = root.join("plain.aretkey");
        let recovery = root.join("recovery.txt");
        let password = "test-only long wallet password";
        key.save_encrypted_testnet_file(&encrypted, password)
            .unwrap();
        key.save_plain_testnet_file(&plain).unwrap();
        key.save_recovery_file(&recovery).unwrap();

        let encrypted_link = root.join("encrypted-link.aretkey");
        let plain_link = root.join("plain-link.aretkey");
        let recovery_link = root.join("recovery-link.txt");
        symlink(&encrypted, &encrypted_link).unwrap();
        symlink(&plain, &plain_link).unwrap();
        symlink(&recovery, &recovery_link).unwrap();

        assert!(WalletKey::file_is_encrypted(&encrypted_link).is_err());
        assert!(WalletKey::file_is_encrypted(&plain_link).is_err());
        assert!(WalletKey::load_encrypted_testnet_file(&encrypted_link, password).is_err());
        assert!(WalletKey::load_plain_testnet_file(&plain_link).is_err());
        assert!(WalletKey::load_recovery_file(&recovery_link, key.account_id()).is_err());

        assert!(WalletKey::file_is_encrypted(&encrypted).unwrap());
        assert!(!WalletKey::file_is_encrypted(&plain).unwrap());
        assert_eq!(
            WalletKey::load_encrypted_testnet_file(&encrypted, password)
                .unwrap()
                .account_id(),
            key.account_id()
        );
        assert_eq!(
            WalletKey::load_plain_testnet_file(&plain)
                .unwrap()
                .account_id(),
            key.account_id()
        );
        assert_eq!(
            WalletKey::load_recovery_file(&recovery, key.account_id())
                .unwrap()
                .account_id(),
            key.account_id()
        );

        assert!(WalletKey::file_is_encrypted(&root).is_err());
        assert!(WalletKey::load_encrypted_testnet_file(&root, password).is_err());
        assert!(WalletKey::load_plain_testnet_file(&root).is_err());
        assert!(WalletKey::load_recovery_file(&root, key.account_id()).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn wallet_file_swap_to_symlink_of_same_inode_fails_after_preflight() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "aretovus-wallet-swap-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let requested = root.join("wallet.aretkey");
        let moved = root.join("original.aretkey");
        fs::write(&requested, b"same-inode symlink race test").unwrap();
        // Simulate the precise point where a hostile local actor replaces
        // the checked path before the open syscall. The target inode is the
        // *same* as before, defeating a post-open dev/ino comparison alone.
        let before = fs::symlink_metadata(&requested).unwrap();
        fs::rename(&requested, &moved).unwrap();
        symlink(&moved, &requested).unwrap();
        assert!(open_verified_wallet_file(&requested, &before).is_err());
        assert!(open_regular_wallet_file(&moved).is_ok());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn key_file_rejects_corruption() {
        let path = std::env::temp_dir().join(format!(
            "aretovus-wallet-corrupt-{}-{}",
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

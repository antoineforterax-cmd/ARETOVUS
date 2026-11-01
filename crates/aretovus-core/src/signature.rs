//! Vérification des signatures des transactions par comptes.
//!
//! Testnet utilise le profil v3 natif ML-DSA-65 : l'identifiant de compte est
//! dérivé de la clé publique, et l'autorisation contient clé et signature.
//! Regtest conserve v1 Ed25519 et v2 hybride pour les tests de compatibilité.

use ed25519_dalek::{Signature, VerifyingKey as VerifyingKeyEd25519};

use crate::accounts::{
    AccountTransaction, HYBRID_ACCOUNT_TX_VERSION, HYBRID_AUTH_BYTES, ML_DSA_65_PUBLIC_KEY_BYTES,
    ML_DSA_65_SIGNATURE_BYTES, POST_QUANTUM_ACCOUNT_TX_VERSION, POST_QUANTUM_AUTH_BYTES,
};
use crate::hash::{tagged_hash, Domain};
use crate::network::NetworkId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureError {
    InvalidPublicKey,
    InvalidSignatureEncoding,
    InvalidSignature,
    InvalidHybridAccountId,
    InvalidPostQuantumAccountId,
    /// Legacy Ed25519/hybrid profiles are never valid on native ARETOVUS Testnet.
    UnsupportedTestnetTransactionProfile,
}

// This is defense in depth: consensus already rejects v1/v2 for Testnet.
// Check before invoking legacy crypto or consulting any signature cache.
fn require_native_testnet_profile(tx: &AccountTransaction) -> Result<(), SignatureError> {
    if tx.network == NetworkId::Testnet && tx.version != POST_QUANTUM_ACCOUNT_TX_VERSION {
        return Err(SignatureError::UnsupportedTestnetTransactionProfile);
    }
    Ok(())
}

/// Valide les octets de signature d'une transaction contre la clé publique `sender`.
pub fn verify_account_transaction(tx: &AccountTransaction) -> Result<(), SignatureError> {
    require_native_testnet_profile(tx)?;
    if tx.version == POST_QUANTUM_ACCOUNT_TX_VERSION {
        return verify_post_quantum_account_transaction(tx);
    }
    if tx.version == HYBRID_ACCOUNT_TX_VERSION {
        return verify_hybrid_account_transaction(tx);
    }
    let key = VerifyingKeyEd25519::from_bytes(&tx.sender.0)
        .map_err(|_| SignatureError::InvalidPublicKey)?;
    let signature = Signature::from_slice(&tx.auth_data)
        .map_err(|_| SignatureError::InvalidSignatureEncoding)?;
    let digest = tx
        .signing_digest()
        .map_err(|_| SignatureError::InvalidSignature)?;
    key.verify_strict(digest.as_bytes(), &signature)
        .map_err(|_| SignatureError::InvalidSignature)
}

/// Account id for v2 commits to both public keys, preventing key substitution.
pub fn hybrid_account_id(
    ed25519_public_key: &[u8; 32],
    ml_dsa_65_public_key: &[u8],
) -> crate::accounts::AccountId {
    let mut keys = Vec::with_capacity(32 + ml_dsa_65_public_key.len());
    keys.extend_from_slice(ed25519_public_key);
    keys.extend_from_slice(ml_dsa_65_public_key);
    crate::accounts::AccountId(*tagged_hash(Domain::HybridAccountId, &keys).as_bytes())
}

/// Account id for v3 is a domain-separated hash of the encoded ML-DSA-65 public key.
pub fn post_quantum_account_id(ml_dsa_65_public_key: &[u8]) -> crate::accounts::AccountId {
    crate::accounts::AccountId(
        *tagged_hash(Domain::PostQuantumAccountId, ml_dsa_65_public_key).as_bytes(),
    )
}

fn verify_post_quantum_account_transaction(tx: &AccountTransaction) -> Result<(), SignatureError> {
    use ml_dsa::{MlDsa65, Signature as MlDsaSignature, Verifier, VerifyingKey};

    if tx.auth_data.len() != POST_QUANTUM_AUTH_BYTES {
        return Err(SignatureError::InvalidSignatureEncoding);
    }
    let key_end = ML_DSA_65_PUBLIC_KEY_BYTES;
    let mut encoded_key = ml_dsa::EncodedVerifyingKey::<MlDsa65>::default();
    encoded_key.copy_from_slice(&tx.auth_data[..key_end]);
    let key = VerifyingKey::<MlDsa65>::decode(&encoded_key);
    if post_quantum_account_id(&tx.auth_data[..key_end]) != tx.sender {
        return Err(SignatureError::InvalidPostQuantumAccountId);
    }
    let mut encoded_sig = ml_dsa::EncodedSignature::<MlDsa65>::default();
    encoded_sig.copy_from_slice(&tx.auth_data[key_end..]);
    let signature = MlDsaSignature::<MlDsa65>::decode(&encoded_sig)
        .ok_or(SignatureError::InvalidSignatureEncoding)?;
    let digest = tx
        .signing_digest()
        .map_err(|_| SignatureError::InvalidSignature)?;
    key.verify(digest.as_bytes(), &signature)
        .map_err(|_| SignatureError::InvalidSignature)
}

fn verify_hybrid_account_transaction(tx: &AccountTransaction) -> Result<(), SignatureError> {
    use ml_dsa::{MlDsa65, Signature as MlDsaSignature, Verifier, VerifyingKey};

    if tx.auth_data.len() != HYBRID_AUTH_BYTES {
        return Err(SignatureError::InvalidSignatureEncoding);
    }
    let ed_key_bytes: [u8; 32] = tx.auth_data[..32]
        .try_into()
        .map_err(|_| SignatureError::InvalidSignatureEncoding)?;
    let ml_key_start = 32;
    let ml_key_end = ml_key_start + ML_DSA_65_PUBLIC_KEY_BYTES;
    let ed_sig_end = ml_key_end + 64;
    let ml_sig_end = ed_sig_end + ML_DSA_65_SIGNATURE_BYTES;

    let mut encoded_key = ml_dsa::EncodedVerifyingKey::<MlDsa65>::default();
    encoded_key.copy_from_slice(&tx.auth_data[ml_key_start..ml_key_end]);
    let ml_key = VerifyingKey::<MlDsa65>::decode(&encoded_key);
    if hybrid_account_id(&ed_key_bytes, &tx.auth_data[ml_key_start..ml_key_end]) != tx.sender {
        return Err(SignatureError::InvalidHybridAccountId);
    }

    let digest = tx
        .signing_digest()
        .map_err(|_| SignatureError::InvalidSignature)?;
    let ed_key = VerifyingKeyEd25519::from_bytes(&ed_key_bytes)
        .map_err(|_| SignatureError::InvalidPublicKey)?;
    let ed_sig = Signature::from_slice(&tx.auth_data[ml_key_end..ed_sig_end])
        .map_err(|_| SignatureError::InvalidSignatureEncoding)?;
    ed_key
        .verify_strict(digest.as_bytes(), &ed_sig)
        .map_err(|_| SignatureError::InvalidSignature)?;

    let mut encoded_sig = ml_dsa::EncodedSignature::<MlDsa65>::default();
    encoded_sig.copy_from_slice(&tx.auth_data[ed_sig_end..ml_sig_end]);
    let ml_sig = MlDsaSignature::<MlDsa65>::decode(&encoded_sig)
        .ok_or(SignatureError::InvalidSignatureEncoding)?;
    ml_key
        .verify(digest.as_bytes(), &ml_sig)
        .map_err(|_| SignatureError::InvalidSignature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{AccountId, AccountTransaction, ACCOUNT_TX_VERSION};
    use crate::amount::Amount;
    use crate::network::NetworkId;
    use ed25519_dalek::{Signer, SigningKey};

    fn signed_transaction() -> (SigningKey, AccountTransaction) {
        let key = SigningKey::from_bytes(&[7; 32]);
        let mut tx = AccountTransaction {
            version: ACCOUNT_TX_VERSION,
            network: NetworkId::Regtest,
            sender: AccountId(key.verifying_key().to_bytes()),
            nonce: 3,
            recipient: AccountId([8; 32]),
            amount: Amount::from_units(25).unwrap(),
            fee: Amount::from_units(2).unwrap(),
            auth_data: Vec::new(),
        };
        tx.auth_data = key
            .sign(tx.signing_digest().unwrap().as_bytes())
            .to_bytes()
            .to_vec();
        (key, tx)
    }

    #[test]
    fn obsolete_ed25519_transaction_cannot_be_reused_on_native_testnet() {
        let (key, mut tx) = signed_transaction();
        // Ensure this is a *validly signed* v1 transaction on Regtest,
        // then sign again for Testnet to prove that the version gate,
        // rather than a changed digest, refuses the obsolete profile.
        assert_eq!(verify_account_transaction(&tx), Ok(()));
        tx.network = NetworkId::Testnet;
        tx.auth_data = key
            .sign(tx.signing_digest().unwrap().as_bytes())
            .to_bytes()
            .to_vec();
        assert_eq!(
            verify_account_transaction(&tx),
            Err(SignatureError::UnsupportedTestnetTransactionProfile)
        );
        let mut cached = SignatureCache::new(4);
        assert_eq!(
            cached.verify(&tx),
            Err(SignatureError::UnsupportedTestnetTransactionProfile)
        );
        assert_eq!(
            verify_account_transaction_cached(&tx),
            Err(SignatureError::UnsupportedTestnetTransactionProfile)
        );
    }

    #[test]
    fn hybrid_v2_requires_both_ed25519_and_ml_dsa_65_signatures() {
        use crate::accounts::{HYBRID_ACCOUNT_TX_VERSION, HYBRID_AUTH_BYTES};
        use ml_dsa::{Keypair, MlDsa65, Signer as MlSigner, SigningKey as MlSigningKey};

        let ed = SigningKey::from_bytes(&[31; 32]);
        let ml_seed = ml_dsa::Seed::default();
        let ml = MlSigningKey::<MlDsa65>::from_seed(&ml_seed);
        let ml_public = ml.verifying_key().encode();
        let sender = hybrid_account_id(&ed.verifying_key().to_bytes(), ml_public.as_slice());
        let mut tx = AccountTransaction {
            version: HYBRID_ACCOUNT_TX_VERSION,
            network: NetworkId::Regtest,
            sender,
            nonce: 1,
            recipient: AccountId([44; 32]),
            amount: Amount::from_units(4).unwrap(),
            fee: Amount::from_units(1).unwrap(),
            auth_data: vec![0; HYBRID_AUTH_BYTES],
        };
        tx.auth_data[..32].copy_from_slice(&ed.verifying_key().to_bytes());
        tx.auth_data[32..32 + ml_public.len()].copy_from_slice(ml_public.as_slice());
        let digest = tx.signing_digest().unwrap();
        let ed_sig = ed.sign(digest.as_bytes()).to_bytes();
        let ml_sig = ml.sign(digest.as_bytes()).encode();
        let ed_sig_start = 32 + ml_public.len();
        tx.auth_data[ed_sig_start..ed_sig_start + ed_sig.len()].copy_from_slice(&ed_sig);
        tx.auth_data[ed_sig_start + ed_sig.len()..].copy_from_slice(ml_sig.as_slice());
        assert_eq!(verify_account_transaction(&tx), Ok(()));

        // Valid v2 hybrid signatures MUST NOT authorize a native Testnet
        // transaction even if both Ed25519 and ML-DSA-65 signatures verify.
        let regtest_tx = tx.clone();
        tx.network = NetworkId::Testnet;
        let digest = tx.signing_digest().unwrap();
        let ed_sig = ed.sign(digest.as_bytes()).to_bytes();
        let ml_sig = ml.sign(digest.as_bytes()).encode();
        tx.auth_data[ed_sig_start..ed_sig_start + ed_sig.len()].copy_from_slice(&ed_sig);
        tx.auth_data[ed_sig_start + ed_sig.len()..].copy_from_slice(ml_sig.as_slice());
        assert_eq!(
            verify_account_transaction(&tx),
            Err(SignatureError::UnsupportedTestnetTransactionProfile)
        );
        tx = regtest_tx;
        tx.amount = Amount::from_units(5).unwrap();
        assert_eq!(
            verify_account_transaction(&tx),
            Err(SignatureError::InvalidSignature)
        );
    }

    #[test]
    fn old_network_domain_signature_is_rejected_for_native_mldsa_testnet() {
        use crate::codec::{CanonicalEncode, Encoder};
        use ml_dsa::{Keypair, MlDsa65, Signer as MlSigner, SigningKey as MlSigningKey};
        use sha2::{Digest, Sha256};

        let signing_key = MlSigningKey::<MlDsa65>::from_seed(&ml_dsa::Seed::default());
        let public_key = signing_key.verifying_key().encode();
        let mut tx = AccountTransaction {
            version: POST_QUANTUM_ACCOUNT_TX_VERSION,
            network: NetworkId::Testnet,
            sender: post_quantum_account_id(public_key.as_slice()),
            nonce: 4,
            recipient: AccountId([0x45; 32]),
            amount: Amount::from_units(11).unwrap(),
            fee: Amount::from_units(1).unwrap(),
            auth_data: vec![0; POST_QUANTUM_AUTH_BYTES],
        };
        tx.auth_data[..ML_DSA_65_PUBLIC_KEY_BYTES].copy_from_slice(public_key.as_slice());

        // Reconstruct the old signing-domain digest using a byte-only
        // historical fixture. The active source never accepts that domain.
        // This is the domain used before the ARETOVUS network separation.
        let old_tag: &[u8] = &[ 0x41, 0x55, 0x52, 0x4f, 0x52, 0x41, 0x2f, 0x65, 0x78, 0x70, 0x2f, 0x76, 0x30, 0x2f, 0x74, 0x78, 0x2d, 0x73, 0x69, 0x67, 0x6e, 0x69, 0x6e, 0x67 ];
        let mut enc = Encoder::new();
        enc.put_u32(tx.version);
        tx.network.encode(&mut enc).unwrap();
        tx.sender.encode(&mut enc).unwrap();
        enc.put_u64(tx.nonce);
        tx.recipient.encode(&mut enc).unwrap();
        tx.amount.encode(&mut enc).unwrap();
        tx.fee.encode(&mut enc).unwrap();
        let payload = enc.into_bytes();
        let mut sha = Sha256::new();
        sha.update([old_tag.len() as u8]);
        sha.update(old_tag);
        sha.update(payload);
        let old_digest = sha.finalize();

        let old_signature = signing_key.sign(old_digest.as_slice()).encode();
        tx.auth_data[ML_DSA_65_PUBLIC_KEY_BYTES..].copy_from_slice(old_signature.as_slice());
        assert_eq!(
            verify_account_transaction(&tx),
            Err(SignatureError::InvalidSignature)
        );

        // A correctly signed native transaction with exactly the same
        // fields and key remains valid; only the network domain changes.
        let native_signature = signing_key
            .sign(tx.signing_digest().unwrap().as_bytes())
            .encode();
        tx.auth_data[ML_DSA_65_PUBLIC_KEY_BYTES..].copy_from_slice(native_signature.as_slice());
        assert_eq!(verify_account_transaction(&tx), Ok(()));
    }

    #[test]
    fn native_post_quantum_v3_uses_only_ml_dsa_65() {
        use crate::accounts::{POST_QUANTUM_ACCOUNT_TX_VERSION, POST_QUANTUM_AUTH_BYTES};
        use ml_dsa::{Keypair, MlDsa65, Signer as MlSigner, SigningKey as MlSigningKey};

        let signing_key = MlSigningKey::<MlDsa65>::from_seed(&ml_dsa::Seed::default());
        let public_key = signing_key.verifying_key().encode();
        let mut tx = AccountTransaction {
            version: POST_QUANTUM_ACCOUNT_TX_VERSION,
            network: NetworkId::Testnet,
            sender: post_quantum_account_id(public_key.as_slice()),
            nonce: 0,
            recipient: AccountId([0x33; 32]),
            amount: Amount::from_units(5).unwrap(),
            fee: Amount::from_units(1).unwrap(),
            auth_data: vec![0; POST_QUANTUM_AUTH_BYTES],
        };
        tx.auth_data[..ML_DSA_65_PUBLIC_KEY_BYTES].copy_from_slice(public_key.as_slice());
        let signature = signing_key
            .sign(tx.signing_digest().unwrap().as_bytes())
            .encode();
        tx.auth_data[ML_DSA_65_PUBLIC_KEY_BYTES..].copy_from_slice(signature.as_slice());
        assert_eq!(verify_account_transaction(&tx), Ok(()));

        tx.amount = Amount::from_units(6).unwrap();
        assert_eq!(
            verify_account_transaction(&tx),
            Err(SignatureError::InvalidSignature)
        );
    }

    #[test]
    fn accepts_signature_for_canonical_signing_digest() {
        let (_, tx) = signed_transaction();
        assert_eq!(verify_account_transaction(&tx), Ok(()));
    }

    #[test]
    fn rejects_modified_transaction_and_malformed_signature() {
        let (_, mut tx) = signed_transaction();
        tx.amount = Amount::from_units(26).unwrap();
        assert_eq!(
            verify_account_transaction(&tx),
            Err(SignatureError::InvalidSignature)
        );
        tx.auth_data.pop();
        assert_eq!(
            verify_account_transaction(&tx),
            Err(SignatureError::InvalidSignatureEncoding)
        );
    }

    #[test]
    fn signature_is_bound_to_the_sender_public_key() {
        let (_, mut tx) = signed_transaction();
        tx.sender = AccountId(SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes());
        assert_eq!(
            verify_account_transaction(&tx),
            Err(SignatureError::InvalidSignature)
        );
    }
}

/// Successful cryptographic checks only; FIFO eviction bounds retained txids.
pub struct SignatureCache {
    capacity: usize,
    entries: std::collections::HashSet<crate::hash::Hash256>,
    order: std::collections::VecDeque<crate::hash::Hash256>,
}

impl SignatureCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: std::collections::HashSet::new(),
            order: std::collections::VecDeque::new(),
        }
    }

    pub fn verify(&mut self, tx: &AccountTransaction) -> Result<(), SignatureError> {
        require_native_testnet_profile(tx)?;
        let id = tx.txid().map_err(|_| SignatureError::InvalidSignature)?;
        if self.entries.contains(&id) {
            return Ok(());
        }
        verify_account_transaction(tx)?;
        self.remember(id);
        Ok(())
    }

    fn remember(&mut self, id: crate::hash::Hash256) {
        if self.capacity == 0 || self.entries.contains(&id) {
            return;
        }
        if self.entries.len() == self.capacity {
            let oldest = self.order.pop_front().expect("cache order matches entries");
            self.entries.remove(&oldest);
        }
        self.entries.insert(id);
        self.order.push_back(id);
    }
}

/// Process-local cache used only by mempool state transitions.
/// Locks are released during cryptography; concurrent misses may duplicate work.
pub fn verify_account_transaction_cached(tx: &AccountTransaction) -> Result<(), SignatureError> {
    require_native_testnet_profile(tx)?;
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<SignatureCache>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(SignatureCache::new(4096)));
    let id = tx.txid().map_err(|_| SignatureError::InvalidSignature)?;
    if let Ok(guard) = cache.lock() {
        if guard.entries.contains(&id) {
            return Ok(());
        }
    }
    verify_account_transaction(tx)?;
    if let Ok(mut guard) = cache.lock() {
        guard.remember(id);
    }
    Ok(())
}

#[cfg(test)]
mod cache_bound_tests {
    use super::*;

    #[test]
    fn fifo_capacity_and_duplicates_remain_bounded() {
        let mut cache = SignatureCache::new(2);
        let ids: Vec<_> = (0..3).map(|n| crate::hash::sha256_plain(&[n])).collect();
        cache.remember(ids[0]);
        cache.remember(ids[1]);
        cache.remember(ids[1]);
        assert_eq!(cache.order.len(), 2);
        cache.remember(ids[2]);
        assert_eq!(cache.entries.len(), 2);
        assert!(!cache.entries.contains(&ids[0]));
        assert!(cache.entries.contains(&ids[1]));
        assert!(cache.entries.contains(&ids[2]));
    }
}

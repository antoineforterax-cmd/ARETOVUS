//! Vérification Ed25519 des transferts expérimentaux par comptes.
//!
//! L'identifiant de compte est la clé publique Ed25519 brute (32 octets) et `auth_data`
//! contient exactement une signature Ed25519 (64 octets) du `signing_digest` canonique.
//! Ce choix est propre au prototype v1 et ne constitue pas un format d'adresse utilisateur.

use ed25519_dalek::{Signature, VerifyingKey as VerifyingKeyEd25519};

use crate::accounts::{
    AccountTransaction, HYBRID_ACCOUNT_TX_VERSION, HYBRID_AUTH_BYTES, ML_DSA_65_PUBLIC_KEY_BYTES,
    ML_DSA_65_SIGNATURE_BYTES,
};
use crate::hash::{tagged_hash, Domain};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureError {
    InvalidPublicKey,
    InvalidSignatureEncoding,
    InvalidSignature,
    InvalidHybridAccountId,
}

/// Valide les octets de signature d'une transaction contre la clé publique `sender`.
pub fn verify_account_transaction(tx: &AccountTransaction) -> Result<(), SignatureError> {
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
    fn hybrid_v2_requires_both_ed25519_and_ml_dsa_65_signatures() {
        use crate::accounts::{HYBRID_ACCOUNT_TX_VERSION, HYBRID_AUTH_BYTES};
        use ml_dsa::{MlDsa65, Signer as MlSigner, SigningKey as MlSigningKey};

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

        tx.amount = Amount::from_units(5).unwrap();
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

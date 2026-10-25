//! Vérification Ed25519 des transferts expérimentaux par comptes.
//!
//! L'identifiant de compte est la clé publique Ed25519 brute (32 octets) et `auth_data`
//! contient exactement une signature Ed25519 (64 octets) du `signing_digest` canonique.
//! Ce choix est propre au prototype v1 et ne constitue pas un format d'adresse utilisateur.

use ed25519_dalek::{Signature, VerifyingKey};

use crate::accounts::AccountTransaction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignatureError {
    InvalidPublicKey,
    InvalidSignatureEncoding,
    InvalidSignature,
}

/// Valide les octets de signature d'une transaction contre la clé publique `sender`.
pub fn verify_account_transaction(tx: &AccountTransaction) -> Result<(), SignatureError> {
    let key =
        VerifyingKey::from_bytes(&tx.sender.0).map_err(|_| SignatureError::InvalidPublicKey)?;
    let signature = Signature::from_slice(&tx.auth_data)
        .map_err(|_| SignatureError::InvalidSignatureEncoding)?;
    let digest = tx
        .signing_digest()
        .map_err(|_| SignatureError::InvalidSignature)?;
    key.verify_strict(digest.as_bytes(), &signature)
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
    fn account_id_must_be_a_valid_ed25519_public_key() {
        let (_, mut tx) = signed_transaction();
        tx.sender = AccountId([0; 32]);
        assert_eq!(
            verify_account_transaction(&tx),
            Err(SignatureError::InvalidPublicKey)
        );
    }
}

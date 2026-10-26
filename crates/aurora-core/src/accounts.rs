//! Transaction par comptes — modèle expérimental non signé.
//!
//! Ce module définit la forme canonique de base uniquement. Il ne vérifie pas les
//! signatures, les soldes, les nonces contre l'état de la chaîne, ni les frais minimums.
//! Ces contrôles appartiennent au validateur d'état, qui sera implémenté séparément.

use crate::amount::{Amount, AmountError};
use crate::codec::{CanonicalDecode, CanonicalEncode, DecodeError, Decoder, EncodeError, Encoder};
use crate::hash::{tagged_hash, Domain, Hash256};
use crate::network::NetworkId;
use std::fmt;

pub const ACCOUNT_TX_VERSION: u32 = 1;
/// Experimental hybrid Ed25519 + FIPS 204 ML-DSA-65 transaction version.
pub const HYBRID_ACCOUNT_TX_VERSION: u32 = 2;
/// Native post-quantum ML-DSA-65 transaction profile for the experimental testnet.
pub const POST_QUANTUM_ACCOUNT_TX_VERSION: u32 = 3;
pub const ACCOUNT_ID_LEN: usize = 32;
/// ML-DSA-65 encoded public key and signature sizes from FIPS 204.
pub const ML_DSA_65_PUBLIC_KEY_BYTES: usize = 1_952;
pub const ML_DSA_65_SIGNATURE_BYTES: usize = 3_309;
/// Fixed v2 payload: Ed25519 public key, ML-DSA key, Ed25519 signature, ML-DSA signature.
pub const HYBRID_AUTH_BYTES: usize =
    32 + ML_DSA_65_PUBLIC_KEY_BYTES + 64 + ML_DSA_65_SIGNATURE_BYTES;
/// Fixed v3 payload: ML-DSA-65 public key followed by its signature.
pub const POST_QUANTUM_AUTH_BYTES: usize = ML_DSA_65_PUBLIC_KEY_BYTES + ML_DSA_65_SIGNATURE_BYTES;
pub const MAX_AUTH_BYTES: usize = 8_192;

/// Identifiant de compte brut. Le format d'adresse lisible reste à définir.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AccountId(pub [u8; ACCOUNT_ID_LEN]);

impl CanonicalEncode for AccountId {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        enc.put_fixed(&self.0);
        Ok(())
    }
}

impl CanonicalDecode for AccountId {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Self(dec.get_array::<ACCOUNT_ID_LEN>()?))
    }
}

/// Transfert de compte. Les données d'autorisation sont opaques à ce stade.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountTransaction {
    pub version: u32,
    pub network: NetworkId,
    pub sender: AccountId,
    pub nonce: u64,
    pub recipient: AccountId,
    pub amount: Amount,
    pub fee: Amount,
    pub auth_data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountTxError {
    UnsupportedVersion,
    ZeroAmount,
    SameSenderAndRecipient,
    AuthDataTooLarge,
    TotalOutOfRange(AmountError),
}

impl fmt::Display for AccountTxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion => write!(f, "version de transaction non prise en charge"),
            Self::ZeroAmount => write!(f, "le montant du transfert doit être positif"),
            Self::SameSenderAndRecipient => write!(f, "expéditeur et destinataire identiques"),
            Self::AuthDataTooLarge => write!(f, "données d'autorisation trop volumineuses"),
            Self::TotalOutOfRange(err) => write!(f, "montant total hors limites : {err}"),
        }
    }
}

impl std::error::Error for AccountTxError {}

impl AccountTransaction {
    /// Identifiant déterministe des octets canoniques complets, autorisation incluse.
    pub fn txid(&self) -> Result<Hash256, EncodeError> {
        Ok(tagged_hash(
            Domain::AccountTxId,
            &self.to_canonical_bytes()?,
        ))
    }

    /// Empreinte des champs qu'une signature doit autoriser.
    ///
    /// `auth_data` est volontairement exclu pour éviter une dépendance circulaire entre la
    /// signature et les octets de la signature elle-même. Le domaine est versionné et distinct
    /// du txid. Ce format est expérimental : ne pas signer de fonds réels avec cette version.
    pub fn signing_digest(&self) -> Result<Hash256, EncodeError> {
        let mut enc = Encoder::new();
        enc.put_u32(self.version);
        self.network.encode(&mut enc)?;
        self.sender.encode(&mut enc)?;
        enc.put_u64(self.nonce);
        self.recipient.encode(&mut enc)?;
        self.amount.encode(&mut enc)?;
        self.fee.encode(&mut enc)?;
        Ok(tagged_hash(Domain::TxSigning, &enc.into_bytes()))
    }

    /// Contrôles indépendants de l'état et de la cryptographie.
    pub fn check_stateless(&self) -> Result<(), AccountTxError> {
        if !matches!(
            self.version,
            ACCOUNT_TX_VERSION | HYBRID_ACCOUNT_TX_VERSION | POST_QUANTUM_ACCOUNT_TX_VERSION
        ) {
            return Err(AccountTxError::UnsupportedVersion);
        }
        if (self.version == HYBRID_ACCOUNT_TX_VERSION && self.auth_data.len() != HYBRID_AUTH_BYTES)
            || (self.version == POST_QUANTUM_ACCOUNT_TX_VERSION
                && self.auth_data.len() != POST_QUANTUM_AUTH_BYTES)
        {
            return Err(AccountTxError::AuthDataTooLarge);
        }
        if self.amount == Amount::ZERO {
            return Err(AccountTxError::ZeroAmount);
        }
        if self.sender == self.recipient {
            return Err(AccountTxError::SameSenderAndRecipient);
        }
        if self.auth_data.len() > MAX_AUTH_BYTES {
            return Err(AccountTxError::AuthDataTooLarge);
        }
        self.amount
            .checked_add(self.fee)
            .map_err(AccountTxError::TotalOutOfRange)?;
        Ok(())
    }
}

impl CanonicalEncode for AccountTransaction {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        enc.put_u32(self.version);
        self.network.encode(enc)?;
        self.sender.encode(enc)?;
        enc.put_u64(self.nonce);
        self.recipient.encode(enc)?;
        self.amount.encode(enc)?;
        self.fee.encode(enc)?;
        enc.put_var_bytes(&self.auth_data, MAX_AUTH_BYTES)
    }
}

impl CanonicalDecode for AccountTransaction {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let version = dec.get_u32()?;
        if !matches!(
            version,
            ACCOUNT_TX_VERSION | HYBRID_ACCOUNT_TX_VERSION | POST_QUANTUM_ACCOUNT_TX_VERSION
        ) {
            return Err(DecodeError::InvalidValue("version de transaction inconnue"));
        }
        Ok(Self {
            version,
            network: NetworkId::decode(dec)?,
            sender: AccountId::decode(dec)?,
            nonce: dec.get_u64()?,
            recipient: AccountId::decode(dec)?,
            amount: Amount::decode(dec)?,
            fee: Amount::decode(dec)?,
            auth_data: dec.get_var_bytes(MAX_AUTH_BYTES)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AccountTransaction {
        AccountTransaction {
            version: ACCOUNT_TX_VERSION,
            network: NetworkId::Regtest,
            sender: AccountId([1; ACCOUNT_ID_LEN]),
            nonce: 0,
            recipient: AccountId([2; ACCOUNT_ID_LEN]),
            amount: Amount::from_units(100).unwrap(),
            fee: Amount::from_units(3).unwrap(),
            auth_data: vec![],
        }
    }

    #[test]
    fn canonical_roundtrip() {
        let tx = sample();
        let bytes = tx.to_canonical_bytes().unwrap();
        assert_eq!(
            AccountTransaction::from_canonical_bytes(&bytes).unwrap(),
            tx
        );
    }

    #[test]
    fn txid_is_deterministic_and_commits_to_nonce() {
        let tx = sample();
        let id = tx.txid().unwrap();
        assert_eq!(tx.txid().unwrap(), id);
        let mut changed = tx.clone();
        changed.nonce += 1;
        assert_ne!(changed.txid().unwrap(), id);
    }

    #[test]
    fn signing_digest_excludes_authorization_but_commits_to_transaction_fields() {
        let tx = sample();
        let digest = tx.signing_digest().unwrap();

        let mut different_auth = tx.clone();
        different_auth.auth_data = vec![9, 8, 7];
        assert_eq!(different_auth.signing_digest().unwrap(), digest);
        assert_ne!(different_auth.txid().unwrap(), tx.txid().unwrap());

        let mut different_amount = tx.clone();
        different_amount.amount = Amount::from_units(101).unwrap();
        assert_ne!(different_amount.signing_digest().unwrap(), digest);

        let mut different_network = tx.clone();
        different_network.network = NetworkId::Testnet;
        assert_ne!(different_network.signing_digest().unwrap(), digest);

        let mut different_fee = tx.clone();
        different_fee.fee = Amount::from_units(4).unwrap();
        assert_ne!(different_fee.signing_digest().unwrap(), digest);
    }

    #[test]
    fn signing_digest_is_distinct_from_transaction_id() {
        let tx = sample();
        assert_ne!(tx.signing_digest().unwrap(), tx.txid().unwrap());
    }

    #[test]
    fn rejects_zero_amount_and_same_account() {
        let mut tx = sample();
        tx.amount = Amount::ZERO;
        assert_eq!(tx.check_stateless(), Err(AccountTxError::ZeroAmount));
        let mut tx = sample();
        tx.recipient = tx.sender;
        assert_eq!(
            tx.check_stateless(),
            Err(AccountTxError::SameSenderAndRecipient)
        );
    }

    #[test]
    fn rejects_total_above_amount_bound() {
        let mut tx = sample();
        tx.amount = Amount::MAX;
        tx.fee = Amount::from_units(1).unwrap();
        assert!(matches!(
            tx.check_stateless(),
            Err(AccountTxError::TotalOutOfRange(AmountError::AboveCap))
        ));
    }

    #[test]
    fn hybrid_version_has_a_fixed_canonical_authorization_size() {
        let mut tx = sample();
        tx.version = HYBRID_ACCOUNT_TX_VERSION;
        tx.auth_data = vec![0; HYBRID_AUTH_BYTES];
        assert_eq!(tx.check_stateless(), Ok(()));
        let encoded = tx.to_canonical_bytes().unwrap();
        assert_eq!(
            AccountTransaction::from_canonical_bytes(&encoded).unwrap(),
            tx
        );
        tx.auth_data.pop();
        assert_eq!(tx.check_stateless(), Err(AccountTxError::AuthDataTooLarge));
    }

    #[test]
    fn native_post_quantum_v3_roundtrips_with_fixed_authorization_size() {
        let mut tx = sample();
        tx.version = POST_QUANTUM_ACCOUNT_TX_VERSION;
        tx.auth_data = vec![0; POST_QUANTUM_AUTH_BYTES];
        assert_eq!(tx.check_stateless(), Ok(()));
        let encoded = tx.to_canonical_bytes().unwrap();
        assert_eq!(
            AccountTransaction::from_canonical_bytes(&encoded).unwrap(),
            tx
        );
        tx.auth_data.pop();
        assert_eq!(tx.check_stateless(), Err(AccountTxError::AuthDataTooLarge));
    }

    #[test]
    fn rejects_oversized_authorization_data() {
        let mut tx = sample();
        tx.auth_data = vec![0; MAX_AUTH_BYTES + 1];
        assert_eq!(tx.check_stateless(), Err(AccountTxError::AuthDataTooLarge));
        assert!(tx.to_canonical_bytes().is_err());
    }

    #[test]
    fn rejects_wrong_network_encoding_and_trailing_bytes() {
        let tx = sample();
        let mut bytes = tx.to_canonical_bytes().unwrap();
        bytes[4] = 255;
        assert!(AccountTransaction::from_canonical_bytes(&bytes).is_err());
        let mut bytes = tx.to_canonical_bytes().unwrap();
        bytes.push(0);
        assert!(matches!(
            AccountTransaction::from_canonical_bytes(&bytes),
            Err(DecodeError::TrailingBytes { remaining: 1 })
        ));
    }
}

//! Transaction **expérimentale et jetable**.
//!
//! Cette structure ne préjuge **pas** de la décision TX-01 (UTXO ou comptes) : elle a la forme
//! « entrées / sorties » uniquement parce qu'il faut une structure concrète pour tester la
//! sérialisation. Le prototype de comparaison UTXO / comptes (phase 0b) reste à écrire, et ce
//! module pourra être supprimé ou remplacé après lui.
//!
//! Limites explicites :
//! - `auth_data` est un champ opaque : **aucune signature n'est vérifiée** (CRY-01 ouvert) ;
//! - la validation est sans état : on ne sait pas si une entrée référence un fonds existant ;
//! - les bornes de taille sont arbitraires (FEE-02 / FEE-03 ouverts) ;
//! - l'identifiant de transaction couvre tout l'encodage, `auth_data` inclus (TX-05 ouvert).

use std::collections::HashSet;
use std::fmt;

use crate::amount::{Amount, AmountError};
use crate::codec::{
    decode_vec, encode_vec, CanonicalDecode, CanonicalEncode, DecodeError, Decoder, EncodeError,
    Encoder,
};
use crate::hash::{tagged_hash, Domain, Hash256};
use crate::network::NetworkId;

/// Version de format de la transaction expérimentale.
pub const TX_VERSION: u32 = 0;
/// Nombre maximal d'entrées (valeur arbitraire de prototype).
pub const MAX_INPUTS: usize = 256;
/// Nombre maximal de sorties (valeur arbitraire de prototype).
pub const MAX_OUTPUTS: usize = 256;
/// Taille maximale des données d'autorisation (valeur arbitraire de prototype).
pub const MAX_AUTH_BYTES: usize = 8_192;
/// Taille d'un destinataire en octets (format d'adresse non défini : TX-08 ouvert).
pub const RECIPIENT_LEN: usize = 32;

/// Référence à une sortie précédente.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OutPoint {
    /// Identifiant de la transaction qui a créé la sortie.
    pub txid: Hash256,
    /// Position de la sortie dans cette transaction.
    pub index: u32,
}

/// Entrée de transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TxInput {
    /// Sortie dépensée.
    pub previous: OutPoint,
}

/// Sortie de transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TxOutput {
    /// Montant transféré.
    pub amount: Amount,
    /// Destinataire (octets bruts, sans format d'adresse).
    pub recipient: [u8; RECIPIENT_LEN],
}

/// Transaction expérimentale.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    /// Version du format ([`TX_VERSION`]).
    pub version: u32,
    /// Réseau visé.
    pub network: NetworkId,
    /// Entrées.
    pub inputs: Vec<TxInput>,
    /// Sorties.
    pub outputs: Vec<TxOutput>,
    /// Frais déclarés.
    pub fee: Amount,
    /// Données d'autorisation opaques, **non vérifiées**.
    pub auth_data: Vec<u8>,
}

/// Erreur de validation sans état.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxError {
    /// Version de transaction non prise en charge.
    UnsupportedVersion,
    /// Trop d'entrées pour le format expérimental.
    TooManyInputs,
    /// Trop de sorties pour le format expérimental.
    TooManyOutputs,
    /// Données d'autorisation trop volumineuses.
    AuthDataTooLarge,
    /// Aucune sortie.
    NoOutputs,
    /// La même sortie est dépensée deux fois dans la transaction.
    DuplicateInput,
    /// Le total des sorties et des frais sort de l'intervalle autorisé.
    TotalOutOfRange(AmountError),
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxError::UnsupportedVersion => write!(f, "version de transaction non prise en charge"),
            TxError::TooManyInputs => write!(f, "trop d'entrées"),
            TxError::TooManyOutputs => write!(f, "trop de sorties"),
            TxError::AuthDataTooLarge => write!(f, "données d'autorisation trop volumineuses"),
            TxError::NoOutputs => write!(f, "aucune sortie"),
            TxError::DuplicateInput => write!(f, "entrée dupliquée"),
            TxError::TotalOutOfRange(err) => write!(f, "total hors limites : {err}"),
        }
    }
}

impl std::error::Error for TxError {}

impl Transaction {
    /// Identifiant : condensat du domaine `TxId` sur l'encodage canonique complet.
    pub fn txid(&self) -> Result<Hash256, EncodeError> {
        let bytes = self.to_canonical_bytes()?;
        Ok(tagged_hash(Domain::TxId, &bytes))
    }

    /// Vérifications sans état. Ne vérifie ni les signatures, ni l'existence des fonds.
    pub fn check_stateless(&self) -> Result<(), TxError> {
        if self.version != TX_VERSION {
            return Err(TxError::UnsupportedVersion);
        }
        if self.inputs.len() > MAX_INPUTS {
            return Err(TxError::TooManyInputs);
        }
        if self.outputs.len() > MAX_OUTPUTS {
            return Err(TxError::TooManyOutputs);
        }
        if self.auth_data.len() > MAX_AUTH_BYTES {
            return Err(TxError::AuthDataTooLarge);
        }
        if self.outputs.is_empty() {
            return Err(TxError::NoOutputs);
        }
        let mut seen = HashSet::with_capacity(self.inputs.len());
        for input in &self.inputs {
            if !seen.insert(input.previous) {
                return Err(TxError::DuplicateInput);
            }
        }
        let total = Amount::checked_sum(self.outputs.iter().map(|output| output.amount))
            .map_err(TxError::TotalOutOfRange)?;
        total
            .checked_add(self.fee)
            .map_err(TxError::TotalOutOfRange)?;
        Ok(())
    }
}

impl CanonicalEncode for OutPoint {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        self.txid.encode(enc)?;
        enc.put_u32(self.index);
        Ok(())
    }
}

impl CanonicalDecode for OutPoint {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let txid = Hash256::decode(dec)?;
        let index = dec.get_u32()?;
        Ok(OutPoint { txid, index })
    }
}

impl CanonicalEncode for TxInput {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        self.previous.encode(enc)
    }
}

impl CanonicalDecode for TxInput {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(TxInput {
            previous: OutPoint::decode(dec)?,
        })
    }
}

impl CanonicalEncode for TxOutput {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        self.amount.encode(enc)?;
        enc.put_fixed(&self.recipient);
        Ok(())
    }
}

impl CanonicalDecode for TxOutput {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let amount = Amount::decode(dec)?;
        let recipient = dec.get_array::<RECIPIENT_LEN>()?;
        Ok(TxOutput { amount, recipient })
    }
}

impl CanonicalEncode for Transaction {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        enc.put_u32(self.version);
        self.network.encode(enc)?;
        encode_vec(&self.inputs, MAX_INPUTS, enc)?;
        encode_vec(&self.outputs, MAX_OUTPUTS, enc)?;
        self.fee.encode(enc)?;
        enc.put_var_bytes(&self.auth_data, MAX_AUTH_BYTES)
    }
}

impl CanonicalDecode for Transaction {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let version = dec.get_u32()?;
        if version != TX_VERSION {
            return Err(DecodeError::InvalidValue("version de transaction inconnue"));
        }
        let network = NetworkId::decode(dec)?;
        let inputs = decode_vec::<TxInput>(dec, MAX_INPUTS)?;
        let outputs = decode_vec::<TxOutput>(dec, MAX_OUTPUTS)?;
        let fee = Amount::decode(dec)?;
        let auth_data = dec.get_var_bytes(MAX_AUTH_BYTES)?;
        Ok(Transaction {
            version,
            network,
            inputs,
            outputs,
            fee,
            auth_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amount::MAX_UNITS;

    fn out(units: u64) -> TxOutput {
        TxOutput {
            amount: Amount::from_units(units).unwrap(),
            recipient: [7u8; RECIPIENT_LEN],
        }
    }

    fn input(seed: u8, index: u32) -> TxInput {
        TxInput {
            previous: OutPoint {
                txid: Hash256::from_bytes([seed; 32]),
                index,
            },
        }
    }

    fn base_tx() -> Transaction {
        Transaction {
            version: TX_VERSION,
            network: NetworkId::Regtest,
            inputs: vec![input(1, 0)],
            outputs: vec![out(1_000)],
            fee: Amount::from_units(10).unwrap(),
            auth_data: vec![],
        }
    }

    #[test]
    fn valid_transaction_passes_stateless_check() {
        assert_eq!(base_tx().check_stateless(), Ok(()));
    }

    #[test]
    fn unsupported_version_is_rejected_even_for_in_memory_transactions() {
        let mut tx = base_tx();
        tx.version = TX_VERSION + 1;
        assert_eq!(tx.check_stateless(), Err(TxError::UnsupportedVersion));
    }

    #[test]
    fn too_many_inputs_are_rejected_even_in_memory() {
        let mut tx = base_tx();
        tx.inputs = (0..=MAX_INPUTS)
            .map(|index| input(1, index as u32))
            .collect();
        assert_eq!(tx.check_stateless(), Err(TxError::TooManyInputs));
    }

    #[test]
    fn too_many_outputs_are_rejected_even_in_memory() {
        let mut tx = base_tx();
        tx.outputs = vec![out(1); MAX_OUTPUTS + 1];
        assert_eq!(tx.check_stateless(), Err(TxError::TooManyOutputs));
    }

    #[test]
    fn oversized_auth_data_is_rejected_even_in_memory() {
        let mut tx = base_tx();
        tx.auth_data = vec![0; MAX_AUTH_BYTES + 1];
        assert_eq!(tx.check_stateless(), Err(TxError::AuthDataTooLarge));
    }

    #[test]
    fn no_outputs_is_rejected() {
        let mut tx = base_tx();
        tx.outputs.clear();
        assert_eq!(tx.check_stateless(), Err(TxError::NoOutputs));
    }

    #[test]
    fn duplicate_input_is_rejected() {
        let mut tx = base_tx();
        tx.inputs = vec![input(1, 0), input(1, 0)];
        assert_eq!(tx.check_stateless(), Err(TxError::DuplicateInput));
    }

    #[test]
    fn same_txid_different_index_is_not_a_duplicate() {
        let mut tx = base_tx();
        tx.inputs = vec![input(1, 0), input(1, 1)];
        assert_eq!(tx.check_stateless(), Ok(()));
    }

    #[test]
    fn outputs_plus_fee_above_cap_is_rejected() {
        let mut tx = base_tx();
        tx.outputs = vec![out(MAX_UNITS)];
        tx.fee = Amount::from_units(1).unwrap();
        assert_eq!(
            tx.check_stateless(),
            Err(TxError::TotalOutOfRange(AmountError::AboveCap))
        );
    }

    #[test]
    fn txid_changes_when_any_field_changes() {
        let base = base_tx();
        let id = base.txid().unwrap();

        let mut other = base.clone();
        other.fee = Amount::from_units(11).unwrap();
        assert_ne!(other.txid().unwrap(), id);

        let mut other = base.clone();
        other.network = NetworkId::Testnet;
        assert_ne!(other.txid().unwrap(), id);

        let mut other = base.clone();
        other.auth_data = vec![1];
        assert_ne!(other.txid().unwrap(), id);

        assert_eq!(base.txid().unwrap(), id);
    }

    #[test]
    fn roundtrip_and_oversize_are_handled() {
        let tx = base_tx();
        let bytes = tx.to_canonical_bytes().unwrap();
        assert_eq!(Transaction::from_canonical_bytes(&bytes).unwrap(), tx);

        let mut big = base_tx();
        big.auth_data = vec![0u8; MAX_AUTH_BYTES + 1];
        assert!(big.to_canonical_bytes().is_err());
    }
}

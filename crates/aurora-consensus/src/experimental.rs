//! Bloc expérimental et jetable.
//!
//! Hypothèses (non validées, voir `docs/DECISION_REGISTER.md`, annexe B) :
//! - l'en-tête ne contient **pas** de cible de difficulté ni de preuve de travail (POW-03 ouvert) ;
//! - l'engagement sur les transactions est un condensat de la liste ordonnée des identifiants,
//!   précédé du nombre de transactions : **ce n'est pas un arbre de Merkle** (TX-04 ouvert) ;
//! - aucune règle d'horodatage (POW-07 ouvert), aucune récompense, aucune limite de taille en
//!   octets (FEE-02 ouvert) ; seul un nombre maximal de transactions est imposé.

use std::collections::HashSet;
use std::fmt;

use aurora_core::codec::{
    decode_vec, encode_vec, CanonicalDecode, CanonicalEncode, DecodeError, Decoder, EncodeError,
    Encoder,
};
use aurora_core::experimental::{Transaction, TxError};
use aurora_core::hash::{tagged_hash, Domain, Hash256};
use aurora_core::merkle::{merkle_root, MAX_MERKLE_LEAVES};
use aurora_core::network::NetworkId;

/// Version de format de l'en-tête expérimental.
pub const HEADER_VERSION: u32 = 1;
/// Nombre maximal de transactions par bloc (valeur arbitraire de prototype).
pub const MAX_BLOCK_TXS: usize = 10_000;

/// En-tête de bloc expérimental.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockHeader {
    /// Version du format ([`HEADER_VERSION`]).
    pub version: u32,
    /// Réseau du bloc.
    pub network: NetworkId,
    /// Condensat de l'en-tête parent.
    pub prev_hash: Hash256,
    /// Racine de Merkle des identifiants de transaction ordonnés.
    pub tx_merkle_root: Hash256,
    /// Engagement historique du contenu (égal à la racine dans le format générique).
    pub tx_commitment: Hash256,
    /// Horodatage déclaré, **non vérifié**.
    pub timestamp: u64,
    /// Nonce libre, sans signification tant que le PoW n'est pas défini.
    pub nonce: u64,
}

/// Bloc expérimental.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// En-tête.
    pub header: BlockHeader,
    /// Transactions, dans l'ordre.
    pub transactions: Vec<Transaction>,
}

/// Erreur de validation sans état d'un bloc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockError {
    /// Version d'en-tête non prise en charge.
    UnsupportedHeaderVersion,
    /// Trop de transactions.
    TooManyTransactions,
    /// Une transaction ne vise pas le même réseau que le bloc.
    NetworkMismatch,
    /// Une transaction est invalide.
    InvalidTransaction(TxError),
    /// Deux transactions ont le même identifiant.
    DuplicateTransaction,
    /// L'engagement de l'en-tête ne correspond pas aux transactions.
    CommitmentMismatch,
    /// Erreur d'encodage lors du calcul d'un identifiant ou de l'engagement.
    Encoding(EncodeError),
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockError::UnsupportedHeaderVersion => {
                write!(f, "version d'en-tête non prise en charge")
            }
            BlockError::TooManyTransactions => write!(f, "trop de transactions"),
            BlockError::NetworkMismatch => write!(f, "réseau de transaction différent du bloc"),
            BlockError::InvalidTransaction(err) => write!(f, "transaction invalide : {err}"),
            BlockError::DuplicateTransaction => write!(f, "transaction dupliquée"),
            BlockError::CommitmentMismatch => write!(f, "engagement de transactions incorrect"),
            BlockError::Encoding(err) => write!(f, "erreur d'encodage : {err}"),
        }
    }
}

impl std::error::Error for BlockError {}

/// Engagement sur la liste ordonnée des transactions (hypothèse expérimentale, pas un Merkle).
pub fn compute_tx_commitment(transactions: &[Transaction]) -> Result<Hash256, EncodeError> {
    if transactions.len() > MAX_BLOCK_TXS || transactions.len() > MAX_MERKLE_LEAVES {
        return Err(EncodeError::LengthTooLarge {
            len: transactions.len(),
            max: MAX_BLOCK_TXS.min(MAX_MERKLE_LEAVES),
        });
    }
    let txids = transactions
        .iter()
        .map(|tx| tx.txid())
        .collect::<Result<Vec<_>, _>>()?;
    merkle_root(&txids).map_err(|_| EncodeError::LengthTooLarge {
        len: transactions.len(),
        max: MAX_BLOCK_TXS.min(MAX_MERKLE_LEAVES),
    })
}

impl BlockHeader {
    /// Condensat de l'en-tête : domaine `BlockHeader` sur l'encodage canonique.
    pub fn hash(&self) -> Result<Hash256, EncodeError> {
        let bytes = self.to_canonical_bytes()?;
        Ok(tagged_hash(Domain::BlockHeader, &bytes))
    }
}

impl Block {
    /// Construit un bloc dont l'engagement est calculé à partir des transactions.
    pub fn build(
        network: NetworkId,
        prev_hash: Hash256,
        timestamp: u64,
        nonce: u64,
        transactions: Vec<Transaction>,
    ) -> Result<Block, EncodeError> {
        let tx_commitment = compute_tx_commitment(&transactions)?;
        Ok(Block {
            header: BlockHeader {
                version: HEADER_VERSION,
                network,
                prev_hash,
                tx_merkle_root: tx_commitment,
                tx_commitment,
                timestamp,
                nonce,
            },
            transactions,
        })
    }

    /// Vérifications sans état. Ne vérifie ni preuve de travail, ni lien avec la chaîne, ni
    /// signatures, ni existence des fonds.
    pub fn check_stateless(&self) -> Result<(), BlockError> {
        if self.header.version != HEADER_VERSION {
            return Err(BlockError::UnsupportedHeaderVersion);
        }
        if self.transactions.len() > MAX_BLOCK_TXS {
            return Err(BlockError::TooManyTransactions);
        }
        let mut seen = HashSet::with_capacity(self.transactions.len());
        for tx in &self.transactions {
            if tx.network != self.header.network {
                return Err(BlockError::NetworkMismatch);
            }
            tx.check_stateless()
                .map_err(BlockError::InvalidTransaction)?;
            let id = tx.txid().map_err(BlockError::Encoding)?;
            if !seen.insert(id) {
                return Err(BlockError::DuplicateTransaction);
            }
        }
        let expected = compute_tx_commitment(&self.transactions).map_err(BlockError::Encoding)?;
        if expected != self.header.tx_merkle_root || expected != self.header.tx_commitment {
            return Err(BlockError::CommitmentMismatch);
        }
        Ok(())
    }
}

impl CanonicalEncode for BlockHeader {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        enc.put_u32(self.version);
        self.network.encode(enc)?;
        self.prev_hash.encode(enc)?;
        self.tx_commitment.encode(enc)?;
        self.tx_merkle_root.encode(enc)?;
        enc.put_u64(self.timestamp);
        enc.put_u64(self.nonce);
        Ok(())
    }
}

impl CanonicalDecode for BlockHeader {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let version = dec.get_u32()?;
        if version != HEADER_VERSION {
            return Err(DecodeError::InvalidValue("version d'en-tête inconnue"));
        }
        let network = NetworkId::decode(dec)?;
        let prev_hash = Hash256::decode(dec)?;
        let tx_commitment = Hash256::decode(dec)?;
        let tx_merkle_root = Hash256::decode(dec)?;
        let timestamp = dec.get_u64()?;
        let nonce = dec.get_u64()?;
        Ok(BlockHeader {
            version,
            network,
            prev_hash,
            tx_commitment,
            tx_merkle_root,
            timestamp,
            nonce,
        })
    }
}

impl CanonicalEncode for Block {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        self.header.encode(enc)?;
        encode_vec(&self.transactions, MAX_BLOCK_TXS, enc)
    }
}

impl CanonicalDecode for Block {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let header = BlockHeader::decode(dec)?;
        let transactions = decode_vec::<Transaction>(dec, MAX_BLOCK_TXS)?;
        Ok(Block {
            header,
            transactions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::amount::Amount;
    use aurora_core::experimental::{OutPoint, TxInput, TxOutput, RECIPIENT_LEN, TX_VERSION};

    fn tx(seed: u8, network: NetworkId) -> Transaction {
        Transaction {
            version: TX_VERSION,
            network,
            inputs: vec![TxInput {
                previous: OutPoint {
                    txid: Hash256::from_bytes([seed; 32]),
                    index: 0,
                },
            }],
            outputs: vec![TxOutput {
                amount: Amount::from_units(1_000).unwrap(),
                recipient: [seed; RECIPIENT_LEN],
            }],
            fee: Amount::from_units(10).unwrap(),
            auth_data: vec![],
        }
    }

    fn block_of(txs: Vec<Transaction>) -> Block {
        Block::build(NetworkId::Regtest, Hash256::ZERO, 1_000, 0, txs).unwrap()
    }

    #[test]
    fn unsupported_header_version_is_rejected_in_memory() {
        let mut block = block_of(vec![]);
        block.header.version = HEADER_VERSION + 1;
        assert_eq!(
            block.check_stateless(),
            Err(BlockError::UnsupportedHeaderVersion)
        );
    }

    #[test]
    fn valid_block_passes_and_roundtrips() {
        let block = block_of(vec![tx(1, NetworkId::Regtest), tx(2, NetworkId::Regtest)]);
        assert_eq!(block.check_stateless(), Ok(()));
        let bytes = block.to_canonical_bytes().unwrap();
        assert_eq!(Block::from_canonical_bytes(&bytes).unwrap(), block);
    }

    #[test]
    fn empty_block_has_a_defined_commitment() {
        let block = block_of(vec![]);
        assert_eq!(block.check_stateless(), Ok(()));
    }

    #[test]
    fn tampered_commitment_is_rejected() {
        let mut block = block_of(vec![tx(1, NetworkId::Regtest)]);
        block.header.tx_commitment = Hash256::ZERO;
        assert_eq!(block.check_stateless(), Err(BlockError::CommitmentMismatch));
    }

    #[test]
    fn adding_a_transaction_after_build_is_detected() {
        let mut block = block_of(vec![tx(1, NetworkId::Regtest)]);
        block.transactions.push(tx(2, NetworkId::Regtest));
        assert_eq!(block.check_stateless(), Err(BlockError::CommitmentMismatch));
    }

    #[test]
    fn order_of_transactions_changes_the_commitment() {
        let a = block_of(vec![tx(1, NetworkId::Regtest), tx(2, NetworkId::Regtest)]);
        let b = block_of(vec![tx(2, NetworkId::Regtest), tx(1, NetworkId::Regtest)]);
        assert_ne!(a.header.tx_commitment, b.header.tx_commitment);
    }

    #[test]
    fn network_mismatch_is_rejected() {
        let block = block_of(vec![tx(1, NetworkId::Testnet)]);
        assert_eq!(block.check_stateless(), Err(BlockError::NetworkMismatch));
    }

    #[test]
    fn duplicate_transaction_is_rejected() {
        let block = block_of(vec![tx(1, NetworkId::Regtest), tx(1, NetworkId::Regtest)]);
        assert_eq!(
            block.check_stateless(),
            Err(BlockError::DuplicateTransaction)
        );
    }

    #[test]
    fn invalid_transaction_is_reported() {
        let mut bad = tx(1, NetworkId::Regtest);
        bad.outputs.clear();
        let block = block_of(vec![bad]);
        assert_eq!(
            block.check_stateless(),
            Err(BlockError::InvalidTransaction(TxError::NoOutputs))
        );
    }

    #[test]
    fn header_hash_depends_on_every_field() {
        let base = block_of(vec![tx(1, NetworkId::Regtest)]).header;
        let h = base.hash().unwrap();

        let mut other = base;
        other.nonce += 1;
        assert_ne!(other.hash().unwrap(), h);

        let mut other = base;
        other.timestamp += 1;
        assert_ne!(other.hash().unwrap(), h);

        let mut other = base;
        other.prev_hash = Hash256::from_bytes([1; 32]);
        assert_ne!(other.hash().unwrap(), h);

        assert_eq!(base.hash().unwrap(), h);
    }

    #[test]
    fn too_many_transactions_are_rejected_even_in_memory() {
        let mut block = block_of(vec![]);
        block.transactions = vec![tx(1, NetworkId::Regtest); MAX_BLOCK_TXS + 1];
        assert_eq!(
            block.check_stateless(),
            Err(BlockError::TooManyTransactions)
        );
    }

    #[test]
    fn unknown_header_version_is_rejected() {
        let block = block_of(vec![]);
        let mut bytes = block.to_canonical_bytes().unwrap();
        bytes[3] = 1;
        assert!(matches!(
            Block::from_canonical_bytes(&bytes),
            Err(DecodeError::InvalidValue(_))
        ));
    }
}

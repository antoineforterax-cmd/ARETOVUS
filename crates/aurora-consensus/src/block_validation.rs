//! Validation expérimentale combinant les contrôles sans état et le PoW.
//!
//! Ce module ne constitue pas un validateur de chaîne : la difficulté est fournie par l'appelant,
//! sans ajustement, règle de temps, validation d'état ni sélection par travail cumulé.
//! Ne pas utiliser avec de vrais fonds.

use std::fmt;

use crate::experimental::{Block, BlockError};
use crate::proof_of_work::{check_proof, PowError};

/// Erreur de validation expérimentale d'un bloc.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockPowError {
    /// Échec des contrôles sans état (réseau, transactions, engagement).
    Stateless(BlockError),
    /// Preuve de travail invalide ou difficulté incorrecte.
    ProofOfWork(PowError),
    /// Le hash de l'en-tête ne satisfait pas la difficulté demandée.
    InsufficientWork,
}

impl fmt::Display for BlockPowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stateless(err) => write!(f, "bloc invalide : {err}"),
            Self::ProofOfWork(err) => write!(f, "preuve de travail invalide : {err}"),
            Self::InsufficientWork => write!(f, "preuve de travail insuffisante"),
        }
    }
}

impl std::error::Error for BlockPowError {}

/// Vérifie les règles sans état puis la preuve de travail du bloc.
///
/// La difficulté est fournie explicitement par l'appelant. Cette fonction ne décide pas quelle
/// difficulté est légitime pour une hauteur donnée et ne vérifie pas l'état de la chaîne.
pub fn check_block_pow(block: &Block, difficulty_bits: u16) -> Result<(), BlockPowError> {
    block.check_stateless().map_err(BlockPowError::Stateless)?;
    match check_proof(&block.header, difficulty_bits).map_err(BlockPowError::ProofOfWork)? {
        true => Ok(()),
        false => Err(BlockPowError::InsufficientWork),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::Block;
    use aurora_core::hash::Hash256;
    use aurora_core::network::NetworkId;

    fn empty_block() -> Block {
        Block::build(NetworkId::Regtest, Hash256::ZERO, 1, 0, vec![]).unwrap()
    }

    #[test]
    fn zero_difficulty_accepts_a_stateless_valid_block() {
        assert_eq!(check_block_pow(&empty_block(), 0), Ok(()));
    }

    #[test]
    fn rejects_bad_transaction_commitment_before_pow() {
        let mut block = empty_block();
        block.header.tx_commitment = Hash256::ZERO;
        assert_eq!(
            check_block_pow(&block, 0),
            Err(BlockPowError::Stateless(BlockError::CommitmentMismatch))
        );
    }

    #[test]
    fn rejects_insufficient_work() {
        let block = empty_block();
        // A single fixed header has a negligible chance of satisfying 255 leading zero bits.
        assert_eq!(
            check_block_pow(&block, 255),
            Err(BlockPowError::InsufficientWork)
        );
    }

    #[test]
    fn rejects_invalid_difficulty() {
        assert_eq!(
            check_block_pow(&empty_block(), 256),
            Err(BlockPowError::ProofOfWork(PowError::InvalidDifficulty))
        );
    }
}

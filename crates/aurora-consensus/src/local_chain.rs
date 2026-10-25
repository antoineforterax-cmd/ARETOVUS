//! Chaîne linéaire en mémoire pour les tests locaux AURORA.
//!
//! Limites explicites : difficulté constante fournie à la construction, aucun ajustement,
//! aucune horodatation validée, aucune validation d'état/solde, aucune branche concurrente,
//! aucune persistance ni réseau. Ce n'est pas un consensus de production.

use std::fmt;

use aurora_core::hash::Hash256;
use aurora_core::network::NetworkId;

use crate::block_validation::{check_block_pow, BlockPowError};
use crate::experimental::Block;

/// Difficulté maximale de ce prototype de chaîne : le travail cumulé tient dans un u128.
pub const MAX_CHAIN_DIFFICULTY_BITS: u16 = 127;

/// Chaîne linéaire locale. Les blocs sont conservés dans l'ordre d'ajout.
#[derive(Clone, Debug)]
pub struct LocalChain {
    network: NetworkId,
    difficulty_bits: u16,
    blocks: Vec<Block>,
    cumulative_work: u128,
}

/// Erreur lors de la création ou de l'extension de la chaîne locale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainError {
    InvalidDifficulty,
    InvalidGenesisParent,
    WrongNetwork,
    WrongParent,
    InvalidBlock(BlockPowError),
    WorkOverflow,
}

impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDifficulty => write!(f, "difficulté de chaîne invalide"),
            Self::InvalidGenesisParent => write!(f, "le bloc initial doit avoir un parent nul"),
            Self::WrongNetwork => write!(f, "réseau du bloc différent de celui de la chaîne"),
            Self::WrongParent => write!(f, "le parent du bloc ne correspond pas au sommet"),
            Self::InvalidBlock(err) => write!(f, "bloc rejeté : {err}"),
            Self::WorkOverflow => write!(f, "débordement du travail cumulé"),
        }
    }
}

impl std::error::Error for ChainError {}

impl LocalChain {
    /// Crée une chaîne locale à partir d'un bloc initial déjà miné.
    pub fn new(genesis: Block, difficulty_bits: u16) -> Result<Self, ChainError> {
        if difficulty_bits > MAX_CHAIN_DIFFICULTY_BITS {
            return Err(ChainError::InvalidDifficulty);
        }
        if genesis.header.prev_hash != Hash256::ZERO {
            return Err(ChainError::InvalidGenesisParent);
        }
        check_block_pow(&genesis, difficulty_bits).map_err(ChainError::InvalidBlock)?;
        let cumulative_work = Self::work_for(difficulty_bits)?;
        Ok(Self {
            network: genesis.header.network,
            difficulty_bits,
            blocks: vec![genesis],
            cumulative_work,
        })
    }

    /// Valide et ajoute un bloc dont le parent doit être le sommet actuel.
    pub fn append(&mut self, block: Block) -> Result<(), ChainError> {
        if block.header.network != self.network {
            return Err(ChainError::WrongNetwork);
        }
        let expected_parent = self.tip_hash().ok_or(ChainError::InvalidGenesisParent)?;
        if block.header.prev_hash != expected_parent {
            return Err(ChainError::WrongParent);
        }
        check_block_pow(&block, self.difficulty_bits).map_err(ChainError::InvalidBlock)?;
        let work = Self::work_for(self.difficulty_bits)?;
        let next_work = self
            .cumulative_work
            .checked_add(work)
            .ok_or(ChainError::WorkOverflow)?;
        self.blocks.push(block);
        self.cumulative_work = next_work;
        Ok(())
    }

    fn work_for(difficulty_bits: u16) -> Result<u128, ChainError> {
        if difficulty_bits > MAX_CHAIN_DIFFICULTY_BITS {
            return Err(ChainError::InvalidDifficulty);
        }
        Ok(1u128 << difficulty_bits)
    }

    pub fn network(&self) -> NetworkId {
        self.network
    }

    pub fn difficulty_bits(&self) -> u16 {
        self.difficulty_bits
    }

    pub fn height(&self) -> u64 {
        (self.blocks.len() - 1) as u64
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn tip(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn tip_hash(&self) -> Option<Hash256> {
        self.tip().and_then(|block| block.header.hash().ok())
    }

    pub fn cumulative_work(&self) -> u128 {
        self.cumulative_work
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_of_work::mine_nonce;

    fn block(parent: Hash256, timestamp: u64) -> Block {
        Block::build(NetworkId::Regtest, parent, timestamp, 0, vec![]).unwrap()
    }

    fn mined(mut block: Block, difficulty: u16) -> Block {
        block.header.nonce = mine_nonce(&block.header, difficulty, 0, 1_000_000).unwrap();
        block
    }

    #[test]
    fn appends_only_a_block_linked_to_the_tip() {
        let genesis = mined(block(Hash256::ZERO, 1), 4);
        let mut chain = LocalChain::new(genesis, 4).unwrap();
        let parent = chain.tip_hash().unwrap();
        let next = mined(block(parent, 2), 4);
        chain.append(next).unwrap();
        assert_eq!(chain.height(), 1);
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.cumulative_work(), 32);
    }

    #[test]
    fn rejects_a_block_with_the_wrong_parent() {
        let genesis = mined(block(Hash256::ZERO, 1), 0);
        let mut chain = LocalChain::new(genesis, 0).unwrap();
        assert_eq!(
            chain.append(block(Hash256::ZERO, 2)),
            Err(ChainError::WrongParent)
        );
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn rejects_a_block_from_another_network() {
        let genesis = block(Hash256::ZERO, 1);
        let mut chain = LocalChain::new(genesis, 0).unwrap();
        let other =
            Block::build(NetworkId::Testnet, chain.tip_hash().unwrap(), 2, 0, vec![]).unwrap();
        assert_eq!(chain.append(other), Err(ChainError::WrongNetwork));
    }

    #[test]
    fn rejects_excessive_difficulty() {
        assert!(matches!(
            LocalChain::new(block(Hash256::ZERO, 1), 128),
            Err(ChainError::InvalidDifficulty)
        ));
    }
}

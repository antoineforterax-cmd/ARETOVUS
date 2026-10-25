//! Sélecteur expérimental de branches en mémoire à difficulté fixe.
//!
//! Toutes les branches acceptées utilisent la même difficulté; leur travail est donc égal
//! par bloc. La sélection par hauteur équivaut ici au travail cumulé. Ce raccourci ne convient
//! pas à une difficulté variable et n'inclut pas de validation d'état ni de persistance.

use std::collections::BTreeMap;
use std::fmt;

use aurora_core::{hash::Hash256, network::NetworkId};

use crate::{
    block_validation::{check_block_pow, BlockPowError},
    experimental::{Block, BlockError},
};

#[derive(Clone, Copy, Debug)]
struct Node {
    parent: Hash256,
    height: u64,
}

#[derive(Clone, Debug)]
pub struct ForkChain {
    network: NetworkId,
    difficulty_bits: u16,
    nodes: BTreeMap<Hash256, Node>,
    blocks: BTreeMap<Hash256, Block>,
    canonical: Vec<Hash256>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChainUpdate {
    pub became_best: bool,
    /// Première hauteur remplacée, ou `None` si le sommet canonique n'a pas changé.
    pub reorg_from_height: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForkChainError {
    InvalidGenesisParent,
    UnknownParent,
    DuplicateBlock,
    WrongNetwork,
    InvalidBlock(BlockPowError),
    HeightOverflow,
}

impl fmt::Display for ForkChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidGenesisParent => write!(f, "le bloc initial doit avoir un parent nul"),
            Self::UnknownParent => write!(f, "parent inconnu"),
            Self::DuplicateBlock => write!(f, "bloc déjà connu"),
            Self::WrongNetwork => write!(f, "réseau du bloc différent de celui de la chaîne"),
            Self::InvalidBlock(err) => write!(f, "bloc rejeté : {err}"),
            Self::HeightOverflow => write!(f, "hauteur de chaîne hors limites"),
        }
    }
}

impl std::error::Error for ForkChainError {}

impl ForkChain {
    pub fn new(genesis: Block, difficulty_bits: u16) -> Result<Self, ForkChainError> {
        if genesis.header.prev_hash != Hash256::ZERO {
            return Err(ForkChainError::InvalidGenesisParent);
        }
        check_block_pow(&genesis, difficulty_bits).map_err(ForkChainError::InvalidBlock)?;
        let hash = genesis.header.hash().map_err(|err| {
            ForkChainError::InvalidBlock(BlockPowError::Stateless(BlockError::Encoding(err)))
        })?;
        let mut nodes = BTreeMap::new();
        nodes.insert(
            hash,
            Node {
                parent: Hash256::ZERO,
                height: 0,
            },
        );
        let mut blocks = BTreeMap::new();
        blocks.insert(hash, genesis);
        Ok(Self {
            network: blocks[&hash].header.network,
            difficulty_bits,
            nodes,
            blocks,
            canonical: vec![hash],
        })
    }

    /// Valide un bloc enfant de toute branche connue et réorganise au besoin.
    pub fn submit(&mut self, block: Block) -> Result<ChainUpdate, ForkChainError> {
        if block.header.network != self.network {
            return Err(ForkChainError::WrongNetwork);
        }
        let parent = block.header.prev_hash;
        let parent_node = self
            .nodes
            .get(&parent)
            .ok_or(ForkChainError::UnknownParent)?;
        check_block_pow(&block, self.difficulty_bits).map_err(ForkChainError::InvalidBlock)?;
        let hash = block.header.hash().map_err(|err| {
            ForkChainError::InvalidBlock(BlockPowError::Stateless(BlockError::Encoding(err)))
        })?;
        if self.nodes.contains_key(&hash) {
            return Err(ForkChainError::DuplicateBlock);
        }
        let height = parent_node
            .height
            .checked_add(1)
            .ok_or(ForkChainError::HeightOverflow)?;
        self.nodes.insert(hash, Node { parent, height });
        self.blocks.insert(hash, block);

        let tip_hash = *self.canonical.last().expect("genesis always exists");
        let tip_node = self.nodes[&tip_hash];
        let is_better = height > tip_node.height || (height == tip_node.height && hash < tip_hash);
        if !is_better {
            return Ok(ChainUpdate {
                became_best: false,
                reorg_from_height: None,
            });
        }

        let candidate = self.path_to_genesis(hash);
        let common_len = candidate
            .iter()
            .zip(&self.canonical)
            .take_while(|(a, b)| a == b)
            .count();
        let reorg_from_height = (common_len < self.canonical.len()).then_some(common_len as u64);
        self.canonical = candidate;
        Ok(ChainUpdate {
            became_best: true,
            reorg_from_height,
        })
    }

    fn path_to_genesis(&self, mut hash: Hash256) -> Vec<Hash256> {
        let mut path = Vec::new();
        loop {
            path.push(hash);
            let node = self.nodes[&hash];
            if node.height == 0 {
                break;
            }
            hash = node.parent;
        }
        path.reverse();
        path
    }

    pub fn height(&self) -> u64 {
        (self.canonical.len() - 1) as u64
    }
    pub fn tip_hash(&self) -> Hash256 {
        *self.canonical.last().expect("genesis always exists")
    }
    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.canonical.iter().map(|hash| &self.blocks[hash])
    }
    pub fn difficulty_bits(&self) -> u16 {
        self.difficulty_bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_of_work::mine_nonce;

    fn block(parent: Hash256, timestamp: u64) -> Block {
        Block::build(NetworkId::Regtest, parent, timestamp, 0, vec![]).unwrap()
    }

    fn mined(mut block: Block, bits: u16) -> Block {
        block.header.nonce = mine_nonce(&block.header, bits, 0, 1_000_000).unwrap();
        block
    }

    #[test]
    fn longer_valid_branch_becomes_canonical_and_reports_reorg_height() {
        let genesis = mined(block(Hash256::ZERO, 1), 0);
        let mut chain = ForkChain::new(genesis, 0).unwrap();
        let genesis_hash = chain.tip_hash();
        let a1 = mined(block(genesis_hash, 2), 0);
        let a1_hash = a1.header.hash().unwrap();
        assert_eq!(
            chain.submit(a1).unwrap(),
            ChainUpdate {
                became_best: true,
                reorg_from_height: None
            }
        );
        let b1 = mined(block(genesis_hash, 3), 0);
        let b1_hash = b1.header.hash().unwrap();
        assert!(!chain.submit(b1).unwrap().became_best);
        let b2 = mined(block(b1_hash, 4), 0);
        let update = chain.submit(b2).unwrap();
        assert_eq!(
            update,
            ChainUpdate {
                became_best: true,
                reorg_from_height: Some(1)
            }
        );
        assert_ne!(chain.tip_hash(), a1_hash);
        assert_eq!(chain.height(), 2);
    }

    #[test]
    fn rejects_unknown_parent_without_changing_canonical_chain() {
        let genesis = mined(block(Hash256::ZERO, 1), 0);
        let mut chain = ForkChain::new(genesis, 0).unwrap();
        let before = chain.tip_hash();
        assert_eq!(
            chain.submit(block(Hash256::from_bytes([7; 32]), 2)),
            Err(ForkChainError::UnknownParent)
        );
        assert_eq!(chain.tip_hash(), before);
    }

    #[test]
    fn equal_height_tie_break_is_deterministic() {
        let genesis = mined(block(Hash256::ZERO, 1), 0);
        let mut chain = ForkChain::new(genesis, 0).unwrap();
        let parent = chain.tip_hash();
        let left = mined(block(parent, 2), 0);
        let left_hash = left.header.hash().unwrap();
        chain.submit(left).unwrap();
        let right = mined(block(parent, 3), 0);
        let right_hash = right.header.hash().unwrap();
        let update = chain.submit(right).unwrap();
        assert_eq!(update.became_best, right_hash < left_hash);
        assert_eq!(chain.tip_hash(), left_hash.min(right_hash));
    }
}

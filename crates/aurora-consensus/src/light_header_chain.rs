//! Header-only consensus verification for experimental SPV clients.
use std::collections::BTreeMap;

use aurora_core::{
    hash::Hash256,
    network::NetworkId,
};

use crate::{
    difficulty::{ChainWork, DifficultyParams},
    experimental::{BlockHeader, HEADER_VERSION},
    proof_of_work::{check_proof, PowError},
};

const MAX_HEADER_NODES: usize = 1_000_000;

#[derive(Clone, Copy, Debug)]
struct HeaderNode {
    parent: Hash256,
    height: u64,
    timestamp: u64,
    difficulty_bits: u16,
    chain_work: ChainWork,
}

#[derive(Clone, Debug)]
pub struct LightHeaderChain {
    network: NetworkId,
    initial_difficulty_bits: u16,
    difficulty_params: DifficultyParams,
    nodes: BTreeMap<Hash256, HeaderNode>,
    headers: BTreeMap<Hash256, BlockHeader>,
    canonical: Vec<Hash256>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderChainError {
    InvalidGenesis,
    InvalidDifficultyParams,
    UnknownParent,
    DuplicateHeader,
    TooManyHeaders,
    HeightOverflow,
    InvalidVersionOrNetwork,
    InvalidTimestamp,
    InvalidDifficulty,
    InvalidProof(PowError),
    InsufficientWork,
    WorkOverflow,
    Encoding,
}

impl LightHeaderChain {
    pub fn new(
        genesis: BlockHeader,
        expected_genesis_hash: Hash256,
        initial_difficulty_bits: u16,
        difficulty_params: DifficultyParams,
    ) -> Result<Self, HeaderChainError> {
        if genesis.prev_hash != Hash256::ZERO
            || genesis.hash().map_err(|_| HeaderChainError::Encoding)? != expected_genesis_hash
            || !difficulty_params.validate(initial_difficulty_bits)
        {
            return Err(if !difficulty_params.validate(initial_difficulty_bits) {
                HeaderChainError::InvalidDifficultyParams
            } else {
                HeaderChainError::InvalidGenesis
            });
        }
        if !check_proof(&genesis, initial_difficulty_bits).map_err(HeaderChainError::InvalidProof)? {
            return Err(HeaderChainError::InsufficientWork);
        }
        let network = genesis.network;
        let mut nodes = BTreeMap::new();
        nodes.insert(
            expected_genesis_hash,
            HeaderNode {
                parent: Hash256::ZERO,
                height: 0,
                timestamp: genesis.timestamp,
                difficulty_bits: initial_difficulty_bits,
                chain_work: ChainWork::for_difficulty(initial_difficulty_bits),
            },
        );
        let mut headers = BTreeMap::new();
        headers.insert(expected_genesis_hash, genesis);
        Ok(Self {
            network,
            initial_difficulty_bits,
            difficulty_params,
            nodes,
            headers,
            canonical: vec![expected_genesis_hash],
        })
    }

    /// Validate a header and retain it even when it extends a secondary branch.
    pub fn submit(&mut self, header: BlockHeader) -> Result<bool, HeaderChainError> {
        if header.version != HEADER_VERSION || header.network != self.network {
            return Err(HeaderChainError::InvalidVersionOrNetwork);
        }
        if self.nodes.len() >= MAX_HEADER_NODES {
            return Err(HeaderChainError::TooManyHeaders);
        }
        let parent_hash = header.prev_hash;
        let parent = *self
            .nodes
            .get(&parent_hash)
            .ok_or(HeaderChainError::UnknownParent)?;
        let hash = header.hash().map_err(|_| HeaderChainError::Encoding)?;
        if self.nodes.contains_key(&hash) {
            return Err(HeaderChainError::DuplicateHeader);
        }
        let height = parent
            .height
            .checked_add(1)
            .ok_or(HeaderChainError::HeightOverflow)?;
        if header.timestamp <= parent.timestamp || header.timestamp <= self.median_time(parent_hash)
        {
            return Err(HeaderChainError::InvalidTimestamp);
        }
        let expected_bits = if self.difficulty_params.adjustment_interval > 0
            && height % self.difficulty_params.adjustment_interval == 0
        {
            let anchor_height = height - self.difficulty_params.adjustment_interval;
            let anchor_hash = self.ancestor_at_height(parent_hash, anchor_height)?;
            let elapsed = header
                .timestamp
                .saturating_sub(self.nodes[&anchor_hash].timestamp)
                .max(1);
            self.difficulty_params.retarget(parent.difficulty_bits, elapsed)
        } else {
            parent.difficulty_bits
        };
        if !check_proof(&header, expected_bits).map_err(HeaderChainError::InvalidProof)? {
            return Err(HeaderChainError::InsufficientWork);
        }
        let work = parent
            .chain_work
            .checked_add(ChainWork::for_difficulty(expected_bits))
            .ok_or(HeaderChainError::WorkOverflow)?;
        let mut nodes = self.nodes.clone();
        let mut headers = self.headers.clone();
        nodes.insert(
            hash,
            HeaderNode {
                parent: parent_hash,
                height,
                timestamp: header.timestamp,
                difficulty_bits: expected_bits,
                chain_work: work,
            },
        );
        headers.insert(hash, header);
        let current_tip = *self.canonical.last().expect("genesis exists");
        let current = self.nodes[&current_tip];
        if work <= current.chain_work
            && (work != current.chain_work || hash >= current_tip)
        {
            self.nodes = nodes;
            self.headers = headers;
            return Ok(false);
        }
        let mut path = Vec::with_capacity(height as usize + 1);
        let mut cursor = hash;
        loop {
            path.push(cursor);
            let node = nodes.get(&cursor).expect("inserted header path is complete");
            if node.height == 0 {
                break;
            }
            cursor = node.parent;
        }
        path.reverse();
        self.nodes = nodes;
        self.headers = headers;
        self.canonical = path;
        Ok(true)
    }

    fn ancestor_at_height(
        &self,
        mut hash: Hash256,
        target_height: u64,
    ) -> Result<Hash256, HeaderChainError> {
        loop {
            let node = self.nodes.get(&hash).ok_or(HeaderChainError::UnknownParent)?;
            if node.height == target_height {
                return Ok(hash);
            }
            if node.height < target_height {
                return Err(HeaderChainError::UnknownParent);
            }
            hash = node.parent;
        }
    }

    fn median_time(&self, mut hash: Hash256) -> u64 {
        let mut timestamps = Vec::with_capacity(11);
        loop {
            let node = self.nodes.get(&hash).expect("known ancestor");
            timestamps.push(node.timestamp);
            if timestamps.len() == 11 || node.height == 0 {
                break;
            }
            hash = node.parent;
        }
        timestamps.sort_unstable();
        timestamps[timestamps.len() / 2]
    }

    pub fn height(&self) -> u64 {
        (self.canonical.len() - 1) as u64
    }

    pub fn tip_hash(&self) -> Hash256 {
        *self.canonical.last().expect("genesis exists")
    }

    pub fn tip_header(&self) -> BlockHeader {
        self.headers[&self.tip_hash()]
    }

    pub fn chain_work(&self) -> ChainWork {
        self.nodes[&self.tip_hash()].chain_work
    }

    pub fn header_at_height(&self, height: u64) -> Option<BlockHeader> {
        self.canonical
            .get(usize::try_from(height).ok()?)
            .and_then(|hash| self.headers.get(hash))
            .copied()
    }

    pub fn is_canonical(&self, hash: Hash256) -> bool {
        self.nodes.get(&hash).is_some_and(|node| {
            self.canonical
                .get(node.height as usize)
                .is_some_and(|canonical| *canonical == hash)
        })
    }

    /// Confirmation count includes the containing block itself.
    pub fn confirmations(&self, hash: Hash256) -> Option<u64> {
        let node = self.nodes.get(&hash)?;
        self.is_canonical(hash)
            .then_some(self.height().saturating_sub(node.height).saturating_add(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        account_block::{AccountBlock, check_account_block_pow},
        proof_of_work::mine_nonce,
        testnet::{testnet_genesis_block, TESTNET_DIFFICULTY, TESTNET_GENESIS_HASH, TESTNET_INITIAL_BITS},
    };
    use aurora_core::accounts::AccountTransaction;

    fn child(parent: Hash256, timestamp: u64, bits: u16) -> BlockHeader {
        let mut block =
            AccountBlock::build(NetworkId::Testnet, parent, timestamp, 0, Vec::<AccountTransaction>::new())
                .unwrap();
        block.header.nonce = mine_nonce(&block.header, bits, 0, u64::MAX).unwrap();
        block.header
    }

    #[test]
    fn verifies_canonical_header_sequence_and_inclusion_depth() {
        let genesis = testnet_genesis_block();
        let mut chain = LightHeaderChain::new(
            genesis.header,
            TESTNET_GENESIS_HASH,
            TESTNET_INITIAL_BITS,
            TESTNET_DIFFICULTY,
        )
        .unwrap();
        let first = child(TESTNET_GENESIS_HASH, 61, TESTNET_INITIAL_BITS);
        assert!(chain.submit(first).unwrap());
        assert_eq!(chain.height(), 1);
        assert_eq!(chain.confirmations(first.hash().unwrap()), Some(1));
        let second = child(first.hash().unwrap(), 121, TESTNET_INITIAL_BITS);
        assert!(chain.submit(second).unwrap());
        assert_eq!(chain.confirmations(first.hash().unwrap()), Some(2));
    }

    #[test]
    fn enforces_retarget_timestamps_and_rejects_bad_genesis() {
        let genesis = testnet_genesis_block();
        let mut chain = LightHeaderChain::new(
            genesis.header,
            TESTNET_GENESIS_HASH,
            TESTNET_INITIAL_BITS,
            TESTNET_DIFFICULTY,
        )
        .unwrap();
        let mut parent = TESTNET_GENESIS_HASH;
        for (height, timestamp) in [(1, 61), (2, 121), (3, 181), (4, 241), (5, 301), (6, 361), (7, 421), (8, 481), (9, 541)] {
            let header = child(parent, timestamp, TESTNET_INITIAL_BITS);
            parent = header.hash().unwrap();
            chain.submit(header).unwrap();
            assert_eq!(height, chain.height());
        }
        let wrong_bits = child(parent, 10_000, TESTNET_INITIAL_BITS);
        assert_eq!(chain.submit(wrong_bits), Err(HeaderChainError::InsufficientWork));
        let mut bad_genesis = genesis.header;
        bad_genesis.timestamp += 1;
        assert_eq!(
            LightHeaderChain::new(bad_genesis, TESTNET_GENESIS_HASH, TESTNET_INITIAL_BITS, TESTNET_DIFFICULTY),
            Err(HeaderChainError::InvalidGenesis)
        );
    }

    #[test]
    fn rejects_headers_that_do_not_advance_parent_time() {
        let genesis = testnet_genesis_block();
        let mut chain = LightHeaderChain::new(
            genesis.header,
            TESTNET_GENESIS_HASH,
            TESTNET_INITIAL_BITS,
            TESTNET_DIFFICULTY,
        )
        .unwrap();
        let bad = child(TESTNET_GENESIS_HASH, 1, TESTNET_INITIAL_BITS);
        assert_eq!(chain.submit(bad), Err(HeaderChainError::InvalidTimestamp));
        assert_eq!(chain.height(), 0);
    }
}

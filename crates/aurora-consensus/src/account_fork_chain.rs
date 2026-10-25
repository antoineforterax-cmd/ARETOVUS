//! Sélection expérimentale de branches pour les blocs par comptes, difficulté fixe.
use std::{collections::BTreeMap, fmt};

use crate::{
    account_block::{validate_and_apply_account_block, AccountBlock, AccountBlockValidationError},
    difficulty::{ChainWork, DifficultyParams},
    fork_chain::ChainUpdate,
};
use aurora_core::{codec::EncodeError, hash::Hash256, state::AccountLedger};

#[derive(Clone, Copy, Debug)]
struct Node {
    parent: Hash256,
    height: u64,
    timestamp: u64,
    difficulty_bits: u16,
    chain_work: ChainWork,
}

#[derive(Clone, Debug)]
pub struct AccountForkChain {
    initial_difficulty_bits: u16,
    difficulty_params: DifficultyParams,
    nodes: BTreeMap<Hash256, Node>,
    blocks: BTreeMap<Hash256, AccountBlock>,
    states: BTreeMap<Hash256, AccountLedger>,
    canonical: Vec<Hash256>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountForkChainError {
    InvalidGenesisParent,
    UnknownParent,
    DuplicateBlock,
    HeightOverflow,
    InvalidDifficultyParams,
    InvalidTimestamp,
    WorkOverflow,
    Validation(AccountBlockValidationError),
    Encoding(EncodeError),
}
impl fmt::Display for AccountForkChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AccountForkChainError {}

impl AccountForkChain {
    pub fn new(
        genesis: AccountBlock,
        difficulty_bits: u16,
        initial_state: AccountLedger,
    ) -> Result<Self, AccountForkChainError> {
        Self::new_with_params(
            genesis,
            difficulty_bits,
            DifficultyParams::fixed(difficulty_bits),
            initial_state,
        )
    }

    pub fn new_with_params(
        genesis: AccountBlock,
        initial_difficulty_bits: u16,
        difficulty_params: DifficultyParams,
        mut initial_state: AccountLedger,
    ) -> Result<Self, AccountForkChainError> {
        if genesis.header.prev_hash != Hash256::ZERO {
            return Err(AccountForkChainError::InvalidGenesisParent);
        }
        if !difficulty_params.validate(initial_difficulty_bits) {
            return Err(AccountForkChainError::InvalidDifficultyParams);
        }
        validate_and_apply_account_block(&genesis, initial_difficulty_bits, &mut initial_state)
            .map_err(AccountForkChainError::Validation)?;
        let hash = genesis
            .header
            .hash()
            .map_err(AccountForkChainError::Encoding)?;
        let mut nodes = BTreeMap::new();
        nodes.insert(
            hash,
            Node {
                parent: Hash256::ZERO,
                height: 0,
                timestamp: genesis.header.timestamp,
                difficulty_bits: initial_difficulty_bits,
                chain_work: ChainWork::for_difficulty(initial_difficulty_bits),
            },
        );
        let mut blocks = BTreeMap::new();
        blocks.insert(hash, genesis);
        let mut states = BTreeMap::new();
        states.insert(hash, initial_state);
        Ok(Self {
            initial_difficulty_bits,
            difficulty_params,
            nodes,
            blocks,
            states,
            canonical: vec![hash],
        })
    }

    /// Valide un bloc contre l'état de son parent; garde l'état calculé même pour une branche secondaire.
    pub fn submit(&mut self, block: AccountBlock) -> Result<ChainUpdate, AccountForkChainError> {
        let parent = block.header.prev_hash;
        let parent_node = *self
            .nodes
            .get(&parent)
            .ok_or(AccountForkChainError::UnknownParent)?;
        let hash = block
            .header
            .hash()
            .map_err(AccountForkChainError::Encoding)?;
        if self.nodes.contains_key(&hash) {
            return Err(AccountForkChainError::DuplicateBlock);
        }
        let height = parent_node
            .height
            .checked_add(1)
            .ok_or(AccountForkChainError::HeightOverflow)?;
        if block.header.timestamp <= self.median_time(parent) {
            return Err(AccountForkChainError::InvalidTimestamp);
        }
        let expected_bits = if self.difficulty_params.adjustment_interval > 0
            && height % self.difficulty_params.adjustment_interval == 0
        {
            let anchor_height = height - self.difficulty_params.adjustment_interval;
            let anchor_hash = self.ancestor_at_height(parent, anchor_height)?;
            let elapsed = self
                .median_time(parent)
                .saturating_sub(self.median_time(anchor_hash))
                .max(1);
            self.difficulty_params
                .retarget(parent_node.difficulty_bits, elapsed)
        } else {
            parent_node.difficulty_bits
        };
        let mut state = self
            .states
            .get(&parent)
            .expect("every known block has a state snapshot")
            .clone();
        validate_and_apply_account_block(&block, expected_bits, &mut state)
            .map_err(AccountForkChainError::Validation)?;
        let chain_work = parent_node
            .chain_work
            .checked_add(ChainWork::for_difficulty(expected_bits))
            .ok_or(AccountForkChainError::WorkOverflow)?;
        self.nodes.insert(
            hash,
            Node {
                parent,
                height,
                timestamp: block.header.timestamp,
                difficulty_bits: expected_bits,
                chain_work,
            },
        );
        self.blocks.insert(hash, block);
        self.states.insert(hash, state);

        let current_tip = *self.canonical.last().expect("genesis always exists");
        let current = self.nodes[&current_tip];
        if chain_work < current.chain_work
            || (chain_work == current.chain_work && hash >= current_tip)
        {
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

    fn ancestor_at_height(
        &self,
        mut hash: Hash256,
        height: u64,
    ) -> Result<Hash256, AccountForkChainError> {
        loop {
            let node = self
                .nodes
                .get(&hash)
                .ok_or(AccountForkChainError::UnknownParent)?;
            if node.height == height {
                return Ok(hash);
            }
            if node.height < height {
                return Err(AccountForkChainError::InvalidDifficultyParams);
            }
            hash = node.parent;
        }
    }

    fn median_time(&self, mut hash: Hash256) -> u64 {
        let mut times = Vec::with_capacity(11);
        loop {
            let node = self.nodes[&hash];
            times.push(node.timestamp);
            if times.len() == 11 || node.height == 0 {
                break;
            }
            hash = node.parent;
        }
        times.sort_unstable();
        times[times.len() / 2]
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

    pub fn difficulty_bits(&self) -> u16 {
        self.nodes[&self.tip_hash()].difficulty_bits
    }

    pub fn initial_difficulty_bits(&self) -> u16 {
        self.initial_difficulty_bits
    }

    pub fn difficulty_params(&self) -> DifficultyParams {
        self.difficulty_params
    }

    pub fn chain_work(&self) -> ChainWork {
        self.nodes[&self.tip_hash()].chain_work
    }

    pub fn height(&self) -> u64 {
        (self.canonical.len() - 1) as u64
    }
    pub fn tip_hash(&self) -> Hash256 {
        *self.canonical.last().expect("genesis always exists")
    }
    pub fn state(&self) -> &AccountLedger {
        &self.states[&self.tip_hash()]
    }
    pub fn blocks(&self) -> impl Iterator<Item = &AccountBlock> {
        self.canonical.iter().map(|hash| &self.blocks[hash])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::{
        accounts::{AccountId, AccountTransaction, ACCOUNT_TX_VERSION},
        amount::Amount,
        network::NetworkId,
    };
    use ed25519_dalek::{Signer, SigningKey};

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[6; 32])
    }
    fn account(byte: u8) -> AccountId {
        AccountId([byte; 32])
    }
    fn tx(nonce: u64, recipient: u8, amount: u64) -> AccountTransaction {
        let signing_key = key();
        let mut tx = AccountTransaction {
            version: ACCOUNT_TX_VERSION,
            network: NetworkId::Regtest,
            sender: AccountId(signing_key.verifying_key().to_bytes()),
            nonce,
            recipient: account(recipient),
            amount: Amount::from_units(amount).unwrap(),
            fee: Amount::from_units(1).unwrap(),
            auth_data: Vec::new(),
        };
        tx.auth_data = signing_key
            .sign(tx.signing_digest().unwrap().as_bytes())
            .to_bytes()
            .to_vec();
        tx
    }
    fn block(parent: Hash256, timestamp: u64, txs: Vec<AccountTransaction>) -> AccountBlock {
        AccountBlock::build(NetworkId::Regtest, parent, timestamp, 0, txs).unwrap()
    }
    fn initial_state() -> AccountLedger {
        let mut state = AccountLedger::new(NetworkId::Regtest);
        state
            .credit_issuance(
                AccountId(key().verifying_key().to_bytes()),
                Amount::from_units(1_000).unwrap(),
            )
            .unwrap();
        state
    }

    #[test]
    fn reorganization_also_switches_the_canonical_account_state() {
        let genesis = block(Hash256::ZERO, 1, vec![]);
        let mut chain = AccountForkChain::new(genesis, 0, initial_state()).unwrap();
        let genesis_hash = chain.tip_hash();
        let branch_a = block(genesis_hash, 2, vec![tx(0, 10, 10)]);
        let hash_a = branch_a.header.hash().unwrap();
        assert!(chain.submit(branch_a).unwrap().became_best);

        let branch_b = (3..100)
            .map(|time| block(genesis_hash, time, vec![tx(0, 11, 20)]))
            .find(|candidate| candidate.header.hash().unwrap() > hash_a)
            .expect("deterministic candidate with a higher hash exists");
        let hash_b = branch_b.header.hash().unwrap();
        assert!(!chain.submit(branch_b).unwrap().became_best);
        assert_eq!(chain.state().account(account(10)).balance.to_units(), 10);
        assert_eq!(chain.state().account(account(11)).balance.to_units(), 0);

        let branch_b2 = block(hash_b, 100, vec![tx(1, 12, 5)]);
        let update = chain.submit(branch_b2).unwrap();
        assert_eq!(
            update,
            ChainUpdate {
                became_best: true,
                reorg_from_height: Some(1)
            }
        );
        assert_eq!(chain.height(), 2);
        assert_eq!(chain.state().account(account(10)).balance.to_units(), 0);
        assert_eq!(chain.state().account(account(11)).balance.to_units(), 20);
        assert_eq!(chain.state().account(account(12)).balance.to_units(), 5);
        assert_eq!(chain.state().total_issued().to_units(), 1_000);
    }

    #[test]
    fn retarget_is_enforced_and_canonical_work_is_tracked() {
        use crate::proof_of_work::{check_proof, mine_nonce};
        let params = DifficultyParams {
            target_block_seconds: 10,
            adjustment_interval: 2,
            max_adjustment_bits: 2,
            min_bits: 1,
            max_bits: 8,
        };
        let genesis = block(Hash256::ZERO, 100, vec![]);
        let mut chain =
            AccountForkChain::new_with_params(genesis, 2, params, initial_state()).unwrap();
        let first = block(chain.tip_hash(), 108, vec![]);
        let first_nonce = mine_nonce(&first.header, 2, 0, 100_000).unwrap();
        let mut first = first;
        first.header.nonce = first_nonce;
        chain.submit(first).unwrap();
        assert_eq!(chain.difficulty_bits(), 2);

        let second = block(chain.tip_hash(), 120, vec![]);
        let mut rejected = None;
        for nonce in 0..100_000 {
            let mut candidate = second.clone();
            candidate.header.nonce = nonce;
            if check_proof(&candidate.header, 2).unwrap()
                && !check_proof(&candidate.header, 3).unwrap()
            {
                rejected = Some(candidate);
                break;
            }
        }
        assert!(matches!(
            chain.submit(rejected.unwrap()),
            Err(AccountForkChainError::Validation(
                AccountBlockValidationError::InsufficientWork
            ))
        ));

        let second = block(chain.tip_hash(), 120, vec![]);
        let nonce = mine_nonce(&second.header, 3, 0, 100_000).unwrap();
        let mut second = second;
        second.header.nonce = nonce;
        chain.submit(second).unwrap();
        assert_eq!(chain.height(), 2);
        assert_eq!(chain.difficulty_bits(), 3);
        assert_eq!(
            chain.chain_work(),
            ChainWork::for_difficulty(2)
                .checked_add(ChainWork::for_difficulty(2))
                .unwrap()
                .checked_add(ChainWork::for_difficulty(3))
                .unwrap()
        );
    }

    #[test]
    fn child_state_rejects_nonce_replay_against_its_parent() {
        let genesis = block(Hash256::ZERO, 1, vec![]);
        let mut chain = AccountForkChain::new(genesis, 0, initial_state()).unwrap();
        let parent = chain.tip_hash();
        let first = block(parent, 2, vec![tx(0, 10, 10)]);
        let first_hash = first.header.hash().unwrap();
        chain.submit(first).unwrap();
        let invalid_child = block(first_hash, 3, vec![tx(0, 11, 20)]);
        assert!(matches!(
            chain.submit(invalid_child),
            Err(AccountForkChainError::Validation(
                AccountBlockValidationError::State(_)
            ))
        ));
        assert_eq!(chain.height(), 1);
    }
}

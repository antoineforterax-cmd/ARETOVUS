//! Sélection expérimentale de branches pour les blocs par comptes, difficulté fixe.
use std::{collections::BTreeMap, fmt};

use crate::{
    account_block::{validate_and_apply_account_block, AccountBlock, AccountBlockValidationError},
    fork_chain::ChainUpdate,
};
use aurora_core::{codec::EncodeError, hash::Hash256, state::AccountLedger};

#[derive(Clone, Copy, Debug)]
struct Node {
    parent: Hash256,
    height: u64,
}

#[derive(Clone, Debug)]
pub struct AccountForkChain {
    difficulty_bits: u16,
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
        mut genesis: AccountBlock,
        difficulty_bits: u16,
        mut initial_state: AccountLedger,
    ) -> Result<Self, AccountForkChainError> {
        if genesis.header.prev_hash != Hash256::ZERO {
            return Err(AccountForkChainError::InvalidGenesisParent);
        }
        validate_and_apply_account_block(&genesis, difficulty_bits, &mut initial_state)
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
            },
        );
        let mut blocks = BTreeMap::new();
        blocks.insert(hash, genesis);
        let mut states = BTreeMap::new();
        states.insert(hash, initial_state);
        Ok(Self {
            difficulty_bits,
            nodes,
            blocks,
            states,
            canonical: vec![hash],
        })
    }

    /// Valide un bloc contre l'état de son parent; garde l'état calculé même pour une branche secondaire.
    pub fn submit(&mut self, block: AccountBlock) -> Result<ChainUpdate, AccountForkChainError> {
        let parent = block.header.prev_hash;
        let parent_node = self
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
        let mut state = self
            .states
            .get(&parent)
            .expect("every known block has a state snapshot")
            .clone();
        validate_and_apply_account_block(&block, self.difficulty_bits, &mut state)
            .map_err(AccountForkChainError::Validation)?;
        self.nodes.insert(hash, Node { parent, height });
        self.blocks.insert(hash, block);
        self.states.insert(hash, state);

        let current_tip = *self.canonical.last().expect("genesis always exists");
        let current_height = self.nodes[&current_tip].height;
        if height < current_height || (height == current_height && hash >= current_tip) {
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

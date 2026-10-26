//! Sélection expérimentale de branches pour les blocs par comptes, difficulté fixe.
use std::{collections::BTreeMap, fmt};

use crate::{
    account_block::{validate_and_apply_account_block, AccountBlock, AccountBlockValidationError},
    difficulty::{ChainWork, DifficultyParams},
    fork_chain::ChainUpdate,
    reward::{apply_block_reward, RewardError, RewardParams},
};
use aurora_core::{
    accounts::POST_QUANTUM_ACCOUNT_TX_VERSION, codec::EncodeError, hash::Hash256,
    network::NetworkId, state::AccountLedger,
};

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
    reward_params: RewardParams,
    nodes: BTreeMap<Hash256, Node>,
    blocks: BTreeMap<Hash256, AccountBlock>,
    ordered: Vec<Hash256>,
    states: BTreeMap<Hash256, AccountLedger>,
    canonical: Vec<Hash256>,
    canonical_tx_index: BTreeMap<Hash256, (u64, usize)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountForkChainError {
    InvalidGenesisParent,
    InvalidGenesisBody,
    UnknownParent,
    DuplicateBlock,
    HeightOverflow,
    InvalidDifficultyParams,
    InvalidTimestamp,
    InvalidTransactionProfile,
    WorkOverflow,
    Validation(AccountBlockValidationError),
    Reward(RewardError),
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
        Self::new_with_protocol_params(
            genesis,
            difficulty_bits,
            DifficultyParams::fixed(difficulty_bits),
            RewardParams::disabled(),
            initial_state,
        )
    }

    pub fn new_with_params(
        genesis: AccountBlock,
        initial_difficulty_bits: u16,
        difficulty_params: DifficultyParams,
        initial_state: AccountLedger,
    ) -> Result<Self, AccountForkChainError> {
        Self::new_with_protocol_params(
            genesis,
            initial_difficulty_bits,
            difficulty_params,
            RewardParams::disabled(),
            initial_state,
        )
    }

    pub fn new_with_protocol_params(
        genesis: AccountBlock,
        initial_difficulty_bits: u16,
        difficulty_params: DifficultyParams,
        reward_params: RewardParams,
        mut initial_state: AccountLedger,
    ) -> Result<Self, AccountForkChainError> {
        if genesis.header.prev_hash != Hash256::ZERO {
            return Err(AccountForkChainError::InvalidGenesisParent);
        }
        if genesis.miner.is_some() || !genesis.transactions.is_empty() {
            return Err(AccountForkChainError::InvalidGenesisBody);
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
            reward_params,
            nodes,
            blocks,
            ordered: vec![hash],
            states,
            canonical: vec![hash],
            canonical_tx_index: BTreeMap::new(),
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
        // Each branch has strictly increasing timestamps, and every block must also
        // advance beyond the median of up to eleven prior blocks.
        if block.header.timestamp <= parent_node.timestamp
            || block.header.timestamp <= self.median_time(parent)
        {
            return Err(AccountForkChainError::InvalidTimestamp);
        }
        let expected_bits = if self.difficulty_params.adjustment_interval > 0
            && height % self.difficulty_params.adjustment_interval == 0
        {
            let anchor_height = height - self.difficulty_params.adjustment_interval;
            let anchor_hash = self.ancestor_at_height(parent, anchor_height)?;
            // The candidate closes this window. Measuring its timestamp against the
            // exact ancestor H-interval covers exactly `interval` block spacings.
            let elapsed = block
                .header
                .timestamp
                .saturating_sub(self.nodes[&anchor_hash].timestamp)
                .max(1);
            self.difficulty_params
                .retarget(parent_node.difficulty_bits, elapsed)
        } else {
            parent_node.difficulty_bits
        };
        // Testnet v2 is a consensus-incompatible, native ML-DSA-only profile.
        // Regtest keeps legacy profiles available for compatibility and tooling.
        if self.states[&parent].network() == NetworkId::Testnet
            && block
                .transactions
                .iter()
                .any(|tx| tx.version != POST_QUANTUM_ACCOUNT_TX_VERSION)
        {
            return Err(AccountForkChainError::InvalidTransactionProfile);
        }
        let mut state = self
            .states
            .get(&parent)
            .expect("every known block has a state snapshot")
            .clone();
        let total_fees = validate_and_apply_account_block(&block, expected_bits, &mut state)
            .map_err(AccountForkChainError::Validation)?;
        apply_block_reward(&mut state, block.miner, total_fees, self.reward_params)
            .map_err(AccountForkChainError::Reward)?;
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
        let position = self
            .ordered
            .binary_search_by(|known| (self.nodes[known].height, *known).cmp(&(height, hash)))
            .unwrap_or_else(|position| position);
        self.ordered.insert(position, hash);

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
        for old_hash in self.canonical.iter().skip(common_len) {
            for tx in &self.blocks[old_hash].transactions {
                let txid = tx.txid().expect("validated transaction has a canonical id");
                self.canonical_tx_index.remove(&txid);
            }
        }
        for (height, new_hash) in candidate.iter().enumerate().skip(common_len) {
            for (index, tx) in self.blocks[new_hash].transactions.iter().enumerate() {
                let txid = tx.txid().expect("validated transaction has a canonical id");
                self.canonical_tx_index.insert(txid, (height as u64, index));
            }
        }
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

    /// Difficulty required for the next block on the canonical tip.
    pub fn next_difficulty_bits(
        &self,
        candidate_timestamp: u64,
    ) -> Result<u16, AccountForkChainError> {
        let parent = self.tip_hash();
        let node = self.nodes[&parent];
        let height = node
            .height
            .checked_add(1)
            .ok_or(AccountForkChainError::HeightOverflow)?;
        if self.difficulty_params.adjustment_interval > 0
            && height % self.difficulty_params.adjustment_interval == 0
        {
            let anchor_height = height - self.difficulty_params.adjustment_interval;
            let anchor = self.ancestor_at_height(parent, anchor_height)?;
            let elapsed = candidate_timestamp
                .saturating_sub(self.nodes[&anchor].timestamp)
                .max(1);
            Ok(self
                .difficulty_params
                .retarget(node.difficulty_bits, elapsed))
        } else {
            Ok(node.difficulty_bits)
        }
    }

    /// Timestamp at the canonical tip, used by the miner to preserve monotonicity.
    pub fn tip_timestamp(&self) -> u64 {
        self.nodes[&self.tip_hash()].timestamp
    }

    /// Median timestamp of up to eleven ancestors at the canonical tip.
    pub fn median_time_past(&self) -> u64 {
        self.median_time(self.tip_hash())
    }

    pub fn initial_difficulty_bits(&self) -> u16 {
        self.initial_difficulty_bits
    }

    pub fn difficulty_params(&self) -> DifficultyParams {
        self.difficulty_params
    }

    pub fn reward_params(&self) -> RewardParams {
        self.reward_params
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

    /// Locate a confirmed transaction on the selected branch without scanning all blocks.
    pub fn canonical_transaction(&self, txid: Hash256) -> Option<(u64, usize, &AccountBlock)> {
        let &(height, index) = self.canonical_tx_index.get(&txid)?;
        let hash = self.canonical.get(usize::try_from(height).ok()?)?;
        Some((height, index, self.blocks.get(hash)?))
    }

    /// All accepted branches in deterministic topological order.
    pub fn all_blocks_by_height(&self) -> Vec<&AccountBlock> {
        self.ordered.iter().map(|hash| &self.blocks[hash]).collect()
    }

    /// A bounded page without sorting or scanning all prior blocks.
    pub fn block_page(&self, start: usize, limit: usize) -> Vec<&AccountBlock> {
        if start >= self.ordered.len() {
            return Vec::new();
        }
        self.ordered[start..]
            .iter()
            .take(limit)
            .map(|hash| &self.blocks[hash])
            .collect()
    }

    pub fn known_block_count(&self) -> usize {
        self.blocks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_block::AccountBlockError;
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
        tx_with_fee(nonce, recipient, amount, 1)
    }
    fn tx_with_fee(nonce: u64, recipient: u8, amount: u64, fee: u64) -> AccountTransaction {
        let signing_key = key();
        let mut tx = AccountTransaction {
            version: ACCOUNT_TX_VERSION,
            network: NetworkId::Regtest,
            sender: AccountId(signing_key.verifying_key().to_bytes()),
            nonce,
            recipient: account(recipient),
            amount: Amount::from_units(amount).unwrap(),
            fee: Amount::from_units(fee).unwrap(),
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
    fn testnet_accepts_and_applies_a_native_ml_dsa_transaction() {
        use aurora_core::accounts::{
            ML_DSA_65_PUBLIC_KEY_BYTES, POST_QUANTUM_ACCOUNT_TX_VERSION, POST_QUANTUM_AUTH_BYTES,
        };
        use ml_dsa::{Keypair, MlDsa65, Signer as MlSigner, SigningKey as MlSigningKey};

        let key = MlSigningKey::<MlDsa65>::from_seed(&ml_dsa::Seed::default());
        let public_key = key.verifying_key().encode();
        let sender = aurora_core::signature::post_quantum_account_id(public_key.as_slice());
        let mut state = AccountLedger::new(NetworkId::Testnet);
        state
            .credit_issuance(sender, Amount::from_units(10).unwrap())
            .unwrap();
        let genesis = AccountBlock::build(NetworkId::Testnet, Hash256::ZERO, 1, 0, vec![]).unwrap();
        let mut chain = AccountForkChain::new(genesis, 0, state).unwrap();
        let mut transaction = AccountTransaction {
            version: POST_QUANTUM_ACCOUNT_TX_VERSION,
            network: NetworkId::Testnet,
            sender,
            nonce: 0,
            recipient: account(77),
            amount: Amount::from_units(4).unwrap(),
            fee: Amount::from_units(1).unwrap(),
            auth_data: vec![0; POST_QUANTUM_AUTH_BYTES],
        };
        transaction.auth_data[..ML_DSA_65_PUBLIC_KEY_BYTES].copy_from_slice(public_key.as_slice());
        let signature = key
            .sign(transaction.signing_digest().unwrap().as_bytes())
            .encode();
        transaction.auth_data[ML_DSA_65_PUBLIC_KEY_BYTES..].copy_from_slice(signature.as_slice());
        let block = AccountBlock::build(
            NetworkId::Testnet,
            chain.tip_hash(),
            2,
            0,
            vec![transaction],
        )
        .unwrap();

        chain.submit(block).unwrap();
        assert_eq!(chain.state().account(sender).balance.to_units(), 5);
        assert_eq!(chain.state().account(sender).next_nonce, 1);
        assert_eq!(chain.state().account(account(77)).balance.to_units(), 4);
    }

    #[test]
    fn testnet_rejects_legacy_ed25519_and_hybrid_transaction_profiles() {
        let genesis = AccountBlock::build(NetworkId::Testnet, Hash256::ZERO, 1, 0, vec![]).unwrap();
        let mut chain =
            AccountForkChain::new(genesis, 0, AccountLedger::new(NetworkId::Testnet)).unwrap();

        for legacy_version in [
            aurora_core::accounts::ACCOUNT_TX_VERSION,
            aurora_core::accounts::HYBRID_ACCOUNT_TX_VERSION,
        ] {
            let mut legacy = tx(0, 9, 1);
            legacy.network = NetworkId::Testnet;
            legacy.version = legacy_version;
            legacy.auth_data = if legacy_version == aurora_core::accounts::ACCOUNT_TX_VERSION {
                vec![0; 64]
            } else {
                vec![0; aurora_core::accounts::HYBRID_AUTH_BYTES]
            };
            let block =
                AccountBlock::build(NetworkId::Testnet, chain.tip_hash(), 2, 0, vec![legacy])
                    .unwrap();
            assert_eq!(
                chain.submit(block),
                Err(AccountForkChainError::InvalidTransactionProfile)
            );
        }
    }

    #[test]
    fn reorganization_also_switches_the_canonical_account_state() {
        let genesis = block(Hash256::ZERO, 1, vec![]);
        let mut chain = AccountForkChain::new(genesis, 0, initial_state()).unwrap();
        let genesis_hash = chain.tip_hash();
        let branch_a = block(genesis_hash, 2, vec![tx(0, 10, 10)]);
        let tx_a = branch_a.transactions[0].txid().unwrap();
        let hash_a = branch_a.header.hash().unwrap();
        assert!(chain.submit(branch_a).unwrap().became_best);
        assert_eq!(chain.canonical_transaction(tx_a).unwrap().0, 1);

        let branch_b = (3..100)
            .map(|time| block(genesis_hash, time, vec![tx(0, 11, 20)]))
            .find(|candidate| candidate.header.hash().unwrap() > hash_a)
            .expect("deterministic candidate with a higher hash exists");
        let tx_b = branch_b.transactions[0].txid().unwrap();
        let hash_b = branch_b.header.hash().unwrap();
        assert!(!chain.submit(branch_b).unwrap().became_best);
        assert!(chain.canonical_transaction(tx_b).is_none());
        assert_eq!(chain.state().account(account(10)).balance.to_units(), 10);
        assert_eq!(chain.state().account(account(11)).balance.to_units(), 0);

        let branch_b2 = block(hash_b, 100, vec![tx(1, 12, 5)]);
        let tx_b2 = branch_b2.transactions[0].txid().unwrap();
        let update = chain.submit(branch_b2).unwrap();
        assert_eq!(
            update,
            ChainUpdate {
                became_best: true,
                reorg_from_height: Some(1)
            }
        );
        assert_eq!(chain.height(), 2);
        assert!(chain.canonical_transaction(tx_a).is_none());
        assert_eq!(chain.canonical_transaction(tx_b).unwrap().0, 1);
        assert_eq!(chain.canonical_transaction(tx_b2).unwrap().0, 2);
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
        let mut genesis = block(Hash256::ZERO, 100, vec![]);
        genesis.header.nonce = mine_nonce(&genesis.header, 2, 0, 100_000).unwrap();
        let mut chain =
            AccountForkChain::new_with_params(genesis, 2, params, initial_state()).unwrap();
        let first = block(chain.tip_hash(), 101, vec![]);
        let first_nonce = mine_nonce(&first.header, 2, 0, 100_000).unwrap();
        let mut first = first;
        first.header.nonce = first_nonce;
        chain.submit(first).unwrap();
        assert_eq!(chain.difficulty_bits(), 2);

        let second = block(chain.tip_hash(), 102, vec![]);
        assert_eq!(chain.next_difficulty_bits(102).unwrap(), 4);
        let mut rejected = None;
        for nonce in 0..100_000 {
            let mut candidate = second.clone();
            candidate.header.nonce = nonce;
            if check_proof(&candidate.header, 3).unwrap()
                && !check_proof(&candidate.header, 4).unwrap()
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

        let second = block(chain.tip_hash(), 102, vec![]);
        let nonce = mine_nonce(&second.header, 4, 0, 100_000).unwrap();
        let mut second = second;
        second.header.nonce = nonce;
        chain.submit(second).unwrap();
        assert_eq!(chain.height(), 2);
        assert_eq!(chain.difficulty_bits(), 4);
        assert_eq!(
            chain.chain_work(),
            ChainWork::for_difficulty(2)
                .checked_add(ChainWork::for_difficulty(2))
                .unwrap()
                .checked_add(ChainWork::for_difficulty(4))
                .unwrap()
        );
    }

    #[test]
    fn timestamps_must_advance_parent_and_retarget_uses_candidate_time_bounded() {
        use crate::proof_of_work::mine_nonce;
        let params = DifficultyParams {
            target_block_seconds: 10,
            adjustment_interval: 2,
            max_adjustment_bits: 1,
            min_bits: 1,
            max_bits: 8,
        };
        let mut genesis = block(Hash256::ZERO, 100, vec![]);
        genesis.header.nonce = mine_nonce(&genesis.header, 2, 0, 100_000).unwrap();
        let mut chain =
            AccountForkChain::new_with_params(genesis, 2, params, initial_state()).unwrap();
        let mut first = block(chain.tip_hash(), 101, vec![]);
        first.header.nonce = mine_nonce(&first.header, 2, 0, 100_000).unwrap();
        let first_hash = first.header.hash().unwrap();
        chain.submit(first).unwrap();

        // Equality with the direct parent is invalid even if it is above the MTP.
        assert_eq!(
            chain.submit(block(first_hash, 101, vec![])),
            Err(AccountForkChainError::InvalidTimestamp)
        );
        // A far-future candidate can move difficulty by only one bit in this window.
        assert_eq!(chain.next_difficulty_bits(1_000_000).unwrap(), 1);
        assert_eq!(chain.next_difficulty_bits(102).unwrap(), 3);
    }

    #[test]
    fn block_reward_splits_issuance_fees_and_enforces_miner_commitment() {
        let treasury = account(51);
        let miner = account(50);
        let genesis = block(Hash256::ZERO, 1, vec![]);
        let mut chain = AccountForkChain::new_with_protocol_params(
            genesis,
            0,
            DifficultyParams::fixed(0),
            RewardParams::experimental_testnet(treasury),
            initial_state(),
        )
        .unwrap();
        let candidate = AccountBlock::build_with_miner(
            NetworkId::Regtest,
            chain.tip_hash(),
            2,
            0,
            Some(miner),
            vec![tx_with_fee(0, 10, 10, 5)],
        )
        .unwrap();
        let mut tampered = candidate.clone();
        tampered.miner = Some(account(52));
        assert!(matches!(
            chain.submit(tampered),
            Err(AccountForkChainError::Validation(
                AccountBlockValidationError::Stateless(AccountBlockError::CommitmentMismatch)
            ))
        ));
        let issuance = crate::reward::scheduled_issuance(chain.state().total_issued()).to_units();
        let treasury_issuance = issuance / 20;
        chain.submit(candidate).unwrap();
        assert_eq!(
            chain.state().account(miner).balance.to_units(),
            issuance - treasury_issuance + 2
        );
        assert_eq!(
            chain.state().account(treasury).balance.to_units(),
            treasury_issuance
        );
        assert_eq!(chain.state().total_issued().to_units(), 1_000 + issuance);
    }

    #[test]
    fn reorganization_replays_rewards_from_the_winning_parent_state() {
        let treasury = account(60);
        let miner_a = account(61);
        let miner_b = account(62);
        let miner_c = account(63);
        let genesis = block(Hash256::ZERO, 1, vec![]);
        let mut chain = AccountForkChain::new_with_protocol_params(
            genesis,
            0,
            DifficultyParams::fixed(0),
            RewardParams::experimental_testnet(treasury),
            AccountLedger::new(NetworkId::Regtest),
        )
        .unwrap();
        let parent = chain.tip_hash();
        let first_issuance = crate::reward::scheduled_issuance(Amount::ZERO).to_units();
        let second_issuance =
            crate::reward::scheduled_issuance(Amount::from_units(first_issuance).unwrap())
                .to_units();
        let parent_miner_issuance = first_issuance - first_issuance / 20;
        let child_miner_issuance = second_issuance - second_issuance / 20;
        let branch_a =
            AccountBlock::build_with_miner(NetworkId::Regtest, parent, 2, 0, Some(miner_a), vec![])
                .unwrap();
        let hash_a = branch_a.header.hash().unwrap();
        let branch_b =
            AccountBlock::build_with_miner(NetworkId::Regtest, parent, 3, 0, Some(miner_b), vec![])
                .unwrap();
        let hash_b = branch_b.header.hash().unwrap();
        chain.submit(branch_a).unwrap();
        chain.submit(branch_b).unwrap();
        let losing_parent = if chain.tip_hash() == hash_a {
            hash_b
        } else {
            hash_a
        };
        let child = AccountBlock::build_with_miner(
            NetworkId::Regtest,
            losing_parent,
            4,
            0,
            Some(miner_c),
            vec![],
        )
        .unwrap();
        let child_hash = child.header.hash().unwrap();
        assert!(chain.submit(child).unwrap().became_best);
        assert_eq!(chain.tip_hash(), child_hash);
        assert_eq!(chain.height(), 2);
        assert_eq!(
            chain.state().total_issued().to_units(),
            first_issuance + second_issuance
        );
        assert_eq!(
            chain.state().account(miner_c).balance.to_units(),
            child_miner_issuance
        );
        assert_eq!(
            chain.state().account(miner_a).balance.to_units()
                + chain.state().account(miner_b).balance.to_units(),
            parent_miner_issuance
        );
    }

    #[test]
    fn reward_enabled_chain_rejects_missing_miner_without_mutation() {
        let genesis = block(Hash256::ZERO, 1, vec![]);
        let mut chain = AccountForkChain::new_with_protocol_params(
            genesis,
            0,
            DifficultyParams::fixed(0),
            RewardParams::experimental_testnet(account(51)),
            initial_state(),
        )
        .unwrap();
        let block = block(chain.tip_hash(), 2, vec![]);
        assert_eq!(
            chain.submit(block),
            Err(AccountForkChainError::Reward(RewardError::MissingMiner))
        );
        assert_eq!(chain.height(), 0);
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

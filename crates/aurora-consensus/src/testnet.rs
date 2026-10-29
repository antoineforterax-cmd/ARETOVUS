//! Canonical parameters shared by testnet nodes and lightweight clients.
use aurora_core::{
    accounts::AccountId,
    hash::{sha256_plain, Hash256},
    network::NetworkId,
};

use crate::{
    account_block::AccountBlock,
    account_fork_chain::AccountForkChain,
    difficulty::DifficultyParams,
    reward::{RewardParams, EMISSION_SCHEDULE_ID},
};

pub const TESTNET_INITIAL_BITS: u16 = 8;
/// Local admission policy for full nodes and SPV clients; not a consensus timestamp rule.
pub const TESTNET_MAX_FUTURE_BLOCK_SECS: u64 = 2 * 60 * 60;
pub const TESTNET_DIFFICULTY: DifficultyParams = DifficultyParams {
    target_block_seconds: 60,
    adjustment_interval: 10,
    max_adjustment_bits: 2,
    min_bits: 4,
    max_bits: 20,
};
pub const TESTNET_TREASURY: AccountId = AccountId([0xA0; 32]);
pub const TESTNET_GENESIS_HASH: Hash256 = Hash256::from_bytes([
    0x00, 0x52, 0x60, 0x5a, 0xb0, 0xf6, 0x85, 0x34, 0x27, 0x78, 0xfc, 0x74, 0x49, 0xee, 0xcd, 0x87,
    0x8e, 0xc0, 0x06, 0xe3, 0x99, 0x44, 0x3e, 0x90, 0x80, 0xf2, 0xd8, 0xb1, 0x6c, 0xe5, 0x3a, 0x76,
]);

fn fingerprint_parts(
    network: NetworkId,
    genesis_hash: Hash256,
    initial_bits: u16,
    difficulty: DifficultyParams,
    reward: RewardParams,
) -> Hash256 {
    let mut bytes = Vec::with_capacity(128);
    bytes.push(network.to_byte());
    bytes.extend_from_slice(EMISSION_SCHEDULE_ID);
    bytes.extend_from_slice(b"AURORA/testnet/native-ml-dsa-65-merkle-header/v4");
    bytes.extend_from_slice(genesis_hash.as_bytes());
    bytes.extend_from_slice(&initial_bits.to_be_bytes());
    bytes.extend_from_slice(&difficulty.target_block_seconds.to_be_bytes());
    bytes.extend_from_slice(&difficulty.adjustment_interval.to_be_bytes());
    bytes.extend_from_slice(&difficulty.max_adjustment_bits.to_be_bytes());
    bytes.extend_from_slice(&difficulty.min_bits.to_be_bytes());
    bytes.extend_from_slice(&difficulty.max_bits.to_be_bytes());
    bytes.push(u8::from(reward.enabled));
    bytes.extend_from_slice(&reward.treasury_account.0);
    sha256_plain(&bytes)
}

/// Fingerprint of a chain's consensus and peer-wire rules.
pub fn protocol_fingerprint(chain: &AccountForkChain) -> Hash256 {
    let genesis = chain.blocks().next().expect("genesis exists");
    fingerprint_parts(
        genesis.header.network,
        genesis.header.hash().expect("genesis header encodes"),
        chain.initial_difficulty_bits(),
        chain.difficulty_params(),
        chain.reward_params(),
    )
}

/// Expected fingerprint pinned by testnet wallet clients.
pub fn testnet_protocol_fingerprint() -> Hash256 {
    fingerprint_parts(
        NetworkId::Testnet,
        TESTNET_GENESIS_HASH,
        TESTNET_INITIAL_BITS,
        TESTNET_DIFFICULTY,
        RewardParams::experimental_testnet(TESTNET_TREASURY),
    )
}

/// Deterministic empty testnet genesis, including its fixed 8-bit proof of work.
pub fn testnet_genesis_block() -> AccountBlock {
    AccountBlock::build(NetworkId::Testnet, Hash256::ZERO, 1, 189, Vec::new())
        .expect("fixed testnet genesis encodes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_block::check_account_block_pow;

    #[test]
    fn wallet_fingerprint_matches_the_canonical_testnet_chain() {
        use aurora_core::state::AccountLedger;

        let chain = AccountForkChain::new_with_protocol_params(
            testnet_genesis_block(),
            TESTNET_INITIAL_BITS,
            TESTNET_DIFFICULTY,
            RewardParams::experimental_testnet(TESTNET_TREASURY),
            AccountLedger::new(NetworkId::Testnet),
        )
        .unwrap();
        assert_eq!(protocol_fingerprint(&chain), testnet_protocol_fingerprint());
    }

    #[test]
    fn fixed_genesis_matches_its_published_hash_and_work() {
        let genesis = testnet_genesis_block();
        assert_eq!(genesis.header.hash().unwrap(), TESTNET_GENESIS_HASH);
        assert_eq!(genesis.header.prev_hash, Hash256::ZERO);
        assert_eq!(genesis.header.timestamp, 1);
        assert_eq!(
            check_account_block_pow(&genesis, TESTNET_INITIAL_BITS),
            Ok(())
        );
    }
}

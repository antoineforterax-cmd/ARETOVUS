//! Canonical parameters shared by testnet nodes and lightweight clients.
use aurora_core::{accounts::AccountId, hash::Hash256, network::NetworkId};

use crate::{account_block::AccountBlock, difficulty::DifficultyParams};

pub const TESTNET_INITIAL_BITS: u16 = 8;
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

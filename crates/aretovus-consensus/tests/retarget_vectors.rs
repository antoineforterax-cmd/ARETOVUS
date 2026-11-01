//! Differential checks against the standalone Python integer model.
use std::collections::{BTreeMap, HashSet};

use aretovus_consensus::{
    account_block::{AccountBlock, AccountBlockValidationError},
    account_fork_chain::{AccountForkChain, AccountForkChainError},
    difficulty::ChainWork,
    light_header_chain::{HeaderChainError, LightHeaderChain},
    proof_of_work::leading_zero_bits,
    testnet::{
        testnet_genesis_block, TESTNET_DIFFICULTY, TESTNET_GENESIS_HASH, TESTNET_INITIAL_BITS,
    },
};
use aretovus_core::{hash::Hash256, network::NetworkId, state::AccountLedger};

#[derive(Clone, Debug)]
struct Row {
    height: u64,
    timestamp: u64,
    bits: u16,
    work: u64,
}

fn scenarios() -> BTreeMap<&'static str, Vec<Row>> {
    let mut cases = BTreeMap::new();
    for line in include_str!("../../../tests/vectors-retarget/chains.tsv")
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 5);
        cases.entry(fields[0]).or_insert_with(Vec::new).push(Row {
            height: fields[1].parse().unwrap(),
            timestamp: fields[2].parse().unwrap(),
            bits: fields[3].parse().unwrap(),
            work: fields[4].parse().unwrap(),
        });
    }
    assert_eq!(cases.len(), 7);
    cases
}

fn full_chain() -> AccountForkChain {
    AccountForkChain::new_with_params(
        testnet_genesis_block(),
        TESTNET_INITIAL_BITS,
        TESTNET_DIFFICULTY,
        AccountLedger::new(NetworkId::Testnet),
    )
    .unwrap()
}

fn light_chain() -> LightHeaderChain {
    LightHeaderChain::new(
        testnet_genesis_block().header,
        TESTNET_GENESIS_HASH,
        TESTNET_INITIAL_BITS,
        TESTNET_DIFFICULTY,
    )
    .unwrap()
}

fn work(value: u64) -> ChainWork {
    let mut total = ChainWork::default();
    for bit in 0..64 {
        if value & (1u64 << bit) != 0 {
            total = total.checked_add(ChainWork::for_difficulty(bit)).unwrap();
        }
    }
    total
}

// Exact work avoids accidentally passing a too-strict validator by overmining.
fn block(parent: Hash256, timestamp: u64, bits: u16) -> AccountBlock {
    let mut block = AccountBlock::build(NetworkId::Testnet, parent, timestamp, 0, vec![]).unwrap();
    for nonce in 0..10_000_000 {
        block.header.nonce = nonce;
        if leading_zero_bits(&block.header.hash().unwrap()) == bits {
            return block;
        }
    }
    panic!("bounded test mining exhausted at {bits} bits");
}

#[test]
fn independent_retarget_boundary_vectors_match_rust() {
    let mut count = 0;
    for line in include_str!("../../../tests/vectors-retarget/boundaries.tsv")
        .lines()
        .skip(1)
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 3);
        let parent: u16 = fields[0].parse().unwrap();
        let elapsed: u64 = fields[1].parse().unwrap();
        let expected: u16 = fields[2].parse().unwrap();
        assert_eq!(
            TESTNET_DIFFICULTY.retarget(parent, elapsed),
            expected,
            "{line}"
        );
        count += 1;
    }
    assert_eq!(count, 323);
}

#[test]
fn independent_timestamp_traces_match_full_node_and_spv() {
    for (name, rows) in scenarios() {
        let mut full = full_chain();
        let mut light = light_chain();
        assert_eq!(rows.len(), 31);
        assert_eq!(rows[0].work, 256);
        for row in rows.iter().skip(1) {
            assert_eq!(
                full.next_difficulty_bits(row.timestamp).unwrap(),
                row.bits,
                "{name} at height {}",
                row.height
            );
            // Rejection must be atomic, including at retarget boundaries.
            if row.height % 10 == 0 {
                let weak = block(full.tip_hash(), row.timestamp, row.bits - 1);
                let before = (full.tip_hash(), full.chain_work(), full.known_block_count());
                assert_eq!(
                    full.submit(weak.clone()),
                    Err(AccountForkChainError::Validation(
                        AccountBlockValidationError::InsufficientWork
                    ))
                );
                assert_eq!(
                    light.submit(weak.header),
                    Err(HeaderChainError::InsufficientWork)
                );
                assert_eq!(
                    (full.tip_hash(), full.chain_work(), full.known_block_count()),
                    before
                );
                assert_eq!(light.tip_hash(), before.0);
                assert_eq!(light.chain_work(), before.1);
            }
            let next = block(full.tip_hash(), row.timestamp, row.bits);
            full.submit(next.clone()).unwrap();
            assert!(light.submit(next.header).unwrap());
            assert_eq!(full.height(), row.height);
            assert_eq!(full.tip_timestamp(), row.timestamp);
            assert_eq!(full.difficulty_bits(), row.bits);
            assert_eq!(full.chain_work(), work(row.work), "{name}");
            assert_eq!(light.chain_work(), work(row.work), "{name}");
            assert_eq!(light.tip_hash(), full.tip_hash());
        }
        // Consensus replay is clock-independent, even for the "future" trace.
        let mut replay = full_chain();
        for saved in full.all_blocks_by_arrival().into_iter().skip(1) {
            replay.submit(saved.clone()).unwrap();
        }
        assert_eq!(replay.tip_hash(), full.tip_hash());
        assert_eq!(replay.chain_work(), full.chain_work());
    }
}

#[test]
fn competing_timestamp_branches_select_same_work_in_any_arrival_order() {
    let cases = scenarios();
    let mut branches = Vec::new();
    for name in ["regular", "fast", "slow"] {
        let mut parent = TESTNET_GENESIS_HASH;
        let mut branch = Vec::new();
        for row in cases[name].iter().skip(1) {
            let next = block(parent, row.timestamp, row.bits);
            parent = next.header.hash().unwrap();
            branch.push((next, row.clone()));
        }
        branches.push(branch);
    }
    for order in [vec![0, 1, 2], vec![2, 1, 0], vec![1, 0, 2]] {
        let mut full = full_chain();
        let mut light = light_chain();
        let mut known = HashSet::from([TESTNET_GENESIS_HASH]);
        let mut best = (256u64, TESTNET_GENESIS_HASH, 0u64);
        for index in order {
            for (next, row) in &branches[index] {
                let hash = next.header.hash().unwrap();
                if !known.insert(hash) {
                    continue;
                }
                if row.work > best.0 || (row.work == best.0 && hash < best.1) {
                    best = (row.work, hash, row.height);
                }
                full.submit(next.clone()).unwrap();
                light.submit(next.header).unwrap();
                assert_eq!(full.tip_hash(), best.1);
                assert_eq!(light.tip_hash(), best.1);
                assert_eq!(full.height(), best.2);
                assert_eq!(light.height(), best.2);
                assert_eq!(full.chain_work(), work(best.0));
                assert_eq!(light.chain_work(), work(best.0));
            }
        }
        assert_eq!(
            full.tip_hash(),
            branches[1].last().unwrap().0.header.hash().unwrap()
        );
        assert_eq!(full.known_block_count(), known.len());
    }
}

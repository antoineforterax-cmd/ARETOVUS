//! Reproducible microbenchmark; never installs a cache in production.
use std::{hint::black_box, time::Instant};

use aurora_core::{
    accounts::{
        AccountId, AccountTransaction, ML_DSA_65_PUBLIC_KEY_BYTES, POST_QUANTUM_ACCOUNT_TX_VERSION,
        POST_QUANTUM_AUTH_BYTES,
    },
    amount::Amount,
    network::NetworkId,
    signature::{post_quantum_account_id, verify_account_transaction, SignatureCache},
    state::AccountLedger,
};
use ml_dsa::{Keypair, MlDsa65, Signer, SigningKey};

fn transactions(count: usize) -> Vec<AccountTransaction> {
    // Public test seed; signing and key generation are outside timed sections.
    let key = SigningKey::<MlDsa65>::from_seed(&ml_dsa::Seed::default());
    let public = key.verifying_key().encode();
    (0..count)
        .map(|nonce| {
            let mut tx = AccountTransaction {
                version: POST_QUANTUM_ACCOUNT_TX_VERSION,
                network: NetworkId::Testnet,
                sender: post_quantum_account_id(public.as_slice()),
                nonce: nonce as u64,
                recipient: AccountId([0x33; 32]),
                amount: Amount::from_units(5).unwrap(),
                fee: Amount::from_units(1).unwrap(),
                auth_data: vec![0; POST_QUANTUM_AUTH_BYTES],
            };
            tx.auth_data[..ML_DSA_65_PUBLIC_KEY_BYTES].copy_from_slice(public.as_slice());
            let signature = key.sign(tx.signing_digest().unwrap().as_bytes()).encode();
            tx.auth_data[ML_DSA_65_PUBLIC_KEY_BYTES..].copy_from_slice(signature.as_slice());
            tx
        })
        .collect()
}

fn funded(tx: &AccountTransaction) -> AccountLedger {
    let mut ledger = AccountLedger::new(NetworkId::Testnet);
    ledger
        .credit_issuance(tx.sender, Amount::from_units(100_000).unwrap())
        .unwrap();
    ledger
}

#[test]
fn cache_identity_cannot_replace_state_validation() {
    let tx = transactions(1).remove(0);
    verify_account_transaction(&tx).unwrap();
    let id = tx.txid().unwrap();
    let mut modified = tx.clone();
    modified.amount = Amount::from_units(6).unwrap();
    assert_ne!(modified.txid().unwrap(), id);
    assert!(verify_account_transaction(&modified).is_err());
    let mut modified = tx.clone();
    modified.auth_data[ML_DSA_65_PUBLIC_KEY_BYTES] ^= 1;
    assert_ne!(modified.txid().unwrap(), id);
    assert!(verify_account_transaction(&modified).is_err());

    let mut state = funded(&tx);
    state.apply_transaction(&tx).unwrap();
    let before = state.clone();
    // Same valid signature and txid, but a different nonce state rejects replay.
    assert!(state.apply_transaction(&tx).is_err());
    assert_eq!(state, before);
    // A rollback restores eligibility; validity cannot be cached by txid alone.
    assert!(funded(&tx).apply_transaction(&tx).is_ok());
    assert!(AccountLedger::new(NetworkId::Testnet)
        .apply_transaction(&tx)
        .is_err());
}

#[test]
#[ignore = "release microbenchmark executed by mempool-benchmark workflow"]
fn measure_repeated_mempool_validation() {
    const ROUNDS: usize = 16;
    println!("BENCH_HEADER,size,trial,rounds,verify_ns,txid_ns,replay_ns,signature_cache_model_ns");
    for size in [1, 64, 256] {
        let txs = transactions(size);
        let initial = funded(&txs[0]);
        for tx in &txs {
            verify_account_transaction(tx).unwrap();
        }
        for trial in 0..3 {
            let start = Instant::now();
            for _ in 0..ROUNDS {
                for tx in &txs {
                    verify_account_transaction(black_box(tx)).unwrap();
                }
            }
            let verify = start.elapsed().as_nanos();
            let start = Instant::now();
            for _ in 0..ROUNDS {
                for tx in &txs {
                    black_box(tx.txid().unwrap());
                }
            }
            let txid = start.elapsed().as_nanos();
            let start = Instant::now();
            for _ in 0..ROUNDS {
                let mut state = initial.clone();
                for tx in &txs {
                    state.apply_transaction(black_box(tx)).unwrap();
                }
                assert_eq!(state.account(txs[0].sender).next_nonce, size as u64);
                black_box(state);
            }
            let replay = start.elapsed().as_nanos();
            let start = Instant::now();
            let mut verified = SignatureCache::new(4096);
            for _ in 0..ROUNDS {
                for tx in &txs {
                    verified.verify(black_box(tx)).unwrap();
                }
            }
            let cache = start.elapsed().as_nanos();
            println!("BENCH,{size},{trial},{ROUNDS},{verify},{txid},{replay},{cache}");
        }
    }
}

#[test]
fn cached_checks_preserve_state_and_reject_mutations() {
    let txs = transactions(3);
    let mut cache = SignatureCache::new(2);
    for tx in &txs {
        cache.verify(tx).unwrap();
    }
    // Eviction must only cause a recheck, never alter acceptance.
    cache.verify(&txs[0]).unwrap();
    let mut invalid = txs[0].clone();
    invalid.fee = Amount::from_units(2).unwrap();
    assert!(cache.verify(&invalid).is_err());
    assert!(cache.verify(&invalid).is_err());
    for capacity in [0, 1, 2] {
        let mut local = SignatureCache::new(capacity);
        local.verify(&txs[0]).unwrap();
        local.verify(&txs[0]).unwrap();
    }
    let mut cached = funded(&txs[0]);
    let mut plain = cached.clone();
    for tx in &txs {
        assert_eq!(
            cached.apply_transaction_cached(tx),
            plain.apply_transaction(tx)
        );
        assert_eq!(cached, plain);
    }
    let before = cached.clone();
    assert!(cached.apply_transaction_cached(&txs[0]).is_err());
    assert_eq!(cached, before);
    let mut rolled_back = funded(&txs[0]);
    assert!(rolled_back.apply_transaction_cached(&txs[0]).is_ok());
    let mut unfunded = AccountLedger::new(NetworkId::Testnet);
    assert!(unfunded.apply_transaction_cached(&txs[0]).is_err());
}

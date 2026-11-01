//! Real branch selection with native ML-DSA-65 transactions and a warm cache.
use aretovus_consensus::{account_block::AccountBlock, account_fork_chain::AccountForkChain};
use aretovus_core::{
    accounts::{
        AccountId, AccountTransaction, ML_DSA_65_PUBLIC_KEY_BYTES, POST_QUANTUM_ACCOUNT_TX_VERSION,
        POST_QUANTUM_AUTH_BYTES,
    },
    amount::Amount,
    hash::Hash256,
    network::NetworkId,
    signature::post_quantum_account_id,
    state::{AccountLedger, StateError},
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

fn block(parent: Hash256, time: u64, txs: Vec<AccountTransaction>) -> AccountBlock {
    AccountBlock::build(NetworkId::Testnet, parent, time, 0, txs).unwrap()
}

fn chain(tx: &AccountTransaction) -> AccountForkChain {
    AccountForkChain::new(block(Hash256::ZERO, 1, vec![]), 0, funded(tx)).unwrap()
}

fn admission(chain: &AccountForkChain, tx: &AccountTransaction) -> Result<Amount, StateError> {
    let mut plain = chain.state().clone();
    let mut cached = plain.clone();
    let before = cached.clone();
    let expected = plain.apply_transaction(tx);
    let actual = cached.apply_transaction_cached(tx);
    assert_eq!(actual, expected);
    assert_eq!(cached, plain);
    if actual.is_err() {
        assert_eq!(cached, before);
    }
    actual
}

#[test]
fn orphaned_signed_payment_becomes_admissible_after_repeated_real_reorgs() {
    let tx = transactions(1).remove(0);
    let mut chain = chain(&tx);
    let genesis = chain.tip_hash();
    let initial = chain.state().clone();
    // Warm the process cache before confirmation.
    assert_eq!(admission(&chain, &tx), Ok(tx.fee));
    let first = block(genesis, 2, vec![tx.clone()]);
    let first_hash = first.header.hash().unwrap();
    chain.submit(first).unwrap();
    assert!(matches!(
        admission(&chain, &tx),
        Err(StateError::NonceMismatch { .. })
    ));
    assert!(chain.canonical_transaction(tx.txid().unwrap()).is_some());

    let side = block(genesis, 3, vec![]);
    let side_hash = side.header.hash().unwrap();
    chain.submit(side).unwrap();
    let side_tip = block(side_hash, 4, vec![]);
    let side_tip_hash = side_tip.header.hash().unwrap();
    chain.submit(side_tip).unwrap();
    assert_eq!(chain.tip_hash(), side_tip_hash);
    assert_eq!(chain.state(), &initial);
    assert!(chain.canonical_transaction(tx.txid().unwrap()).is_none());
    assert_eq!(admission(&chain, &tx), Ok(tx.fee));

    let old_second = block(first_hash, 5, vec![]);
    let old_second_hash = old_second.header.hash().unwrap();
    chain.submit(old_second).unwrap();
    chain.submit(block(old_second_hash, 6, vec![])).unwrap();
    assert_eq!(chain.height(), 3);
    assert_eq!(chain.state().account(tx.sender).next_nonce, 1);
    assert!(admission(&chain, &tx).is_err());

    let side_third = block(side_tip_hash, 7, vec![]);
    let side_third_hash = side_third.header.hash().unwrap();
    chain.submit(side_third).unwrap();
    chain.submit(block(side_third_hash, 8, vec![])).unwrap();
    assert_eq!(chain.height(), 4);
    assert_eq!(chain.state(), &initial);
    assert_eq!(admission(&chain, &tx), Ok(tx.fee));
}

#[test]
fn warm_signature_cache_rejects_a_payment_conflicting_with_the_new_branch() {
    let tx = transactions(1).remove(0);
    let mut chain = chain(&tx);
    let genesis = chain.tip_hash();
    assert_eq!(admission(&chain, &tx), Ok(tx.fee));
    chain.submit(block(genesis, 2, vec![tx.clone()])).unwrap();

    // Same key and nonce, different signed recipient: one of two conflicting payments.
    let key = SigningKey::<MlDsa65>::from_seed(&ml_dsa::Seed::default());
    let mut conflicting = tx.clone();
    conflicting.recipient = AccountId([0x44; 32]);
    let signature = key
        .sign(conflicting.signing_digest().unwrap().as_bytes())
        .encode();
    conflicting.auth_data[ML_DSA_65_PUBLIC_KEY_BYTES..].copy_from_slice(signature.as_slice());
    let side = block(genesis, 3, vec![conflicting.clone()]);
    let side_hash = side.header.hash().unwrap();
    chain.submit(side).unwrap();
    chain.submit(block(side_hash, 4, vec![])).unwrap();
    assert!(chain.canonical_transaction(tx.txid().unwrap()).is_none());
    assert!(chain
        .canonical_transaction(conflicting.txid().unwrap())
        .is_some());
    assert!(matches!(
        admission(&chain, &tx),
        Err(StateError::NonceMismatch { .. })
    ));
    assert_eq!(chain.state().account(tx.recipient).balance, Amount::ZERO);
    assert_eq!(
        chain.state().account(conflicting.recipient).balance,
        tx.amount
    );
}

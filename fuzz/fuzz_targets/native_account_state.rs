#![no_main]

// Differential fuzzing of the native ARETOVUS Testnet account transition.
// All credentials are deterministic, synthetic and have no monetary value.
// Keep the historical Regtest/Ed25519 target intact for compatibility tests.
use aretovus_core::{
    accounts::{
        AccountId, AccountTransaction, ML_DSA_65_PUBLIC_KEY_BYTES,
        POST_QUANTUM_ACCOUNT_TX_VERSION, POST_QUANTUM_AUTH_BYTES,
    },
    amount::Amount,
    network::NetworkId,
    signature::post_quantum_account_id,
    state::AccountLedger,
};
use libfuzzer_sys::fuzz_target;
use ml_dsa::{Keypair, MlDsa65, Signer as MlSigner, SigningKey as MlSigningKey};

fuzz_target!(|data: &[u8]| {
    let signing_key = MlSigningKey::<MlDsa65>::from_seed(&ml_dsa::Seed::default());
    let public_key = signing_key.verifying_key().encode();
    let sender = post_quantum_account_id(public_key.as_slice());
    let mut plain = AccountLedger::new(NetworkId::Testnet);
    plain
        .credit_issuance(sender, Amount::from_units(512).unwrap())
        .unwrap();
    let mut cached = plain.clone();
    let initial = plain.clone();
    let mut accepted = Vec::new();

    // Bounded to eight transitions so corpus expansion cannot allocate
    // unbounded signatures or run unbounded state replay.
    for chunk in data.chunks(5).take(8) {
        let byte = |i: usize| chunk.get(i).copied().unwrap_or(0);
        if byte(0) & 8 != 0 {
            // Simulate a reorganisation rolling both implementations back.
            plain = initial.clone();
            cached = initial.clone();
            accepted.clear();
        }
        let mut tx = AccountTransaction {
            version: POST_QUANTUM_ACCOUNT_TX_VERSION,
            network: if byte(0) & 16 == 0 {
                NetworkId::Testnet
            } else {
                NetworkId::Regtest
            },
            sender,
            nonce: u64::from(byte(1) % 10),
            recipient: if byte(0) & 2 == 0 {
                AccountId([0x2b; 32])
            } else {
                sender
            },
            amount: Amount::from_units(u64::from(byte(2)) + 1).unwrap(),
            fee: Amount::from_units(u64::from(byte(3) % 8)).unwrap(),
            auth_data: vec![0; POST_QUANTUM_AUTH_BYTES],
        };
        tx.auth_data[..ML_DSA_65_PUBLIC_KEY_BYTES]
            .copy_from_slice(public_key.as_slice());
        let signature = signing_key
            .sign(tx.signing_digest().unwrap().as_bytes())
            .encode();
        tx.auth_data[ML_DSA_65_PUBLIC_KEY_BYTES..]
            .copy_from_slice(signature.as_slice());
        if byte(0) & 1 != 0 {
            let signature_byte = ML_DSA_65_PUBLIC_KEY_BYTES + usize::from(byte(4)) % signature.len();
            tx.auth_data[signature_byte] ^= 1;
        }

        let before = plain.clone();
        let expected = plain.apply_transaction(&tx);
        let actual = cached.apply_transaction_cached(&tx);
        assert_eq!(actual, expected);
        assert_eq!(cached, plain);
        if expected.is_err() {
            assert_eq!(plain, before);
        }
        assert_eq!(plain.total_issued(), initial.total_issued());

        // Rebuild the accepted prefix from its parent; adding an invalid
        // transaction must not partially apply a previously valid prefix.
        let mut sequence = accepted.clone();
        sequence.push(tx.clone());
        let mut replay = initial.clone();
        let result = replay.apply_batch(&sequence);
        assert_eq!(result.is_ok(), expected.is_ok());
        if expected.is_ok() {
            assert_eq!(replay, plain);
            accepted.push(tx);
        } else {
            assert_eq!(replay, initial);
        }
    }
});

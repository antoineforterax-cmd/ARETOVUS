#![no_main]
use aurora_core::{
    accounts::{AccountId, AccountTransaction, ACCOUNT_TX_VERSION},
    amount::Amount,
    network::NetworkId,
    state::AccountLedger,
};
use ed25519_dalek::{Signer, SigningKey};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // At most eight transitions per input, with valid signatures to reach state logic.
    let key = SigningKey::from_bytes(&[7; 32]);
    let sender = AccountId(key.verifying_key().to_bytes());
    let mut plain = AccountLedger::new(NetworkId::Regtest);
    let balance = u64::from(data.first().copied().unwrap_or(0)) + 1;
    plain.credit_issuance(sender, Amount::from_units(balance).unwrap()).unwrap();
    let mut cached = plain.clone();
    let initial = plain.clone();
    for chunk in data.chunks(5).take(8) {
        let byte = |i: usize| chunk.get(i).copied().unwrap_or(0);
        if byte(0) & 8 != 0 {
            // Simulate restoration of an ancestor state while cache entries survive.
            plain = initial.clone();
            cached = initial.clone();
        }
        let mut tx = AccountTransaction {
            version: ACCOUNT_TX_VERSION,
            network: if byte(0) & 4 == 0 { NetworkId::Regtest } else { NetworkId::Testnet },
            sender,
            nonce: u64::from(byte(1) % 10),
            recipient: if byte(0) & 2 == 0 { AccountId([9; 32]) } else { sender },
            amount: Amount::from_units(u64::from(byte(2))).unwrap(),
            fee: Amount::from_units(u64::from(byte(3))).unwrap(),
            auth_data: Vec::new(),
        };
        tx.auth_data = key.sign(tx.signing_digest().unwrap().as_bytes()).to_bytes().to_vec();
        if byte(0) & 1 != 0 {
            tx.auth_data[usize::from(byte(4)) % 64] ^= 1;
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
    }
});

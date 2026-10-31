//! Vecteurs de test d'aretovus-core.
//!
//! Les valeurs attendues ont été calculées par une implémentation **indépendante** en Python
//! (`scripts/reference_vectors.py`, `hashlib` + `struct`), pas par le code Rust testé.
//! Statut : écrits ; leur exécution doit être faite avec `cargo test` (voir README).

use aretovus_core::amount::{Amount, MAX_UNITS};
use aretovus_core::codec::{CanonicalDecode, CanonicalEncode};
use aretovus_core::experimental::{
    OutPoint, Transaction, TxInput, TxOutput, RECIPIENT_LEN, TX_VERSION,
};
use aretovus_core::hash::{sha256_plain, tagged_hash, Domain, Hash256};
use aretovus_core::network::NetworkId;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn tx1() -> Transaction {
    Transaction {
        version: TX_VERSION,
        network: NetworkId::Regtest,
        inputs: vec![TxInput {
            previous: OutPoint {
                txid: Hash256::from_bytes([0x11; 32]),
                index: 0,
            },
        }],
        outputs: vec![TxOutput {
            amount: Amount::from_units(5_000_000_000).unwrap(),
            recipient: [0x22; RECIPIENT_LEN],
        }],
        fee: Amount::from_units(10_000).unwrap(),
        auth_data: vec![0xAA, 0xBB, 0xCC],
    }
}

fn tx2() -> Transaction {
    Transaction {
        version: TX_VERSION,
        network: NetworkId::Regtest,
        inputs: vec![TxInput {
            previous: OutPoint {
                txid: tx1().txid().unwrap(),
                index: 0,
            },
        }],
        outputs: vec![TxOutput {
            amount: Amount::from_units(4_999_990_000).unwrap(),
            recipient: [0x33; RECIPIENT_LEN],
        }],
        fee: Amount::from_units(10_000).unwrap(),
        auth_data: vec![],
    }
}

#[test]
fn sha256_matches_public_vectors() {
    assert_eq!(
        sha256_plain(b"").to_hex(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_plain(b"abc").to_hex(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn tagged_hash_of_empty_data_matches_reference() {
    assert_eq!(
        tagged_hash(Domain::TxId, b"").to_hex(),
        "c8af3d0f5a1e8e357e188f31742362196cefa94157300fcc1377f1cc13f8c5a7"
    );
    assert_eq!(
        tagged_hash(Domain::BlockHeader, b"").to_hex(),
        "88f1cdcca2dd6e583f249d5764203952a21f9181d80de4ca63ee0b06083e06b8"
    );
    assert_eq!(
        tagged_hash(Domain::TxList, b"").to_hex(),
        "70ac7aa3e093b0ba1c2c2076afc9a7d4d23e91c4d88860258ecf87a1a9daaae1"
    );
}

#[test]
fn max_amount_encoding_matches_reference() {
    let bytes = Amount::MAX.to_canonical_bytes().unwrap();
    assert_eq!(hex(&bytes), "00b1a2bc2ec50000");
    assert_eq!(Amount::MAX.to_units(), MAX_UNITS);
    assert_eq!(Amount::from_canonical_bytes(&bytes).unwrap(), Amount::MAX);
}

#[test]
fn transaction_one_encoding_and_id_match_reference() {
    let tx = tx1();
    let bytes = tx.to_canonical_bytes().unwrap();
    assert_eq!(bytes.len(), 104);
    assert_eq!(
        hex(&bytes),
        "00000000010000000111111111111111111111111111111111111111111111111111111111111111110000000000000001000000012a05f2002222222222222222222222222222222222222222222222222222222222222222000000000000271000000003aabbcc"
    );
    assert_eq!(
        tx.txid().unwrap().to_hex(),
        "48b93d8400fb94ef233c23d9f94fc09e14714de2129ef781b2ef70ac06de578b"
    );
    assert_eq!(Transaction::from_canonical_bytes(&bytes).unwrap(), tx);
}

#[test]
fn transaction_two_encoding_and_id_match_reference() {
    let tx = tx2();
    let bytes = tx.to_canonical_bytes().unwrap();
    assert_eq!(
        hex(&bytes),
        "00000000010000000148b93d8400fb94ef233c23d9f94fc09e14714de2129ef781b2ef70ac06de578b0000000000000001000000012a05caf03333333333333333333333333333333333333333333333333333333333333333000000000000271000000000"
    );
    assert_eq!(
        tx.txid().unwrap().to_hex(),
        "06ceef76cbe28135768625998a3973b2fc09011edd19c2b58f4e74fca8b8fb55"
    );
}

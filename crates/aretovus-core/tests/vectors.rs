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
        "efbc63de5a9bb42d1a17d2176cb374431a00d0f709e12a8aca22889624e08839"
    );
    assert_eq!(
        tagged_hash(Domain::BlockHeader, b"").to_hex(),
        "74435e734173833e888ecf6563179afbb984570049a2748f07e0586d19a598ea"
    );
    assert_eq!(
        tagged_hash(Domain::TxList, b"").to_hex(),
        "7c4c17849e31cc98e71ea3e7a76056f6679bc7d2d385205c42f296566dfeea9b"
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
        "96838aa57e50df880a40fe87f15ef0c4250e3ad216273106089189ea7d739074"
    );
    assert_eq!(Transaction::from_canonical_bytes(&bytes).unwrap(), tx);
}

#[test]
fn transaction_two_encoding_and_id_match_reference() {
    let tx = tx2();
    let bytes = tx.to_canonical_bytes().unwrap();
    assert_eq!(
        hex(&bytes),
        "00000000010000000196838aa57e50df880a40fe87f15ef0c4250e3ad216273106089189ea7d7390740000000000000001000000012a05caf03333333333333333333333333333333333333333333333333333333333333333000000000000271000000000"
    );
    assert_eq!(
        tx.txid().unwrap().to_hex(),
        "6a28eb863f7eefed622fe2b62beecd4500e4ebb95cd55381716ca64101b6070c"
    );
}

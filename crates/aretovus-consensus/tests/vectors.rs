//! Vecteurs de test d'aretovus-consensus.
//!
//! Valeurs attendues calculées par `scripts/reference_vectors.py` (implémentation indépendante).
//! Statut : écrits ; leur exécution doit être faite avec `cargo test` (voir README).

use aretovus_consensus::experimental::{compute_tx_commitment, Block};
use aretovus_core::amount::Amount;
use aretovus_core::codec::CanonicalEncode;
use aretovus_core::experimental::{
    OutPoint, Transaction, TxInput, TxOutput, RECIPIENT_LEN, TX_VERSION,
};
use aretovus_core::hash::Hash256;
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
fn commitments_match_reference() {
    assert_eq!(
        compute_tx_commitment(&[]).unwrap().to_hex(),
        "683bd9fd56e18dff17bbd6bea89f84a0af012bef8d60f5cee9952adf81ec72d2"
    );
    assert_eq!(
        compute_tx_commitment(&[tx1()]).unwrap().to_hex(),
        "cf70686fd0747d48da72846b6cb629b6182c5f37143eef459a07a2a2b3047169"
    );
    assert_eq!(
        compute_tx_commitment(&[tx1(), tx2()]).unwrap().to_hex(),
        "f3de0f78f15dfb150933f0ef17d33e25aaae4609eb09ce90b5d8ca3ee9dba71e"
    );
}

#[test]
fn empty_block_header_matches_reference() {
    let block = Block::build(NetworkId::Regtest, Hash256::ZERO, 1_000_000, 42, vec![]).unwrap();
    assert_eq!(
        hex(&block.header.to_canonical_bytes().unwrap()),
        "00000001010000000000000000000000000000000000000000000000000000000000000000683bd9fd56e18dff17bbd6bea89f84a0af012bef8d60f5cee9952adf81ec72d2683bd9fd56e18dff17bbd6bea89f84a0af012bef8d60f5cee9952adf81ec72d200000000000f4240000000000000002a"
    );
    assert_eq!(
        block.header.hash().unwrap().to_hex(),
        "f1f2afdce26385cb9d8369c5b9b90d01356729c8f538bf31f6986954ab0ed75a"
    );
}

#[test]
fn one_transaction_block_header_matches_reference() {
    let block = Block::build(
        NetworkId::Regtest,
        Hash256::ZERO,
        1_000_000,
        42,
        vec![tx1()],
    )
    .unwrap();
    assert_eq!(
        hex(&block.header.to_canonical_bytes().unwrap()),
        "00000001010000000000000000000000000000000000000000000000000000000000000000cf70686fd0747d48da72846b6cb629b6182c5f37143eef459a07a2a2b3047169cf70686fd0747d48da72846b6cb629b6182c5f37143eef459a07a2a2b304716900000000000f4240000000000000002a"
    );
    assert_eq!(
        block.header.hash().unwrap().to_hex(),
        "44dd3881d22fea87a701197b948029ae22bd05582f77e0519b89ba6be8336bff"
    );
    assert_eq!(block.check_stateless(), Ok(()));
}

#[test]
fn two_transaction_block_header_matches_reference() {
    let block = Block::build(
        NetworkId::Regtest,
        Hash256::ZERO,
        1_000_000,
        42,
        vec![tx1(), tx2()],
    )
    .unwrap();
    assert_eq!(
        hex(&block.header.to_canonical_bytes().unwrap()),
        "00000001010000000000000000000000000000000000000000000000000000000000000000f3de0f78f15dfb150933f0ef17d33e25aaae4609eb09ce90b5d8ca3ee9dba71ef3de0f78f15dfb150933f0ef17d33e25aaae4609eb09ce90b5d8ca3ee9dba71e00000000000f4240000000000000002a"
    );
    assert_eq!(
        block.header.hash().unwrap().to_hex(),
        "dbcab0a7c3369942c0d1c96b47dc2c3eca015f2043186001204028ae29f100c7"
    );
    assert_eq!(block.check_stateless(), Ok(()));
}

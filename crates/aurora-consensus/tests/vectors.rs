//! Vecteurs de test d'aurora-consensus.
//!
//! Valeurs attendues calculées par `scripts/reference_vectors.py` (implémentation indépendante).
//! Statut : écrits ; leur exécution doit être faite avec `cargo test` (voir README).

use aurora_consensus::experimental::{compute_tx_commitment, Block};
use aurora_core::amount::Amount;
use aurora_core::codec::CanonicalEncode;
use aurora_core::experimental::{OutPoint, Transaction, TxInput, TxOutput, RECIPIENT_LEN, TX_VERSION};
use aurora_core::hash::Hash256;
use aurora_core::network::NetworkId;

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
        "f8c2cae75a91f9d2df0fc9d26ec0de98b103bb072049407ebde0b4cafa589516"
    );
    assert_eq!(
        compute_tx_commitment(&[tx1()]).unwrap().to_hex(),
        "71b23551dbdcc79a762605698484299b6ba5c0dca1c261fa7b80568327b1b02d"
    );
    assert_eq!(
        compute_tx_commitment(&[tx1(), tx2()]).unwrap().to_hex(),
        "41694d5373926956529de59351fe6f3a1f209ec1b139e2516364bcf659c69a0a"
    );
}

#[test]
fn empty_block_header_matches_reference() {
    let block = Block::build(NetworkId::Regtest, Hash256::ZERO, 1_000_000, 42, vec![]).unwrap();
    assert_eq!(
        hex(&block.header.to_canonical_bytes().unwrap()),
        "00000000010000000000000000000000000000000000000000000000000000000000000000f8c2cae75a91f9d2df0fc9d26ec0de98b103bb072049407ebde0b4cafa58951600000000000f4240000000000000002a"
    );
    assert_eq!(
        block.header.hash().unwrap().to_hex(),
        "7cdc1b45083edb0ba682c8b9e325462c1330018c9ed2881dff9933dd66383c38"
    );
}

#[test]
fn one_transaction_block_header_matches_reference() {
    let block = Block::build(NetworkId::Regtest, Hash256::ZERO, 1_000_000, 42, vec![tx1()]).unwrap();
    assert_eq!(
        hex(&block.header.to_canonical_bytes().unwrap()),
        "0000000001000000000000000000000000000000000000000000000000000000000000000071b23551dbdcc79a762605698484299b6ba5c0dca1c261fa7b80568327b1b02d00000000000f4240000000000000002a"
    );
    assert_eq!(
        block.header.hash().unwrap().to_hex(),
        "2e101bb47474581a451b11591a891b64e8f68b3516edfdba3edf547150ee617e"
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
        "0000000001000000000000000000000000000000000000000000000000000000000000000041694d5373926956529de59351fe6f3a1f209ec1b139e2516364bcf659c69a0a00000000000f4240000000000000002a"
    );
    assert_eq!(
        block.header.hash().unwrap().to_hex(),
        "050e3fc83f1e69ad6eea800dbe4f7edad2b7da9c0ccc22da2ff0ff64ef7585e2"
    );
    assert_eq!(block.check_stateless(), Ok(()));
}

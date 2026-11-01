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
        "a481414fc620e2ab059b4dab124d18f807f3239e2b46c54b8ded6b0009cc4cd1"
    );
    assert_eq!(
        compute_tx_commitment(&[tx1()]).unwrap().to_hex(),
        "451775c211bcb127d30aa0680d4b26c41dbb0687b159131c09e86c7b71f46532"
    );
    assert_eq!(
        compute_tx_commitment(&[tx1(), tx2()]).unwrap().to_hex(),
        "93fb7610a3eac93a03ba4e328ced8df4d39a22a7cce0676463ce806c0f9b7b27"
    );
}

#[test]
fn empty_block_header_matches_reference() {
    let block = Block::build(NetworkId::Regtest, Hash256::ZERO, 1_000_000, 42, vec![]).unwrap();
    assert_eq!(
        hex(&block.header.to_canonical_bytes().unwrap()),
        "00000001010000000000000000000000000000000000000000000000000000000000000000a481414fc620e2ab059b4dab124d18f807f3239e2b46c54b8ded6b0009cc4cd1a481414fc620e2ab059b4dab124d18f807f3239e2b46c54b8ded6b0009cc4cd100000000000f4240000000000000002a"
    );
    assert_eq!(
        block.header.hash().unwrap().to_hex(),
        "0a7a88e6ecfe3cc0f43f273a12ebf8c8178f7b6987531129dfea7edcb412e42c"
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
        "00000001010000000000000000000000000000000000000000000000000000000000000000451775c211bcb127d30aa0680d4b26c41dbb0687b159131c09e86c7b71f46532451775c211bcb127d30aa0680d4b26c41dbb0687b159131c09e86c7b71f4653200000000000f4240000000000000002a"
    );
    assert_eq!(
        block.header.hash().unwrap().to_hex(),
        "f497258b870d07d7f93e3c0b9057aa12a381e204b2b98e148a37a79ac164faef"
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
        "0000000101000000000000000000000000000000000000000000000000000000000000000093fb7610a3eac93a03ba4e328ced8df4d39a22a7cce0676463ce806c0f9b7b2793fb7610a3eac93a03ba4e328ced8df4d39a22a7cce0676463ce806c0f9b7b2700000000000f4240000000000000002a"
    );
    assert_eq!(
        block.header.hash().unwrap().to_hex(),
        "6ffd433a509a18bc262f28c4893dd785c46519dc121fafbbae56d74ccbc8060a"
    );
    assert_eq!(block.check_stateless(), Ok(()));
}

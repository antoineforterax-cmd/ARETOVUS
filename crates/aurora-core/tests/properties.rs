//! Tests « de propriétés » sans dépendance externe : générateur pseudo-aléatoire déterministe.
//!
//! Propriétés vérifiées sur la transaction expérimentale :
//! 1. aller-retour : `decode(encode(x)) == x` ;
//! 2. canonicité : si `decode(octets)` réussit, alors `encode(valeur) == octets` ;
//! 3. tout préfixe strict d'un encodage valide est refusé ; tout octet en trop est refusé ;
//! 4. des données aléatoires ou altérées ne provoquent jamais de panique ;
//! 5. une longueur annoncée excessive est refusée.
//!
//! Limites : ce n'est pas du fuzzing ni un vrai moteur de propriétés (pas de réduction des
//! contre-exemples). `proptest` et `cargo-fuzz` sont prévus (TST-03), pas encore installés.
//! Statut : écrits ; leur exécution doit être faite avec `cargo test` (voir README).

use aurora_core::amount::{Amount, MAX_UNITS};
use aurora_core::codec::{CanonicalDecode, CanonicalEncode, DecodeError};
use aurora_core::experimental::{
    OutPoint, Transaction, TxInput, TxOutput, MAX_AUTH_BYTES, RECIPIENT_LEN, TX_VERSION,
};
use aurora_core::hash::Hash256;
use aurora_core::network::NetworkId;

/// Générateur xorshift64 : déterministe, sans dépendance, **non cryptographique** (tests seulement).
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }

    fn byte(&mut self) -> u8 {
        (self.next_u64() & 0xff) as u8
    }

    fn hash(&mut self) -> Hash256 {
        let mut bytes = [0u8; 32];
        for b in bytes.iter_mut() {
            *b = self.byte();
        }
        Hash256::from_bytes(bytes)
    }

    fn amount(&mut self) -> Amount {
        Amount::from_units(self.below(MAX_UNITS + 1)).unwrap()
    }
}

fn random_tx(rng: &mut Rng) -> Transaction {
    let n_inputs = rng.below(5) as usize;
    let n_outputs = 1 + rng.below(4) as usize;
    let network = match rng.below(3) {
        0 => NetworkId::Regtest,
        1 => NetworkId::Testnet,
        _ => NetworkId::Mainnet,
    };
    let mut inputs = Vec::new();
    for _ in 0..n_inputs {
        inputs.push(TxInput {
            previous: OutPoint {
                txid: rng.hash(),
                index: rng.below(1_000) as u32,
            },
        });
    }
    let mut outputs = Vec::new();
    for _ in 0..n_outputs {
        let mut recipient = [0u8; RECIPIENT_LEN];
        for b in recipient.iter_mut() {
            *b = rng.byte();
        }
        outputs.push(TxOutput {
            amount: rng.amount(),
            recipient,
        });
    }
    let auth_len = rng.below(64) as usize;
    let auth_data: Vec<u8> = (0..auth_len).map(|_| rng.byte()).collect();
    Transaction {
        version: TX_VERSION,
        network,
        inputs,
        outputs,
        fee: rng.amount(),
        auth_data,
    }
}

#[test]
fn roundtrip_random_transactions() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..500 {
        let tx = random_tx(&mut rng);
        let bytes = tx.to_canonical_bytes().unwrap();
        let decoded = Transaction::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(decoded, tx);
        assert_eq!(decoded.to_canonical_bytes().unwrap(), bytes);
    }
}

#[test]
fn every_strict_prefix_is_rejected_and_trailing_byte_too() {
    let mut rng = Rng(0x1234_5678_9ABC_DEF1);
    for _ in 0..50 {
        let tx = random_tx(&mut rng);
        let bytes = tx.to_canonical_bytes().unwrap();
        for cut in 0..bytes.len() {
            assert!(
                Transaction::from_canonical_bytes(&bytes[..cut]).is_err(),
                "le préfixe de longueur {cut} a été accepté"
            );
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(matches!(
            Transaction::from_canonical_bytes(&longer),
            Err(DecodeError::TrailingBytes { .. })
        ));
    }
}

#[test]
fn mutated_encodings_stay_canonical_or_fail() {
    let mut rng = Rng(0x0FED_CBA9_8765_4321);
    for _ in 0..20 {
        let tx = random_tx(&mut rng);
        let bytes = tx.to_canonical_bytes().unwrap();
        for position in 0..bytes.len() {
            for mask in [0x01u8, 0x02, 0x10, 0x80, 0xFF] {
                let mut mutated = bytes.clone();
                mutated[position] ^= mask;
                if let Ok(decoded) = Transaction::from_canonical_bytes(&mutated) {
                    assert_eq!(decoded.to_canonical_bytes().unwrap(), mutated);
                }
            }
        }
    }
}

#[test]
fn random_garbage_never_panics_and_is_canonical_when_accepted() {
    let mut rng = Rng(0xDEAD_BEEF_CAFE_F00D);
    for _ in 0..3_000 {
        let len = rng.below(200) as usize;
        let garbage: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        if let Ok(decoded) = Transaction::from_canonical_bytes(&garbage) {
            assert_eq!(decoded.to_canonical_bytes().unwrap(), garbage);
        }
    }
}

#[test]
fn excessive_length_claims_are_rejected() {
    // version 0, réseau 1, puis un nombre d'entrées annoncé de 0xFFFF_FFFF.
    let bytes = [0, 0, 0, 0, 1, 0xFF, 0xFF, 0xFF, 0xFF];
    assert!(matches!(
        Transaction::from_canonical_bytes(&bytes),
        Err(DecodeError::LengthTooLarge { .. })
    ));
}

#[test]
fn unknown_network_and_version_are_rejected() {
    let mut rng = Rng(42);
    let tx = random_tx(&mut rng);
    let bytes = tx.to_canonical_bytes().unwrap();

    let mut bad_network = bytes.clone();
    bad_network[4] = 9;
    assert!(matches!(
        Transaction::from_canonical_bytes(&bad_network),
        Err(DecodeError::InvalidValue(_))
    ));

    let mut bad_version = bytes;
    bad_version[3] = 1;
    assert!(matches!(
        Transaction::from_canonical_bytes(&bad_version),
        Err(DecodeError::InvalidValue(_))
    ));
}

#[test]
fn amount_above_cap_is_rejected_on_decode() {
    assert!(Amount::from_canonical_bytes(&MAX_UNITS.to_be_bytes()).is_ok());
    assert!(matches!(
        Amount::from_canonical_bytes(&(MAX_UNITS + 1).to_be_bytes()),
        Err(DecodeError::InvalidValue(_))
    ));
}

#[test]
fn oversized_auth_data_cannot_be_encoded() {
    let mut rng = Rng(7);
    let mut tx = random_tx(&mut rng);
    tx.auth_data = vec![0u8; MAX_AUTH_BYTES + 1];
    assert!(tx.to_canonical_bytes().is_err());
}

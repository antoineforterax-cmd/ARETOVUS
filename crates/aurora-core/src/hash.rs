//! Hachage SHA-256 avec séparation de domaines (hypothèse TX-03, non validée).
//!
//! Construction expérimentale :
//! `H(domaine, données) = SHA-256( longueur_du_tag (1 octet) || tag || données )`.
//! Le SHA-256 vient de la bibliothèque `sha2` (RustCrypto) : aucune primitive maison.
//! Le choix SHA-256 simple / double / autre reste ouvert dans le registre.

use std::fmt;

use sha2::{Digest, Sha256};

use crate::codec::{CanonicalDecode, CanonicalEncode, DecodeError, Decoder, EncodeError, Encoder};

/// Taille d'un condensat en octets.
pub const HASH_LEN: usize = 32;

/// Condensat de 32 octets.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash256([u8; HASH_LEN]);

impl Hash256 {
    /// Condensat nul (sert de parent au premier bloc d'expérience).
    pub const ZERO: Hash256 = Hash256([0u8; HASH_LEN]);

    /// Crée un condensat à partir d'octets bruts.
    pub const fn from_bytes(bytes: [u8; HASH_LEN]) -> Self {
        Hash256(bytes)
    }

    /// Octets bruts.
    pub const fn as_bytes(&self) -> &[u8; HASH_LEN] {
        &self.0
    }

    /// Représentation hexadécimale minuscule (64 caractères).
    pub fn to_hex(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(HASH_LEN * 2);
        for byte in self.0 {
            out.push(HEX[usize::from(byte >> 4)] as char);
            out.push(HEX[usize::from(byte & 0x0f)] as char);
        }
        out
    }
}

impl fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash256({})", self.to_hex())
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl CanonicalEncode for Hash256 {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        enc.put_fixed(&self.0);
        Ok(())
    }
}

impl CanonicalDecode for Hash256 {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        Ok(Hash256(dec.get_array::<HASH_LEN>()?))
    }
}

const TAG_TXID: &[u8] = b"AURORA/exp/v0/txid";
const TAG_ACCOUNT_TXID: &[u8] = b"AURORA/exp/v0/account-txid";
const TAG_TX_SIGNING: &[u8] = b"AURORA/exp/v0/tx-signing";
const TAG_BLOCK_HEADER: &[u8] = b"AURORA/exp/v0/block-header";
const TAG_TX_LIST: &[u8] = b"AURORA/exp/v0/tx-list";
const TAG_ACCOUNT_TX_LIST: &[u8] = b"AURORA/exp/v0/account-tx-list";
const TAG_HYBRID_ACCOUNT_ID: &[u8] = b"AURORA/exp/v0/hybrid-account-id";
const TAG_POST_QUANTUM_ACCOUNT_ID: &[u8] = b"AURORA/exp/v0/post-quantum-account-id";

/// Domaine de hachage : deux domaines différents ne produisent jamais le même condensat
/// pour les mêmes données.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domain {
    /// Identifiant de transaction.
    TxId,
    /// Account transaction identifier.
    AccountTxId,
    /// Empreinte des champs signés d'une transaction, sans les données d'autorisation.
    TxSigning,
    /// En-tête de bloc.
    BlockHeader,
    /// Engagement sur la liste des transactions d'un bloc (pas un arbre de Merkle : TX-04 ouvert).
    TxList,
    /// Ordered commitment for experimental account transactions.
    AccountTxList,
    /// Account identifier committing to both hybrid public keys.
    HybridAccountId,
    /// Account identifier derived from the native ML-DSA-65 public key.
    PostQuantumAccountId,
}

impl Domain {
    /// Étiquette du domaine (moins de 256 octets, vérifié par un test).
    pub fn tag(self) -> &'static [u8] {
        match self {
            Domain::TxId => TAG_TXID,
            Domain::AccountTxId => TAG_ACCOUNT_TXID,
            Domain::TxSigning => TAG_TX_SIGNING,
            Domain::BlockHeader => TAG_BLOCK_HEADER,
            Domain::TxList => TAG_TX_LIST,
            Domain::AccountTxList => TAG_ACCOUNT_TX_LIST,
            Domain::HybridAccountId => TAG_HYBRID_ACCOUNT_ID,
            Domain::PostQuantumAccountId => TAG_POST_QUANTUM_ACCOUNT_ID,
        }
    }
}

fn finish(hasher: Sha256) -> Hash256 {
    let digest = hasher.finalize();
    let mut out = [0u8; HASH_LEN];
    out.copy_from_slice(&digest);
    Hash256(out)
}

/// SHA-256 simple, sans domaine. Exposé pour vérifier le câblage avec des vecteurs publics.
pub fn sha256_plain(data: &[u8]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    finish(hasher)
}

/// Condensat avec séparation de domaine.
pub fn tagged_hash(domain: Domain, data: &[u8]) -> Hash256 {
    let tag = domain.tag();
    let mut hasher = Sha256::new();
    hasher.update([tag.len() as u8]);
    hasher.update(tag);
    hasher.update(data);
    finish(hasher)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_DOMAINS: [Domain; 8] = [
        Domain::TxId,
        Domain::AccountTxId,
        Domain::TxSigning,
        Domain::BlockHeader,
        Domain::TxList,
        Domain::AccountTxList,
        Domain::HybridAccountId,
        Domain::PostQuantumAccountId,
    ];

    #[test]
    fn domain_tags_fit_in_one_byte_length() {
        for domain in ALL_DOMAINS {
            assert!(domain.tag().len() < 256);
        }
    }

    #[test]
    fn domain_tags_are_distinct() {
        for (i, a) in ALL_DOMAINS.iter().enumerate() {
            for b in ALL_DOMAINS.iter().skip(i + 1) {
                assert_ne!(a.tag(), b.tag());
            }
        }
    }

    #[test]
    fn same_data_different_domain_gives_different_hash() {
        let a = tagged_hash(Domain::TxId, b"data");
        let b = tagged_hash(Domain::BlockHeader, b"data");
        assert_ne!(a, b);
    }

    #[test]
    fn hex_is_lowercase_and_64_chars() {
        let h = Hash256::from_bytes([0xAB; HASH_LEN]);
        let hex = h.to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex.starts_with("abab"));
        assert_eq!(hex, hex.to_lowercase());
    }

    #[test]
    fn sha256_plain_matches_the_published_abc_test_vector() {
        // Vecteur de référence SHA-256 : FIPS 180-4, message ASCII "abc".
        assert_eq!(
            sha256_plain(b"abc").to_hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn codec_roundtrip() {
        let h = tagged_hash(Domain::TxId, b"x");
        let bytes = h.to_canonical_bytes().unwrap();
        assert_eq!(bytes.len(), HASH_LEN);
        assert_eq!(Hash256::from_canonical_bytes(&bytes).unwrap(), h);
        assert_eq!(
            Hash256::from_canonical_bytes(&bytes[..31]),
            Err(DecodeError::UnexpectedEof)
        );
    }
}

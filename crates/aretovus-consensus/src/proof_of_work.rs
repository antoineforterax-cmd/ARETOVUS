//! Outil de preuve de travail pour les expériences locales ARETOVUS.
//!
//! ATTENTION : ce module n'est PAS encore le consensus de la blockchain. La difficulté est
//! fournie par l'appelant, n'est pas encodée dans l'en-tête et n'est pas ajustée par une chaîne.
//! Il ne doit pas être utilisé pour un réseau public ni pour sécuriser des fonds.
//!
//! Le mécanisme actuel compte les bits à zéro en tête du hash SHA-256 de l'en-tête. C'est une
//! méthode simple pour tester le cycle nonce/hash, pas une décision définitive sur le PoW ARETOVUS.

use crate::experimental::BlockHeader;
use aretovus_core::codec::EncodeError;
use aretovus_core::hash::Hash256;

/// Difficulté maximale supportée par ce helper (un hash de 256 bits ne peut pas garantir 256 bits à zéro).
pub const MAX_LEADING_ZERO_BITS: u16 = 255;

/// Erreur du helper de minage expérimental.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowError {
    /// Difficulté hors de l'intervalle autorisé.
    InvalidDifficulty,
    /// Tous les nonces de la plage demandée ont été essayés.
    NonceSpaceExhausted,
    /// L'en-tête n'a pas pu être encodé.
    Encoding(EncodeError),
}

impl std::fmt::Display for PowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowError::InvalidDifficulty => write!(f, "difficulté expérimentale invalide"),
            PowError::NonceSpaceExhausted => write!(f, "espace de nonces épuisé"),
            PowError::Encoding(err) => write!(f, "encodage d'en-tête impossible : {err}"),
        }
    }
}

impl std::error::Error for PowError {}

/// Compte les bits à zéro consécutifs au début du hash, du bit de poids fort au bit de poids faible.
pub fn leading_zero_bits(hash: &Hash256) -> u16 {
    let mut total = 0u16;
    for byte in hash.as_bytes() {
        if *byte == 0 {
            total += 8;
        } else {
            total += byte.leading_zeros() as u16;
            break;
        }
    }
    total
}

/// Vérifie si l'en-tête satisfait une difficulté exprimée en bits à zéro en tête.
///
/// La difficulté doit provenir d'une règle externe explicite. Cette fonction ne valide ni la chaîne,
/// ni l'horodatage, ni la récompense, ni la difficulté autorisée par le protocole.
pub fn check_proof(header: &BlockHeader, difficulty_bits: u16) -> Result<bool, PowError> {
    if difficulty_bits > MAX_LEADING_ZERO_BITS {
        return Err(PowError::InvalidDifficulty);
    }
    let hash = header.hash().map_err(PowError::Encoding)?;
    Ok(leading_zero_bits(&hash) >= difficulty_bits)
}

/// Cherche un nonce dans la plage semi-ouverte [start_nonce, end_nonce_exclusive).
///
/// Renvoie le nonce trouvé sans modifier l'en-tête fourni. La plage bornée permet de rendre les
/// expériences et les tests reproductibles, sans boucle de minage infinie.
pub fn mine_nonce(
    header: &BlockHeader,
    difficulty_bits: u16,
    start_nonce: u64,
    end_nonce_exclusive: u64,
) -> Result<u64, PowError> {
    if difficulty_bits > MAX_LEADING_ZERO_BITS {
        return Err(PowError::InvalidDifficulty);
    }
    if start_nonce >= end_nonce_exclusive {
        return Err(PowError::NonceSpaceExhausted);
    }

    for nonce in start_nonce..end_nonce_exclusive {
        let mut candidate = *header;
        candidate.nonce = nonce;
        if check_proof(&candidate, difficulty_bits)? {
            return Ok(nonce);
        }
    }
    Err(PowError::NonceSpaceExhausted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aretovus_core::hash::Hash256;
    use aretovus_core::network::NetworkId;

    fn header() -> BlockHeader {
        BlockHeader {
            version: crate::experimental::HEADER_VERSION,
            network: NetworkId::Regtest,
            prev_hash: Hash256::ZERO,
            tx_commitment: Hash256::ZERO,
            tx_merkle_root: Hash256::ZERO,
            timestamp: 1,
            nonce: 0,
        }
    }

    #[test]
    fn counts_leading_zero_bits() {
        assert_eq!(leading_zero_bits(&Hash256::from_bytes([0; 32])), 256);
        let mut bytes = [0xff; 32];
        bytes[0] = 0b0001_1111;
        assert_eq!(leading_zero_bits(&Hash256::from_bytes(bytes)), 3);
        let mut bytes = [0xff; 32];
        bytes[0] = 0;
        bytes[1] = 0;
        bytes[2] = 0b0000_1111;
        assert_eq!(leading_zero_bits(&Hash256::from_bytes(bytes)), 20);
    }

    #[test]
    fn zero_difficulty_accepts_any_valid_header() {
        assert_eq!(check_proof(&header(), 0), Ok(true));
    }

    #[test]
    fn rejects_out_of_range_difficulty() {
        assert_eq!(
            check_proof(&header(), 256),
            Err(PowError::InvalidDifficulty)
        );
        assert_eq!(
            mine_nonce(&header(), 256, 0, 10),
            Err(PowError::InvalidDifficulty)
        );
    }

    #[test]
    fn mines_and_verifies_a_low_difficulty_nonce() {
        let original = header();
        let nonce = mine_nonce(&original, 8, 0, 100_000).unwrap();
        let mut candidate = original;
        candidate.nonce = nonce;
        assert_eq!(check_proof(&candidate, 8), Ok(true));
        assert_eq!(
            original.nonce, 0,
            "la fonction ne doit pas modifier l'en-tête original"
        );
    }

    #[test]
    fn bounded_search_reports_exhaustion() {
        assert_eq!(
            mine_nonce(&header(), 255, 0, 1),
            Err(PowError::NonceSpaceExhausted)
        );
        assert_eq!(
            mine_nonce(&header(), 0, 5, 5),
            Err(PowError::NonceSpaceExhausted)
        );
    }
}

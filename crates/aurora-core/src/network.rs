//! Identifiants des environnements (hypothèse TX-09, non validée).
//!
//! Chaque transaction et chaque bloc porte son réseau : une donnée de test ne peut pas être
//! prise pour une donnée d'un autre environnement. `Mainnet` est réservé : aucun réseau de
//! production n'existe, et rien ici n'autorise l'usage de vrais fonds.

use crate::codec::{CanonicalDecode, CanonicalEncode, DecodeError, Decoder, EncodeError, Encoder};

/// Environnement d'exécution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NetworkId {
    /// Réseau local de développement.
    Regtest,
    /// Réseau de test partagé.
    Testnet,
    /// Réseau de production (réservé, inexistant).
    Mainnet,
}

impl NetworkId {
    /// Octet utilisé dans l'encodage canonique.
    pub const fn to_byte(self) -> u8 {
        match self {
            NetworkId::Regtest => 1,
            NetworkId::Testnet => 2,
            NetworkId::Mainnet => 3,
        }
    }

    /// Lit un octet ; toute valeur inconnue est refusée.
    pub const fn from_byte(byte: u8) -> Option<NetworkId> {
        match byte {
            1 => Some(NetworkId::Regtest),
            2 => Some(NetworkId::Testnet),
            3 => Some(NetworkId::Mainnet),
            _ => None,
        }
    }

    /// Nom lisible.
    pub const fn name(self) -> &'static str {
        match self {
            NetworkId::Regtest => "regtest",
            NetworkId::Testnet => "testnet",
            NetworkId::Mainnet => "mainnet",
        }
    }
}

impl CanonicalEncode for NetworkId {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        enc.put_u8(self.to_byte());
        Ok(())
    }
}

impl CanonicalDecode for NetworkId {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let byte = dec.get_u8()?;
        NetworkId::from_byte(byte).ok_or(DecodeError::InvalidValue("identifiant de réseau inconnu"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_mapping_roundtrips() {
        for net in [NetworkId::Regtest, NetworkId::Testnet, NetworkId::Mainnet] {
            assert_eq!(NetworkId::from_byte(net.to_byte()), Some(net));
        }
    }

    #[test]
    fn unknown_bytes_are_rejected() {
        assert_eq!(NetworkId::from_byte(0), None);
        assert_eq!(NetworkId::from_byte(4), None);
        assert!(matches!(
            NetworkId::from_canonical_bytes(&[9]),
            Err(DecodeError::InvalidValue(_))
        ));
    }

    #[test]
    fn names_are_distinct() {
        assert_ne!(NetworkId::Regtest.name(), NetworkId::Testnet.name());
        assert_ne!(NetworkId::Testnet.name(), NetworkId::Mainnet.name());
    }
}

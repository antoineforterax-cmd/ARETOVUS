//! Montants monétaires : entiers uniquement.
//!
//! Hypothèses du prototype (non validées, voir `docs/DECISION_REGISTER.md`) :
//! - unité minimale : 10^-8 AUR, soit 8 décimales (TX-06) ;
//! - plafond de sécurité du type : 500 000 000 AUR (plafond de garde-fou du prototype,
//!   pas une règle d'émission).

use std::fmt;

use crate::codec::{CanonicalDecode, CanonicalEncode, DecodeError, Decoder, EncodeError, Encoder};

/// Nombre de décimales (hypothèse TX-06).
pub const DECIMALS: usize = 8;
/// Nombre d'unités minimales dans 1 AUR (10^8).
pub const UNITS_PER_COIN: u64 = 100_000_000;
/// Plafond de garde-fou en AUR (proposition de travail).
pub const MAX_COINS: u64 = 500_000_000;
/// Plafond hypothétique en unités minimales.
pub const MAX_UNITS: u64 = MAX_COINS * UNITS_PER_COIN;

/// Erreur d'arithmétique sur les montants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmountError {
    /// Le montant dépasse le plafond hypothétique.
    AboveCap,
    /// Dépassement de capacité lors d'une addition.
    Overflow,
    /// Résultat négatif lors d'une soustraction.
    Underflow,
}

impl fmt::Display for AmountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AmountError::AboveCap => write!(f, "montant au-dessus du plafond"),
            AmountError::Overflow => write!(f, "dépassement de capacité"),
            AmountError::Underflow => write!(f, "résultat négatif"),
        }
    }
}

impl std::error::Error for AmountError {}

/// Montant en unités minimales, toujours inférieur ou égal à [`MAX_UNITS`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amount(u64);

impl Amount {
    /// Montant nul.
    pub const ZERO: Amount = Amount(0);
    /// Montant maximal admis par le prototype.
    pub const MAX: Amount = Amount(MAX_UNITS);

    /// Crée un montant à partir d'unités minimales.
    pub fn from_units(units: u64) -> Result<Self, AmountError> {
        if units > MAX_UNITS {
            Err(AmountError::AboveCap)
        } else {
            Ok(Amount(units))
        }
    }

    /// Valeur en unités minimales.
    pub const fn to_units(self) -> u64 {
        self.0
    }

    /// Addition vérifiée : jamais de dépassement silencieux, jamais au-dessus du plafond.
    pub fn checked_add(self, other: Amount) -> Result<Amount, AmountError> {
        let sum = self.0.checked_add(other.0).ok_or(AmountError::Overflow)?;
        Amount::from_units(sum)
    }

    /// Soustraction vérifiée.
    pub fn checked_sub(self, other: Amount) -> Result<Amount, AmountError> {
        self.0
            .checked_sub(other.0)
            .map(Amount)
            .ok_or(AmountError::Underflow)
    }

    /// Somme vérifiée d'une suite de montants.
    pub fn checked_sum<I: IntoIterator<Item = Amount>>(items: I) -> Result<Amount, AmountError> {
        let mut total = Amount::ZERO;
        for item in items {
            total = total.checked_add(item)?;
        }
        Ok(total)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.0 / UNITS_PER_COIN;
        let frac = self.0 % UNITS_PER_COIN;
        write!(f, "{whole}.{frac:0width$}", width = DECIMALS)
    }
}

impl CanonicalEncode for Amount {
    fn encode(&self, enc: &mut Encoder) -> Result<(), EncodeError> {
        enc.put_u64(self.0);
        Ok(())
    }
}

impl CanonicalDecode for Amount {
    fn decode(dec: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let units = dec.get_u64()?;
        Amount::from_units(units)
            .map_err(|_| DecodeError::InvalidValue("montant au-dessus du plafond expérimental"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_per_coin_matches_decimals() {
        assert_eq!(10u64.pow(DECIMALS as u32), UNITS_PER_COIN);
    }

    #[test]
    fn cap_fits_in_u64_with_room() {
        assert_eq!(MAX_UNITS, 50_000_000_000_000_000);
    }

    #[test]
    fn from_units_accepts_bounds_and_rejects_above() {
        assert_eq!(Amount::from_units(0).unwrap(), Amount::ZERO);
        assert_eq!(Amount::from_units(MAX_UNITS).unwrap(), Amount::MAX);
        assert_eq!(
            Amount::from_units(MAX_UNITS + 1),
            Err(AmountError::AboveCap)
        );
        assert_eq!(Amount::from_units(u64::MAX), Err(AmountError::AboveCap));
    }

    #[test]
    fn checked_add_detects_cap_and_overflow() {
        let one = Amount::from_units(1).unwrap();
        assert_eq!(Amount::MAX.checked_add(one), Err(AmountError::AboveCap));
        assert_eq!(one.checked_add(one).unwrap().to_units(), 2);
    }

    #[test]
    fn checked_sub_detects_underflow() {
        let one = Amount::from_units(1).unwrap();
        assert_eq!(Amount::ZERO.checked_sub(one), Err(AmountError::Underflow));
        assert_eq!(one.checked_sub(one).unwrap(), Amount::ZERO);
    }

    #[test]
    fn checked_sum_works_and_fails_above_cap() {
        let a = Amount::from_units(10).unwrap();
        let b = Amount::from_units(32).unwrap();
        assert_eq!(Amount::checked_sum([a, b]).unwrap().to_units(), 42);
        assert_eq!(
            Amount::checked_sum([Amount::MAX, a]),
            Err(AmountError::AboveCap)
        );
        assert_eq!(Amount::checked_sum([]).unwrap(), Amount::ZERO);
    }

    #[test]
    fn display_uses_eight_decimals() {
        assert_eq!(Amount::ZERO.to_string(), "0.00000000");
        assert_eq!(Amount::from_units(1).unwrap().to_string(), "0.00000001");
        assert_eq!(
            Amount::from_units(150_000_000).unwrap().to_string(),
            "1.50000000"
        );
        assert_eq!(Amount::MAX.to_string(), "500000000.00000000");
    }

    #[test]
    fn codec_roundtrip_and_cap_enforced_on_decode() {
        let amount = Amount::from_units(123_456_789).unwrap();
        let bytes = amount.to_canonical_bytes().unwrap();
        assert_eq!(bytes.len(), 8);
        assert_eq!(Amount::from_canonical_bytes(&bytes).unwrap(), amount);

        let too_big = (MAX_UNITS + 1).to_be_bytes();
        assert!(matches!(
            Amount::from_canonical_bytes(&too_big),
            Err(DecodeError::InvalidValue(_))
        ));
    }
}

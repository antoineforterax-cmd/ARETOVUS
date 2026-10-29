//! Local payment review and voluntary limits, independent of consensus.
use std::io::{self, BufRead, Read};

use aurora_core::{accounts::AccountId, amount::Amount};

#[derive(Clone, Copy, Debug, Default)]
pub struct PaymentLimits {
    pub max_amount: Option<Amount>,
    pub max_fee: Option<Amount>,
}

pub fn check_payment(
    sender: AccountId,
    recipient: AccountId,
    amount: Amount,
    fee: Amount,
    balance: Amount,
    limits: PaymentLimits,
) -> Result<Amount, &'static str> {
    if sender == recipient {
        return Err("sender and recipient must differ");
    }
    if amount == Amount::ZERO {
        return Err("payment amount must be positive");
    }
    if limits.max_amount.is_some_and(|limit| amount > limit) {
        return Err("payment exceeds the voluntary amount limit");
    }
    if limits.max_fee.is_some_and(|limit| fee > limit) {
        return Err("payment exceeds the voluntary fee limit");
    }
    let debit = amount
        .checked_add(fee)
        .map_err(|_| "payment total exceeds monetary bounds")?;
    if debit > balance {
        return Err("insufficient pending-adjusted balance");
    }
    Ok(debit)
}

/// EOF, cancellation and any different destination fail closed.
pub fn read_confirmation(reader: &mut impl BufRead, destination: &str) -> io::Result<bool> {
    let mut line = String::new();
    // Bound input even when stdin is supplied by another process.
    reader.take(128).read_line(&mut line)?;
    if !line.ends_with('\n') {
        return Ok(false);
    }
    Ok(line.trim_end_matches(['\r', '\n']) == format!("SEND {destination}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn units(value: u64) -> Amount {
        Amount::from_units(value).unwrap()
    }

    #[test]
    fn voluntary_limits_and_debit_are_exact() {
        let a = AccountId([1; 32]);
        let b = AccountId([2; 32]);
        let limits = PaymentLimits {
            max_amount: Some(units(10)),
            max_fee: Some(units(2)),
        };
        assert_eq!(
            check_payment(a, b, units(10), units(2), units(12), limits),
            Ok(units(12))
        );
        assert!(check_payment(a, b, units(11), units(1), units(100), limits).is_err());
        assert!(check_payment(a, b, units(9), units(3), units(100), limits).is_err());
        assert!(check_payment(a, b, units(10), units(2), units(11), limits).is_err());
        assert!(check_payment(a, a, units(1), units(0), units(100), limits).is_err());
        assert!(check_payment(a, b, units(0), units(0), units(100), limits).is_err());
        assert!(check_payment(
            a,
            b,
            Amount::MAX,
            units(1),
            Amount::MAX,
            PaymentLimits::default()
        )
        .is_err());
    }

    #[test]
    fn confirmation_requires_the_complete_exact_destination() {
        let destination = "ab".repeat(32);
        for text in [
            "".to_owned(),
            "yes\n".to_owned(),
            format!("SEND {destination}"),
            "x".repeat(4096),
            format!("SEND {}\n", "ac".repeat(32)),
            format!("SEND {destination} extra\n"),
        ] {
            assert!(!read_confirmation(&mut text.as_bytes(), &destination).unwrap());
        }
        assert!(read_confirmation(
            &mut format!("SEND {destination}\r\n").as_bytes(),
            &destination
        )
        .unwrap());
    }
}

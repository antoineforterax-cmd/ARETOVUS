//! Local payment review and voluntary limits, independent of consensus.
use std::io::{self, BufRead, Read};

use aretovus_core::{accounts::AccountId, amount::Amount};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaymentLimits {
    pub max_amount: Option<Amount>,
    pub max_fee: Option<Amount>,
}

impl PaymentLimits {
    /// Compose persistent and one-shot limits by retaining the stricter bound.
    pub fn stricter(self, other: Self) -> Self {
        fn minimum<T: Ord>(a: Option<T>, b: Option<T>) -> Option<T> {
            match (a, b) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, None) | (None, a) => a,
            }
        }
        Self {
            max_amount: minimum(self.max_amount, other.max_amount),
            max_fee: minimum(self.max_fee, other.max_fee),
        }
    }
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

/// Complete, deterministic authorization text. Always present the exact
/// phrase to the signer and compare against independently entered input.
pub fn payment_intent_confirmation_phrase(
    destination: &str,
    amount: Amount,
    fee: Amount,
    nonce: u64,
) -> String {
    format!("SEND ARETOVUS-TESTNET {destination} {amount} {fee} {nonce}")
}

/// Verify the signed bytes against the independently reviewed CLI intent.
/// This catches accidental field substitution or signing a different payload.
/// It cannot detect a malicious executable that controls both sides of the check.
pub fn verify_signed_payment_intent(
    transaction: &aretovus_core::accounts::AccountTransaction,
    sender: AccountId,
    recipient: AccountId,
    amount: Amount,
    fee: Amount,
    nonce: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    use aretovus_core::{accounts::POST_QUANTUM_ACCOUNT_TX_VERSION, network::NetworkId};
    if transaction.version != POST_QUANTUM_ACCOUNT_TX_VERSION
        || transaction.network != NetworkId::Testnet
        || transaction.sender != sender
        || transaction.recipient != recipient
        || transaction.amount != amount
        || transaction.fee != fee
        || transaction.nonce != nonce
    {
        return Err("signed transaction differs from reviewed payment intent".into());
    }
    crate::signed_payment::validate(transaction)?;
    Ok(())
}

/// Require a human to independently repeat the transaction's network,
/// complete recipient, amount, fee and nonce before any signing or broadcast.
///
/// A correct confirmation does not prove the terminal or wallet binary is honest.
/// It does protect against accidental approval of a different displayed intent.
pub fn read_payment_intent_confirmation(
    reader: &mut impl BufRead,
    destination: &str,
    amount: Amount,
    fee: Amount,
    nonce: u64,
) -> io::Result<bool> {
    let expected = payment_intent_confirmation_phrase(destination, amount, fee, nonce);
    // More than enough for 64 hex digits and the maximum u64-formatted amounts.
    // EOF, overlong lines and anything but one complete answer fail closed.
    let mut line = String::new();
    reader.take(192).read_line(&mut line)?;
    let Some(answer) = line.strip_suffix('\n') else {
        return Ok(false);
    };
    let answer = answer.strip_suffix('\r').unwrap_or(answer);
    Ok(answer == expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn units(value: u64) -> Amount {
        Amount::from_units(value).unwrap()
    }

    #[test]
    fn one_shot_limits_can_only_tighten_persistent_limits() {
        let stored = PaymentLimits {
            max_amount: Some(units(10)),
            max_fee: Some(units(2)),
        };
        assert_eq!(stored.stricter(PaymentLimits::default()), stored);
        let looser = PaymentLimits {
            max_amount: Some(units(100)),
            max_fee: Some(units(20)),
        };
        assert_eq!(stored.stricter(looser), stored);
        let tighter = PaymentLimits {
            max_amount: Some(units(5)),
            max_fee: Some(units(1)),
        };
        assert_eq!(stored.stricter(tighter), tighter);
        assert_eq!(PaymentLimits::default().stricter(stored), stored);
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
    fn signed_payment_must_match_the_reviewed_intent_even_if_resigned() {
        use crate::{build_signed_transaction, WalletKey};
        let key = WalletKey::from_seed_bytes([98; 32]);
        let sender = key.account_id();
        let recipient = AccountId([0x39; 32]);
        let amount = units(100);
        let fee = units(5);
        let nonce = 7;
        let transaction = build_signed_transaction(&key, recipient, amount, fee, nonce).unwrap();
        assert!(
            verify_signed_payment_intent(&transaction, sender, recipient, amount, fee, nonce)
                .is_ok()
        );
        let altered = [
            build_signed_transaction(&key, AccountId([0x40; 32]), amount, fee, nonce).unwrap(),
            build_signed_transaction(&key, recipient, units(101), fee, nonce).unwrap(),
            build_signed_transaction(&key, recipient, amount, units(6), nonce).unwrap(),
            build_signed_transaction(&key, recipient, amount, fee, nonce + 1).unwrap(),
        ];
        for signed in &altered {
            assert!(
                verify_signed_payment_intent(signed, sender, recipient, amount, fee, nonce)
                    .is_err(),
                "resigning a changed field must not bypass intent review"
            );
        }
        assert!(verify_signed_payment_intent(
            &transaction,
            AccountId([0x50; 32]),
            recipient,
            amount,
            fee,
            nonce
        )
        .is_err());
        let mut tampered = transaction;
        tampered.auth_data[100] ^= 1;
        assert!(
            verify_signed_payment_intent(&tampered, sender, recipient, amount, fee, nonce).is_err()
        );
    }

    #[test]
    fn approval_binds_all_spend_fields_and_rejects_mutations() {
        let destination = "ab".repeat(32);
        let amount = units(1_000_000_000);
        let fee = units(1_000_000);
        let nonce = 42;
        let phrase = payment_intent_confirmation_phrase(&destination, amount, fee, nonce);
        assert!(phrase.starts_with("SEND ARETOVUS-TESTNET "));
        for valid in [format!("{phrase}\n"), format!("{phrase}\r\n")] {
            assert!(read_payment_intent_confirmation(
                &mut valid.as_bytes(),
                &destination,
                amount,
                fee,
                nonce
            )
            .unwrap());
        }
        let alternatives = [
            phrase.clone(),
            format!("{}\n", phrase.replace("ARETOVUS-TESTNET", "ARETOVUS-MAINNET")),
            format!(
                "{}\n",
                payment_intent_confirmation_phrase(&"ac".repeat(32), amount, fee, nonce)
            ),
            format!(
                "{}\n",
                payment_intent_confirmation_phrase(&destination, units(1_100_000_000), fee, nonce)
            ),
            format!(
                "{}\n",
                payment_intent_confirmation_phrase(&destination, amount, units(2_000_000), nonce)
            ),
            format!(
                "{}\n",
                payment_intent_confirmation_phrase(&destination, amount, fee, nonce + 1)
            ),
            format!("{phrase} trailing\n"),
            format!(" {phrase}\n"),
            "x".repeat(1024),
        ];
        for candidate in alternatives {
            assert!(
                !read_payment_intent_confirmation(
                    &mut candidate.as_bytes(),
                    &destination,
                    amount,
                    fee,
                    nonce
                )
                .unwrap(),
                "unexpectedly accepted {candidate:?}"
            );
        }
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

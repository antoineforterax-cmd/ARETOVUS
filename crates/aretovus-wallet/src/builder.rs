//! Canonical v3 transaction construction and ARETOVUS amount parsing.
use std::error::Error;

use aretovus_core::{
    accounts::{AccountId, AccountTransaction, POST_QUANTUM_ACCOUNT_TX_VERSION},
    amount::{Amount, UNITS_PER_COIN},
    network::NetworkId,
};
use ml_dsa::Signer;

use crate::keys::WalletKey;

/// Parse a non-negative decimal ARETOVUS amount without floating-point arithmetic.
pub fn parse_amount(value: &str) -> Result<Amount, Box<dyn Error>> {
    let (whole, fraction) = match value.split_once('.') {
        Some((whole, fraction)) if !fraction.contains('.') => (whole, fraction),
        Some(_) => return Err("amount must contain at most one decimal point".into()),
        None => (value, ""),
    };
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() > 8
    {
        return Err("amount must be a non-negative decimal with at most 8 decimals".into());
    }
    let whole: u64 = whole.parse()?;
    let mut fractional: u64 = if fraction.is_empty() {
        0
    } else {
        fraction.parse()?
    };
    for _ in fraction.len()..8 {
        fractional *= 10;
    }
    let units = whole
        .checked_mul(UNITS_PER_COIN)
        .and_then(|n| n.checked_add(fractional))
        .ok_or("amount overflow")?;
    Ok(Amount::from_units(units)?)
}

/// Decode the current 32-byte account identifier representation (64 lowercase/uppercase hex).
pub fn parse_account_id(value: &str) -> Result<AccountId, Box<dyn Error>> {
    Ok(AccountId::from_hex(value)?)
}

pub fn account_id_hex(id: AccountId) -> String {
    let mut out = String::with_capacity(64);
    for byte in id.0 {
        use std::fmt::Write as _;
        write!(&mut out, "{byte:02x}").expect("writing to String cannot fail");
    }
    out
}

/// Build and sign a native ML-DSA-65 testnet transfer.
pub fn build_signed_transaction(
    key: &WalletKey,
    recipient: AccountId,
    amount: Amount,
    fee: Amount,
    nonce: u64,
) -> Result<AccountTransaction, Box<dyn Error>> {
    let mut tx = AccountTransaction {
        version: POST_QUANTUM_ACCOUNT_TX_VERSION,
        network: NetworkId::Testnet,
        sender: key.account_id(),
        nonce,
        recipient,
        amount,
        fee,
        auth_data: vec![0; aretovus_core::accounts::POST_QUANTUM_AUTH_BYTES],
    };
    let public_key = key.public_key();
    tx.auth_data[..aretovus_core::accounts::ML_DSA_65_PUBLIC_KEY_BYTES].copy_from_slice(public_key);
    let signature = key
        .signing_key()
        .sign(tx.signing_digest()?.as_bytes())
        .encode();
    tx.auth_data[aretovus_core::accounts::ML_DSA_65_PUBLIC_KEY_BYTES..]
        .copy_from_slice(signature.as_slice());
    tx.check_stateless()?;
    Ok(tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amount_parser_is_exact_and_bounded() {
        assert_eq!(parse_amount("1").unwrap().to_units(), 100_000_000);
        assert_eq!(parse_amount("0.00000001").unwrap().to_units(), 1);
        assert_eq!(parse_amount("12.34005").unwrap().to_units(), 1_234_005_000);
        assert!(parse_amount("-1").is_err());
        assert!(parse_amount("1.000000001").is_err());
        assert!(parse_amount("1.2.3").is_err());
    }

    #[test]
    fn account_id_hex_roundtrips() {
        let id = AccountId([0xab; 32]);
        assert_eq!(parse_account_id(&account_id_hex(id)).unwrap(), id);
    }

    #[test]
    fn built_v3_transaction_verifies_and_has_expected_auth_size() {
        let key = WalletKey::from_seed_bytes([19; 32]);
        let tx = build_signed_transaction(
            &key,
            AccountId([0x25; 32]),
            Amount::from_units(10).unwrap(),
            Amount::from_units(1).unwrap(),
            0,
        )
        .unwrap();
        assert_eq!(tx.auth_data.len(), 5_261);
        assert!(aretovus_core::signature::verify_account_transaction(&tx).is_ok());
    }
}

#[cfg(test)]
mod untrusted_account_input_tests {
    use super::*;

    #[test]
    fn wallet_rejects_unicode_and_signed_hex_pairs() {
        let unicode = format!("a€{}", "0".repeat(60));
        assert_eq!(unicode.len(), 64);
        assert!(parse_account_id(&unicode).is_err());
        assert!(parse_account_id(&format!("+f{}", "0".repeat(62))).is_err());
    }
}

//! Canonical v3 transaction construction and AUR amount parsing.
use std::error::Error;

use aurora_core::{
    accounts::{AccountId, AccountTransaction, POST_QUANTUM_ACCOUNT_TX_VERSION},
    amount::{Amount, UNITS_PER_COIN},
    network::NetworkId,
};
use ml_dsa::Signer;

use crate::keys::WalletKey;

/// Parse a non-negative decimal AUR amount without floating-point arithmetic.
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
    if value.len() != 64 {
        return Err("account id must contain exactly 64 hexadecimal characters".into());
    }
    let mut bytes = [0u8; 32];
    for (i, slot) in bytes.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&value[i * 2..i * 2 + 2], 16)?;
    }
    Ok(AccountId(bytes))
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
        auth_data: vec![0; aurora_core::accounts::POST_QUANTUM_AUTH_BYTES],
    };
    let public_key = key.public_key();
    tx.auth_data[..aurora_core::accounts::ML_DSA_65_PUBLIC_KEY_BYTES]
        .copy_from_slice(public_key);
    let signature = key
        .signing_key()
        .sign(tx.signing_digest()?.as_bytes())
        .encode();
    tx.auth_data[aurora_core::accounts::ML_DSA_65_PUBLIC_KEY_BYTES..]
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
        assert!(aurora_core::signature::verify_account_transaction(&tx).is_ok());
    }
}

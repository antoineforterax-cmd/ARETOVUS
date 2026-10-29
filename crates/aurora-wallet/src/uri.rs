//! Strict experimental aurora://testnet payment request encoding.
use std::error::Error;

use aurora_core::{accounts::AccountId, amount::Amount};

use crate::builder::{account_id_hex, parse_account_id, parse_amount};

const PREFIX: &str = "aurora://testnet/";
const MAX_MEMO_BYTES: usize = 128;
const MAX_URI_BYTES: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentRequest {
    pub account: AccountId,
    pub amount: Option<Amount>,
    pub memo: Option<String>,
}

pub fn encode_payment_uri(request: &PaymentRequest) -> Result<String, Box<dyn Error>> {
    if request.amount.is_some_and(|amount| amount.to_units() == 0) {
        return Err("payment amount must be positive".into());
    }
    if let Some(memo) = &request.memo {
        validate_memo(memo)?;
    }
    let mut uri = format!("{PREFIX}{}", account_id_hex(request.account));
    let mut query = Vec::new();
    if let Some(amount) = request.amount {
        query.push(format!("amount={}", format_amount(amount)));
    }
    if let Some(memo) = &request.memo {
        query.push(format!("memo={}", percent_encode(memo.as_bytes())));
    }
    if !query.is_empty() {
        uri.push('?');
        uri.push_str(&query.join("&"));
    }
    Ok(uri)
}

pub fn parse_payment_uri(uri: &str) -> Result<PaymentRequest, Box<dyn Error>> {
    if uri.len() > MAX_URI_BYTES {
        return Err("payment URI exceeds 1024 bytes".into());
    }
    if !uri.is_ascii() || uri.contains('#') {
        return Err("payment URI must be ASCII encoded and contain no fragment".into());
    }
    let rest = uri
        .strip_prefix(PREFIX)
        .ok_or("unsupported payment URI scheme or network")?;
    let (path, query) = match rest.split_once('?') {
        Some((path, query)) => (path, Some(query)),
        None => (rest, None),
    };
    let account = parse_account_id(path)?;
    if account_id_hex(account) != path {
        return Err("payment URI account must use lowercase canonical hexadecimal".into());
    }

    let mut amount = None;
    let mut memo = None;
    if let Some(query) = query {
        if query.is_empty() {
            return Err("payment URI query cannot be empty".into());
        }
        for item in query.split('&') {
            let (key, value) = item
                .split_once('=')
                .ok_or("payment URI query item must contain '='")?;
            match key {
                "amount" if amount.is_none() => {
                    let parsed = parse_amount(value)?;
                    if parsed.to_units() == 0 {
                        return Err("payment amount must be positive".into());
                    }
                    amount = Some(parsed);
                }
                "memo" if memo.is_none() => {
                    let bytes = percent_decode(value)?;
                    let text = String::from_utf8(bytes)?;
                    validate_memo(&text)?;
                    memo = Some(text);
                }
                "amount" | "memo" => return Err("duplicate payment URI parameter".into()),
                _ => return Err("unknown payment URI parameter".into()),
            }
        }
    }
    Ok(PaymentRequest {
        account,
        amount,
        memo,
    })
}

fn format_amount(amount: Amount) -> String {
    let units = amount.to_units();
    let whole = units / 100_000_000;
    let fraction = units % 100_000_000;
    if fraction == 0 {
        return whole.to_string();
    }
    let decimal = format!("{fraction:08}");
    format!("{whole}.{}", decimal.trim_end_matches('0'))
}

fn validate_memo(memo: &str) -> Result<(), Box<dyn Error>> {
    if memo.len() > MAX_MEMO_BYTES {
        return Err("payment memo exceeds 128 UTF-8 bytes".into());
    }
    if memo.chars().any(|ch| {
        ch.is_control() || matches!(ch, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
    }) {
        return Err("payment memo cannot contain control or directional formatting characters".into());
    }
    Ok(())
}

fn percent_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(bytes.len());
    for &byte in bytes {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }
    out
}

fn percent_decode(value: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if value.len() > MAX_MEMO_BYTES * 3 {
        return Err("encoded payment memo exceeds size limit".into());
    }
    let bytes = value.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut cursor = 0;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'%' => {
                let end = cursor.checked_add(3).ok_or("memo encoding overflow")?;
                let escape = bytes
                    .get(cursor + 1..end)
                    .ok_or("truncated percent escape")?;
                let high = hex_nibble(escape[0]).ok_or("invalid percent escape")?;
                let low = hex_nibble(escape[1]).ok_or("invalid percent escape")?;
                output.push((high << 4) | low);
                cursor = end;
            }
            byte if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') => {
                output.push(byte);
                cursor += 1;
            }
            _ => return Err("memo must percent-encode reserved and non-ASCII characters".into()),
        }
    }
    if output.len() > MAX_MEMO_BYTES {
        return Err("payment memo exceeds 128 UTF-8 bytes".into());
    }
    Ok(output)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_uri_roundtrips_amount_and_utf8_memo() {
        let request = PaymentRequest {
            account: AccountId([0xab; 32]),
            amount: Some(parse_amount("12.3405").unwrap()),
            memo: Some("café & thé".to_owned()),
        };
        let uri = encode_payment_uri(&request).unwrap();
        assert!(uri.starts_with("aurora://testnet/"));
        assert!(uri.contains("amount=12.3405"));
        assert!(uri.contains("caf%C3%A9%20%26%20th%C3%A9"));
        assert_eq!(parse_payment_uri(&uri).unwrap(), request);
    }

    #[test]
    fn payment_uri_rejects_network_confusion_duplicates_and_bad_escapes() {
        let address = "ab".repeat(32);
        assert!(parse_payment_uri(&format!("aurora://mainnet/{address}")).is_err());
        assert!(
            parse_payment_uri(&format!("aurora://testnet/{address}?amount=1&amount=2")).is_err()
        );
        assert!(parse_payment_uri(&format!("aurora://testnet/{address}?memo=%Q0")).is_err());
        assert!(parse_payment_uri(&format!("aurora://testnet/{address}?unexpected=1")).is_err());
        assert!(parse_payment_uri(&format!("aurora://testnet/{address}#fragment")).is_err());
    }

    #[test]
    fn payment_uri_rejects_zero_or_overprecision_and_control_memos() {
        let address = "ab".repeat(32);
        assert!(parse_payment_uri(&format!("aurora://testnet/{address}?amount=0")).is_err());
        assert!(
            parse_payment_uri(&format!("aurora://testnet/{address}?amount=1.000000001")).is_err()
        );
        assert!(parse_payment_uri(&format!("aurora://testnet/{address}?memo=%0A")).is_err());
    }

    #[test]
    fn encoder_uses_minimal_exact_amount_decimal() {
        let request = PaymentRequest {
            account: AccountId([0; 32]),
            amount: Some(Amount::from_units(120_000_000).unwrap()),
            memo: None,
        };
        assert!(encode_payment_uri(&request)
            .unwrap()
            .ends_with("?amount=1.2"));
    }
}

#[cfg(test)]
mod resource_and_display_tests {
    use super::*;

    #[test]
    fn maximum_valid_memo_roundtrips_but_oversize_input_is_rejected() {
        let request = PaymentRequest {
            account: AccountId([3; 32]),
            amount: Some(Amount::MAX),
            memo: Some(" ".repeat(MAX_MEMO_BYTES)),
        };
        let uri = encode_payment_uri(&request).unwrap();
        assert!(uri.len() <= MAX_URI_BYTES);
        assert_eq!(parse_payment_uri(&uri).unwrap(), request);
        assert!(parse_payment_uri(&format!("aurora://testnet/{}?memo={}", "03".repeat(32), "%20".repeat(129))).is_err());
        assert!(parse_payment_uri(&"x".repeat(MAX_URI_BYTES + 1)).is_err());
        assert!(percent_decode(&"%20".repeat(MAX_MEMO_BYTES + 1)).is_err());
    }

    #[test]
    fn directional_controls_are_rejected_without_banning_visible_utf8() {
        for ch in ['\u{061c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202e}', '\u{2066}', '\u{2069}'] {
            let request = PaymentRequest { account: AccountId([3; 32]), amount: None, memo: Some(format!("invoice{ch}123")) };
            assert!(encode_payment_uri(&request).is_err());
            let uri = format!("aurora://testnet/{}?memo={}", "03".repeat(32), percent_encode(request.memo.as_ref().unwrap().as_bytes()));
            assert!(parse_payment_uri(&uri).is_err());
        }
        let request = PaymentRequest { account: AccountId([3; 32]), amount: None, memo: Some("café مرحبا".to_owned()) };
        assert_eq!(parse_payment_uri(&encode_payment_uri(&request).unwrap()).unwrap(), request);
    }
}

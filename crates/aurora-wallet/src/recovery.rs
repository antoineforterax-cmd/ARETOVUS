//! Printable, checksummed recovery seed for the existing v3 testnet identity.
//! This is a secret backup encoding, not encryption or key rotation.
use zeroize::Zeroizing;

use aurora_core::hash::sha256_plain;

const PREFIX: &str = "aurora-testnet-recovery-v1:";
const CODE_LEN: usize = PREFIX.len() + 64 + 1 + 64;
const HEX: &[u8; 16] = b"0123456789abcdef";

pub(crate) fn encode_seed(seed: &[u8; 32]) -> Zeroizing<String> {
    let mut code = Zeroizing::new(String::with_capacity(CODE_LEN + 2));
    code.push_str(PREFIX);
    for &byte in seed {
        code.push(HEX[usize::from(byte >> 4)] as char);
        code.push(HEX[usize::from(byte & 15)] as char);
    }
    let mut checked = Zeroizing::new(Vec::with_capacity(PREFIX.len() + 32));
    checked.extend_from_slice(PREFIX.as_bytes());
    checked.extend_from_slice(seed);
    code.push(':');
    code.push_str(&sha256_plain(&checked).to_hex());
    code
}

pub(crate) fn decode_seed(code: &str) -> Result<Zeroizing<[u8; 32]>, &'static str> {
    // Permit one normal text-file line ending; never normalize secret characters.
    let code = code
        .strip_suffix("\r\n")
        .or_else(|| code.strip_suffix('\n'))
        .unwrap_or(code);
    if code.len() != CODE_LEN || !code.starts_with(PREFIX) {
        return Err("invalid testnet recovery code format");
    }
    let mut seed = Zeroizing::new([0u8; 32]);
    let encoded = &code.as_bytes()[PREFIX.len()..PREFIX.len() + 64];
    for (byte, pair) in seed.iter_mut().zip(encoded.as_chunks::<2>().0) {
        let digit = |value| match value {
            b'0'..=b'9' => Ok(value - b'0'),
            b'a'..=b'f' => Ok(value - b'a' + 10),
            _ => Err("invalid testnet recovery code format"),
        };
        *byte = (digit(pair[0])? << 4) | digit(pair[1])?;
    }
    if encode_seed(&seed).as_bytes() != code.as_bytes() {
        return Err("recovery code checksum mismatch");
    }
    Ok(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_roundtrips_with_fixed_prefix_and_explicit_line_endings() {
        let seed = [0xab; 32];
        let code = encode_seed(&seed);
        assert!(code.starts_with(&format!("{PREFIX}{}", "ab".repeat(32))));
        assert_eq!(code.len(), CODE_LEN);
        assert_eq!(*decode_seed(&code).unwrap(), seed);
        for ending in ["\n", "\r\n"] {
            let mut text = Zeroizing::new(code.to_string());
            text.push_str(ending);
            assert_eq!(*decode_seed(&text).unwrap(), seed);
        }
    }

    #[test]
    fn every_single_character_change_and_truncation_is_rejected() {
        let code = encode_seed(&[0x42; 32]);
        for index in 0..code.len() {
            let mut corrupt = Zeroizing::new(code.as_bytes().to_vec());
            corrupt[index] = if corrupt[index] == b'0' { b'1' } else { b'0' };
            assert!(decode_seed(std::str::from_utf8(&corrupt).unwrap()).is_err());
            assert!(decode_seed(&code[..index]).is_err());
        }
        for text in ["", "é", "aurora-mainnet-recovery-v1:", "+f"] {
            assert!(decode_seed(text).is_err());
        }
        let mut trailing = Zeroizing::new(code.to_string());
        trailing.push(' ');
        assert!(decode_seed(&trailing).is_err());
        trailing.push_str(&"x".repeat(4096));
        assert!(decode_seed(&trailing).is_err());
    }

    #[test]
    fn arbitrary_utf8_never_panics() {
        // Byte slicing is only used after the fixed byte-length and ASCII prefix checks.
        let mut text = PREFIX.to_owned();
        text.push_str(&"é".repeat(64));
        text.push('x');
        assert_eq!(text.len(), CODE_LEN);
        assert!(decode_seed(&text).is_err());
    }
}

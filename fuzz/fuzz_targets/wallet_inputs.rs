#![no_main]

use aretovus_wallet::uri::{encode_payment_uri, parse_payment_uri};
use libfuzzer_sys::fuzz_target;

fn check_roundtrip(uri: &str) {
    if let Ok(request) = parse_payment_uri(uri) {
        let canonical = encode_payment_uri(&request).expect("accepted request must encode");
        let decoded = parse_payment_uri(&canonical).expect("encoded request must decode");
        assert_eq!(request, decoded);
        assert_eq!(
            canonical,
            encode_payment_uri(&decoded).expect("canonical re-encoding must succeed")
        );
    }
}

fuzz_target!(|bytes: &[u8]| {
    if bytes.len() > 1024 {
        return;
    }
    if let Ok(uri) = std::str::from_utf8(bytes) {
        check_roundtrip(uri);
        // Exercise percent-decoding and length guards with a valid account prefix.
        let synthesized = format!(
            "aretovus://testnet/{}?memo={uri}",
            "0".repeat(64)
        );
        check_roundtrip(&synthesized);
        let synthesized_amount = format!(
            "aretovus://testnet/{}?amount={uri}",
            "0".repeat(64)
        );
        check_roundtrip(&synthesized_amount);
    }
});

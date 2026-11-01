//! Public NIST ACVP examples for the exact ML-DSA-65 dependency used by ARETOVUS.
//! This is a regression suite, not an ACVTS validation or FIPS certificate.

use ml_dsa::{
    EncodedSignature, EncodedVerifyingKey, ExpandedSigningKey, ExpandedSigningKeyBytes, Keypair,
    MlDsa65, Seed, Signature, Signer, SigningKey, Verifier, VerifyingKey,
};
use std::collections::BTreeMap;

type Case = BTreeMap<&'static str, &'static str>;

fn cases(input: &'static str) -> Vec<Case> {
    let mut cases = Vec::new();
    let mut case = Case::new();
    for line in input.lines() {
        if line.starts_with('#') {
            continue;
        }
        if line.is_empty() {
            if !case.is_empty() {
                cases.push(std::mem::take(&mut case));
            }
            continue;
        }
        let (name, value) = line.split_once('=').expect("fixture field");
        assert!(
            case.insert(name, value).is_none(),
            "duplicate fixture field"
        );
    }
    if !case.is_empty() {
        cases.push(case);
    }
    cases
}

fn hex(input: &str) -> Vec<u8> {
    assert!(input.len().is_multiple_of(2), "odd hex length");
    input
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let high = char::from(pair[0]).to_digit(16).expect("hex digit");
            let low = char::from(pair[1]).to_digit(16).expect("hex digit");
            ((high << 4) | low) as u8
        })
        .collect()
}

fn verifying_key(case: &Case) -> VerifyingKey<MlDsa65> {
    let mut encoded = EncodedVerifyingKey::<MlDsa65>::default();
    encoded.copy_from_slice(&hex(case["pk"]));
    VerifyingKey::decode(&encoded)
}

#[test]
fn nist_acvp_ml_dsa_65_key_generation_public_keys() {
    let cases = cases(include_str!("../../../tests/vectors-fips204/keygen.txt"));
    assert_eq!(cases.len(), 25);
    for case in cases {
        let mut seed = Seed::default();
        seed.copy_from_slice(&hex(case["seed"]));
        let key = SigningKey::<MlDsa65>::from_seed(&seed);
        assert_eq!(
            key.verifying_key().encode().as_slice(),
            hex(case["pk"]),
            "NIST keyGen tcId={}",
            case["tcId"]
        );
    }
}

#[test]
fn nist_acvp_ml_dsa_65_external_pure_signature_verification() {
    let cases = cases(include_str!("../../../tests/vectors-fips204/sigver.txt"));
    assert_eq!(cases.len(), 15);
    assert_eq!(
        cases
            .iter()
            .filter(|case| case["testPassed"] == "true")
            .count(),
        3
    );
    for case in cases {
        let key = verifying_key(&case);
        let message = hex(case["message"]);
        let context = hex(case["context"]);
        let mut encoded = EncodedSignature::<MlDsa65>::default();
        encoded.copy_from_slice(&hex(case["signature"]));
        let signature = Signature::decode(&encoded);
        let accepted = signature
            .as_ref()
            .is_some_and(|signature| key.verify_with_context(&message, &context, signature));
        assert_eq!(
            accepted,
            case["testPassed"] == "true",
            "NIST sigVer tcId={} ({})",
            case["tcId"],
            case["reason"]
        );
        if let Some(signature) = signature.filter(|_| accepted) {
            let mut changed_context = context;
            // The valid examples in this pinned group all have a nonempty context.
            changed_context[0] ^= 1;
            assert!(!key.verify_with_context(&message, &changed_context, &signature));
            assert!(!key.verify_with_context(&message, &[0; 256], &signature));
        }
    }
}

#[test]
fn nist_acvp_ml_dsa_65_deterministic_signatures_match_byte_for_byte() {
    let cases = cases(include_str!("../../../tests/vectors-fips204/siggen.txt"));
    assert_eq!(cases.len(), 3);
    for case in cases {
        let mut expanded = ExpandedSigningKeyBytes::<MlDsa65>::default();
        expanded.copy_from_slice(&hex(case["sk"]));
        // NIST publishes these fixed known-good test keys in expanded form.
        // Production wallet loading uses a seed and never calls this API.
        #[allow(deprecated)]
        let key = ExpandedSigningKey::<MlDsa65>::from_expanded(&expanded);
        let message = hex(case["message"]);
        let context = hex(case["context"]);
        let signature = key.sign_deterministic(&message, &context).unwrap();
        assert_eq!(
            signature.encode().as_slice(),
            hex(case["signature"]),
            "NIST sigGen tcId={}",
            case["tcId"]
        );
        let public_key = verifying_key(&case);
        assert!(public_key.verify_with_context(&message, &context, &signature));
        if context.is_empty() {
            // Same empty-context Signer/Verifier entry points as ARETOVUS v3.
            assert_eq!(key.sign(&message).encode(), signature.encode());
            assert!(public_key.verify(&message, &signature).is_ok());
        }
        assert!(key.sign_deterministic(&message, &[0; 256]).is_err());
    }
}

#[test]
fn ml_dsa_65_rejects_exact_z_norm_boundaries() {
    let cases = cases(include_str!("../../../tests/vectors-fips204/sigver.txt"));
    let case = cases.iter().find(|case| case["tcId"] == "33").unwrap();
    let mut original = EncodedSignature::<MlDsa65>::default();
    original.copy_from_slice(&hex(case["signature"]));
    assert!(Signature::<MlDsa65>::decode(&original).is_some());

    // FIPS 204 Table 1 / Algorithms 18, 26, 27:
    // ML-DSA-65 has gamma1=2^19, beta=196, a 48-byte challenge and
    // little-endian 20-bit encodings of gamma1-z. Equality is forbidden.
    let replace_first_z = |packed_z: u32| {
        let mut encoded = EncodedSignature::<MlDsa65>::default();
        encoded.copy_from_slice(original.as_slice());
        let bytes = packed_z.to_le_bytes();
        encoded[48] = bytes[0];
        encoded[49] = bytes[1];
        encoded[50] = (encoded[50] & 0xf0) | (bytes[2] & 0x0f);
        encoded
    };
    for packed_z in [0, 196, (1 << 20) - 196, (1 << 20) - 1] {
        assert!(
            Signature::<MlDsa65>::decode(&replace_first_z(packed_z)).is_none(),
            "out-of-range first z coefficient: packed={packed_z}"
        );
    }
    for packed_z in [197, (1 << 20) - 197] {
        assert!(
            Signature::<MlDsa65>::decode(&replace_first_z(packed_z)).is_some(),
            "in-range first z coefficient: packed={packed_z}"
        );
    }
}

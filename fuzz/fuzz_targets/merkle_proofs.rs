#![no_main]

use aurora_core::{
    hash::Hash256,
    merkle::{verify_merkle_proof, MerkleProof},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    if bytes.len() < 73 {
        return;
    }
    let txid = Hash256::from_bytes(bytes[..32].try_into().expect("fixed slice"));
    let root = Hash256::from_bytes(bytes[32..64].try_into().expect("fixed slice"));
    let index = u32::from_be_bytes(bytes[64..68].try_into().expect("fixed slice"));
    let leaf_count = u32::from_be_bytes(bytes[68..72].try_into().expect("fixed slice"));
    let sibling_count = usize::from(bytes[72]).min(17);
    let mut cursor = 73usize;
    let mut siblings = Vec::with_capacity(sibling_count);
    for _ in 0..sibling_count {
        let Some(end) = cursor.checked_add(32) else {
            return;
        };
        let Some(encoded) = bytes.get(cursor..end) else {
            return;
        };
        siblings.push(Hash256::from_bytes(encoded.try_into().expect("fixed hash")));
        cursor = end;
    }
    let proof = MerkleProof {
        index,
        leaf_count,
        siblings,
    };
    let _ = verify_merkle_proof(txid, &proof, root);
});

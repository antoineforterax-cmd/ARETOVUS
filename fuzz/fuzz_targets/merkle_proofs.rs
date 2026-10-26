#![no_main]

use aurora_core::{
    hash::Hash256,
    merkle::{verify_merkle_proof, MerkleProof, MAX_MERKLE_LEAVES},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    if bytes.len() < 69 {
        return;
    }
    let txid = Hash256::from_bytes(bytes[..32].try_into().expect("fixed slice"));
    let root = Hash256::from_bytes(bytes[32..64].try_into().expect("fixed slice"));
    let index = u32::from_be_bytes(bytes[64..68].try_into().expect("fixed slice"));
    let leaf_count = u32::from_be_bytes(bytes[68..72].try_into().unwrap_or([0; 4]));
    let sibling_count_offset = if bytes.len() >= 72 { 72 } else { return };
    let sibling_count = usize::from(bytes[sibling_count_offset]).min(17);
    let mut cursor = sibling_count_offset + 1;
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
    if leaf_count as usize > MAX_MERKLE_LEAVES {
        let _ = verify_merkle_proof(
            txid,
            &MerkleProof {
                index,
                leaf_count,
                siblings,
            },
            root,
        );
        return;
    }
    let _ = verify_merkle_proof(
        txid,
        &MerkleProof {
            index,
            leaf_count,
            siblings,
        },
        root,
    );
});

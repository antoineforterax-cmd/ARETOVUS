#![no_main]

use aretovus_core::hash::sha256_plain;
use aretovus_storage::fuzz_parse_storage_bytes;
use libfuzzer_sys::fuzz_target;

fn checked(mut body: Vec<u8>) -> Vec<u8> {
    let digest = sha256_plain(&body);
    body.extend_from_slice(digest.as_bytes());
    body
}

fuzz_target!(|bytes: &[u8]| {
    if bytes.len() > 512 {
        return;
    }
    fuzz_parse_storage_bytes(bytes);

    // Recompute genuine checksums to reach fields behind the checksum gate.
    let mut page = vec![0u8; 56];
    page[..8].copy_from_slice(b"ARETOPAG");
    page[8..12].copy_from_slice(&1u32.to_be_bytes());
    page[20..24].copy_from_slice(&1u32.to_be_bytes());
    for (index, slot) in page[24..].iter_mut().enumerate() {
        *slot = bytes.get(index).copied().unwrap_or(0);
    }
    fuzz_parse_storage_bytes(&checked(page));

    let mut snapshot = vec![0u8; 77];
    snapshot[..8].copy_from_slice(b"ARETOSNP");
    snapshot[8..12].copy_from_slice(&4u32.to_be_bytes());
    snapshot[36] = bytes.first().copied().unwrap_or(0) % 3;
    snapshot[69..73].copy_from_slice(&1u32.to_be_bytes());
    snapshot[73..77].copy_from_slice(&(bytes.len() as u32).to_be_bytes());
    snapshot.extend_from_slice(bytes);
    fuzz_parse_storage_bytes(&checked(snapshot));
});

#![no_main]

use aurora_node::p2p::{decode_peer_list, read_blocks};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let _ = read_blocks(bytes);
    let _ = decode_peer_list(bytes);
});

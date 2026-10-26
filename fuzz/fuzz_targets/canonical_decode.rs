#![no_main]

use aurora_consensus::{account_block::AccountBlock, experimental::BlockHeader};
use aurora_core::{accounts::AccountTransaction, codec::CanonicalDecode};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let _ = AccountTransaction::from_canonical_bytes(bytes);
    let _ = AccountBlock::from_canonical_bytes(bytes);
    let _ = BlockHeader::from_canonical_bytes(bytes);
});

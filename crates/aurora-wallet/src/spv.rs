//! End-to-end SPV inclusion verification from a testnet node.
use std::{
    error::Error,
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use aurora_consensus::{
    light_header_chain::LightHeaderChain,
    testnet::{
        testnet_genesis_block, TESTNET_DIFFICULTY, TESTNET_GENESIS_HASH, TESTNET_INITIAL_BITS,
        TESTNET_MAX_FUTURE_BLOCK_SECS,
    },
};
use aurora_core::{hash::Hash256, merkle::verify_merkle_proof};

use crate::rpc;

const HEADER_PAGE_SIZE: u16 = 64;
const MAX_SPV_HEADERS: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerifiedInclusion {
    pub txid: Hash256,
    pub block_hash: Hash256,
    pub block_height: u64,
    pub tip_hash: Hash256,
    pub tip_height: u64,
    /// Includes the block containing the transaction.
    pub confirmations: u64,
}

/// Download and validate a header chain from the pinned testnet genesis, then verify
/// a bounded transaction Merkle proof against a canonical header.
pub fn verify_transaction_inclusion(
    node: SocketAddr,
    txid: Hash256,
    minimum_confirmations: u64,
) -> Result<VerifiedInclusion, Box<dyn Error>> {
    if minimum_confirmations == 0 {
        return Err("minimum confirmations must be at least one".into());
    }
    let genesis = testnet_genesis_block();
    let mut chain = LightHeaderChain::new(
        genesis.header,
        TESTNET_GENESIS_HASH,
        TESTNET_INITIAL_BITS,
        TESTNET_DIFFICULTY,
    )?;
    let mut stream = rpc::connect(node)?;
    let first_page = rpc::get_headers_on_stream(&mut stream, 1, HEADER_PAGE_SIZE)?;
    let announced_tip = first_page.tip_height;
    if announced_tip > MAX_SPV_HEADERS {
        return Err("testnet header count exceeds the SPV client limit".into());
    }
    let mut start = 1u64;
    let mut page = first_page;
    loop {
        if page.start_height != start || page.tip_height != announced_tip {
            return Err("peer changed its header chain during SPV synchronization".into());
        }
        let expected_count = if start > announced_tip {
            0
        } else {
            (announced_tip - start + 1).min(u64::from(HEADER_PAGE_SIZE)) as usize
        };
        if page.headers.len() != expected_count {
            return Err("peer returned an incomplete SPV header page".into());
        }
        for header in page.headers {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
            check_future_header_at(header.timestamp, now)?;
            chain.submit(header)?;
        }
        if start > announced_tip {
            break;
        }
        start = start.saturating_add(expected_count as u64);
        if start > announced_tip {
            break;
        }
        page = rpc::get_headers_on_stream(&mut stream, start, HEADER_PAGE_SIZE)?;
    }

    let proof = rpc::get_transaction_proof_on_stream(&mut stream, txid)?
        .ok_or("transaction is not present in the peer's canonical blocks")?;
    let expected_header = chain
        .header_at_height(proof.height)
        .ok_or("proof height is outside the verified header chain")?;
    if expected_header != proof.header {
        return Err("transaction proof header is not on the verified canonical chain".into());
    }
    let block_hash = proof.header.hash()?;
    let confirmations = chain
        .confirmations(block_hash)
        .ok_or("transaction proof block is not canonical")?;
    if confirmations < minimum_confirmations {
        return Err(format!(
            "transaction has {confirmations} confirmation(s), need {minimum_confirmations}"
        )
        .into());
    }
    if !verify_merkle_proof(txid, &proof.proof, proof.header.tx_merkle_root) {
        return Err("transaction Merkle proof does not match the header root".into());
    }
    Ok(VerifiedInclusion {
        txid,
        block_hash,
        block_height: proof.height,
        tip_hash: chain.tip_hash(),
        tip_height: chain.height(),
        confirmations,
    })
}

fn check_future_header_at(timestamp: u64, now: u64) -> Result<(), &'static str> {
    if timestamp > now.saturating_add(TESTNET_MAX_FUTURE_BLOCK_SECS) {
        return Err("SPV header timestamp exceeds local future allowance");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spv_future_header_admission_has_exact_boundary() {
        let now = 1_700_000_000;
        assert!(check_future_header_at(now + TESTNET_MAX_FUTURE_BLOCK_SECS, now).is_ok());
        assert!(check_future_header_at(now + TESTNET_MAX_FUTURE_BLOCK_SECS + 1, now).is_err());
        assert!(check_future_header_at(u64::MAX, u64::MAX).is_ok());
    }
}

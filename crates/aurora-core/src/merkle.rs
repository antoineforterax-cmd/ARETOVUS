//! Domain-separated transaction Merkle roots and bounded inclusion proofs.
use crate::hash::{tagged_hash, Domain, Hash256};

/// The consensus limit keeps proof construction and verification bounded.
pub const MAX_MERKLE_LEAVES: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MerkleError {
    TooManyLeaves,
    EmptyProof,
    IndexOutOfRange,
    InvalidProofShape,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MerkleProof {
    pub index: u32,
    pub leaf_count: u32,
    pub siblings: Vec<Hash256>,
}

fn hash_leaf(txid: Hash256) -> Hash256 {
    tagged_hash(Domain::MerkleLeaf, txid.as_bytes())
}

fn hash_node(left: Hash256, right: Hash256) -> Hash256 {
    let mut bytes = [0u8; 64];
    bytes[..32].copy_from_slice(left.as_bytes());
    bytes[32..].copy_from_slice(right.as_bytes());
    tagged_hash(Domain::MerkleNode, &bytes)
}

pub fn empty_merkle_root() -> Hash256 {
    tagged_hash(Domain::MerkleEmpty, &[])
}

/// Compute a root over transaction IDs in their exact block order.
/// Odd levels duplicate their final node; leaf/node/empty domains are distinct.
pub fn merkle_root(txids: &[Hash256]) -> Result<Hash256, MerkleError> {
    if txids.len() > MAX_MERKLE_LEAVES {
        return Err(MerkleError::TooManyLeaves);
    }
    if txids.is_empty() {
        return Ok(empty_merkle_root());
    }
    let mut level = txids.iter().copied().map(hash_leaf).collect::<Vec<_>>();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            let right = pair.get(1).copied().unwrap_or(pair[0]);
            next.push(hash_node(pair[0], right));
        }
        level = next;
    }
    Ok(level[0])
}

/// Build an inclusion proof for the transaction at the given index.
pub fn merkle_proof(txids: &[Hash256], index: usize) -> Result<MerkleProof, MerkleError> {
    if txids.is_empty() {
        return Err(MerkleError::EmptyProof);
    }
    if txids.len() > MAX_MERKLE_LEAVES {
        return Err(MerkleError::TooManyLeaves);
    }
    if index >= txids.len() {
        return Err(MerkleError::IndexOutOfRange);
    }
    let mut level = txids.iter().copied().map(hash_leaf).collect::<Vec<_>>();
    let mut position = index;
    let mut siblings = Vec::new();
    while level.len() > 1 {
        let sibling = if position % 2 == 0 {
            level.get(position + 1).copied().unwrap_or(level[position])
        } else {
            level[position - 1]
        };
        siblings.push(sibling);
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            next.push(hash_node(pair[0], pair.get(1).copied().unwrap_or(pair[0])));
        }
        position /= 2;
        level = next;
    }
    Ok(MerkleProof {
        index: index as u32,
        leaf_count: txids.len() as u32,
        siblings,
    })
}

/// Verify an inclusion proof against the root committed by a block header.
pub fn verify_merkle_proof(txid: Hash256, proof: &MerkleProof, expected_root: Hash256) -> bool {
    let count = proof.leaf_count as usize;
    let mut position = proof.index as usize;
    if count == 0 || count > MAX_MERKLE_LEAVES || position >= count {
        return false;
    }
    let mut levels = 0usize;
    let mut width = count;
    while width > 1 {
        levels += 1;
        width = width.div_ceil(2);
    }
    if proof.siblings.len() != levels {
        return false;
    }
    let mut current = hash_leaf(txid);
    width = count;
    for sibling in &proof.siblings {
        if position % 2 == 0 {
            if position + 1 >= width && *sibling != current {
                return false;
            }
            current = hash_node(current, *sibling);
        } else {
            current = hash_node(*sibling, current);
        }
        position /= 2;
        width = width.div_ceil(2);
    }
    current == expected_root
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(count: usize) -> Vec<Hash256> {
        (0..count)
            .map(|n| Hash256::from_bytes((n as u32).to_be_bytes().repeat(8).try_into().unwrap()))
            .collect()
    }

    #[test]
    fn empty_and_single_leaf_roots_are_defined() {
        assert_eq!(merkle_root(&[]).unwrap(), empty_merkle_root());
        let leaves = ids(1);
        let root = merkle_root(&leaves).unwrap();
        let proof = merkle_proof(&leaves, 0).unwrap();
        assert!(verify_merkle_proof(leaves[0], &proof, root));
        assert!(proof.siblings.is_empty());
    }

    #[test]
    fn proofs_cover_even_and_odd_tree_widths() {
        for count in 1..20 {
            let leaves = ids(count);
            let root = merkle_root(&leaves).unwrap();
            for (index, txid) in leaves.iter().enumerate() {
                let proof = merkle_proof(&leaves, index).unwrap();
                assert!(verify_merkle_proof(*txid, &proof, root), "count={} index={}", count, index);
            }
        }
    }

    #[test]
    fn proof_rejects_tampering_and_malformed_shape() {
        let leaves = ids(5);
        let root = merkle_root(&leaves).unwrap();
        let mut proof = merkle_proof(&leaves, 4).unwrap();
        assert!(!verify_merkle_proof(Hash256::ZERO, &proof, root));
        proof.siblings.pop();
        assert!(!verify_merkle_proof(leaves[4], &proof, root));
        let mut proof = merkle_proof(&leaves, 4).unwrap();
        proof.siblings[0] = Hash256::ZERO;
        assert!(!verify_merkle_proof(leaves[4], &proof, root));
    }

    #[test]
    fn proof_construction_rejects_bad_indices_and_unbounded_lists() {
        let leaves = ids(2);
        assert_eq!(merkle_proof(&[], 0), Err(MerkleError::EmptyProof));
        assert_eq!(merkle_proof(&leaves, 2), Err(MerkleError::IndexOutOfRange));
        assert_eq!(merkle_root(&ids(MAX_MERKLE_LEAVES + 1)), Err(MerkleError::TooManyLeaves));
    }
}

//! Deterministic bounded window retargeting and cumulative proof-of-work accounting.
use std::cmp::Ordering;

use crate::proof_of_work::MAX_LEADING_ZERO_BITS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DifficultyParams {
    pub target_block_seconds: u64,
    /// Zero keeps difficulty fixed; otherwise a retarget occurs each interval.
    pub adjustment_interval: u64,
    pub max_adjustment_bits: u16,
    pub min_bits: u16,
    pub max_bits: u16,
}

impl DifficultyParams {
    pub fn fixed(bits: u16) -> Self {
        Self {
            target_block_seconds: 0,
            adjustment_interval: 0,
            max_adjustment_bits: 0,
            min_bits: bits,
            max_bits: bits,
        }
    }

    pub fn validate(self, initial_bits: u16) -> bool {
        self.min_bits <= self.max_bits
            && self.max_bits <= MAX_LEADING_ZERO_BITS
            && (self.min_bits..=self.max_bits).contains(&initial_bits)
            && (self.adjustment_interval == 0
                || (self.target_block_seconds > 0 && self.max_adjustment_bits > 0))
    }

    /// Retarget by whole powers of two, bounded per window. Actual time below half
    /// target raises one bit; above double target lowers one bit, up to the cap.
    /// Inputs are medians from the chain, never the local wall clock.
    pub fn retarget(self, parent_bits: u16, actual_elapsed: u64) -> u16 {
        if self.adjustment_interval == 0 {
            return parent_bits;
        }
        let mut expected = self
            .target_block_seconds
            .saturating_mul(self.adjustment_interval);
        let mut actual = actual_elapsed.max(1);
        let mut bits = parent_bits;
        for _ in 0..self.max_adjustment_bits {
            if actual.saturating_mul(2) < expected && bits < self.max_bits {
                bits += 1;
                actual = actual.saturating_mul(2);
            } else if actual > expected.saturating_mul(2) && bits > self.min_bits {
                bits -= 1;
                expected = expected.saturating_mul(2);
            } else {
                break;
            }
        }
        bits
    }
}

/// A 320-bit unsigned work total; enough for 2^255 work per block over u64 heights.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ChainWork([u64; 5]);

impl ChainWork {
    pub fn for_difficulty(bits: u16) -> Self {
        let mut limbs = [0; 5];
        let bit = usize::from(bits);
        limbs[bit / 64] = 1u64 << (bit % 64);
        Self(limbs)
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        let mut out = [0; 5];
        let mut carry = 0u128;
        for (i, slot) in out.iter_mut().enumerate() {
            let sum = u128::from(self.0[i]) + u128::from(other.0[i]) + carry;
            *slot = sum as u64;
            carry = sum >> 64;
        }
        (carry == 0).then_some(Self(out))
    }
}

impl Ord for ChainWork {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.iter().rev().cmp(other.0.iter().rev())
    }
}
impl PartialOrd for ChainWork {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retarget_uses_bounded_integer_steps() {
        let p = DifficultyParams {
            target_block_seconds: 120,
            adjustment_interval: 10,
            max_adjustment_bits: 2,
            min_bits: 1,
            max_bits: 20,
        };
        assert_eq!(p.retarget(8, 100), 10);
        assert_eq!(p.retarget(8, 10_000), 6);
        assert_eq!(p.retarget(1, 1), 3);
        assert_eq!(p.retarget(20, 10_000), 18);
    }

    #[test]
    fn cumulative_work_compares_expected_hash_cost_not_height() {
        let two_easy = ChainWork::for_difficulty(2)
            .checked_add(ChainWork::for_difficulty(2))
            .unwrap();
        let one_harder = ChainWork::for_difficulty(3);
        assert!(one_harder > two_easy);
        assert!(ChainWork::for_difficulty(255) > ChainWork::for_difficulty(254));
    }
}

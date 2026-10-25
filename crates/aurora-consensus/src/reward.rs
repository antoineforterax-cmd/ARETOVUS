//! Experimental account-block issuance and fee distribution.
use std::fmt;

use aurora_core::{
    accounts::AccountId,
    amount::{Amount, AmountError, MAX_UNITS, UNITS_PER_COIN},
    state::{AccountLedger, StateError},
};

const CAP_UNITS: u64 = MAX_UNITS;
const TIERS: [(u64, u64); 6] = [
    (5_000_000, 500),
    (25_000_000, 100),
    (75_000_000, 60),
    (175_000_000, 30),
    (350_000_000, 10),
    (500_000_000, 1),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewardParams {
    /// The network's fixed treasury account. Its identity is part of the chain configuration.
    pub treasury_account: AccountId,
    pub enabled: bool,
}

impl RewardParams {
    pub const fn disabled() -> Self {
        Self {
            treasury_account: AccountId([0; 32]),
            enabled: false,
        }
    }
    pub const fn experimental_testnet(treasury_account: AccountId) -> Self {
        Self {
            treasury_account,
            enabled: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewardError {
    MissingMiner,
    UnexpectedMiner,
    Amount(AmountError),
    State(StateError),
}
impl fmt::Display for RewardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for RewardError {}
impl From<AmountError> for RewardError {
    fn from(e: AmountError) -> Self {
        Self::Amount(e)
    }
}
impl From<StateError> for RewardError {
    fn from(e: StateError) -> Self {
        Self::State(e)
    }
}

/// Returns the gross block issuance for the repository's `fast_start_500` profile.
/// The reward tier uses gross supply at the start of the block and is clipped to the 500M AUR cap.
pub fn scheduled_issuance(issued: Amount) -> Amount {
    let issued_units = issued.to_units();
    if issued_units >= CAP_UNITS {
        return Amount::ZERO;
    }
    let reward_coins = TIERS
        .iter()
        .find(|(threshold_coins, _)| issued_units < threshold_coins.saturating_mul(UNITS_PER_COIN))
        .map_or(0, |(_, reward_coins)| *reward_coins);
    let proposed = reward_coins.saturating_mul(UNITS_PER_COIN);
    Amount::from_units(proposed.min(CAP_UNITS - issued_units))
        .expect("clipped reward is within amount cap")
}

/// Applies deterministic issuance and fees after transfers have been validated on `ledger`.
/// Five percent of issuance goes to treasury (floor); half of fees goes to miner (floor),
/// and the remainder is burned. Mutations are atomic.
pub fn apply_block_reward(
    ledger: &mut AccountLedger,
    miner: Option<AccountId>,
    total_fees: Amount,
    params: RewardParams,
) -> Result<(), RewardError> {
    if !params.enabled {
        return if miner.is_some() {
            Err(RewardError::UnexpectedMiner)
        } else {
            Ok(())
        };
    }
    let issuance = scheduled_issuance(ledger.total_issued());
    let miner_fees = Amount::from_units(total_fees.to_units() / 2)?;
    let has_payout = issuance != Amount::ZERO || miner_fees != Amount::ZERO;
    let miner = match (miner, has_payout) {
        (Some(miner), _) => miner,
        (None, true) => return Err(RewardError::MissingMiner),
        (None, false) => return Ok(()),
    };
    let treasury_issuance = Amount::from_units(issuance.to_units() / 20)?;
    let miner_issuance = Amount::from_units(issuance.to_units() - treasury_issuance.to_units())?;

    let mut candidate = ledger.clone();
    candidate.credit_issuance(miner, miner_issuance)?;
    candidate.credit_issuance(params.treasury_account, treasury_issuance)?;
    candidate.credit_fee_reward(miner, miner_fees)?;
    *ledger = candidate;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aurora_core::network::NetworkId;

    #[test]
    fn fast_start_tiers_and_cap_are_integer_and_clipped() {
        assert_eq!(
            scheduled_issuance(Amount::ZERO).to_units(),
            500 * UNITS_PER_COIN
        );
        assert_eq!(
            scheduled_issuance(Amount::from_units(5_000_000 * UNITS_PER_COIN).unwrap()).to_units(),
            100 * UNITS_PER_COIN
        );
        assert_eq!(
            scheduled_issuance(Amount::from_units(CAP_UNITS - 1).unwrap()).to_units(),
            1
        );
        assert_eq!(
            scheduled_issuance(Amount::from_units(CAP_UNITS).unwrap()),
            Amount::ZERO
        );
    }

    #[test]
    fn fee_split_rounds_miner_down_and_burns_remainder() {
        let miner = AccountId([1; 32]);
        let treasury = AccountId([2; 32]);
        let mut ledger = AccountLedger::new(NetworkId::Regtest);
        let fees = Amount::from_units(5).unwrap();
        apply_block_reward(
            &mut ledger,
            Some(miner),
            fees,
            RewardParams::experimental_testnet(treasury),
        )
        .unwrap();
        assert_eq!(
            ledger.account(miner).balance.to_units(),
            475 * UNITS_PER_COIN + 2
        );
        assert_eq!(
            ledger.account(treasury).balance.to_units(),
            25 * UNITS_PER_COIN
        );
        assert_eq!(ledger.total_issued().to_units(), 500 * UNITS_PER_COIN);
        assert_eq!(
            ledger.account(miner).balance.to_units() + ledger.account(treasury).balance.to_units(),
            500 * UNITS_PER_COIN + 2
        );
    }
}

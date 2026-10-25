#!/usr/bin/env python3
"""AURORA issuance schedule simulator — research tool, not consensus code.

Run:
    python3 scripts/emission_sim.py

Uses integer atomic units only. The default tiers are illustrative candidates:
they are intentionally not protocol parameters. The simulator compares several
reward schedules over supply thresholds and reports the block count/time needed
to reach each threshold. Fees are excluded because burning fees changes net
supply, not gross issuance or the strict gross issuance cap.
"""

from dataclasses import dataclass

DECIMALS = 100_000_000
CAP_AUR = 500_000_000
CAP = CAP_AUR * DECIMALS
BLOCK_TARGET_SECONDS = 120
SECONDS_PER_YEAR = 365.2425 * 24 * 60 * 60


@dataclass(frozen=True)
class Tier:
    until_supply_aur: int
    reward_aur: int


# Candidate schedule: compare against alternatives; do not treat as approved.
CANDIDATES = {
    "slow_start_50": [
        Tier(5_000_000, 50),
        Tier(25_000_000, 30),
        Tier(75_000_000, 20),
        Tier(175_000_000, 10),
        Tier(CAP_AUR, 1),
    ],
    "balanced_100": [
        Tier(5_000_000, 100),
        Tier(25_000_000, 60),
        Tier(75_000_000, 30),
        Tier(175_000_000, 10),
        Tier(CAP_AUR, 1),
    ],
    "fast_start_500": [
        Tier(5_000_000, 500),
        Tier(25_000_000, 100),
        Tier(75_000_000, 60),
        Tier(175_000_000, 30),
        Tier(350_000_000, 10),
        Tier(CAP_AUR, 1),
    ],
}


def validate_tiers(tiers: list[Tier]) -> None:
    previous = 0
    previous_reward = None
    for tier in tiers:
        if tier.until_supply_aur <= previous or tier.until_supply_aur > CAP_AUR:
            raise ValueError("Supply thresholds must increase and not exceed the cap")
        if tier.reward_aur <= 0:
            raise ValueError("Rewards must be positive integers in this candidate model")
        if previous_reward is not None and tier.reward_aur > previous_reward:
            raise ValueError("Rewards must not increase between tiers")
        previous = tier.until_supply_aur
        previous_reward = tier.reward_aur
    if not tiers or tiers[-1].until_supply_aur != CAP_AUR:
        raise ValueError("Last tier must end at the strict supply cap")


def simulate(name: str, tiers: list[Tier]) -> None:
    validate_tiers(tiers)
    issued = 0
    blocks = 0
    treasury = 0
    miner_issuance = 0
    print(f"\n## {name}")
    print("threshold_AUR,reward_AUR,blocks_in_tier,elapsed_years,cumulative_issued_AUR,treasury_AUR,miner_issuance_AUR")

    for tier in tiers:
        threshold = tier.until_supply_aur * DECIMALS
        start_issued = issued
        reward = tier.reward_aur * DECIMALS
        blocks_in_tier = 0

        while issued < threshold:
            # Enforce the strict cap on every block, even if a reward exceeds
            # the remaining amount. This is the final monetary safety bound.
            gross = min(reward, CAP - issued, threshold - issued)
            if gross <= 0:
                break
            treasury_part = gross // 20  # proposed 5% rounding convention
            miner_part = gross - treasury_part
            issued += gross
            treasury += treasury_part
            miner_issuance += miner_part
            blocks += 1
            blocks_in_tier += 1

        elapsed_years = blocks * BLOCK_TARGET_SECONDS / SECONDS_PER_YEAR
        print(
            f"{tier.until_supply_aur},{tier.reward_aur},{blocks_in_tier},"
            f"{elapsed_years:.4f},{issued / DECIMALS:.8f},"
            f"{treasury / DECIMALS:.8f},{miner_issuance / DECIMALS:.8f}"
        )
        if issued >= CAP:
            break

    assert 0 <= issued <= CAP
    assert treasury + miner_issuance == issued
    print(f"TOTAL: issued={issued / DECIMALS:.8f} AUR, "
          f"treasury={treasury / DECIMALS:.8f} AUR, "
          f"miner_issuance={miner_issuance / DECIMALS:.8f} AUR, "
          f"blocks={blocks}, years={blocks * BLOCK_TARGET_SECONDS / SECONDS_PER_YEAR:.4f}")
    if issued < CAP:
        print(f"REMAINDER: {CAP_AUR - issued / DECIMALS:.8f} AUR not issued")
    print("NOTE: fixed target-time assumption; no fees, orphan blocks, hashrate, "
          "market value, or miner costs are modeled.")


def main() -> None:
    for name, tiers in CANDIDATES.items():
        simulate(name, tiers)


if __name__ == "__main__":
    main()

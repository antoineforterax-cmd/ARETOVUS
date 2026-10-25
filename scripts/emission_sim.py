#!/usr/bin/env python3
"""AURORA issuance schedule simulator — research tool, not consensus code.

Run:
    python3 scripts/emission_sim.py

All amounts use integer atomic units. Tier thresholds are evaluated against
supply at the START of a block: a block may cross a tier threshold, and the
next block uses the next applicable tier. The final reward is clipped to the
strict gross issuance cap. Fixed 120-second block timing is only a scenario.
"""

from dataclasses import dataclass
from math import ceil

DECIMALS = 100_000_000
CAP_AUR = 500_000_000
CAP = CAP_AUR * DECIMALS
BLOCK_TARGET_SECONDS = 120
SECONDS_PER_YEAR = 365.2425 * 24 * 60 * 60


@dataclass(frozen=True)
class Tier:
    until_supply_aur: int
    reward_aur: int


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
    previous_threshold = 0
    previous_reward = None
    for tier in tiers:
        if not previous_threshold < tier.until_supply_aur <= CAP_AUR:
            raise ValueError("Thresholds must strictly increase and not exceed cap")
        if tier.reward_aur <= 0:
            raise ValueError("Rewards must be positive")
        if previous_reward is not None and tier.reward_aur > previous_reward:
            raise ValueError("Rewards must not increase between tiers")
        previous_threshold = tier.until_supply_aur
        previous_reward = tier.reward_aur
    if not tiers or tiers[-1].until_supply_aur != CAP_AUR:
        raise ValueError("Last tier must end at the strict cap")


def years_for_blocks(blocks: int) -> float:
    return blocks * BLOCK_TARGET_SECONDS / SECONDS_PER_YEAR


def simulate(name: str, tiers: list[Tier]) -> dict[str, int | float | str]:
    validate_tiers(tiers)
    issued = 0
    blocks = 0
    treasury = 0
    miner_issuance = 0
    print(f"\n## {name}")
    print("threshold_AUR,reward_AUR,blocks_in_tier,elapsed_years,cumulative_issued_AUR,treasury_AUR,miner_issuance_AUR")

    for tier in tiers:
        threshold = tier.until_supply_aur * DECIMALS
        if issued >= threshold:
            continue
        reward = tier.reward_aur * DECIMALS
        # Supply is checked before each block. A block can cross a threshold;
        # the new tier applies to the following block.
        blocks_in_tier = min(ceil((threshold - issued) / reward), ceil((CAP - issued) / reward))
        if blocks_in_tier <= 0:
            continue
        gross = min(blocks_in_tier * reward, CAP - issued)
        # All candidate rewards and the cap are whole AUR, so 5% is exact
        # in atomic units. This calculation is equivalent to per-block floors.
        treasury_part = (gross // 20)
        miner_part = gross - treasury_part
        issued += gross
        blocks += blocks_in_tier
        treasury += treasury_part
        miner_issuance += miner_part
        print(
            f"{tier.until_supply_aur},{tier.reward_aur},{blocks_in_tier},"
            f"{years_for_blocks(blocks):.4f},{issued / DECIMALS:.8f},"
            f"{treasury / DECIMALS:.8f},{miner_issuance / DECIMALS:.8f}"
        )
        if issued >= CAP:
            break

    assert 0 <= issued <= CAP
    assert treasury + miner_issuance == issued
    print(
        f"TOTAL: issued={issued / DECIMALS:.8f} AUR, "
        f"treasury={treasury / DECIMALS:.8f} AUR, "
        f"miner_issuance={miner_issuance / DECIMALS:.8f} AUR, "
        f"blocks={blocks}, years={years_for_blocks(blocks):.4f}"
    )
    if issued < CAP:
        print(f"REMAINDER: {CAP_AUR - issued / DECIMALS:.8f} AUR not issued")
    print("NOTE: fixed target-time assumption; excludes fees, orphan blocks, "
          "hashrate, market value, and miner costs.")
    return {"name": name, "issued": issued, "blocks": blocks, "years": years_for_blocks(blocks)}


def main() -> None:
    results = [simulate(name, tiers) for name, tiers in CANDIDATES.items()]
    print("\n## Comparison")
    for result in results:
        print(f"{result['name']}: {result['blocks']:,} blocks; {result['years']:.2f} years")


if __name__ == "__main__":
    main()

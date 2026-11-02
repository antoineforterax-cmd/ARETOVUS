#!/usr/bin/env python3
"""D2: deterministic *model*, not executed blockchain storage or TCP sync."""
from __future__ import annotations

import argparse
import json
from typing import Any

PAGE_BLOCKS = 64
DEFAULT_HEIGHTS = (1_000_000, 10_000_000, 100_000_000, 500_000_000)


def estimate(blocks: int, *, page_size: int = PAGE_BLOCKS,
             assumed_bytes_per_block: int | None = None) -> dict[str, Any]:
    if blocks < 1 or page_size < 1:
        raise ValueError("blocks and page_size must be positive")
    if assumed_bytes_per_block is not None and assumed_bytes_per_block < 1:
        raise ValueError("assumed_bytes_per_block must be positive or omitted")
    snapshots = (blocks + page_size - 1) // page_size
    # Historical storage code rechecks every retained path on each
    # persistence page, including the genesis block. This counts the
    # minimum number of path checks for a linear chain without forks.
    checks = snapshots + page_size * snapshots * (snapshots - 1) // 2 + blocks
    result: dict[str, Any] = {
        "blocks_after_genesis": blocks,
        "genesis_blocks": 1,
        "page_blocks_assumed": page_size,
        "snapshot_calls_modelled": snapshots,
        "minimal_existing_path_metadata_checks_modelled": checks,
        "model_only": True,
        "not_measured": ["wall_time", "CPU", "RAM", "actual_disk",
                         "TCP_sessions", "physical_block_persistence"],
        "assumptions": [
            "single linear chain, no side branches",
            "one snapshot per 64 incoming blocks, final short page included",
            "genesis block included in each path scan",
            "all existing block paths checked at every snapshot",
            "no batching and no change to the production storage policy",
        ],
    }
    if assumed_bytes_per_block is not None:
        result["assumed_bytes_per_block"] = assumed_bytes_per_block
        result["illustrative_raw_block_bytes"] = (blocks + 1) * assumed_bytes_per_block
        result["illustrative_raw_block_gib"] = round(
            result["illustrative_raw_block_bytes"] / (1024 ** 3), 4
        )
        result["raw_size_excludes"] = ["index", "snapshots", "branches", "wallet",
                                       "filesystem overhead", "replicas", "logs"]
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--blocks", type=int, nargs="+", default=list(DEFAULT_HEIGHTS))
    parser.add_argument("--assumed-bytes-per-block", type=int, default=None)
    args = parser.parse_args()
    results = [estimate(n, assumed_bytes_per_block=args.assumed_bytes_per_block)
               for n in args.blocks]
    print(json.dumps({"study": "ARETOVUS D2 model only; no blocks generated",
                      "results": results}, indent=2))


if __name__ == "__main__":
    main()

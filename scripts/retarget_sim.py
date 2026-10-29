#!/usr/bin/env python3
"""Independent integer oracle for the current AURORA testnet retarget.

No Rust imports, subprocesses or third-party packages. The threshold formula
counts powers of two instead of reproducing the Rust mutable retarget loop.
"""
import argparse
import json
from pathlib import Path

U64_MAX = (1 << 64) - 1
TARGET = 600
INTERVAL = 10
INITIAL_BITS = 8
MIN_BITS = 4
MAX_BITS = 20
MAX_STEP = 2
FUTURE_ALLOWANCE = 7200
FIXTURES = Path(__file__).resolve().parents[1] / "tests" / "vectors-retarget"


def retarget(parent_bits: int, elapsed: int) -> int:
    if not MIN_BITS <= parent_bits <= MAX_BITS or not 0 <= elapsed <= U64_MAX:
        raise ValueError("input outside testnet domain")
    actual = max(1, elapsed)
    up = sum(actual * (1 << k) < TARGET for k in range(1, MAX_STEP + 1))
    down = sum(actual > TARGET * (1 << k) for k in range(1, MAX_STEP + 1))
    return min(MAX_BITS, max(MIN_BITS, parent_bits + up - down))


def admitted(timestamp: int, now: int) -> bool:
    return timestamp <= min(U64_MAX, now + FUTURE_ALLOWANCE)


def scenarios() -> dict[str, list[int]]:
    return {
        "regular": [60] * 30,
        "fast": [10] * 30,
        "slow": [300] * 30,
        "pause": [60] * 9 + [86400] + [60] * 20,
        "drift": [61] * 30,
        "alternating": [10] * 10 + [300] * 10 + [60] * 10,
        "future": [1000] * 30,
    }


def outputs() -> tuple[dict[str, str], dict]:
    thresholds = [0, 1, 149, 150, 151, 299, 300, 301, 599, 600,
                  601, 1199, 1200, 1201, 2399, 2400, 2401, 86400, U64_MAX]
    rows = ["parent_bits\telapsed\texpected_bits"]
    for bits in range(MIN_BITS, MAX_BITS + 1):
        for elapsed in thresholds:
            rows.append(f"{bits}\t{elapsed}\t{retarget(bits, elapsed)}")
    files = {"boundaries.tsv": "\n".join(rows) + "\n"}
    rows = ["scenario\theight\ttimestamp\tbits\twork"]
    report = {}
    for name, intervals in scenarios().items():
        timestamps, bits, work = [1], INITIAL_BITS, 1 << INITIAL_BITS
        rows.append(f"{name}\t0\t1\t{bits}\t{work}")
        for height, spacing in enumerate(intervals, 1):
            timestamp = timestamps[-1] + spacing
            if height % INTERVAL == 0:
                bits = retarget(bits, timestamp - timestamps[height - INTERVAL])
            timestamps.append(timestamp)
            work += 1 << bits
            rows.append(f"{name}\t{height}\t{timestamp}\t{bits}\t{work}")
        report[name] = {"height": len(intervals), "timestamp": timestamps[-1],
                        "bits": bits, "work": work,
                        "admitted_at_now_1": admitted(timestamps[-1], 1)}
    files["chains.tsv"] = "\n".join(rows) + "\n"
    cases = [(7200, 1), (7201, 1), (7202, 1), (7202, 2),
             (1, 7202), (U64_MAX, U64_MAX), (U64_MAX, U64_MAX - 7200),
             (U64_MAX, U64_MAX - 7201)]
    rows = ["timestamp\tnow\tadmitted"]
    rows += [f"{ts}\t{now}\t{int(admitted(ts, now))}" for ts, now in cases]
    files["admission.tsv"] = "\n".join(rows) + "\n"
    return files, report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="compare checked-in fixtures")
    mode.add_argument("--write", action="store_true", help="regenerate fixtures explicitly")
    args = parser.parse_args()
    # Hand-derived anchors guard the oracle itself.
    assert [retarget(8, t) for t in (149, 150, 299, 300, 1200, 1201, 2400, 2401)] == [
        10, 9, 9, 8, 8, 7, 7, 6
    ]
    assert retarget(4, U64_MAX) == 4 and retarget(20, 0) == 20
    files, report = outputs()
    if args.write:
        FIXTURES.mkdir(parents=True, exist_ok=True)
        for name, content in files.items():
            (FIXTURES / name).write_text(content, encoding="utf-8", newline="\n")
    if args.check:
        for name, expected in files.items():
            if (FIXTURES / name).read_bytes() != expected.encode("utf-8"):
                raise SystemExit(f"fixture mismatch: {name}; review before --write")
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

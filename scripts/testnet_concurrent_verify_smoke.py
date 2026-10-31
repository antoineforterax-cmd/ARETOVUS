#!/usr/bin/env python3
"""CI smoke: verify complete on-disk snapshots while a testnet miner writes.

One machine, one miner, and read-only verifier subprocesses. This is NOT a
physical two-PC test, a performance benchmark, or a 4-hour endurance claim.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

TIP = re.compile(
    r"verified testnet data: height=(\d+) tip=([a-f0-9]{64}) "
    r"known_blocks=(\d+) work=ChainWork\(\[([\d, ]+)\]\)"
)
MINER_ACCOUNT = "a7" * 32


def parse_snapshot(output: str) -> tuple[int, str, int]:
    matches = TIP.findall(output)
    if len(matches) != 1:
        raise ValueError("missing or ambiguous verified height/tip/work")
    height, tip, known, work = matches[0]
    if int(known) < int(height) + 1 or not work.strip():
        raise ValueError("malformed verified chain status")
    return int(height), tip, int(known)


def probe(node: Path, directory: Path, timeout: float) -> tuple[float, tuple[int, str, int]]:
    started = time.monotonic()
    result = subprocess.run(
        [str(node), "testnet", "verify", "--data-dir", str(directory)],
        capture_output=True, text=True, timeout=timeout, check=False,
    )
    elapsed = time.monotonic() - started
    if result.returncode:
        raise RuntimeError(
            f"testnet verify exited {result.returncode}; "
            "concurrent storage read may be inconsistent"
        )
    return round(elapsed, 4), parse_snapshot(result.stdout)


def check_progress(previous: tuple[int, str, int] | None, current: tuple[int, str, int]) -> None:
    """Reject contradictory read-only snapshots from a single, fork-free miner."""
    if previous is None:
        return
    old_height, old_tip, old_known = previous
    height, tip, known = current
    if height < old_height:
        raise AssertionError("verified canonical height went backwards")
    if height == old_height and tip != old_tip:
        raise AssertionError("verified canonical tip changed at unchanged height")
    if known < old_known:
        raise AssertionError("verified known-block count went backwards")


def self_test() -> None:
    good = (
        "verified testnet data: height=12 tip=" + "ab" * 32
        + " known_blocks=13 work=ChainWork([4096, 0, 0, 0, 0])"
    )
    assert parse_snapshot(good) == (12, "ab" * 32, 13)
    for bad in (
        "", good.replace("height=12", "height=1").replace("known_blocks=13", "known_blocks=1"),
        good.replace("tip=" + "ab" * 32, "tip=INVALID"),
        good + "\n" + good,
    ):
        try:
            parse_snapshot(bad)
        except ValueError:
            continue
        raise AssertionError("invalid snapshot accepted")
    first = (12, "ab" * 32, 13)
    check_progress(None, first)
    check_progress(first, first)
    check_progress(first, (13, "cd" * 32, 14))
    for previous, current in (
        (first, (11, "cd" * 32, 13)),    # height regression
        (first, (12, "cd" * 32, 13)),    # same-height tip mutation
        (first, (13, "cd" * 32, 12)),    # known-block regression
    ):
        try:
            check_progress(previous, current)
        except AssertionError:
            continue
        raise AssertionError("inconsistent verified sequence accepted")
    print("concurrent_verify_parser_and_sequence_self_test=ok")


def exercise(node: Path, blocks: int, timeout: float, probe_timeout: float) -> dict:
    node = node.resolve(strict=True)
    started = time.monotonic()
    samples: list[dict] = []
    miner_status = ""
    previous_snapshot: tuple[int, str, int] | None = None
    with tempfile.TemporaryDirectory(prefix="aurora-verify-overlap-") as root:
        root_path = Path(root)
        directory = root_path / "chain"
        log_path = root_path / "miner.log"
        with log_path.open("w+", encoding="utf-8") as log:
            miner = subprocess.Popen(
                [str(node), "testnet", "mine", "--data-dir", str(directory),
                 "--miner", MINER_ACCOUNT, "--count", str(blocks)],
                stdout=log, stderr=subprocess.STDOUT, text=True,
            )
            try:
                while miner.poll() is None:
                    if time.monotonic() - started > timeout:
                        raise TimeoutError("miner exceeded overall CI timeout")
                    # The very first snapshot may not yet exist; do not treat
                    # pre-genesis startup as a storage reader failure.
                    if not any(directory.glob("snapshot-*.auridx")):
                        time.sleep(0.04)
                        continue
                    measured_at = time.monotonic()
                    seconds, (height, tip, known) = probe(node, directory, probe_timeout)
                    if height > blocks:
                        raise AssertionError("snapshot exceeds requested mining height")
                    check_progress(previous_snapshot, (height, tip, known))
                    previous_snapshot = (height, tip, known)
                    samples.append({
                        "height": height, "tip": tip, "known_blocks": known,
                        "verify_seconds": seconds,
                        "overlap": miner.poll() is None,
                        "start_offset_seconds": round(measured_at - started, 3),
                    })
                    time.sleep(0.04)
                exit_code = miner.wait(timeout=5)
                if exit_code:
                    log.flush()
                    log.seek(0)
                    raise RuntimeError(
                        f"testnet miner exited {exit_code}: " + log.read()[-1400:]
                    )
                final_seconds, final = probe(node, directory, probe_timeout)
                if final[0] != blocks:
                    raise AssertionError(
                        f"expected {blocks} mined blocks, verify returned {final[0]}"
                    )
                check_progress(previous_snapshot, final)
                miner_status = "completed"
            finally:
                if miner.poll() is None:
                    miner.terminate()
                    try:
                        miner.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        miner.kill()
                        miner.wait(timeout=5)
    return {
        "type": "single-runner-testnet-verify-under-mining",
        "miner_status": miner_status,
        "blocks": blocks,
        "during_mining_checks": len(samples),
        "overlapping_checks": sum(bool(x["overlap"]) for x in samples),
        "distinct_overlapping_heights": len({
            x["height"] for x in samples if x["overlap"]
        }),
        "worst_verify_seconds": max((x["verify_seconds"] for x in samples), default=None),
        "final_verify_seconds": final_seconds,
        "final_height": final[0],
        "final_tip": final[1],
        "elapsed_seconds": round(time.monotonic() - started, 3),
        "samples": samples,
        "physical_two_host_test": False,
    }


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--node", type=Path)
    p.add_argument("--blocks", type=int, default=24)
    p.add_argument("--max-seconds", type=float, default=180)
    p.add_argument("--probe-timeout", type=float, default=30)
    p.add_argument("--min-distinct-heights", type=int, default=2)
    p.add_argument("--output", type=Path)
    p.add_argument("--self-test", action="store_true")
    args = p.parse_args()
    if args.self_test:
        self_test()
        return 0
    if not args.node:
        p.error("--node is required")
    if not 4 <= args.blocks <= 100:
        p.error("--blocks must be 4..100")
    if not 5 <= args.probe_timeout <= 120 or not 30 <= args.max_seconds <= 600:
        p.error("timeouts outside allowed safety bounds")
    if not 1 <= args.min_distinct_heights <= 10:
        p.error("--min-distinct-heights must be 1..10")
    report = exercise(args.node, args.blocks, args.max_seconds, args.probe_timeout)
    # Preserve evidence even if a stricter coverage gate rejects the run.
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    if not report["during_mining_checks"]:
        raise AssertionError("no verifier exercised while miner was active")
    if not report["overlapping_checks"]:
        raise AssertionError("no read/write overlap observed")
    if report["distinct_overlapping_heights"] < args.min_distinct_heights:
        raise AssertionError(
            "insufficient in-flight snapshot progress: "
            f"observed {report['distinct_overlapping_heights']} distinct heights, "
            f"required {args.min_distinct_heights}"
        )
    print(
        f"concurrent_verify=ok blocks={report['blocks']} "
        f"overlaps={report['overlapping_checks']} "
        f"distinct_heights={report['distinct_overlapping_heights']} "
        f"final_height={report['final_height']} "
        f"max_verification_seconds={report['worst_verify_seconds']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

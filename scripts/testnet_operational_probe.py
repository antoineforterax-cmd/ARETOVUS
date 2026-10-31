#!/usr/bin/env python3
"""Read-only operational evidence logger. Does not mine, start peers or assert a multi-host test."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

TIP_PATTERN = re.compile(r"\bheight=(\d+)\s+tip=([0-9a-f]{64})\b")


def parse_tip(output: str) -> tuple[int, str]:
    match = TIP_PATTERN.search(output)
    if match is None:
        raise ValueError("verify output lacks height and 64-hex tip")
    return int(match.group(1)), match.group(2)


def binary_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while data := source.read(1024 * 1024):
            digest.update(data)
    return digest.hexdigest()


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def classify_completion(
    requested_seconds: float,
    elapsed_seconds: float,
    failures: int,
    binary_unchanged: bool,
    distinct_heights: int,
) -> tuple[bool, bool, bool]:
    # Read-only evidence classification, not a statement about network attacks.
    passed = (
        elapsed_seconds >= requested_seconds
        and failures == 0
        and binary_unchanged
        and distinct_heights >= 2
    )
    four_hours = passed and requested_seconds >= 4 * 3600 and elapsed_seconds >= 4 * 3600
    seventy_two_hours = (
        passed and requested_seconds >= 72 * 3600 and elapsed_seconds >= 72 * 3600
    )
    return passed, four_hours, seventy_two_hours


def self_test() -> None:
    assert parse_tip("height=42 tip=" + "ab" * 32) == (42, "ab" * 32)
    for bad in ("height=42", "height=42 tip=" + "X" * 64, "height=-1 tip=" + "a" * 64):
        try:
            parse_tip(bad)
        except ValueError:
            pass
        else:
            raise AssertionError(f"invalid tip accepted: {bad}")
    assert classify_completion(4 * 3600, 4 * 3600, 0, True, 2) == (
        True, True, False
    )
    assert classify_completion(72 * 3600, 4 * 3600, 0, True, 2) == (
        False, False, False
    )
    assert classify_completion(72 * 3600, 72 * 3600, 0, True, 2) == (
        True, True, True
    )
    assert classify_completion(4 * 3600, 4 * 3600, 1, True, 2) == (
        False, False, False
    )
    assert classify_completion(4 * 3600, 4 * 3600, 0, True, 1) == (
        False, False, False
    )
    print("operational probe parsing and honest 4h/72h classification tests passed")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--data-dir", type=Path)
    parser.add_argument("--host-label", type=str, default="node-a")
    parser.add_argument("--hours", type=float, default=4.0)
    parser.add_argument("--interval-seconds", type=float, default=300.0)
    parser.add_argument("--command-timeout", type=float, default=300.0)
    parser.add_argument("--output", type=Path, default=Path("aretovus-operational-evidence.jsonl"))
    parser.add_argument("--commit", default="unrecorded")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    if args.binary is None or args.data_dir is None:
        parser.error("--binary and --data-dir are required unless --self-test")
    if not 4 <= args.hours <= 168 or not 1 <= args.interval_seconds <= 3600:
        parser.error("hours must be 4..168 and interval-seconds 1..3600")
    if not 5 <= args.command_timeout <= 3600:
        parser.error("command-timeout must be 5..3600 seconds")
    if not re.fullmatch(r"[a-zA-Z0-9_.-]{1,64}", args.host_label):
        parser.error("host-label must contain only A-Z, a-z, 0-9, _, -, .")
    if not re.fullmatch(r"[a-fA-F0-9]{40}", args.commit):
        parser.error("--commit must be a real 40-character Git SHA (git rev-parse HEAD)")
    executable = args.binary.resolve()
    data_dir = args.data_dir.resolve()
    if not executable.is_file() or not data_dir.is_dir():
        parser.error("binary file and existing node data-dir required")
    output = args.output.resolve()
    if output == executable or output.is_relative_to(data_dir):
        parser.error("output evidence must be outside node data-dir")
    output.parent.mkdir(parents=True, exist_ok=True)
    digest = binary_sha256(executable)
    started = time.monotonic()
    duration = args.hours * 3600.0
    deadline = started + duration
    failures = 0
    samples = 0
    heights: set[int] = set()
    last_tip: tuple[int, str] | None = None

    def record(handle, event: str, **values: object) -> None:
        handle.write(json.dumps({"event": event, "utc": utc_now(),
                                 "host_label": args.host_label, **values}, sort_keys=True) + "\n")
        handle.flush()

    # A four-hour physical test is a useful intermediate gate, not 72h certification.
    # Never infer hardware independence from the host-label supplied by the operator.
    # Append-only evidence: do not overwrite older runs or silently count them as part of this run.
    with output.open("a", encoding="utf-8") as evidence:
        record(evidence, "start", duration_required_seconds=duration, commit=args.commit,
               binary_sha256=digest, initial_monotonic=time.monotonic(), version=1)
        while True:
            started_sample = time.monotonic()
            sample: dict[str, object] = {
                "elapsed_seconds": round(started_sample - started, 3),
                "free_disk_bytes": shutil.disk_usage(data_dir).free,
            }
            try:
                result = subprocess.run(
                    [str(executable), "testnet", "verify", "--data-dir", str(data_dir)],
                    capture_output=True, text=True, timeout=args.command_timeout, check=False
                )
                if result.returncode != 0:
                    raise ValueError(f"verify returned {result.returncode}")
                height, tip = parse_tip(result.stdout)
                heights.add(height)
                last_tip = (height, tip)
                sample.update(status="ok", height=height, tip=tip,
                              verify_seconds=round(time.monotonic() - started_sample, 3))
            except (OSError, ValueError, subprocess.TimeoutExpired) as exc:
                failures += 1
                sample.update(status="error", error=type(exc).__name__)
            samples += 1
            record(evidence, "sample", **sample)
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                break
            time.sleep(min(args.interval_seconds, remaining))
        elapsed = time.monotonic() - started
        unchanged = binary_sha256(executable) == digest
        duration_reached = elapsed >= duration
        # 72-hour evidence must contain actual change over time, not just a frozen node.
        successful, counts_as_4h, counts_as_72h = classify_completion(
            duration, elapsed, failures, unchanged, len(heights)
        )
        record(evidence, "finish", elapsed_seconds=round(elapsed, 3), samples=samples,
               errors=failures, binary_unchanged=unchanged, distinct_heights=len(heights),
               last_height=last_tip[0] if last_tip else None,
               last_tip=last_tip[1] if last_tip else None,
               duration_reached=duration_reached, local_probe_passed=successful,
               counts_as_4h=counts_as_4h, counts_as_72h=counts_as_72h)
    print(f"Evidence: {output}; local probe passed={successful}; "
          f"4h threshold={counts_as_4h}; "
          f"72h threshold={counts_as_72h}")
    return 0 if successful else 1


if __name__ == "__main__":
    sys.exit(main())

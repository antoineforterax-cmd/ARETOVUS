#!/usr/bin/env python3
"""Cross-check two completed independent AURORA 72-hour evidence logs; no external attestations."""
from __future__ import annotations

import argparse
import json
from datetime import datetime
from pathlib import Path


def latest_run(path: Path) -> tuple[dict, list[dict], dict]:
    current: tuple[dict, list[dict], dict] | None = None
    with path.open(encoding="utf-8") as source:
        for raw in source:
            if not raw.strip():
                continue
            event = json.loads(raw)
            kind = event.get("event")
            if kind == "start":
                current = (event, [], {})
            elif kind == "sample" and current is not None:
                current[1].append(event)
            elif kind == "finish" and current is not None:
                current[2].update(event)
            elif kind not in ("sample", "finish"):
                raise ValueError("unknown evidence event")
    if current is None or not current[2]:
        raise ValueError(f"{path}: latest run has no completion marker")
    return current


def validate_single(path: Path) -> tuple[str, str, int, str]:
    start, samples, finish = latest_run(path)
    label = start["host_label"]
    commit = start["commit"]
    if commit == "unrecorded" or len(commit) != 40:
        raise ValueError(f"{label}: a 40-character commit SHA must be recorded")
    if not start["binary_sha256"] or len(start["binary_sha256"]) != 64:
        raise ValueError(f"{label}: missing binary checksum")
    if not finish["counts_as_72h"] or not finish["local_probe_passed"]:
        raise ValueError(f"{label}: no completed 72-hour local evidence")
    if finish["elapsed_seconds"] < 72 * 3600 or start["duration_required_seconds"] < 72 * 3600:
        raise ValueError(f"{label}: elapsed duration below 72 h")
    if finish["errors"] != 0 or not finish["binary_unchanged"]:
        raise ValueError(f"{label}: errors or binary changed during run")
    if len(samples) != finish["samples"] or len(samples) < 2:
        raise ValueError(f"{label}: sample count mismatch")
    if any(sample.get("status") != "ok" or sample.get("host_label") != label for sample in samples):
        raise ValueError(f"{label}: invalid sample")
    final = samples[-1]
    if final["height"] != finish["last_height"] or final["tip"] != finish["last_tip"]:
        raise ValueError(f"{label}: inconsistent final tip")
    utc_elapsed = (datetime.fromisoformat(finish["utc"]) -
                   datetime.fromisoformat(start["utc"])).total_seconds()
    # Allow minor UTC rounding and NTP corrections, but not a 72 h claim
    # from a run whose timestamps span only minutes.
    if utc_elapsed < 71.5 * 3600:
        raise ValueError(f"{label}: UTC sample interval below 71.5 h")
    if finish["distinct_heights"] < 2:
        raise ValueError(f"{label}: no observed block progression")
    return label, commit, final["height"], final["tip"]


def compare(left: Path, right: Path) -> tuple[int, str]:
    a, b = validate_single(left), validate_single(right)
    if a[0] == b[0]:
        raise ValueError("host labels must differ")
    if a[1] != b[1]:
        raise ValueError("hosts ran different AURORA commits")
    if a[2:] != b[2:]:
        raise ValueError("host height or tip differs; stop mining, resync and collect final proof")
    return a[2], a[3]


def self_test() -> None:
    assert callable(compare)
    try:
        validate_single(Path("/nonexistent-aurora-evidence.jsonl"))
    except FileNotFoundError:
        pass
    else:
        raise AssertionError("missing evidence was accepted")
    print("cross-host evidence comparison parser self-test passed")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--left", type=Path)
    parser.add_argument("--right", type=Path)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    if args.left is None or args.right is None:
        parser.error("--left and --right required")
    height, tip = compare(args.left, args.right)
    print(f"Evidence format checks passed; both hosts report height={height} tip={tip}")
    print("Operator must still verify distinct physical hardware, network incidents and genuine logs.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

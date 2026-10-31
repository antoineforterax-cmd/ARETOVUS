#!/usr/bin/env python3
"""Check two completed AURORA physical-host logs; never attest hardware identity."""
from __future__ import annotations

import argparse
import json
import re
import tempfile
from datetime import datetime, timedelta, timezone
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
            elif kind == "sample" and current is not None and not current[2]:
                current[1].append(event)
            elif kind == "finish" and current is not None and not current[2]:
                current[2].update(event)
            else:
                raise ValueError(f"{path}: invalid event order or unknown event")
    if current is None or not current[2]:
        raise ValueError(f"{path}: latest run has no completion marker")
    return current


def validate_single(path: Path, minimum_hours: int) -> tuple[str, str, int, str, datetime, datetime]:
    start, samples, finish = latest_run(path)
    label = start["host_label"]
    commit = start["commit"]
    if not re.fullmatch(r"[a-fA-F0-9]{40}", commit):
        raise ValueError(f"{label}: a real 40-character Git SHA is required")
    if not re.fullmatch(r"[a-fA-F0-9]{64}", start["binary_sha256"]):
        raise ValueError(f"{label}: missing or malformed binary checksum")
    if not finish.get("local_probe_passed") or not finish.get("duration_reached"):
        raise ValueError(f"{label}: no completed local endurance evidence")
    if minimum_hours == 72 and not finish.get("counts_as_72h"):
        raise ValueError(f"{label}: 72h gate not met")
    if minimum_hours == 4 and not (
        finish.get("counts_as_4h") or finish.get("counts_as_72h")
    ):
        raise ValueError(f"{label}: 4h gate not met")
    required = minimum_hours * 3600
    if finish["elapsed_seconds"] < required or start["duration_required_seconds"] < required:
        raise ValueError(f"{label}: elapsed duration below {minimum_hours}h")
    if finish["errors"] != 0 or not finish["binary_unchanged"]:
        raise ValueError(f"{label}: errors or binary changed during run")
    if len(samples) != finish["samples"] or len(samples) < 2:
        raise ValueError(f"{label}: sample count mismatch")
    if any(sample.get("status") != "ok" or sample.get("host_label") != label
           for sample in samples):
        raise ValueError(f"{label}: invalid sample")
    if any(samples[i]["elapsed_seconds"] > samples[i + 1]["elapsed_seconds"]
           for i in range(len(samples) - 1)):
        raise ValueError(f"{label}: elapsed sample time moved backwards")
    final = samples[-1]
    if final["height"] != finish["last_height"] or final["tip"] != finish["last_tip"]:
        raise ValueError(f"{label}: inconsistent final tip")
    if finish["distinct_heights"] < 2 or len({sample["height"] for sample in samples}) < 2:
        raise ValueError(f"{label}: no observed block progression")
    start_time = datetime.fromisoformat(start["utc"])
    finish_time = datetime.fromisoformat(finish["utc"])
    if start_time.tzinfo is None or finish_time.tzinfo is None:
        raise ValueError(f"{label}: UTC timestamps must include timezone")
    # Tolerate at most 30 minutes of wall-clock adjustments. This check is
    # supplementary to monotonic elapsed time recorded by the probe.
    if (finish_time - start_time).total_seconds() < required - 1800:
        raise ValueError(f"{label}: UTC interval below {minimum_hours}h gate")
    return label, commit, final["height"], final["tip"], start_time, finish_time


def compare(left: Path, right: Path, minimum_hours: int = 4) -> tuple[int, str]:
    if minimum_hours not in (4, 72):
        raise ValueError("minimum_hours must be 4 or 72")
    a, b = validate_single(left, minimum_hours), validate_single(right, minimum_hours)
    if a[0] == b[0]:
        raise ValueError("host labels must differ")
    if a[1] != b[1]:
        raise ValueError("hosts ran different AURORA commits")
    if a[2:4] != b[2:4]:
        raise ValueError("host height or tip differs; stop mining and resync")
    overlap = (min(a[5], b[5]) - max(a[4], b[4])).total_seconds()
    if overlap < minimum_hours * 3600 * 0.75:
        raise ValueError("two host observation windows do not sufficiently overlap")
    return a[2], a[3]


def self_test() -> None:
    # Synthetic fixtures test only the parser and gate logic; never count as
    # evidence of physical uptime, network resilience, or 4h of testing.
    def fixture(label: str, hours: int, tip: str, start_offset_hours: int = 0) -> str:
        begin = datetime(2026, 10, 31, tzinfo=timezone.utc) + timedelta(hours=start_offset_hours)
        end = begin + timedelta(hours=hours)
        common = {"host_label": label}
        events = [
            {**common, "event": "start", "utc": begin.isoformat(), "commit": "a" * 40,
             "binary_sha256": "b" * 64, "duration_required_seconds": hours * 3600},
            {**common, "event": "sample", "utc": begin.isoformat(), "status": "ok",
             "height": 1, "tip": "c" * 64, "elapsed_seconds": 0},
            {**common, "event": "sample", "utc": end.isoformat(), "status": "ok",
             "height": 2, "tip": tip, "elapsed_seconds": hours * 3600},
            {**common, "event": "finish", "utc": end.isoformat(),
             "local_probe_passed": True, "duration_reached": True,
             "counts_as_4h": hours >= 4, "counts_as_72h": hours >= 72,
             "elapsed_seconds": hours * 3600, "errors": 0,
             "binary_unchanged": True, "samples": 2, "distinct_heights": 2,
             "last_height": 2, "last_tip": tip},
        ]
        return "\n".join(json.dumps(e) for e in events) + "\n"

    with tempfile.TemporaryDirectory(prefix="aurora-evidence-selftest-") as d:
        left, right = Path(d, "a.jsonl"), Path(d, "b.jsonl")
        left.write_text(fixture("pc-a", 4, "d" * 64), encoding="utf-8")
        right.write_text(fixture("pc-b", 4, "d" * 64), encoding="utf-8")
        assert compare(left, right, 4) == (2, "d" * 64)
        try:
            compare(left, right, 72)
        except ValueError:
            pass
        else:
            raise AssertionError("4h test improperly accepted as 72h")
        right.write_text(fixture("pc-b", 4, "e" * 64), encoding="utf-8")
        try:
            compare(left, right)
        except ValueError:
            pass
        else:
            raise AssertionError("mismatched tips accepted")
        right.write_text(fixture("pc-b", 4, "d" * 64, start_offset_hours=8), encoding="utf-8")
        try:
            compare(left, right)
        except ValueError:
            pass
        else:
            raise AssertionError("nonoverlapping windows accepted")
        left.write_text(fixture("pc-a", 72, "d" * 64), encoding="utf-8")
        right.write_text(fixture("pc-b", 72, "d" * 64), encoding="utf-8")
        assert compare(left, right, 72) == (2, "d" * 64)
    print("synthetic 4h/72h gates, tip mismatch, overlap checks passed")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--left", type=Path)
    parser.add_argument("--right", type=Path)
    parser.add_argument("--minimum-hours", type=int, choices=(4, 72), default=4)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    if args.left is None or args.right is None:
        parser.error("--left and --right required")
    height, tip = compare(args.left, args.right, args.minimum_hours)
    print(f"Local {args.minimum_hours}h evidence format checks passed; "
          f"both host logs report height={height} tip={tip}")
    print("Operator must still verify independent physical PCs and genuine logs.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

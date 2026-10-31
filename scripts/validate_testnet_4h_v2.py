#!/usr/bin/env python3
"""Strict v2 gate: two completed node probes AND two intact resource timelines.

Static evidence verification only. Does not attest physical identity or uptime.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

from compare_testnet_evidence import compare, latest_run


def validate_timeline(node_evidence: Path, resource_evidence: Path,
                      minimum_hours: int) -> tuple[int, float]:
    node_start, _, node_finish = latest_run(node_evidence)
    expected_hash = node_finish.get("resource_log_sha256")
    if not isinstance(expected_hash, str) or len(expected_hash) != 64 or any(
        x not in "0123456789abcdef" for x in expected_hash
    ):
        raise ValueError("Node finish lacks a valid resource trace checksum")
    actual = hashlib.sha256(resource_evidence.read_bytes()).hexdigest()
    if actual != expected_hash:
        raise ValueError("Resource trace SHA256 mismatch")
    begin, samples, finish = latest_run(resource_evidence)
    if begin.get("host_label") != node_start["host_label"]:
        raise ValueError("Resource trace belongs to a different host label")
    interval = begin.get("interval_seconds")
    if not isinstance(interval, (int, float)) or not 1 <= interval <= 300:
        raise ValueError("Invalid resource sample interval")
    if not isinstance(finish.get("errors"), int) or finish["errors"] != 0:
        raise ValueError("Resource monitor recorded errors")
    if finish.get("samples") != len(samples) or len(samples) < 2:
        raise ValueError("Resource sample count mismatch")
    observed = []
    for sample in samples:
        if sample.get("event") != "sample" or sample.get("status") != "ok":
            raise ValueError("Resource sample failed")
        if sample.get("host_label") != node_start["host_label"]:
            raise ValueError("Resource sample host mismatch")
        sec = sample.get("elapsed_seconds")
        if not isinstance(sec, (int, float)) or not math.isfinite(sec) or sec < 0:
            raise ValueError("Malformed resource sample time")
        observed.append(sec)
        disk = sample.get("free_disk_bytes")
        if not isinstance(disk, int) or disk < 0:
            raise ValueError("Invalid free space sample")
        for key in ("ram_load_percent", "cpu_percent"):
            value = sample.get(key)
            if value is not None and (
                not isinstance(value, (int, float)) or not math.isfinite(value)
                or not 0 <= value <= 100
            ):
                raise ValueError("Invalid " + key)
        if begin.get("platform") in ("win32", "linux"):
            if not isinstance(sample.get("ram_load_percent"), (int, float)):
                raise ValueError("Missing RAM load on Windows/Linux")
            if not isinstance(sample.get("available_ram_bytes"), int):
                raise ValueError("Missing available RAM")
    if observed[0] > max(15, 2 * interval):
        raise ValueError("Resource monitoring started too late")
    if any(a >= b for a, b in zip(observed, observed[1:])):
        raise ValueError("Resource monotonic timestamps invalid")
    max_gap = max(15, 3 * interval)
    if any(b - a > max_gap for a, b in zip(observed, observed[1:])):
        raise ValueError("Resource observation gap exceeds allowed interval")
    elapsed = finish.get("elapsed_seconds")
    if not isinstance(elapsed, (int, float)) or not math.isfinite(elapsed):
        raise ValueError("Resource finish elapsed time invalid")
    if elapsed < minimum_hours * 3600 - max(20, 2 * interval):
        raise ValueError("Insufficient continuous resource monitoring coverage")
    if observed[-1] > elapsed or elapsed - observed[-1] > max_gap:
        raise ValueError("Resource trace ends too early")
    return len(samples), max(b - a for a, b in zip(observed, observed[1:]))


def self_test() -> None:
    def record(path: Path, objects: list[dict]) -> None:
        path.write_text("".join(json.dumps(x, sort_keys=True) + "\n" for x in objects),
                        encoding="utf-8")

    with tempfile.TemporaryDirectory(prefix="aurora-physical-v2-") as folder:
        root = Path(folder)
        begin = datetime(2026, 10, 31, tzinfo=timezone.utc)
        tip = "d" * 64
        commit = "b" * 40
        for label in ("pc-a", "pc-b"):
            telemetry = root / f"{label}.resources.jsonl"
            events = [{"event": "start", "host_label": label, "interval_seconds": 100,
                       "version": 1, "platform": "win32"}]
            for second in range(0, 14401, 100):
                events.append({
                    "event": "sample", "host_label": label, "status": "ok",
                    "elapsed_seconds": second, "ram_load_percent": 90,
                    "available_ram_bytes": 500000000, "free_disk_bytes": 1000000000,
                    "cpu_percent": None if second == 0 else 35,
                })
            events.append({"event": "finish", "host_label": label,
                           "elapsed_seconds": 14401, "errors": 0, "samples": 145})
            record(telemetry, events)
            digest = hashlib.sha256(telemetry.read_bytes()).hexdigest()
            raw = root / f"{label}.jsonl"
            items = [
                {"event": "start", "host_label": label, "utc": begin.isoformat(),
                 "duration_required_seconds": 14400, "commit": commit,
                 "binary_sha256": "a" * 64},
                {"event": "sample", "host_label": label, "utc": begin.isoformat(),
                 "elapsed_seconds": 0, "height": 1, "tip": "c" * 64, "status": "ok"},
                {"event": "sample", "host_label": label,
                 "utc": (begin + timedelta(seconds=14401)).isoformat(),
                 "elapsed_seconds": 14401, "height": 2002, "tip": tip, "status": "ok"},
                {"event": "finish", "host_label": label,
                 "utc": (begin + timedelta(seconds=14401)).isoformat(),
                 "elapsed_seconds": 14401, "last_height": 2002,
                 "last_tip": tip, "errors": 0, "samples": 2,
                 "distinct_heights": 2, "binary_unchanged": True,
                 "duration_reached": True, "local_probe_passed": True,
                 "counts_as_4h": True, "counts_as_72h": False,
                 "resource_log_sha256": digest},
            ]
            record(raw, items)
        assert compare(root / "pc-a.jsonl", root / "pc-b.jsonl", 4)[0] == 2002
        assert validate_timeline(root / "pc-a.jsonl", root / "pc-a.resources.jsonl", 4)[0] == 145
        assert validate_timeline(root / "pc-b.jsonl", root / "pc-b.resources.jsonl", 4)[0] == 145

        with (root / "pc-b.resources.jsonl").open("a", encoding="utf-8") as out:
            out.write("tampered\n")
        try:
            validate_timeline(root / "pc-b.jsonl", root / "pc-b.resources.jsonl", 4)
        except ValueError:
            pass
        else:
            raise AssertionError("Tampered monitoring log was accepted")

        node = root / "pc-b.jsonl"
        lines = node.read_text().splitlines()
        end = json.loads(lines[-1])
        end["errors"] = 1
        end["local_probe_passed"] = False
        end["counts_as_4h"] = False
        lines[-1] = json.dumps(end)
        node.write_text("\n".join(lines) + "\n", encoding="utf-8")
        try:
            compare(root / "pc-a.jsonl", node, 4)
        except ValueError:
            pass
        else:
            raise AssertionError("Failed 4h probe passed the gate")
    print("physical_4h_v2_evidence_selftest=ok (synthetic fixtures only)")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--left", type=Path)
    parser.add_argument("--right", type=Path)
    parser.add_argument("--left-resources", type=Path)
    parser.add_argument("--right-resources", type=Path)
    parser.add_argument("--minimum-hours", choices=(4, 72), type=int, default=4)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    if any(x is None for x in
           (args.left, args.right, args.left_resources, args.right_resources)):
        parser.error("--left, --right, --left-resources, --right-resources required")
    height, tip = compare(args.left, args.right, args.minimum_hours)
    a = validate_timeline(args.left, args.left_resources, args.minimum_hours)
    b = validate_timeline(args.right, args.right_resources, args.minimum_hours)
    print(f"Evidence format checks v2 passed: height={height} tip={tip}; "
          f"resources samples A={a[0]} B={b[0]}; max gaps {a[1]}s/{b[1]}s")
    print("Requires independent confirmation of two physical hosts and genuine logs.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

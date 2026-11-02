#!/usr/bin/env python3
"""D1: bounded read-only verify latency while a disposable miner writes snapshots.

Local testnet only. This checks read consistency under a concurrent writer;
it does not bypass the writer lock or simulate a physical power failure.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import statistics
import subprocess
import tempfile
import time


def verify(binary: Path, directory: Path) -> tuple[int, str, float]:
    start = time.monotonic()
    result = subprocess.run(
        [str(binary), "testnet", "verify", "--data-dir", str(directory)],
        text=True, capture_output=True, timeout=12,
    )
    duration = time.monotonic() - start
    match = re.search(r"height=(\d+) tip=([0-9a-f]{64})", result.stdout)
    if result.returncode != 0 or not match:
        raise AssertionError("read-only verify rejected snapshot while miner active: "
                             + (result.stdout + result.stderr)[-1200:])
    return int(match[1]), match[2], duration


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--binary", required=True, type=Path)
    p.add_argument("--report", required=True, type=Path)
    p.add_argument("--samples", type=int, default=8)
    a = p.parse_args()
    if not 2 <= a.samples <= 32 or os.name != "posix":
        p.error("2..32 samples on Unix only")
    binary = a.binary.resolve(strict=True)
    a.report.parent.mkdir(parents=True, exist_ok=True)
    report: dict[str, object] = {
        "status": "failed", "git_sha": os.getenv("GITHUB_SHA", "unknown"),
        "run_id": os.getenv("GITHUB_RUN_ID", "local"),
        "scope": "temporary local node, live writer and read-only verify",
        "not_measured": ["two physical PCs", "72h endurance", "100k-block verify latency"],
    }
    with tempfile.TemporaryDirectory(prefix="aretovus-d1-verify space ") as temp:
        root = Path(temp)
        storage = root / "node data"
        miner_log = root / "miner.log"
        try:
            seed = subprocess.run(
                [str(binary), "testnet", "mine", "--data-dir", str(storage),
                 "--miner", "11" * 32, "--count", "8"],
                text=True, capture_output=True, timeout=50,
            )
            if seed.returncode != 0:
                raise RuntimeError("initial 8 blocks failed: " + (seed.stdout + seed.stderr)[-800:])
            initial_height, initial_tip, _ = verify(binary, storage)
            if initial_height != 8:
                raise AssertionError("expected seed height 8")
            with miner_log.open("wb") as out:
                miner = subprocess.Popen(
                    [str(binary), "testnet", "mine", "--data-dir", str(storage),
                     "--miner", "11" * 32, "--count", "10000"],
                    stdout=out, stderr=subprocess.STDOUT,
                )
                heights: list[int] = []
                timings: list[float] = []
                try:
                    for index in range(a.samples):
                        if miner.poll() is not None:
                            raise AssertionError("miner exited before read-only sample "
                                                 + str(index))
                        height, _tip, latency = verify(binary, storage)
                        if height < initial_height:
                            raise AssertionError("live verify fell below committed seed")
                        heights.append(height)
                        timings.append(latency)
                        time.sleep(0.03)
                finally:
                    if miner.poll() is None:
                        miner.terminate()
                    try:
                        miner.wait(timeout=6)
                    except subprocess.TimeoutExpired:
                        miner.kill()
                        miner.wait(timeout=6)
            final_height, final_tip, final_latency = verify(binary, storage)
            if final_height < max(heights):
                raise AssertionError("post-mining verify lost a previously visible committed tip")
            report.update({
                "samples": len(timings),
                "initial_height": initial_height,
                "live_observed_heights": heights,
                "post_stop_height": final_height,
                "read_only_verify_seconds": [round(t, 5) for t in timings],
                "verify_median_seconds": round(statistics.median(timings), 5),
                "verify_max_seconds": round(max(timings), 5),
                "post_stop_verify_seconds": round(final_latency, 5),
                "status": "success",
            })
            print(json.dumps(report))
        except Exception as error:
            report["error"] = f"{type(error).__name__}: {error}"
            if miner_log.is_file():
                report["miner_log_tail"] = miner_log.read_text(errors="replace")[-800:]
            raise
        finally:
            a.report.write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()

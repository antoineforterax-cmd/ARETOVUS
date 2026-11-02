#!/usr/bin/env python3
"""Check sequential P2P lease recovery on a disposable loopback node only."""
from __future__ import annotations

import argparse
import json
import os
import re
import socket
import subprocess
import tempfile
import time
from pathlib import Path

from testnet_admission_probe import wallet_probe


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--cycles", type=int, default=8)
    parser.add_argument("--recovery-seconds", type=float, default=2.0)
    args = parser.parse_args()
    if not 1 <= args.cycles <= 16 or not 0.1 <= args.recovery_seconds <= 5:
        parser.error("cycles must be 1..16 and recovery-seconds 0.1..5")
    binary = args.binary.resolve(strict=True)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    report = {
        "status": "failed",
        "head_sha": os.getenv("GITHUB_SHA", "local"),
        "run_id": os.getenv("GITHUB_RUN_ID", "local"),
        "topology": "one disposable local node, sequential clients, 127.0.0.1 only",
        "cycles": [],
    }
    with tempfile.TemporaryDirectory(prefix="aretovus-lease-recovery-") as temporary:
        log = Path(temporary) / "server.log"
        with log.open("wb") as output:
            node = subprocess.Popen(
                [str(binary), "testnet", "serve",
                 "--data-dir", str(Path(temporary) / "node"),
                 "--listen", "127.0.0.1:0"],
                stdout=output, stderr=subprocess.STDOUT,
            )
            try:
                deadline = time.monotonic() + 15
                while True:
                    match = re.search(
                        r"listening on 127\.0\.0\.1:(\d+)",
                        log.read_text(errors="replace"),
                    )
                    if match is not None:
                        port = int(match.group(1))
                        break
                    if node.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError("local node startup failed")
                    time.sleep(0.02)

                report["baseline_wallet_seconds"] = wallet_probe(port)
                for index in range(args.cycles):
                    # Exactly one short-lived invalid client at a time.
                    with socket.create_connection(("127.0.0.1", port), timeout=3) as client:
                        client.sendall(b"\x00\x00\x00\x00")
                    started = time.monotonic()
                    attempts = 0
                    last_error = None
                    while True:
                        attempts += 1
                        try:
                            latency = wallet_probe(port)
                            break
                        except (ConnectionError, OSError, TimeoutError) as exc:
                            last_error = type(exc).__name__
                            if time.monotonic() - started >= args.recovery_seconds:
                                raise AssertionError(
                                    f"wallet did not recover in cycle {index}: {last_error}"
                                ) from exc
                            time.sleep(0.05)
                    report["cycles"].append({
                        "index": index,
                        "attempts": attempts,
                        "recovery_seconds": round(time.monotonic() - started, 4),
                        "wallet_seconds": round(latency, 4),
                        "initial_error": last_error,
                    })
                    if node.poll() is not None:
                        raise AssertionError("node exited during sequential recovery test")
                report["status"] = "success"
                print(json.dumps(report), flush=True)
            except Exception as error:
                report["error"] = f"{type(error).__name__}: {error}"
                raise
            finally:
                node.terminate()
                try:
                    node.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    node.kill()
                    node.wait(timeout=5)
                args.report.write_text(
                    json.dumps(report, indent=2) + "\n", encoding="utf-8"
                )


if __name__ == "__main__":
    main()

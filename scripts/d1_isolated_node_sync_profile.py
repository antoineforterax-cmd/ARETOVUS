#!/usr/bin/env python3
"""D1: isolated process RSS/CPU during a disposable, loopback-only testnet sync.

This is a short CI baseline, not physical multi-host endurance or a benchmark
of 100k blocks. No wallet keys, privileged operations or public peers.
"""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time


def resident_bytes(pid: int) -> int:
    status = Path(f"/proc/{pid}/status").read_text(encoding="utf-8")
    match = re.search(r"^VmRSS:\s+(\d+)\s+kB", status, flags=re.MULTILINE)
    if match is None:
        raise RuntimeError(f"RSS missing for process {pid}")
    return int(match.group(1)) * 1024


def cpu_seconds(pid: int) -> float:
    stat = Path(f"/proc/{pid}/stat").read_text(encoding="utf-8")
    fields = stat.rsplit(")", 1)[1].split()
    return (int(fields[11]) + int(fields[12])) / os.sysconf("SC_CLK_TCK")


def verified_state(binary: Path, directory: Path) -> tuple[int, str]:
    result = subprocess.run(
        [str(binary), "testnet", "verify", "--data-dir", str(directory)],
        text=True, capture_output=True, timeout=45,
    )
    if result.returncode:
        raise AssertionError(f"verify failed: {(result.stdout + result.stderr)[-1500:]}")
    match = re.search(r"height=(\d+) tip=([0-9a-f]{64})", result.stdout)
    if not match:
        raise AssertionError("verify produced no height and tip")
    return (int(match.group(1)), match.group(2))


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--binary", type=Path, required=True)
    p.add_argument("--report", type=Path, required=True)
    p.add_argument("--blocks", type=int, default=128)
    p.add_argument("--max-rss-mib", type=int, default=512)
    a = p.parse_args()
    if not 8 <= a.blocks <= 512 or not 16 <= a.max_rss_mib <= 2048:
        p.error("only 8..512 blocks and 16..2048 MiB RSS are permitted")
    if os.name != "posix" or not Path("/proc/self/status").exists():
        p.error("Linux /proc required for isolated process sampling")
    binary = a.binary.resolve(strict=True)
    report: dict[str, object] = {
        "status": "failed", "scope": "single Ubuntu runner, two local node processes",
        "git_sha": os.getenv("GITHUB_SHA", "unknown"),
        "run_id": os.getenv("GITHUB_RUN_ID", "local"),
        "requested_blocks": a.blocks,
        "not_measured": ["public WAN", "physical two PCs", "72h soak", "100k-block RSS"],
    }
    a.report.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="aretovus-d1-sync space ") as temp:
        root = Path(temp)
        source, target = root / "source", root / "target"
        server_log = root / "server.log"
        try:
            mine = subprocess.run([
                str(binary), "testnet", "mine", "--data-dir", str(source),
                "--miner", "11" * 32, "--count", str(a.blocks),
            ], text=True, capture_output=True, timeout=180)
            if mine.returncode:
                raise AssertionError("mine failed: " + (mine.stdout + mine.stderr)[-1500:])
            expected_height, expected_tip = verified_state(binary, source)
            if expected_height != a.blocks:
                raise AssertionError(f"expected {a.blocks} blocks; got {expected_height}")

            with server_log.open("wb") as server_output:
                server = subprocess.Popen([
                    str(binary), "testnet", "serve", "--data-dir", str(source),
                    "--listen", "127.0.0.1:0",
                ], stdout=server_output, stderr=subprocess.STDOUT)
                try:
                    deadline = time.monotonic() + 20
                    while True:
                        text = server_log.read_text(errors="replace")
                        match = re.search(r"listening on 127\.0\.0\.1:(\d+)", text)
                        if match:
                            port = int(match.group(1))
                            break
                        if server.poll() is not None or time.monotonic() > deadline:
                            raise RuntimeError("source listener unavailable: " + text[-1200:])
                        time.sleep(0.025)

                    server_rss_max = resident_bytes(server.pid)
                    server_cpu_before = cpu_seconds(server.pid)
                    client_log = root / "client.log"
                    with client_log.open("wb") as client_output:
                        started = time.monotonic()
                        client = subprocess.Popen([
                            str(binary), "testnet", "sync", "--data-dir", str(target),
                            "--peer", f"127.0.0.1:{port}",
                        ], stdout=client_output, stderr=subprocess.STDOUT)
                        client_rss_max = 0
                        client_cpu_at_last = 0.0
                        samples = 0
                        try:
                            while client.poll() is None:
                                # A short-lived sync can exit between poll()
                                # and reading /proc. Do not mistake a zombie
                                # without VmRSS for a node failure; only skip
                                # the sample if the child actually exited.
                                try:
                                    sampled_rss = resident_bytes(client.pid)
                                    sampled_cpu = cpu_seconds(client.pid)
                                except (FileNotFoundError, RuntimeError):
                                    if client.poll() is not None:
                                        break
                                    raise
                                client_rss_max = max(client_rss_max, sampled_rss)
                                client_cpu_at_last = sampled_cpu
                                server_rss_max = max(server_rss_max, resident_bytes(server.pid))
                                samples += 1
                                time.sleep(0.01)
                            elapsed = time.monotonic() - started
                            rc = client.wait(timeout=5)
                        finally:
                            if client.poll() is None:
                                client.kill()
                                client.wait(timeout=5)
                    if rc != 0:
                        raise AssertionError("target sync failed: " + client_log.read_text(errors="replace")[-1800:])
                    if samples < 1 or client_rss_max == 0:
                        raise AssertionError("sync completed before isolated RSS could be sampled")
                    result = verified_state(binary, target)
                    if result != (expected_height, expected_tip):
                        raise AssertionError(f"post-sync mismatch: expected {(expected_height, expected_tip)}, got {result}")
                    server_cpu_used = cpu_seconds(server.pid) - server_cpu_before
                    report.update({
                        "height": expected_height, "tip": expected_tip,
                        "duration_seconds": round(elapsed, 3),
                        "sample_interval_seconds": 0.01,
                        "sample_count": samples,
                        "source_server_rss_peak_bytes_sampled": server_rss_max,
                        "target_sync_rss_peak_bytes_sampled": client_rss_max,
                        "server_cpu_seconds_sampled": round(server_cpu_used, 4),
                        "target_cpu_seconds_last_sample": round(client_cpu_at_last, 4),
                        "source_disk_file_bytes": sum(f.stat().st_size for f in source.rglob("*") if f.is_file()),
                        "target_disk_file_bytes": sum(f.stat().st_size for f in target.rglob("*") if f.is_file()),
                    })
                    if max(server_rss_max, client_rss_max) > a.max_rss_mib * (1024 ** 2):
                        raise AssertionError("sampled node RSS exceeded configured safety cap")
                    report["status"] = "success"
                    print(json.dumps(report))
                finally:
                    server.terminate()
                    try:
                        server.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        server.kill()
                        server.wait(timeout=5)
        except Exception as error:
            report["error"] = f"{type(error).__name__}: {error}"
            raise
        finally:
            a.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Measured local three-node endurance run using an already built node binary."""
from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import os
import re
import shutil
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

from testnet_smoke import free_address, wait_listening, run_node
from testnet_soak import await_tip, checked_tip


def utc() -> str:
    return datetime.now(timezone.utc).isoformat()


def resident_bytes(pid: int) -> int:
    if os.name == "nt":
        from ctypes import wintypes

        class Counters(ctypes.Structure):
            _fields_ = [("cb", wintypes.DWORD), ("faults", wintypes.DWORD)] + [
                (name, ctypes.c_size_t) for name in (
                    "peak_working_set", "working_set", "peak_paged_pool", "paged_pool",
                    "peak_nonpaged_pool", "nonpaged_pool", "pagefile", "peak_pagefile")]

        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        psapi = ctypes.WinDLL("psapi", use_last_error=True)
        kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
        kernel.OpenProcess.restype = wintypes.HANDLE
        kernel.CloseHandle.argtypes = [wintypes.HANDLE]
        psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(Counters), wintypes.DWORD]
        handle = kernel.OpenProcess(0x1000, False, pid)
        if not handle:
            raise ctypes.WinError(ctypes.get_last_error())
        try:
            counters = Counters()
            counters.cb = ctypes.sizeof(counters)
            if not psapi.GetProcessMemoryInfo(handle, ctypes.byref(counters), counters.cb):
                raise ctypes.WinError(ctypes.get_last_error())
            return counters.working_set
        finally:
            kernel.CloseHandle(handle)
    status = Path(f"/proc/{pid}/status")
    if status.exists():
        match = re.search(r"^VmRSS:\s+(\d+)\s+kB", status.read_text(), re.MULTILINE)
        if match is None:
            raise RuntimeError(f"RSS missing for process {pid}")
        return int(match.group(1)) * 1024
    return int(subprocess.check_output(["ps", "-o", "rss=", "-p", str(pid)], text=True)) * 1024


def directory_bytes(path: Path) -> int:
    total = 0
    for file in path.rglob("*"):
        try:
            if file.is_file():
                total += file.stat().st_size
        except FileNotFoundError:  # Old manifests may be pruned during measurement.
            pass
    return total


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new directory; never reuse node data")
    parser.add_argument("--hours", type=float, default=72)
    parser.add_argument("--interval-seconds", type=float, default=60)
    parser.add_argument("--restart-every", type=int, default=30)
    parser.add_argument("--max-rounds", type=int, default=0, help="short validation only; 0 means duration-based")
    parser.add_argument("--max-disk-gib", type=float, default=20)
    parser.add_argument("--max-node-memory-gib", type=float, default=4)
    args = parser.parse_args()
    if not (0 < args.hours <= 168 and 0 <= args.interval_seconds <= 300
            and args.restart_every >= 2 and args.max_rounds >= 0
            and args.max_disk_gib > 0 and args.max_node_memory_gib > 0):
        parser.error("invalid duration, interval, restart count or resource limit")
    binary = args.binary.resolve(strict=True)
    base = args.output.resolve()
    base.mkdir(parents=True, exist_ok=False)
    dirs = {name: base / name for name in ("a", "b", "c", "miner")}
    addresses = {name: free_address() for name in ("a", "b", "c")}
    while len(set(addresses.values())) != 3:
        addresses = {name: free_address() for name in ("a", "b", "c")}
    servers = {}
    started = time.monotonic()
    metadata = {"started_utc": utc(), "supervisor_pid": os.getpid(),
                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                "requested_hours": args.hours, "topology": "three processes on one host",
                "interval_seconds": args.interval_seconds, "restart_every": args.restart_every,
                "max_rounds": args.max_rounds, "addresses": addresses}
    (base / "metadata.json").write_text(json.dumps(metadata, indent=2), encoding="utf-8")

    def endpoint(name):
        return "%s:%d" % addresses[name]

    def record(event):
        event.update(utc=utc(), elapsed_seconds=round(time.monotonic() - started, 3))
        with (base / "measurements.jsonl").open("a", encoding="utf-8") as stream:
            stream.write(json.dumps(event) + "\n")
            stream.flush()
            os.fsync(stream.fileno())

    def start(name, peer):
        log = base / f"{name}.log"
        stream = log.open("a", encoding="utf-8")
        try:
            process = subprocess.Popen(
                [str(binary), "testnet", "serve", "--data-dir", str(dirs[name]),
                 "--listen", endpoint(name), "--peer", endpoint(peer)],
                stdout=stream, stderr=subprocess.STDOUT, text=True)
        except BaseException:
            stream.close()
            raise
        servers[name] = process, stream
        wait_listening(process, addresses[name], log)
        record({"event": "start", "node": name, "pid": process.pid})

    def stop(name, force=False):
        process, stream = servers.pop(name)
        try:
            if process.poll() is None:
                process.kill() if force else process.terminate()
            process.wait(timeout=10)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)
            stream.close()
        record({"event": "forced_stop" if force else "stop", "node": name})

    rounds = 0
    result = {"status": "failed", "duration_completed": False}
    try:
        start("a", "b")
        start("b", "a")
        start("c", "b")
        last_activity = time.monotonic()
        while True:
            if time.monotonic() - last_activity > args.interval_seconds + 300:
                raise RuntimeError("unexplained monitoring gap; do not count suspension as continuous endurance")
            if time.monotonic() - started >= args.hours * 3600:
                result.update(status="duration_passed", duration_completed=True)
                break
            if (base / "STOP").exists():
                result["status"] = "stopped_by_user"
                break
            rounds += 1
            round_started = time.monotonic()
            restart = rounds % args.restart_every == 0
            if restart:
                stop("b", force=True)
            output = run_node(binary, "testnet", "mine", "--data-dir", str(dirs["miner"]),
                              "--miner", "41" * 32, "--count", "1", "--peer", endpoint("a"))
            match = re.search(r"height=(\d+) tip=([0-9a-f]{64})", output)
            if match is None or int(match.group(1)) != rounds:
                raise AssertionError(f"unexpected mining result: {output}")
            expected = rounds, match.group(2)
            await_tip(binary, dirs["a"], expected, servers["a"][0])
            recovery_started = time.monotonic()
            if restart:
                start("b", "a")
            await_tip(binary, dirs["b"], expected, servers["b"][0])
            await_tip(binary, dirs["c"], expected, servers["c"][0])
            memory = {name: resident_bytes(process.pid) for name, (process, _) in servers.items()}
            disk = directory_bytes(base)
            record({"event": "converged", "height": rounds, "tip": expected[1],
                    "forced_restart_b": restart, "rss_bytes": memory, "disk_bytes": disk,
                    "recovery_seconds": time.monotonic() - recovery_started if restart else None,
                    "round_seconds": time.monotonic() - round_started})
            print(f"height={rounds} converged=3 restart_b={restart} disk_bytes={disk}", flush=True)
            if disk > args.max_disk_gib * 1024**3 or max(memory.values()) > args.max_node_memory_gib * 1024**3:
                raise RuntimeError("configured resource limit exceeded; measurements retained")
            last_activity = time.monotonic()
            if args.max_rounds and rounds >= args.max_rounds:
                result["status"] = "short_run_passed"
                break
            delay = max(0, min(args.interval_seconds - (time.monotonic() - round_started),
                               args.hours * 3600 - (time.monotonic() - started)))
            time.sleep(delay)
        for name in list(servers):
            stop(name)
        final = checked_tip(binary, dirs["c"])
        restored = base / "b-restored"
        shutil.copytree(dirs["b"], restored)
        if checked_tip(binary, restored) != final:
            raise AssertionError("backup restoration diverged")
        result.update(backup_restored=True, final_height=final[0], final_tip=final[1])
        return 0
    except BaseException as error:
        result.update(status="failed", duration_completed=False, error=str(error))
        raise
    finally:
        for name in list(servers):
            stop(name)
        result.update(finished_utc=utc(), elapsed_seconds=time.monotonic() - started, rounds=rounds)
        (base / "summary.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
        record({"event": "summary", **result})
        print(f"evidence={base}", flush=True)


if __name__ == "__main__":
    raise SystemExit(main())

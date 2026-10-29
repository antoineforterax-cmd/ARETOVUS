#!/usr/bin/env python3
"""Repeated process-level network partitions, fork convergence and fresh-peer sync."""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import time
from pathlib import Path

from testnet_smoke import ROOT, free_address, run_node, wait_listening


def state(binary: Path, directory: Path) -> dict:
    output = run_node(binary, "testnet", "verify", "--data-dir", str(directory))
    match = re.search(r"height=(\d+) tip=([0-9a-f]{64}) known_blocks=(\d+) work=(.+)", output)
    if match is None:
        raise AssertionError(f"missing verified chain state: {output}")
    return {"height": int(match[1]), "tip": match[2],
            "known_blocks": int(match[3]), "work": match[4].strip()}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--rounds", type=int, default=3)
    args = parser.parse_args()
    if not 1 <= args.rounds <= 3:
        parser.error("rounds must be 1..3 to keep the scenario below the first retarget")
    binary = args.binary.resolve(strict=True)
    base = args.output.resolve()
    base.mkdir(parents=True, exist_ok=False)
    servers = {}
    records = []

    def endpoint(address):
        return f"{address[0]}:{address[1]}"

    def addresses():
        a, b = free_address(), free_address()
        while a == b:
            b = free_address()
        return {"a": a, "b": b}

    def start(name, address, label, peer=None):
        log = base / f"{label}-{name}.log"
        stream = log.open("w", encoding="utf-8")
        command = [str(binary), "testnet", "serve", "--data-dir", str(base / name),
                   "--listen", endpoint(address)]
        if peer is not None:
            command += ["--peer", endpoint(peer)]
        process = subprocess.Popen(command, cwd=ROOT, stdout=stream,
                                   stderr=subprocess.STDOUT, text=True)
        servers[name] = process, stream
        wait_listening(process, address, log)

    def stop_all():
        for name in list(servers):
            process, stream = servers.pop(name)
            if process.poll() is None:
                process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
            stream.close()

    try:
        for number in range(1, args.rounds + 1):
            isolated = addresses()
            start("a", isolated["a"], f"split-{number}")
            start("b", isolated["b"], f"split-{number}")
            for name, count, miner in [("a", 2, "42" * 32), ("b", 1, "43" * 32)]:
                run_node(binary, "testnet", "mine", "--data-dir", str(base / f"miner-{name}"),
                         "--miner", miner, "--count", str(count),
                         "--peer", endpoint(isolated[name]))
            split = {name: state(binary, base / name) for name in ("a", "b")}
            if split["a"]["height"] != 2 * number or split["b"]["height"] != 2 * number - 1:
                raise AssertionError(f"partition did not create expected forks: {split}")
            if split["a"]["tip"] == split["b"]["tip"]:
                raise AssertionError("isolated nodes did not diverge")
            expected = split["a"]
            stop_all()

            connected = addresses()
            started = time.monotonic()
            start("a", connected["a"], f"heal-{number}", connected["b"])
            start("b", connected["b"], f"heal-{number}", connected["a"])
            deadline = time.monotonic() + 45
            while True:
                if any(process.poll() is not None for process, _ in servers.values()):
                    raise RuntimeError("a connected node exited")
                healed = {name: state(binary, base / name) for name in ("a", "b")}
                if all(item["tip"] == expected["tip"]
                       and item["height"] == expected["height"]
                       and item["work"] == expected["work"]
                       and item["known_blocks"] == 1 + 3 * number
                       for item in healed.values()):
                    break
                if time.monotonic() >= deadline:
                    raise TimeoutError(f"partition did not heal: {healed}; expected {expected}")
                time.sleep(0.25)
            elapsed = time.monotonic() - started
            fresh = base / f"fresh-{number}"
            run_node(binary, "testnet", "sync", "--data-dir", str(fresh),
                     "--peer", endpoint(connected["b"]))
            fresh_state = state(binary, fresh)
            if fresh_state != healed["b"]:
                raise AssertionError(f"fresh peer differs: {fresh_state} != {healed['b']}")
            stop_all()
            restarted = {name: state(binary, base / name) for name in ("a", "b")}
            if restarted != healed:
                raise AssertionError("persisted states changed after stopping the servers")
            record = {"round": number, "split": split, "healed": healed,
                      "fresh": fresh_state, "convergence_seconds": elapsed}
            records.append(record)
            (base / "results.json").write_text(json.dumps(records, indent=2) + "\n", encoding="utf-8")
            print(json.dumps(record), flush=True)
        print(f"partition cycles passed: {args.rounds}", flush=True)
        return 0
    finally:
        stop_all()


if __name__ == "__main__":
    raise SystemExit(main())

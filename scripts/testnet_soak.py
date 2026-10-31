#!/usr/bin/env python3
"""Longer three-node testnet soak with peer restart and backup restoration."""
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

from testnet_smoke import ROOT, CARGO, free_address, run_node, wait_listening


def checked_tip(binary: Path, data_dir: Path) -> tuple[int, str]:
    output = run_node(binary, "testnet", "verify", "--data-dir", str(data_dir))
    match = re.search(r"height=(\d+) tip=([0-9a-f]{64})", output)
    if match is None:
        raise AssertionError(f"verify did not report a tip: {output}")
    return int(match.group(1)), match.group(2)


def await_tip(binary: Path, data_dir: Path, expected: tuple[int, str], process: subprocess.Popen[str]) -> None:
    deadline = time.monotonic() + 50
    last: tuple[int, str] | None = None
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(f"node exited while awaiting {expected}")
        last = checked_tip(binary, data_dir)
        if last == expected:
            return
        time.sleep(0.5)
    raise TimeoutError(f"{data_dir.name} stayed at {last}, expected {expected}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rounds", type=int, default=36)
    parser.add_argument("--interval-seconds", type=float, default=2.0)
    args = parser.parse_args()
    if not 1 <= args.rounds <= 200 or not 0 <= args.interval_seconds <= 60:
        parser.error("rounds must be 1..200 and interval-seconds 0..60")

    subprocess.run([CARGO, "build", "--locked", "-p", "aretovus-node"], cwd=ROOT, check=True)
    binary = ROOT / "target" / "debug" / "aretovus-node"
    if not binary.is_file():
        binary = binary.with_suffix(".exe")
    if not binary.is_file():
        raise FileNotFoundError("aretovus-node build output is missing")

    base = Path(tempfile.mkdtemp(prefix="aurora-soak-", dir=ROOT / "target"))
    dirs = {name: base / name for name in ("a", "b", "c", "miner")}
    addresses = {name: free_address() for name in ("a", "b", "c")}
    while len({address[1] for address in addresses.values()}) != 3:
        addresses = {name: free_address() for name in ("a", "b", "c")}
    servers: dict[str, tuple[subprocess.Popen[str], object]] = {}

    def endpoint(name: str) -> str:
        host, port = addresses[name]
        return f"{host}:{port}"

    def start(name: str, peer: str) -> None:
        log = base / f"{name}.log"
        stream = log.open("a", encoding="utf-8")
        process = subprocess.Popen(
            [str(binary), "testnet", "serve", "--data-dir", str(dirs[name]),
             "--listen", endpoint(name), "--peer", endpoint(peer)],
            cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, text=True,
        )
        servers[name] = process, stream
        wait_listening(process, addresses[name], log)

    def stop(name: str) -> None:
        process, stream = servers.pop(name)
        if process.poll() is None:
            process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
        stream.close()

    try:
        start("a", "b")
        start("b", "a")
        start("c", "b")
        miner = "41" * 32
        start_time = time.monotonic()
        for round_number in range(1, args.rounds + 1):
            restart_b = round_number % 9 == 0
            if restart_b:
                stop("b")
            output = run_node(
                binary, "testnet", "mine", "--data-dir", str(dirs["miner"]),
                "--miner", miner, "--count", "1", "--peer", endpoint("a"),
            )
            match = re.search(r"height=(\d+) tip=([0-9a-f]{64})", output)
            if match is None:
                raise AssertionError(f"miner did not report a tip: {output}")
            expected = int(match.group(1)), match.group(2)
            if expected[0] != round_number:
                raise AssertionError(f"miner height {expected[0]} != round {round_number}")
            await_tip(binary, dirs["a"], expected, servers["a"][0])
            if restart_b:
                start("b", "a")
            await_tip(binary, dirs["b"], expected, servers["b"][0])
            await_tip(binary, dirs["c"], expected, servers["c"][0])
            print(f"round={round_number} height={expected[0]} tip={expected[1]} "
                  f"restarted_b={restart_b}", flush=True)
            time.sleep(args.interval_seconds)

        for name in list(servers):
            stop(name)
        final = checked_tip(binary, dirs["c"])
        backup = base / "b-restored"
        shutil.copytree(dirs["b"], backup)
        if checked_tip(binary, backup) != final:
            raise AssertionError("restored full data-directory copy disagrees with the network")
        elapsed = time.monotonic() - start_time
        print(f"three-node soak passed: rounds={args.rounds} elapsed_seconds={elapsed:.1f} "
              f"height={final[0]} tip={final[1]}", flush=True)
        return 0
    finally:
        for name in list(servers):
            stop(name)
        print(f"soak logs and data: {base}", flush=True)


if __name__ == "__main__":
    raise SystemExit(main())

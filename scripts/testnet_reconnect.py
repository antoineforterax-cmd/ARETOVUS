#!/usr/bin/env python3
"""Exercise live P2P catch-up after source loss, reconnect and receiver restart."""
from __future__ import annotations

import argparse
import json
import subprocess
import time
from pathlib import Path

from testnet_partitions import state
from testnet_smoke import ROOT, free_address, run_node, wait_listening


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--deadline-seconds", type=int, default=75)
    args = parser.parse_args()
    if not 20 <= args.deadline_seconds <= 120:
        parser.error("deadline-seconds must be 20..120")
    binary = args.binary.resolve(strict=True)
    base = args.output.resolve()
    base.mkdir(parents=True, exist_ok=False)

    addresses = {"source": free_address(), "receiver": free_address()}
    while addresses["source"] == addresses["receiver"]:
        addresses["receiver"] = free_address()
    folders = {name: base / name for name in ("source", "receiver", "miner")}
    running = {}
    observations = []
    started = time.monotonic()

    def endpoint(name: str) -> str:
        host, port = addresses[name]
        return f"{host}:{port}"

    def record(label: str, source: dict, receiver: dict) -> None:
        observations.append({
            "stage": label, "source": source, "receiver": receiver,
            "elapsed_seconds": round(time.monotonic() - started, 3),
        })
        (base / "results.json").write_text(
            json.dumps(observations, indent=2) + "\n", encoding="utf-8"
        )

    def start(name: str, peer: str | None = None) -> None:
        log = base / f"{name}-{len(observations)}.log"
        stream = log.open("w", encoding="utf-8")
        command = [
            str(binary), "testnet", "serve", "--data-dir", str(folders[name]),
            "--listen", endpoint(name),
        ]
        if peer:
            command.extend(["--peer", endpoint(peer)])
        process = subprocess.Popen(
            command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, text=True
        )
        running[name] = process, stream
        wait_listening(process, addresses[name], log)

    def stop(name: str, force: bool = False) -> None:
        process, stream = running.pop(name)
        if process.poll() is None:
            if force:
                process.kill()
            else:
                process.terminate()
        try:
            process.wait(timeout=8)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)
        stream.close()

    def check_alive() -> None:
        for name, (process, _) in running.items():
            if process.poll() is not None:
                raise RuntimeError(f"{name} server exited unexpectedly")

    def await_equal(expected: dict, label: str) -> dict:
        deadline = time.monotonic() + args.deadline_seconds
        last = None
        while time.monotonic() < deadline:
            check_alive()
            last = state(binary, folders["receiver"])
            if last == expected:
                return last
            time.sleep(0.3)
        raise TimeoutError(f"{label}: receiver state {last} != source {expected}")

    try:
        # A deterministic partial P2P checkpoint: a first real network sync
        # finishes before disconnecting the source. No timing-sensitive kill
        # while a block is half-written is claimed.
        run_node(
            binary, "testnet", "mine", "--data-dir", str(folders["source"]),
            "--miner", "42" * 32, "--count", "6",
        )
        first = state(binary, folders["source"])
        if first["height"] != 6 or first["known_blocks"] != 7:
            raise AssertionError(f"unexpected first chain: {first}")

        start("source")
        start("receiver", "source")
        record("initial_sync", first, await_equal(first, "initial sync"))

        # The receiver stays online while its configured source disconnects.
        stop("source")
        if state(binary, folders["receiver"]) != first:
            raise AssertionError("receiver changed during source outage")
        run_node(
            binary, "testnet", "mine", "--data-dir", str(folders["source"]),
            "--miner", "42" * 32, "--count", "3",
        )
        second = state(binary, folders["source"])
        if second["height"] != 9 or second["known_blocks"] != 10:
            raise AssertionError(f"unexpected extended chain: {second}")
        if state(binary, folders["receiver"]) != first:
            raise AssertionError("receiver advanced without source connectivity")
        record("source_offline", second, first)

        # Reuse the original TCP endpoint: the receiver must retry, not
        # require a manual sync command or loss of its previous blocks.
        start("source")
        record("source_reconnected", second, await_equal(second, "reconnect"))

        # Force-stop receiver after convergence and verify its disk state.
        stop("receiver", force=True)
        if state(binary, folders["receiver"]) != second:
            raise AssertionError("receiver lost its persisted state after kill")
        start("receiver", "source")

        # An online third miner submits a fresh block to the restored source.
        run_node(
            binary, "testnet", "mine", "--data-dir", str(folders["miner"]),
            "--miner", "42" * 32, "--count", "1",
            "--peer", endpoint("source"),
        )
        deadline = time.monotonic() + args.deadline_seconds
        third = None
        while time.monotonic() < deadline:
            check_alive()
            third = state(binary, folders["source"])
            if third["height"] == 10 and third["known_blocks"] == 11:
                break
            time.sleep(0.3)
        else:
            raise TimeoutError(f"source did not accept third miner: {third}")
        record("receiver_restarted", third, await_equal(third, "receiver restart"))
        stop("receiver")
        stop("source")
        if state(binary, folders["receiver"]) != third:
            raise AssertionError("receiver persisted state differs after clean stop")
        print(json.dumps({
            "result": "passed", "network_blocks": third["known_blocks"],
            "height": third["height"], "tip": third["tip"],
            "elapsed_seconds": round(time.monotonic() - started, 3),
        }), flush=True)
        return 0
    finally:
        for name in list(running):
            stop(name)


if __name__ == "__main__":
    raise SystemExit(main())

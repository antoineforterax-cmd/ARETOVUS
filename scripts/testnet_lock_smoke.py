#!/usr/bin/env python3
"""Check that a testnet data directory has one writer across processes."""
from __future__ import annotations

import os
import socket
import subprocess
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NODE = ROOT / "target" / "release" / ("aretovus-node.exe" if os.name == "nt" else "aretovus-node")
MINER = "42" * 32


def free_port() -> int:
    with socket.socket() as candidate:
        candidate.bind(("127.0.0.1", 0))
        return candidate.getsockname()[1]


def wait_listening(server: subprocess.Popen[bytes], port: int, log: Path) -> None:
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        if server.poll() is not None:
            raise RuntimeError(f"server exited early: {log.read_text(errors='replace')}")
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.25):
                return
        except OSError:
            time.sleep(0.1)
    raise TimeoutError(f"server did not listen: {log.read_text(errors='replace')}")


def main() -> None:
    if not NODE.is_file():
        raise FileNotFoundError(NODE)
    with tempfile.TemporaryDirectory(prefix="aretovus-lock-smoke-") as temporary:
        data_dir = Path(temporary) / "node"
        log = Path(temporary) / "server.log"
        port = free_port()
        with log.open("wb") as stream:
            server = subprocess.Popen(
                [str(NODE), "testnet", "serve", "--data-dir", str(data_dir),
                 "--listen", f"127.0.0.1:{port}"],
                cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT,
            )
            try:
                wait_listening(server, port, log)
                mine_args = [
                    str(NODE), "testnet", "mine", "--data-dir", str(data_dir),
                    "--miner", MINER, "--count", "1",
                ]
                blocked = subprocess.run(
                    mine_args, cwd=ROOT, capture_output=True, text=True, timeout=15,
                )
                if blocked.returncode == 0 or "already in use or cannot be locked" not in blocked.stderr:
                    raise AssertionError(
                        f"concurrent writer was not rejected: {blocked.stdout}{blocked.stderr}"
                    )
            finally:
                if server.poll() is None:
                    server.terminate()
                    try:
                        server.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        server.kill()
                        server.wait(timeout=5)

        resumed = subprocess.run(
            mine_args, cwd=ROOT, capture_output=True, text=True, timeout=30,
        )
        if resumed.returncode or "height=1" not in resumed.stdout:
            raise AssertionError(f"lock was not released: {resumed.stdout}{resumed.stderr}")
        checked = subprocess.run(
            [str(NODE), "testnet", "verify", "--data-dir", str(data_dir)],
            cwd=ROOT, capture_output=True, text=True, timeout=15,
        )
        if checked.returncode or "height=1" not in checked.stdout:
            raise AssertionError(f"saved block was not verified: {checked.stdout}{checked.stderr}")
    print("cross-platform data-directory lock smoke passed")


if __name__ == "__main__":
    main()

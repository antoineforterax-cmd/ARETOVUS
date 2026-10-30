#!/usr/bin/env python3
"""Verify the *packaged* AURORA Windows node across two loopback processes.

This is not a two-computer test: both nodes share one Windows CI runner.
No wallet secrets, real funds, network configuration, or external hosts.
"""
from __future__ import annotations

import argparse
import re
import socket
import subprocess
import tempfile
import time
from pathlib import Path

TIP = re.compile(
    r"verified testnet data: height=(\d+) tip=([0-9a-fA-F]{64}) "
    r"known_blocks=(\d+) work="
)


def run(*args: object, timeout: float = 90) -> str:
    completed = subprocess.run(
        [str(arg) for arg in args],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        timeout=timeout,
        check=False,
    )
    if completed.returncode:
        raise RuntimeError(
            f"Command failed ({completed.returncode}): {args[0:3]}\n"
            + completed.stdout[-3000:]
        )
    return completed.stdout


def choose_port() -> int:
    # Reserve only briefly. All sockets remain bound to loopback.
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def await_listener(proc: subprocess.Popen[bytes], port: int, log: Path) -> None:
    end = time.monotonic() + 20
    while time.monotonic() < end:
        if proc.poll() is not None:
            raise RuntimeError(
                f"Node exited before listening on {port}:\n"
                + log.read_text(errors="replace")[-3000:]
            )
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.2):
                return
        except OSError:
            time.sleep(0.2)
    raise TimeoutError(f"Node did not listen on {port}: {log.read_text(errors='replace')[-3000:]}")


def start_server(
    node: Path, path: Path, port: int, log: Path, peer: int | None = None
) -> subprocess.Popen[bytes]:
    command = [
        str(node), "testnet", "serve", "--data-dir", str(path),
        "--listen", f"127.0.0.1:{port}",
    ]
    if peer is not None:
        command += ["--peer", f"127.0.0.1:{peer}"]
    with log.open("wb") as stream:
        process = subprocess.Popen(command, stdout=stream, stderr=subprocess.STDOUT)
    await_listener(process, port, log)
    return process


def stop(process: subprocess.Popen[bytes] | None) -> None:
    if process is not None and process.poll() is None:
        # Deliberately abrupt: verify snapshot recovery after an unclean exit.
        process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=10)


def state(node: Path, directory: Path) -> tuple[int, str, int, str]:
    verified = run(node, "testnet", "verify", "--data-dir", directory)
    match = TIP.search(verified)
    if not match:
        raise AssertionError(f"Unexpected verify output: {verified[-1000:]}")
    return int(match[1]), match[2].lower(), int(match[3]), verified.strip()


def await_tip(
    node: Path,
    directory: Path,
    expected: tuple[int, str, int, str],
    timeout: float = 55,
) -> float:
    begin = time.monotonic()
    observed = "not yet available"
    while time.monotonic() - begin < timeout:
        try:
            now = state(node, directory)
            observed = now[3]
            if now[0] == expected[0] and now[1] == expected[1]:
                if now[2] != expected[2]:
                    raise AssertionError(
                        f"Same tip but inconsistent known_blocks: {now[2]} != {expected[2]}"
                    )
                return time.monotonic() - begin
        except (OSError, RuntimeError):
            pass
        time.sleep(0.3)
    raise TimeoutError(
        f"Two-node convergence failed: expected {expected[3]}, observed {observed}"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package-root", type=Path, required=True)
    args = parser.parse_args()
    package = args.package_root.resolve()
    node = package / "bin" / "aurora-node.exe"
    if not node.is_file():
        raise FileNotFoundError(node)
    with tempfile.TemporaryDirectory(prefix="aurora-packaged-two-nodes-") as temp:
        root = Path(temp)
        a_data, b_data, miner_data = (
            root / "server-a", root / "server-b", root / "miner"
        )
        a_port, b_port = choose_port(), choose_port()
        while b_port == a_port:
            b_port = choose_port()
        a = b = None
        try:
            a = start_server(node, a_data, a_port, root / "server-a.log")
            miner = "41" * 32  # Invalid spending identity is fine for testnet mining.
            run(
                node, "testnet", "mine", "--data-dir", miner_data,
                "--miner", miner, "--count", "1", "--peer", f"127.0.0.1:{a_port}"
            )
            source1 = state(node, a_data)
            if source1[0] != 1:
                raise AssertionError(f"Mine/propagation failed: {source1[3]}")
            b = start_server(node, b_data, b_port, root / "server-b.log", a_port)
            initial = await_tip(node, b_data, source1)
            stop(b)
            b = None

            run(
                node, "testnet", "mine", "--data-dir", miner_data,
                "--miner", miner, "--count", "1", "--peer", f"127.0.0.1:{a_port}"
            )
            source2 = state(node, a_data)
            if source2[0] != 2 or source2[1] == source1[1]:
                raise AssertionError(f"Second block did not propagate: {source2[3]}")
            b = start_server(node, b_data, b_port, root / "server-b-restart.log", a_port)
            resumed = await_tip(node, b_data, source2)
            stop(b)
            b = None
            stop(a)
            a = None

            persisted_a, persisted_b = state(node, a_data), state(node, b_data)
            if persisted_a[:3] != persisted_b[:3]:
                raise AssertionError(
                    f"Persistent chain divergence: {persisted_a[3]} vs {persisted_b[3]}"
                )
            if persisted_a[0] != 2:
                raise AssertionError(f"Unexpected final height: {persisted_a[3]}")
            print(
                "packaged_windows_two_node_smoke=success "
                f"height={persisted_a[0]} tip={persisted_a[1]} "
                f"known_blocks={persisted_a[2]} "
                f"initial_sync_seconds={initial:.3f} resumed_sync_seconds={resumed:.3f}"
            )
        finally:
            stop(b)
            stop(a)


if __name__ == "__main__":
    main()

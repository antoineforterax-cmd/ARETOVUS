#!/usr/bin/env python3
"""Bounded loopback-only admission probe; Linux supports distinct 127/8 sources."""
import argparse
import json
import re
import socket
import struct
import subprocess
import tempfile
import time
from contextlib import ExitStack
from pathlib import Path


def receive(sock: socket.socket, count: int) -> bytes:
    result = bytearray()
    while len(result) < count:
        chunk = sock.recv(count - len(result))
        if not chunk:
            raise ConnectionError("server closed the connection")
        result.extend(chunk)
    return bytes(result)


def wallet_probe(port: int) -> float:
    started = time.monotonic()
    with socket.create_connection(("127.0.0.1", port), timeout=5) as client:
        client.sendall(struct.pack(">I", 1) + bytes([5]))
        size = struct.unpack(">I", receive(client, 4))[0]
        if size != 74 or receive(client, size)[:2] != bytes([0x85, 2]):
            raise AssertionError("invalid wallet handshake response")
    return time.monotonic() - started


def run_case(binary: Path, kind: str, maximum: float | None) -> dict:
    with tempfile.TemporaryDirectory(prefix="aurora-admission-") as temporary:
        root = Path(temporary)
        log = root / "server.log"
        with log.open("wb") as output:
            server = subprocess.Popen(
                [str(binary), "testnet", "serve", "--data-dir", str(root / "node"),
                 "--listen", "127.0.0.1:0"],
                stdout=output, stderr=subprocess.STDOUT,
            )
            try:
                deadline = time.monotonic() + 15
                while True:
                    match = re.search(r"listening on 127\.0\.0\.1:(\d+)", log.read_text())
                    if match:
                        port = int(match.group(1))
                        break
                    if server.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError(log.read_text())
                    time.sleep(0.01)
                idle = wallet_probe(port)
                with ExitStack() as clients:
                    trickle_clients = []
                    count = 32 if kind.startswith("saturated_") else 8
                    for index in range(count):
                        sock = clients.enter_context(socket.socket())
                        sock.settimeout(5)
                        if kind.startswith("saturated_"):
                            # Eight loopback IPs, four sockets per IP; exhaust
                            # the global limit without violating per-IP caps.
                            source = f"127.0.0.{index // 4 + 2}"
                        elif kind.endswith("after_hello"):
                            source = "127.0.0.2" if index < 4 else "127.0.0.3"
                        else:
                            source = "127.0.0.2" if kind == "same_ip" else f"127.0.0.{index + 2}"
                        sock.bind((source, 0))
                        sock.connect(("127.0.0.1", port))
                        if kind.endswith("after_hello"):
                            sock.sendall(struct.pack(">I", 1) + bytes([5]))
                            size = struct.unpack(">I", receive(sock, 4))[0]
                            if size != 74 or receive(sock, size)[:2] != bytes([0x85, 2]):
                                raise AssertionError("post-handshake session was not established")
                            if kind in ("partial_after_hello", "saturated_after_hello", "saturated_trickle_after_hello"):
                                # Complete hello, then deliberately stall midway
                                # through a valid bounded request frame.
                                sock.sendall(struct.pack(">I", 4 * 1024 * 1024) + bytes([4]))
                            else:
                                sock.sendall(struct.pack(">I", 1) + bytes([4]))
                                size = struct.unpack(">I", receive(sock, 4))[0]
                                receive(sock, size)
                        if kind == "partial_multi_ip":
                            sock.sendall(struct.pack(">I", 73) + bytes([1]))
                        if kind == "saturated_trickle_after_hello":
                            trickle_clients.append(sock)
                    # Give all sessions time to be admitted. The full-capacity
                    # scenario tests reclamation only after the idle grace period.
                    if kind == "saturated_trickle_after_hello":
                        # Keep every incomplete request "recently active" with
                        # single-byte writes. Before the fairness fix, these 32
                        # sessions monopolize admission until the 10 s deadline.
                        deadline = time.monotonic() + 2.3
                        while time.monotonic() < deadline:
                            for sock in trickle_clients:
                                sock.sendall(b"\\0")
                            time.sleep(0.15)
                    else:
                        time.sleep(2.1 if kind == "saturated_after_hello" else 0.15)
                    latency = wallet_probe(port)
                    if maximum is not None and latency >= maximum:
                        raise AssertionError(f"{kind}: wallet wait {latency:.3f}s >= {maximum}s")
                    started = time.monotonic()
                    sync = subprocess.run(
                        [str(binary), "testnet", "sync", "--data-dir", str(root / "peer"),
                         "--peer", f"127.0.0.1:{port}"],
                        capture_output=True, text=True, timeout=15,
                    )
                    sync_seconds = time.monotonic() - started
                    if sync.returncode or "height=0" not in sync.stdout:
                        raise AssertionError(f"sync failed: {sync.stdout}{sync.stderr}")
                # Closed hostile sockets must release leases and capacity.
                time.sleep(0.1)
                recovered = wallet_probe(port)
                return {"case": kind, "idle_seconds": idle, "wallet_seconds": latency,
                        "sync_seconds": sync_seconds, "recovered_seconds": recovered,
                        "connections": count}
            finally:
                server.terminate()
                try:
                    server.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    server.kill()
                    server.wait(timeout=5)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--max-wallet-seconds", type=float)
    parser.add_argument("--post-handshake", action="store_true")
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    results = []
    try:
        cases = ["same_ip", "silent_multi_ip", "partial_multi_ip"]
        if args.post_handshake:
            cases += ["idle_after_hello", "partial_after_hello",
                      "saturated_after_hello", "saturated_trickle_after_hello"]
        for kind in cases:
            result = run_case(binary, kind, args.max_wallet_seconds)
            results.append(result)
            print(json.dumps(result), flush=True)
    finally:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(results, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()

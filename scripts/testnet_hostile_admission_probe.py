#!/usr/bin/env python3
"""C1: bounded, authorized loopback-only adversarial P2P admission probe."""
from __future__ import annotations

import argparse
import json
import os
import re
import socket
import struct
import subprocess
import tempfile
import time
from contextlib import ExitStack
from pathlib import Path

from testnet_admission_probe import receive, wallet_probe


def send_frame(sock: socket.socket, payload: bytes) -> None:
    sock.sendall(struct.pack(">I", len(payload)) + payload)


def resident_bytes(pid: int) -> int:
    status = Path(f"/proc/{pid}/status").read_text(encoding="utf-8")
    matched = re.search(r"^VmRSS:\s+(\d+)\s+kB", status, re.MULTILINE)
    if matched is None:
        raise RuntimeError("VmRSS absent from Linux /proc status")
    return int(matched.group(1)) * 1024


def cpu_seconds(pid: int) -> float:
    # /proc/[pid]/stat has a command in parentheses; split after the final ')'.
    fields = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
    return (int(fields[11]) + int(fields[12])) / os.sysconf("SC_CLK_TCK")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--max-wallet-seconds", type=float, default=1.5)
    parser.add_argument("--max-rss-mib", type=int, default=512)
    parser.add_argument("--max-cpu-seconds", type=float, default=20.0)
    args = parser.parse_args()
    if args.max_wallet_seconds <= 0 or args.max_rss_mib <= 0 or args.max_cpu_seconds <= 0:
        parser.error("all bounds must be positive")
    if os.name != "posix" or not Path("/proc/self/status").exists():
        parser.error("this measured regression probe requires Linux /proc")

    binary = args.binary.resolve(strict=True)
    report = {
        "status": "failed",
        "topology": "one node + synthetic clients on loopback only",
        "git_sha": os.getenv("GITHUB_SHA", "unavailable"),
        "run_id": os.getenv("GITHUB_RUN_ID", "local"),
        "limits": {"wallet_seconds": args.max_wallet_seconds,
                   "rss_mib": args.max_rss_mib,
                   "cpu_seconds": args.max_cpu_seconds},
        "cases": [],
    }
    args.report.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="aretovus-c1-loopback-") as temporary:
        root = Path(temporary)
        output_path = root / "server.log"
        with output_path.open("wb") as stream:
            server = subprocess.Popen([
                str(binary), "testnet", "serve", "--data-dir", str(root / "node"),
                "--listen", "127.0.0.1:0",
            ], stdout=stream, stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic() + 15
                while True:
                    match = re.search(r"listening on 127\.0\.0\.1:(\d+)", output_path.read_text(errors="replace"))
                    if match:
                        port = int(match.group(1))
                        break
                    if server.poll() is not None or time.monotonic() >= deadline:
                        raise RuntimeError("server did not start:\n" + output_path.read_text(errors="replace"))
                    time.sleep(0.02)

                rss_peak = resident_bytes(server.pid)
                cpu_before = cpu_seconds(server.pid)

                def sample(label: str) -> None:
                    nonlocal rss_peak
                    if server.poll() is not None:
                        raise AssertionError(f"node died during {label}")
                    rss_peak = max(rss_peak, resident_bytes(server.pid))

                def wallet(label: str) -> None:
                    elapsed = wallet_probe(port)
                    report["cases"].append({"case": label, "wallet_seconds": elapsed})
                    if elapsed >= args.max_wallet_seconds:
                        raise AssertionError(f"{label}: wallet latency {elapsed:.3f}s exceeds bound")
                    sample(label)

                wallet("baseline")

                # Invalid frame lengths, inconsistent handshake and partial headers
                # must not terminate the server or monopolize admission workers.
                malformed = [
                    b"\x00\x00\x00\x00",
                    struct.pack(">I", 4 * 1024 * 1024 + 1),
                    struct.pack(">I", 73) + bytes([1]) + b"\x00" * 72,
                    struct.pack(">I", 73) + bytes([5]) + b"\x00" * 72,
                    struct.pack(">I", 1) + b"\xff",
                    b"\x00\x00",
                ]
                for index, payload in enumerate(malformed):
                    with socket.create_connection(("127.0.0.1", port), timeout=3) as conn:
                        conn.settimeout(3)
                        conn.sendall(payload)
                    sample(f"malformed-{index}")
                wallet("after_malformed_handshakes")

                # A legitimate peer can reuse a session; 64 bounded requests
                # must not cause an unbounded growth or unexpectedly drop it.
                with socket.create_connection(("127.0.0.1", port), timeout=3) as conn:
                    conn.settimeout(4)
                    send_frame(conn, b"\x05")
                    length = struct.unpack(">I", receive(conn, 4))[0]
                    hello = receive(conn, length)
                    if length != 74 or hello[:2] != b"\x85\x02":
                        raise AssertionError("unexpected wallet handshake")
                    for _ in range(64):
                        send_frame(conn, b"\x04")
                        length = struct.unpack(">I", receive(conn, 4))[0]
                        reply = receive(conn, length)
                        if length > 1024 or not reply or reply[0] != 0x84:
                            raise AssertionError("malformed peer-list response")
                wallet("after_repeated_requests")

                # Saturation with 8 distinct loopback IPs, four per IP.
                # Each socket completes HELLO then sends only the beginning
                # of a declared 4 MiB frame; the legitimate wallet must survive.
                with ExitStack() as stack:
                    for index in range(32):
                        conn = stack.enter_context(socket.socket())
                        conn.settimeout(4)
                        conn.bind((f"127.0.0.{2 + index // 4}", 0))
                        conn.connect(("127.0.0.1", port))
                        send_frame(conn, b"\x05")
                        length = struct.unpack(">I", receive(conn, 4))[0]
                        reply = receive(conn, length)
                        if length != 74 or reply[:2] != b"\x85\x02":
                            raise AssertionError("saturation setup failed")
                        conn.sendall(struct.pack(">I", 4 * 1024 * 1024) + b"\x04")
                    # At full capacity, reclaim slow/incomplete sessions,
                    # not a completed, queued, or in-flight request.
                    time.sleep(2.3)
                    sample("saturated_after_hello")
                    wallet("wallet_under_32_slow_sessions")
                time.sleep(0.15)
                wallet("wallet_after_release")

                sync = subprocess.run([
                    str(binary), "testnet", "sync", "--data-dir", str(root / "fresh"),
                    "--peer", f"127.0.0.1:{port}",
                ], text=True, capture_output=True, timeout=20)
                if sync.returncode != 0 or "height=0" not in sync.stdout:
                    raise AssertionError(f"sync recovery failed: {sync.stdout} {sync.stderr}")
                sample("fresh_peer_sync")
                report["cases"].append({"case": "fresh_peer_sync", "passed": True})

                cpu_used = cpu_seconds(server.pid) - cpu_before
                report["rss_peak_bytes"] = rss_peak
                report["cpu_seconds"] = round(cpu_used, 3)
                if rss_peak > args.max_rss_mib * 1024 * 1024:
                    raise AssertionError("node RSS exceeded configured limit")
                if cpu_used > args.max_cpu_seconds:
                    raise AssertionError("node CPU time exceeded configured limit")
                report["status"] = "success"
                print(json.dumps(report), flush=True)
            except Exception as error:
                report["error"] = f"{type(error).__name__}: {error}"
                raise
            finally:
                server.terminate()
                try:
                    server.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    server.kill()
                    server.wait(timeout=5)
                args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()

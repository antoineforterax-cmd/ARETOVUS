#!/usr/bin/env python3
"""ARETOVUS testnet LAN assistant: offline verified, explicit same-LAN commands."""
from __future__ import annotations

import argparse
import hashlib
import ipaddress
import os
from pathlib import Path
import re
import subprocess
import sys

PACKAGE_ROOT = Path(__file__).resolve().parents[1]
NODE_PORT = 19444
CLIENT_PORT = 19445
REVISION = re.compile(r"(?m)^Source commit: ([0-9a-f]{12})\r?$")


def private_lan_ip(value: str) -> str:
    """RFC1918 IPv4 only; reject public, loopback, multicast, and zone IDs."""
    address = ipaddress.IPv4Address(value)
    ranges = ("10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16")
    if not any(address in ipaddress.IPv4Network(item) for item in ranges):
        raise ValueError("Use an IPv4 address from a private LAN (RFC1918)")
    return str(address)


def node_binary(root: Path) -> Path:
    suffix = ".exe" if os.name == "nt" else ""
    return root / "bin" / f"aretovus-node{suffix}"


def inspect_package(root: Path) -> tuple[str, Path]:
    """Checks extracted local file integrity; this does NOT authenticate a publisher."""
    stamp = (root / "EXPERIMENTAL_TESTNET.txt").read_text(encoding="utf-8")
    match = REVISION.search(stamp)
    if match is None:
        raise ValueError("Invalid or missing package source revision")
    checksums = (root / "SHA256SUMS.txt").read_text(encoding="utf-8").splitlines()
    digests: dict[str, str] = {}
    for line in checksums:
        if not re.fullmatch(r"[0-9a-f]{64}  [A-Za-z0-9_./-]+", line):
            raise ValueError("Invalid SHA256SUMS entry")
        digest, relative = line.split("  ", 1)
        path = Path(relative)
        if (path.is_absolute() or ".." in path.parts or relative in digests):
            raise ValueError("Unsafe or duplicate checksum path")
        source = root / path
        current = root
        for component in path.parts:
            current = current / component
            # Reject symlinked files and intermediate directories, not just
            # the final node or wallet binary path.
            if current.is_symlink():
                raise ValueError(f"Symbolic link in package path: {relative}")
        if not source.is_file():
            raise ValueError(f"Missing or invalid package file: {relative}")
        if hashlib.sha256(source.read_bytes()).hexdigest() != digest:
            raise ValueError(f"Package integrity failure: {relative}")
        digests[relative] = digest
    suffix = ".exe" if os.name == "nt" else ""
    for component in ("node", "wallet"):
        filename = f"bin/aretovus-{component}{suffix}"
        if filename not in digests:
            raise ValueError(f"Missing critical binary in manifest: {filename}")
    executable = node_binary(root)
    if not executable.is_file() or executable.is_symlink():
        raise ValueError("Invalid executable")
    return match.group(1), executable


def command_for(role: str, local_ip: str, peer_ip: str | None, root: Path) -> list[str]:
    local_ip = private_lan_ip(local_ip)
    exe = str(node_binary(root))
    if role == "server":
        if peer_ip:
            raise ValueError("Server does not accept --peer-ip")
        return [exe, "testnet", "serve", "--data-dir",
                str(root / "two-pc" / "assistant-a"), "--listen",
                f"{local_ip}:{NODE_PORT}"]
    if role != "client":
        raise ValueError("Choose server or client")
    if peer_ip is None:
        raise ValueError("Client requires --peer-ip")
    peer_ip = private_lan_ip(peer_ip)
    if peer_ip == local_ip:
        raise ValueError("Peer and local IP must be distinct")
    return [exe, "testnet", "serve", "--data-dir",
            str(root / "two-pc" / "assistant-b"), "--listen",
            f"127.0.0.1:{CLIENT_PORT}", "--peer", f"{peer_ip}:{NODE_PORT}"]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--role", choices=("server", "client"))
    parser.add_argument("--local-ip")
    parser.add_argument("--peer-ip")
    parser.add_argument("--verify-package", action="store_true")
    parser.add_argument("--run", action="store_true",
                        help="Start an explicitly selected local testnet node")
    args = parser.parse_args()
    try:
        revision, exe = inspect_package(PACKAGE_ROOT)
        print(f"ARETOVUS experimental testnet; source revision: {revision}")
        print("Local SHA256 checks pass. Publisher authenticity is NOT established.")
        if args.verify_package and not args.role:
            return 0
        if not args.role or not args.local_ip:
            parser.error("select --role and --local-ip (or --verify-package)")
        command = command_for(args.role, args.local_ip, args.peer_ip, PACKAGE_ROOT)
        print(f"Role={args.role}, data folder={command[4]}, network restricted to private LAN")
        if not args.run:
            print("Plan verified; use --run to start the node, Ctrl+C to stop.")
            return 0
        if os.name != "nt":
            exe.chmod(exe.stat().st_mode | 0o100)
        print("Starting testnet node; no wallet keys or port forwarding.", flush=True)
        return subprocess.call(command, cwd=PACKAGE_ROOT)
    except (ValueError, FileNotFoundError, OSError) as exc:
        print(f"ARETOVUS LAN preparation refused: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())

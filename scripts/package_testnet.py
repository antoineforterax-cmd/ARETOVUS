#!/usr/bin/env python3
"""Package reproducible, unsigned AURORA experimental testnet binaries."""
from __future__ import annotations

import hashlib
import os
import platform
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BIN_DIR = ROOT / "target" / "release"
DIST = ROOT / "dist"


def main() -> int:
    revision = subprocess.check_output(
        ["git", "rev-parse", "--short=12", "HEAD"], cwd=ROOT, text=True
    ).strip()
    operating_system = {"win32": "windows", "darwin": "macos"}.get(
        sys.platform, "linux"
    )
    architecture = platform.machine().lower().replace("amd64", "x86_64")
    suffix = ".exe" if os.name == "nt" else ""
    files = [
        (BIN_DIR / f"aurora-node{suffix}", f"bin/aurora-node{suffix}"),
        (BIN_DIR / f"aurora-wallet{suffix}", f"bin/aurora-wallet{suffix}"),
        (ROOT / "README.md", "README.md"),
        (ROOT / "docs" / "TESTNET_GUIDE.md", "docs/TESTNET_GUIDE.md"),
        (ROOT / "docs" / "TESTNET_OPERATIONS.md", "docs/TESTNET_OPERATIONS.md"),
        (ROOT / "docs" / "PROTOCOL.md", "docs/PROTOCOL.md"),
    ]
    for launcher in ("AURORA-LAN.cmd", "AURORA-LAN.sh", "AURORA-LAN.command"):
        files.append((ROOT / launcher, launcher))
    for name in ("TWO_PC_TEST.md", "INTEGRATION_TWO_PC.md"):
        files.append((ROOT / "docs" / name, f"docs/{name}"))
    for name in ("LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "PROVENANCE.md", "SECURITY.md"):
        files.append((ROOT / name, name))
    for name in ("testnet_endurance.py", "testnet_smoke.py", "testnet_soak.py", "two_pc_preflight.ps1", "lan_assistant.ps1", "lan_assistant_unix.py"):
        files.append((ROOT / "scripts" / name, f"scripts/{name}"))
    files.append((ROOT / "docs" / "ENDURANCE_RUN.md", "docs/ENDURANCE_RUN.md"))
    for source, _ in files:
        if not source.is_file():
            raise FileNotFoundError(source)
    DIST.mkdir(exist_ok=True)
    package = DIST / f"aurora-testnet-{operating_system}-{architecture}-{revision}.zip"
    checksums = []
    with zipfile.ZipFile(package, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for source, destination in files:
            data = source.read_bytes()
            archive.write(source, arcname=destination)
            checksums.append(f"{hashlib.sha256(data).hexdigest()}  {destination}")
        archive.writestr(
            "SHA256SUMS.txt", "\n".join(checksums) + "\n"
        )
        archive.writestr(
            "EXPERIMENTAL_TESTNET.txt",
            "AURORA experimental testnet build.\n"
            f"Source commit: {revision}\n"
            "Unsigned CI artifact. No mainnet or real-funds security claim.\n"
            "Read docs/TESTNET_GUIDE.md before starting a node.\n",
        )
    with zipfile.ZipFile(package) as archive:
        if archive.testzip() is not None:
            raise RuntimeError("package failed ZIP integrity check")
        for line in archive.read("SHA256SUMS.txt").decode().splitlines():
            expected, name = line.split("  ", 1)
            if hashlib.sha256(archive.read(name)).hexdigest() != expected:
                raise RuntimeError(f"package hash mismatch: {name}")
    print(f"package={package} sha256={hashlib.sha256(package.read_bytes()).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

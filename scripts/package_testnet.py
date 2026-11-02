#!/usr/bin/env python3
"""Package reproducible, unsigned ARETOVUS experimental testnet binaries."""
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
        (BIN_DIR / f"aretovus-node{suffix}", f"bin/aretovus-node{suffix}"),
        (BIN_DIR / f"aretovus-wallet{suffix}", f"bin/aretovus-wallet{suffix}"),
        (ROOT / "README.md", "README.md"),
        (ROOT / "docs" / "TESTNET_GUIDE.md", "docs/TESTNET_GUIDE.md"),
        (ROOT / "docs" / "TESTNET_OPERATIONS.md", "docs/TESTNET_OPERATIONS.md"),
        (ROOT / "docs" / "PROTOCOL.md", "docs/PROTOCOL.md"),
    ]
    for name in ("TWO_PC_TEST.md", "INTEGRATION_TWO_PC.md"):
        files.append((ROOT / "docs" / name, f"docs/{name}"))
    for name in ("LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "PROVENANCE.md", "SECURITY.md"):
        files.append((ROOT / name, name))
    for name in ("testnet_endurance.py", "testnet_smoke.py", "testnet_soak.py", "two_pc_preflight.ps1"):
        files.append((ROOT / "scripts" / name, f"scripts/{name}"))
    files.append((ROOT / "docs" / "ENDURANCE_RUN.md", "docs/ENDURANCE_RUN.md"))
    for source, _ in files:
        if not source.is_file():
            raise FileNotFoundError(source)
    DIST.mkdir(exist_ok=True)
    package = DIST / f"aretovus-testnet-{operating_system}-{architecture}-{revision}.zip"
    # Fixed ZIP timestamps, creator OS, mode bits and member order make
    # archives independent of source mtimes and file-system permissions.
    # Cross-platform executables remain different builds by design.
    def archive_entry(name: str, data: bytes, *, executable: bool = False) -> zipfile.ZipInfo:
        info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
        info.compress_type = zipfile.ZIP_DEFLATED
        info.create_system = 3
        info.external_attr = ((0o100755 if executable else 0o100644) << 16)
        return info

    checksums = []
    with zipfile.ZipFile(package, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for source, destination in files:
            data = source.read_bytes()
            archive.writestr(
                archive_entry(destination, data, executable=destination.startswith("bin/")),
                data,
                compress_type=zipfile.ZIP_DEFLATED,
                compresslevel=9,
            )
            checksums.append(f"{hashlib.sha256(data).hexdigest()}  {destination}")
        archive.writestr(
            archive_entry("SHA256SUMS.txt", b""),
            "\n".join(checksums) + "\n",
        )
        archive.writestr(
            archive_entry("EXPERIMENTAL_TESTNET.txt", b""),
            "ARETOVUS experimental testnet build.\n"
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

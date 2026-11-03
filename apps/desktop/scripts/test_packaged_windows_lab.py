#!/usr/bin/env python3
"""Exercise the REAL executables from the verified and extracted Windows lab ZIP.

This is an ephemeral CI test, not a desktop GUI smoke test or a release.
"""
from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile

import package_windows_lab as lab


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", type=Path, required=True)
    args = parser.parse_args()
    archive_path = args.archive.resolve(strict=True)
    lab.verify_archive(archive_path)

    with tempfile.TemporaryDirectory(prefix="ARETOVUS extracted Windows lab with spaces ") as temp:
        root = Path(temp)
        with zipfile.ZipFile(archive_path) as archive:
            # The archive was already checked against a fixed, flat allowlist
            # of entry names. Never extract an unverified ZIP.
            archive.extractall(root)

        folder = root / lab.ROOT.rstrip("/")
        expected = set(lab.BINARIES) | {lab.MANIFEST, lab.SUMS, lab.GUIDE}
        if not folder.is_dir() or {entry.name for entry in folder.iterdir()} != expected:
            raise AssertionError("Portable package does not contain exactly expected files")

        binaries = {name: folder / name for name in lab.BINARIES}
        before = {name: sha256(path) for name, path in binaries.items()}
        test = Path(__file__).with_name("test_native_integration.py")
        result = subprocess.run(
            [
                sys.executable, str(test),
                "--node", str(binaries["aretovus-node.exe"]),
                "--bridge", str(binaries["aretovus-local-bridge.exe"]),
                "--explorer", str(binaries["aretovus-explorer.exe"]),
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=180,
            check=False,
        )
        if result.returncode != 0:
            # No secrets are supplied; cap captured logs before showing CI error.
            raise AssertionError(
                "Extracted testnet companion execution failed: "
                + result.stderr.decode("utf-8", errors="replace")[-1800:]
            )
        after = {name: sha256(path) for name, path in binaries.items()}
        if before != after:
            raise AssertionError("Companion changed while running disposable testnet integration")
        if {entry.name for entry in folder.iterdir()} != expected:
            raise AssertionError("Companion wrote unexpected files into its install directory")

    print("Extracted ZIP companions passed real disposable-chain test, with no install-dir mutation")
    print("GUI remains untested interactively; no signature, release, wallet or mainnet claim")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

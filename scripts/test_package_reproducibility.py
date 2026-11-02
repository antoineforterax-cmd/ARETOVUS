#!/usr/bin/env python3
"""Deterministic package metadata test without compiling Rust or publishing a release."""
from __future__ import annotations

import hashlib
import os
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

import package_testnet


class DeterministicPackageTest(unittest.TestCase):
    def test_repacking_unchanged_bytes_after_mtime_changes_is_identical(self) -> None:
        original_bin, original_dist = package_testnet.BIN_DIR, package_testnet.DIST
        with tempfile.TemporaryDirectory(prefix="aretovus-package-test-") as temp:
            root = Path(temp)
            binary_directory = root / "bin"
            binary_directory.mkdir()
            suffix = ".exe" if os.name == "nt" else ""
            binary_names = [f"aretovus-node{suffix}", f"aretovus-wallet{suffix}"]
            for name in binary_names:
                (binary_directory / name).write_bytes(b"synthetic test binary " + name.encode())
            package_testnet.BIN_DIR = binary_directory
            package_testnet.DIST = root / "dist"
            try:
                self.assertEqual(package_testnet.main(), 0)
                packages = sorted(package_testnet.DIST.glob("*.zip"))
                self.assertEqual(len(packages), 1)
                one = packages[0].read_bytes()
                # File modification times must not influence the ZIP digest.
                for index, name in enumerate(binary_names):
                    os.utime(binary_directory / name, (1400000000 + index, 1700000000 + index))
                self.assertEqual(package_testnet.main(), 0)
                two = packages[0].read_bytes()
                self.assertEqual(hashlib.sha256(one).hexdigest(), hashlib.sha256(two).hexdigest())
                with zipfile.ZipFile(packages[0]) as archive:
                    self.assertIsNone(archive.testzip())
                    self.assertEqual(len(archive.namelist()), len(set(archive.namelist())))
                    for info in archive.infolist():
                        self.assertEqual(info.date_time, (1980, 1, 1, 0, 0, 0))
                        self.assertEqual(info.create_system, 3)
                        expected_mode = 0o100755 if info.filename.startswith("bin/") else 0o100644
                        self.assertEqual(info.external_attr >> 16, expected_mode)
                    for line in archive.read("SHA256SUMS.txt").decode().splitlines():
                        digest, name = line.split("  ", 1)
                        self.assertEqual(digest, hashlib.sha256(archive.read(name)).hexdigest())
                    self.assertIn(b"Unsigned CI artifact", archive.read("EXPERIMENTAL_TESTNET.txt"))
            finally:
                package_testnet.BIN_DIR = original_bin
                package_testnet.DIST = original_dist


if __name__ == "__main__":
    unittest.main()

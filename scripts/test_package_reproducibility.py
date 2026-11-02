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
import lan_pair


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
                    self.assertIn("scripts/lan_pair.py", archive.namelist())
                # Exercise offline LAN helper against a real extracted package,
                # including its synthetic node files and path with spaces.
                extract_root = root / "ARETOVUS package with spaces"
                with zipfile.ZipFile(packages[0]) as archive:
                    archive.extractall(extract_root)
                revision, node = lan_pair.inspect_package(extract_root)
                self.assertEqual(len(revision), 12)
                self.assertTrue(node.is_file())
                server = lan_pair.command_for("server", "192.168.1.11", None, extract_root)
                client = lan_pair.command_for(
                    "client", "192.168.1.12", "192.168.1.11", extract_root
                )
                self.assertIn("assistant-a", server[4])
                self.assertIn("assistant-b", client[4])
                with self.assertRaises(ValueError):
                    lan_pair.command_for("client", "192.168.1.12", "8.8.8.8", extract_root)
                with self.assertRaises(ValueError):
                    lan_pair.command_for("client", "192.168.1.12", "192.168.1.12", extract_root)
                node.write_bytes(node.read_bytes() + b"tampered")
                with self.assertRaises(ValueError):
                    lan_pair.inspect_package(extract_root)
            finally:
                package_testnet.BIN_DIR = original_bin
                package_testnet.DIST = original_dist

    def test_symlinked_ancestor_of_binary_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory(prefix="aretovus-symlink-package-") as temp:
            root = Path(temp)
            trusted = root / "original-bin"
            trusted.mkdir()
            suffix = ".exe" if os.name == "nt" else ""
            checksums = []
            for component in ("node", "wallet"):
                relative = f"bin/aretovus-{component}{suffix}"
                data = f"synthetic-{component}".encode()
                (trusted / f"aretovus-{component}{suffix}").write_bytes(data)
                checksums.append(hashlib.sha256(data).hexdigest() + "  " + relative)
            (root / "SHA256SUMS.txt").write_text(
                "\n".join(checksums) + "\n", encoding="utf-8"
            )
            (root / "EXPERIMENTAL_TESTNET.txt").write_text(
                "Source commit: abcdef123456\n", encoding="utf-8"
            )
            try:
                (root / "bin").symlink_to(trusted, target_is_directory=True)
            except OSError:
                self.skipTest("directory symlinks unavailable without privileges")
            with self.assertRaisesRegex(ValueError, "Symbolic link in package path"):
                lan_pair.inspect_package(root)


if __name__ == "__main__":
    unittest.main()

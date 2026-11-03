#!/usr/bin/env python3
"""Adversarial tests for unsigned Windows lab ZIP construction (no executable runs)."""
from __future__ import annotations

import contextlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile

import package_windows_lab as lab

SHA = "a" * 40


class PortableLabTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="ARETOVUS safe ZIP test with spaces ")
        self.root = Path(self.temp.name)
        self.binaries = {}
        for name in lab.BINARIES:
            path = self.root / name
            # Minimal PE magic fixture for PACKAGING validation only. It is
            # explicitly not an executable or a demonstration of a running app.
            # Synthetic x64 PE header for archive validation only. The bytes
            # are not executable and do not substitute a real Windows build.
            image = bytearray(0x80 + 26)
            image[:2] = b"MZ"
            image[0x3C:0x40] = (0x80).to_bytes(4, "little")
            image[0x80:0x84] = b"PE\\0\\0"
            image[0x84:0x86] = (0x8664).to_bytes(2, "little")
            image[0x80 + 24:0x80 + 26] = (0x20B).to_bytes(2, "little")
            path.write_bytes(bytes(image) + (name * 8).encode())
            self.binaries[name] = path

    def tearDown(self) -> None:
        self.temp.cleanup()

    def pack(self, output: Path, *, sha=SHA) -> None:
        class Args:
            pass
        args = Args()
        args.desktop = self.binaries[lab.BINARIES[0]]
        args.bridge = self.binaries[lab.BINARIES[1]]
        args.explorer = self.binaries[lab.BINARIES[2]]
        args.node = self.binaries[lab.BINARIES[3]]
        args.source_sha = sha
        args.output = output
        lab.create_archive(args)

    def silent(self, fn) -> None:
        with contextlib.redirect_stdout(io.StringIO()):
            fn()

    def test_valid_archive_has_exact_side_by_side_manifest_and_deterministic_bytes(self):
        first = self.root / "portable-one.zip"
        second = self.root / "portable-two.zip"
        self.silent(lambda: self.pack(first))
        self.silent(lambda: self.pack(second))
        self.assertEqual(first.read_bytes(), second.read_bytes())
        with zipfile.ZipFile(first) as zipped:
            self.assertEqual(
                set(zipped.namelist()),
                {lab.ROOT + n for n in (*lab.BINARIES, lab.MANIFEST, lab.SUMS, lab.GUIDE)},
            )
            self.assertIn(b"NOT a", zipped.read(lab.ROOT + lab.GUIDE))
            self.assertNotIn("wallet", zipped.namelist())
        self.silent(lambda: lab.verify_archive(first))

    def test_rejects_non_windows_symlink_and_missing_companion_without_output(self):
        out = self.root / "invalid.zip"
        self.binaries[lab.BINARIES[1]].write_bytes(b"ELF" * 20)
        with self.assertRaisesRegex(ValueError, "Windows PE"):
            self.pack(out)
        self.assertFalse(out.exists())
        self.binaries[lab.BINARIES[1]].unlink()
        with self.assertRaisesRegex(ValueError, "Missing"):
            self.pack(out)
        self.assertFalse(out.exists())

    def test_mz_only_fake_windows_executable_is_rejected(self):
        target = self.binaries[lab.BINARIES[0]]
        target.write_bytes(b"MZ" + b"fake binary" * 12)
        with self.assertRaisesRegex(ValueError, "PE|Windows"):
            self.pack(self.root / "fake-mz.zip")

    def test_rejects_windows_pe_of_wrong_architecture(self):
        target = self.binaries[lab.BINARIES[1]]
        with target.open("r+b") as file:
            file.seek(0x84)
            file.write((0x14C).to_bytes(2, "little"))  # i386, not AMD64
        with self.assertRaisesRegex(ValueError, "AMD64|architecture"):
            self.pack(self.root / "wrong-arch.zip")

    def test_rejects_invalid_source_commit_or_overwriting_existing_archive(self):
        out = self.root / "invalid.zip"
        with self.assertRaisesRegex(ValueError, "source SHA"):
            self.pack(out, sha="short")
        self.assertFalse(out.exists())
        self.silent(lambda: self.pack(out))
        with self.assertRaisesRegex(ValueError, "replace"):
            self.pack(out)

    def test_racing_output_creator_is_never_deleted(self):
        out = self.root / "raced-by-another-process.zip"
        other_owner_data = b"other process owns this content"

        def racing_creator(path, mode, **kwargs):
            self.assertEqual(mode, "x")
            self.assertEqual(Path(path), out)
            out.write_bytes(other_owner_data)
            raise FileExistsError("competing process won exclusive create")

        with patch.object(lab.zipfile, "ZipFile", side_effect=racing_creator):
            with self.assertRaises(FileExistsError):
                self.pack(out)
        self.assertEqual(out.read_bytes(), other_owner_data)

    def test_tampered_companion_is_rejected(self):
        out = self.root / "original.zip"
        self.silent(lambda: self.pack(out))
        bad = self.root / "tampered.zip"
        with zipfile.ZipFile(out) as source, zipfile.ZipFile(bad, "w") as dest:
            for item in source.infolist():
                data = source.read(item.filename)
                if item.filename.endswith(lab.BINARIES[2]):
                    data += b"unexpected bytes"
                dest.writestr(item, data)
        with self.assertRaisesRegex(ValueError, "checksum mismatch"):
            lab.verify_archive(bad)

    def test_tampered_safety_guide_or_manifest_injection_is_rejected(self):
        original = self.root / "good.zip"
        self.silent(lambda: self.pack(original))
        for target, suffix in ((lab.GUIDE, "guide"), (lab.MANIFEST, "metadata")):
            bad = self.root / (suffix + "-tampered.zip")
            with zipfile.ZipFile(original) as src, zipfile.ZipFile(bad, "w") as dst:
                for item in src.infolist():
                    data = src.read(item.filename)
                    if item.filename == lab.ROOT + target:
                        if target == lab.GUIDE:
                            data = data.replace(b"DO NOT disable", b"PLEASE disable")
                        else:
                            import json
                            content = json.loads(data)
                            content["untrusted_extra"] = "unexpected"
                            data = json.dumps(content).encode()
                    dst.writestr(item, data)
            with self.assertRaisesRegex(ValueError, "Guide checksum|Unexpected metadata"):
                lab.verify_archive(bad)

    def test_extra_and_duplicate_entries_are_rejected(self):
        out = self.root / "good.zip"
        self.silent(lambda: self.pack(out))
        for injected in ("../../outside.exe", lab.ROOT + lab.BINARIES[0]):
            attack = self.root / ("bad-" + str(len(injected)) + ".zip")
            with zipfile.ZipFile(out) as source, zipfile.ZipFile(attack, "w") as dest:
                for member in source.infolist():
                    dest.writestr(member, source.read(member.filename))
                with contextlib.redirect_stderr(io.StringIO()):
                    dest.writestr(injected, b"MZdanger")
            with self.assertRaisesRegex(ValueError, "Unexpected, duplicate"):
                lab.verify_archive(attack)

    def test_maximum_binary_size_is_bounded_before_opening(self):
        binary = self.binaries[lab.BINARIES[0]]
        with binary.open("wb") as handle:
            handle.truncate(lab.MAX_EXE + 1)
        with self.assertRaisesRegex(ValueError, "size"):
            self.pack(self.root / "huge.zip")

    def test_only_testnet_unsigned_manifest_passes(self):
        original = self.root / "good.zip"
        self.silent(lambda: self.pack(original))
        for key, value in (("signed", True), ("network", "mainnet"), ("release_approved", True)):
            bad = self.root / f"{key}.zip"
            with zipfile.ZipFile(original) as src, zipfile.ZipFile(bad, "w") as dst:
                for item in src.infolist():
                    data = src.read(item.filename)
                    if item.filename == lab.ROOT + lab.MANIFEST:
                        import json
                        obj = json.loads(data)
                        obj[key] = value
                        data = json.dumps(obj).encode()
                    dst.writestr(item, data)
            with self.assertRaisesRegex(ValueError, "Invalid lab metadata"):
                lab.verify_archive(bad)


if __name__ == "__main__":
    unittest.main()

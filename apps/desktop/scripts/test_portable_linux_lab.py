#!/usr/bin/env python3
"""Exercise the Linux lab archive contract without running fixture binaries."""
from __future__ import annotations

import gzip
import os
from pathlib import Path
import stat
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import package_linux_lab as lab

SHA = "a" * 40


class PortableLinuxLabTests(unittest.TestCase):
    def setUp(self):
        folder = tempfile.TemporaryDirectory(prefix="ARETOVUS Linux lab tests ")
        self.addCleanup(folder.cleanup)
        self.root = Path(folder.name)
        self.inputs = {}
        for name in lab.BINARIES:
            path = self.root / name
            ident = b"\x7fELF\x02\x01\x01" + bytes(9)
            header = lab.ELF_HEADER.pack(ident, 3, 62, 1, 128, 64, 0, 0, 64, 56, 1, 0, 0, 0)
            path.write_bytes(header + bytes(56) + name.encode())
            self.inputs[name] = path
        self.original = self.root / "valid.tar.gz"
        lab.pack_archive(self.inputs, SHA, self.original)

    def rewrite(self, tar_bytes: bytes) -> Path:
        altered = self.root / "altered.tar.gz"
        with altered.open("wb") as output:
            with gzip.GzipFile(filename="", fileobj=output, mode="wb", mtime=0) as stream:
                stream.write(tar_bytes)
        return altered

    def edit_header(self, target: str, edit) -> Path:
        with tarfile.open(self.original) as archive:
            offset = archive.getmember(target).offset
        data = bytearray(gzip.decompress(self.original.read_bytes()))
        info = tarfile.TarInfo.frombuf(bytes(data[offset:offset + 512]), "ascii", "strict")
        edit(info)
        data[offset:offset + 512] = info.tobuf(format=tarfile.USTAR_FORMAT)
        return self.rewrite(data)

    def test_round_trip_modes_and_determinism(self):
        second = self.root / "second.tar.gz"
        self.inputs[lab.BINARIES[0]].chmod(0o600)
        os.utime(self.inputs[lab.BINARIES[1]], (123, 123))
        lab.pack_archive(self.inputs, SHA, second)
        self.assertEqual(self.original.read_bytes(), second.read_bytes())
        with tarfile.open(self.original) as archive:
            self.assertEqual(archive.getnames(), [lab.MANIFEST, lab.SUMS, *lab.PAYLOADS])
        destination = self.root / "extract with spaces"
        lab.verify_archive(self.original, SHA, destination)
        self.assertEqual(stat.S_IMODE(destination.stat().st_mode), 0o700)
        for name in lab.BINARIES:
            item = destination / name
            self.assertEqual(item.read_bytes(), self.inputs[name].read_bytes())
            self.assertEqual(stat.S_IMODE(item.stat().st_mode), 0o755)
        for name in (lab.MANIFEST, lab.SUMS, lab.GUIDE, *lab.NOTICES):
            self.assertEqual(stat.S_IMODE((destination / name).stat().st_mode), 0o644)
        for name in lab.NOTICES:
            self.assertEqual((destination / name).read_bytes(), (lab.SOURCE_ROOT / name).read_bytes())

    def test_wrong_sha_or_existing_destination_is_rejected(self):
        destination = self.root / "destination"
        with self.assertRaisesRegex(ValueError, "source SHA"):
            lab.verify_archive(self.original, "b" * 40, destination)
        self.assertFalse(destination.exists())
        destination.mkdir()
        sentinel = destination / "do not touch"
        sentinel.write_bytes(b"existing data")
        with self.assertRaises(FileExistsError):
            lab.verify_archive(self.original, SHA, destination)
        self.assertEqual(list(destination.iterdir()), [sentinel])
        self.assertEqual(sentinel.read_bytes(), b"existing data")

    def test_invalid_architecture_and_oversize_reject_before_publication(self):
        target = self.inputs[lab.BINARIES[0]]
        wrong = bytearray(target.read_bytes())
        wrong[18:20] = (183).to_bytes(2, "little")
        target.write_bytes(wrong)
        output = self.root / "wrong.tar.gz"
        with self.assertRaisesRegex(ValueError, "x86-64"):
            lab.pack_archive(self.inputs, SHA, output)
        with target.open("wb") as source:
            source.truncate(lab.MAX_EXE + 1)
        with self.assertRaisesRegex(ValueError, "size"):
            lab.pack_archive(self.inputs, SHA, output)
        self.assertFalse(output.exists())

    def test_symlinked_and_non_regular_inputs_are_rejected(self):
        target = self.inputs[lab.BINARIES[0]]
        target.unlink()
        target.symlink_to(self.inputs[lab.BINARIES[1]])
        output = self.root / "invalid.tar.gz"
        with self.assertRaises(OSError):
            lab.pack_archive(self.inputs, SHA, output)
        target.unlink()
        os.mkfifo(target)
        with self.assertRaisesRegex(ValueError, "regular file"):
            lab.pack_archive(self.inputs, SHA, output)
        self.assertFalse(output.exists())

    def test_output_race_does_not_replace_other_file(self):
        output = self.root / "race.tar.gz"
        real_link = os.link

        def competitor(source, target):
            Path(target).write_bytes(b"other process won")
            return real_link(source, target)

        with patch.object(lab.os, "link", side_effect=competitor):
            with self.assertRaises(FileExistsError):
                lab.pack_archive(self.inputs, SHA, output)
        self.assertEqual(output.read_bytes(), b"other process won")
        self.assertEqual(list(self.root.glob(".aretovus-pack-*")), [])

    def test_failed_extraction_does_not_leave_partial_output(self):
        destination = self.root / "destination"
        with patch.object(lab, "publish_directory", side_effect=OSError("simulated disk error")):
            with self.assertRaisesRegex(OSError, "disk error"):
                lab.verify_archive(self.original, SHA, destination)
        self.assertFalse(destination.exists())
        self.assertEqual(list(self.root.glob(".aretovus-verify-*")), [])

    def test_extraction_race_preserves_empty_destination_and_cleans_staging(self):
        destination = self.root / "destination"
        real_publish = lab.publish_directory

        def competitor(staging, target):
            target.mkdir(mode=0o755)
            target.chmod(0o755)
            real_publish(staging, target)

        with patch.object(lab, "publish_directory", side_effect=competitor):
            with self.assertRaises(FileExistsError):
                lab.verify_archive(self.original, SHA, destination)
        self.assertEqual(list(destination.iterdir()), [])
        self.assertEqual(stat.S_IMODE(destination.stat().st_mode), 0o755)
        self.assertEqual(list(self.root.glob(".aretovus-verify-*")), [])

    def test_tampered_binary_and_manifest_leave_no_extraction_target(self):
        data = gzip.decompress(self.original.read_bytes())
        with tarfile.open(self.original) as archive:
            manifest_offset = archive.getmember(lab.MANIFEST).offset_data
            node = archive.getmember(lab.BINARIES[-1])
        for offset in (manifest_offset, node.offset_data + node.size - 1):
            with self.subTest(offset=offset):
                altered = bytearray(data)
                altered[offset] ^= 1
                destination = self.root / "target"
                with self.assertRaisesRegex(ValueError, "manifest|Checksum"):
                    lab.verify_archive(self.rewrite(altered), SHA, destination)
                self.assertFalse(destination.exists())

    def test_traversal_duplicate_links_permissions_and_pax_rejected(self):
        target = lab.BINARIES[0]
        for key, value in (
            ("name", "../escaped"),
            ("name", "/escaped"),
            ("name", lab.MANIFEST),
            ("mode", 0o777),
            ("mode", 0o4755),
            ("type", tarfile.SYMTYPE),
            ("type", tarfile.LNKTYPE),
            ("type", tarfile.XHDTYPE),
            ("type", tarfile.GNUTYPE_LONGNAME),
            ("size", lab.MAX_EXE + 1),
        ):
            with self.subTest(key=key, value=value):
                malicious = self.edit_header(target, lambda entry: setattr(entry, key, value))
                with self.assertRaisesRegex(ValueError, "Invalid USTAR header"):
                    lab.verify_archive(malicious, SHA)

    def test_truncated_extra_and_concatenated_streams_rejected(self):
        original = self.original.read_bytes()
        raw = gzip.decompress(original)
        for content in (original[:-2], original + b"\0", original + gzip.compress(b"")):
            with self.subTest(ending=content[-12:]):
                altered = self.root / "broken.tar.gz"
                altered.write_bytes(content)
                with self.assertRaises(ValueError):
                    lab.verify_archive(altered, SHA)
        with self.assertRaisesRegex(ValueError, "Unexpected data"):
            lab.verify_archive(self.rewrite(raw + b"trailing"), SHA)
        with self.assertRaisesRegex(ValueError, "padding"):
            lab.verify_archive(self.rewrite(raw[:-1] + b"x"), SHA)

    def test_rejects_optional_gzip_fields_and_reads_binary_across_chunks(self):
        original = self.original.read_bytes()
        named = self.root / "named.tar.gz"
        named.write_bytes(original[:3] + b"\x08" + original[4:10] + b"archive\0" + original[10:])
        with self.assertRaisesRegex(ValueError, "gzip header"):
            lab.verify_archive(named, SHA)

        with self.inputs[lab.BINARIES[0]].open("ab") as target:
            target.write(os.urandom(2 * lab.CHUNK))
        large = self.root / "large.tar.gz"
        lab.pack_archive(self.inputs, SHA, large)
        destination = self.root / "target"
        lab.verify_archive(large, SHA, destination)
        self.assertEqual(
            (destination / lab.BINARIES[0]).read_bytes(), self.inputs[lab.BINARIES[0]].read_bytes()
        )


if __name__ == "__main__":
    unittest.main()

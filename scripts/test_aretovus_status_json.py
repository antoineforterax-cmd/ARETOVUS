#!/usr/bin/env python3
"""Tests for the local ARETOVUS V2 read-only JSON boundary."""
from __future__ import annotations

from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from aretovus_status_json import StatusError, parse_verified_status, verified_status

TIP = "a1" * 32
GOOD = (
    f"verified testnet data: height=42 tip={TIP} "
    "known_blocks=44 work=ChainWork([100, 0, 0, 0, 0])\n"
)


class ReadOnlyStatusTests(unittest.TestCase):
    def test_canonical_valid_state(self) -> None:
        out = parse_verified_status(GOOD)
        self.assertEqual((out["height"], out["known_blocks"]), (42, 44))
        self.assertEqual(out["network"], "testnet")
        self.assertEqual(out["tip"], TIP)
        self.assertTrue(out["read_only"])

    def test_accepts_genesis_and_large_known_count(self) -> None:
        valid = (f"verified testnet data: height=0 tip={TIP} "
                 "known_blocks=1 work=ChainWork([0, 0, 0, 0, 0])\n")
        self.assertEqual(parse_verified_status(valid)["height"], 0)

    def test_rejects_untrusted_or_garbled_output(self) -> None:
        bad = (
            "", "warning\n" + GOOD, GOOD + GOOD,
            GOOD.replace("height=42", "height=-1"),
            GOOD.replace("known_blocks=44", "known_blocks=42"),
            GOOD.replace("ChainWork([100, 0, 0, 0, 0])", "100"),
            GOOD.replace(TIP, "../" + TIP[3:]),
            GOOD.replace("height=42", "height=18446744073709551616"),
            GOOD + "x" * 4097,
            GOOD.replace("height=42", "height=abc"),
            GOOD.replace("work=ChainWork", "work=\x1b[31mChainWork"),
        )
        for output in bad:
            with self.subTest(output=output[:70]), self.assertRaises(StatusError):
                parse_verified_status(output)

    def test_never_invokes_shell_or_non_readonly_command(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            node = root / "node with spaces"
            node.write_bytes(b"placeholder")
            data = root / "node data"
            data.mkdir()
            with patch("subprocess.run", return_value=subprocess.CompletedProcess(
                args=[], returncode=0, stdout=GOOD.encode(), stderr=b""
            )) as runner:
                self.assertEqual(verified_status(node, data)["height"], 42)
                args, kwargs = runner.call_args
                self.assertEqual(args[0][1:3], ["testnet", "verify"])
                self.assertEqual(args[0][3], "--data-dir")
                self.assertFalse(kwargs.get("shell", False))
                self.assertLessEqual(kwargs["timeout"], 120)

    def test_refuse_error_without_leaking_stderr(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            node = root / "node"
            node.touch()
            data = root / "data"
            data.mkdir()
            with patch("subprocess.run", return_value=subprocess.CompletedProcess(
                args=[], returncode=1, stdout=b"", stderr=b"private C:/User/Secret"
            )):
                with self.assertRaisesRegex(StatusError, "verify_failed"):
                    verified_status(node, data)
            with patch("subprocess.run", side_effect=subprocess.TimeoutExpired(
                cmd="testnet verify", timeout=1
            )):
                with self.assertRaisesRegex(StatusError, "verify_timeout"):
                    verified_status(node, data, timeout=1)

    def test_timeout_bounds_and_missing_files(self) -> None:
        with self.assertRaisesRegex(StatusError, "invalid_timeout"):
            verified_status(Path("absent"), Path("absent"), timeout=0)
        with self.assertRaises(FileNotFoundError):
            verified_status(Path("absent"), Path("absent"), timeout=1)


if __name__ == "__main__":
    unittest.main()

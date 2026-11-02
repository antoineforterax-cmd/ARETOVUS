#!/usr/bin/env python3
"""Security and rendering regressions for the ARETOVUS offline status page."""
from __future__ import annotations

from pathlib import Path
import tempfile
import unittest

from aretovus_snapshot_page import checked_payload, render, write_atomic


def valid() -> dict[str, object]:
    return {
        "schema_version": 1,
        "network": "testnet",
        "status": "verified",
        "height": 128,
        "known_blocks": 130,
        "tip": "a1" * 32,
        "chain_work_display": "ChainWork([200, 0, 0, 0, 0])",
        "source": "aretovus-node testnet verify",
        "read_only": True,
    }


class OfflineDashboardTests(unittest.TestCase):
    def test_real_contract_produces_readable_offline_page(self) -> None:
        page = render(valid())
        self.assertIn("ARETOVUS", page)
        self.assertIn("Instantané", page)
        self.assertIn("Blocs connus", page)
        self.assertIn('>128</div>', page)
        self.assertIn('>130</div>', page)
        self.assertIn("a1" * 32, page)
        self.assertIn("ChainWork([200, 0, 0, 0, 0])", page)
        self.assertIn("sa provenance n\u0027est pas authentifiée", page)\n        self.assertIn("Statut déclaré : vérifié", page)\n        self.assertIn("Source : fichier JSON fourni", page)
        self.assertIn("default-src 'none'", page)
        self.assertIn(":root {color-scheme:", page)
        self.assertNotIn("<script", page)
        self.assertNotIn('src="https:', page)

    def test_never_accept_unverified_or_forged_status(self) -> None:
        for field, value in (
            ("status", "error"), ("network", "regtest"),
            ("read_only", False), ("height", -1),
            ("height", True), ("known_blocks", 0),
            ("tip", '"><script>alert(1)</script>'),
            ("chain_work_display", "123<script>"),
            ("schema_version", 2),
        ):
            with self.subTest(field=field), self.assertRaises(ValueError):
                invalid = valid()
                invalid[field] = value
                render(invalid)
        with self.assertRaises(ValueError):
            checked_payload(["not an object"])

    def test_atomic_output_and_refuse_symlink_target(self) -> None:
        with tempfile.TemporaryDirectory(prefix="ARETOVUS ui spaces ") as temp:
            root = Path(temp)
            page = root / "Node status with spaces.html"
            write_atomic(page, render(valid()))
            self.assertIn("ARETOVUS", page.read_text(encoding="utf-8"))
            write_atomic(page, render(valid()))
            self.assertTrue(page.is_file())
            alias = root / "do not overwrite"
            try:
                alias.symlink_to(page)
            except OSError:
                self.skipTest("symlink not permitted by this OS")
            with self.assertRaisesRegex(ValueError, "refuse_symlink_output"):
                write_atomic(alias, render(valid()))
            self.assertTrue(page.is_file())


if __name__ == "__main__":
    unittest.main()

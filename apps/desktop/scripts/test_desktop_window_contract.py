#!/usr/bin/env python3
"""Platform-neutral adversarial contract tests for the real Win32 GUI smoke."""
from __future__ import annotations

import sys
import unittest

import test_desktop_window as gui


class Process:
    pid = 98765

    def __init__(self, return_code=None):
        self.return_code = return_code

    def poll(self):
        return self.return_code


class WindowContractTests(unittest.TestCase):
    def setUp(self):
        self.clock = [0.0]

    def now(self):
        return self.clock[0]

    def sleep(self, seconds):
        self.clock[0] += seconds

    def wait(self, proc, find):
        return gui.wait_for_desktop_window(
            proc, find, timeout=1.0, now=self.now, sleep=self.sleep,
        )

    def test_exact_title_and_correct_process_id(self):
        seen = []
        def find(pid):
            seen.append(pid)
            return [gui.EXPECTED_TITLE]
        self.assertEqual(self.wait(Process(), find), gui.EXPECTED_TITLE)
        self.assertEqual(seen, [98765])

    def test_reject_spoofed_window_or_wrong_title(self):
        with self.assertRaisesRegex(TimeoutError, "No visible"):
            self.wait(Process(), lambda pid: [gui.EXPECTED_TITLE + " (spoofed)"])

    def test_reject_unrelated_window_even_if_visible(self):
        with self.assertRaisesRegex(TimeoutError, "No visible"):
            self.wait(Process(), lambda pid: ["Other app", "ARETOVUS launch failed"])

    def test_reject_process_that_quits_before_window(self):
        with self.assertRaisesRegex(RuntimeError, "exited"):
            self.wait(Process(return_code=0), lambda pid: [gui.EXPECTED_TITLE])

    def test_no_window_has_bounded_timeout(self):
        with self.assertRaises(TimeoutError):
            self.wait(Process(), lambda pid: [])
        self.assertLessEqual(self.clock[0], 1.25)

    @unittest.skipIf(sys.platform == "win32", "Windows-specific enumeration is exercised by CI")
    def test_non_windows_cannot_forge_real_gui_pass(self):
        with self.assertRaisesRegex(RuntimeError, "Windows"):
            gui.top_level_windows_for_pid(123)


if __name__ == "__main__":
    unittest.main()

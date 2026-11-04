#!/usr/bin/env python3
"""Windows CI: prove the extracted, unsigned ARETOVUS desktop opens a window.

This test does NOT validate React content, WebView2 accessibility, signing,
screen rendering, P2P connectivity, or suitability for actual funds.
"""
from __future__ import annotations

import argparse
import ctypes
import hashlib
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import zipfile

import package_windows_lab as lab

EXPECTED_TITLE = "ARETOVUS — Node Desktop"
WINDOW_TIMEOUT_SECONDS = 45


def binary_digest(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def top_level_windows_for_pid(pid: int) -> list[str]:
    """Enumerate actual visible top-level Win32 windows owned by one PID."""
    if sys.platform != "win32":
        raise RuntimeError("A Windows GUI runner is mandatory for this test")

    from ctypes import wintypes

    user32 = ctypes.WinDLL("user32", use_last_error=True)
    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)
    user32.EnumWindows.argtypes = (callback_type, wintypes.LPARAM)
    user32.EnumWindows.restype = wintypes.BOOL
    user32.GetWindowThreadProcessId.argtypes = (
        wintypes.HWND, ctypes.POINTER(wintypes.DWORD),
    )
    user32.GetWindowThreadProcessId.restype = wintypes.DWORD
    user32.IsWindowVisible.argtypes = (wintypes.HWND,)
    user32.IsWindowVisible.restype = wintypes.BOOL
    user32.GetWindowTextLengthW.argtypes = (wintypes.HWND,)
    user32.GetWindowTextLengthW.restype = ctypes.c_int
    user32.GetWindowTextW.argtypes = (wintypes.HWND, wintypes.LPWSTR, ctypes.c_int)
    user32.GetWindowTextW.restype = ctypes.c_int
    results: list[str] = []

    @callback_type
    def visit(hwnd, _param):
        owner = wintypes.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(owner))
        if owner.value != pid or not user32.IsWindowVisible(hwnd):
            return True
        length = user32.GetWindowTextLengthW(hwnd)
        if length <= 0 or length > 2048:
            return True
        title = ctypes.create_unicode_buffer(length + 1)
        if user32.GetWindowTextW(hwnd, title, len(title)):
            results.append(title.value)
        return True

    if not user32.EnumWindows(visit, 0):
        raise RuntimeError("EnumWindows failed; cannot attest visible GUI")
    return results


def wait_for_desktop_window(
    process,
    find_windows,
    *,
    timeout: float = WINDOW_TIMEOUT_SECONDS,
    now=time.monotonic,
    sleep=time.sleep,
) -> str:
    """Require one visible matching window owned by the launched GUI PID."""
    deadline = now() + timeout
    while now() < deadline:
        if process.poll() is not None:
            raise RuntimeError("Desktop process exited before a verified window appeared")
        titles = find_windows(process.pid)
        if EXPECTED_TITLE in titles:
            return EXPECTED_TITLE
        sleep(0.25)
    raise TimeoutError("No visible ARETOVUS desktop window before the bounded deadline")


def smoke_archive(archive: Path) -> None:
    if sys.platform != "win32":
        raise RuntimeError("Windows only: cannot substitute a Linux simulated GUI")

    lab.verify_archive(archive)
    with tempfile.TemporaryDirectory(prefix="ARETOVUS real desktop window test ") as temp:
        folder = Path(temp) / lab.ROOT.rstrip("/")
        with zipfile.ZipFile(archive) as zipped:
            # verify_archive requires a fixed flat allowlist and checks all
            # members before any extracted file is trusted.
            zipped.extractall(temp)
        expected = set(lab.BINARIES) | {lab.MANIFEST, lab.SUMS, lab.GUIDE}
        if not folder.is_dir() or {item.name for item in folder.iterdir()} != expected:
            raise RuntimeError("Unexpected archive layout after extraction")
        executables = {name: folder / name for name in lab.BINARIES}
        hashes_before = {name: binary_digest(path) for name, path in executables.items()}

        # No user-supplied data directory or remote service is contacted. No
        # credentials are passed intentionally to the GUI in a CI job.
        environment = os.environ.copy()
        for secret_name in (
            "GITHUB_TOKEN", "GH_TOKEN", "ACTIONS_RUNTIME_TOKEN",
            "ACTIONS_ID_TOKEN_REQUEST_TOKEN", "ACTIONS_ID_TOKEN_REQUEST_URL",
        ):
            environment.pop(secret_name, None)
        diagnostic = Path(temp) / "desktop-stderr.log"
        with diagnostic.open("wb") as errors:
            proc = subprocess.Popen(
                [str(executables["aretovus-desktop.exe"])],
                cwd=folder,
                stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL,
                stderr=errors,
                env=environment,
            )
            try:
                wait_for_desktop_window(proc, top_level_windows_for_pid)
                # A transient startup splash screen is insufficient.
                time.sleep(2)
                if proc.poll() is not None or EXPECTED_TITLE not in top_level_windows_for_pid(proc.pid):
                    raise RuntimeError("The desktop window disappeared after startup")
            except Exception as error:
                snippet = diagnostic.read_bytes()[-1536:].decode("utf-8", errors="replace")
                raise RuntimeError(f"{error}; stderr tail: {snippet}") from error
            finally:
                if proc.poll() is None:
                    proc.terminate()
                try:
                    proc.wait(timeout=12)
                except subprocess.TimeoutExpired:
                    proc.kill()
                    proc.wait(timeout=5)

        if hashes_before != {name: binary_digest(path) for name, path in executables.items()}:
            raise RuntimeError("The running GUI changed a shipped executable")
        if {item.name for item in folder.iterdir()} != expected:
            raise RuntimeError("The running GUI wrote files into its own installation directory")

    print("PASS: actual extracted ARETOVUS desktop opened a stable visible window on Windows")
    print("NOT CHECKED: rendered React content, accessibility, GUI interactions, installer or funds")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", required=True, type=Path)
    args = parser.parse_args()
    smoke_archive(args.archive.resolve(strict=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

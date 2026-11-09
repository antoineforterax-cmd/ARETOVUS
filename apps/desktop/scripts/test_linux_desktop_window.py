#!/usr/bin/env python3
"""Check that the release Tauri process opens a stable X11 window in CI."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


TITLE = "ARETOVUS — Node Desktop"


def visible_windows(pid: int) -> set[str]:
    search = subprocess.run(
        ["xdotool", "search", "--onlyvisible", "--pid", str(pid)],
        capture_output=True, text=True, timeout=2,
    )
    if search.returncode:
        return set()
    windows = set()
    for window in search.stdout.splitlines():
        name = subprocess.run(
            ["xdotool", "getwindowname", window],
            capture_output=True, text=True, encoding="utf-8", timeout=2,
        )
        if name.returncode == 0 and name.stdout.removesuffix("\n") == TITLE:
            windows.add(window)
    return windows


def verify_window(process: subprocess.Popen) -> None:
    deadline = time.monotonic() + 30
    first_seen = {}
    while time.monotonic() < deadline:
        exit_code = process.poll()
        if exit_code is not None:
            raise RuntimeError(f"Desktop exited before opening a stable window (exit {exit_code})")
        windows = visible_windows(process.pid)
        now = time.monotonic()
        first_seen = {window: first_seen.get(window, now) for window in windows}
        if now < deadline and any(now - seen >= 2 for seen in first_seen.values()):
            if process.poll() is None:
                print(f"Verified visible ARETOVUS window from desktop PID {process.pid}")
                return
        time.sleep(0.2)
    raise RuntimeError(f"No stable ARETOVUS window from desktop PID {process.pid}")


def main(binary: Path) -> None:
    binary = binary.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="aretovus-linux-window-") as home:
        runtime = Path(home) / "runtime"
        runtime.mkdir(mode=0o700)
        env = {
            "DISPLAY": os.environ["DISPLAY"],
            "XAUTHORITY": os.environ["XAUTHORITY"],
            "PATH": os.defpath,
            "HOME": home,
            "XDG_RUNTIME_DIR": str(runtime),
            "GDK_BACKEND": "x11",
            "LANG": "C.UTF-8",
        }
        process = subprocess.Popen([str(binary)], cwd=home, env=env, stdin=subprocess.DEVNULL)
        try:
            verify_window(process)
        finally:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=5)


if __name__ == "__main__":
    main(Path(sys.argv[1]))

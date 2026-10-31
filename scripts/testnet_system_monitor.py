#!/usr/bin/env python3
"""Passive system telemetry for AURORA endurance checks, Python stdlib only.

Records OS-wide CPU/RAM snapshots, not AURORA's individual resource consumption.
Runs in a background thread while the verification subprocess is blocked.
No admin rights, network access, secrets, or protocol changes.
"""
from __future__ import annotations

import ctypes
import json
import shutil
import sys
import threading
import time
from pathlib import Path


def _windows_cpu_ticks() -> tuple[int, int]:
    class FileTime(ctypes.Structure):
        _fields_ = [("low", ctypes.c_uint32), ("high", ctypes.c_uint32)]

    idle, kernel, user = FileTime(), FileTime(), FileTime()
    if not ctypes.windll.kernel32.GetSystemTimes(
        ctypes.byref(idle), ctypes.byref(kernel), ctypes.byref(user)
    ):
        raise OSError("GetSystemTimes failed")

    def value(x: FileTime) -> int:
        return (int(x.high) << 32) | int(x.low)

    # Windows kernel time includes idle time.
    return value(idle), value(kernel) + value(user)


def _linux_cpu_ticks() -> tuple[int, int]:
    fields = Path("/proc/stat").read_text(encoding="ascii").splitlines()[0].split()
    if fields[0] != "cpu" or len(fields) < 5:
        raise ValueError("invalid /proc/stat")
    ticks = [int(t) for t in fields[1:]]
    # Linux idle plus iowait counts as non-busy.
    return ticks[3] + (ticks[4] if len(ticks) > 4 else 0), sum(ticks)


def cpu_percent(previous: tuple[int, int] | None,
                current: tuple[int, int]) -> float | None:
    if previous is None:
        return None
    idle = current[0] - previous[0]
    total = current[1] - previous[1]
    if total <= 0 or idle < 0 or idle > total:
        return None
    return round(max(0.0, min(100.0, 100 * (1 - idle / total))), 2)


def _windows_memory() -> tuple[int, int, int]:
    class MemoryStatus(ctypes.Structure):
        _fields_ = [
            ("length", ctypes.c_ulong), ("load", ctypes.c_ulong),
            ("total_phys", ctypes.c_ulonglong), ("avail_phys", ctypes.c_ulonglong),
            ("total_page", ctypes.c_ulonglong), ("avail_page", ctypes.c_ulonglong),
            ("total_virtual", ctypes.c_ulonglong), ("avail_virtual", ctypes.c_ulonglong),
            ("avail_extended", ctypes.c_ulonglong),
        ]
    status = MemoryStatus()
    status.length = ctypes.sizeof(status)
    if not ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(status)):
        raise OSError("GlobalMemoryStatusEx failed")
    if not status.total_phys or status.avail_phys > status.total_phys:
        raise ValueError("invalid Windows RAM readings")
    return int(status.load), int(status.avail_phys), int(status.total_phys)


def _linux_memory() -> tuple[int, int, int]:
    fields = {}
    for line in Path("/proc/meminfo").read_text(encoding="ascii").splitlines():
        parts = line.split()
        if len(parts) >= 2 and parts[0].endswith(":"):
            fields[parts[0][:-1]] = int(parts[1]) * 1024
    available = fields["MemAvailable"]
    total = fields["MemTotal"]
    if total <= 0 or available > total:
        raise ValueError("invalid Linux RAM readings")
    return round(100 * (1 - available / total)), available, total


def snapshot(data_dir: Path, previous_cpu: tuple[int, int] | None
             ) -> tuple[dict[str, object], tuple[int, int] | None]:
    cpu_now = None
    ram = None
    if sys.platform == "win32":
        cpu_now = _windows_cpu_ticks()
        ram = _windows_memory()
    elif sys.platform.startswith("linux"):
        cpu_now = _linux_cpu_ticks()
        ram = _linux_memory()
    result: dict[str, object] = {
        "free_disk_bytes": shutil.disk_usage(data_dir).free,
        "cpu_percent": cpu_percent(previous_cpu, cpu_now) if cpu_now else None,
        "ram_load_percent": ram[0] if ram else None,
        "available_ram_bytes": ram[1] if ram else None,
        "total_ram_bytes": ram[2] if ram else None,
        "disk_io_utilization_percent": None,  # Not measured; do not invent it.
    }
    return result, cpu_now


class ResourceMonitor:
    """Separate append-only JSONL file; never mix with strict node evidence."""

    def __init__(self, path: Path, data_dir: Path, host_label: str,
                 interval_seconds: float = 5.0):
        if not 1 <= interval_seconds <= 300:
            raise ValueError("resource interval must be 1..300 seconds")
        self.path = path
        self.data_dir = data_dir
        self.label = host_label
        self.interval = interval_seconds
        self.samples = 0
        self.errors = 0
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None
        self._handle = None
        self._completed = False

    def _write(self, event: str, **values: object) -> None:
        assert self._handle is not None
        self._handle.write(json.dumps(
            {"event": event, "host_label": self.label,
             "unix_utc": round(time.time(), 3), **values},
            sort_keys=True
        ) + "\n")
        self._handle.flush()

    def __enter__(self) -> "ResourceMonitor":
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self._handle = self.path.open("a", encoding="utf-8")
        self._write("start", interval_seconds=self.interval, version=1, platform=sys.platform)
        self._start_time = time.monotonic()
        self._thread = threading.Thread(target=self._loop, name="aurora-resource-probe",
                                        daemon=True)
        self._thread.start()
        return self

    def _loop(self) -> None:
        previous = None
        while not self._stop.is_set():
            elapsed = round(time.monotonic() - self._start_time, 3)
            try:
                values, previous = snapshot(self.data_dir, previous)
                self._write("sample", status="ok", elapsed_seconds=elapsed, **values)
            except Exception as exc:  # Unexpected metric failures must fail endurance gate
                self.errors += 1
                self._write("sample", status="error", elapsed_seconds=elapsed,
                            error=type(exc).__name__)
            self.samples += 1
            self._stop.wait(self.interval)
        self._write("finish", samples=self.samples, errors=self.errors,
                    elapsed_seconds=round(time.monotonic() - self._start_time, 3))
        self._completed = True

    def stop(self) -> int:
        if self._thread is None:
            return self.errors
        self._stop.set()
        self._thread.join(timeout=15)
        if self._thread.is_alive() or not self._completed:
            raise RuntimeError("Resource monitor did not finish cleanly")
        assert self._handle is not None
        self._handle.close()
        self._handle = None
        self._thread = None
        return self.errors

    def __exit__(self, _type, _value, _traceback) -> None:
        self.stop()


def self_test() -> None:
    import tempfile
    from unittest.mock import patch

    assert cpu_percent((100, 1000), (120, 1100)) == 80.0
    assert cpu_percent(None, (0, 10)) is None
    assert cpu_percent((0, 20), (10, 10)) is None
    with tempfile.TemporaryDirectory(prefix="aurora-resources-") as temp:
        root = Path(temp)
        path = root / "telemetry.jsonl"
        with patch(__name__ + ".snapshot", side_effect=[
            ({"ram_load_percent": 90, "cpu_percent": None}, (1, 10)),
            ({"ram_load_percent": 73, "cpu_percent": 25.0}, (2, 20)),
        ]):
            with ResourceMonitor(path, root, "host-test", interval_seconds=1) as m:
                for _ in range(40):
                    if m.samples >= 1:
                        break
                    time.sleep(0.025)
                assert m.samples >= 1
                assert m.errors == 0
        events = [json.loads(line) for line in path.read_text().splitlines()]
        assert events[0]["event"] == "start"
        assert events[-1]["event"] == "finish"
        assert events[-1]["errors"] == 0
        assert events[1]["ram_load_percent"] == 90
        with patch(__name__ + ".snapshot", side_effect=OSError("private path")):
            with ResourceMonitor(path, root, "host-test", interval_seconds=1) as m:
                for _ in range(40):
                    if m.errors >= 1:
                        break
                    time.sleep(0.025)
                assert m.errors >= 1
        assert "private path" not in path.read_text(encoding="utf-8")
    print("resource_monitor_selftest=ok")


if __name__ == "__main__":
    self_test()

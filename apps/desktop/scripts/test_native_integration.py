#!/usr/bin/env python3
"""Windows CI: real native ARETOVUS node -> read-only bridge, disposable data only."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import tempfile

REQUEST = b'{"schema_version":1,"method":"status"}'
MINER = "11" * 32


def execute(binary: Path, data_dir: Path, payload: bytes, timeout: int = 30):
    return subprocess.run(
        [str(binary), "--data-dir", str(data_dir)],
        input=payload,
        capture_output=True,
        check=False,
        timeout=timeout,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--node", type=Path, required=True)
    parser.add_argument("--bridge", type=Path, required=True)
    args = parser.parse_args()
    node = args.node.resolve(strict=True)
    bridge = args.bridge.resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="ARETOVUS desktop IPC with spaces ") as temp:
        root = Path(temp)
        data = root / "fresh testnet directory"
        data.mkdir()
        mined = subprocess.run(
            [str(node), "testnet", "mine", "--data-dir", str(data),
             "--miner", MINER, "--count", "1"],
            capture_output=True, check=False, timeout=90,
        )
        assert mined.returncode == 0, "Failed to mine disposable native testnet block"

        good = execute(bridge, data, REQUEST)
        assert good.returncode == 0, "Native bridge refused verified snapshot"
        assert len(good.stdout) <= 4096
        state = json.loads(good.stdout)
        assert state["schema_version"] == 1 and state["network"] == "testnet"
        assert state["status"] == "verified" and state["read_only"] is True
        assert state["source"] == "aretovus-native-stdio"
        assert state["height"] == 1 and state["known_blocks"] >= 2
        assert len(state["tip"]) == 64

        for payload in (
            b'{"schema_version":1,"method":"sign"}',
            b'{"schema_version":1,"method":"status","method":"sign"}',
            b"a" * 10000,
        ):
            rejected = execute(bridge, data, payload)
            assert rejected.returncode == 2
            answer = json.loads(rejected.stdout)
            assert answer["status"] == "error" and answer["read_only"] is True
            assert str(data).encode() not in rejected.stdout
            assert b"Traceback" not in rejected.stdout

        absent = root / "must not be created"
        result = execute(bridge, absent, REQUEST)
        assert result.returncode == 2 and not absent.exists()
        assert json.loads(result.stdout)["code"] == "invalid_data_dir"

        print("ARETOVUS desktop IPC native test: real node + one block + safe read-only bridge passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

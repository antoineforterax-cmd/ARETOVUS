#!/usr/bin/env python3
"""ARETOVUS V2: machine-readable, read-only status for a *local verified node*.

Uses the current Rust node's `testnet verify` command, not an HTTP server.
No signing, wallet key access, network port or unverified explorer data.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

SCHEMA_VERSION = 1
OUTPUT_LIMIT = 4096
WORK_REPR = re.compile(r"ChainWork\(\[[0-9, ]{1,240}\]\)\Z")
VERIFY_LINE = re.compile(
    r"verified testnet data: height=([0-9]+) tip=([0-9a-f]{64}) "
    r"known_blocks=([0-9]+) work=(.+)\Z"
)


class StatusError(ValueError):
    """A bounded, public error code: never echo arbitrary node stderr."""


def parse_verified_status(output: str) -> dict[str, object]:
    if len(output) > OUTPUT_LIMIT:
        raise StatusError("output_too_large")
    lines = output.splitlines()
    # Reject additional output rather than accidentally interpreting an
    # arbitrary log line or warning as a verified and trustworthy state.
    if len(lines) != 1:
        raise StatusError("unexpected_output")
    match = VERIFY_LINE.fullmatch(lines[0])
    if not match:
        raise StatusError("unexpected_output")
    height, tip, blocks, work_repr = match.groups()
    if not WORK_REPR.fullmatch(work_repr):
        raise StatusError("unexpected_work_format")
    height_num, known = int(height), int(blocks)
    if known < height_num + 1 or height_num > 2**64 - 1 or known > 2**64 - 1:
        raise StatusError("inconsistent_chain_counts")
    return {
        "schema_version": SCHEMA_VERSION,
        "network": "testnet",
        "status": "verified",
        "height": height_num,
        "tip": tip,
        "known_blocks": known,
        # For display only: Debug formatting is NOT a public numeric wire API.
        "chain_work_display": work_repr,
        "source": "aretovus-node testnet verify",
        "read_only": True,
    }


def verified_status(node_binary: Path, data_dir: Path, timeout: float = 30) -> dict[str, object]:
    if not (0 < timeout <= 120):
        raise StatusError("invalid_timeout")
    node = node_binary.expanduser().resolve(strict=True)
    directory = data_dir.expanduser().resolve(strict=True)
    if not node.is_file() or not directory.is_dir():
        raise StatusError("invalid_local_path")
    try:
        result = subprocess.run(
            [str(node), "testnet", "verify", "--data-dir", str(directory)],
            capture_output=True, timeout=timeout, check=False,
        )
    except subprocess.TimeoutExpired as err:
        raise StatusError("verify_timeout") from err
    except OSError as err:
        raise StatusError("node_unavailable") from err
    if result.returncode != 0:
        raise StatusError("verify_failed")
    if len(result.stdout) > OUTPUT_LIMIT:
        raise StatusError("output_too_large")
    try:
        output = result.stdout.decode("utf-8", errors="strict")
    except UnicodeDecodeError as err:
        raise StatusError("unexpected_encoding") from err
    return parse_verified_status(output)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--node-binary", required=True, type=Path)
    parser.add_argument("--data-dir", required=True, type=Path)
    parser.add_argument("--timeout", type=float, default=30)
    args = parser.parse_args()
    try:
        result = verified_status(args.node_binary, args.data_dir, args.timeout)
    except (StatusError, OSError, ValueError) as err:
        # No raw paths, OS or node stderr in the UI-facing response.
        code = str(err) if isinstance(err, StatusError) else "invalid_local_path"
        print(json.dumps({
            "schema_version": SCHEMA_VERSION,
            "status": "error",
            "code": code,
            "read_only": True,
        }, sort_keys=True))
        return 2
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())

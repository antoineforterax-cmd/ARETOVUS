#!/usr/bin/env python3
"""Build and verify an unsigned, CI-only portable ARETOVUS testnet lab ZIP.

Do not use as a release, installer, or provenance/signature attestation.
Never touches a testnet data directory or runs any of the binaries.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import zipfile

ROOT = "ARETOVUS-Desktop-Lab/"
BINARIES = (
    "aretovus-desktop.exe",
    "aretovus-local-bridge.exe",
    "aretovus-explorer.exe",
    "aretovus-node.exe",
)
MANIFEST = "manifest.json"
SUMS = "SHA256SUMS.txt"
GUIDE = "README-LAB.txt"
MAX_EXE = 256 * 1024 * 1024
MAX_METADATA = 16 * 1024
TIME = (2026, 11, 2, 0, 0, 0)

README = """ARETOVUS DESKTOP LAB — UNSIGNED EXPERIMENTAL TESTNET ONLY

This is a CI-built PORTABLE ZIP, NOT an installer or a public release.
Never use this build with real assets, private production keys or funds.

Binaries are placed side-by-side so the Tauri host can find its native
companions without PATH or a shell. The desktop is read-only, and it
displays only verified LOCAL snapshots; it cannot confirm global P2P sync.
aretovus-node.exe is a separate experimental CLI for explicit testing.
The GUI does not automatically start that CLI or expose a mining button.

1. Extract this ZIP to a private, non-shared folder on Windows.
2. Verify checksums against SHA256SUMS.txt. Checksums are NOT signatures
   and a modified archive can contain modified checksums.
3. Install the Microsoft WebView2 runtime if Windows does not provide it.
4. Create fresh native ARETOVUS testnet data using the node CLI separately.
   NEVER reuse legacy data or actual wallet backups.
5. Open aretovus-desktop.exe and select an EXISTING local testnet data
   directory. Missing/invalid snapshots produce errors; no fake balances.
6. If Windows warns about unsigned software, DO NOT disable security
   protections to bypass it. Use only on test machines you control.

This archive is not code-signed, not externally audited, not a mainnet,
not a wallet, and not approved for handling real funds. Check GitHub CI
for the exact source commit in manifest.json before trusting test evidence.
"""


def digest_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def file_digest(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def validate_binary(path: Path, name: str) -> Path:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"Missing or symlinked companion: {name}")
    size = path.stat().st_size
    if size < 2 or size > MAX_EXE:
        raise ValueError(f"Invalid native binary size: {name}")
    with path.open("rb") as handle:
        if handle.read(2) != b"MZ":
            raise ValueError(f"Companion is not a Windows PE executable: {name}")
    return path


def zip_info(name: str) -> zipfile.ZipInfo:
    info = zipfile.ZipInfo(ROOT + name, TIME)
    info.compress_type = zipfile.ZIP_STORED
    info.create_system = 0
    info.external_attr = 0x20
    return info


def create_archive(args: argparse.Namespace) -> None:
    source_sha = args.source_sha.lower()
    if not re.fullmatch(r"[0-9a-f]{40}", source_sha):
        raise ValueError("source SHA must be a full 40-character Git SHA")
    inputs = {
        name: validate_binary(Path(getattr(args, key)), name)
        for name, key in (
            (BINARIES[0], "desktop"),
            (BINARIES[1], "bridge"),
            (BINARIES[2], "explorer"),
            (BINARIES[3], "node"),
        )
    }
    output = Path(args.output)
    if output.is_symlink() or output.exists():
        raise ValueError("Refusing to replace an existing archive")
    if any(output.resolve() == path.resolve() for path in inputs.values()):
        raise ValueError("Output overlaps input executable")
    output.parent.mkdir(parents=True, exist_ok=True)

    hashes = {name: file_digest(path) for name, path in inputs.items()}
    manifest = {
        "schema_version": 1,
        "product": "ARETOVUS Desktop Lab",
        "network": "testnet",
        "read_only_gui": True,
        "signed": False,
        "release_approved": False,
        "source_commit": source_sha,
        "binaries_sha256": hashes,
        "guide_sha256": digest_bytes(README.encode("utf-8")),
    }
    payloads = {
        MANIFEST: (json.dumps(manifest, sort_keys=True, indent=2) + "\n").encode(),
        SUMS: "".join(f"{hashes[name]}  {name}\n" for name in BINARIES).encode(),
        GUIDE: README.encode("utf-8"),
    }
    if any(len(v) > MAX_METADATA for v in payloads.values()):
        raise ValueError("Metadata too large")
    # ZipFile(mode="x") uses exclusive creation. If another process wins
    # the race between exists() and open(), never delete *its* file.
    archive_created_by_us = False
    try:
        with zipfile.ZipFile(output, "x", allowZip64=True) as archive:
            archive_created_by_us = True
            for name in BINARIES:
                with inputs[name].open("rb") as source, archive.open(zip_info(name), "w", force_zip64=True) as target:
                    for chunk in iter(lambda: source.read(1024 * 1024), b""):
                        target.write(chunk)
            for name, data in payloads.items():
                archive.writestr(zip_info(name), data)
        verify_archive(output)
    except Exception:
        if archive_created_by_us:
            output.unlink(missing_ok=True)
        raise
    print(f"ARETOVUS lab archive created and verified: {output.name}; source {source_sha}")


def verify_archive(path: Path) -> None:
    expected = {ROOT + name for name in (*BINARIES, MANIFEST, SUMS, GUIDE)}
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        if len(names) != len(expected) or set(names) != expected:
            raise ValueError("Unexpected, duplicate or missing archive entries")
        if any(
            item.file_size > (MAX_EXE if item.filename.endswith(".exe") else MAX_METADATA)
            or item.compress_type != zipfile.ZIP_STORED
            or item.filename.startswith("/")
            or ".." in Path(item.filename).parts
            for item in archive.infolist()
        ):
            raise ValueError("Invalid entry limits, paths or compression")
        manifest = json.loads(archive.read(ROOT + MANIFEST))
        if (
            manifest.get("schema_version") != 1
            or manifest.get("network") != "testnet"
            or manifest.get("read_only_gui") is not True
            or manifest.get("signed") is not False
            or manifest.get("release_approved") is not False
            or not isinstance(manifest.get("source_commit"), str)
            or re.fullmatch(r"[0-9a-f]{40}", manifest["source_commit"]) is None
        ):
            raise ValueError("Invalid lab metadata")
        if set(manifest) != {
            "schema_version", "product", "network", "read_only_gui",
            "signed", "release_approved", "source_commit",
            "binaries_sha256", "guide_sha256",
        } or manifest["product"] != "ARETOVUS Desktop Lab":
            raise ValueError("Unexpected metadata keys or product identity")
        guide_hash = manifest["guide_sha256"]
        if not isinstance(guide_hash, str) or not re.fullmatch(r"[0-9a-f]{64}", guide_hash):
            raise ValueError("Invalid guide hash format")
        guide_bytes = archive.read(ROOT + GUIDE)
        if digest_bytes(guide_bytes) != guide_hash or guide_bytes != README.encode("utf-8"):
            raise ValueError("Guide checksum mismatch or unexpected safety guidance")
        hashes = manifest.get("binaries_sha256")
        if not isinstance(hashes, dict) or set(hashes) != set(BINARIES):
            raise ValueError("Incorrect companion inventory")
        checksums = archive.read(ROOT + SUMS).decode("ascii")
        expected_sums = "".join(f"{hashes[name]}  {name}\n" for name in BINARIES)
        if checksums != expected_sums:
            raise ValueError("Checksum listing does not match manifest")
        for name in BINARIES:
            sha = hashes[name]
            if not isinstance(sha, str) or re.fullmatch(r"[0-9a-f]{64}", sha) is None:
                raise ValueError(f"Bad hash format: {name}")
            digest = hashlib.sha256()
            with archive.open(ROOT + name) as handle:
                first = handle.read(2)
                if first != b"MZ":
                    raise ValueError(f"Not a Windows PE executable: {name}")
                digest.update(first)
                for block in iter(lambda: handle.read(1024 * 1024), b""):
                    digest.update(block)
            if digest.hexdigest() != sha:
                raise ValueError(f"Companion checksum mismatch: {name}")
    print("ARETOVUS lab archive structure, limits and checksums verified (not a signature)")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    make = sub.add_parser("pack")
    make.add_argument("--desktop", required=True)
    make.add_argument("--bridge", required=True)
    make.add_argument("--explorer", required=True)
    make.add_argument("--node", required=True)
    make.add_argument("--output", required=True)
    make.add_argument("--source-sha", required=True)
    check = sub.add_parser("verify")
    check.add_argument("--archive", required=True)
    args = parser.parse_args()
    try:
        if args.action == "pack":
            create_archive(args)
        else:
            verify_archive(Path(args.archive))
    except (OSError, ValueError, zipfile.BadZipFile, json.JSONDecodeError) as error:
        print(f"Packaging failed: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

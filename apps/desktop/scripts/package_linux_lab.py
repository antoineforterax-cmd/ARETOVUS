#!/usr/bin/env python3
"""Create and verify the unsigned Linux x86-64 portable testnet lab archive."""
from __future__ import annotations

import argparse
from contextlib import ExitStack, nullcontext
import ctypes
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import re
import stat
import struct
import sys
import tarfile
import tempfile
import zlib

BINARIES = (
    "aretovus-desktop",
    "aretovus-local-bridge",
    "aretovus-explorer",
    "aretovus-node",
)
MANIFEST = "manifest.json"
SUMS = "SHA256SUMS.txt"
GUIDE = "README-LAB.txt"
NOTICES = ("LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "PROVENANCE.md", "SECURITY.md")
PAYLOADS = (*BINARIES, GUIDE, *NOTICES)
MAX_EXE = 256 * 1024 * 1024
MAX_METADATA = 16 * 1024
MAX_ARCHIVE = len(BINARIES) * MAX_EXE + 1024 * 1024
CHUNK = 1024 * 1024
ELF_HEADER = struct.Struct("<16sHHIQQQIHHHHHH")
GZIP_HEADER = b"\x1f\x8b\x08\x00\x00\x00\x00\x00\x02\xff"
SOURCE_ROOT = Path(__file__).resolve().parents[3]

README = b"""ARETOVUS DESKTOP LAB - UNSIGNED EXPERIMENTAL TESTNET ONLY

This CI archive is for Linux x86-64. The system needs GTK 3 and
WebKitGTK 4.1 (Ubuntu 24.04: libgtk-3-0t64, libwebkit2gtk-4.1-0).
Read the manifest and confirm the source revision against the CI run.
Use the repository's package_linux_lab.py verify command with the full
expected commit and --extract-to pointing to a new private directory.
Checksums detect corruption. The archive and binaries are unsigned.

Keep the four executables together in a private, trusted directory.
Use the separate node CLI and the testnet guide to create a disposable
native testnet snapshot. Launch ./aretovus-desktop and select that
directory in the read-only GUI. Use only data and machines you control.
Keep real keys, funds and historical wallets away from the lab.
"""


def source_sha(value: str) -> str:
    if re.fullmatch(r"[0-9a-f]{40}", value) is None:
        raise ValueError("expected a full lowercase 40-character source SHA")
    return value


def regular_file(path: Path, limit: int):
    handle = os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK), "rb")
    try:
        info = os.fstat(handle.fileno())
        if not stat.S_ISREG(info.st_mode) or not 0 < info.st_size <= limit:
            raise ValueError(f"Invalid regular file or size: {path}")
    except (OSError, ValueError):
        handle.close()
        raise
    return handle, info.st_size


def validate_elf(prefix: bytes, size: int, name: str) -> None:
    if len(prefix) != ELF_HEADER.size:
        raise ValueError(f"Truncated ELF header: {name}")
    ident, kind, machine, version, entry, phoff, _, _, ehsize, phentsize, phnum, *_ = (
        ELF_HEADER.unpack(prefix)
    )
    if (
        ident[:7] != b"\x7fELF\x02\x01\x01"
        or ident[7] not in (0, 3)
        or kind not in (2, 3)
        or machine != 62
        or version != 1
        or entry == 0
        or ehsize != ELF_HEADER.size
        or phoff < ehsize
        or phentsize != 56
        or not 0 < phnum < 0xFFFF
        or phoff + phentsize * phnum > size
    ):
        raise ValueError(f"Expected an x86-64 Linux ELF executable: {name}")


def read_exact(handle, size: int) -> bytes:
    data = handle.read(size)
    if len(data) != size:
        raise ValueError("Truncated archive member or input file")
    return data


def payload_digest(handle, name: str, size: int, target=None) -> str:
    digest = hashlib.sha256()
    if name in BINARIES:
        if size < ELF_HEADER.size:
            raise ValueError(f"Truncated ELF header: {name}")
        prefix = read_exact(handle, ELF_HEADER.size)
        validate_elf(prefix, size, name)
        digest.update(prefix)
        if target is not None:
            target.write(prefix)
        size -= len(prefix)
    while size:
        block = read_exact(handle, min(CHUNK, size))
        digest.update(block)
        if target is not None:
            target.write(block)
        size -= len(block)
    return digest.hexdigest()


def entry_info(name: str, size: int) -> tarfile.TarInfo:
    entry = tarfile.TarInfo(name)
    entry.size = size
    entry.mode = 0o755 if name in BINARIES else 0o644
    return entry


def manifest_bytes(sha: str, hashes: dict[str, str]) -> bytes:
    manifest = {
        "schema_version": 1,
        "product": "ARETOVUS Desktop Lab",
        "platform": "linux-x86_64",
        "network": "testnet",
        "read_only_gui": True,
        "signed": False,
        "release_approved": False,
        "source_commit": sha,
        "files_sha256": hashes,
    }
    return (json.dumps(manifest, sort_keys=True, indent=2) + "\n").encode("utf-8")


def checksums_bytes(hashes: dict[str, str]) -> bytes:
    return "".join(f"{hashes[name]}  {name}\n" for name in PAYLOADS).encode("ascii")


def read_checksums(content: bytes) -> dict[str, str]:
    lines = content.splitlines()
    if len(lines) != len(PAYLOADS):
        raise ValueError("Invalid checksum inventory")
    hashes = {}
    for line, name in zip(lines, PAYLOADS):
        match = re.fullmatch(rb"([0-9a-f]{64})  " + re.escape(name.encode("ascii")), line)
        if match is None:
            raise ValueError(f"Invalid checksum entry: {name}")
        hashes[name] = match.group(1).decode("ascii")
    if content != checksums_bytes(hashes):
        raise ValueError("Invalid checksum formatting")
    return hashes


def read_header(handle, name: str) -> int:
    header = read_exact(handle, tarfile.BLOCKSIZE)
    # Read exactly one header at a time. TarFile iteration can internally read
    # a PAX or GNU extension payload before giving the caller the next member.
    entry = tarfile.TarInfo.frombuf(header, "ascii", "strict")
    maximum = MAX_EXE if name in BINARIES else MAX_METADATA
    if (
        not 0 < entry.size <= maximum
        or header != entry_info(name, entry.size).tobuf(format=tarfile.USTAR_FORMAT)
    ):
        raise ValueError(f"Invalid USTAR header for {name}")
    return entry.size


def padding(handle, size: int) -> None:
    if any(read_exact(handle, size)):
        raise ValueError("Unexpected nonzero USTAR padding")


def read_metadata(handle, name: str) -> tuple[bytes, int]:
    size = read_header(handle, name)
    data = read_exact(handle, size)
    pad_size = -size % tarfile.BLOCKSIZE
    padding(handle, pad_size)
    return data, tarfile.BLOCKSIZE + size + pad_size


def verify_payloads(handle, expected_sha: str, staging: Path | None) -> None:
    manifest, manifest_length = read_metadata(handle, MANIFEST)
    checksums, sums_length = read_metadata(handle, SUMS)
    hashes = read_checksums(checksums)
    if manifest != manifest_bytes(expected_sha, hashes):
        raise ValueError("Unexpected manifest or source SHA")
    offset = manifest_length + sums_length

    for name in PAYLOADS:
        if name == GUIDE:
            guide, guide_length = read_metadata(handle, name)
            if guide != README or hashlib.sha256(guide).hexdigest() != hashes[name]:
                raise ValueError("Unexpected testnet guide or guide checksum")
            if staging is not None:
                (staging / name).write_bytes(guide)
            offset += guide_length
            continue
        size = read_header(handle, name)
        target = (staging / name).open("xb") if staging is not None else nullcontext()
        with target as opened:
            if payload_digest(handle, name, size, opened) != hashes[name]:
                raise ValueError(f"Checksum mismatch: {name}")
        pad_size = -size % tarfile.BLOCKSIZE
        padding(handle, pad_size)
        offset += tarfile.BLOCKSIZE + size + pad_size

    end_markers = 2 * tarfile.BLOCKSIZE
    padding(handle, end_markers + (-(offset + end_markers) % tarfile.RECORDSIZE))
    if staging is not None:
        (staging / MANIFEST).write_bytes(manifest)
        (staging / SUMS).write_bytes(checksums)


class GzipReader:
    """Bound each read and reject concatenated members and trailing bytes."""

    def __init__(self, source, size: int):
        self.source = source
        self.decoder = zlib.decompressobj(wbits=31)
        self.pending = read_exact(source, len(GZIP_HEADER))
        if self.pending != GZIP_HEADER:
            raise ValueError("Invalid deterministic gzip header")
        self.remaining = size - len(self.pending)

    def read(self, size: int) -> bytes:
        data = bytearray()
        while len(data) < size and not self.decoder.eof:
            if not self.pending:
                self.pending = self.source.read(min(CHUNK, self.remaining))
                if not self.pending:
                    raise ValueError("Truncated gzip stream")
                self.remaining -= len(self.pending)
            data.extend(self.decoder.decompress(self.pending, size - len(data)))
            self.pending = self.decoder.unconsumed_tail
        return bytes(data)

    def finish(self) -> None:
        if self.read(1) or self.decoder.unused_data or self.remaining or self.source.read(1):
            raise ValueError("Unexpected data after end of archive")


def inspect_archive(path: Path, expected_sha: str, staging: Path | None) -> None:
    source, size = regular_file(path, MAX_ARCHIVE)
    with source:
        stream = GzipReader(source, size)
        verify_payloads(stream, expected_sha, staging)
        stream.finish()


def publish_directory(source: Path, destination: Path) -> None:
    # POSIX rename can replace an existing empty directory. Linux renameat2
    # publishes the entire verified directory or fails without replacement.
    try:
        renameat2 = ctypes.CDLL(None, use_errno=True).renameat2
    except AttributeError as error:
        raise OSError("Safe extraction requires Linux renameat2") from error
    renameat2.argtypes = (ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p, ctypes.c_uint)
    renameat2.restype = ctypes.c_int
    at_fdcwd = -100
    rename_noreplace = 1
    if renameat2(at_fdcwd, os.fsencode(source), at_fdcwd, os.fsencode(destination), rename_noreplace):
        error = ctypes.get_errno()
        raise OSError(error, os.strerror(error), destination)


def verify_archive(path: Path, expected_sha: str, extract_to: Path | None = None) -> None:
    source_sha(expected_sha)
    if extract_to is None:
        inspect_archive(path, expected_sha, None)
        return
    with tempfile.TemporaryDirectory(prefix=".aretovus-verify-", dir=extract_to.parent) as folder:
        staging = Path(folder)
        inspect_archive(path, expected_sha, staging)
        for name in (MANIFEST, SUMS, *PAYLOADS):
            (staging / name).chmod(entry_info(name, 0).mode)
        publish_directory(staging, extract_to)


def pack_archive(binaries: dict[str, Path], sha: str, output: Path) -> None:
    source_sha(sha)
    output.parent.mkdir(parents=True, exist_ok=True)
    with ExitStack() as stack:
        inputs = {}
        for name in BINARIES:
            handle, size = regular_file(binaries[name], MAX_EXE)
            inputs[name] = (stack.enter_context(handle), size)
        inputs[GUIDE] = (io.BytesIO(README), len(README))
        for name in NOTICES:
            handle, size = regular_file(SOURCE_ROOT / name, MAX_METADATA)
            inputs[name] = (stack.enter_context(handle), size)
        hashes = {
            name: payload_digest(handle, name, size) for name, (handle, size) in inputs.items()
        }
        manifest, sums = manifest_bytes(sha, hashes), checksums_bytes(hashes)
        with tempfile.NamedTemporaryFile(dir=output.parent, prefix=".aretovus-pack-") as temp:
            with gzip.GzipFile(filename="", fileobj=temp, mode="wb", mtime=0) as stream:
                with tarfile.open(mode="w|", fileobj=stream, format=tarfile.USTAR_FORMAT) as archive:
                    archive.addfile(entry_info(MANIFEST, len(manifest)), io.BytesIO(manifest))
                    archive.addfile(entry_info(SUMS, len(sums)), io.BytesIO(sums))
                    for name, (handle, size) in inputs.items():
                        handle.seek(0)
                        archive.addfile(entry_info(name, size), handle)
            temp.flush()
            verify_archive(Path(temp.name), sha)
            # Only fully verified bytes get published. A concurrent winner survives.
            os.link(temp.name, output)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    make = sub.add_parser("pack")
    for key in ("desktop", "bridge", "explorer", "node", "output"):
        make.add_argument(f"--{key}", type=Path, required=True)
    make.add_argument("--source-sha", required=True)
    check = sub.add_parser("verify")
    check.add_argument("--archive", type=Path, required=True)
    check.add_argument("--expected-sha", required=True)
    check.add_argument("--extract-to", type=Path)
    args = parser.parse_args()
    try:
        if args.action == "pack":
            binaries = dict(zip(BINARIES, (args.desktop, args.bridge, args.explorer, args.node)))
            pack_archive(binaries, args.source_sha, args.output)
            print(f"Packed and verified {args.output.name} from {args.source_sha}")
        else:
            verify_archive(args.archive, args.expected_sha, args.extract_to)
            print(f"Verified archive structure and checksums for {args.expected_sha}")
    except (OSError, ValueError, EOFError, tarfile.TarError, zlib.error) as error:
        print(f"Packaging failed: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

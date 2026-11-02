#!/usr/bin/env python3
"""Verify the exclusively ARETOVUS-branded active sources and fixed testnet IDs."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CRATES = ("core", "consensus", "storage", "node", "wallet")
OLD_BRAND = bytes.fromhex("6175726f7261")
OLD_TOKENS = [bytes.fromhex(x) for x in (
    "617572696478", "617572626c6b", "6175726b6579", "61757270616765",
    "4155524f52494458", "4155524f52534e50", "4155524f52504147",
    "4155524b455933", "4155524b455934",
)]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def audit_full_tracked_tree() -> None:
    """Review every tracked source, not only the Rust and workflow directories."""
    import re
    import subprocess

    listed = subprocess.check_output(["git", "ls-files", "-z"], cwd=ROOT)
    former_ticker = bytes.fromhex("415552").lower()
    original_credit = b"copyright (c) 2026 " + OLD_BRAND.upper().lower() + b" contributors"
    failures = []
    for item in listed.split(b"\0"):
        if not item:
            continue
        path = item.decode("utf-8")
        lower_path = item.lower()
        if OLD_BRAND in lower_path or any(token.lower() in lower_path for token in OLD_TOKENS):
            failures.append(f"outdated file path: {path}")
        historical = path.startswith(("docs/consolidation/", "tests/", "fuzz/seeds/"))
        historical = historical or (path.startswith("crates/") and "/tests/" in path)
        if historical:
            continue
        data = (ROOT / path).read_bytes().lower()
        if path in ("LICENSE", "LICENSE-MIT"):
            # Keep legally required original contributor attribution intact.
            assert original_credit in data, path
            data = data.replace(original_credit, b"")
        if OLD_BRAND in data or any(token.lower() in data for token in OLD_TOKENS):
            failures.append(f"old protocol or brand marker: {path}")
        if re.search(rb"(?<![a-z0-9_])" + former_ticker + rb"(?![a-z0-9_])", data):
            failures.append(f"old standalone ticker: {path}")
    assert not failures, "\n".join(failures)

    # A rename must not accidentally allow historical private-key exports into Git.
    for suffix in ("aretkey", bytes.fromhex("6175726b6579").decode("ascii")):
        status = subprocess.run(
            ["git", "check-ignore", "-q", "--no-index", "wallet." + suffix],
            cwd=ROOT, check=False
        )
        assert status.returncode == 0, "wallet key files must remain ignored"


def main() -> None:
    audit_full_tracked_tree()
    # NIST fixture guidance is actively maintained, even though test fixtures are historical.
    assert OLD_BRAND not in read("tests/vectors-fips204/README.md").lower().encode("utf-8")
    workspace = read("Cargo.toml")
    lockfile = read("Cargo.lock")
    for component in CRATES:
        crate = ROOT / "crates" / f"aretovus-{component}"
        assert crate.joinpath("Cargo.toml").is_file(), crate
        assert f"crates/aretovus-{component}" in workspace, component
        assert f'name = "aretovus-{component}"' in lockfile, component
        assert f'name = "aretovus-{component}"' in read(f"crates/aretovus-{component}/Cargo.toml")

    for path in (ROOT / "crates", ROOT / "scripts", ROOT / ".github" / "workflows"):
        for file in path.rglob("*"):
            if not file.is_file():
                continue
            raw = file.read_bytes()
            lowered = raw.lower()
            assert OLD_BRAND not in lowered, f"obsolete project name in {file.relative_to(ROOT)}"
            assert all(token.lower() not in lowered for token in OLD_TOKENS), file

    assert read("README.md").startswith("# ARETOVUS")
    # Legal history and copyright must not be erased in a branding change.
    assert read("LICENSE").startswith("ARETOVUS is available, at your option,")
    expected_legacy_credit = "Copyright (c) 2026 " + OLD_BRAND.decode("ascii").upper() + " contributors"
    assert expected_legacy_credit in read("LICENSE")
    assert expected_legacy_credit in read("LICENSE-MIT")
    assert 'src="assets/aretovus-logo.svg"' in read("README.md")
    assert 'src="../../assets/aretovus-logo.svg"' in read("docs/guide-fr/README.md")
    logo = read("assets/aretovus-logo.svg")
    assert logo.startswith('<svg xmlns="http://www.w3.org/2000/svg"')
    assert "ARETOVUS" in logo and "<script" not in logo.lower()
    assert "git clone https://github.com/antoineforterax-cmd/ARETOVUS.git" in read("CONTRIBUTING.md")
    assert "aretovus-node" in read("scripts/package_testnet.py")
    assert "aretovus-wallet" in read("scripts/package_testnet.py")

    tags = read("crates/aretovus-core/src/hash.rs")
    for marker in (
        "ARETOVUS/exp/v0/txid", "ARETOVUS/exp/v0/account-txid",
        "ARETOVUS/exp/v0/tx-signing", "ARETOVUS/exp/v0/block-header",
        "ARETOVUS/exp/v0/tx-list", "ARETOVUS/exp/v0/account-tx-list",
        "ARETOVUS/exp/v1/merkle-leaf", "ARETOVUS/exp/v1/merkle-node",
        "ARETOVUS/exp/v1/merkle-empty",
    ):
        assert marker in tags, marker
    storage = read("crates/aretovus-storage/src/lib.rs")
    for magic in ('b"ARETOSNP"', 'b"ARETOIDX"', 'b"ARETOPAG"', '"aretidx"', '"aretblk"', '"aretpage"'):
        assert magic in storage, magic
    keys = read("crates/aretovus-wallet/src/keys.rs")
    assert 'b"ARETOVK3"' in keys and 'b"ARETOVK4"' in keys
    assert 'dir.join(".aretovus.lock")' in read("crates/aretovus-node/src/main.rs")
    assert "rejects_foreign_or_unrecognized_data_without_overwriting" in read("crates/aretovus-node/src/main.rs")
    assert 'const PREFIX: &str = "aretovus://testnet/";' in read("crates/aretovus-wallet/src/uri.rs")
    assert 'const PREFIX: &str = "aretovus-testnet-recovery-v1:";' in read("crates/aretovus-wallet/src/recovery.rs")
    assert 'const HEADER: &str = "ARETOVUS-TESTNET-LIMITS-v1";' in read("crates/aretovus-wallet/src/policy.rs")
    assert "SEND ARETOVUS-TESTNET" in read("crates/aretovus-wallet/src/safety.rs")
    assert "ARETOVUS/testnet/native-ml-dsa-65-merkle-header/v4" in read("crates/aretovus-consensus/src/testnet.rs")
    assert "0x00, 0x28, 0x97, 0xe9" in read("crates/aretovus-consensus/src/testnet.rs")
    print("ARETOVUS native protocol identity static checks passed")


if __name__ == "__main__":
    main()

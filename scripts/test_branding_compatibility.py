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


def main() -> None:
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

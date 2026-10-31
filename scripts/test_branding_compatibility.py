#!/usr/bin/env python3
"""Verify the ARETOVUS public brand without altering the legacy testnet identity."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CRATES = ("core", "consensus", "storage", "node", "wallet")


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def main() -> None:
    manifests = ["Cargo.toml", "Cargo.lock", "fuzz/Cargo.toml"]
    for component in CRATES:
        crate = ROOT / "crates" / f"aretovus-{component}"
        assert crate.joinpath("Cargo.toml").is_file(), crate
        assert not ROOT.joinpath("crates", f"aurora-{component}").exists()
        manifests.append(f"crates/aretovus-{component}/Cargo.toml")
    for manifest in manifests:
        source = read(manifest)
        for component in CRATES:
            assert f'name = "aretovus-{component}"' in source or manifest != "Cargo.lock" and f"aretovus-{component}" in source or component not in ("node", "storage", "wallet") and manifest == "Cargo.toml", (manifest, component)
            assert f'name = "aurora-{component}"' not in source, (manifest, component)

    for path in ("README.md", "CONTRIBUTING.md", "SECURITY.md", "AGENTS.md"):
        source = read(path)
        assert "github.com/antoineforterax-cmd/AURORA" not in source, path
    assert read("README.md").startswith("# ARETOVUS")
    assert "git clone https://github.com/antoineforterax-cmd/ARETOVUS.git" in read("CONTRIBUTING.md")
    assert "aretovus-node" in read("scripts/package_testnet.py")
    assert "aretovus-wallet" in read("scripts/package_testnet.py")
    assert "aretovus-seed" in read("docs/TESTNET_GUIDE.md")
    assert "aretovus-soak-" in read("scripts/testnet_soak.py")
    assert "aretovus-soak-" in read(".github/workflows/testnet-soak.yml")
    assert "aretovus-soak-" in read(".github/workflows/massive-regression-matrix.yml")

    source_hash = read("crates/aretovus-core/src/hash.rs")
    for tag in (
        "AURORA/exp/v0/txid",
        "AURORA/exp/v0/account-txid",
        "AURORA/exp/v0/tx-signing",
        "AURORA/exp/v0/block-header",
        "AURORA/exp/v0/tx-list",
        "AURORA/exp/v0/account-tx-list",
        "AURORA/exp/v0/hybrid-account-id",
        "AURORA/exp/v0/post-quantum-account-id",
        "AURORA/exp/v1/merkle-leaf",
        "AURORA/exp/v1/merkle-node",
        "AURORA/exp/v1/merkle-empty",
    ):
        assert tag in source_hash, f"domain separator changed: {tag}"
    assert "AURORIDX" in read("crates/aretovus-storage/src/lib.rs")
    assert 'dir.join(".aurora.lock")' in read("crates/aretovus-node/src/main.rs")
    assert 'const PREFIX: &str = "aurora://testnet/";' in read("crates/aretovus-wallet/src/uri.rs")
    assert 'const PREFIX: &str = "aurora-testnet-recovery-v1:";' in read("crates/aretovus-wallet/src/recovery.rs")
    assert 'const HEADER: &str = "AURORA-TESTNET-LIMITS-v1";' in read("crates/aretovus-wallet/src/policy.rs")
    assert "SEND AURORA-TESTNET" in read("crates/aretovus-wallet/src/safety.rs")
    assert "AURORA/testnet/native-ml-dsa-65-merkle-header/v4" in read("crates/aretovus-consensus/src/testnet.rs")
    assert "require_expected_testnet_identity" in read("crates/aretovus-node/src/main.rs")
    assert "testnet_commands_reject_valid_snapshot_with_wrong_genesis" in read("crates/aretovus-node/src/main.rs")
    assert "OpenOptions::new().write(true).open(path)?" in read("crates/aretovus-storage/src/lib.rs")
    assert "cargo test --locked -p aretovus-storage" in read(".github/workflows/storage-windows-manifest-retry.yml")

    print("ARETOVUS branding and legacy testnet identity checks passed")


if __name__ == "__main__":
    main()

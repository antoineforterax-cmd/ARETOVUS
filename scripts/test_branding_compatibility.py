#!/usr/bin/env python3
"""Verify the ARETOVUS public brand without altering the legacy testnet identity."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CRATES = ("core", "consensus", "storage", "node", "wallet")


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def main() -> None:
    manifests = ["Cargo.toml", "Cargo.lock", "fuzz/Cargo.toml"]
    workspace = read("Cargo.toml")
    lockfile = read("Cargo.lock")
    for component in CRATES:
        crate = ROOT / "crates" / f"aretovus-{component}"
        assert crate.joinpath("Cargo.toml").is_file(), crate
        assert not ROOT.joinpath("crates", f"aretovus-{component}").exists()
        manifest = f"crates/aretovus-{component}/Cargo.toml"
        manifests.append(manifest)
        assert f"crates/aretovus-{component}" in workspace, component
        assert f'name = "aretovus-{component}"' in lockfile, component
        assert f'name = "aretovus-{component}"' in read(manifest), component
        assert "ARETOVUS" not in read(manifest), manifest
    for manifest in manifests:
        source = read(manifest)
        for component in CRATES:
            assert f"aretovus-{component}" not in source, (manifest, component)

    for path in ("README.md", "CONTRIBUTING.md", "SECURITY.md", "AGENTS.md"):
        source = read(path)
        assert "github.com/antoineforterax-cmd/ARETOVUS" not in source, path
    assert read("README.md").startswith("# ARETOVUS")
    assert 'src="assets/aretovus-logo.svg"' in read("README.md")
    assert 'src="../../assets/aretovus-logo.svg"' in read("docs/guide-fr/README.md")
    logo = read("assets/aretovus-logo.svg")
    assert logo.startswith('<svg xmlns="http://www.w3.org/2000/svg"')
    assert 'viewBox="0 0 1371 1536"' in logo
    assert 'ARETOVUS' in logo
    assert 'fill="#08090b"' in logo and 'fill="#f4f0e7"' in logo
    assert "<script" not in logo.lower() and "http://" not in logo.replace("http://www.w3.org/2000/svg", "")
    assert "git clone https://github.com/antoineforterax-cmd/ARETOVUS.git" in read("CONTRIBUTING.md")
    assert "aretovus-node" in read("scripts/package_testnet.py")
    assert "aretovus-wallet" in read("scripts/package_testnet.py")
    assert "aretovus-seed" in read("docs/TESTNET_GUIDE.md")
    assert "aretovus-soak-" in read("scripts/testnet_soak.py")
    assert "aretovus-soak-" in read(".github/workflows/testnet-soak.yml")
    assert "aretovus-soak-" in read(".github/workflows/massive-regression-matrix.yml")

    source_hash = read("crates/aretovus-core/src/hash.rs")
    for tag in (
        "ARETOVUS/exp/v0/txid",
        "ARETOVUS/exp/v0/account-txid",
        "ARETOVUS/exp/v0/tx-signing",
        "ARETOVUS/exp/v0/block-header",
        "ARETOVUS/exp/v0/tx-list",
        "ARETOVUS/exp/v0/account-tx-list",
        "ARETOVUS/exp/v0/hybrid-account-id",
        "ARETOVUS/exp/v0/post-quantum-account-id",
        "ARETOVUS/exp/v1/merkle-leaf",
        "ARETOVUS/exp/v1/merkle-node",
        "ARETOVUS/exp/v1/merkle-empty",
    ):
        assert tag in source_hash, f"domain separator changed: {tag}"
    assert "ARETOIDX" in read("crates/aretovus-storage/src/lib.rs")
    assert 'dir.join(".aretovus.lock")' in read("crates/aretovus-node/src/main.rs")
    assert 'const PREFIX: &str = "aretovus://testnet/";' in read("crates/aretovus-wallet/src/uri.rs")
    assert 'const PREFIX: &str = "aretovus-testnet-recovery-v1:";' in read("crates/aretovus-wallet/src/recovery.rs")
    assert 'const HEADER: &str = "ARETOVUS-TESTNET-LIMITS-v1";' in read("crates/aretovus-wallet/src/policy.rs")
    assert "SEND ARETOVUS-TESTNET" in read("crates/aretovus-wallet/src/safety.rs")
    assert "ARETOVUS/testnet/native-ml-dsa-65-merkle-header/v4" in read("crates/aretovus-consensus/src/testnet.rs")
    assert "require_expected_testnet_identity" in read("crates/aretovus-node/src/main.rs")
    assert "testnet_commands_reject_valid_snapshot_with_wrong_genesis" in read("crates/aretovus-node/src/main.rs")
    assert "OpenOptions::new().write(true).open(path)?" in read("crates/aretovus-storage/src/lib.rs")
    assert "cargo test --locked -p aretovus-storage" in read(".github/workflows/storage-windows-manifest-retry.yml")

    print("ARETOVUS branding and legacy testnet identity checks passed")


if __name__ == "__main__":
    main()

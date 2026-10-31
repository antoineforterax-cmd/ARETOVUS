#!/usr/bin/env python3
"""Prevent accidental protocol identity changes during the ARETOVUS brand migration."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CRATES = ("core", "consensus", "storage", "node", "wallet")


def main() -> None:
    manifests = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "fuzz" / "Cargo.toml"]
    for component in CRATES:
        crate = ROOT / "crates" / f"aretovus-{component}"
        assert crate.joinpath("Cargo.toml").exists(), crate
        assert not ROOT.joinpath("crates", f"aurora-{component}").exists()
        manifests.append(crate / "Cargo.toml")
    for manifest in manifests:
        source = manifest.read_text(encoding="utf-8")
        assert "aurora-core" not in source and "aurora-consensus" not in source
        for component in CRATES:
            assert f'name = "aurora-{component}"' not in source, (manifest, component)
    source_hash = (ROOT / "crates/aretovus-core/src/hash.rs").read_text(encoding="utf-8")
    for tag in ("AURORA/exp/v0/txid", "AURORA/exp/v0/account-txid", "AURORA/exp/v1/merkle-leaf"):
        assert tag in source_hash, f"legacy cryptographic domain separator changed: {tag}"
    storage = (ROOT / "crates/aretovus-storage/src/lib.rs").read_text(encoding="utf-8")
    assert "AURORIDX" in storage, "storage format magic must remain unchanged"
    scheme = (ROOT / "crates/aretovus-wallet/src/uri.rs").read_text(encoding="utf-8")
    assert '"aurora://testnet/"' in scheme, "legacy payment URI must remain compatible"
    genesis = (ROOT / "crates/aretovus-consensus/src/testnet.rs").read_text(encoding="utf-8")
    assert "AURORA/testnet/native-ml-dsa-65-merkle-header/v4" in genesis
    print("ARETOVUS rebrand manifest and preserved protocol identity checks passed")


if __name__ == "__main__":
    main()

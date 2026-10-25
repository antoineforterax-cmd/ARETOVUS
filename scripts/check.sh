#!/usr/bin/env bash
# Vérifications locales identiques à la CI. Arrête au premier échec.
set -euo pipefail

echo "== 1/4 formatage"
cargo fmt --all -- --check

echo "== 2/4 clippy (avertissements = erreurs)"
cargo clippy --workspace --all-targets -- -D warnings

echo "== 3/4 compilation"
cargo build --workspace

echo "== 4/4 tests"
cargo test --workspace

echo "Tout est passé."

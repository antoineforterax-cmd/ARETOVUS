# Fuzzing AURORA

The main CI runs a deterministic malformed-input smoke test against transaction, block, header, block-inventory and peer-list decoders. It is a quick regression check, not a substitute for coverage-guided fuzzing.

The separate `fuzz/` workspace contains libFuzzer targets for canonical transaction/block/header decoding and Merkle proof verification. GitHub Actions runs each target for 30 seconds on Linux for pull requests and manual dispatches.

## Local run

The libFuzzer toolchain requires nightly Rust, a C++ compiler, and cargo-fuzz. The supported runner in CI is Linux:

```sh
rustup toolchain install nightly --profile minimal
cargo +stable install cargo-fuzz --version 0.13.2 --locked
cargo +nightly fuzz run canonical_decode -- -max_total_time=60
cargo +nightly fuzz run merkle_proofs -- -max_total_time=60
```

Corpus and crash artifacts are written under `fuzz/corpus/` and `fuzz/artifacts/`. Preserve minimized crash inputs as regression cases and fix the underlying decoder before continuing. CI time limits are deliberately short; release candidates need longer campaigns and an independent review of any findings.

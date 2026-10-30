# AURORA — integrated testnet validation gates (31 October 2026)

This is a **status snapshot**, not a security certification. Check GitHub Actions again before using these results: a workflow that is in progress must not be counted as successful.

## Integrated changes

- PR #79: Windows packaged two-node wallet payment/SPV and rejected dry-run payment smoke. Merged as `ffce28b4d9b20704ddeb2d6778a46dd4dee480a6`.
- PR #82: slow/trickling P2P session reclamation and admission probe. Merged as `101447d6de732f245a7de84d6e5d073079758958`.
- PR #83: storage phase profiling at 2,048 and 8,192 blocks on the same runner. Merged as `7633c16f66ca6942da8dc91012ebac5edf759af2`.

The three PR heads had successful targeted CI before merge. These runs **do not** validate the combined `main` tree.

## Required post-merge evidence

1. Pin `main` SHA `7633c16f66ca6942da8dc91012ebac5edf759af2` (or record a newer SHA explicitly). Confirm that the full Rust CI run for that exact SHA completed successfully.
2. Confirm the packaged Windows, Linux and macOS jobs after integration, including the Windows two-node wallet/SPV smoke and rejected-payment assertions.
3. Re-run P2P admission and trickle saturation against the integrated binary, including wallet responsiveness, honest peer sync and capacity recovery after hostile clients close.
4. Execute storage profiles at 2,048 and 8,192 blocks; retain separate logs, resource measurements and restored height/tip/block-count assertions. These are synthetic local Regtest storage measurements, **not** a 100k-block network sync or disk-full test.
5. Record any failing job, its exact SHA, reproducer and fix in a separate PR. Do not silently change consensus or economic rules.

## Still unverified for durable public testnet / mainnet

- Two physical machines across real network conditions, NAT, clock skew and prolonged partitions.
- Real network synchronization beyond 100,000 blocks, not merely local storage/reload.
- Actual disk-full, interrupted power, off-host restore and backup-corruption exercises.
- Completed and evidenced 72-hour endurance run.
- Independent human audit of consensus, cryptography, wallet, P2P and storage.

**Release gate:** Do not claim production security or recommend real-value funds until the relevant evidence and independent review are complete. Historical checklists may describe already-merged PRs as drafts; use the live PR state and exact commit hashes above instead.

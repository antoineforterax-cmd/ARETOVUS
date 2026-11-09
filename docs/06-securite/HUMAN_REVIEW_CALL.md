# Draft for users.rust-lang.org and GitHub Discussions

Status: prepared, not posted. Replace the review commit only after the corresponding CI checks have finished, and retain the exact commit in both posts.

## Title

Seeking a Rust code review of an experimental blockchain: consensus, storage and bounded TCP peers

## Post

I'm working on ARETOVUS, a small experimental blockchain in Rust. The repository is public, and I'd like help finding design and implementation mistakes before extending it further.

Repository: https://github.com/antoineforterax-cmd/ARETOVUS

Work in progress: https://github.com/antoineforterax-cmd/ARETOVUS/pull/2

The review baseline is commit `7d0f1ee598235b7194f92081424c008c4c628ceb`. Later documentation and test-harness changes are outside this baseline until explicitly added to the review. The review brief is `docs/INDEPENDENT_REVIEW.md` at that revision.

Most of the code was generated and revised with AI assistance over a very short period. It has automated tests and some manual testnet runs, and I've also personally reviewed and corrected code, including Python work. What is still missing is a documented independent security review by someone with relevant skills. I am not presenting it as suitable for real funds, and this is not a token sale announcement.

The workspace has five crates: transaction/account primitives, consensus, storage, a TCP node, and a CLI wallet. It uses native ML-DSA-65 transaction signatures, a simple SHA-256 proof of work, cumulative-work fork choice and paginated persistent block storage.

I would especially welcome a review of:

- Validation boundaries, deterministic encoding and transaction state changes.
- Difficulty adjustment, timestamps, cumulative work and reorganisation accounting.
- Crash recovery, corrupt storage handling and migration between storage formats.
- Resource limits and denial-of-service risks in peer synchronisation.

There are known limitations, including replay and cloning costs, unfinished operational rules for the treasury, and no demonstrated multi-host endurance run. The test reports and previous AI counter-review are in `docs/`. The repository's `PROVENANCE.md` on the feature branch explains how the code was produced and checked.

Students, independent developers and skilled hobbyists are welcome; you do not need to work for a security firm. A focused review of one module, a reproducible failure or a test exposing a mistake would already help. Please include the commit and commands you used. For a potentially exploitable issue, please use the confidential reporting route in `SECURITY.md`; if private reporting is unavailable, ask for a confidential contact without posting exploit details publicly.

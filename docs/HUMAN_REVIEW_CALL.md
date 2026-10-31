# Appel à revue Rust externe — modèle à personnaliser

**Statut : brouillon non publié.** Ce texte est un modèle d'invitation, **pas** une annonce de revue réussie. Avant publication, relever `git rev-parse HEAD` sur la branche exacte, conserver le SHA complet, vérifier la CI du **même** SHA et actualiser les risques à partir du rapport d'essais. Ne pas réutiliser les anciennes références à la PR #2 et au commit `7d0f1ee` comme s'ils décrivaient `main`.

## Title

Seeking an independent Rust code review for AURORA, an experimental blockchain testnet

## Post

I'm maintaining [AURORA](https://github.com/antoineforterax-cmd/AURORA), an experimental Rust blockchain testnet. I am seeking focused, reproducible reviews of its implementation before extending its capabilities or considering any deployment involving real funds.

**Exact code revision to review:** `<REPLACE_WITH_40_CHAR_COMMIT_SHA>` — **replace before posting**. The linked [review brief](https://github.com/antoineforterax-cmd/AURORA/blob/main/docs/INDEPENDENT_REVIEW.md) and [validation report](https://github.com/antoineforterax-cmd/AURORA/blob/main/docs/TESTNET_VALIDATION_REPORT.md) should be read with their historical dates in mind.

The workspace has five crates covering account/transaction primitives, consensus and PoW, persistent storage, a bounded TCP P2P node, and a CLI wallet. Native ML-DSA-65 testnet signatures are implemented. The node and wallet remain **experimental and unaudited by an independent human reviewer**; please do not use real funds.

The project has used AI-assisted implementation and code review. We document this in [PROVENANCE.md](https://github.com/antoineforterax-cmd/AURORA/blob/main/PROVENANCE.md). Automated CI and tests support specific claims but are not substitutes for a professional or volunteer independent audit.

I'd value focused reviews of:
- Deterministic encoding, signatures, money conservation and state changes.
- Difficulty, timestamps, cumulative-work fork choice, reorgs and emission.
- Storage corruption, write interruption, recovery and migrations.
- P2P/RPC parsing, resource limits, sync and availability.
- Wallet secret management, recovery, SPV trust assumptions and transaction intent.

A minimal reproducible bug, an executable negative test or a careful code review of one subsystem would be highly appreciated. Please include the source SHA, commands, conditions, expected/actual outcome, and remaining uncertainties. References to old PRs or tests must be clearly dated.

For a potentially exploitable security defect, use [SECURITY.md](https://github.com/antoineforterax-cmd/AURORA/blob/main/SECURITY.md) and request a confidential route; do not publish working exploits or secrets in a public issue. No bounty or paid engagement is promised.

Contributors are welcome under the project's [MIT OR Apache-2.0 dual licensing terms](https://github.com/antoineforterax-cmd/AURORA/blob/main/CONTRIBUTING.md). The three license files are intentionally present; they do not indicate three different licensing schemes.

Thank you for taking a rigorous look at the code!

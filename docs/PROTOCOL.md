# AURORA Protocol — Phase 0 Draft

**Status:** design proposal; not a consensus specification  
**Version:** 0.1-draft  
**Network:** none — this document does not define a deployable network  
**Security status:** experimental; unaudited; never use real funds

> This document organizes the questions that must be answered before AURORA can be considered a functioning blockchain. It does not silently validate any open item in `docs/DECISION_REGISTER.md`. Only the project owner can mark a decision VALIDATED, and only after the evidence required by that register exists.

## 1. Purpose

AURORA is an experimental project to learn how to design, implement, test, and evaluate an independent blockchain in Rust. The objective is not to copy Bitcoin or Ethereum feature-for-feature, but to establish measurable requirements and compare alternatives before committing to protocol rules.

The initial implementation must prioritize correctness, reproducibility, bounded resource use, and clear failure modes over performance claims or feature count.

## 2. Scope and non-goals

### Experimental account-transfer profile (branch `feature/emission-schedule-v0`)

The current branch implements an explicitly experimental account-transfer path. For this profile,
`sender` is the raw 32-byte Ed25519 public key and `auth_data` is exactly 64 bytes containing an
Ed25519 signature over the 32-byte `signing_digest`. The digest commits to version, network,
sender, nonce, recipient, amount, and fee in canonical encoding, under the distinct
`AURORA/exp/v0/tx-signing` hash domain. The transaction identifier continues to include the full
canonical encoding, including the signature. The maintained `ed25519-dalek` crate performs strict
signature verification before any ledger mutation.

This profile exists to make a local experimental node coherent; it does not settle the repository's
open UTXO-vs-account decision or define user-facing addresses. Keys, wire format, economics, block
state validation, difficulty adjustment, and chain reorganization remain experimental or incomplete.
Do not use it for real funds or describe it as production-ready.

### In scope for the research phase

- Transaction and state models, including a comparison of UTXO and account-based designs.
- Canonical encoding, cryptographic hashes, transaction identifiers, and commitments.
- Authentication and signature verification, including a benchmark of classical and post-quantum candidates.
- Proof-of-Work (PoW), target representation, difficulty adjustment, and cumulative-chain-work rules.
- Block validation, state transitions, fees, issuance, and overflow protection.
- Peer-to-peer networking, denial-of-service controls, storage, reorganization handling, and node recovery.
- Wallet key management, address design, backup and recovery trade-offs.
- Privacy analysis, governance assumptions, test strategy, reproducible benchmarks, and a threat model.

### Out of scope for the first prototype

- Real-money use, exchange listings, price promises, or claims that mining will be profitable.
- Mainnet launch before a functioning testnet, independent review, and security testing.
- Marketing claims such as “more secure than Bitcoin,” “untraceable,” or “quantum-proof” without defined threat models and evidence.
- Remote computer control or unrelated applications embedded in consensus. Any such product would require a separate design and threat model.

## 3. Design principles

1. **Determinism:** independent correct nodes must compute the same result from the same valid chain and rules.
2. **Explicit rules:** consensus-critical behaviour must be specified, versioned, and covered by tests.
3. **Canonical bytes:** the same logical object must not have multiple valid encodings.
4. **Safe arithmetic:** monetary values use integer units; overflow, underflow, invalid ranges, and excessive values must be rejected.
5. **Domain separation:** hashes used for different purposes must not be accidentally interchangeable.
6. **Bounded work:** parsing, validation, memory use, block size, transaction count, and peer requests need explicit limits.
7. **Independent verification:** a node must validate received data rather than trust a miner, peer, explorer, or wallet.
8. **Evidence before claims:** tests, benchmarks, threat analysis, and independent review are required; generated code is not presumed secure.
9. **Network separation:** testnet and any future production network must not accept each other's transactions or blocks by accident.
10. **Conservative upgrades:** changes to consensus rules require explicit compatibility analysis and a documented activation plan.

## 4. Current prototype versus intended protocol

The existing repository is a skeleton, not a complete blockchain. Its current code includes experimental integer amounts, serialization and hash helpers, an experimental block/transaction commitment, and a node that warns that it is not a real node. The current transaction commitment is not a Merkle tree.

At present there is no complete PoW, peer-to-peer network, wallet, signature verification, persistent chain state, complete double-spend prevention, block reward, fee market, or reorganization logic. Authentication data that is not actually verified must never be treated as proof of ownership.

Nothing in this draft changes those limitations.

## 5. Proposed development phases

### Phase 0 — Specification and research

- Inventory the existing code and document each assumption.
- Compare transaction/state models and cryptographic options.
- Define attacker capabilities, safety properties, and measurable acceptance criteria.
- Keep unresolved choices in the decision register.

**Exit criteria:** each consensus-critical choice has a rationale, alternatives, test plan, and an owner decision; unresolved choices remain explicitly open.

### Phase 1 — Deterministic core

- Define canonical types and encoding.
- Implement checked monetary arithmetic and domain-separated hashes.
- Implement signature verification only after the authentication format is selected.
- Add unit tests, property tests, fuzz targets, and known-answer vectors where applicable.

**Exit criteria:** reproducible test suite; malformed inputs rejected; no unchecked assumptions about user-controlled data.

### Phase 2 — State and transaction validation

- Select and specify UTXO or account state.
- Define transaction validity, replay protection, fee calculation, and double-spend prevention.
- Define the state commitment and block transaction commitment.
- Test invalid, duplicate, reordered, and adversarial transactions.

### Phase 3 — Consensus and local chain

- Specify block headers, target encoding, PoW validation, difficulty adjustment, chain selection by cumulative work, timestamps, and reorganization rules.
- Add reward/issuance only after economic modelling and consensus rules are reviewed.
- Test boundary conditions, timestamp manipulation, withheld blocks, and competing chains.

### Phase 4 — Testnet and networking

- Add peer discovery and message framing with strict size/time limits.
- Validate all remote data and rate-limit expensive operations.
- Test partitions, reconnects, peer churn, malformed messages, spam, and denial-of-service cases.
- Publish a testnet with no real monetary value and clear reset/upgrade policies.

### Phase 5 — Wallet, usability, and privacy

- Design key creation, secure storage, backups, addresses, fee estimates, and transaction confirmation.
- Evaluate hardware-wallet integration against supported standards and vendor constraints.
- Evaluate privacy features against explicit adversaries and legal/operational trade-offs.
- Treat lost-key recovery as a separate opt-in trust model, not a hidden administrator backdoor.

### Phase 6 — External review before any production launch

- Independent code and protocol review.
- Security audit and remediation of findings.
- Reproducible load, stress, and recovery tests.
- Upgrade, incident response, key compromise, and disclosure procedures.
- Explicit project-owner go/no-go decision. A successful CI run alone is not an audit.

## 6. Open protocol decisions

No item below is validated by this document.

| Area | Questions to resolve | Evidence required |
|---|---|---|
| State model | UTXO, account model, or another design? | Complexity, parallelism, privacy, and attack-surface comparison |
| Encoding | Fixed-width binary, length-prefixed fields, or another canonical format? | Vectors, parser tests, compatibility review |
| Hashes | SHA-256 construction and domain tags; where double hashing is useful or not | Known-answer tests and protocol analysis |
| Transaction ID | Which fields are committed, including authentication data? | Malleability and replay analysis |
| Signatures | Classical baseline and post-quantum options such as ML-DSA or SLH-DSA | Security-level comparison, signature/key sizes, verification and signing benchmarks |
| Commitments | Merkle tree design, empty-tree rules, odd-leaf rules, and proof format | Vectors and edge-case tests |
| PoW | SHA-256 versus memory-hard alternatives | Hardware centralization, energy, implementation, and attack analysis |
| Difficulty | Retarget method, manipulation resistance, startup behaviour | Simulation over varied hash-rate and adversarial scenarios |
| Timing | Target block interval and timestamp constraints | Simulation and usability/security trade-off |
| Monetary units | Decimal places, maximum supply, issuance curve, fees, and subsidy schedule | Economic model, overflow tests, long-horizon simulations |
| Block limits | Byte/weight limits, transaction count, execution cost | Adversarial resource tests and hardware measurements |
| Chain selection | Cumulative-work rules and tie handling | Reorganization and fork simulations |
| Addresses | Checksum, network prefix, versioning, and error detection | Human-error and interoperability tests |
| Governance | Upgrade proposal, activation, emergency response, and decision rights | Threat model and transparent procedures |

Existing repository hypotheses (including eight decimal places and a possible 105 million AUR cap) remain hypotheses until separately justified and approved. They must not be treated as settled monetary policy.

## 7. Mining and difficulty: design requirements

PoW difficulty is a threshold that blocks must satisfy; it is not a fixed measure of how “secure” a coin is. Difficulty should respond to observed block production according to a specified algorithm. Network security also depends on honest hash power, its distribution, incentives, connectivity, and attack cost.

The design must not promise that an individual miner will earn a “good amount” before difficulty rises. If the network has little hash power, blocks may be cheap to find but the network can also be easier to attack. Early coins may have no market value, and a later price is unknowable.

Before testnet mining, implement a local regtest/development mode with deliberately easy, clearly non-production difficulty. For any future public testnet, publish the rules and make clear that its coins have no guaranteed value. For a future production network, measure expected block time, difficulty adjustment, hash-rate concentration, and attack scenarios rather than selecting an arbitrary difficulty “harder than Bitcoin.”

Any mining reward, supply cap, pre-mine, founder allocation, or early-miner advantage must be disclosed and modelled before consensus rules are finalized. No private shortcut or secret easier difficulty may exist for a founder.

## 8. Candidate differentiation: hypotheses, not slogans

AURORA does not yet have a demonstrated advantage over Bitcoin or Ethereum. Differentiation must come from measurable trade-offs, not from accumulating features.

Candidate areas to investigate:

- **Post-quantum readiness:** benchmark standardized signature schemes and consider hybrid migration plans. Larger signatures and keys may increase bandwidth, storage, and fees; no scheme should be described as “quantum-proof.”
- **Low fees and useful throughput:** measure total node resource costs, propagation, state growth, and congestion behaviour. A low fee alone can make spam cheaper.
- **Privacy:** specify what observers can infer and which adversaries are considered. Privacy claims require dedicated analysis.
- **Recovery:** compare self-custody with opt-in social or delayed recovery. Any recovery path creates additional trust, collusion, coercion, and attack risks.
- **Anti-concentration:** investigate mining-pool and validator concentration using transparent metrics. Arbitrary penalties based on wallet size or “whale” status can be unfair, gameable, and harmful to credible neutrality.
- **Wallet/hardware compatibility:** define supported signing standards and integration requirements rather than assuming compatibility.
- **Networking efficiency:** benchmark propagation and resource use under realistic and adversarial conditions.

For each candidate, record the problem, baseline, proposed mechanism, measurable success threshold, costs, attack paths, and a kill criterion if evidence is poor.

## 9. Threat model baseline

Assume an attacker may:
- send malformed, oversized, duplicated, delayed, or strategically ordered messages;
- create many identities or peers, disconnect selectively, and partition parts of the network;
- control some fraction of mining hash power and vary it over time;
- attempt double spends, transaction replay, timestamp manipulation, and chain reorganizations;
- exploit parser bugs, integer overflows, resource exhaustion, dependency vulnerabilities, and unsafe key handling;
- socially engineer users, steal backups, or compromise endpoints.

The design must state what it does **not** defend against, including sufficiently powerful majority-hash attacks, compromised user devices, lost keys without recovery, and failures in external infrastructure. Risk scores and mitigations must be added and tested; this baseline is not proof that these risks are mitigated.

## 10. Required quality gates

Before a feature is called complete, require relevant items from this list:

- Unit and integration tests, including invalid inputs and boundary cases.
- Property-based tests and fuzzing for parsers, serialization, and state transitions.
- Known-answer vectors for cryptographic operations and canonical encodings.
- Reproducible benchmarks with hardware, compiler, configuration, and dataset documented.
- Dependency and supply-chain review; pinned and audited dependencies where practical.
- No secrets or private keys in logs, test fixtures, commits, or telemetry.
- Consensus changes documented in the decision register with compatibility impact.
- CI passing on a clean checkout, with no claim that CI proves protocol security.
- Independent review for cryptography, consensus, wallet security, and any production release.

## 11. Working agreement and decision status

The assistant may research, draft, implement, test, and recommend. The project owner retains the decision to validate protocol choices. An item should only be marked VALIDATED when the decision register's evidence requirements are satisfied and the owner explicitly approves it.

The current phase is **Phase 0 — Specification and research**. This draft is a roadmap, not a promise of completion, a guarantee of mining returns, or authorization to use real funds.

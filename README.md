# AURORA — prototype expérimental

> **Statut : base expérimentale. Ce n'est PAS une blockchain utilisable, PAS sécurisée, PAS auditée.**
> Aucune preuve de travail, aucun réseau, aucun portefeuille, aucune signature. Ne jamais utiliser
> avec de vrais fonds. Toutes les règles ici sont des **hypothèses** listées dans
> `docs/DECISION_REGISTER.md` (annexe B) ; aucune décision n'est validée.

## Ce que contient ce dépôt (v0.0.1)

| Crate | Contenu actuel |
|---|---|
| `aurora-core` | montants entiers, sérialisation canonique, hachage SHA-256 avec domaines, transaction expérimentale |
| `aurora-consensus` | bloc expérimental, engagement sur les transactions (pas un Merkle), validation **sans état** |
| `aurora-node` | exécutable qui affiche un avertissement, rien d'autre |
| `aurora-wallet` | crate vide et réservée (aucune clé) |

Le stockage sera extrait en premier dans une crate indépendante quand sa complexité et ses tests le
justifieront (pas avant).

## Commandes

Prérequis : Rust stable (https://rustup.rs) et accès à crates.io pour télécharger `sha2`.

```bash
cargo fmt --all            # première fois : formate le code (le code a été écrit sans rustfmt)
./scripts/check.sh         # fmt --check + clippy + build + tests
cargo test --workspace     # tests seuls
cargo run -p aurora-node   # affiche l'avertissement
python3 scripts/reference_vectors.py   # recalcule les valeurs attendues des vecteurs (Python indépendant)
```

Après le premier build réussi : committer `Cargo.lock` et figer la version de Rust dans
`rust-toolchain.toml` (GOV-01).

## Dépendances

| Dépendance | Rôle | Pourquoi |
|---|---|---|
| `sha2` 0.10 (RustCrypto) | SHA-256 | Évite toute primitive cryptographique maison. À re-vérifier : maintenance, version, `cargo audit`. |

Aucune autre dépendance. Les « tests de propriétés » utilisent un générateur maison déterministe
(non cryptographique, tests uniquement) ; `proptest` et `cargo-fuzz` sont prévus (TST-03).

## Règles de travail (GOV-04)

- Le code généré par IA est du code **non vérifié** tant qu'il n'a pas été compilé, testé et relu.
- Aucune règle de consensus, de politique monétaire ou de cryptographie n'est ajoutée ou changée
  sans décision du porteur dans `docs/DECISION_REGISTER.md`.
- Aucune clé, aucun secret dans le dépôt (`.gitignore` exclut `.env`, `*.key`, `*.pem`, `*.seed`).
- Distinguer toujours : tests **exécutés**, tests **écrits mais non exécutés**, tests **prévus**.

## Documentation

- `docs/DECISION_REGISTER.md` : décisions, statuts, justifications, tests requis, hypothèses du code.
- `docs/CONSENSUS.md` : état des règles de consensus (aucune validée).
- `docs/THREAT_MODEL.md` : menaces et mesures, sans score tant que la méthode n'est pas validée.

# AURORA — prototype expérimental

> **Statut : base expérimentale. Ce n'est PAS une blockchain utilisable, PAS sécurisée, PAS auditée.**
> Le prototype inclut une vérification Ed25519 de transferts par comptes et des snapshots de chaîne validés, mais aucun réseau P2P ni portefeuille. Ne jamais utiliser
> avec de vrais fonds. Toutes les règles ici sont des **hypothèses** listées dans
> `docs/DECISION_REGISTER.md` (annexe B) ; aucune décision n'est validée.

## Ce que contient ce dépôt (v0.0.1)

| Crate | Contenu actuel |
|---|---|
| `aurora-core` | montants entiers, sérialisation canonique, SHA-256 à domaines, transferts par comptes et vérification Ed25519 expérimentale |
| `aurora-consensus` | blocs de comptes signés, application atomique, retarget expérimental et sélection par travail cumulé |
| `aurora-storage` | snapshots versionnés et contrôlés par checksum de toutes les branches acceptées ; reconstruction par rejeu |
| `aurora-node` | commande locale regtest, snapshots optionnels avec `--data-dir`, aucun service réseau ni minage |
| `aurora-wallet` | crate vide et réservée (aucune clé) |

La persistance conserve les blocs acceptés de toutes les branches et les rejoue au redémarrage; le format reste un snapshot simple, pas une base optimisée. Le profil fixe de regtest et les paramètres de retarget sont enregistrés afin de rejouer les mêmes contrôles après redémarrage. La règle de retarget est expérimentale, par fenêtres et par pas entiers bornés; elle n’est pas une règle validée pour réseau public. Aucun reward de bloc n’est encore appliqué.

## Commandes

Prérequis : Rust stable (https://rustup.rs) et accès à crates.io pour télécharger les dépendances (`sha2`, `ed25519-dalek`).

```bash
cargo fmt --all            # première fois : formate le code (le code a été écrit sans rustfmt)
./scripts/check.sh         # fmt --check + clippy + build + tests
cargo test --workspace     # tests seuls
cargo run -p aurora-node -- regtest  # valide la genèse locale en mémoire
cargo run -p aurora-node -- regtest --data-dir ./aurora-regtest-data  # charge ou sauvegarde un snapshot
python3 scripts/reference_vectors.py   # recalcule les valeurs attendues des vecteurs (Python indépendant)
```

`aurora-node regtest --data-dir` crée le snapshot de genèse s’il n’existe pas, puis revalide les blocs enregistrés au redémarrage. Ce mode ne démarre aucun pair et n’ajoute pas de blocs.

## Persistance expérimentale

Le format `AURORSNP` version 2 contient les paramètres de difficulté, tous les blocs acceptés en ordre topologique, leurs encodages canoniques et un SHA-256 de contrôle. Chaque snapshot immuable est écrit sous un nom déterminé par la hauteur, l’empreinte de tête et le checksum. Au chargement, les blocs sont rejoués et revalidés depuis la genèse avec l’état initial fourni par le réseau. Les snapshots corrompus ou de version inconnue sont rejetés. Il reste à éprouver les pannes, sauvegardes et migrations; ce format ne remplace pas une base de données optimisée.

## Dépendances

| Dépendance | Rôle | Pourquoi |
|---|---|---|
| `sha2` 0.10 (RustCrypto) | SHA-256 | Évite toute primitive cryptographique maison. À re-vérifier : maintenance, version, `cargo audit`. |
| `ed25519-dalek` 2.x | Signature Ed25519 stricte | Vérification de signature via une bibliothèque maintenue ; compte identifié par sa clé publique brute de 32 octets. |

Les « tests de propriétés » utilisent un générateur maison déterministe
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

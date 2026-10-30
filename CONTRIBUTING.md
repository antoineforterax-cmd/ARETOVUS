# Contribuer à AURORA

Pour tout développement assisté par IA, lire d'abord [AGENTS.md](AGENTS.md) : ordre de travail, cartographie des PR, tests de sécurité adversariaux et règles avant fusion. Les décisions de protocole restent sous le contrôle du responsable du projet.

Le travail courant est sur `feature/emission-schedule-v0`, PR #2. Ne partez pas du principe que `main` contient ce testnet. Commencez par lire [ARCHITECTURE.md](ARCHITECTURE.md), [PROVENANCE.md](PROVENANCE.md) et les [règles de consensus](docs/CONSENSUS.md).

## Compiler et vérifier

Installer Rust stable, Cargo et Python 3.12 ou plus récent. Sous Windows, compiler avec la cible MSVC demande Visual C++ Build Tools et le Windows SDK; les binaires publiés n’exigent pas cet environnement de compilation.

```sh
git clone https://github.com/antoineforterax-cmd/AURORA.git
cd AURORA
git switch feature/emission-schedule-v0
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
python scripts/testnet_smoke.py
```

Le test de volume est séparé car il écrit plus de 100 000 fichiers de blocs :

```sh
cargo test --locked -p aurora-storage --lib volume_above_100k_blocks_with_side_branch_and_restart -- --ignored --nocapture
```

Il couvre le stockage et le rejeu; il ne constitue pas un essai réseau de 72 heures. Les campagnes libFuzzer sont décrites dans [FUZZING.md](docs/FUZZING.md).

## Proposer une modification

Ouvrir une issue avec un scénario précis, ou choisir une `good first issue`. Faire une branche, ajouter un test qui reproduit le problème, puis proposer une PR vers la branche de travail. Décrire le comportement avant/après, le coût en ressources, les résultats des tests et les effets sur les anciennes données. Ne pas modifier silencieusement genèse, émission, sérialisation, signatures ou règles de réorganisation. Une modification de protocole demande une spécification et une décision de compatibilité.

Ne jamais ajouter de clé privée, mot de passe, jeton ou dossier de nœud. Les fins de ligne sont LF. Les contributions sont proposées sous `MIT OR Apache-2.0`; les licences des dépendances et leurs avis restent applicables. Décrire les outils et méthodes utiles à la relecture, puis vérifier chaque modification proposée.

## Périmètre

Priorité au cœur, aux preuves de fonctionnement, à la documentation et à l’exploitation du testnet. Explorateur, site, vente simulée, interfaces graphiques et mobile sont reportés aux repreneurs. Aucun fonds réel, promesse de rendement ou déclaration de sécurité mainnet. Pour une faille exploitable, suivre [SECURITY.md](SECURITY.md) plutôt que publier les détails dans une issue.

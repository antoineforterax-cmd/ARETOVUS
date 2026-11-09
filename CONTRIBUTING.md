# Contribuer à ARETOVUS\n\n**Nouveau sur le projet ?** Suivez le [parcours de première contribution avec ou sans IA](docs/CONTRIBUER_AVEC_IA.md), puis l'[état actuel et les limites](docs/ETAT_ACTUEL.md). Pour la réutilisation, les forks et l'attribution, voir [licences et droits](docs/LICENCES_ET_DROITS.md). Les documents anciens sont indexés dans [les archives](docs/ARCHIVES_ET_RAPPORTS.md).

Pour tout développement assisté par IA, lire d'abord [AGENTS.md](AGENTS.md) : ordre de travail, cartographie des PR, tests de sécurité adversariaux et règles avant fusion. Les décisions de protocole restent sous le contrôle du responsable du projet.

Le testnet expérimental est désormais intégré à `main`. Les branches historiques et les PR anciennes ne doivent pas être considérées comme la référence courante ; vérifier le SHA de `main` et ses contrôles GitHub Actions avant de commencer. Lire [ARCHITECTURE.md](ARCHITECTURE.md), [PROVENANCE.md](PROVENANCE.md), les [règles de consensus](docs/CONSENSUS.md) et les [portes de validation](docs/POST_MERGE_VALIDATION_GATES_2026-10-31.md).

## Si vous débutez (avec ou sans IA)

Commencez par la [carte de documentation](docs/README.md) puis le [guide français](docs/guide-fr/README.md) ; ce dernier décrit une ancienne release et doit être confronté au SHA que vous testez. Vous pouvez apporter une correction de documentation, un test de non-régression ou une amélioration isolée sans maîtriser tout le consensus.

Pour une contribution assistée par IA, **expliquez ce que l'outil a proposé, ce que vous avez vous-même vérifié et ce qui reste non testé**. Ne présentez jamais une relecture d'IA comme un audit humain indépendant. Préférez une petite PR reproductible à une refonte massive non relue. Les changements de consensus, de cryptographie, de garde des clés ou d'économie nécessitent une décision et une revue spécifiques.

## Compiler et vérifier

Installer Rust stable, Cargo et Python 3.12 ou plus récent. Sous Windows, compiler avec la cible MSVC demande Visual C++ Build Tools et le Windows SDK; les binaires publiés n’exigent pas cet environnement de compilation.

```sh
git clone https://github.com/antoineforterax-cmd/ARETOVUS.git
cd ARETOVUS
git switch main
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
python scripts/testnet_smoke.py
```

Le test de volume est séparé car il écrit plus de 100 000 fichiers de blocs :

```sh
cargo test --locked -p aretovus-storage --lib volume_above_100k_blocks_with_side_branch_and_restart -- --ignored --nocapture
```

Il couvre le stockage et le rejeu; il ne constitue pas un essai réseau de 72 heures. Les campagnes libFuzzer sont décrites dans [FUZZING.md](docs/FUZZING.md).

## Proposer une modification

Ouvrir une issue avec un scénario précis, ou choisir une `good first issue`. Faire une branche, ajouter un test qui reproduit le problème, puis proposer une PR vers `main` (ou vers la base indiquée par une PR empilée), après vérification des dépendances. Décrire le comportement avant/après, le coût en ressources, les résultats des tests et les effets sur les anciennes données. Ne pas modifier silencieusement genèse, émission, sérialisation, signatures ou règles de réorganisation. Une modification de protocole demande une spécification et une décision de compatibilité.

Ne jamais ajouter de clé privée, mot de passe, jeton ou dossier de nœud. Les fins de ligne sont LF. Les contributions sont proposées sous `MIT OR Apache-2.0`; les licences des dépendances et leurs avis restent applicables. Décrire les outils et méthodes utiles à la relecture, puis vérifier chaque modification proposée.

## Périmètre

Priorité au cœur, aux preuves de fonctionnement, à la documentation et à l’exploitation du testnet. Explorateur, site, vente simulée, interfaces graphiques et mobile sont reportés aux repreneurs. Aucun fonds réel, promesse de rendement ou déclaration de sécurité mainnet. Pour une faille exploitable, suivre [SECURITY.md](SECURITY.md) plutôt que publier les détails dans une issue.

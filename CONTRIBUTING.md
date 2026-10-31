# Contribuer à AURORA

Pour tout développement assisté par IA, lire d'abord [AGENTS.md](AGENTS.md) : ordre de travail, cartographie des PR, tests de sécurité adversariaux et règles avant fusion. Les décisions de protocole restent sous le contrôle du responsable du projet.

Le testnet expérimental est désormais intégré à `main`. Les branches historiques et les PR anciennes ne doivent pas être considérées comme la référence courante ; vérifier le SHA de `main` et ses contrôles GitHub Actions avant de commencer. Lire [ARCHITECTURE.md](ARCHITECTURE.md), [PROVENANCE.md](PROVENANCE.md), les [règles de consensus](docs/CONSENSUS.md) et les [portes de validation](docs/POST_MERGE_VALIDATION_GATES_2026-10-31.md).

## Parcours d'accueil (source unique, sans recopier les documents)

1. Lire [README.md](README.md) pour le statut actuel : **testnet expérimental, aucun fonds réel**. Lire ensuite [PROVENANCE.md](PROVENANCE.md) pour l'origine du code et les limites des vérifications assistées par IA.
2. Comprendre les cinq crates avec [ARCHITECTURE.md](ARCHITECTURE.md) et les règles effectivement exécutées avec [docs/CONSENSUS.md](docs/CONSENSUS.md) et [docs/TESTNET_DECISIONS.md](docs/TESTNET_DECISIONS.md).
3. Pour les menaces, consulter les [risques actuels sur la garde](docs/SECURITY_CUSTODY_THREAT_MODEL.md), les [critères de finalisation](docs/FINALIZATION_CHECKLIST.md), le [rapport de validation](docs/TESTNET_VALIDATION_REPORT.md) et les [preuves à obtenir avant mise en service](docs/POST_MERGE_VALIDATION_GATES_2026-10-31.md). [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md) est une **archive**, pas la description des protections présentes.
4. Choisir une tâche clairement bornée dans les [issues](https://github.com/antoineforterax-cmd/AURORA/issues), puis consigner le SHA, le test de reproduction et les limites. La [feuille de route #123](https://github.com/antoineforterax-cmd/AURORA/issues/123) décrit la priorité donnée au noyau sûr avant les nouvelles fonctionnalités.
5. Pour une **revue indépendante**, suivre le [dossier de revue](docs/INDEPENDENT_REVIEW.md) et, pour solliciter un volontaire, le [modèle d'appel à relecture](docs/HUMAN_REVIEW_CALL.md). Ne pas confondre ancienne PR citée dans un historique, branche ouverte et état réel de `main`.

**Règle documentaire :** conserver un seul document de référence par sujet ; ajouter des liens au lieu de recopier des spécifications. Les relevés historiques datés restent archivés avec leur commit, sans être présentés comme l'état courant. `AGENTS.md` et la politique d'implémentation contiennent encore une cartographie historique : **vérifier GitHub avant toute action**.

## Compiler et vérifier

Installer Rust stable, Cargo et Python 3.12 ou plus récent. Sous Windows, compiler avec la cible MSVC demande Visual C++ Build Tools et le Windows SDK; les binaires publiés n’exigent pas cet environnement de compilation.

```sh
git clone https://github.com/antoineforterax-cmd/AURORA.git
cd AURORA
git switch main
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

Ouvrir une issue avec un scénario précis, ou choisir une `good first issue`. Faire une branche, ajouter un test qui reproduit le problème, puis proposer une PR vers `main` (ou vers la base indiquée par une PR empilée), après vérification des dépendances. Décrire le comportement avant/après, le coût en ressources, les résultats des tests et les effets sur les anciennes données. Ne pas modifier silencieusement genèse, émission, sérialisation, signatures ou règles de réorganisation. Une modification de protocole demande une spécification et une décision de compatibilité.

Ne jamais ajouter de clé privée, mot de passe, jeton ou dossier de nœud. Les fins de ligne sont LF. Les contributions sont proposées sous `MIT OR Apache-2.0`; les licences des dépendances et leurs avis restent applicables. Décrire les outils et méthodes utiles à la relecture, puis vérifier chaque modification proposée.

## Licences et droits des contributions

AURORA est proposé sous la formule **`MIT OR Apache-2.0`** : le destinataire choisit **l'une des deux licences**. Les trois fichiers racine sont intentionnels : [`LICENSE`](LICENSE) regroupe les deux textes intégraux, [`LICENSE-MIT`](LICENSE-MIT) contient MIT seule et [`LICENSE-APACHE`](LICENSE-APACHE) contient Apache 2.0 seule. **Ne pas les supprimer ou les remplacer comme de simples doublons** sans décision explicite et revue de compatibilité.

La configuration [`Cargo.toml`](Cargo.toml) porte le même identifiant SPDX pour le workspace et ses crates. [`deny.toml`](deny.toml) fixe une liste de licences autorisées pour les dépendances, ce qui **ne dispense pas** d'examiner les attributions, exceptions et avis propres aux nouveaux composants. Les vecteurs NIST possèdent un [avis de provenance distinct](tests/vectors-fips204/NIST-NOTICE.txt).

N'ajouter que des contributions dont vous avez le droit de concéder l'usage sous `MIT OR Apache-2.0`. Ne copier aucun code, média, schéma ou jeu de données tiers sans conserver sa provenance et vérifier ses droits de réutilisation. Signaler explicitement toute dépendance ou tout contenu sous une autre licence dans la PR ; une approbation juridique n'est pas présumée.

## Périmètre

Priorité au cœur, aux preuves de fonctionnement, à la documentation et à l’exploitation du testnet. Explorateur, site, vente simulée, interfaces graphiques et mobile sont reportés aux repreneurs. Aucun fonds réel, promesse de rendement ou déclaration de sécurité mainnet. Pour une faille exploitable, suivre [SECURITY.md](SECURITY.md) plutôt que publier les détails dans une issue.

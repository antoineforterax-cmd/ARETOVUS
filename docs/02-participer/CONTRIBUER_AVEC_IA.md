# Contribuer à ARETOVUS avec ou sans intelligence artificielle

Ce guide complète [CONTRIBUTING.md](../CONTRIBUTING.md) et [AGENTS.md](../AGENTS.md). Il s'adresse à une personne débutante, à un développeur Rust, ou à un relecteur sécurité.

> ARETOVUS est un laboratoire blockchain. Tout test doit utiliser un environnement autorisé, isolé et **sans fonds réels**. Aucun assistant IA, test CI ou relecture interne ne constitue une certification du protocole.

## Quel parcours choisir ?

| Votre situation | Premier objectif réaliste | Document à ouvrir |
| --- | --- | --- |
| Vous découvrez la blockchain | Comprendre ce que fait une transaction, un bloc et un nœud | [Explication complète](COMPRENDRE_ARETOVUS.md) |
| Vous ne codez pas encore | Relever une erreur, améliorer un tutoriel, essayer une release historique | [Guide d'introduction](guide-fr/README.md) |
| Vous commencez Rust | Écrire un test isolé ou simplifier une portion non critique | [Architecture](../ARCHITECTURE.md) et [contribution](../CONTRIBUTING.md) |
| Vous utilisez une IA pour coder | Produire de petits changements, expliquer les hypothèses et reproduire les tests | [Politique d'implémentation](planning/IMPLEMENTATION_POLICY.md) |
| Vous maîtrisez la sécurité | Examiner les invariants, proposer un cas adversarial reproductible | [Revue indépendante attendue](INDEPENDENT_REVIEW.md) |

## Faire sa première proposition

1. **Lisez** [l'état actuel](ETAT_ACTUEL.md) et cherchez un problème précisément défini dans les [issues GitHub](https://github.com/antoineforterax-cmd/ARETOVUS/issues). Ne supposez pas qu'une issue ancienne est encore pertinente.
2. **Choisissez une petite tâche** : faute dans un guide, lien cassé, test de décodeur, message d'erreur. Le consensus, l'émission, la cryptographie, les secrets et le portefeuille nécessitent une revue plus exigeante.
3. **Créez une branche** (ou un fork si c'est votre mode d'accès) depuis un SHA identifié. Ne travaillez pas directement sur `main`.
4. **Documentez avant/après**, ajoutez un test de régression s'il y a un comportement à corriger, et notez précisément les commandes exécutées et leurs sorties.
5. **Ouvrez une pull request en brouillon** avec la raison du changement, le SHA, les fichiers, les risques, les preuves et ce qui **n'a pas été testé**.
6. **Relisez et corrigez** à partir des commentaires ; les mainteneurs décident de l'intégration. Un correctif proposé n'est pas une fonctionnalité livrée.

## Que faire si une IA écrit le code ?

L'IA peut aider à expliquer Rust, proposer des tests, comparer des alternatives et préparer des patchs. **Elle peut aussi produire un code qui compile, passe des tests incomplets et contient une vulnérabilité.**

Pour une contribution assistée par IA :

- Indiquez **quels outils ont été utilisés et pour quelles parties**, sans prétendre que leur sortie a été auditée.
- Exigez de l'IA un scénario de panne, des entrées hostiles, les limites de la solution et un test reproduisant le défaut.
- Vérifiez vous-même les chemins, la version Rust, les arguments, les licences, les dépendances et les résultats.
- Demandez une **contre-revue séparée** ; un second agent IA est une relecture automatisée, **pas un audit humain indépendant**.
- N'envoyez **jamais de clés privées**, secrets de déploiement ou données d'un portefeuille réel dans le chatbot, un ticket ou une PR.
- Pour les composants critiques, la compétence humaine indépendante est indispensable avant tout usage financier réel.

## Commandes de contrôle du workspace (depuis la racine)

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
python3 scripts/testnet_smoke.py
```

Ce sont des **commandes à exécuter** sur votre environnement, **pas une affirmation qu'elles ont réussi**. Sur Windows, Python peut être lancé via `py -3` selon votre installation. Consultez les éventuelles dépendances supplémentaires du [guide testnet](TESTNET_GUIDE.md).

## Peut-on faire sa version personnelle ?

Oui, les licences MIT/Apache-2.0 autorisent les forks, sous réserve des conditions applicables. Vous pouvez modifier ARETOVUS pour apprendre, sans lancer une monnaie publique ni gérer des fonds réels. L'objectif communautaire est de **favoriser le retour des corrections et idées utiles dans le projet principal**. Voir [licences, droits et attribution](LICENCES_ET_DROITS.md).

## Tests et expériences : ce qu'il faut séparer

- **Réussite locale** : votre machine et votre SHA.
- **CI GitHub** : résultats d'un workflow, pour un commit donné.
- **Expérience multi-processus** : plusieurs processus sur un ordinateur, pas nécessairement plusieurs ordinateurs.
- **Essai physique multi-PC** : machines distinctes et preuves de convergence.
- **Audit humain extérieur** : personne indépendante, périmètre, constats, correctifs et revalidation documentés.

N'écrivez jamais « sécurisé » ou « prêt pour le mainnet » à partir d'une simple CI verte.

## Respecter les frontières

Ne modifiez pas la genèse, les règles d'émission, les signatures admises, les formats de consensus ni la politique de récupération **sans proposition de protocole et accord explicite**. Les changements de sécurité doivent être signalés de manière responsable via [SECURITY.md](../SECURITY.md). Toute difficulté de compréhension constitue un retour utile : le but est d'ouvrir la participation **sans banaliser les risques**.

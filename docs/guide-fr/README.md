# AURORA, de A à Z — guide pour débutants

Bienvenue ! **Pas besoin de savoir coder** pour comprendre l'objectif d'AURORA ou lancer un premier essai. Il s'agit d'une **blockchain expérimentale** : les AUR de test n'ont pas de valeur garantie, aucun réseau public stable ni audit humain indépendant n'est annoncé, et il ne faut pas utiliser de vrais fonds.

## AURORA expliqué simplement

Un **wallet** (portefeuille) garde les clés permettant de signer un paiement. Un **nœud** vérifie les transactions et conserve sa copie de la chaîne. Un **mineur** cherche un bloc valide. Si deux nœuds affichent le même sommet (`tip`) et la même hauteur, ils ont convergé **à cet instant** ; cela ne suffit pas à prouver leur sécurité à long terme.

Le logiciel actuel permet des essais de minage, de transfert, de sauvegarde et de synchronisation. La protection avancée de la vie privée, le coffre de récupération, les applications mobiles et l'explorateur graphique **ne sont pas encore livrés**.

## Choisissez votre parcours

| Votre profil | Parcours conseillé |
| --- | --- |
| **Je découvre la blockchain** | [1. Comprendre](01-comprendre.md) → [2. Économie](02-economie.md) → [7. Technologie et limites](07-technologie-et-suite.md) |
| **Je veux essayer AURORA sans compiler** | [3. Installation](03-installation.md) → [4. Démarrer un réseau](04-reseau-et-minage.md) → [5. Portefeuille](05-portefeuille.md) → [6. Exploitation](06-exploitation.md) |
| **Je veux comprendre l'avenir du projet** | [Vision et priorités](../planning/README.md) → [Décisions à prendre](../planning/DECISIONS.md) → [Cahier des charges](../planning/CAHIER_DES_CHARGES.md) |
| **Je suis développeur ou reviewer** | [Architecture](../../ARCHITECTURE.md) → [Contribuer](../../CONTRIBUTING.md) → [Dossier de revue](../INDEPENDENT_REVIEW.md) |
| **Je veux vérifier une affirmation** | [Règles du consensus](../CONSENSUS.md) → [Tests et limites](../TESTNET_VALIDATION_REPORT.md) |

## Sommaire des sept chapitres

1. [Comprendre AURORA et les mots essentiels](01-comprendre.md)
2. [Économie : émission, mineurs, frais et burn](02-economie.md)
3. [Installation de la version expérimentale](03-installation.md)
4. [Démarrer un nœud et miner](04-reseau-et-minage.md)
5. [Portefeuille, paiements et sauvegardes](05-portefeuille.md)
6. [Vérification, restauration et dépannage](06-exploitation.md)
7. [Technologie, risques et prochaines étapes](07-technologie-et-suite.md)

## Quelle version ce guide utilise-t-il ?

Les exemples de téléchargement sont liés à la release **`testnet-v0.0.1-experimental.1`**, publiée le 27 octobre 2026 à partir de **`421d443370e83a9ffa0c128da9f09d6f99b0ef8f`**. **Ce n'est pas nécessairement la dernière révision du développement.** Pour les essais de deux ordinateurs sous Windows avec une archive GitHub Actions récente, suivre [TWO_PC_TEST.md](../TWO_PC_TEST.md) et noter le SHA exact du binaire.

Les commandes principales emploient PowerShell : `bin/aurora-node.exe` et `bin/aurora-wallet.exe`. Sur Linux ou macOS, les arguments sont similaires mais les chemins changent et il n'y a pas de `.exe`. Chaque nœud utilise son propre répertoire `--data-dir` : ouvrir deux fenêtres sur un seul dossier ne crée pas deux nœuds autonomes.

## Documentation et transparence

Le code et la spécification au **commit examiné** font foi sur le comportement. Les paramètres expérimentaux sont dans les [décisions testnet](../TESTNET_DECISIONS.md), les limites dans le [rapport d'essais](../TESTNET_VALIDATION_REPORT.md), et l'historique de développement assisté par IA dans [PROVENANCE.md](../../PROVENANCE.md). Les licences sont [MIT OR Apache-2.0](../../LICENSE) ; une faille potentielle se signale selon [SECURITY.md](../../SECURITY.md).

Ce guide explique le projet au public ; il ne remplace ni les validations techniques ni une revue humaine indépendante.

# Documentation ARETOVUS

**Bienvenue !** ARETOVUS est une blockchain expérimentale en Rust développée avec l'aide de l'IA, sous direction humaine et ouverte à la collaboration. Ce dépôt est un **laboratoire technique** : aucun mainnet autorisé, aucune utilisation avec des fonds réels, aucun audit humain externe indépendant approfondi confirmé.

Pour découvrir le projet, **suivez les étapes dans l'ordre**. Il n'est pas nécessaire de connaître Rust.

## Pour commencer, choisissez votre parcours

| Je veux… | Première lecture | Suite |
| --- | --- | --- |
| **Comprendre sans coder** | [Comment fonctionne ARETOVUS](01-demarrer/COMPRENDRE_ARETOVUS.md) | [Ce qui fonctionne actuellement](01-demarrer/ETAT_ACTUEL.md) |
| **Installer et tester** | [Guide français de la release expérimentale](guide-fr/README.md) | [Nœud, minage et wallet en local](TESTNET_GUIDE.md) |
| **Participer avec ou sans IA** | [Première contribution](02-participer/CONTRIBUER_AVEC_IA.md) | [Règles des contributions](../CONTRIBUTING.md) |
| **Comprendre le code** | [Architecture Rust](../ARCHITECTURE.md) | [Consensus](CONSENSUS.md) et [protocole](PROTOCOL.md) |
| **Examiner la sécurité** | [Modèle de menaces](THREAT_MODEL.md) | [Audit indépendant restant à faire](06-securite/INDEPENDENT_REVIEW.md) |
| **Comprendre les licences** | [Droits, contributions et forks](02-participer/LICENCES_ET_DROITS.md) | [Textes juridiques originaux](../LICENSE) |
| **Comparer les anciennes décisions** | [Choix et contradictions clarifiés](05-pilotage/DECISIONS_CLARIFIEES.md) | [Rapports historiques](90-archives/ARCHIVES_ET_RAPPORTS.md) |

## Les dossiers spécialisés

| Dossier | Ce qu'on y trouve |
| --- | --- |
| [01 — Bien démarrer](01-demarrer/) | Présentation accessible, explication des transactions et état vérifié |
| [02 — Participer](02-participer/) | Premières contributions, travail humain–IA, droits et licences |
| [03 — Recherche](03-recherche/) | Modèles alternatifs, études crypto/PQ, stockage, migrations |
| [04 — Exploiter](04-exploiter/) | Deux PC, endurance, sauvegardes, wallet et procédures pratiques |
| [05 — Pilotage](05-pilotage/) | Décisions réconciliées, suivi documentaire, provenance historique |
| [06 — Sécurité](06-securite/) | Audits à réaliser, cryptographie, garde des clés, contre-revues |
| [07 — Rapports](07-rapports/) | Mesures, tests adversariaux, simulations et constats datés |
| [90 — Archives](90-archives/) | Index historique et correspondances des anciens chemins |
| [Guide français](guide-fr/) | Tutoriel reproductible d'une **ancienne release datée**, sans garantie de compatibilité avec `main` |
| [Planification](planning/) | Propositions et ADR, pas des décisions de consensus déjà validées |
| [Validation](validation/) | Protocoles d'essais et procédures de preuve |
| [Consolidation](consolidation/) | États techniques historiques datés |

## Pourquoi quelques fichiers restent à la racine de `docs/` ?

Ces fichiers sont des **références stables** auxquelles renvoient d'autres outils, scripts de création des archives et guides :

- [CONSENSUS.md](CONSENSUS.md) : règles expérimentales de validation.
- [PROTOCOL.md](PROTOCOL.md) : formats, messages et protocole.
- [TESTNET_DECISIONS.md](TESTNET_DECISIONS.md) et [EMISSION_MODEL.md](EMISSION_MODEL.md) : paramètres du testnet (ne préjugent pas du mainnet).
- [TESTNET_GUIDE.md](TESTNET_GUIDE.md) et [TESTNET_OPERATIONS.md](TESTNET_OPERATIONS.md) : démarrage et exploitation.
- [TESTNET_VALIDATION_REPORT.md](TESTNET_VALIDATION_REPORT.md) : essais historiques avec références des commits.
- [THREAT_MODEL.md](THREAT_MODEL.md), [FUZZING.md](FUZZING.md) : sécurité et tests.
- [DECISION_REGISTER.md](DECISION_REGISTER.md) et [FINALIZATION_CHECKLIST.md](FINALIZATION_CHECKLIST.md) : **archives de décisions et de jalons**, à lire avec leurs avertissements historiques.

Les déplacer automatiquement casserait certains scripts ou des références historiques. Le but est de **réduire les confusions sans sacrifier les preuves ni la reproductibilité**.

## Comment savoir ce qui est vrai ?

1. Pour un **comportement du logiciel**, le code et ses tests au **SHA Git exact** sont déterminants ; un document ancien ne suffit pas.
2. Pour l'**avancement V2**, consulter [l'issue #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) et les PR ; [#142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142) concerne les étapes antérieures.
3. Pour les **preuves**, lire les [runs GitHub Actions](https://github.com/antoineforterax-cmd/ARETOVUS/actions) et les limitations des essais.
4. **Une CI verte, une revue IA et un essai sur deux ordinateurs ne valent pas un audit indépendant.** Aucun usage financier réel n'est autorisé par cette documentation.

**Une erreur, une contradiction ou un lien cassé ?** Commencez par [les conventions de documentation](05-pilotage/CONVENTIONS_DOCUMENTAIRES.md), puis proposez une correction ciblée par pull request.

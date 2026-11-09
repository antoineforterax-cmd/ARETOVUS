# Documentation ARETOVUS — commencer ici

> **Statut : laboratoire blockchain expérimental.** Aucun mainnet, aucun fonds réel, aucune garantie de sécurité et **aucun audit humain externe indépendant confirmé**. Les essais, revues par IA et CI ne remplacent pas un audit. Les documents datés décrivent le commit ou le prototype qu'ils examinent, pas nécessairement `main` aujourd'hui.

Cette page est le **point d'entrée documentaire**. L'objectif est de guider les nouveaux lecteurs sans effacer les rapports, preuves ni décisions historiques.

## Je veux…

| Mon objectif | Point de départ | Ensuite |
| --- | --- | --- |
| **Découvrir sans programmer** | [Guide français de A à Z](guide-fr/README.md) | [Comprendre ARETOVUS](guide-fr/01-comprendre.md) |
| **Essayer un réseau de test local** | [Guide du testnet](TESTNET_GUIDE.md) | [Réseau et minage](guide-fr/04-reseau-et-minage.md) |
| **Comprendre le code** | [Architecture générale](../ARCHITECTURE.md) | [Protocole](PROTOCOL.md), [consensus](CONSENSUS.md) |
| **Contribuer avec ou sans IA** | [Guide de contribution](../CONTRIBUTING.md) | [Politique de développement](planning/IMPLEMENTATION_POLICY.md), [règles d'agents](../AGENTS.md) |
| **Examiner la sécurité** | [Politique de signalement](../SECURITY.md) | [Modèle de menaces](THREAT_MODEL.md), [revue indépendante attendue](INDEPENDENT_REVIEW.md) |
| **Savoir ce qui reste à faire** | [Feuille de route V2 active, issue #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) | [Plan maître antérieur, issue #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142) |
| **Vérifier ce qui a été testé** | [Rapport de validation](TESTNET_VALIDATION_REPORT.md) | [GitHub Actions](https://github.com/antoineforterax-cmd/ARETOVUS/actions) et SHA précis |

**Attention aux versions :** le [guide français](guide-fr/README.md) documente notamment une release du **27 octobre 2026**, pas automatiquement la dernière version de `main`. Lisez toujours la date, la branche et le SHA. Ne réutilisez pas d'anciennes données/clés sur le réseau ARETOVUS natif sans suivre la [migration d'identité](PROTOCOL_IDENTITY_MIGRATION.md).

## Carte thématique

### 1 — Comprendre le système

- [Architecture du dépôt](../ARCHITECTURE.md)
- [Protocole](PROTOCOL.md), [règles de consensus](CONSENSUS.md), [modèle d'émission](EMISSION_MODEL.md)
- [Décisions effectives du testnet](TESTNET_DECISIONS.md), [cryptographie post-quantique et limites](POST_QUANTUM.md)
- [Guide d'introduction](guide-fr/README.md)

### 2 — Installer, exécuter, maintenir

- [Guide de lancement du testnet](TESTNET_GUIDE.md)
- [Exploitation et dépannage](TESTNET_OPERATIONS.md)
- [Tests physiques sur deux PC](TWO_PC_TEST.md) et [intégration deux PC](INTEGRATION_TWO_PC.md)
- [Validation du testnet](TESTNET_VALIDATION_REPORT.md), [fuzzing](FUZZING.md)

### 3 — Sécurité et audits

- [Signalement responsable](../SECURITY.md)
- [Modèle de menaces](THREAT_MODEL.md), [protection des clés](SECURITY_CUSTODY_THREAT_MODEL.md)
- [Revue humaine indépendante — travail attendu, pas audit accompli](INDEPENDENT_REVIEW.md)
- [Contre-revue par IA — vérification interne seulement](AI_COUNTER_REVIEW.md)

### 4 — Développer et contribuer

- [Contribuer](../CONTRIBUTING.md)
- [Règles des agents et garde-fous](../AGENTS.md)
- [Politique d'implémentation](planning/IMPLEMENTATION_POLICY.md)
- [Cahier des charges et propositions](planning/README.md)
- [Provenance des contributions, notamment IA](../PROVENANCE.md)

### 5 — Décisions et feuille de route

- **[Issue #178 — feuille de route V2](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178)** : suivi de l'application, du wallet, de l'explorateur, du P2P et de la validation future.
- **[Issue #142 — plan maître précédent](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142)** : conserver les preuves antérieures ; ne pas recopier ses cases comme statut actuel.
- [Registre des décisions](DECISION_REGISTER.md) et [dossier de planification](planning/README.md).
- [Ancienne feuille de route narrative](ROADMAP_EXECUTION.md) et [ancienne checklist](FINALIZATION_CHECKLIST.md) : documents **historiques**, à ne pas utiliser seuls pour juger la version courante.

### 6 — Archives et résultats techniques détaillés

Les autres fichiers de `docs/`, notamment les matrices adversariales, rapports de performance, bilans datés et études de conception, sont conservés. Ils sont **des preuves ou des propositions contextualisées**, pas nécessairement des spécifications en vigueur. Il est volontairement possible de [parcourir tout le dossier](https://github.com/antoineforterax-cmd/ARETOVUS/tree/main/docs) sans afficher chaque rapport sur la page d'accueil.

## Comment vérifier une affirmation technique

1. Identifier la **branche**, le **SHA** et la **date** de la preuve.
2. Comparer le code et le comportement exécuté à ce SHA, et consulter les workflows GitHub Actions terminés.
3. Distinguer un test local, un test CI multi-processus, un essai **sur deux machines physiques** et une **revue humaine indépendante** : ce sont des niveaux différents.
4. Pour toute modification du consensus, de la cryptographie, des fonds, des formats ou de l'économie : ouvrir une décision dédiée ; **ne pas fusionner à partir de cette seule documentation**.

**Perspective :** ARETOVUS vise d'abord un laboratoire open source collaboratif humain–IA. Un éventuel mainnet avec des unités ayant une valeur réelle reste un objectif conditionnel à l'examen indépendant, à la sécurité du protocole et à l'analyse juridique ; **ce n'est pas un lancement annoncé**.

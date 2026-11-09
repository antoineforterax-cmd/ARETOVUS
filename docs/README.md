# Documentation ARETOVUS — point d'entrée unique

> **ARETOVUS est un laboratoire blockchain expérimental en Rust. Pas de mainnet, pas de fonds réels, pas de sécurité certifiée.** Aucun audit humain extérieur indépendant approfondi et documenté n'est confirmé. Le projet est largement développé avec l'assistance de l'IA et cherche des contributions humaines.

Ce sommaire est la porte d'entrée de la documentation. **Il n'existe pas six feuilles de route équivalentes** : l'[issue V2 #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) organise le développement courant ; l'[issue #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142) garde le suivi des étapes précédentes. Les anciens rapports restent consultables, mais ils ne remplacent ni le code au SHA exact, ni les décisions du testnet.

## Commencer selon son profil

| Votre besoin | Commencez ici | Pour approfondir |
| --- | --- | --- |
| **Je ne connais rien à la blockchain** | [Comprendre ARETOVUS de A à Z](COMPRENDRE_ARETOVUS.md) | [Guide débutant de la release historique](guide-fr/README.md) |
| **Je veux essayer un nœud ou miner sans fonds réels** | [Installation de la release expérimentale](guide-fr/03-installation.md) | [Réseau local et minage](guide-fr/04-reseau-et-minage.md), [guide technique testnet](TESTNET_GUIDE.md) |
| **Je veux connaître le véritable état du projet** | **[État actuel et limites](ETAT_ACTUEL.md)** | [Preuves et tests](TESTNET_VALIDATION_REPORT.md), [GitHub Actions](https://github.com/antoineforterax-cmd/ARETOVUS/actions) |
| **Je suis développeur ou relecteur** | [Architecture](../ARCHITECTURE.md) | [Protocole](PROTOCOL.md), [consensus](CONSENSUS.md), [code Rust](../crates) |
| **Je veux aider, avec ou sans IA** | [Première contribution](CONTRIBUER_AVEC_IA.md) | [Règles de PR](../CONTRIBUTING.md), [garde-fous](../AGENTS.md) |
| **Je veux examiner la sécurité** | [Modèle de menaces](THREAT_MODEL.md) | [Revue indépendante attendue](INDEPENDENT_REVIEW.md), [signalement privé](../SECURITY.md) |
| **Je veux connaître les licences et les droits** | [Licences et droits](LICENCES_ET_DROITS.md) | [Textes officiels](../LICENSE), [provenance](../PROVENANCE.md) |
| **Je vois une contradiction entre deux documents** | **[Décisions clarifiées](DECISIONS_CLARIFIEES.md)** | [Décisions testnet](TESTNET_DECISIONS.md), [catalogue des archives](ARCHIVES_ET_RAPPORTS.md) |

## Les six repères de la documentation

1. **Découvrir** — [explication de la blockchain](COMPRENDRE_ARETOVUS.md), [guide pas à pas](guide-fr/README.md), [glossaire](COMPRENDRE_ARETOVUS.md#10-petit-glossaire).
2. **Fonctionnement technique** — [architecture](../ARCHITECTURE.md), [protocole](PROTOCOL.md), [consensus](CONSENSUS.md), [économie du testnet](EMISSION_MODEL.md).
3. **Installer et utiliser le laboratoire** — [testnet](TESTNET_GUIDE.md), [exploitation](TESTNET_OPERATIONS.md), [desktop expérimental](../apps/desktop/README.md).
4. **Sécurité et limites** — [état actuel](ETAT_ACTUEL.md), [menaces](THREAT_MODEL.md), [revue externe à réaliser](INDEPENDENT_REVIEW.md), [preuves et résultats](TESTNET_VALIDATION_REPORT.md).
5. **Participer** — [guide humain–IA](CONTRIBUER_AVEC_IA.md), [contribuer](../CONTRIBUTING.md), [politique d'implémentation](planning/IMPLEMENTATION_POLICY.md), [licences](LICENCES_ET_DROITS.md).
6. **Suivi et archives** — [roadmap V2](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178), [reconciliation](DECISIONS_CLARIFIEES.md), [catalogue des rapports anciens](ARCHIVES_ET_RAPPORTS.md).

## Comment interpréter ce dépôt

- **Implémenté dans un testnet** ne signifie pas **prêt pour des fonds réels**.
- Une **CI verte** valide une exécution précise, **pas** l'absence de toutes les vulnérabilités.
- Une **contre-revue IA** n'est pas une **revue humaine externe**.
- Un guide **daté d'octobre** n'est pas automatiquement valable pour les binaires de novembre.
- Les paramètres testnet **n'engagent pas un futur mainnet** : les modifications critiques nécessitent des décisions spécifiques.
- Une fonctionnalité annoncée dans un cahier des charges est parfois une **ambition**, et non un code livré.

## Et les dizaines d'autres fichiers ?

Ils sont rangés **par sujet et par statut** dans [ARCHIVES_ET_RAPPORTS.md](ARCHIVES_ET_RAPPORTS.md). Les preuves historiques et les anciens liens ne sont volontairement **pas supprimés** par simple nettoyage visuel. Pour éviter de recréer le désordre : [conventions d'organisation documentaire](CONVENTIONS_DOCUMENTAIRES.md). Pour les questions de gouvernance et de nouvelles orientations, consulter [le dossier de planification](planning/README.md) en distinguant bien propositions et règles existantes.

**Premier essai conseillé :** [Comprendre ARETOVUS](COMPRENDRE_ARETOVUS.md) → [Installer un ancien testnet local](guide-fr/03-installation.md) → [Miner un bloc de test](guide-fr/04-reseau-et-minage.md) → [Comprendre comment contribuer](CONTRIBUER_AVEC_IA.md).

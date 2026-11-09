# ARETOVUS — laboratoire blockchain expérimental

<p align="center">
  <img src="assets/aretovus-logo.svg" alt="Emblème ARETOVUS : temple stylisé et lettre A" width="250">
</p>

**Blockchain open source en Rust · développement humain–IA · réseau de test expérimental**

> **Sécurité : aucun mainnet, aucun fonds réel.** Le logiciel et le protocole n'ont pas fait l'objet d'un audit humain externe indépendant confirmé. Ils ne doivent pas être utilisés pour recevoir, conserver ou échanger des actifs de valeur. Des tests automatisés réussis ne constituent pas une garantie de sécurité.

ARETOVUS explore la construction, le test et l'amélioration collaborative d'une blockchain. Le porteur du projet définit les objectifs et pilote le développement, **largement assisté par IA**. Les contributions humaines, contre-revues techniques et expérimentations reproductibles sont bienvenues. Voir [provenance et vérifications](PROVENANCE.md).

La finalité immédiate est un **laboratoire ouvert et compréhensible**. L'hypothèse d'un futur réseau public doté d'un véritable minage et d'unités susceptibles d'avoir une valeur réelle reste **conditionnelle**, sans date ni promesse de lancement.

## Commencer en deux minutes

| Vous voulez… | Rendez-vous ici |
| --- | --- |
| Découvrir le projet, même sans savoir coder | [Guide débutant en français](docs/guide-fr/README.md) |
| Trouver la bonne documentation sans vous perdre | **[Carte de la documentation](docs/README.md)** |
| Comprendre le consensus, les signatures et le réseau | [Architecture](ARCHITECTURE.md) et [protocole](docs/PROTOCOL.md) |
| Installer et expérimenter en local | [Guide testnet](docs/TESTNET_GUIDE.md) |
| Contribuer, seul ou avec une IA | [CONTRIBUTING.md](CONTRIBUTING.md) |
| Vérifier les tests et les limites | [Rapport de validation](docs/TESTNET_VALIDATION_REPORT.md) |
| Consulter les priorités du développement | **[Feuille de route V2 (issue #178)](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178)** |
| Signaler une vulnérabilité | [SECURITY.md](SECURITY.md) |

Le [guide débutant](docs/guide-fr/README.md) explique une **ancienne release expérimentale d'octobre 2026**. Vérifiez toujours le commit et la version des binaires avant de reproduire une commande.

## Que contient le projet ?

| Élément | Portée actuelle |
| --- | --- |
| **Cœur Rust et consensus** | Transactions, blocs, comptes, signatures natives ML-DSA-65 sur testnet, preuve de travail SHA-256, validations et réorganisations |
| **Nœud et stockage** | Nœud TCP/P2P expérimental, minage local, synchronisation, sauvegarde et restauration |
| **Portefeuille** | Outils en ligne de commande, clés chiffrées, paiements expérimentaux et vérifications SPV |
| **Application et explorateur** | Prototypes desktop et consultation en lecture seule, encore incomplets ; suivre les PR et l'[issue #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) pour le périmètre réel |
| **Sécurité et validation** | Tests Rust/CI, campagnes ciblées et rapports de risques ; **aucune certification de sécurité ni revue humaine indépendante confirmée** |

Le code et les [règles de consensus](docs/CONSENSUS.md) à un **SHA précis** font foi pour les comportements exécutés. Les rapports historiques et la documentation de planification ne prouvent pas à eux seuls l'état courant.

### Un essai technique local

Sur un environnement de développement Rust/Python adapté :

```sh
git clone https://github.com/antoineforterax-cmd/ARETOVUS.git
cd ARETOVUS
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
python3 scripts/testnet_smoke.py
```

Ce sont les **commandes de vérification prévues**, pas des résultats de tests obtenus lors de la rédaction de cette page. Pour exécuter un nœud et miner des unités de test, suivre le [guide testnet](docs/TESTNET_GUIDE.md) dans un environnement isolé et autorisé ; ne jamais y utiliser de clés ou de fonds réels.

## État, risques et compatibilité

- **Statut : testnet expérimental, NO-GO mainnet/fonds réels.** Les conditions d'un éventuel lancement public ne sont pas remplies.
- **Réseau natif ARETOVUS :** identité de protocole, genèse, clés et formats incompatibles avec les anciennes versions du réseau. **Ne réutilisez pas un ancien dossier de données comme s'il était compatible** ; consulter la [migration](docs/PROTOCOL_IDENTITY_MIGRATION.md).
- **Limites à examiner :** tests adversariaux prolongés, réseaux et machines indépendants, résistance aux attaques, cryptographie/gestion des clés, gouvernance et évaluation juridique. Voir [revue indépendante attendue](docs/INDEPENDENT_REVIEW.md).
- **Traçabilité :** [GitHub Actions](https://github.com/antoineforterax-cmd/ARETOVUS/actions), [preuves et limites](docs/TESTNET_VALIDATION_REPORT.md). Les résultats dépendent du commit testé.
- **Feuilles de route :** l'[issue #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) couvre le chantier V2 ; l'[issue #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142) préserve les jalons antérieurs. Les descriptions datées restent historiques.

Les détails économiques du **testnet uniquement** sont documentés dans [EMISSION_MODEL.md](docs/EMISSION_MODEL.md). Ils ne sont pas une offre de cryptoactifs ni une promesse de valeur.

## Participer

Les débutants peuvent commencer par la documentation, les tests et l'expérimentation sur un réseau isolé. Les développeurs peuvent proposer des correctifs et des fonctionnalités sous forme de petites pull requests, avec reproductions, risques et preuves de test. Les mécanismes critiques exigent une revue et une décision spécifiques : [guide de contribution](CONTRIBUTING.md), [règles de développement](AGENTS.md).

Les personnes qui découvrent une faille doivent utiliser les instructions de [signalement responsable](SECURITY.md), et non publier une exploitation détaillée.

## Licence et historique

Le code est proposé sous **MIT OR Apache-2.0**, sous réserve des droits et licences des contributions concernées : [LICENSE](LICENSE). Le projet accepte les contributions au dépôt commun, même si ces licences autorisent aussi la création de versions dérivées.

Pour retrouver la présentation technique antérieure, consulter l'[historique du README sur GitHub](https://github.com/antoineforterax-cmd/ARETOVUS/commits/main/README.md) ; aucun historique Git n'est effacé par cette réorganisation documentaire.

# AURORA — blockchain indépendante en développement

> **État au 29 octobre 2026 : projet open source expérimental, sans mainnet ni audit humain indépendant. Ne pas utiliser avec des fonds réels.**

## Où en est AURORA ?

**Attention aux branches :** la branche `main` contient toujours le **prototype historique initial** (code Rust minimal). Le testnet nettement plus avancé est développé sur [`feature/emission-schedule-v0`](https://github.com/antoineforterax-cmd/AURORA/tree/feature/emission-schedule-v0), via la [PR #2](https://github.com/antoineforterax-cmd/AURORA/pull/2), non fusionnée. **Ce README décrit l'ensemble du projet, pas les fonctions présentes dans `main`.**

Le nom **v0.0.1** correspond au prototype historique ; il ne représente pas l'avancement global. Une [release expérimentale `testnet-v0.0.1-experimental.1`](https://github.com/antoineforterax-cmd/AURORA/releases/tag/testnet-v0.0.1-experimental.1) a été publiée le 27 octobre 2026 à partir d'un commit précis ; ce n'est ni un mainnet ni la dernière branche de développement.

## Ce qui est déjà développé sur la branche testnet

| Composant | État sur la branche de développement |
| --- | --- |
| Noyau Rust et comptes | Montants entiers, encodage canonique, comptes, soldes et nonces |
| Signatures | ML-DSA-65 natives v3 pour le testnet ; formats anciens limités au Regtest |
| Consensus | PoW SHA-256, blocs et preuves Merkle, difficulté ajustée, travail cumulé, réorganisations |
| Économie expérimentale | Plafond brut 500 millions AUR, huit décimales, émission décroissante, 95 % mineur / 5 % trésorerie de test, moitié des frais brûlée |
| Stockage | Persistance v6, index paginés, vérifications et reprise après certaines pannes |
| Réseau | P2P TCP expérimental, pairs configurés, synchronisation et relais des transactions |
| Portefeuille | CLI : clés chiffrées, paiements, sauvegarde/restauration, vérification SPV |
| Vérification | CI Rust, smoke multi-nœuds, fuzzing et essais ciblés documentés |

**Ces capacités ne sont pas toutes présentes dans `main`.** Les améliorations plus récentes sont réparties entre des PR supplémentaires non fusionnées, notamment [#13 à #22](https://github.com/antoineforterax-cmd/AURORA/pulls). Leurs résultats ne doivent pas être attribués à la branche de base sans intégration et nouveaux tests.

## Ce qui reste à faire

### Priorité 0 — fiabiliser et intégrer
- Revoir et intégrer les PR dans un ordre contrôlé ; obtenir une branche de référence cohérente, CI verte et versionnée.
- Achever les scénarios d'horodatages futurs et de retarget ([#3](https://github.com/antoineforterax-cmd/AURORA/issues/3)).
- Poursuivre les tests de saturation P2P, y compris après handshake, sous NAT et avec plusieurs sources ([#4](https://github.com/antoineforterax-cmd/AURORA/issues/4)).
- Prouver la synchronisation réseau et les réorganisations sur de gros volumes, pas seulement le stockage de 100 003 blocs ([#5](https://github.com/antoineforterax-cmd/AURORA/issues/5)).
- Obtenir et publier les résultats complets de l'essai 72 h, puis mener des tests multi-machines, pannes et fuzzing prolongé.

### Priorité 1 — spécifier les innovations avant implémentation
- Confidentialité : protection des IP et métadonnées, destinataires, expéditeurs et montants. Tor/I2P, relais anonymisants et preuves confidentielles restent **à étudier** ; aucune garantie d'anonymat absolu.
- Sécurité des fonds : prévention locale des arnaques, coffre optionnel, délais et récupération sécurisée. Les mécanismes avancés ne sont **pas implémentés**.
- Trésorerie et gouvernance : règles de dépense, contrôle, sécurité et transparence encore à décider.
- Évaluer la compatibilité post-quantique du système complet ; ML-DSA-65 seul ne suffit pas.

### Priorité 2 — écosystème et accessibilité
- Explorateur, site de présentation, portefeuille graphique, installation simplifiée et mobile.
- Tests d'ergonomie, exploitation publique, guides utilisateurs et contributions externes.
- Revue humaine indépendante et analyse juridique avant tout usage financier réel ou annonce mainnet.

## Comprendre et essayer le projet

1. [Guide français « AURORA, de A à Z »](https://github.com/antoineforterax-cmd/AURORA/blob/feature/emission-schedule-v0/docs/guide-fr/README.md).
2. [README technique du testnet](https://github.com/antoineforterax-cmd/AURORA/blob/feature/emission-schedule-v0/README.md).
3. [Cahier des charges et ambitions](https://github.com/antoineforterax-cmd/AURORA/blob/docs/aurora-master-plan-2026-10/docs/planning/CAHIER_DES_CHARGES.md).
4. [Feuille de route stratégique](https://github.com/antoineforterax-cmd/AURORA/blob/docs/aurora-master-plan-2026-10/docs/planning/ROADMAP.md).
5. [Checklist technique et limites](https://github.com/antoineforterax-cmd/AURORA/blob/feature/emission-schedule-v0/docs/FINALIZATION_CHECKLIST.md).
6. [Release testnet expérimentale](https://github.com/antoineforterax-cmd/AURORA/releases/tag/testnet-v0.0.1-experimental.1) (binaires liés à un commit ancien, non signés et non audités).

Pour contribuer au **testnet avancé**, suivre [CONTRIBUTING.md sur la branche de développement](https://github.com/antoineforterax-cmd/AURORA/blob/feature/emission-schedule-v0/CONTRIBUTING.md). Un simple `git clone` récupère `main`, **pas** le testnet : sélectionner explicitement la branche indiquée.

## Règles de prudence

Les tests automatisés, revues IA et benchmarks ponctuels ne remplacent pas un audit humain indépendant. Les paramètres du testnet ne sont pas une promesse pour le mainnet. Les propositions de confidentialité et de récupération ne doivent pas être présentées comme livrées. Tout changement de consensus exige une décision explicite, une spécification, des tests et un plan de compatibilité.

**Licence :** MIT OR Apache-2.0 (voir les fichiers de licence du dépôt).

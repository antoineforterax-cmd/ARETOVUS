# AURORA — Cahier des charges consolidé

**Version** 0.1 / 2026-10-28 / **statut : proposition documentaire, à confronter au Word historique**.

## 1. Mission et priorités
Concevoir une blockchain indépendante en Rust, décentralisée et vérifiable, avec priorité à la sécurité, à la robustesse et à la maintenabilité. Explorer des frais raisonnables, une participation accessible au réseau, une confidentialité forte et une trajectoire cryptographique post-quantique. Ces ambitions sont des **objectifs**, non des garanties ou des fonctionnalités déjà livrées.

## 2. Exigences non fonctionnelles
- **Sécurité** : validation déterministe, encodage canonique, limites de ressources, tests adversariaux, revue indépendante avant tout usage financier réel.
- **Décentralisation** : mesurer la concentration du minage et les coûts réels d'un nœud, ne pas déduire l'accessibilité du seul algorithme PoW.
- **Intégrité économique** : émission et frais calculés de façon déterministe, vérification de l'offre sur chaque branche, gestion explicite des réorganisations.
- **Disponibilité** : synchronisation multi-nœuds, résistance aux partitions, saturation et redémarrages, reprise après panne.
- **Maintenabilité** : modules séparés, documentation versionnée, tests reproductibles, changements de consensus gouvernés.
- **Confidentialité** : définir séparément les menaces réseau, transactions, portefeuille et minage ; ne pas promettre d'anonymat absolu.
- **Compatibilité** : versions réseau, migrations d'état et formats de stockage explicites, jamais implicites.

## 3. Périmètre implémenté du testnet (non mainnet)
- Rust : `aurora-core`, `aurora-consensus`, `aurora-node`, `aurora-wallet`.
- Comptes, soldes et nonces ; transactions v3 signées ML-DSA-65 ; SHA-256 pour les hashes et PoW.
- En-têtes canoniques, racines et preuves Merkle, sélection au travail cumulé, retarget, MTP et réorganisations.
- Cible de 60 s par bloc ; taille maximale de bloc 4 MiB ; difficulté initiale de test très faible.
- Stockage persistant, index paginés et checksummés, restauration et tests de reprise.
- Transport P2P TCP à pairs configurés, découverte limitée opt-in, wallet CLI et tests multi-nœuds bornés.
- Émission brute maximale de 500 000 000 AUR à 8 décimales ; genèse vide ; première récompense 500 AUR ; 5 % trésorerie expérimentale / 95 % mineur ; frais 50 % mineur / 50 % brûlés. Les détails exacts sont définis dans `docs/PROTOCOL.md` et `docs/TESTNET_DECISIONS.md`.

## 4. Hors périmètre actuel
- Mainnet, fonds réels, promesses de sécurité, audit humain indépendant.
- Trésorerie dépensable et gouvernance opérationnelle.
- DID, récupération de compte par gardiens, recyclage ou confiscation de soldes.
- Confidentialité native des montants, expéditeurs ou destinataires ; Tor/I2P et Dandelion++ natifs.
- PoW memory-hard, ASERT, libp2p/Kademlia, application mobile, explorer et site finalisés.
- Compatibilité matérielle des wallets garantie ou résistance post-quantique du **protocole entier**.

## 5. Critères d'acceptation généraux
Chaque fonctionnalité possède une spécification et un modèle de menace, des invariants testables, des cas adversariaux, des mesures de performance, un plan de compatibilité/migration et un retour arrière pour l'expérimentation. Aucun objectif de recherche ne devient exigence de consensus sans décision explicite.

## 6. Points de divergence à conserver ouverts
L'historique évoquait d'autres plafonds monétaires, une évolution possible du PoW et des exigences d'anonymat renforcées. Le testnet exécute aujourd'hui des règles précises différentes. Ne pas modifier l'une ou l'autre sans rapprocher le Word original, l'historique Git et le registre des décisions.

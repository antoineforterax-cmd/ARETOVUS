# ARETOVUS — Cahier des charges consolidé

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
- Rust : `aretovus-core`, `aretovus-consensus`, `aretovus-node`, `aretovus-wallet`.
- Comptes, soldes et nonces ; transactions v3 signées ML-DSA-65 ; SHA-256 pour les hashes et PoW.
- En-têtes canoniques, racines et preuves Merkle, sélection au travail cumulé, retarget, MTP et réorganisations.
- Cible de 60 s par bloc ; taille maximale de bloc 4 MiB ; difficulté initiale de test très faible.
- Stockage persistant, index paginés et checksummés, restauration et tests de reprise.
- Transport P2P TCP à pairs configurés, découverte limitée opt-in, wallet CLI et tests multi-nœuds bornés.
- Émission brute maximale de 500 000 000 ARETOVUS à 8 décimales ; genèse vide ; première récompense 500 ARETOVUS ; 5 % trésorerie expérimentale / 95 % mineur ; frais 50 % mineur / 50 % brûlés. Les détails exacts sont définis dans `docs/PROTOCOL.md` et `docs/TESTNET_DECISIONS.md`.

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


## 7. Protection contre la perte de fonds, les erreurs et les arnaques (objectif prioritaire de recherche)

ARETOVUS vise un **parcours de sécurité volontaire** qui aide à prévenir la perte accidentelle de coins et certaines fraudes, sans promettre l'annulation d'une transaction irréversible ni confier les fonds à un administrateur central.

### Protection contre la perte d'accès
- Sauvegarde et restauration du portefeuille, y compris en cas de perte ou destruction de l'appareil.
- Étude d'un coffre de récupération et de gardiens choisis par l'utilisateur, avec délais, notifications et protections contre la collusion.
- Séparer récupération de l'accès et récupération d'un transfert déjà confirmé ; cette dernière n'est pas garantie.
- Aucun tiers ne doit pouvoir récupérer ou saisir les fonds par défaut.

### Protection contre les erreurs et arnaques
- Vérification claire du destinataire, du réseau, du montant et des frais avant signature.
- Alertes sur changements d'adresse, demandes de signature inhabituelles et destinations potentiellement risquées, avec limites explicites de la détection.
- Confirmation renforcée, limites volontaires et délais optionnels pour les transferts importants ; prévoir des mécanismes de révocation **avant** exécution lorsqu'un délai est choisi.
- Journal local des autorisations et notifications sans exposer les secrets ni compromettre la confidentialité.

### Contraintes
Ces protections doivent préserver l'autonomie de l'utilisateur, être facultatives lorsque possible, rester compatibles avec la confidentialité visée et ne pas créer de porte dérobée ou de gel arbitraire. Les mécanismes de récupération impliquant le consensus ne sont **pas implémentés** ; ils nécessitent une conception et une validation spécifiques.


## 8. Différenciation stratégique : confidentialité maximale, données minimales et performance

**Statut : vision prioritaire du projet, à concevoir et mesurer ; aucune garantie d'anonymat absolu.**

ARETOVUS doit chercher à réunir deux atouts forts : **la protection contre la perte et les arnaques** (section 7) et **une confidentialité poussée dès la conception**. Ces deux piliers doivent fonctionner ensemble sans rendre le réseau inutilement lourd, lent ou centralisé.

### Principe de minimisation
- Le portefeuille ne doit collecter, conserver ou transmettre que les informations nécessaires à l'opération demandée et à la sécurité de l'utilisateur.
- Distinguer les données privées du portefeuille, les métadonnées réseau, les preuves nécessaires au consensus et les données réellement publiques.
- Éviter la publication de données personnelles, de soldes, de relations entre adresses et d'historique financier lorsqu'une architecture cryptographique vérifiable permet de s'en passer.
- Éviter les comptes centraux et la télémétrie identifiante obligatoires ; prévoir des journaux techniques sobres et sans secrets.
- Préserver la possibilité de vérifier indépendamment les règles monétaires, l'absence de doubles dépenses et la validité des transactions.

### Confidentialité de bout en bout à étudier
- Confidentialité des montants, des expéditeurs et des destinataires, selon un modèle de menace explicite.
- Protection de l'adresse IP et des corrélations de diffusion ; étude de Tor/I2P et des stratégies de relais.
- Protection des métadonnées du portefeuille, y compris lors de la synchronisation et de la récupération.
- Compatibilité avec les signatures et preuves post-quantiques à examiner primitive par primitive.
- Possibilité d'une divulgation sélective sous contrôle de l'utilisateur, sans compromettre les autres utilisateurs.

### Performance et fonctionnement du réseau
- Rechercher un compromis mesuré entre confidentialité, frais, rapidité, bande passante, RAM, CPU et taille des preuves.
- Permettre la vérification par des nœuds indépendants ; ne pas supprimer les données ou engagements nécessaires au consensus.
- Établir des budgets de ressources et des benchmarks avant de retenir une architecture.
- Comparer plusieurs options techniques plutôt que promettre simultanément confidentialité maximale et vitesse maximale sans preuve.

### Articulation avec l'anti-perte et l'anti-arnaque
Les protections du portefeuille (alertes, limites volontaires, délais optionnels, récupération avec gardiens) ne doivent pas exiger de dévoiler systématiquement les transactions à un opérateur central. Étudier des vérifications locales, des autorisations limitées et des procédures de récupération préservant la vie privée. Aucun mécanisme ne garantit de récupérer un transfert irréversible déjà confirmé ni de détecter toutes les fraudes.

### Décision de périmètre
Il s'agit d'un **objectif produit prioritaire**, pas d'un changement immédiat du consensus testnet v3. L'architecture actuelle de comptes publics ne fournit pas cette confidentialité : une étude de faisabilité et éventuellement un nouveau modèle d'état seront nécessaires avant intégration.


## 9. Idées complémentaires — évolutions futures non prioritaires

**Statut : pistes facultatives, non validées, non planifiées et non implémentées.** Cette section est placée en fin de cahier des charges pour conserver les idées sans détourner les efforts des fondations, de la confidentialité et de la sécurité du réseau.

1. **Coffre-fort intelligent à plusieurs niveaux** : séparer les fonds du quotidien et l'épargne, avec plafonds volontaires, délais de sécurité et confirmations renforcées pour les opérations sensibles.
2. **Assistant anti-arnaque local** : avertissements avant signature, contrôle du destinataire et des permissions, analyse de risques sans télémétrie centralisée obligatoire. Aucune détection parfaite n'est promise.
3. **Récupération sociale facultative** : gardiens choisis par l'utilisateur, délais et protections contre la collusion ; sans donner aux gardiens un accès direct et unilatéral aux fonds. À coordonner avec la section 7.
4. **Paiements préparés hors ligne** : préparer et signer une transaction sans connexion, puis la transmettre ultérieurement ; la confirmation sur la blockchain nécessite toujours le réseau.
5. **Divulgation sélective** : pouvoir démontrer volontairement un paiement ou une information limitée sans dévoiler tout l'historique, sous réserve de faisabilité cryptographique et de compatibilité post-quantique.
6. **Portefeuille très simple d'utilisation** : protections compréhensibles, parcours guidés, affichage clair des frais et risques, sans masquer les conséquences d'une opération irréversible.

### Conditions avant toute priorisation

- Démontrer l'utilité réelle et éviter les doublons avec les fonctionnalités de sécurité déjà prévues.
- Évaluer les risques de centralisation, de confidentialité, de vol et de verrouillage des fonds.
- Mesurer les coûts en complexité, performances et maintenance.
- Ne rien ajouter au consensus ni à la roadmap active sans décision explicite et tests.

**Priorité actuelle inchangée :** consolider le protocole, sécuriser le testnet et étudier les deux axes différenciants majeurs — confidentialité et protection des fonds.

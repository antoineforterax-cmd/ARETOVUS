# AURORA — décisions actuelles et arbitrages ouverts

> **Réponses du responsable reçues le 31 octobre 2026 :** [12 décisions/orientations et études demandées](RESPONSES_2026-10-31.md). Les règles du testnet n'ont pas changé.

> **Questionnaire complet avec choix A/B/C :** [68 décisions à arbitrer, regroupées par domaine](DECISION_OPTIONS.md). Les recommandations ne sont ni des règles actives ni des décisions automatiquement approuvées. Les 88 IDs du [registre historique](../DECISION_REGISTER.md) restent accessibles pour l'historique mais ne sont pas 88 décisions encore ouvertes.


**Point de référence :** `main` au SHA `b76e0cff6d7e2ee6527cac2506ad005d7360ec4f`, situation consultée le 31 octobre 2026. **Document de décision et de suivi, sans modification de protocole.** Pour les règles exactes exécutées, le code à ce SHA et [TESTNET_DECISIONS.md](../TESTNET_DECISIONS.md) priment. L'ancien [registre phase 0](../DECISION_REGISTER.md) et la [baseline historique](../DECISION_BASELINE.md) sont conservés comme archives, **pas comme états courants**.

## Comment distinguer les statuts ?

- **Actif sur le testnet** : le code et ses tests contiennent un choix, **pas** nécessairement validé pour un futur mainnet.
- **Orientation produit retenue** : choix d'organisation de la feuille de route, sans changement des règles de consensus.
- **Proposition à arbitrer** : option recommandée, mais le responsable ne l'a pas approuvée comme nouvelle règle.
- **À démontrer** : exige mesure, étude de menace, audit ou test reproductible avant d'être fixé.

## A. Déjà décidé pour le testnet expérimental

| Domaine | Décision effectivement codée | Portée / limite |
| --- | --- | --- |
| Projet | Rust, workspace **cinq crates** | Pas un audit |
| Transactions | Comptes, soldes, nonces, encodage canonique | Profil de test, migration requise si changement |
| Signatures | **ML-DSA-65 v3** sur le testnet | Le protocole complet n'est pas post-quantique |
| Consensus | **PoW SHA-256**, cible statistique **60 s**, retarget borné par fenêtre, chaîne au travail cumulé | Pas d'anti-ASIC ni de garanties de temps de confirmation |
| Capacité | Taille maximale de bloc **4 MiB** | Débit réel à mesurer |
| Monnaie | Genèse vide, **500 millions d'AUR** maximum d'émission brute, **8 décimales**, émission proportionnelle décroissante | Alternative ancienne d'environ 105 M non adoptée pour ce testnet |
| Récompenses | **5 %** de l'émission à la trésorerie fixe, **95 %** au mineur ; **50 %** des frais au mineur, reliquat brûlé | Pas de gouvernance ni de dépense de la trésorerie |
| Nœud et portefeuille | P2P TCP borné, stockage v6, wallet CLI et sauvegardes chiffrées | Pas de résistance démontrée à tous les pairs hostiles |
| Licence | **MIT OR Apache-2.0**, au choix de l'utilisateur | Les avis des dépendances restent à respecter |

**Attention :** les lignes historiques `GOV-02` (« licence à comparer »), `POW-04` (« 600 secondes »), `TX-01` (« comptes ou UTXO ? ») et d'autres ont été dépassées **pour le testnet**. Elles ne doivent pas ressurgir comme des questions non résolues sans vérifier le code actuel.

## B. Direction de la première version : décisions de périmètre

| Décision d'organisation | Direction | Pourquoi |
| --- | --- | --- |
| P-V1-01 | **Consolider le noyau avant d'étendre le consensus** | Réduire les failles et faciliter l'audit |
| P-V1-02 | **Testnet v1 : transfert simple, nœuds, wallet CLI, sauvegardes, essais multi-PC** | Mettre à disposition une chaîne testable de bout en bout |
| P-V1-03 | **Explorateur en lecture seule avant l'application mobile ou les contrats intelligents** | Donner de la visibilité sans manipuler de clés dans le navigateur |
| P-V1-04 | **Confidentialité, récupération avancée, PoW alternatif : projets de recherche séparés** | Aucune activation silencieuse, tests de compatibilité nécessaires |
| P-V1-05 | **Pas de mainnet ni de fonds réels avant sécurité mesurée et revue externe** | La CI et les essais sur deux PC ne suffisent pas |

Ces orientations reprennent le [cahier des charges](CAHIER_DES_CHARGES.md) et la [feuille de route #123](https://github.com/antoineforterax-cmd/AURORA/issues/123). Elles guident l'organisation, **sans déclarer ces fonctions livrées**.

## C. Décisions encore ouvertes : propositions concrètes

| ID | Question | Recommandation à débattre | Pourquoi ne pas la figer immédiatement ? |
| --- | --- | --- | --- |
| **A-01** | Le plafond **mainnet** restera-t-il à 500 M ou passera-t-il à ~105 M ? | **Ne pas changer le 500 M du testnet v1** ; simuler séparément au moins les deux offres, distribution et sécurité économique | Changer une règle active crée un réseau incompatible ; aucun intérêt démontré à changer juste le nombre |
| **A-02** | Quel PoW et quel retarget pour le futur réseau ? | **Garder SHA-256 et le retarget actuels pour les tests**, comparer une option mémoire et ASERT sur branche isolée | Les performances, la résistance aux ASIC et la sécurité contre les manipulations temporelles restent à prouver |
| **A-03** | Confidentialité obligatoire ou facultative ? | Étudier la **confidentialité native par défaut** comme hypothèse de recherche, avec divulgation sélective volontaire ; ne pas l'annoncer | Le modèle actuel par comptes publics ne suffit pas ; preuves, coût, risques de fuite et compatibilité PQ non résolus |
| **A-04** | Qui contrôle les fonds de trésorerie ? | **Aucune dépense** tant que droits, gouvernance, traçabilité, conflits d'intérêts et récupération ne sont pas spécifiés | Un solde crédité n'est pas un budget utilisable ; les fonds de test ne prouvent pas un modèle de gouvernance |
| **A-05** | Peut-on restaurer un compte sans sa clé ? | Sauvegardes et restaurations chiffrées d'abord ; recherche d'une **récupération facultative** avec gardiens et délais, hors consensus actuel | Collusion, usurpation et réorganisation pourraient entraîner des pertes irréversibles |
| **A-06** | Quels frais et seuils antispam pour un réseau public ? | Mesurer d'abord les coûts des signatures ML-DSA, du mempool, de la bande passante et du stockage | Des frais arbitrairement bas facilitent le spam ; des frais trop hauts limitent l'usage |
| **A-07** | Quelles garanties de sécurité avant déploiement public ? | Définir des jalons vérifiables : réseau multi-hôtes, synchronisation à volume élevé, panne/disque plein, fuzzing long, audit humain | Les benchmarks de stockage ne prouvent pas l'endurance du réseau ; une courte campagne ne vaut pas 72 h |
| **A-08** | Quelles règles de mise à niveau et de récupération en cas de fork ? | Versionner chaque rupture, documenter migration, activation et retour arrière avant tout changement de consensus | Risque de divergence irréversible entre nœuds et wallets |

**Statut de A-01 à A-08 : propositions, pas décisions validées.** Le projet peut préparer des études et des simulations immédiatement, mais l'intégration d'une règle nouvelle attendra un accord explicite, les tests et la revue adaptés. **Ne pas présenter l'absence de décision comme une absence de travail : c'est une séparation volontaire entre recherche, choix et activation.**

## D. Ordre d'arbitrage recommandé

| Ordre | À produire | Critère permettant une vraie décision |
| --- | --- | --- |
| **Maintenant** | Rapport comparant ce qui est codé et ce qui reste ouvert ; correction des liens périmés | Chaque affirmation liée à un SHA ou à une spécification |
| **Avant testnet v1 public** | Pannes, horodatages, DoS, gros volumes de synchronisation, essais sur hôtes réels | Journaux de test et rapports de risques reproductibles |
| **Avant extension de protocole** | Comparatif PoW/retarget, confidentialité et modèle de récupération ; analyse économique | Modèle de menace, simulations, benchmark, compatibilité documentée |
| **Avant mainnet** | Choix économiques de long terme, gouvernance, procédure d'activation, audit indépendant | Validation explicite du responsable et preuves de correction |

Pour toute nouvelle décision consigner : **date, personne qui approuve, alternatives, impacts, statut, critères de test, preuves et SHA**, ainsi que la migration nécessaire. **Aucune ligne de ce document ne modifie le consensus.**

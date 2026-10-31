
# AURORA — toutes les décisions qui restent à arbitrer

**État : questionnaire d'orientation, pas un vote déjà validé.** Référence de la situation : `main` au SHA `b76e0cff6d7e2ee6527cac2506ad005d7360ec4f` (31 octobre 2026). La documentation d'accueil en [PR #125](https://github.com/antoineforterax-cmd/AURORA/pull/125) distingue les règles expérimentales déjà implémentées des choix futurs.

> **Décisions et orientations exprimées le 31 octobre 2026 :** [registre évolutif des réponses du responsable, y compris le projet d'interface de minage](RESPONSES_2026-10-31.md). Ce document conserve les autres options encore ouvertes.

## Comment répondre

Chaque ligne présente **trois options A / B / C**, puis une **préférence technique proposée (non validée)** et une priorité :
- **P0** : priorité de cadrage de la v1, décision de *périmètre* possible dès maintenant.
- **P1** : à arbitrer après étude/test ; peut modifier le consensus ou la sécurité.
- **P2** : recherche future ; ne doit pas ralentir la stabilisation du noyau.

Répondre par exemple `V01=B, E02=A, K03=C` ; « je diffère » est une réponse légitime. Une préférence proposée ne signifie pas mise en œuvre. **Une décision de consensus exige un accord explicite, des tests adversariaux, une compatibilité et une revue adaptées**, même si le responsable choisit l'option. **Aucune réponse n'autorise automatiquement la fusion de code**.

### Règles déjà actives sur le testnet — on ne les re-vote pas par erreur

Comptes + nonce ; Rust ; transactions testnet ML-DSA-65 ; PoW SHA-256 ; cible statistique 60 s ; retarget existant ; 4 MiB/bloc ; genèse sans préminage ; plafond brut **500 M AUR**, 8 décimales ; émission proportionnelle ; **5 % au compte de trésorerie expérimental** ; moitié des **frais** au mineur et moitié brûlée ; wallet CLI ; TCP P2P ; stockage v6 ; **MIT OR Apache-2.0**. Ce sont des caractéristiques **du profil testnet**, pas un mainnet sécurisé ni une décision irrévocable pour un futur protocole. Références : [consensus](../CONSENSUS.md), [économie](../TESTNET_DECISIONS.md), [protocole](../PROTOCOL.md).

## 1. Version, périmètre et expérience — V

| ID | Question | A | B | C | Proposition | Priorité |
| --- | --- | --- | --- | --- | --- | --- |
| V01 | Qu'appelle-t-on une « v1 complète » ? | CLI/nœuds stables | + explorateur lecture seule | + wallet graphique et mobile | **B**, GUI plus tard, y compris **interface de contrôle du minage** comme objectif différé | P0 |
| V02 | Public visé en premier ? | Développeurs/testeurs | Grand public | Les deux simultanément | **A** | P0 |
| V03 | Premier lancement public ? | Réseau local contrôlé | Testnet public supervisé | Mainnet | **A**, puis B sur critères | P0 |
| V04 | Ordre des composants produit ? | Noyau → explorer → GUI | GUI avant explorer | Mobile d'abord | **A** | P0 |
| V05 | Smart contracts généralistes ? | Dès v1 | Dans une future version isolée | Pas d'objectif prévu | **C** pour v1 | P0 |
| V06 | Paiements hors ligne/Bluetooth ? | Inclusion directe en v1 | Signature hors ligne, diffusion en ligne | R&D plus tard | **B** ; Bluetooth R&D | P2 |
| V07 | Expériences et comparaisons publiques ? | Rapports reproductibles | Démonstrations seulement | Marketing avant preuves | **A** | P0 |
| V08 | Critères de qualité de code ? | Lignes ajoutées | Tests, invariants, lisibilité et ressources | Nombre de fonctions | **B** | P0 |

## 2. Économie et incitations — E

| ID | Question | A | B | C | Proposition | Priorité |
| --- | --- | --- | --- | --- | --- | --- |
| E01 | Plafond du **futur mainnet** ? | 500 M AUR comme testnet | ~105 M AUR | Autre offre bornée justifiée | **Étudier A/B**, ne pas changer testnet | P1 |
| E02 | Distribution initiale mainnet ? | Aucun préminage | Préminage déclaré pour projet | Lancement avec autre distribution explicite | **A** | P1 |
| E03 | Calendrier d'émission durable ? | Formule testnet actuelle | Halvings/paliers | Autre courbe simulée | **A** comme référence, comparer B | P1 |
| E04 | Part de trésorerie future ? | 5 % d'émission | 0 % | Taux différent approuvé et borné | **Comparer A/B** | P1 |
| E05 | Part des frais brûlée ? | 50 % testnet | 0 % | Dynamique selon règles | **A** à tester économiquement | P1 |
| E06 | Frais minimum / antispam ? | Montant statique | Seuil selon coût mesuré | Aucun seuil | **B**, mesure d'abord | P1 |
| E07 | Frais de transaction calculés comment ? | Forfait | Taille + coût de vérification | Marché d'enchères | **B** à simuler | P1 |
| E08 | Maturité des récompenses minières ? | Immédiate (testnet) | Délai de confirmations | Verrouillage programmable | **Comparer A/B** par réorg | P1 |
| E09 | Fonds inactifs / portefeuilles perdus ? | Jamais confisqués | Redistribution automatique | Contrat volontaire opt-in | **A** ; C seulement si volontaire | P0 |
| E10 | « Pénalité » des gros détenteurs ? | Aucune | Taxe sur gros soldes | Limites volontaires wallet | **A**, B contournable et intrusif | P1 |

## 3. Consensus, paramètres et minage — C

| ID | Question | A | B | C | Proposition | Priorité |
| --- | --- | --- | --- | --- | --- | --- |
| C01 | PoW du futur réseau ? | SHA-256 actuel | Memory-hard existant | Alternative étudiée | **Garder A en v1**, benchmark B | P1 |
| C02 | Cible moyenne de bloc ? | 60 s du testnet | 120 s | 600 s | **A pour testnet**, mesurer forks | P1 |
| C03 | Retarget ? | Fenêtre actuelle | ASERT | LWMA/équivalent | **A pour v1**, comparer B/C | P1 |
| C04 | Difficulté initiale et bornes ? | Profil testnet | Calibration selon hashrate | Paramètres adaptatifs supplémentaires | **A en essai**, modéliser B | P1 |
| C05 | Taille maximale de bloc future ? | 4 MiB | Plus petite | Plus grande | **A en test**, mesurer coût réel | P1 |
| C06 | Horodatages adversariaux ? | Règle actuelle | Admission plus stricte | Autre règle versionnée | **Comparer A/B** avec vecteurs | P1 |
| C07 | Activation des nouvelles règles ? | Hauteur fixée | Activation coordonnée/versionnée | Changement non annoncé | **B** avec plan d'échec | P1 |
| C08 | Politique de forks/reorgs ? | Travail cumulé actuel | Finalité ajoutée | Checkpoints de confiance | **A**, étudier risque de reorg | P1 |
| C09 | Niveaux de confirmations affichés ? | Repères fixes | Risque selon contexte | Un seul niveau universel | **B**, simulations nécessaires | P1 |
| C10 | Résistance au minage concentré ? | PoW + diversité volontaire | Changement d'algorithme | Restrictions d'identité | **A mesuré**, étudier B, éviter C | P1 |

## 4. Réseau, stockage et disponibilité — N

| ID | Question | A | B | C | Proposition | Priorité |
| --- | --- | --- | --- | --- | --- | --- |
| N01 | Transport réseau v1 ? | TCP actuel borné | libp2p | Nouveau transport maison | **A** tant que sûr | P0 |
| N02 | Découverte de pairs ? | Pairs manuels | Bootstraps + découverte distribuée | Service central obligatoire | **A v1**, B en R&D | P1 |
| N03 | Traversée NAT/pare-feu ? | Configuration manuelle | Relais facultatifs | Serveur obligatoire | **A**, étudier B | P1 |
| N04 | Défense DoS des pairs ? | Quotas actuels | Budgets/priorités améliorés | Bannissement permanent par IP | **B après mesure** | P1 |
| N05 | Architecture du stockage ? | Stockage v6 | Base embarquée externe | Refonte complète | **A**, benchmark B avant choix | P1 |
| N06 | Nœud élagué ? | Aucun pour v1 | Pruning optionnel | Pruning par défaut | **B futur**, preuve de vérification | P2 |
| N07 | Nœud léger et SPV ? | Full node recommandé | SPV facultatif averti | SPV comme unique choix | **B**, préciser éclipse/trust | P1 |
| N08 | Matériel cible minimal ? | PC Windows/Linux | Raspberry Pi/mini-PC | Smartphone full node dès v1 | **A maintenant**, benchmark B | P1 |
| N09 | Mesure des performances ? | Chiffres isolés | CPU/RAM/disque/réseau/charge reproducibles | Débit théorique annoncé | **B** | P0 |
| N10 | Pannes, sauvegarde et récupération ? | Tests synthétiques | Tests 2 PC, disque plein et restauration hors hôte | Aucune campagne de panne | **B** avant testnet public | P0 |

## 5. Cryptographie, confidentialité et identité — K

| ID | Question | A | B | C | Proposition | Priorité |
| --- | --- | --- | --- | --- | --- | --- |
| K01 | Signature du futur protocole ? | ML-DSA-65 natif | Hybride classique + PQ | Autre standard évalué | **A** comme base testnet, comparer B | P1 |
| K02 | Niveau de sécurité ML-DSA ? | 44 | 65 | 87 | **B (65)** tant que mesures favorables | P1 |
| K03 | Migration cryptographique ? | Algorithme figé | Profils versionnés + plan de migration | Réécriture ad hoc | **B** | P1 |
| K04 | Confidentialité des transactions ? | Comptes publics | Transactions privées par défaut | Privacy optionnelle | **Étude B/C**, sans activation | P1 |
| K05 | Preuves de confidentialité ? | Notes/nullifiers + ZK | Ring signatures | Mélange de moyens non spécifiés | **Comparer**, notamment compatibilité PQ | P1 |
| K06 | Masquage d'IP / métadonnées ? | TCP direct seul | Tor/I2P facultatifs étudiés | Transport anonyme obligatoire | **B d'abord**, mesurer les fuites | P1 |
| K07 | Adresses réutilisables ? | Adresses publiques actuelles | Adresses de réception uniques | Adresses furtives cryptographiques | **B comme objectif UX**, C en R&D | P1 |
| K08 | Divulgation sélective ? | Aucune | Preuves volontaires par utilisateur | Accès privilégié d'administrateur | **B en recherche**, éviter C | P2 |
| K09 | Dérivation et sauvegarde des clés ? | Fichiers chiffrés indépendants | Graines hiérarchiques compatibles PQ | Comptes confiés au serveur | **A en v1**, comparer B | P1 |
| K10 | Confidentialité vs audit de l'offre ? | Montants tous publics | Preuves publiques d'intégrité sans montants visibles | Faire confiance à un opérateur | **B comme objectif de recherche** | P1 |

## 6. Wallet, récupération et UX de sécurité — W

| ID | Question | A | B | C | Proposition | Priorité |
| --- | --- | --- | --- | --- | --- | --- |
| W01 | Première interface utilisateur ? | CLI actuelle | Application PC | Mobile d'abord | **A v1**, B étape suivante | P0 |
| W02 | Qui conserve les clés ? | Appareil utilisateur | Service custodial facultatif | Serveur obligatoire | **A** | P0 |
| W03 | Authentification des RPC wallet ? | Local uniquement | Authentification sur accès distant facultatif | Port ouvert sans contrôle | **A**, B après audit | P1 |
| W04 | Intention avant signature ? | Vérification explicite | Confirmation sur appareil indépendant | Signature automatique silencieuse | **A en v1**, viser B | P1 |
| W05 | Sauvegardes ? | Manuelle chiffrée | Sauvegarde chiffrée guidée/multi-support | Sauvegarde cloud imposée | **B** UX, sans serveur imposé | P1 |
| W06 | Récupération après perte de clé ? | Sauvegarde uniquement | Gardiens volontaires + délai | Administrateur récupère les fonds | **A en v1**, rechercher B | P1 |
| W07 | Délai de sécurité des gros paiements ? | Aucun | Optionnel et révocable avant exécution | Obligatoire pour tous | **B en recherche**, hors consensus actuel | P2 |
| W08 | Protection contre les arnaques ? | Avertissements locaux | Blocage central permanent | Aucune confirmation | **A** et affichage indépendant futur | P1 |
| W09 | Compatibilité wallet matériel ? | Aucune promesse pour v1 | Pilote et revue dédiés | « Compatible Ledger » sans tests | **B plus tard**, ne pas promettre C | P2 |
| W10 | Plateformes clients ? | Windows/Linux/macOS | Android avec wallet léger | Toutes les plateformes dès v1 | **A puis B** | P1 |

## 7. Trésorerie, gouvernance, production et audits — G

| ID | Question | A | B | C | Proposition | Priorité |
| --- | --- | --- | --- | --- | --- | --- |
| G01 | Qui peut dépenser la trésorerie ? | Personne sur testnet | Multisig + règles publiques | Clé unique non auditée | **A aujourd'hui**, B sous étude | P1 |
| G02 | Votes/gouvernance ? | Décisions documentées du responsable | Gouvernance ouverte après lancement | Vote automatique sur chaque détail | **A en v1**, B à concevoir | P1 |
| G03 | Traçabilité du budget ? | Aucun registre | Publication des autorisations et comptes | Opérations opaques | **B avant toute dépense** | P1 |
| G04 | Licences de projet ? | MIT OR Apache-2.0 actuelle | Licence unique future | Copie de licences non compatibles | **A**, vérifier dépendances | P0 |
| G05 | Revue et intégration de code ? | Petite PR + CI + accord | Fusion automatique sans contrôle | Branches longues non testées | **A** | P0 |
| G06 | Audit avant fonds réels ? | Revue indépendante + correctifs | CI seule | Auto-déclaration de sécurité | **A**, condition impérative | P0 |
| G07 | Publication des binaires ? | Archives non signées expérimentales | Provenance, checksums et signatures | Binaire sans origine | **B avant réseau public** | P1 |
| G08 | Gestion des incidents ? | Plan, révocation/alerte, publication | Réaction au cas par cas | Cacher les incidents | **A** | P1 |
| G09 | Politique des dépendances ? | Analyse de licence et vulnérabilités + verrouillage | Mises à jour automatiques non revues | Gel définitif | **A** | P0 |
| G10 | Comment accueillir des contributeurs ? | Guide unique, issues ciblées, revue reproductible | Documentation dupliquée partout | Contribution sans tests | **A** | P0 |

## Comment décider sans se tromper ?

1. **D'abord P0** : les objectifs de produit et la méthode de travail se choisissent rapidement, sans casser les règles de la chaîne.
2. **Ensuite P1 par lots** : pour chaque changement de protocole, produire **une spécification, un modèle de menace, une simulation, des mesures, des tests adversariaux, un impact de compatibilité et une décision signée du responsable**. Si les preuves manquent, noter « à expérimenter ».
3. **Enfin P2** : des directions intéressantes à explorer en parallèle, mais **pas des blocages pour la v1**.
4. Ne pas créer de « choix par défaut » irréversible. **Les options recommandées dans ce tableau restent des recommandations**.

### Ancien registre : les 88 références ne sont pas 88 choix vierges

Le registre historique [DECISION_REGISTER.md](../DECISION_REGISTER.md) contient des IDs GOV, TX, POW, MON, FEE, CRY, NET, WAL, OPT, ENV et TST, avec **des tableaux répétés et des constats datés**. Il ne faut pas compter chaque ligne historique comme une décision encore à prendre : des paramètres sont codés et d'autres propositions sont abandonnées ou remplacées. Les lignes historiques sont conservées telles quelles pour la traçabilité. Cette liste nouvelle est **l'outil de décision actuel** ; les règles effectivement exécutées restent dans le protocole au SHA choisi.

**Note de portée :** les 68 entrées du présent questionnaire couvrent les questions structurantes ; les sous-choix de formats binaires, tailles exactes, bornes ou algorithmes doivent être spécifiés dans leurs propositions techniques, et non approuvés aveuglément ici.

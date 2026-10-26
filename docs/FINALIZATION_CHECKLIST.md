# AURORA — fiche complète des travaux restants

**État au 26 octobre 2026 :** testnet expérimental v3, branche `feature/emission-schedule-v0`, [PR #2](https://github.com/antoineforterax-cmd/AURORA/pull/2) en brouillon et non fusionnée. Aucun fonds réel; aucune affirmation de sécurité mainnet. La [CI du dernier correctif de code](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40405603954) a passé formatage, Clippy, compilation, tests Rust et smoke multi-nœuds. Les campagnes longues de fuzzing et de soak de cette révision sont encore en cours au moment de cette mise à jour. Ce texte est une liste de critères à satisfaire, pas une promesse de date.

## Ce qui existe déjà

- Consensus de comptes testnet v3 : encodage canonique, signatures ML-DSA-65, contrôle des nonces/soldes, blocs avec engagement Merkle, PoW SHA-256, retarget borné, temps médian, travail cumulé, branches secondaires et réorganisations.
- Économie **codée** : genèse sans préminage; plafond brut de 500 millions AUR; émission proportionnelle décroissante (premier bloc : 500 AUR); 5 % de chaque émission au compte testnet de trésorerie et 95 % au mineur; moitié des frais au mineur, reliquat brûlé. Le burn ne réouvre pas le plafond. Voir [les décisions du testnet](TESTNET_DECISIONS.md).
- Stockage v5 par blocs immuables et index checksummés, relecture v4, reprise depuis un index précédent, sauvegarde/restauration testées localement. Verrou exclusif de dossier pour mine, sync et serve.
- Nœud TCP à pairs configurés, synchronisation, propagation de blocs, récupération périodique du mempool, API wallet/SPV. Wallet CLI avec clés chiffrées par défaut, sauvegarde/restauration et contrôle de l’empreinte réseau.
- Archives privées non signées Windows, Linux et macOS; essai du verrou sur ces trois systèmes. Le porteur a miné sur Windows jusqu’à 367 et vérifié localement le dossier à 311. La synchronisation manuelle vers le second dossier n’a été démontrée qu’à la hauteur 3, parce qu’un ancien serveur occupait toujours le port.
- Tests unitaires, smoke, essais à trois processus, fuzzing court et [audit RustSec](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40402139677) des 78 dépendances verrouillées sans vulnérabilité signalée lors de cette exécution.

## 1 — Corriger les risques du cœur avant un testnet durable

| Priorité | Travail | Preuve de fin attendue |
| --- | --- | --- |
| P0 | [Horodatages futurs et retarget — issue #3](https://github.com/antoineforterax-cmd/AURORA/issues/3). Une borne d’admission de deux heures est codée pour mine, serve et sync, avec test de page refusée sans persistance. Compléter les simulations de dérive/réorg, le contrôle des en-têtes SPV et la procédure de mise à niveau. | Vecteurs acceptés/refusés, simulations de dérive et de réorg, fingerprint/version cohérent, CI et smoke verts. Toute rupture de consensus doit indiquer clairement le sort des anciennes données testnet. |
| P0 | [Saturation des connexions P2P — issue #4](https://github.com/antoineforterax-cmd/AURORA/issues/4). Délai de handshake de deux secondes et plafond de quatre sessions par IP codés; tester huit connexions hostiles, le neuvième pair légitime et l’équité lorsque plusieurs nœuds partagent une IP ou que plusieurs IP attaquent. | Essai reproductible de huit connexions hostiles et d’un neuvième pair légitime; délais, ressources et récupération mesurés; tests wallet et sync toujours verts. |
| P0 | [Plafond de 100 000 blocs connus — issue #5](https://github.com/antoineforterax-cmd/AURORA/issues/5). La [conception v6](STORAGE_V6_DESIGN.md) décrit l’inventaire append-only et l’index paginé; un ordre d’arrivée stable en mémoire et son test sont codés. Implémenter ensuite le stockage et la synchronisation pour franchir cette borne sans refuser un bloc valide. Traiter la croissance des états et les branches secondaires sans changer silencieusement les réorganisations possibles. | Test à petite borne puis essai de volume, redémarrage et sync après franchissement; mesures de mémoire, disque et temps de rejeu; règles de réorg documentées. |
| P1 | Lectures réseau coûteuses. L’index des transactions confirmées et l’ordre des blocs ont été ajoutés après la [contre-revue IA](AI_COUNTER_REVIEW.md). | Mesurer la latence de soumission d’un bloc pendant des demandes répétées de preuves/status absents et de pages de blocs, avec une chaîne volumineuse; fixer des budgets par pair si nécessaire. |
| P1 | Pairs hostiles et partitions. Étendre les essais aux inventaires mensongers, pages tronquées ou contradictoires, déconnexions répétées, réponses lentes, bloc/transaction invalides, réorgs concurrents et épuisement des quotas. | Tests déterministes qui vérifient rejet, absence de publication partielle, disponibilité des autres pairs et convergence après partition. |
| P1 | Pannes de stockage réelles. Le test actuel simule un échec d’écriture; il ne prouve pas un disque plein, une panne électrique ou la restauration hors machine. | Procédures exécutées sur machines de test, sauvegarde complète hors hôte, comparaison hauteur **et** hash, reprise après corruption et rapport des pertes possibles. |

Le [rapport d’essais](TESTNET_VALIDATION_REPORT.md) distingue les preuves déjà obtenues de ces scénarios manquants. La limite de 100 000 blocs et la saturation encore possible des huit workers interdisent de qualifier aujourd’hui ce réseau de testnet public durable.

## 2 — Contre-revues et assurance de sécurité

1. **Contre-revue IA initiale : faite, mais partielle.** Un second agent a lu le code sans le modifier et a relevé quatre risques; un correctif de lectures réseau est passé en CI. Les trois autres ont leurs issues et leurs tests à écrire. Voir [le rapport de contre-revue](AI_COUNTER_REVIEW.md).
2. **Contre-revues IA répétées : à faire après les correctifs.** Repartir d’un contexte vierge et d’un commit épinglé. Demander séparément une revue consensus/économie, une revue cryptographie/wallet, une revue stockage, et une revue réseau/SPV. Chaque constat doit préciser chemin, scénario, gravité, test reproductible, correction et résultat de réexamen. Une revue par IA peut manquer les mêmes classes de défauts qu’une autre.
3. **Outils automatiques : à approfondir.** Maintenir RustSec et CI; conserver et enrichir les corpus libFuzzer des décodeurs et preuves, exécuter des campagnes de plusieurs heures/jours, mesurer la couverture, examiner les crashs, ajouter des tests différentiels ou invariants sur émission, état et reorg. Aucun résultat vert n’est une preuve générale de sécurité.
4. **Revue humaine indépendante : non faite.** Préparer un commit stable, le [dossier de revue](INDEPENDENT_REVIEW.md), les spécifications et les résultats reproductibles; faire examiner le tout par une personne compétente distincte des auteurs; recevoir un rapport écrit, corriger les constats élevés/critiques et faire vérifier les corrections. Le dépôt privé demande un accès accordé par son propriétaire. Une voie possible est une collaboration GitHub avec un reviewer volontaire; aucun reviewer ni budget n’est engagé aujourd’hui.

**Porte de sécurité :** aucune déclaration « sécurisé pour des fonds réels » ou « mainnet prêt » avant résolution des défauts bloquants, revue humaine indépendante et nouveaux essais complets.

## 3 — Terminer les choix de protocole et de gouvernance

- **Économie mécanique : déjà codée.** Garder des vecteurs de référence pour le plafond de 500 millions, l’émission lissée, le zéro préminage, le 95/5, le partage fixe 50/50 des frais, le burn et les réorganisations. Tout taux de burn dynamique serait une nouvelle règle de consensus, pas une option de l’interface.
- **Trésorerie : décision encore ouverte.** Le compte fixe reçoit 5 % mais aucun contrat, administrateur, procédure de dépense, contrôle public ou gouvernance n’existe. Définir qui propose, autorise et publie une dépense, comment les clés sont protégées, et que se passe-t-il lors d’un reorg. Ne pas présenter les fonds comme disponibles tant que cette politique n’existe pas.
- **DID, défis et récupération : conception non activée.** [La proposition](DID_RECOVERY_DESIGN.md) décrit des gardiens, seuils et délais. Il reste à décider si elle entre dans une version suivante, spécifier l’identité stable des comptes, traiter collusion/censure/perte de clés, créer un profil compatible, implémenter et auditer. Le v3 ne récupère pas une clé perdue.
- **Inactivité : aucune saisie ni recyclage.** Si ce mécanisme reste souhaité, il faut une décision explicite sur les droits de propriété, seuils, preuves, recours et risques d’abus, puis une spécification et une revue distinctes.
- **Mises à niveau :** fixer règles de compatibilité, identifiants de protocole, activation, migration ou remise à zéro du testnet, et rollback. Les archives anciennes et nouvelles ne doivent pas se connecter silencieusement avec des règles différentes.
- **Cryptographie :** continuer la revue de l’implémentation ML-DSA-65 et des domaines de signature. ML-DSA protège l’autorisation des transactions; le PoW et les hashes sont en SHA-256. Ne pas qualifier l’ensemble de « 100 % post-quantique » sans analyse précise.

## 4 — Stabiliser et distribuer le testnet

- Fixer une version de testnet et une toolchain Rust reproductible, les paramètres et un commit; publier notes de version, sommes de contrôle et, avant une diffusion large, une provenance/signature des archives. Les artefacts CI actuels sont privés, non signés et temporaires.
- Choisir des opérateurs et héberger plusieurs nœuds semences sur des machines distinctes. Configurer réseau, pare-feu, journaux, supervision, sauvegardes et gestion des incidents. Aucune infrastructure publique officielle n’existe dans le dépôt.
- Exécuter une campagne multi-hôtes Windows/Linux/macOS prolongée : transactions, minage, synchronisation depuis zéro, déconnexion/reconnexion, partitions et reorgs. Relever pour chaque nœud hauteur, hash du sommet, travail cumulé, temps de rattrapage, disponibilité et usage disque/mémoire. Le retour actuel du porteur prouve l’exécution de binaires sur plusieurs OS, pas leur convergence entre hôtes.
- Vérifier sur Windows le rattrapage du second dossier de 3 à au moins 367 après arrêt du serveur ancien et redémarrage sur le dossier miné. Utiliser le [guide](TESTNET_GUIDE.md) et les [procédures d’exploitation](TESTNET_OPERATIONS.md).
- Geler le protocole expérimental seulement après correction et répétition des CI, soak et fuzzing; prévoir une procédure de mise à jour, sauvegarde/restauration, diagnostic et signalement d’incident.

## 5 — Interfaces et services, dès que l’API et le réseau sont stabilisés

Ces produits peuvent être prototypés sur le testnet après les corrections P0 et la stabilisation de l’API. Ils ne doivent pas recevoir de fonds réels tant que la porte de sécurité n’est pas franchie.

| Produit | Déjà là | Reste à construire et vérifier |
| --- | --- | --- |
| **Portefeuille quotidien** | CLI Rust, clés ML-DSA-65 chiffrées, sauvegarde/restauration, envoi, solde et preuve SPV. | Interface Windows/macOS/Linux puis mobile, création/restauration guidées, protection des secrets par système, QR/URI aurora://, carnet d’adresses, frais lisibles, états en attente/confirmés, avertissement de reorg, essais d’ergonomie et de perte d’appareil. Ne jamais exporter la graine en clair par défaut. |
| **Explorateur** | En-têtes et preuves disponibles via le nœud; aucun service Web. | Indexeur en lecture seule, API paginée, pages blocs/transactions/adresses, recherche, représentation de l’émission/frais/burn, traitement des reorgs et des branches orphelines, vérification que l’API ne bloque pas le consensus, tests sur plusieurs nœuds. |
| **Site de présentation** | Documentation dans le dépôt. | Site expliquant but d’AURORA, fonctionnement, économie réellement codée, installation du nœud/minage, statut expérimental, risques, sources et versions téléchargeables. Éviter les promesses de rendement ou de valeur. |
| **Vente/échange** | Aucun marché ni service de garde. | Choisir d’abord le modèle : vente directe entre utilisateurs, carnet d’ordres testnet, ou intermédiaire qui détient des fonds. Spécifier propriété des ordres, appariement, règlement, annulation, frais, litiges, garde des clés, anti-fraude et responsabilités. Faire une analyse juridique et opérationnelle avant toute vente réelle ou intégration de paiement. Une « bourse générale » ne peut pas être supposée exister simplement parce qu’un ordre de vente est créé. |

**Ordre de réalisation :** corriger le cœur et l’API → prototype wallet et explorateur sur testnet → site explicatif et documentation de téléchargement → seulement ensuite prototype de marché de test. Le développement d’interface peut préparer des maquettes plus tôt, mais ses données et promesses doivent correspondre aux règles effectivement exécutées.

## Critère final et dépendances externes

AURORA ne sera « finalisée » que si tous les points suivants sont prouvés sur la même version gelée : défauts P0 corrigés, essais hostiles et multi-hôtes prolongés, stockage au-delà de l’horizon visé, fuzzing/mesures conservés, économie et gouvernance décidées, revue humaine indépendante avec corrections revérifiées, procédures d’exploitation, wallet/explorateur/site testés, et traitement séparé de la vente réelle.

Ce qui nécessite une intervention extérieure : choix et accès d’un reviewer humain au dépôt privé; machines/hébergement et opérateurs du réseau public de test; décisions de gouvernance de la trésorerie, DID et inactivité; analyse juridique pour une vente réelle. L’agent peut continuer le code, les tests, la documentation et les prototypes sans attendre ces éléments, mais ne peut pas fabriquer leurs preuves ni décider à la place du propriétaire.

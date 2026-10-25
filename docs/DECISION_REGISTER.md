# DECISION_REGISTER.md — AURORA

**Version du registre :** 1.1 (cahier des charges v4.0 + corrections v4.1)
**Dernière mise à jour :** 2026-10-24
**Décideur unique :** le porteur du projet.

> **Aucun résultat de benchmark, de simulation ou d'audit n'existe à ce jour.**
> **Aucun test du prototype n'a été exécuté à ce jour** (Rust n'était pas installé dans l'environnement de rédaction) : voir annexe B.
> Ce registre ne contient que des décisions à prendre, des hypothèses et des méthodes de vérification.
> Les chiffres marqués « calcul indicatif » sont de l'arithmétique, pas des mesures.

---

## 1. Règles d'utilisation

1. **Seul le porteur du projet peut passer une décision à un statut « VALIDÉ ».** Un assistant (IA ou humain) peut proposer, comparer, mesurer ; il ne valide jamais.
2. Toute décision qui touche le consensus (tag `C`) est **critique** : la changer plus tard revient à créer un fork.
3. Pour passer à **VALIDÉ POUR LE PROTOCOLE**, il faut : une justification écrite, un résultat vérifiable (rapport, test, benchmark avec la commande pour le reproduire), et une revue indépendante adaptée au risque.
4. Une décision n'est jamais supprimée : elle devient **DÉPRÉCIÉE** avec la raison et la date.
5. Chaque changement de statut est noté dans le journal (section 9).
6. Si le code contredit le registre, c'est le code qui a tort jusqu'à décision contraire.

### Statuts autorisés (seuls ces 7)

| Statut | Sens |
|---|---|
| PROPOSÉ | Idée de départ, pas encore comparée |
| À COMPARER | Plusieurs options restent ouvertes |
| À SIMULER | Demande des résultats quantitatifs (simulation) |
| À TESTER | Demande un prototype ou un benchmark |
| VALIDÉ POUR LE PROTOTYPE | Retenu pour les expériences, révisable |
| VALIDÉ POUR LE PROTOCOLE | Documenté, testé, revu, accepté |
| DÉPRÉCIÉ | Abandonné, conservé pour l'historique |

Passage normal : PROPOSÉ → (À COMPARER / À SIMULER / À TESTER) → VALIDÉ POUR LE PROTOTYPE → VALIDÉ POUR LE PROTOCOLE. Un retour en arrière est permis avec justification.

### Tags

- `C` : touche le **consensus** (changement = fork)
- `S` : sécurité ou cryptographie
- `U` : expérience utilisateur ou portefeuille
- `P` : politique locale du nœud (n'affecte pas le consensus)
- `O` : outillage, processus
- `V1` : dans le périmètre de la première version / `+` : plus tard

---

## 2. Tableau maître

### GOV — Processus et gouvernance technique

| ID | Décision | Statut | Point de départ / options | Condition de validation | Phase | Dépend de | Tags |
|---|---|---|---|---|---|---|---|
| GOV-01 | Langage Rust stable | PROPOSÉ | Rust stable, version épinglée dans `rust-toolchain.toml` | Build reproductible, CI verte | 1 | — | O V1 |
| GOV-02 | Licence du dépôt | À COMPARER | Licence permissive (MIT / Apache-2.0) ou autre | Choix explicite, compatible avec les dépendances (`cargo deny`) | 0a | — | O V1 |
| GOV-03 | Politique de dépendances | PROPOSÉ | Toute dépendance justifiée ici ; `cargo audit` et `cargo deny` bloquants en CI | CI en place | 1 | GOV-01 | S O V1 |
| GOV-04 | Traitement du code généré par IA | PROPOSÉ | Aucun code de consensus ou de crypto fusionné sans tests réellement exécutés et relecture humaine | Règle écrite dans `CONTRIBUTING.md` | 0a | — | S O V1 |
| GOV-05 | Procédure de modification du consensus | PROPOSÉ | Proposition, implémentation de référence, tests, testnet, décision (cahier §24) | `MIGRATION_POLICY.md` rédigé | 0a | — | C V1 |
| GOV-06 | Périmètre de la v1 | PROPOSÉ | Cahier §2.3 : ni contrats généralistes, ni DeFi, ni confidentialité avancée | Validé par le porteur | 0a | — | O V1 |

### TX — Transactions, formats, identifiants

| ID | Décision | Statut | Point de départ / options | Condition de validation | Phase | Dépend de | Tags |
|---|---|---|---|---|---|---|---|
| TX-01 | Modèle UTXO ou comptes | À COMPARER | Voir fiche F-01 | Prototype jetable + analyse écrite | 0b | — | C V1 |
| TX-02 | Format de sérialisation canonique | À COMPARER | Format binaire minimal défini par spécification, ou bibliothèque existante. Un format générique ne garantit pas forcément un encodage unique : à vérifier. | Une seule représentation valide par objet ; rejet des encodages non canoniques ; vecteurs de test | 0b / 1 | TX-01 | C V1 |
| TX-03 | Construction du hachage : SHA-256 simple, double ou autre ; séparation de domaines | À COMPARER | Cahier §5.3 | Document + vecteurs de test | 0b | TX-02 | C S V1 |
| TX-04 | Arbre de Merkle (nombre impair de feuilles, ambiguïtés de structure) | À COMPARER | | Tests de propriétés | 1 | TX-03 | C V1 |
| TX-05 | Identifiant de transaction (inclut-il les signatures ?) | À COMPARER | | Malléabilité testée | 1 | TX-01, TX-02, CRY-01 | C S V1 |
| TX-06 | Unité minimale et nombre de décimales | À COMPARER | Voir F-05 | Cohérent avec le simulateur d'émission | 0b | MON-03 | C V1 |
| TX-07 | Type entier des montants et opérations vérifiées | PROPOSÉ | Entiers non signés, jamais de flottants. Avec 8 décimales : 105 M × 10^8 = 1,05 × 10^16, ce qui tient dans un entier 64 bits (max ≈ 1,8 × 10^19) — calcul indicatif | Tests de dépassement | 1 | TX-06 | C S V1 |
| TX-08 | Format d'adresse (préfixe réseau, contrôle d'intégrité, version, algorithme encodé) | À COMPARER | | Vecteurs de test ; une faute de frappe est rejetée | 1 | CRY-01, CRY-06 | C U V1 |
| TX-09 | Identifiants de réseau et domaine de signature par réseau | PROPOSÉ | Regtest, testnet, mainnet distincts | Une signature d'un réseau n'est pas valide sur un autre (test) | 1 | ENV-01 | C S V1 |
| TX-10 | Protection contre la relecture | À COMPARER | Dépend du modèle (TX-01) | Tests de rejeu | 1 | TX-01 | C S V1 |

### POW — Consensus et minage

| ID | Décision | Statut | Point de départ / options | Condition de validation | Phase | Dépend de | Tags |
|---|---|---|---|---|---|---|---|
| POW-01 | Algorithme de PoW | À COMPARER | SHA-256 / orienté mémoire / autre. Voir F-02 | Rapport comparatif + simulation | 0b → 2 | — | C V1 |
| POW-02 | Hachage de l'en-tête de bloc | À COMPARER | | Vecteurs de test | 1 | POW-01, TX-03 | C V1 |
| POW-03 | Structure de l'en-tête (champs, taille, version, nonce) | À COMPARER | | Spécification + tests | 1 | POW-01 | C V1 |
| POW-04 | Temps cible de bloc | À SIMULER | 600 s | Simulation de stabilité, de forks et d'expérience utilisateur | 3 | POW-01 | C V1 |
| POW-05 | Algorithme de difficulté | À COMPARER | LWMA / ASERT / équivalent. Voir F-03 | Simulations adversariales (cahier §6.3) | 2 | POW-04 | C V1 |
| POW-06 | Paramètres de difficulté (fenêtre ou demi-vie, bornes, démarrage) | À SIMULER | L'ancien « facteur 2 / 60 blocs » n'est pas conservé automatiquement | Idem | 2 | POW-05 | C V1 |
| POW-07 | Règles temporelles (médiane, dérive maximale dans le futur, horloge locale) | À COMPARER | Cahier §6.4 | Tests d'horodatages manipulés | 1–2 | — | C V1 |
| POW-08 | Sélection de chaîne : travail cumulé, calcul du travail par bloc, égalités | PROPOSÉ | Principe du cahier | Tests de forks et de réorganisations | 2 | POW-05 | C V1 |
| POW-09 | Maturité des récompenses de bloc | À COMPARER | | Test de réorganisation avec dépense | 1 | — | C V1 |
| POW-10 | Versions de protocole et règles d'activation | À COMPARER | | Essai sur testnet | 2 | GOV-05 | C V1 |
| POW-11 | Checkpoints ou exceptions de consensus | PROPOSÉ | Aucun sans étude préalable (cahier §6.5) | — | — | — | C V1 |
| POW-12 | Mesures pour un réseau naissant (hashrate initial faible) | À SIMULER | Attaques à 10, 25, 40, 50 % et au-delà | Rapport de simulation | 3 | POW-01 | C V1 |
| POW-13 | Démarrage du réseau : qui mine au début, nœuds de départ | À COMPARER | | Plan documenté | 5 | POW-12 | C P V1 |

### MON — Monnaie et émission

| ID | Décision | Statut | Point de départ / options | Condition de validation | Phase | Dépend de | Tags |
|---|---|---|---|---|---|---|---|
| MON-01 | Plafond monétaire | À SIMULER | 105 000 000 AUR (hypothèse) | Calcul par le code + vérification par un outil indépendant | 3 | MON-03, TX-06 | C V1 |
| MON-02 | Récompense initiale | À SIMULER | 50 AUR (hypothèse) | Scénarios du cahier §7.3 | 3 | — | C V1 |
| MON-03 | Calendrier de réduction | À COMPARER | Moitié tous les 1 050 000 blocs comme exemple ; alternatives plus courtes ou courbes différentes. Voir F-05 | Simulateur | 3 | POW-04 | C V1 |
| MON-04 | Règles d'arrondi et dernière récompense | À COMPARER | Pas de récompense finale artificielle pour atteindre le chiffre affiché | Calcul indépendant | 3 | TX-06 | C V1 |
| MON-05 | Distribution initiale et bloc de genèse | À COMPARER | Voir F-06 | Distribution publiée et vérifiable | 0b | — | C V1 |
| MON-06 | Partage des frais (mineur / fonds éventuel) | À COMPARER | | Simulation | 3 | OPT-02 | C V1 |
| MON-07 | Simulateur : méthode, hypothèses publiées | PROPOSÉ | Reproductible, scénarios défavorables inclus | Hypothèses et commandes publiées | 0b | — | O V1 |

### FEE — Frais, poids, capacité

| ID | Décision | Statut | Point de départ / options | Condition de validation | Phase | Dépend de | Tags |
|---|---|---|---|---|---|---|---|
| FEE-01 | Poids d'une transaction (octets, coût de validation) | À COMPARER | Pas de rabais artificiel sur les signatures (cahier §8.2). Voir F-07 | Benchmarks | 0b → 3 | CRY-01 | C V1 |
| FEE-02 | Taille maximale d'un bloc | À TESTER | 1 Mio = point de comparaison seulement | Tests de charge et de propagation | 3 | FEE-01 | C V1 |
| FEE-03 | Taille maximale d'une transaction | À TESTER | Idem | Idem | 3 | FEE-01 | C V1 |
| FEE-04 | Modèle de frais | À COMPARER | | Simulation | 3 | FEE-01 | C P V1 |
| FEE-05 | Politique de mempool (taille, expiration, dépendances) | À COMPARER | Cahier §8.4 | Tests | 3 | FEE-04 | P V1 |
| FEE-06 | Remplacement d'une transaction en attente | À COMPARER | | Règles explicites + tests | 3 | FEE-05 | P V1 |
| FEE-07 | Frais minimum de relais, protection contre le spam | À COMPARER | | Tests adversariaux | 3 | FEE-04 | P V1 |

### CRY — Cryptographie

| ID | Décision | Statut | Point de départ / options | Condition de validation | Phase | Dépend de | Tags |
|---|---|---|---|---|---|---|---|
| CRY-01 | Famille de signature : post-quantique seul, hybride, ou migration planifiée | À COMPARER | Voir F-04 | Benchmarks + analyse | 0b | — | C S V1 |
| CRY-02 | Niveau ML-DSA (44, 65 ou 87) | À TESTER | ML-DSA-65 candidat | Benchmarks | 0b | CRY-01 | C S V1 |
| CRY-03 | SLH-DSA comme alternative de comparaison | À COMPARER | Remis dans la comparaison (absent du cahier v4.0) | Benchmarks | 0b | CRY-01 | S V1 |
| CRY-04 | Bibliothèque de signature | À COMPARER | Candidats d'après une analyse de 2025 : RustCrypto `ml-dsa`, `aws-lc-rs`, `libcrux`, `liboqs-rust`. État d'audit à re-vérifier à la date du choix | Fiche de comparaison (maintenance, audit, licence, Android) | 0b | CRY-01 | S V1 |
| CRY-05 | Règle de validation d'un mode hybride (les deux signatures obligatoires ?) | À COMPARER | Seulement si CRY-01 = hybride | Analyse de composition | 0b | CRY-01 | C S V1 |
| CRY-06 | Identifiants d'algorithme et versions de format de clé et de signature | PROPOSÉ | Cahier §9.5 | Tests de confusion d'algorithmes | 1 | CRY-01 | C S V1 |
| CRY-07 | Graine et dérivation de clés (pas de dérivation de type BIP32 par défaut avec ML-DSA) | À COMPARER | Voir F-08 | Prototype + restauration testée | 0b | CRY-01 | S U V1 |
| CRY-08 | Source d'aléa | PROPOSÉ | Générateur du système d'exploitation via bibliothèque établie | Revue + tests | 1 | — | S V1 |
| CRY-09 | Chiffrement du fichier portefeuille et dérivation depuis le mot de passe | À COMPARER | | Revue | 4 | — | S U V1 |
| CRY-10 | Format de sauvegarde versionné | À COMPARER | | Restauration sur environnement propre | 4 | CRY-07 | S U V1 |
| CRY-11 | Politique de migration cryptographique | PROPOSÉ | Cahier §24.1 | `MIGRATION_POLICY.md` | 0a | GOV-05 | C S V1 |
| CRY-12 | Portefeuilles matériels | À COMPARER | | Appareil réel testé | 7 | CRY-01 | S U + |

### NET — Réseau, stockage, nœuds

| ID | Décision | Statut | Point de départ / options | Condition de validation | Phase | Dépend de | Tags |
|---|---|---|---|---|---|---|---|
| NET-01 | Bibliothèque P2P ou protocole spécifique | À COMPARER | | Justification écrite | 2 | — | P V1 |
| NET-02 | Messages, versions de protocole, limites de taille | À COMPARER | | Fuzzing | 2 | NET-01 | P V1 |
| NET-03 | Découverte de pairs et nœuds de départ | À COMPARER | Attention au risque de centralisation | Analyse | 2 | NET-01 | P V1 |
| NET-04 | Quotas, bannissement, protection contre le déni de service | À COMPARER | | Tests adversariaux | 2 | NET-02 | P S V1 |
| NET-05 | Moteur de stockage | À COMPARER | | Tests de panne et de corruption | 1 | — | P V1 |
| NET-06 | Index de l'état (selon UTXO ou comptes) | À COMPARER | | Tests | 1 | TX-01, NET-05 | P V1 |
| NET-07 | Nœud élagué | À COMPARER | | Garanties documentées | + | NET-05 | P + |
| NET-08 | Matériel modeste : Raspberry Pi | À TESTER | Modèle précis à noter | Benchmark publié | 3 | TST-01 | P V1 |
| NET-09 | API RPC locale (écoute locale, authentification) | À COMPARER | | Tests d'accès non autorisé | 2 | — | S P V1 |

### WAL — Portefeuille, mobile, accès distant

| ID | Décision | Statut | Point de départ / options | Condition de validation | Phase | Dépend de | Tags |
|---|---|---|---|---|---|---|---|
| WAL-01 | Où vivent les clés (fichier local, appareil sécurisé) | À COMPARER | Les clés ne sont jamais envoyées au nœud | Revue | 4 | CRY-07 | S U V1 |
| WAL-02 | Mode léger : preuves et hypothèses de confiance | À COMPARER | Dépend de TX-04 | Limites de confiance documentées | 6 | TX-04 | S U + |
| WAL-03 | Plateformes | PROPOSÉ | Windows, Linux, Android ; iOS hors périmètre initial | Validé par le porteur | 0a | — | U V1 |
| WAL-04 | Partage du code Rust avec Android | À COMPARER | | Prototype qui compile et signe sur un téléphone | 6 | CRY-04 | U + |
| WAL-05 | API mobile : appairage, authentification, permissions séparées | À COMPARER | | Tests d'accès, révocation | 6 | NET-09 | S U + |
| WAL-06 | Accès distant | PROPOSÉ | Réseau local d'abord, VPN ensuite (cahier §12.4) | Analyse de configuration | 6 | WAL-05 | S + |
| WAL-07 | Informations affichées avant signature | PROPOSÉ | Destinataire, montant, frais, total, réseau | Tests d'interface | 4 | TX-08 | S U V1 |
| WAL-08 | Statuts de transaction affichés | PROPOSÉ | Préparée, signée, diffusée, en attente, incluse, confirmée | Tests | 4 | — | U V1 |
| WAL-09 | Niveaux de confirmation affichés | À SIMULER | 0, 1, 2–3, 6 (repères du cahier §23) | Probabilités de réorganisation simulées | 3 | POW-12 | U V1 |

### OPT — Fonctions facultatives

| ID | Décision | Statut | Point de départ / options | Condition de validation | Phase | Dépend de | Tags |
|---|---|---|---|---|---|---|---|
| OPT-01 | Coffre de récupération | À COMPARER | À comparer avec sauvegardes multiples et multisignature | Analyse cryptographique + modèle de menace | + | TX-01, OPT-04 | C S + |
| OPT-02 | Fonds de résilience | PROPOSÉ | Facultatif ; hors v1 par défaut | Utilité démontrée + gouvernance | + | OPT-04 | C + |
| OPT-03 | Délai de récupération | À SIMULER | 144 blocs ≈ 24 h à 600 s par bloc (144 × 600 s = 86 400 s) | Simulation | + | OPT-01 | C + |
| OPT-04 | Mécanisme de verrouillage des fonds (multisignature, délais) | À COMPARER | Aucun n'est spécifié ; nécessaire pour OPT-01 et OPT-02 | Spécification + coût en octets avec signatures post-quantiques | + | TX-01 | C + |
| OPT-05 | Confidentialité avancée | PROPOSÉ | Hors périmètre initial | Étude indépendante | + | — | C + |
| OPT-06 | Extension de navigateur | PROPOSÉ | Indépendante du consensus | — | + | — | O + |
| OPT-07 | Minage en pool | PROPOSÉ | | — | + | POW-01 | P + |

### ENV, TST — Environnements, tests, benchmarks

| ID | Décision | Statut | Point de départ / options | Condition de validation | Phase | Dépend de | Tags |
|---|---|---|---|---|---|---|---|
| ENV-01 | Paramètres séparés regtest, testnet, mainnet | PROPOSÉ | Clés et paramètres jamais partagés | Test de séparation | 1 | — | C S V1 |
| ENV-02 | Politique de mise à jour logicielle (annonce, signature des binaires) | À COMPARER | | Procédure écrite | 5 | — | S V1 |
| ENV-03 | Plan d'incident | PROPOSÉ | | Document | 8 | — | O V1 |
| ENV-04 | Audit indépendant : périmètre et intervenant | À COMPARER | | Rapport reçu | 7 | — | S V1 |
| TST-01 | Matériel de référence des benchmarks | À COMPARER | PC de bureau, PC modeste, Raspberry Pi, téléphone Android : modèles précis à noter | Liste publiée avec les résultats | 0b | — | O V1 |
| TST-02 | Méthode de benchmark reproductible | PROPOSÉ | Outil, nombre de répétitions, versions, commande | Résultats rejouables | 0b | — | O V1 |
| TST-03 | Outils de test | PROPOSÉ | Tests de propriétés, fuzzing, tests différentiels | Mis en place en CI | 1 | — | O V1 |
| TST-04 | Seconde implémentation ou outil indépendant pour l'émission | À COMPARER | Par exemple un script indépendant | Résultats identiques aux deux méthodes | 0b | MON-03 | O V1 |

---

## 3. Fiches des décisions bloquantes

Chaque fiche liste : ce que l'on sait (avec source), ce que l'on ne sait pas, ce qu'il faut mesurer, le critère de validation, les risques. La **proposition** est une suggestion non validée.

### F-01 — Modèle UTXO ou comptes (TX-01)

- **Options :** UTXO (sorties consommées et créées) ; comptes (soldes et compteur anti-rejeu). Voir cahier §5.1.
- **Ce qu'on ne sait pas :** lequel facilite le mieux le portefeuille mobile léger, les preuves, le coffre de récupération et les signatures lourdes.
- **À mesurer :** prototype jetable des deux (types minimaux, deux transactions, détection de double dépense, taille sérialisée avec signature ML-DSA-65).
- **Critère :** comparaison écrite sur sécurité, simplicité, taille, compatibilité avec OPT-04 et WAL-02.
- **Risques :** choisir pour la commodité de l'IA qui écrit le code ; prototype jeté mais décision mal documentée.
- **Proposition (non validée) :** partir sur UTXO pour le prototype, car la vérification des fonds dépensés est plus explicite, puis confirmer ou renverser avec la comparaison.

### F-02 — Algorithme de PoW (POW-01)

- **Ce qu'on sait :** SHA-256 est dominé par des ASIC sur Bitcoin (cahier §6.2). Une chaîne neuve a un hashrate initial faible, donc l'attaque à majorité est moins coûteuse (cahier §6.5).
- **Ce qu'on ne sait pas :** le coût réel d'une attaque pour AURORA ; si un PoW orienté mémoire améliore l'accessibilité sans autres coûts.
- **À mesurer :** rapport comparatif (coût d'attaque, centralisation, énergie, validation sur matériel modeste) + simulation d'attaque simple (10, 25, 40, 50 %).
- **Critère :** rapport relu ; décision écrite avec ses limites.
- **Risques :** annoncer une accessibilité ou une sécurité non démontrée.
- **Critères de décision avant tout testnet public (phase 2) :** (1) le rapport comparatif est écrit et relu ; (2) au moins SHA-256 et une option orientée mémoire ont été comparés sur le matériel de référence ; (3) la simulation d'attaque à 10, 25, 40 et 50 % de hashrate est publiée avec ses hypothèses, en indiquant qu'elle ne garantit pas la sécurité réelle ; (4) une estimation documentée du coût de location de puissance est jointe, avec ses sources ; (5) le porteur accepte par écrit les risques résiduels ; (6) un plan de démarrage du réseau (POW-13) existe. Sans ces six éléments, pas de testnet public.
- **Proposition (non validée) :** garder SHA-256 comme référence expérimentale de prototype, pour avancer, sans le valider pour le protocole.

### F-03 — Difficulté (POW-05, POW-06)

- **Ce qu'on sait :** l'ancienne hypothèse « ajustement à chaque bloc, borne ×2, fenêtre 60 blocs » est mal cohérente. LWMA est à fenêtre ; ASERT utilise un paramètre de demi-vie (à confirmer dans sa spécification).
- **À mesurer :** simulations du cahier §6.3 (hausse ou chute du hashrate, horodatages manipulés, périodes sans blocs, réorganisations).
- **Critère :** formule exacte, bornes, arrondis, comportement au démarrage documentés ; deux nœuds calculent la même difficulté.
- **Risques :** manipulation d'horodatages ; oscillations ; blocage après départ des mineurs.

### F-04 — Signatures (CRY-01 à CRY-05)

Tailles relevées dans des tableaux de bibliothèques et de documentation (liboqs, OpenSSL). **Sources secondaires : à confirmer dans les textes officiels FIPS 204 et FIPS 205.**

| Algorithme | Clé publique | Clé secrète | Signature |
|---|---|---|---|
| ML-DSA-44 | 1 312 o | 2 560 o | 2 420 o |
| ML-DSA-65 | 1 952 o | 4 032 o | 3 309 o |
| ML-DSA-87 | 2 592 o | 4 896 o | 4 627 o |

| SLH-DSA (signature) | 128s | 128f | 192s | 192f | 256s | 256f |
|---|---|---|---|---|---|---|
| Octets | 7 856 | 17 088 | 16 224 | 35 664 | 29 792 | 49 856 |

Clés publiques SLH-DSA : 32, 48 ou 64 octets selon le niveau.

- **Calcul indicatif (non mesuré) :** pour ML-DSA-65, clé publique + signature ≈ 5 261 octets par entrée. Avec un bloc de 1 Mio et 300 octets d'autres données par transaction : environ 188 transactions par bloc, soit environ 0,31 transaction par seconde à 600 s. Sans le surplus : 199 transactions. **À remplacer par des mesures.** Dépend du format d'adresse (si la clé publique est révélée à la dépense).
- **État des bibliothèques Rust (d'après une analyse de 2025, à re-vérifier) :** aucune bibliothèque post-quantique citée n'avait d'audit indépendant ; le README du crate `ml-dsa` de RustCrypto porte un avertissement de ce type. Un fichier de suivi d'écosystème indique une version 0.1.0 de `ml-dsa` en juin 2026 et `slh-dsa` encore en version candidate à cette date.
- **Hypothèses de sécurité (à vérifier dans les standards) :** ML-DSA repose sur des problèmes de réseaux euclidiens (modules) ; SLH-DSA est une signature à base de fonctions de hachage, sans état, aux signatures plus grandes et plus lentes selon la documentation OpenSSL. Les deux familles reposent donc sur des hypothèses différentes, ce qui justifie de garder SLH-DSA dans la comparaison même si sa taille est défavorable.
- **Matrice d'évaluation (on ne choisit pas sur la taille seule) :** hypothèses de sécurité ; taille des clés et signatures ; temps de génération, signature, vérification ; mémoire ; maturité de la bibliothèque Rust (version stable ou non, historique) ; maintenance (dernière publication, nombre de mainteneurs) ; audit disponible (indépendant, date, périmètre) ; interopérabilité (vecteurs officiels de test, accord avec une autre implémentation) ; portabilité Windows, Linux, Android, Raspberry Pi ; facilité de migration ; coût réseau et stockage dans un bloc. Chaque ligne est remplie par un fait sourcé ou par « inconnu ».
- **À mesurer :** génération de clés, signature, vérification, mémoire, taille, sur le PC disponible d'abord, puis Raspberry Pi et Android. Vérifier les vecteurs de test officiels. Vérifier qu'une signature altérée est rejetée.
- **Critère :** tableau de résultats avec matériel, versions et commande.
- **Risques :** bibliothèque non auditée ; benchmark non représentatif ; compilation Android difficile ; hybride mal défini (si deux signatures sont requises, préciser que les deux sont obligatoires).
- **Proposition (non validée) :** benchmarker ML-DSA-44/65/87 et SLH-DSA 128s/128f ; ne décider hybride ou post-quantique seul qu'après mesures.

### F-05 — Émission, plafond, unité (MON-01 à MON-04, TX-06)

Calcul indicatif avec l'hypothèse 50 AUR, moitié tous les 1 050 000 blocs, 8 décimales, entiers tronqués :

- 1 050 000 blocs × 600 s ≈ 19,96 ans par ère.
- Part émise après 1, 2, 3, 4 ères : 50 %, 75 %, 87,5 %, 93,75 %.
- 33 ères avant que la récompense tombe à zéro, soit environ 659 ans.
- Total émis : 104 999 999,8845 AUR. Il manque 0,1155 AUR par rapport à 105 000 000 à cause des troncatures.

Conséquences à trancher : le plafond « 105 M » est une valeur théorique ; le plafond réel dépend de l'unité minimale et de la règle d'arrondi. Les ères très tardives (au-delà de quelques décennies) n'ont aucun poids économique réel.

- **À mesurer :** simulateur avec scénarios du cahier §7.3 ; outil indépendant (TST-04) qui recalcule le total.
- **Critère :** total émis recalculé par deux méthodes, résultats identiques.
- **Risques :** présenter 105 M comme exact ; ajouter une récompense finale artificielle ; calendrier trop long pour la sécurité réelle.
- **Remarque :** 105 M est 5 fois le plafond de Bitcoin sans justification technique propre (cahier §7.1 le reconnaît).

### F-06 — Distribution initiale et genèse (MON-05)

- **Options :** (1) aucune préémission ; (2) allocation limitée et publique ; (3) allocation à un fonds transparent.
- **Critère :** distribution publiée, vérifiable dans le bloc de genèse.
- **Risques :** allocation cachée ; concentration ; confusion testnet / mainnet.
- **Proposition (non validée) :** aucune préémission pour regtest et testnet.

### F-07 — Poids et capacité (FEE-01 à FEE-03)

- **Ce qu'on ne sait pas :** la taille réelle d'une transaction, donc le débit.
- **À mesurer :** taille sérialisée de transactions types (1 entrée / 2 sorties, plusieurs entrées) avec chaque algorithme ; temps de vérification ; propagation.
- **Critère :** modèle de coût justifié par des mesures ; aucun rabais artificiel pour améliorer le nombre de transactions annoncé.
- **Risques :** capacité trop faible pour des frais faibles ; capacité trop élevée pour les nœuds modestes.

### F-08 — Dérivation de clés (CRY-07)

- **Ce qu'on sait (à confirmer) :** les schémas de dérivation de type BIP32 reposent sur des propriétés des courbes elliptiques que ML-DSA n'a pas à ma connaissance. Cela touche la rotation d'adresses, les portefeuilles en lecture seule et la sauvegarde.
- **À mesurer :** prototype de portefeuille avec graine + dérivation déterministe par hachage ; test de restauration complète.
- **Critère :** restauration identique sur environnement propre ; limites (lecture seule, rotation) documentées.
- **Risques :** promettre la lecture seule ou la rotation sans solution.

---

## 4. Chaîne de dépendances (ordre de décision)

1. **TX-01** modèle UTXO ou comptes
2. **TX-02, TX-03** sérialisation et hachage
3. **CRY-01, CRY-02, CRY-03** signatures (mesures)
4. **TX-05, TX-08, CRY-07** identifiant, adresse, dérivation
5. **FEE-01** poids, puis **FEE-02, FEE-03** tailles
6. **POW-01, POW-02, POW-03** PoW et en-tête
7. **POW-04, POW-05, POW-06, POW-07** temps de bloc, difficulté, règles de temps
8. **TX-06, MON-03, MON-04, MON-01** unité, calendrier, arrondis, plafond
9. **MON-05** genèse (peut être tranché tôt)
10. Le reste : réseau, portefeuille, mobile, options

---

## 5. Plan de la phase 0

Les durées ci-dessous sont des **estimations non mesurées**, pour un développeur seul assisté par IA. Elles servent à limiter le périmètre, pas à promettre un calendrier. Une phase s'arrête quand ses critères sont remplis ; si la durée est dépassée de moitié, on réduit le périmètre au lieu d'allonger.

### Phase 0a — Décisions et documentation (durée proposée : 1 semaine)

| Élément | Contenu |
|---|---|
| Fichiers | `docs/DECISION_REGISTER.md`, `docs/CONSENSUS.md`, `docs/THREAT_MODEL.md` (seulement ces trois ; les autres documents sont créés quand ils contiennent des informations vérifiées) |
| Tests | Aucun test de code. Contrôles : statuts autorisés, IDs de dépendance existants, aucune décision marquée validée sans preuve |
| Décisions à prendre par le porteur | GOV-02, GOV-04, GOV-05, GOV-06, WAL-03 |
| Critère de réussite | Les trois documents existent et sont cohérents ; les décisions de gouvernance sont prises |
| Risques | Sur-documentation ; ne jamais passer à la suite |

### Phase 0b — Prototypes jetables et benchmarks (durée proposée : 4 semaines, périmètre limité)

Périmètre : le **PC disponible d'abord**. Raspberry Pi et Android viennent après, seulement si le matériel est disponible, et ne bloquent pas la phase.

| Semaine | Travail | Fichiers | Critère de réussite |
|---|---|---|---|
| 1 | Installer Rust, exécuter pour la première fois `cargo fmt` puis `./scripts/check.sh` sur le prototype v0.0.1 et corriger ce qui échoue. Benchmark de signatures sur le PC : ML-DSA-44/65/87 et SLH-DSA 128s/128f, avec **une** bibliothèque d'abord | `tools/benchmarks/` | Tests du prototype réellement exécutés et résultats consignés ; tableau de mesures avec matériel, versions, commande |
| 2 | Prototype jetable UTXO et comptes (hors workspace, à supprimer ensuite) | `prototypes/` | Comparaison écrite (double dépense, taille, preuves, compatibilité avec OPT-04) ; TX-01 passé en VALIDÉ POUR LE PROTOTYPE par le porteur |
| 3 | Simulateur d'émission v0 avec vérification indépendante ; rapport de comparaison des PoW (documentaire) | `tools/simulator/`, `docs/CONSENSUS.md` | Les deux calculs d'émission donnent le même total ; rapport relu |
| 4 | Simulation d'attaque simple (10, 25, 40, 50 %) ; mise à jour du registre ; décision de passage en phase 1 | `docs/*` | Registre à jour ; décisions de la chaîne §4 (jusqu'à l'étape 4) au moins VALIDÉES POUR LE PROTOTYPE |
| Optionnel | Benchmarks sur Raspberry Pi et Android | `tools/benchmarks/` | Résultats publiés avec le matériel exact |

Risques de la phase 0b : bibliothèque post-quantique non auditée ; benchmark non représentatif ; compilation Android bloquante (ne pas bloquer la suite) ; prototype jetable conservé par inertie ; simulation prise pour une preuve de sécurité.

**Condition pour entrer en phase 1 :** le prototype jetable comparant les modèles de transaction est terminé et TX-01 est au moins VALIDÉ POUR LE PROTOTYPE. Le code de la v0.0.1 (annexe B) relève de la phase 0b : seuls `Amount`, le codec et le hachage sont indépendants du modèle ; la transaction et le bloc sont jetables.

---

## 6. Incohérences relevées dans le cahier des charges v4.0

1. Le tableau initial du cahier §4 utilise des statuts hors liste (« À mesurer », « À définir », « À étudier », « À concevoir », « Facultatif », « Hors périmètre initial »). Ce registre les remplace : À mesurer → À TESTER ; À définir, À étudier, À concevoir → À COMPARER ; Facultatif et Hors périmètre → PROPOSÉ avec tag `+`.
2. Le cahier §5.1 dit que UTXO ou comptes se décide « par prototype », mais le prototype est en phase 1. Ce registre ajoute un prototype jetable en phase 0b.
3. La phase 0 du cahier est trop chargée ; elle est découpée en 0a et 0b.
4. Le registre du cahier était incomplet : sérialisation, hachage, unité, genèse, adresse, maturité, bibliothèques et difficulté n'avaient pas de ligne. Corrigé ci-dessus.
5. SLH-DSA, présent dans la v2.0, avait disparu : remis en CRY-03.
6. Le plafond « 105 millions » est présenté comme cible mais n'a pas de justification technique propre ; voir F-05.
7. La consigne v4.1 impose que la phase 1 commence après le prototype jetable UTXO/comptes (§1.B), mais demande aussi de coder tout de suite types, sérialisation, transaction et bloc (§2). Résolution retenue : ce code est classé phase 0b ; seuls les montants, le codec et le hachage sont indépendants du modèle ; transaction et bloc sont jetables et marqués `experimental`.
8. Le compte « 74 lignes » annoncé pour la version 1.0 de ce registre était faux : le tableau en contient **88**.

---

## 7. Modèle de fiche pour une nouvelle décision

```
### F-XX — Titre (ID concerné)
- Contexte :
- Options :
- Ce qu'on sait (avec source) :
- Ce qu'on ne sait pas :
- À mesurer :
- Critère de validation :
- Risques :
- Proposition (non validée) :
```

---

## 8. Sources et état de vérification

- Tailles ML-DSA : tableaux de la bibliothèque liboqs (open-quantum-safe), concordants avec des implémentations Go et Zig. À confirmer dans FIPS 204 (<https://csrc.nist.gov/pubs/fips/204/final>).
- Tailles SLH-DSA : documentation OpenSSL. À confirmer dans FIPS 205 (<https://csrc.nist.gov/pubs/fips/205/final>).
- État des bibliothèques Rust : analyse publiée en 2025 (Project Eleven) et README du crate `ml-dsa` ; fichier de suivi d'écosystème d'un autre projet pour la version de juin 2026. **Re-vérifier à la date du choix.**
- Calculs d'émission et de débit : script Python local, arithmétique entière ; résultats **indicatifs**.
- Aucune compatibilité de portefeuille matériel n'a été vérifiée.

---

## 9. Journal des changements

| Date | ID | Changement | Par |
|---|---|---|---|
| 2026-10-25 | tous | Version 1.1 : annexe A (justification et preuves requises pour les 88 lignes), annexe B (hypothèses du code), phases 0a/0b avec durées estimées, critères de décision du PoW avant testnet public, matrice d'évaluation des signatures, correction du compte de lignes (88). Aucun statut modifié | Claude (proposition), à valider par le porteur |
| 2026-10-24 | tous | Création du registre v1.0 d'après le cahier v4.0 ; tous les statuts initiaux sont ceux du cahier ou PROPOSÉ / À COMPARER ; aucune décision validée | Claude (proposition), à valider par le porteur |

---

## 10. Annexe A — Justification du point de départ et preuves requises

Chaque ligne du tableau maître. « Justification » explique pourquoi on part de là ; elle n'est pas une validation. « Preuves requises » liste ce qui est nécessaire avant tout statut VALIDÉ.

| ID | Décision | Statut | Justification du point de départ | Preuves requises avant validation |
|---|---|---|---|---|
| GOV-01 | Langage Rust stable | PROPOSÉ | Langage retenu par le cahier ; typage strict et bons outils de test | Build reproductible (Cargo.lock, version figée) ; CI verte |
| GOV-02 | Licence du dépôt | À COMPARER | Une licence doit exister avant toute publication | Compatibilité des licences des dépendances (cargo deny) |
| GOV-03 | Politique de dépendances | PROPOSÉ | Réduit le risque de dépendance vulnérable ou abandonnée | cargo audit et cargo deny en CI ; justification écrite par dépendance |
| GOV-04 | Traitement du code généré par IA | PROPOSÉ | Le code est écrit avec une IA : non vérifié tant qu'il n'est pas testé et relu | Règle écrite ; chaque fusion distingue tests exécutés / non exécutés |
| GOV-05 | Procédure de modification du consensus | PROPOSÉ | Un changement de consensus sans procédure crée des forks accidentels | Procédure rejouée une fois sur regtest |
| GOV-06 | Périmètre de la v1 | PROPOSÉ | Un périmètre limité protège le noyau | Liste du hors-périmètre validée par le porteur |
| TX-01 | Modèle UTXO ou comptes | À COMPARER | Conditionne état, blocs, portefeuille léger et conditions de dépense | Prototype jetable des deux modèles + comparaison écrite |
| TX-02 | Format de sérialisation canonique | À COMPARER | Sans encodage unique, un même objet peut avoir plusieurs identifiants | Vecteurs indépendants ; aller-retour ; rejet des formes non canoniques ; tests de propriétés |
| TX-03 | Construction du hachage : SHA-256 simple, double ou autre ; séparation de domaines | À COMPARER | Le hachage fixe identifiants et PoW ; la séparation de domaines évite les confusions | Vecteurs publics SHA-256 ; vecteurs de domaine indépendants ; revue |
| TX-04 | Arbre de Merkle (nombre impair de feuilles, ambiguïtés de structure) | À COMPARER | L'engagement sur les transactions conditionne les preuves du mode léger | Tests d'ambiguïté (nombre impair, longueurs différentes) ; vecteurs |
| TX-05 | Identifiant de transaction (inclut-il les signatures ?) | À COMPARER | Si une signature modifiable entre dans l'identifiant, l'identifiant peut changer | Tests de malléabilité avec la signature retenue |
| TX-06 | Unité minimale et nombre de décimales | À COMPARER | Fixe la précision des paiements et la taille des entiers | Simulateur d'émission cohérent ; tests de bornes |
| TX-07 | Type entier des montants et opérations vérifiées | PROPOSÉ | Les flottants créent des erreurs d'arrondi ; un entier non signé borné suffit | Tests de dépassement par le haut et par le bas |
| TX-08 | Format d'adresse (préfixe réseau, contrôle d'intégrité, version, algorithme encodé) | À COMPARER | Sans contrôle d'intégrité, les fautes de frappe passent | Vecteurs ; faute de frappe rejetée ; confusion de réseau rejetée |
| TX-09 | Identifiants de réseau et domaine de signature par réseau | PROPOSÉ | Évite de prendre une donnée de test pour celle d'un autre réseau | Test de rejet croisé entre réseaux |
| TX-10 | Protection contre la relecture | À COMPARER | Sans protection, une transaction valide peut être rejouée | Tests de rejeu selon le modèle retenu |
| POW-01 | Algorithme de PoW | À COMPARER | Compromis sécurité / accessibilité reconnu par le cahier (§6.2) | Rapport comparatif + simulation d'attaque ; critères de F-02 |
| POW-02 | Hachage de l'en-tête de bloc | À COMPARER | Dépend du PoW et du hachage retenus | Vecteurs ; test de cible |
| POW-03 | Structure de l'en-tête (champs, taille, version, nonce) | À COMPARER | L'en-tête fixe ce que les mineurs hachent et ce que les nœuds vérifient | Spécification + vecteurs ; rejet des en-têtes mal formés |
| POW-04 | Temps cible de bloc | À SIMULER | 10 minutes est une valeur héritée ; compromis latence / forks | Simulation de stabilité, de forks, de propagation |
| POW-05 | Algorithme de difficulté | À COMPARER | Une difficulté mal conçue permet manipulation ou blocage | Simulations du cahier §6.3 ; même résultat sur deux implémentations |
| POW-06 | Paramètres de difficulté (fenêtre ou demi-vie, bornes, démarrage) | À SIMULER | Les paramètres déterminent réactivité et manipulabilité | Simulations de variations de hashrate et d'horodatages |
| POW-07 | Règles temporelles (médiane, dérive maximale dans le futur, horloge locale) | À COMPARER | Les horodatages sont manipulables : il faut des bornes | Tests d'horodatages futurs, anciens, manipulés |
| POW-08 | Sélection de chaîne : travail cumulé, calcul du travail par bloc, égalités | PROPOSÉ | La longueur seule ne suffit pas : on suit le travail cumulé | Tests de forks : chaîne plus courte avec plus de travail |
| POW-09 | Maturité des récompenses de bloc | À COMPARER | Une récompense dépensée trop tôt disparaît lors d'une réorganisation | Test de réorganisation avec dépense prématurée |
| POW-10 | Versions de protocole et règles d'activation | À COMPARER | Les évolutions doivent s'activer de façon coordonnée | Essai d'activation sur testnet |
| POW-11 | Checkpoints ou exceptions de consensus | PROPOSÉ | Un checkpoint centralisé est un pouvoir de décision | Aucun test ; réexamen seulement après étude |
| POW-12 | Mesures pour un réseau naissant (hashrate initial faible) | À SIMULER | Un réseau naissant est vulnérable à une majorité de hashrate | Simulations à 10, 25, 40, 50 % ; hypothèses publiées ; aucune garantie de sécurité réelle |
| POW-13 | Démarrage du réseau : qui mine au début, nœuds de départ | À COMPARER | Les premiers blocs concentrent la production | Plan de démarrage documenté |
| MON-01 | Plafond monétaire | À SIMULER | Hypothèse du cahier, sans valeur économique propre | Calcul par le code + outil indépendant ; total exact publié |
| MON-02 | Récompense initiale | À SIMULER | Hypothèse expérimentale | Comparaison de scénarios d'émission |
| MON-03 | Calendrier de réduction | À COMPARER | L'exemple de 1 050 000 blocs (≈ 20 ans) étale très fortement l'émission | Simulateur : émission par année, part distribuée |
| MON-04 | Règles d'arrondi et dernière récompense | À COMPARER | Les troncatures rendent le total légèrement inférieur au plafond | Deux calculs indépendants identiques |
| MON-05 | Distribution initiale et bloc de genèse | À COMPARER | La genèse fixe qui détient quoi dès le départ | Distribution publiée et vérifiable dans le bloc de genèse |
| MON-06 | Partage des frais (mineur / fonds éventuel) | À COMPARER | Dépend de l'existence d'un fonds | Simulation |
| MON-07 | Simulateur : méthode, hypothèses publiées | PROPOSÉ | Les décisions monétaires doivent reposer sur des scénarios reproductibles | Hypothèses et commandes publiées ; scénarios défavorables inclus |
| FEE-01 | Poids d'une transaction (octets, coût de validation) | À COMPARER | Les signatures post-quantiques sont lourdes : le coût doit refléter les ressources réelles | Benchmarks de taille et de vérification |
| FEE-02 | Taille maximale d'un bloc | À TESTER | 1 Mio est une valeur héritée servant de point de comparaison | Tests de charge et de propagation ; matériel modeste |
| FEE-03 | Taille maximale d'une transaction | À TESTER | Idem | Idem |
| FEE-04 | Modèle de frais | À COMPARER | Les frais doivent dépendre de la taille et de la demande | Simulation sous faible et forte demande |
| FEE-05 | Politique de mempool (taille, expiration, dépendances) | À COMPARER | Une mempool sans limite est un vecteur de déni de service | Tests de limites mémoire et d'expiration |
| FEE-06 | Remplacement d'une transaction en attente | À COMPARER | Le remplacement peut être exploité pour le spam | Tests des règles de remplacement |
| FEE-07 | Frais minimum de relais, protection contre le spam | À COMPARER | Protège les nœuds contre les transactions abusives | Tests adversariaux |
| CRY-01 | Famille de signature : post-quantique seul, hybride, ou migration planifiée | À COMPARER | Compromis taille / sécurité / maturité à mesurer avant de figer le format | Benchmarks + analyse écrite |
| CRY-02 | Niveau ML-DSA (44, 65 ou 87) | À TESTER | ML-DSA-65 est le candidat du cahier, pas un choix prouvé | Benchmarks des trois niveaux ; vecteurs officiels |
| CRY-03 | SLH-DSA comme alternative de comparaison | À COMPARER | Hypothèses de sécurité différentes de ML-DSA | Benchmarks et tailles mesurées |
| CRY-04 | Bibliothèque de signature | À COMPARER | Aucune primitive maison ; l'état d'audit des bibliothèques est inégal | Fiche de comparaison ; vecteurs officiels ; avis d'audit sourcé |
| CRY-05 | Règle de validation d'un mode hybride (les deux signatures obligatoires ?) | À COMPARER | Un hybride n'est pas automatiquement plus sûr | Analyse de la règle de composition ; tests de confusion |
| CRY-06 | Identifiants d'algorithme et versions de format de clé et de signature | PROPOSÉ | Évite les signatures ambiguës et les retours silencieux à un algorithme plus faible | Tests de confusion d'algorithmes |
| CRY-07 | Graine et dérivation de clés (pas de dérivation de type BIP32 par défaut avec ML-DSA) | À COMPARER | Les dérivations conçues pour les courbes elliptiques ne sont pas supposées compatibles | Prototype + restauration testée |
| CRY-08 | Source d'aléa | PROPOSÉ | Une mauvaise source d'aléa ruine les clés | Revue ; usage exclusif de bibliothèques établies |
| CRY-09 | Chiffrement du fichier portefeuille et dérivation depuis le mot de passe | À COMPARER | Protéger le fichier portefeuille au repos | Revue ; tests de mauvais mot de passe |
| CRY-10 | Format de sauvegarde versionné | À COMPARER | Une sauvegarde non restaurable ne sert à rien | Restauration sur environnement propre |
| CRY-11 | Politique de migration cryptographique | PROPOSÉ | Les migrations mal gérées créent des forks et des pertes | Procédure documentée ; essai sur testnet |
| CRY-12 | Portefeuilles matériels | À COMPARER | Ne pas annoncer une compatibilité non testée | Test sur appareil réel |
| NET-01 | Bibliothèque P2P ou protocole spécifique | À COMPARER | Un protocole maison n'est pas un objectif en soi | Justification écrite ; maintenance de la bibliothèque |
| NET-02 | Messages, versions de protocole, limites de taille | À COMPARER | Les messages mal formés sont une surface d'attaque | Fuzzing des décodeurs |
| NET-03 | Découverte de pairs et nœuds de départ | À COMPARER | Des nœuds de départ fixes créent une dépendance | Analyse de centralisation |
| NET-04 | Quotas, bannissement, protection contre le déni de service | À COMPARER | Pairs malveillants et saturation de ressources | Tests adversariaux |
| NET-05 | Moteur de stockage | À COMPARER | Cohérence après arrêt brutal | Tests de panne, de corruption, de disque plein |
| NET-06 | Index de l'état (selon UTXO ou comptes) | À COMPARER | Dépend du modèle de transaction | Tests de cohérence de l'état |
| NET-07 | Nœud élagué | À COMPARER | Réduit le stockage mais affaiblit certaines garanties | Documentation des garanties |
| NET-08 | Matériel modeste : Raspberry Pi | À TESTER | Objectif d'accessibilité du cahier | Benchmark publié avec modèle exact |
| NET-09 | API RPC locale (écoute locale, authentification) | À COMPARER | Une API d'administration exposée est une cible | Tests d'accès non autorisé |
| WAL-01 | Où vivent les clés (fichier local, appareil sécurisé) | À COMPARER | Les clés ne doivent jamais aller au nœud | Revue ; tests d'isolement |
| WAL-02 | Mode léger : preuves et hypothèses de confiance | À COMPARER | Un mode léger repose sur des hypothèses de confiance | Limites documentées ; preuves vérifiables |
| WAL-03 | Plateformes | PROPOSÉ | Cibles du cahier : Windows, Linux, Android | Validation par le porteur |
| WAL-04 | Partage du code Rust avec Android | À COMPARER | Réutiliser le code Rust évite deux implémentations de la signature | Prototype qui compile et signe sur téléphone |
| WAL-05 | API mobile : appairage, authentification, permissions séparées | À COMPARER | Séparer lecture, administration et portefeuille | Tests d'autorisation, de révocation |
| WAL-06 | Accès distant | PROPOSÉ | Ne jamais exposer l'API sans protection | Analyse de configuration |
| WAL-07 | Informations affichées avant signature | PROPOSÉ | Éviter les signatures à l'aveugle | Tests d'interface |
| WAL-08 | Statuts de transaction affichés | PROPOSÉ | Ne pas présenter une transaction comme confirmée si elle ne l'est pas | Tests de statuts |
| WAL-09 | Niveaux de confirmation affichés | À SIMULER | Les confirmations ne sont pas une garantie de finalité | Probabilités de réorganisation simulées |
| OPT-01 | Coffre de récupération | À COMPARER | Simples sauvegardes ou multisignature peuvent suffire | Analyse cryptographique + modèle de menace |
| OPT-02 | Fonds de résilience | PROPOSÉ | N'est retenu que si son utilité est démontrée | Gouvernance et clés définies |
| OPT-03 | Délai de récupération | À SIMULER | 144 × 600 s = 24 h : simple hypothèse de départ | Simulation |
| OPT-04 | Mécanisme de verrouillage des fonds (multisignature, délais) | À COMPARER | Aucun mécanisme de verrouillage n'est spécifié | Spécification + coût en octets avec signatures post-quantiques |
| OPT-05 | Confidentialité avancée | PROPOSÉ | Hors périmètre initial | Étude indépendante |
| OPT-06 | Extension de navigateur | PROPOSÉ | Indépendante du consensus | Tests de faux positifs et de permissions |
| OPT-07 | Minage en pool | PROPOSÉ | Hors périmètre initial | — |
| ENV-01 | Paramètres séparés regtest, testnet, mainnet | PROPOSÉ | Éviter de mélanger test et production | Test de séparation des clés et paramètres |
| ENV-02 | Politique de mise à jour logicielle (annonce, signature des binaires) | À COMPARER | Une fausse mise à jour est une menace du modèle | Procédure écrite |
| ENV-03 | Plan d'incident | PROPOSÉ | Préparer la réponse avant l'incident | Exercice sur testnet |
| ENV-04 | Audit indépendant : périmètre et intervenant | À COMPARER | Les composants critiques exigent un regard indépendant | Rapport reçu et corrections |
| TST-01 | Matériel de référence des benchmarks | À COMPARER | Des mesures sans matériel précisé ne sont pas comparables | Liste publiée avec les résultats |
| TST-02 | Méthode de benchmark reproductible | PROPOSÉ | Reproductibilité | Résultats rejouables par un tiers |
| TST-03 | Outils de test | PROPOSÉ | Les tests d'exemples ne suffisent pas | Outils présents en CI |
| TST-04 | Seconde implémentation ou outil indépendant pour l'émission | À COMPARER | Un seul calcul peut contenir une erreur | Résultats identiques par deux méthodes |

---

## 11. Annexe B — Hypothèses retenues dans le prototype v0.0.1

Ces choix existent **dans le code** pour pouvoir avancer ; ce ne sont pas des décisions validées. Chacun a des conséquences à connaître.

| ID | Hypothèse dans le code | Où | Conséquence si on la change |
|---|---|---|---|
| TX-06 | 8 décimales ; 1 AUR = 10^8 unités | `aurora-core/src/amount.rs` | Constantes et tous les vecteurs de test à recalculer |
| TX-07 | Montants en entier non signé 64 bits, addition vérifiée, plafond de garde 105 000 000 AUR | `amount.rs` | Le plafond du type n'est pas une règle d'émission (MON-01 ouvert) |
| TX-02 | Entiers big-endian à largeur fixe, longueurs sur 4 octets bornées, rejet des octets en trop | `aurora-core/src/codec.rs` | Changer de format change tous les identifiants et vecteurs |
| TX-03 | SHA-256 simple (bibliothèque `sha2`) de `longueur(tag) ‖ tag ‖ données`, domaines `TxId`, `BlockHeader`, `TxList` | `aurora-core/src/hash.rs` | Double SHA-256 ou autre : identifiants et vecteurs à refaire |
| TX-05 | Identifiant de transaction = hachage de l'encodage complet, `auth_data` inclus | `aurora-core/src/experimental.rs` | Si les signatures sortent de l'identifiant, le format change |
| TX-04 | Engagement du bloc = hachage de `nombre ‖ identifiants`, pas un arbre de Merkle | `aurora-consensus/src/experimental.rs` | Le mode léger ne pourrait pas prouver l'appartenance d'une transaction |
| TX-09 | Réseaux `Regtest=1`, `Testnet=2`, `Mainnet=3` (réservé), présents dans transaction et bloc | `aurora-core/src/network.rs` | Octets et vecteurs à refaire |
| TX-01 | Transaction « entrées / sorties » **par commodité de test** ; ne préjuge pas du modèle | `experimental.rs` | À supprimer ou refaire après le prototype comparatif |
| CRY-01 | Aucune signature : `auth_data` opaque, **non vérifié** | `experimental.rs` | Aucune autorisation de dépense n'est vérifiée aujourd'hui |
| FEE-02/03 | Bornes arbitraires : 256 entrées, 256 sorties, 8 192 octets d'autorisation, 10 000 transactions par bloc | `experimental.rs` (core et consensus) | Valeurs sans justification de performance |
| POW-03 | En-tête sans cible, sans preuve de travail ; nonce libre | `aurora-consensus` | Aucun consensus n'est implémenté |

### État des tests (honnêteté)

- **Écrits :** tests unitaires dans les modules, tests de vecteurs (`tests/vectors.rs`, valeurs issues du script Python indépendant `scripts/reference_vectors.py`), tests de « propriétés » (`tests/properties.rs`, générateur déterministe maison, sans `proptest`).
- **Exécutés :** **aucun test Rust.** Rust n'était pas installé dans l'environnement de rédaction et il n'y avait pas d'accès réseau pour l'installer.
- **Vérifié autrement :** le script Python tourne ; les 19 valeurs attendues qu'il produit figurent bien dans les fichiers de test Rust.
- **Prévus :** `proptest`, `cargo-fuzz`, tests différentiels, `cargo audit`, `cargo deny`.
- **Non vérifié :** compilation, `rustfmt`, `clippy`. Le code peut contenir des erreurs de compilation.

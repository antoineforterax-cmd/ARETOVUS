# Décisions du responsable — suivi des choix du 31 octobre 2026

> **Compte rendu des réponses explicites du responsable du projet**, recueillies le 31 octobre 2026. Ce document consigne la direction recherchée ; il n’autorise aucune modification du consensus, de l’émission, de la difficulté, de la gestion des clés ou de la trésorerie. Il ne valide pas le lancement d'un mainnet. Référence consultée : `main` au SHA `b76e0cff6d7e2ee6527cac2506ad005d7360ec4f` ; questionnaire source : [DECISION_OPTIONS.md](DECISION_OPTIONS.md).

## Réponses consignées

| ID | Réponse du responsable | Niveau d'engagement / interprétation pratique | Suite à mener |
| --- | --- | --- | --- |
| **V01** | **B et C** | Ambition de livrer **un explorateur lecture seule, un wallet graphique puis le mobile**. Retenir B comme jalon intermédiaire et C comme direction d'écosystème, **sans promettre que le mobile sera livré dans la première release testnet v1**. | Définir des jalons séparés, avec critères de sécurité/UX et ressources. |
| **E01** | **A : 500 millions AUR** | Orientation économique retenue ; correspond au plafond brut codé **pour le testnet actuel**. Son adoption définitive pour un mainnet reste conditionnée à une validation de protocole. | Simuler calendrier, sécurité économique, distribution et durée de l'émission ; ne pas réécrire la règle du testnet. |
| **E04** | **A : 5 %** | Orientation retenue : 5 % de **l'émission brute** vers la trésorerie, 95 % vers le mineur (pas 5 % des frais). | Concevoir la gouvernance et l'utilisation effective des fonds avant tout mainnet. |
| **E05** | **A : 50 % de burn des frais** | Orientation retenue : moitié des **frais** brûlée, autre moitié au mineur ; aucun burn des récompenses de bloc au titre de cette règle. | Simuler les incitations de sécurité à long terme après épuisement du plafond d'émission. |
| **C01** | **A : SHA-256** | Retenir SHA-256 comme PoW de référence pour le projet. Cela **ne démontre pas** une résistance aux ASIC ou à la centralisation. | Étudier coûts d'attaque et distribution du minage ; pas de migration de PoW immédiate. |
| **C02** | **entre 60 et 120 secondes** | **Fourchette à comparer, valeur finale non choisie.** Le testnet reste à sa cible de **60 s** jusqu'à décision formelle. | Simulations 60, 90, 120 s avec propagation, réorganisations, confirmation et coût réseau. |
| **C03** | **Comparer** | **Aucune option validée** parmi le retarget actuel, ASERT et LWMA. | Protocole de simulation commun avec hashrate instable, dérive d'horloge, mineurs intermittents et tests adversariaux. |
| **K04** | **Étudier B et C** | **Aucune règle de confidentialité choisie** : comparer confidentialité privée par défaut et mécanismes facultatifs. | Modèle de menace, comparaison cryptographique, implications du modèle de comptes, performance, compatibilité post-quantique. |
| **K06** | **B : Tor/I2P facultatifs** | Orientation de recherche retenue pour **transports optionnels** ; ne pas affirmer qu'ils masquent les montants ou garantissent l'anonymat. | Prototype isolé, mesures de fuites IP/DNS, corrélations et résistance DoS. |
| **W06** | **A puis B** | V1 : **sauvegarde/restauration chiffrées**. Plus tard : **récupération volontaire via gardiens**, sans admin ni confiscation automatique. | Tests de restauration hors hôte, modèle de menace des gardiens et des délais, versionnement compatible. |
| **G01** | **A maintenant ; ensuite décisions et vote de la communauté** | **Aucune dépense de trésorerie dans le profil actuel**. Pour le futur : une équipe **élue d'au moins trois personnes**, compétentes en technique/finance, **conseille sans pouvoir décider seule** ; la communauté conserve la décision. Projet de vote conditionné à la détention d'un minimum d'AUR, seuil amené à décroître avec l'émission. **Aucun barème, quorum, pouvoir de vote ou mécanisme d'exécution n'est encore fixé.** | Étudier risques Sybil/ploutocratie, seuil, élection/révocation des conseillers, exécution des décisions et gouvernance de secours. |
| **G06** | **A : audit indépendant** | Règle de lancement : **pas de fonds réels avant revue humaine extérieure qualifiée**, correction des défauts et validation des preuves. | Définir champ, commit d'audit, critères de sortie et procédure de réévaluation. |

## Deuxième série — réponses du responsable du 31 octobre 2026

Ces huit réponses complètent la première série. **Les orientations sont conservées telles qu'exprimées ; « à comparer » et « je ne sais pas » restent des décisions ouvertes.** Aucun choix de protocole ni nouvelle fonctionnalité n'est activé par cette fiche.

| ID | Réponse du responsable | Interprétation et statut | Suites / précautions |
| --- | --- | --- | --- |
| **E02** | **A : aucun préminage** (confirmé deux fois) | **Orientation retenue.** Aucun AUR attribué avant le premier bloc ; correspond déjà à la genèse vide du testnet. **Attention :** « aucun préminage » ne signifie pas « 100 % aux mineurs », puisque l'émission actuelle partage 95 % mineur / 5 % trésorerie. | Documenter le lancement équitable, l'accès initial au minage et l'absence d'allocation cachée ; aucun changement à la genèse testnet actuelle. |
| **E03** | **Réduire progressivement la récompense ; halving peut-être préférable, incertitude exprimée** | **À comparer — aucune courbe nouvelle validée.** La formule actuelle réduit continûment la récompense selon le reliquat, sans halving. | Simuler (A) décroissance actuelle, (B) halvings à intervalles définis, (C) courbe alternative ; comparer calendrier sur 10 M / 100 M / 500 M blocs, plancher, plafond de 500 M AUR, revenus de minage, budget de sécurité et changements brusques de revenus. Ne pas coder de halving silencieusement. |
| **E06** | **Frais adaptés aux coûts réels** | **Orientation retenue ; méthode ouverte.** Privilégier des frais proportionnés aux ressources et à la pression du réseau, sans annoncer de barème figé. | Distinguer politique locale de relais anti-spam/mempool, coût de vérification et règle de frais ou base-fee consensuelle (modification de protocole). Tester spam, mineurs, petites transactions et coûts ML-DSA. |
| **N02** | **Découverte des pairs automatique et décentralisée** | **Objectif d'architecture retenu**, non livré ; le testnet utilise encore des pairs connus et de la découverte limitée. | Étudier amorçage initial de plusieurs sources, découverte distribuée, défense Sybil/éclipse, confidentialité IP/DNS, NAT et reprise sans serveur central unique. |
| **N08** | **Raspberry Pi par utilisateur** | **Objectif de nœud abordable : chaque participant qui le souhaite doit pouvoir faire fonctionner son propre nœud sur un Raspberry Pi suffisamment capable** ; **ce n'est pas une obligation matérielle faite à chaque utilisateur**. | Fixer modèles de Pi, RAM, support SSD, coût/énergie ; mesurer synchronisation réelle, stockage, CPU, vérification ML-DSA, réseaux hostiles et reorgs avant garantie de compatibilité. |
| **W02** | **Incertain ; service de garde facultatif envisagé** | **NON TRANCHÉ.** Recommandation de prudence : conservation des clés par l'utilisateur **par défaut** ; un service de garde, s'il existe, doit rester **facultatif et hors noyau de confiance consensuel**. Aucun service n'est approuvé. | Comparer non-custodial (défaut), garde facultative tierce et récupération assistée sans dépossession des clés. Modèle de menace, obligation légale, confidentialité, exposition des fonds et interdiction d'une sauvegarde forcée sur serveur. |
| **G02** | **Règles communautaires, conseil élu et votes** | **Orientation retenue** selon [G01 plus haut](#complément-g01--proposition-de-gouvernance-exprimée-le-31-octobre-2026) : conseil élu de 3 personnes minimum, expertise technique/finance, consultatif uniquement ; la communauté décide. | Finaliser éligibilité, méthode anti-Sybil, durée des mandats, élection/révocation, transparence, quorum et exécution cryptographique des décisions. |
| **G07** | **Oui aux binaires signés** | **Objectif de distribution retenu : téléchargements authentifiables, accompagnés de sommes de contrôle et de provenance vérifiable**. Cela ne constitue ni un audit ni une preuve de sécurité du programme. | Choisir identité et rotation des clés de signature, attestation CI, archives reproductibles, documentation de vérification Windows/Linux/macOS, publication des empreintes et gestion des clés compromises. |

**Décisions de direction prises dans cette série :** E02, E06, N02, N08, G02 et G07. **À comparer :** E03. **Indécis / alternatives ouvertes :** W02. Ces réponses ne tranchent pas les sous-paramètres techniques ; une règle de consensus nécessite tests, revue et approbation spécifique avant intégration.

### Attention : halving ≠ difficulté de minage

Le **halving** agit sur la *quantité d'AUR créée par bloc* à des hauteurs déterminées. La **difficulté PoW** ajuste l'effort pour trouver un bloc et peut augmenter ou diminuer. Aucun de ces mécanismes ne se substitue à l'autre. L'émission actuelle d'AURORA dépend du reliquat d'émission, non du hashrate : le changement éventuel serait une **rupture de consensus** exigeant nouveau profil/réseau, vecteurs indépendants et analyse de migration.


## Orientation supplémentaire du responsable — interface de minage (31 octobre 2026)

**Demande explicite :** à terme, **le minage doit pouvoir être lancé, arrêté et surveillé depuis une interface simple**, sans imposer des commandes de terminal à l'utilisateur. Cette orientation UX complète V01 (explorateur, wallet graphique et mobile) ; **elle ne représente pas une 69e décision de consensus**.

**État vérifié sur `main` au SHA `b76e0cff6d7e2ee6527cac2506ad005d7360ec4f` :** `crates/aurora-node/src/main.rs` propose `aurora-node testnet mine --miner ... --data-dir ... --count ... [--peer ...]` : un minage borné par nombre de blocs, **pas encore un service continu ni un tableau de bord**. La [démonstration actuelle](../guide-fr/04-reseau-et-minage.md) utilise des commandes PowerShell et des répertoires distincts pour nœud/mineur.

### Fonctionnalités à concevoir, non livrées

1. **Application locale sur ordinateur** : sélectionner le réseau (testnet clairement identifiable), le nœud, l'adresse publique recevant la récompense ; démarrer/arrêter les opérations ; afficher leur état réel et les erreurs, sans promettre des revenus.
2. **Panneau de contrôle** : hauteur/tip, synchronisation, pairs, tentatives et blocs minés, récompenses en unités AUR, difficulté et débit de hachage **uniquement après instrumentation**, CPU/RAM et éventuellement température/consommation **si mesurables**.
3. **Budget de ressources** : préférences CPU/charge, comportement sur batterie/surchauffe, arrêt explicite et confirmé ; aucune activité clandestine ni démarrage automatique sans consentement.
4. **Sécurité** : ne jamais exposer la clé privée au module de minage ; adresse de récompense seulement ; exécution locale avec permissions minimales, commandes et arguments validés, séparation des dossiers de données et respect de `.aurora.lock` ; interface distante pour Raspberry Pi seulement avec authentification et périmètre réduit.
5. **Ordre réaliste** : fiabiliser le noyau et le nœud → API de contrôle local authentifiée et bornée si réellement nécessaire → démarrage/arrêt sûr et métriques → application graphique PC → surveillance d'un nœud Raspberry Pi et éventuellement mobile en lecture seule. Le CLI reste disponible pour tester et dépanner.
6. **Ne pas confondre** l'interface de minage avec une nouvelle méthode de minage, la preuve d'une rentabilité, ou la preuve que Raspberry Pi est matériellement compétitif face à d'autres mineurs SHA-256.

**Statut : orientation produit exprimée, différée ; architecture, OS supportés, objectifs de performance et sécurité de l'API encore à spécifier. Aucun code Rust, consensus, émission ni règle de bloc n'est modifié par cette orientation.**

## Distinctions importantes

**Choix d'orientation consignés (sans activation du consensus)** : V01, E01, E04, E05, C01, K06, W06, G06. **Orientation à deux phases, mécanique non définie** : G01. **Études explicitement demandées, sans choix final** : C02, C03 et K04.

AURORA reste une **blockchain expérimentale**, au statut de testnet, sans protection contre tous les incidents ni audit externe. Les règles **actives** du code sont décrites par [TESTNET_DECISIONS.md](../TESTNET_DECISIONS.md) et [CONSENSUS.md](../CONSENSUS.md). **Ce procès-verbal ne modifie aucun de ces fichiers ni aucun binaire.**

## Point de vigilance : trésorerie communautaire

Sur le testnet actuel, la trésorerie reçoit 5 % de l'émission à une destination fixe, mais **aucune méthode autorisée de dépense n'est spécifiée**. Une future majorité de votes sur un site ou un forum **ne peut pas, à elle seule, débloquer ces montants**. Il faudra concevoir une autorisation cryptographique et une règle compatible avec les droits existants, éviter les prises de contrôle par faux comptes ou concentration de jetons, et tester les conséquences des réorganisations. **Ne promettre aucune récupération de ces fonds** sur le testnet actuel.

Un vote communautaire peut désigner l'intention politique, mais l'exécution doit disposer de **règles vérifiables et auditables**. Les questions de modalités (« une personne/un vote », preuve d'identité, voix pondérée en AUR, quorum, délai, veto de sécurité, dépense multisignature, proposition et révocation des signataires) restent **ouvertes** ; elles ne sont pas implicitement approuvées.


## Complément G01 — proposition de gouvernance exprimée le 31 octobre 2026

**Proposition du responsable, non encore spécifiée ni implémentée :**

1. **La communauté reste décisionnaire** des dépenses de trésorerie. Un **conseil de compétence élu**, d'au moins **trois personnes**, apporte des évaluations techniques et financières, publie des avis argumentés et aide à guider le projet ; **il ne peut ni approuver une dépense ni transférer les fonds seul**.
2. **Droit de vote conditionnel :** le responsable envisage de réserver le vote formel aux portefeuilles détenant au moins un **solde minimal d'AUR** ; ce seuil diminuerait avec l'évolution de l'émission. Les portefeuilles sous le seuil pourraient continuer à **émettre des propositions / avis** — interprétation à faire confirmer, car le mot « émettre » n'a pas encore été précisé.
3. **Le seuil n'est pas une preuve d'unicité :** une personne peut contrôler et approvisionner plusieurs portefeuilles. Un système « un portefeuille éligible = une voix » se contourne par fractionnement du solde. Un vote pondéré par les AUR empêche l'amplification par fractionnement linéaire, mais donne davantage de pouvoir aux gros détenteurs. Le mode de calcul de la voix reste **à arbitrer** (délégation, contrôle des comptes, pondération, garanties de confidentialité).
4. **Ne pas indexer le droit de vote sur la difficulté brute :** le PoW réajuste la difficulté à la hausse **ou à la baisse**, selon la puissance de minage et le rythme des blocs. La récompense par bloc décroît selon la formule d'émission actuelle ; ce sont deux mécanismes distincts. Proposer, après analyse, un calendrier de seuils **prévisible, déterministe et borné**, fondé sur la hauteur ou l'émission brute cumulée, avec des captures de solde (`snapshots`) à une hauteur fixée pour chaque scrutin.
5. **Pas de gouvernance exécutable sur le compte fixe actuel :** le compte de trésorerie testnet n'a pas de signature autorisée ni de mécanisme de décaissement. Un vote sur un site ne peut pas le débloquer ; toute future dépense requiert une autorisation cryptographiquement vérifiable, des règles versionnées et une revue indépendante.

**Paramètres encore ouverts :** élection/mandat/révocation du conseil ; domaine exact de son avis ; montant minimal initial, plancher et méthode de réduction ; fondement du vote (portefeuille, AUR pondérés, preuve d'unicité) ; dépôt anti-spam éventuel ; quorum, approbation, délais, limites budgétaires ; conflits d'intérêts ; publication des comptes et mode d'exécution des paiements.

**Statut : orientation produit exprimée + hypothèses techniques à comparer ; pas de décision de consensus ni de changement de droits de propriété.**

## Proposition de mécanisme de vote G01-VOTE (étude non validée)

**Origine :** approfondissement demandé par le responsable le 31 octobre 2026 afin de résoudre le problème des votes multipliés par plusieurs portefeuilles. **Ce texte n'implémente aucune règle et n'autorise pas de dépense.**

### Solution candidate A — vote linéaire des AUR, sans identité externe

- À un bloc de référence `H` **finalisé pour le scrutin selon une politique de réorganisations explicite**, extraire la répartition des soldes en unités minimales. Exclure les montants brûlés, la trésorerie non dépensable, les adresses inadmissibles définies par la politique et toute délégation déjà comptée ailleurs ; définir le dénominateur du quorum à partir de cet état.
- Un participant est éligible si son solde en AUR à `H` satisfait un **minimum T(H)**. **Son poids est linéaire en AUR**, sans arrondi par portefeuille donnant davantage de voix : répartir 1 000 AUR sur dix comptes éligibles conserve 1 000 AUR de poids total, pas dix votes indépendants.
- Chaque vote signe `network_id || proposal_id || snapshot_height || choice || nonce` avec séparation de domaines. Vérifier la signature et l'identité de compte, puis **n'accepter qu'une contribution par compte et proposition** ; une modification du choix suit une règle explicite avant clôture. Ne jamais réutiliser la même unité de poids via délégation/copie/transfert.
- Si le wallet délègue son pouvoir, l'enregistrer **avant le snapshot** ; interdire les délégations cycliques, les cumuls de pouvoir et le double vote direct/délégué. Les conditions d'éligibilité et les fonds de référence sont **immuables pendant le scrutin**.
- Avantage : le fractionnement de portefeuilles n'augmente pas artificiellement le poids total. **Limite majeure : l'inégalité liée à la concentration d'AUR demeure** ; ne jamais annoncer « une personne = une voix ».
- Cette solution peut être simulée par un outil externe sur des données de test, sans modifier le consensus ; elle ne peut **pas** autoriser une dépense sur le compte fixe actuel.

### Solution candidate B — un humain, une voix, éligibilité par AUR

- Garder un minimum d'AUR pour voter, mais ajouter un **titre d'unicité par personne** robuste aux identités fabriquées, idéalement prouvable sans dévoiler l'identité, avec un identifiant à usage unique (`nullifier`) par scrutin.
- Les données d'identité et d'éligibilité doivent rester confidentielles ; les preuves d'unicité, la délivrance/révocation des titres, les comptes multiples, les utilisateurs exclus et la compromission des émetteurs doivent être spécifiés et audités.
- **Aucune preuve d'identité décentralisée fiable, privée et universelle n'est intégrée à AURORA**. Un simple « un portefeuille = un vote », des badges locaux, ou le seul montant minimal AUR ne résolvent pas les faux comptes. Tout registre humain ou fournisseur d'attestation impose de nouvelles frontières de confiance, voire des risques de surveillance.
- Statut : **R&D**, pas une promesse immédiate ; des attestations anonymes peuvent réduire des divulgations, mais ne garantissent pas à elles seules l'unicité réelle.

### Règles complémentaires à comparer, communes aux deux

1. **Conseil élu** : au moins trois membres compétents en technique et finances ; avis, analyse et transparence, mais **aucun pouvoir d'exécuter une dépense**. Mandat, révocation, conflits d'intérêts et éventuels remplaçants à fixer.
2. **Seuil d'éligibilité dégressif** : `T(H)` est un barème explicite fondé sur l'émission brute cumulée ou des jalons de hauteur, avec **plancher minimum**, publié avant les scrutins et immuable pendant chaque vote. **Ne pas utiliser la difficulté de minage** : elle augmente ou diminue selon le hashrate.
3. **Protection contre les prises de contrôle** : seuil de participation mesuré sur le pouvoir éligible, majorité qualifiée plus exigeante pour les gros budgets, durée de débat/vote suffisante, délais avant décaissement, budgets plafonnés et approbation de dépenses distincte des modifications de consensus. Les valeurs des seuils de vote, taux de quorum et durées sont **à simuler**, pas décidées.
4. **Paiement gouverné par une règle de validité** : le résultat du vote doit lier immuablement **destinataire, montant, référence de proposition et conditions**, et être vérifié lors de l'exécution. Un simple serveur web affichant « approuvé » n'a aucun droit de dépenser les fonds. L'autorisation cryptographique du compte actuel **n'existe pas**.
5. **Tests adversariaux obligatoires** : fractionnement en milliers de portefeuilles, transferts et prêts avant/pendant les snapshots, achats de voix, forte concentration du capital, votes contradictoires, délégation cyclique, manipulations de quorum, réorganisations, replay inter-réseau, absence de participation et capture du conseil.
6. **Références techniques de comparaison** : [OpenZeppelin Governor / snapshot et timelock](https://docs.openzeppelin.com/contracts/5.x/governance), [Polkadot OpenGov / tracks et validation](https://docs.polkadot.com/reference/governance/), [Gitcoin / résistance Sybil et identité](https://gitcoin.co/research/identity-infrastructure-binding-constraint). AURORA **n'utilise pas** ces contrats EVM : il faudrait concevoir ses propres règles et preuves.

**Position de travail non approuvée :** prototyper **A** uniquement pour étudier les scrutins sur un testnet sans fonds réels ; garder **B** comme recherche si « une personne = une voix » devient une exigence produit. Le responsable doit trancher la finalité du vote avant toute spécification normative.

## Suite pratique

1. Conserver les choix de périmètre et de méthode ; continuer le testnet actuel sans rupture.
2. Produire **trois études comparatives indépendantes** : (a) 60–120 s, (b) retarget actuel vs ASERT/LWMA, (c) confidentialité privée par défaut vs facultative.
3. Écrire une proposition séparée de **gouvernance de trésorerie communautaire**, avec mécanisme de vote et exécution sûrs, **sans activer le compte fixe existant**.
4. Mesurer et documenter les effets des choix économiques retenus, puis obtenir l'accord explicite sur toute éventuelle règle de protocole.
5. Attendre la validation des PR de documentation, sans modification de `main` ni fusion implicite.

**Validation documentaire :** ce relevé décrit les réponses de l'auteur et la manière de les traiter. **Validation technique/mainnet : non accordée**.
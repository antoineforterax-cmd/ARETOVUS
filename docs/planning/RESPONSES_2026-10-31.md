# Décisions du responsable — première série du 31 octobre 2026

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
| **G01** | **A maintenant ; ensuite décisions et vote de la communauté** | **Aucune dépense de trésorerie dans le profil actuel**. Pour le futur : projet de gouvernance communautaire, **sans règle de vote, corps électoral ni autorité de signature encore décidés**. | Spécifier propositions, quorum, droits de vote, risques Sybil/ploutocratie, exécution des décisions et gouvernance de secours. |
| **G06** | **A : audit indépendant** | Règle de lancement : **pas de fonds réels avant revue humaine extérieure qualifiée**, correction des défauts et validation des preuves. | Définir champ, commit d'audit, critères de sortie et procédure de réévaluation. |

## Distinctions importantes

**Choix d'orientation consignés (sans activation du consensus)** : V01, E01, E04, E05, C01, K06, W06, G06. **Orientation à deux phases, mécanique non définie** : G01. **Études explicitement demandées, sans choix final** : C02, C03 et K04.

AURORA reste une **blockchain expérimentale**, au statut de testnet, sans protection contre tous les incidents ni audit externe. Les règles **actives** du code sont décrites par [TESTNET_DECISIONS.md](../TESTNET_DECISIONS.md) et [CONSENSUS.md](../CONSENSUS.md). **Ce procès-verbal ne modifie aucun de ces fichiers ni aucun binaire.**

## Point de vigilance : trésorerie communautaire

Sur le testnet actuel, la trésorerie reçoit 5 % de l'émission à une destination fixe, mais **aucune méthode autorisée de dépense n'est spécifiée**. Une future majorité de votes sur un site ou un forum **ne peut pas, à elle seule, débloquer ces montants**. Il faudra concevoir une autorisation cryptographique et une règle compatible avec les droits existants, éviter les prises de contrôle par faux comptes ou concentration de jetons, et tester les conséquences des réorganisations. **Ne promettre aucune récupération de ces fonds** sur le testnet actuel.

Un vote communautaire peut désigner l'intention politique, mais l'exécution doit disposer de **règles vérifiables et auditables**. Les questions de modalités (« une personne/un vote », preuve d'identité, voix pondérée en AUR, quorum, délai, veto de sécurité, dépense multisignature, proposition et révocation des signataires) restent **ouvertes** ; elles ne sont pas implicitement approuvées.

## Suite pratique

1. Conserver les choix de périmètre et de méthode ; continuer le testnet actuel sans rupture.
2. Produire **trois études comparatives indépendantes** : (a) 60–120 s, (b) retarget actuel vs ASERT/LWMA, (c) confidentialité privée par défaut vs facultative.
3. Écrire une proposition séparée de **gouvernance de trésorerie communautaire**, avec mécanisme de vote et exécution sûrs, **sans activer le compte fixe existant**.
4. Mesurer et documenter les effets des choix économiques retenus, puis obtenir l'accord explicite sur toute éventuelle règle de protocole.
5. Attendre la validation des PR de documentation, sans modification de `main` ni fusion implicite.

**Validation documentaire :** ce relevé décrit les réponses de l'auteur et la manière de les traiter. **Validation technique/mainnet : non accordée**.
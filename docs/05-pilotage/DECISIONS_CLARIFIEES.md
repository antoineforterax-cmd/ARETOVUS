# ARETOVUS — Réconciliation des choix et contradictions documentaires

**Date de constat : 9 novembre 2026. Référence : `main@caa94228b2fb769e3a5ea479ea5fd07485744489`.** Il s'agit d'une **réconciliation de documentation**, pas d'un vote ni d'une modification de consensus. Un document ancien peut avoir été juste pour **son époque ou sa branche** et faux s'il est lu comme l'état actuel.

## Comment décider quelle information est fiable ?

1. Pour un **comportement réel**, inspecter le code, les vecteurs de test et les exécutions sur **le SHA exact**. La description textuelle ne remplace pas l'implémentation.
2. Pour les **règles voulues et les modifications futures**, obtenir une **décision explicite** du responsable du protocole, une spécification, une analyse d'incompatibilité et des tests. L'existence d'une règle codée pour testnet ne la valide **pas automatiquement pour mainnet**.
3. Les documents de **proposition** et les essais datés peuvent être conservés comme **archives**, sans devenir des exigences actuelles par inadvertance.
4. Pour les tâches V2 : [issue #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) ; pour les preuves de jalons antérieurs : [issue #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142). Toute case est à vérifier contre GitHub et le commit concerné.

## Contradictions repérées et interprétation documentée

| Sujet | Ancienne option / phrase qui peut tromper | État décrit pour le testnet natif | Action de documentation |
| --- | --- | --- | --- |
| **Offre** | Plafond de 21 ou ~105 millions dans certaines discussions/propositions initiales | **500 millions ARETOVUS** à huit décimales, plafond brut | Lire [émission](../EMISSION_MODEL.md) ; ne pas traiter l'ancienne proposition comme une règle active |
| **Calendrier d'émission** | Halving ou paliers à simuler | **Proportionnel au reliquat** : `min(reliquat, max(1, floor(reliquat / 1_000_000)))` unités minimales | Les paliers appartiennent à l'historique, pas au consensus testnet courant |
| **Cible et difficulté** | Cible **120 secondes** et algorithme **ASERT** proposés | Cible **60 secondes**, fenêtre **10 blocs**, retarget entier borné | L'ASERT demeure une option de recherche, non un changement approuvé |
| **Signatures** | Signature classique, PQ plus tard | **ML-DSA-65 v3** sur le testnet natif ; SHA-256 demeure pour le hash et le PoW | Ne pas prétendre que tout ARETOVUS est « post-quantique » |
| **Modèle d'état** | Comptes ou UTXO à comparer | **Comptes avec solde et nonce** | Toute migration UTXO ou changement de comptes serait un autre projet de protocole |
| **Taille des blocs** | Anciennes cibles exploratoires (par exemple 1 MiB) | [Consensus](../CONSENSUS.md) documente une borne de corps de bloc de **4 MiB moins la marge de transport** | Vérifier dans le code avant de publier un chiffre comme garantie |
| **Frais** | Burn et frais parfois décrits avec des arrondis ambigus | `miner_fees = floor(F/2)`, `burned = F - miner_fees` | **En cas de frais impairs, l'unité résiduelle est brûlée**, pas donnée au mineur |
| **Trésorerie** | Gouvernance communautaire parfois présentée comme acquise | Adresse fixe créditée de 5 % de l'émission, **sans logique de dépense ni gouvernance** | Ne pas annoncer une réserve accessible |
| **Récupération** | Coffre, gardiens, inactivité | **Conceptions**, pas récupération effective de fonds du testnet | Évaluer les droits de propriété et les attaques avant intégration |
| **Confidentialité** | Ambition « radicalement anonyme » | Transactions de comptes non annoncées comme anonymes ; Tor/I2P et ZK **non livrés** | Décrire comme recherche |
| **Minage** | Accessible aux particuliers / résistance ASIC | **SHA-256 PoW de testnet**, sans preuve de résistance ASIC | Ne promettre ni rentabilité ni décentralisation |
| **Desktop et explorateur** | Ancien README : « reporté », voire « hors périmètre » | **Prototypes de lecture seule existants**, mais pas wallet graphique complet ni minage graphique | Mettre la documentation générale au présent, tout en gardant les notes historiques datées |
| **Feuilles de route** | Plusieurs listes anciennes paraissant toutes « actives » | #178 pour développement V2 ; #142 pour historique de consolidation | Un point d'entrée clair ; conserver les sources et dates |
| **Audit et mainnet** | Un workflow vert serait une preuve suffisante | **Aucun audit humain indépendant confirmé ; NO-GO fonds réels** | Ne pas compter une contre-revue par IA comme revue externe |

Références : [protocole](../PROTOCOL.md), [consensus](../CONSENSUS.md), [décisions testnet](../TESTNET_DECISIONS.md), [programme d'émission](../EMISSION_MODEL.md), [architecture](../../ARCHITECTURE.md), [README desktop](../../apps/desktop/README.md), [planning](../planning/README.md).

## Décisions encore ouvertes — pas de résolution inventée

- **Mainnet :** décision d'aller plus loin, sécurité économique, frais durables, gouvernance, protections anti-concentration, paramètres de protocole et d'émission finale.
- **Confidentialité :** protection des adresses, montants et métadonnées réseau, coûts de preuve, compatibilité avec la transparence/audit.
- **Récupération de fonds :** modèle volontaire, abus possibles, collusion et révocation.
- **Trésorerie :** autorité des dépenses, accès aux clés, gouvernance et conflits d'intérêts.
- **Examen externe :** personnes compétentes, périmètre et preuves vérifiables avant fonds réels.
- **Licence/provenance :** attribution des contributions et dépendances à contrôler juridiquement avant une distribution financière élargie.

Ces points restent **À DÉCIDER**. La présente page ne les transforme pas en choix validés.

## Documents dont le statut est historique

- [DECISION_REGISTER.md](../DECISION_REGISTER.md) : registre de phase 0, nombreuses lignes non réconciliées.
- [ancienne baseline (historique Git)](https://github.com/antoineforterax-cmd/ARETOVUS/blob/caa94228b2fb769e3a5ea479ea5fd07485744489/docs/DECISION_BASELINE.md) : recommandations antérieures à l'implémentation.
- [ancienne feuille d'exécution (historique Git)](https://github.com/antoineforterax-cmd/ARETOVUS/blob/caa94228b2fb769e3a5ea479ea5fd07485744489/docs/ROADMAP_EXECUTION.md) et [FINALIZATION_CHECKLIST.md](../FINALIZATION_CHECKLIST.md) : états et bilans successifs d'octobre.
- [planning/DECISIONS.md](../planning/DECISIONS.md), [ancienne feuille de planification (historique Git)](https://github.com/antoineforterax-cmd/ARETOVUS/blob/caa94228b2fb769e3a5ea479ea5fd07485744489/docs/planning/ROADMAP.md), [planning/ARCHITECTURE_CIBLE.md](../planning/ARCHITECTURE_CIBLE.md) : propositions et constats datés.
- [guide-fr/README.md](../guide-fr/README.md) : guide reproductible d'une release historique, non documentation automatique de la dernière branche.

Voir [le catalogue des archives](../ARCHIVES_ET_RAPPORTS.md) pour les conserver sans surcharger la navigation. **Certains guides et plans dépassés ont été retirés de l'arborescence de travail, mais restent accessibles dans Git ; les rapports de test et registres de décisions ont été conservés.**

## Comment corriger une véritable contradiction de code ?

Ne pas sélectionner arbitrairement un chiffre dans ces pages. Ouvrir une issue avec le SHA, les fichiers, l'invariant concerné, le test qui démontre la divergence et ses conséquences. Un changement de consensus doit faire l'objet d'une décision **spécifique**, d'un plan de compatibilité et d'une revue avant fusion.
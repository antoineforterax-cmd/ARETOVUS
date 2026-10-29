# AURORA — Baseline de décisions de travail et plan d'exécution

**Statut général : propositions historiques de phase 0, conservées pour traçabilité.**

**Mise en contexte du 29 octobre 2026 :** les tableaux et étapes ci-dessous ne
décrivent pas tous les choix du testnet actuel. Les règles exécutées sont dans
[CONSENSUS.md](CONSENSUS.md), [TESTNET_DECISIONS.md](TESTNET_DECISIONS.md) et
[ARCHITECTURE.md](../ARCHITECTURE.md). Les travaux restants sont suivis dans
[FINALIZATION_CHECKLIST.md](FINALIZATION_CHECKLIST.md) et les issues ouvertes.

| Proposition historique ci-dessous | État du testnet constaté |
| --- | --- |
| Comptes proposés, prototype à tester | Comptes avec soldes et nonces implémentés |
| Émission par paliers à simuler | Émission proportionnelle au reliquat, plafond brut de 500 millions |
| Cible 120 secondes et ASERT à tester | Cible 60 secondes et retarget entier par fenêtres de 10 blocs |
| Signatures classiques à comparer, PQ différé | Transactions v3 natives ML-DSA-65 |
| Réseau, stockage et wallet à construire | TCP P2P v4, stockage v6 et wallet CLI disponibles |
| Licence à choisir | MIT OR Apache-2.0 dans le workspace |

Ces constats décrivent le code, pas une validation de sécurité ni une décision
mainnet. La date ci-dessus est celle de la réconciliation documentaire. Les
propositions non implémentées, notamment ASERT, ne deviennent pas des tâches
autorisées de modification du consensus du seul fait de leur présence ici.

**But historique :** éviter de rouvrir toutes les questions à chaque étape et
faire avancer le prototype dans un ordre sûr.

## Règle de décision

- **PROPOSÉ** : valeur par défaut recommandée pour continuer.
- **À TESTER / À SIMULER** : dépend de résultats reproductibles avant intégration.
- **BLOQUANT** : ne doit pas être figé dans le consensus tant que la règle n'est pas assez précise.
- Seul le porteur du projet peut confirmer une décision. Une CI verte ne constitue pas un audit de sécurité.

## 1. Base de travail recommandée

| Domaine | Proposition de travail | Statut | Motif / condition |
|---|---|---|---|
| Objectif v1 | chaîne expérimentale PoW, transferts simples, pas de contrats généralistes, pas de confidentialité avancée | PROPOSÉ | Réduire la surface d'attaque et livrer une testnet limitée |
| Langage | Rust stable, version épinglée | PROPOSÉ | Contrôle de types et outillage ; CI reproductible requise |
| État des transactions | Comptes avec solde + nonce | PROPOSÉ | Correspond au choix de travail du porteur ; prototype à tester |
| Montants | entiers signés? Non : entier non signé borné, unités minimales | À TESTER | Rejeter overflow/underflow ; nombre de décimales à confirmer |
| Unité | 1 AUR = 10^8 unités minimales | PROPOSÉ | Choix de travail pour simulation, pas encore validé |
| Plafond brut | 500 000 000 AUR créés au maximum | PROPOSÉ | Invariant de consensus strict ; burn ne réautorise pas une émission |
| Émission | courbe par paliers, bornée par l'offre restante et par le plafond | À SIMULER | Les seuils exacts viennent du simulateur ; aucun palier arbitraire ne doit être figé |
| Récompense de départ | comparer 50, 100 et 500 AUR ; préférence de travail actuelle : 500 AUR | À SIMULER | Évaluer durée de palier, distribution cumulée et revenu de sécurité |
| Trésorerie | 5 % de la récompense brute, comprise dans celle-ci ; mineur reçoit le reliquat | À SIMULER | Pas de création supplémentaire ; gouvernance et dépenses hors consensus à définir |
| Frais | partage 50 % mineur / 50 % burn | À SIMULER | Règle d'arrondi entière obligatoire ; comparer sécurité/incitation des mineurs |
| Préminage | aucun | PROPOSÉ | Simplicité et distribution vérifiable |
| Consensus | PoW SHA-256 | À TESTER | Définir exactement l'en-tête haché, la cible et le travail cumulé |
| Cible de bloc | 120 secondes | PROPOSÉ | Cible statistique, pas un délai garanti |
| Ajustement de difficulté | ASERT | À TESTER | Fixer ancre, paramètres, arithmétique entière, bornes et vecteurs de référence |
| Sélection de chaîne | chaîne valide avec travail cumulé le plus élevé | BLOQUANT | Définir validité, égalités, réorganisations et calcul du travail |
| Signatures | schéma classique issu d'une bibliothèque maintenue ; ne pas écrire de cryptographie maison | À COMPARER | Choisir courbe/algorithme, encodage, format d'adresse et gestion des clés ensemble |
| Post-quantique | hors consensus v1 immédiat, mais format versionné et plan de migration documenté | PROPOSÉ | Éviter de prétendre « quantum-proof » ; benchmark séparé avant choix |
| Encodage | format binaire canonique versionné | BLOQUANT | Une seule représentation valide, limites de taille, rejet des encodages non canoniques |
| Hachages | SHA-256 avec séparation de domaines par usage | À TESTER | Vecteurs de référence et tests anti-ambiguïté |
| Engagement de transactions | arbre de Merkle défini formellement | BLOQUANT | Cas nombre impair, ordre des feuilles, preuve et identifiants spécifiés |
| Réseau | pas de réseau P2P tant que la validation locale est correcte | PROPOSÉ | Construire d'abord le cœur déterministe |
| Stockage | chaîne/état persistants seulement après règles de réorganisation spécifiées | PROPOSÉ | Tester corruption, redémarrage, reconstruction et migrations |
| Wallet | clés restent côté portefeuille, jamais envoyées au nœud | PROPOSÉ | Afficher réseau, destinataire, montant, frais et total avant signature |
| Récupération | volontaire, explicitement configurée par le propriétaire ; jamais déclenchée par inactivité seule | PROPOSÉ | Nécessite un mécanisme de clés/guardians/verrouillage spécifié ; exclu du premier transfert minimal |
| Gouvernance de trésorerie | aucune autorité de dépense codée avant modèle de menace et règles d'autorisation | BLOQUANT | Risque de capture/vol ; multisignature et timelock à comparer |
| Confidentialité avancée | hors v1 | PROPOSÉ | Étude séparée ; comptes sont intrinsèquement faciles à relier |
| Extension navigateur | produit séparé, hors consensus | PROPOSÉ | Ne pas élargir le périmètre du nœud |
| Licence | MIT ou Apache-2.0, à choisir explicitement | À COMPARER | Vérifier compatibilité et dépendances |
| Dépendances | minimales, justifiées, auditées avec outils adaptés | PROPOSÉ | cargo audit/deny, verrouillage et revue des mises à jour |
| Tests | unitaires + vecteurs indépendants + tests génératifs/fuzzing | PROPOSÉ | Les tests seuls ne prouvent pas la sécurité |
| Déploiement | testnet isolée avant tout réseau avec valeur | BLOQUANT | Pas de mainnet ni de promesse de rentabilité avant revue indépendante |

## 2. Émission : recommandation opérationnelle

Ne pas utiliser une formule flottante. Toute récompense est calculée en unités minimales entières.

```text
remaining = CAP_UNITS - issued_units
scheduled = tier_reward(height, issued_units)
gross_reward = min(scheduled, remaining)
treasury = floor(gross_reward / 20)
miner_issuance = gross_reward - treasury
```

Le partage des frais est comptabilisé séparément de l'émission. Le burn réduit l'offre nette, jamais le plafond brut. Si `remaining == 0`, `gross_reward == 0`. Si une récompense tombe sous une unité minimale, la règle d'arrêt doit être explicite.

### Comparaison à simuler

- Départ : 50, 100 et 500 AUR.
- Paliers candidats : 500 → 100 → 60 → 50 → 30 → 10 → 1 AUR.
- Transition selon hauteur seule, offre créée seule, et règle hybride qui passe au palier suivant dès que le premier seuil (hauteur ou offre) est atteint.
- Scénarios de blocs à la cible, plus rapides et plus lents.
- Rapporter : émissions par année, cumul brut, solde restant, revenu mineur/trésorerie, nombre de blocs par palier, arrondis et fin d'émission.

Ne pas inventer les hauteurs de paliers avant que les simulations soient disponibles. Le seuil d'offre est le garde-fou le plus directement lié au plafond ; la hauteur limite le maintien d'un palier dans le temps.

## 3. Ordre d'implémentation recommandé

### Étape A — Stabiliser le noyau (ordre historique)
1. Inventaire des crates et API actuelles.
2. Types de montants entiers et opérations checked.
3. Encodage canonique + vecteurs indépendants.
4. Hashs avec séparation de domaines.
5. Tests de propriété sur bornes, décodage et identifiants.

### Étape B — Transactions par comptes
1. Adresse et algorithme de signature décidés ensemble.
2. Transaction canonique : chaîne/réseau, nonce, destinataire, montant, frais, signature.
3. Validateur d'état atomique : nonce, solde, frais, overflow, double dépense/rejeu.
4. Tests de transaction invalide, conflit, ordre et rejeu inter-réseau.

### Étape C — Économie et blocs
1. Simulateur d'émission indépendant et reproductible.
2. Règles exactes de récompense, frais, burn et trésorerie.
3. Validation des blocs et de l'offre.
4. Vecteurs de consensus et tests de divergence.

### Étape D — Consensus PoW
1. En-tête canonique et hash du travail.
2. Cible et vérification PoW.
3. ASERT avec paramètres et arithmétique spécifiés.
4. Travail cumulé, branches concurrentes et réorganisations.
5. Tests adversariaux et fuzzing.

### Étape E — Seulement ensuite
Stockage persistant, réseau P2P, portefeuille utilisable, testnet isolée, revue indépendante.

## 4. Conditions minimales pour dire qu'une étape est prête

- Les règles sont écrites sans ambiguïté.
- Les tests sont présents et réellement exécutés.
- Les vecteurs de référence sont reproduits par une méthode indépendante.
- Les erreurs et limites sont testées.
- La CI est verte sur le commit concerné.
- Les décisions de consensus sont consignées avec leurs statuts et la confirmation du porteur.
- Aucune revendication de sécurité n'est fondée uniquement sur la génération de code ou une CI verte.

## 5. Décisions explicitement non prises

- Les seuils exacts des paliers monétaires.
- Le type de signature classique et la bibliothèque finale.
- Le format canonique exact et le format des adresses.
- Les paramètres ASERT et la règle complète de chaîne.
- La gouvernance et les autorisations de dépenses de trésorerie.
- Le modèle exact de frais minimum et de relais.
- La structure de stockage, la P2P et la politique de réorganisation.
- La procédure de migration cryptographique et la récupération de portefeuille.

**Lecture actuelle :** cet ordre de travail est conservé comme historique. Le simulateur d'émission, le noyau par comptes et le réseau existent désormais ; consulter les règles exécutées et la liste des preuves restantes liées en tête avant de reprendre le développement.

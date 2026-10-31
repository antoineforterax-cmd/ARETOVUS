# AURORA — essai physique 4 h v2 avec télémétrie continue

**État : procédure PR en brouillon, non exécutée sur deux PC.** La CI teste des scénarios synthétiques et des composants isolés ; elle ne réalise pas quatre heures réelles. Aucun fonds réel, aucune ouverture Internet ni port-forwarding.

## Pourquoi une v2 ?

Le 31 octobre 2026, deux PC Windows ont convergé à 2 002 blocs (même sommet, 2 003 blocs connus et travail cumulé). PC A a réussi le palier de quatre heures ; PC B ne l'a pas réussi : une vérification a dépassé 300 secondes et une autre a mis 329 secondes. Après la campagne, PC B utilisait encore 90–93 % de RAM avec les applications ouvertes, puis 73 % après fermeture. **La charge mémoire au moment du timeout n'a pas été mesurée**, et le PC B a subi une chute avec des dégâts physiques et des comportements anormaux : ne pas rejouer la campagne sur ce portable sans contrôle matériel.

La PR #129 conserve ce premier constat ; cette v2 ajoute un échantillonnage **indépendant** pendant les appels bloquants à `testnet verify`.

## Prérequis avant le test

- **Deux PC en bon état**, distincts, réseau local privé connu, même commit et binaires vérifiés ; **pas le PC B endommagé** tant qu'il n'a pas été évalué et sauvegardé.
- Les quatre scripts `testnet_operational_probe.py`, `testnet_system_monitor.py`, `compare_testnet_evidence.py` et `validate_testnet_4h_v2.py` doivent venir du **même commit que ce guide**. Ne pas mélanger les scripts v1 et v2 ni supposer que les ZIP historiques les contiennent.
- Python 3.10+ ; sur Windows aucun droit administrateur requis pour la collecte. Pas de clé wallet dans les commandes de collecte.
- Les serveurs de testnet et le mineur utilisent **des dossiers distincts** et sont déjà connectés selon `docs/TWO_PC_TEST.md`. Le collecteur de preuves est en lecture seule, pas un serveur : il ne mine, ne connecte et n'arrête aucun nœud.
- Utiliser un **nom de dossier de preuves neuf** pour l'essai v2 ; les fichiers JSONL sont append-only et peuvent conserver d'anciennes exécutions.

## Démarrer la collecte (un terminal par PC)

Depuis la racine contenant `scripts` et `bin/aurora-node.exe`, récupérer le SHA complet du **commit exact ayant produit le binaire** depuis la provenance GitHub vérifiée. Ne pas recopier aveuglément un SHA partiel ou celui d'une autre version. Dans un checkout construit localement, `$sha = (git rev-parse HEAD).Trim()`.

**PC A (Windows PowerShell) :**

```powershell
$sha = (git rev-parse HEAD).Trim()
python .\scripts\testnet_operational_probe.py --binary .\bin\aurora-node.exe --data-dir .\two-pc\server-a --host-label pc-a --hours 4 --interval-seconds 300 --command-timeout 300 --telemetry-interval-seconds 5 --output .\evidence-pc-a-v2.jsonl --commit $sha
```

**PC B :**

```powershell
$sha = (git rev-parse HEAD).Trim()
python .\scripts\testnet_operational_probe.py --binary .\bin\aurora-node.exe --data-dir .\two-pc\server-b --host-label pc-b --hours 4 --interval-seconds 300 --command-timeout 300 --telemetry-interval-seconds 5 --output .\evidence-pc-b-v2.jsonl --commit $sha
```

**Important :** ces commandes ne sont correctes pour les archives *sans dépôt Git* que si la valeur `$sha` a été obtenue séparément depuis la provenance exacte du paquet ; sinon la commande Git échoue. Les dossiers `two-pc/server-*` sont les dossiers d'essai originaux ; les remplacer par les vrais dossiers sur chaque machine. Le volume de blocs doit progresser pendant l'essai pour passer le contrôle de vitalité. Il n'est pas nécessaire d'en produire 2 000 ni de charger du vrai argent.

Chaque exécution reçoit automatiquement un **identifiant aléatoire de campagne (run ID) de 128 bits**. Il apparaît dans les deux fichiers et dans leurs échantillons. Le validateur rejette un mélange de campagnes, un écart important entre les horodatages de départ/fin et les trous dans les relevés **même si les sommes SHA-256 des fichiers ont été recalculées**. La SHA-256 et cet identifiant empêchent surtout les confusions et modifications accidentelles : **ils ne constituent ni une signature cryptographique d'éditeur ni une attestation indépendante contre la falsification volontaire des deux fichiers.**

Chaque commande écrit **deux fichiers locaux** :
- `evidence-pc-a-v2.jsonl` (respectivement B) : événements `start`, `sample`, `finish` avec hauteur, sommet, latence `verify`, erreurs et SHA256 du journal ressources.
- `evidence-pc-a-v2.resources.jsonl` (respectivement B) : mesures toutes les ~5 secondes **pendant toute la campagne**, y compris pendant un `verify` bloqué : CPU système, RAM globale, RAM libre, espace disque libre. `disk_io_utilization_percent=null` : **l'activité réelle du disque n'est pas instrumentée**.

**Ces métriques sont celles de l'ordinateur entier, pas d'AURORA uniquement.** Si Windows manque de RAM, les observations aideront à identifier la corrélation ; elles ne prouvent pas seules la causalité. L'échantillonnage peut ralentir si Windows est lui-même saturé ; les trous temporels sont détectés par le validateur.

### Aucun contournement des erreurs

- Un `TimeoutExpired`, une erreur de vérification ou une erreur de télémétrie **fait échouer le palier 4 h**, même si les deux chaînes convergent ensuite.
- Ne pas augmenter automatiquement le timeout, altérer les JSONL ou ajouter un événement `finish` à la main.
- En cas d'arrêt brutal, l'absence du `finish` garde la campagne **non validée** ; conserver le journal incomplet pour analyse.
- Ne pas débrancher le PC, désactiver l'antivirus, désactiver le pare-feu ou réaliser de tests de panne sur des machines contenant des données importantes.

## Comparer les quatre fichiers

Après arrêt propre, transférer les deux fichiers de PC B sur PC A, **sans les écraser**, puis comparer :

```powershell
python .\scripts\validate_testnet_4h_v2.py --left .\evidence-pc-a-v2.jsonl --right .\evidence-pc-b-v2.jsonl --left-resources .\evidence-pc-a-v2.resources.jsonl --right-resources .\evidence-pc-b-v2.resources.jsonl --minimum-hours 4
```

Le contrôleur demande : deux `finish` valides, deux journaux ressources dont la somme SHA256 correspond aux résultats des collecteurs, des mesures cohérentes sans trous anormalement longs, zéro erreur et le même dernier `height/tip`. Une ligne de succès **ne prouve pas** à elle seule l'identité des machines physiques ou l'absence de failles.

Vérifier enfin à l'arrêt, **sur chaque machine**, puis conserver les résultats :

```powershell
.\bin\aurora-node.exe testnet verify --data-dir .\two-pc\server-a
```

Sur PC B remplacer `server-a` par `server-b`. Comparer `height`, `tip`, `work` et `known_blocks` ; exporter les sorties et archiver les caractéristiques matérielles ainsi que le journal Windows autour de tout incident.

## Ce qu'il restera après une v2 verte

Tests de partitions réseau et reconnexion réelle, transferts signés inter-PC, redémarrages, restauration de sauvegardes, campagnes de 72 h, réplication sur une chaîne de grande taille, audit externe. Le palier 4 h reste intermédiaire et ne constitue **jamais** une certification mainnet.

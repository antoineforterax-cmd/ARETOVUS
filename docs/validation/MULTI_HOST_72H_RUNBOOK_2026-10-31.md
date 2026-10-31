# AURORA — endurance physique sur deux PC : palier 4 h et option 72 h

**Statut : collecte et procédure prêtes à exécuter, campagnes de 4 h et de 72 h PAS exécutées par ces scripts.** Ceci n'est ni un audit ni une certification de sécurité. N'utiliser que des clés, comptes et fonds **Regtest/Testnet jetables**, aucun fonds réel. Ne jamais exposer le port P2P sur Internet pour ce test.

## Palier réaliste : 4 heures sur deux PC

**Objectif prioritaire : 4 h** sans pannes silencieuses et avec convergence des deux machines, pour une première validation d'endurance. Les deux machines ont déjà servi aux essais : il n'est pas nécessaire de refaire les vérifications élémentaires de connectivité avant chaque campagne si elles sont déjà documentées. **Le palier 4 h ne doit jamais être décrit comme une campagne 72 h**. Les essais de 72 h pourront être conduits plus tard sur du matériel dédié.

Le collecteur `scripts/testnet_operational_probe.py` accepte désormais `--hours 4` (durée par défaut) et inscrit `counts_as_4h` dans son dernier événement JSONL. Il continue à inscrire `counts_as_72h=false` pour cet essai. Il faut que la hauteur change au moins une fois pendant la campagne et que toutes les vérifications réussissent.

Utiliser le **même commit exact** sur les deux PC (à récupérer par `git rev-parse HEAD` sur la source utilisée pour produire le binaire) ; les données du nœud sont indépendantes. Ne jamais mettre un SHA inventé ni un SHA d'une autre version.

Sur **PC A, dans PowerShell**, tandis que son nœud tourne :

```powershell
$sha = (git rev-parse HEAD).Trim()
python scripts/testnet_operational_probe.py --binary .\target\debug\aurora-node.exe --data-dir .\pc-a-data --host-label pc-a --hours 4 --interval-seconds 300 --output .\evidence-pc-a.jsonl --commit $sha
```

Sur **PC B**, avec le même commit :

```powershell
$sha = (git rev-parse HEAD).Trim()
python scripts/testnet_operational_probe.py --binary .\target\debug\aurora-node.exe --data-dir .\pc-b-data --host-label pc-b --hours 4 --interval-seconds 300 --output .\evidence-pc-b.jsonl --commit $sha
```

Les deux commandes doivent être lancées à peu près au même moment. Il faut produire des blocs de test et observer leur propagation pendant les 4 h, puis laisser les chaînes converger avant les derniers prélèvements ; sinon la comparaison du sommet peut échouer. Les interruptions réseau ou redémarrages volontaires doivent être consignés manuellement avec horodatages ; un échec de vérification fait **échouer** le palier strict, et ne doit pas être effacé. Conserver les logs des serveurs séparément.

Rassembler les deux JSONL après les 4 h sur un PC, puis :

```powershell
python scripts/compare_testnet_evidence.py --left .\evidence-pc-a.jsonl --right .\evidence-pc-b.jsonl --minimum-hours 4
```

Le comparateur refuse les durées insuffisantes, sommets différents, erreurs de lecture, commits différents, labels identiques et fenêtres de temps disjointes. Il vérifie un chevauchement d'au moins 3 heures pour le palier de 4 heures. Il **ne peut pas prouver à lui seul** que les fichiers viennent de deux ordinateurs physiques ni garantir l'authenticité de leurs timestamps. Documenter séparément la machine et le réseau utilisés.

La campagne longue reste disponible en remplaçant `--hours 4` par `--hours 72`, et `--minimum-hours 4` par `--minimum-hours 72` ; il s'agit d'un **autre palier**, facultatif pour cette phase d'essai.

## Préparation — deux PC vraiment distincts

1. Réserver deux PC physiques (A et B), sous Windows ou Linux, sur un **réseau privé/VPN contrôlé**. Noter les IP privées A et B, versions de système, CPU, RAM, disque, fuseau, décalage d'horloge et SHA du commit AURORA testé. Tester la connectivité IP; vérifier les pare-feux. Un serveur et deux dossiers dans une seule VM **ne comptent pas**.
2. Installer exactement le même commit testé, vérifier `SHA256SUMS` et les versions, et démarrer avec des **répertoires de données indépendants et vides**. Le profil Testnet ML-DSA-65 v3 n'est pas interchangeable avec les anciens réseaux. Aucun déplacement de clés sensibles.
3. Depuis la racine du dépôt, lancer `cargo build --locked -p aurora-node`, puis utiliser le binaire produit. Pour deux machines sur LAN de confiance, remplacer les adresses privées ci-dessous par vos IP. Si le pare-feu exige une règle, la limiter à l'autre hôte et à la durée de l'essai.

### Sur PC A

Terminal 1, dossier `pc-a-data` réservé au serveur :

```powershell
.\target\debug\aurora-node.exe testnet serve --data-dir .\pc-a-data --listen 0.0.0.0:18444 --peer 192.168.1.11:18444
```

Terminal 2, **dossier différent** réservé au mineur, avec adresse de mineur jetable au format hexadécimal :

```powershell
.\target\debug\aurora-node.exe testnet mine --data-dir .\pc-a-miner --miner 4141414141414141414141414141414141414141414141414141414141414141 --count 1 --peer 192.168.1.10:18444
```

### Sur PC B

```powershell
.\target\debug\aurora-node.exe testnet serve --data-dir .\pc-b-data --listen 0.0.0.0:18444 --peer 192.168.1.10:18444
```

### Comparer automatiquement les deux journaux terminés

Après **arrêt du minage**, attendre la convergence et exporter les deux fichiers de preuves, sans clés. Cette section décrit le mode **72 h** ; pour 4 h, utiliser la commande du début du guide. Placer les deux JSONL sur une machine d'analyse et exécuter :

```powershell
python scripts/compare_testnet_evidence.py --left .\evidence-pc-a.jsonl --right .\evidence-pc-b.jsonl --minimum-hours 72
```

Ce contrôleur refuse une campagne interrompue, un SHA non enregistré, deux appareils portant le même label, une durée inférieure à 72 h, des erreurs de lecture et une discordance hauteur/hash. Il ne peut pas attester tout seul que deux machines physiques différentes existaient ni que les journaux n'ont pas été falsifiés. Conserver les journaux originaux des serveurs et les caractéristiques des hôtes.

**Remarque importante :** l'option `--peer` n'implique pas une découverte automatique de tout le réseau. Si la commande sur votre version ne peut pas se connecter, consigner le message d'erreur et vérifier la syntaxe depuis `--help`, sans modifier la sécurité de l'OS. Les répertoires de serveur et mineur ne doivent jamais être écrits par deux processus en même temps.

## Option supplémentaire — journal d'endurance longue durée (72 h)

Sur **chacune** des deux machines, pendant que son serveur tourne :

```powershell
$sha = (git rev-parse HEAD).Trim()
python scripts/testnet_operational_probe.py --binary .\target\debug\aurora-node.exe --data-dir .\pc-a-data --host-label pc-a --hours 72 --interval-seconds 300 --output .\evidence-pc-a.jsonl --commit $sha
```

Pour B, remplacer `pc-a-data` / `pc-a` / nom du fichier. Le programme échantillonne `testnet verify` sans modification de données, conserve UTC, hauteur, hash du sommet, espace disque, durée de chaque lecture et empreinte SHA-256 du binaire. Un journal `finish` est écrit uniquement à la fin. Une interruption prématurée ne produit **pas** de succès; `counts_as_72h` n'est vrai que si les 72 heures ont été parcourues, que le binaire est inchangé, qu'aucune lecture n'a échoué et que la hauteur a évolué.

Il faut **produire des blocs de test pendant la durée** ; sinon le test de vitalité échoue logiquement. Contrôler manuellement les journaux et la charge CPU/RAM, conserver les sorties des serveurs, l'espace disque, les transactions de test et toute sortie anormale. Le logger vérifie un nœud local; **il ne mesure pas à lui seul les échanges réseau ni les transactions**.

## Scénarios à réaliser et consigner

- Les deux PC atteignent la même **hauteur ET le même hash de sommet** après rattrapage ; vérifier avec `testnet verify --data-dir` sur chaque PC, puis comparer les deux sorties archivées.
- Déconnecter **seulement le réseau privé de B**, observer que A reste disponible, reconnecter B et exiger une convergence. Noter durée, hauteur, hashes avant/après et l'heure UTC.
- Redémarrer B sans toucher aux fichiers. Recharger les index v6 et comparer son sommet avant/après. Ne jamais effacer ni forcer `.aurora.lock`.
- Développer des branches concurrentes avec **des répertoires de minage distincts** et documenter le travail cumulé, la réorganisation, les transactions rejetées et l'état du mempool après reconnexion.
- Sur une **copie hors ligne**, interrompre une sauvegarde, simuler une panne de disque ou tronquer un fichier de test. Ne pas corrompre les dossiers de production de l'essai. Conserver l'ancien point de reprise et revérifier hauteur/hash après restauration.
- Tester une transaction **ML-DSA-65 de valeur nulle économiquement** entre deux wallets de test chiffrés ; comparer présence au mempool, confirmation et preuve SPV. Ne publier ni clés ni mots de passe.
- Hors période de test, effectuer une copie complète arrêtée vers un second PC : `snapshot-*.auridx`, `blocks-v5/` et `index-pages-v6/`. Exécuter `testnet verify` sur la copie.

## Critères de décision

Aucun « essai 72 h réussi » sans **72 heures réelles sur un SHA épinglé**, un journal final pour **chaque** hôte, traces de disponibilité, activité de blocs/transactions, absence de divergence persistante, tests de coupure/rattrapage, vérification finale des copies et tableau d'incidents. Le contrôle Linux GitHub de quelques minutes/heure ne remplace pas cet essai.

La collecte actuelle ne chiffre pas les logs : les conserver dans un dossier privé d'opérateur, vérifier qu'ils ne contiennent pas de données personnelles avant un partage dans GitHub. **Aucun appareil distant n'est accessible depuis le connecteur GitHub** ; l'exécution physique exige les opérateurs des PC.

## Périmètre après validation

Le noyau (consensus, réseau, stockage, wallet CLI) est stabilisé **avant** le prochain palier. Un explorateur **en lecture seule** peut être développé sur un testnet stabilisé, avant le mainnet, à condition de ne jamais autoriser via son API des écritures critiques ou de bloquer le moteur de consensus. GUI et mobile seulement après cette séparation d'architecture; confidentialité native et récupération des fonds nécessitent leurs propres propositions, compatibilités et audits.

# Exploitation et reprise du testnet expérimental AURORA

**Profil :** transactions testnet v3 ML-DSA-65, stockage par blocs immuables, pages d’index et manifeste `AURORIDX` v6 (CI P2P v4 et états passée; campagnes prolongées en cours). Un serveur lancé avec `--peer IP:PORT` réinterroge périodiquement ces adresses; les transactions en attente sont récupérées périodiquement depuis les pairs configurés et revalidées localement; il n’existe pas d’annonce spontanée aux pairs non configurés. Le chargeur accepte les snapshots `AURORSNP` v4 et les index v5; les prochaines sauvegardes sont écrites en v6. Utiliser des pairs connus sur un réseau contrôlé; aucun déploiement public multi-hôtes n’est déclaré stable.

## Vérifier une installation

Construire à partir d’un commit noté et de `Cargo.lock` :

```sh
cargo build --workspace --locked
cargo test --workspace --locked
python3 scripts/testnet_smoke.py
```

La commande suivante **ne crée pas** de genèse et ne modifie pas le dossier. Elle relit les index/snapshots, vérifie les blocs et paramètres de consensus, puis imprime hauteur, sommet et travail cumulé :

```sh
cargo run --locked -p aurora-node -- testnet verify --data-dir ./aurora-seed
```

Une erreur ou un sommet inattendu interdit de traiter la copie comme une restauration réussie. Comparer la hauteur **et** le hash du sommet avec l’état relevé avant la copie; la hauteur seule ne distingue pas deux branches.

## Sauvegarder

1. Arrêter `testnet serve`, `testnet mine` et `testnet sync` qui utilisent ce dossier. Attendre leur fin et noter la hauteur et le sommet via `testnet verify`.
2. Copier **tout** le dossier de données vers un autre support : les fichiers `snapshot-*.auridx`, les sous-dossiers `blocks-v5/` et `index-pages-v6/`, ainsi que les anciens `*.aurora` s’ils existent. Un manifeste v6 sans ses pages et ses blocs ne constitue pas une sauvegarde. Les fichiers `*.tmp` incomplets sont ignorés au chargement. Le fichier `.aurora.lock` peut être copié; il n’est pas un snapshot et le verrou du système d’exploitation ne reste pas actif dans la copie.
3. Exécuter `testnet verify --data-dir` sur la copie et comparer hauteur/sommet. Garder au moins une copie hors machine et noter le commit du nœud qui a créé l’index. Les fichiers de clés wallet sont séparés du dossier du nœud et demandent une sauvegarde propre.

Exemple PowerShell, nœud arrêté :

```powershell
Copy-Item -LiteralPath .\aurora-seed -Destination .\backup-aurora-seed -Recurse
cargo run --locked -p aurora-node -- testnet verify --data-dir .\backup-aurora-seed
```

### Contrôle automatisé de copie hors processus

Le smoke test `scripts/testnet_smoke.py` vérifie une copie complète d'un
dossier de nœud **arrêté** vers un autre dossier temporaire. Il appelle
`testnet verify` sur la source et sur la copie, puis compare la hauteur et
le hash du sommet. Il retire ensuite les pages v6 d'une copie de test,
puis corrompt un bloc immuable dans une seconde copie indépendante. Chaque
copie dégradée doit être rejetée ou retrouver un état différent de la source :
une sauvegarde incomplète ne doit pas être présentée silencieusement comme
équivalente à l'original.

Ce scénario n'utilise **qu'un seul hôte**. Il ne prouve ni la restauration
hors machine, ni la résistance à une panne électrique ou à un disque plein.
Conserver une sauvegarde réellement hors machine et suivre la procédure ci-dessus.

## Restaurer après une panne

1. Arrêter les processus utilisant le dossier. Garder une copie du dossier endommagé et des journaux pour diagnostiquer la panne.
2. Copier la sauvegarde complète dans **un nouveau dossier**. Exécuter `testnet verify` et contrôler hauteur/sommet. Le chargeur ignore un index corrompu et peut revenir à un index valide antérieur; cette reprise peut perdre les derniers blocs locaux.
3. Si la copie est valide, démarrer `testnet serve` sur ce nouveau dossier ou lancer `testnet sync` vers des pairs connus, puis vérifier de nouveau le sommet. Les versions récentes refusent deux processus `serve`, `mine` ou `sync` sur le même dossier. Le verrou est relâché automatiquement quand le processus se termine, y compris après un arrêt brutal; ne pas supprimer `.aurora.lock` pour contourner le refus. Les archives plus anciennes, dont `237e87d9a33c`, doivent être arrêtées manuellement avant de réutiliser leur dossier.

Le code synchronise chaque nouveau fichier et, sur Unix, son répertoire après renommage. Cela réduit le risque de perte après arrêt brutal; aucun essai de panne électrique réelle ou de disque effectivement plein n’a encore été réalisé.

Si un ancien manifeste d'index est illisible ou a un checksum invalide,
le nettoyage automatique des anciens index est suspendu afin de ne pas
supprimer un point de reprise intact à cause des seules dates de fichiers.
**Cela peut faire croître le nombre de manifestes et les données sur disque**
tant que la corruption persiste. Mettre le nœud hors service, préserver
une copie complète et les journaux, puis diagnostiquer et restaurer sur
une copie vérifiée ; ne pas effacer les manifestes endommagés à l'aveugle.
Ce garde-fou vérifie les octets des manifestes, mais ne garantit pas
l'intégrité de leurs pages et blocs référencés : utiliser `testnet verify`
pour vérifier la chaîne récupérée.

## Mettre à jour et revenir en arrière

- Épingler le commit ou une future version publiée; sauvegarder et vérifier les données avant toute mise à jour. Lancer d’abord le nouveau binaire sur **une copie** du dossier et comparer les paramètres, hauteur et sommet.
- La lecture v4/v5 et l’écriture v6 concernent uniquement le format de stockage compatible avec les mêmes règles de consensus. Elle ne migre pas les anciens réseaux testnet Ed25519/hybrides vers le profil v3 ML-DSA-65, ni les soldes de ces réseaux.
- Si une mise à jour change l’empreinte de protocole ou les règles économiques, créer un nouveau dossier/réseau de test selon son guide. Pour revenir à l’ancien binaire, repartir de la sauvegarde créée avant mise à jour; ne pas supposer qu’il sait lire l’index v6.

## Incident et suivi

Relever périodiquement le sommet et la hauteur de plusieurs nœuds indépendants, les erreurs de pairs, les rejets de snapshots, l’espace disque et la durée des synchronisations. En cas de divergence durable, d’index corrompu ou de panne d’écriture : arrêter le minage concerné, préserver logs et dossiers, noter les commits et sommets de chaque nœud, reproduire sur copie, puis consigner l’incident dans le dépôt, sans y publier de clés ni de données sensibles, avant de reprendre.

Le test local CI couvre 20 blocs, des réorganisations et des reprises simulées. Il ne mesure pas encore la disponibilité multi-hôtes pendant plusieurs jours, les partitions réelles, ni la résistance aux attaques d’un réseau ouvert.

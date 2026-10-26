# Lancer une testnet AURORA locale

Cette procédure utilise des pairs connus sur la même machine ou un réseau privé. La testnet est expérimentale, sans audit ni garantie de résistance aux attaques; n’ouvre pas le port à Internet et n’utilise pas de fonds réels.

## Prérequis

- Rust stable et Cargo, avec accès aux dépendances.
- Sous Windows, Visual C++ Build Tools avec l’éditeur de liens `link.exe`.
- Pour une version testnet à émission lissée, choisis un nouveau dossier de données. Les snapshots de l’ancien calendrier par paliers sont incompatibles; garde une copie si tu tiens aux anciennes données.
- Le mineur passé à `--miner` est un identifiant public de 32 octets en hexadécimal, pas une clé privée.

Construis le nœud :

```sh
cargo build --locked -p aurora-node
```

Le binaire se trouve sous `target/debug/aurora-node` (Linux/macOS) ou `target/debug/aurora-node.exe` (Windows).

## Trois terminaux, une machine

L’exemple mine quelques blocs de départ, lance un pair semence, puis synchronise un second répertoire. Remplace les identifiants de mineur par les tiens; ils ne représentent pas une clé secrète.

**Terminal 1 — créer une chaîne et miner trois blocs :**

```sh
cargo run --locked -p aurora-node -- testnet mine --data-dir ./aurora-seed --miner 4242424242424242424242424242424242424242424242424242424242424242 --count 3
```

**Terminal 2 — servir le pair semence sur l’interface locale :**

```sh
cargo run --locked -p aurora-node -- testnet serve --data-dir ./aurora-seed --listen 127.0.0.1:18444
```

**Terminal 3 — synchroniser un répertoire vide :**

```sh
cargo run --locked -p aurora-node -- testnet sync --data-dir ./aurora-peer-b --peer 127.0.0.1:18444
```

La sortie doit indiquer trois blocs acceptés, hauteur 3 et un sommet identique à celui imprimé lors du minage.

Pour écouter sur une interface LAN, remplace `127.0.0.1` par l’adresse privée de la machine et limite le pare-feu au réseau de test. N’utilise pas `0.0.0.0` sur un réseau public.

## Miner et propager vers un pair

Un mineur peut créer une série de blocs puis les pousser vers un serveur joignable :

```sh
cargo run --locked -p aurora-node -- testnet mine --data-dir ./aurora-miner --miner 4242424242424242424242424242424242424242424242424242424242424242 --count 10 --peer 192.168.1.20:18444
```

Pour configurer un nœud supplémentaire, synchronise d’abord son dossier avec `testnet sync`, puis lance `testnet serve` avec ce dossier. Le serveur et les commandes `mine`/`sync` ne doivent pas utiliser simultanément le même dossier : une commande lit son snapshot au démarrage. Arrête `serve` avant de miner ou synchroniser ce dossier, puis redémarre le serveur pour qu’il recharge le snapshot mis à jour.

Plusieurs destinations `--peer` sont permises pour le mineur. Le nœud ne maintient pas encore de minage continu, de reconnexion automatique ou de mempool. `sync --discover` découvre au plus un saut de pairs annoncés, depuis un pair de démarrage configuré.

## Vérifier le scénario multi-nœuds

Depuis la racine du dépôt :

```sh
python3 scripts/testnet_smoke.py
```

Le scénario démarre des processus séparés, vérifie les trois premiers blocs selon la formule proportionnelle, leur propagation, le rechargement d’un snapshot et la synchronisation complète d’un répertoire neuf. GitHub Actions exécute le même scénario.

## Politique monétaire testnet

- Genèse sans préminage; plafond brut de 500 000 000 AUR.
- Émission par bloc : `min(reliquat, max(1, floor(reliquat / 1_000_000)))` en unités minimales; premier bloc de 500 AUR.
- 5 % de l’émission au compte de trésorerie testnet; 95 % au mineur.
- 50 % des frais au mineur, 50 % brûlés, même après le plafond; après l’épuisement de l’émission, les frais sont la seule source de revenu du mineur.
- Le compte trésorerie actuel n’est pas un smart contract et ne comporte pas de mécanisme de dépense.

Détails et limites : [EMISSION_MODEL.md](EMISSION_MODEL.md), [PROTOCOL.md](PROTOCOL.md).

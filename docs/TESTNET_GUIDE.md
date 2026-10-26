# Lancer une testnet AURORA locale

Cette procédure utilise des pairs connus sur la même machine ou un réseau privé. La testnet est expérimentale, sans audit ni garantie de résistance aux attaques; n’ouvre pas le port à Internet et n’utilise pas de fonds réels.

## Prérequis

- Rust stable et Cargo, avec accès aux dépendances.
- Sous Windows, Visual C++ Build Tools avec l’éditeur de liens `link.exe`.
- Pour le profil testnet natif ML-DSA-65 v3 et l’émission lissée, choisis un nouveau dossier de données. Les anciens réseaux de consensus sont incompatibles; conserve une copie de leurs données. Les snapshots v4 du **même** profil peuvent être relus puis enregistrés en v5.
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

Pour configurer un nœud supplémentaire, synchronise d’abord son dossier avec `testnet sync`, puis lance `testnet serve` avec ce dossier et ses `--peer` statiques. Le serveur réinterroge ces pairs en arrière-plan après son démarrage. Le serveur et les commandes `mine`/`sync` ne doivent pas utiliser simultanément le même dossier : une commande lit son snapshot au démarrage. Arrête `serve` avant de miner ou synchroniser ce dossier, puis redémarre le serveur pour qu’il recharge le snapshot mis à jour.

Plusieurs destinations `--peer` sont permises pour le mineur. Le nœud a un mempool local borné et, en mode `serve --peer`, rattrape périodiquement les pairs configurés. Il ne maintient pas de minage continu ni de gossip des transactions. `sync --discover` découvre au plus un saut de pairs annoncés, depuis un pair de démarrage configuré.

## Portefeuille CLI experimental

Construis aussi le portefeuille :

```sh
cargo build --locked -p aurora-wallet
```

Par défaut, `keygen` crée un fichier chiffré `AURKEY4` : Argon2id (64 MiB, 3 passes) dérive la clé depuis un mot de passe demandé sans écho, puis XChaCha20-Poly1305 protège la graine ML-DSA-65 et authentifie l’en-tête. Sauvegarde le fichier et le mot de passe séparément; une perte du mot de passe est irrécupérable. L’option explicite `--plain-testnet` conserve l’ancien format `AURKEY3` en clair pour les scripts de test uniquement. Aucun fonds réel.

Crée une clé locale :

```sh
cargo run --locked -p aurora-wallet -- keygen --out ./alice.aurkey
cargo run --locked -p aurora-wallet -- address --key-file ./alice.aurkey
```

Mine les premiers blocs vers l’adresse affichée par `keygen`, puis lance ou synchronise un nœud testnet qui sert le RPC wallet. Les opérations wallet utilisent l’adresse IPv4/IPv6 et le port TCP du nœud :

```sh
cargo run --locked -p aurora-wallet -- balance --key-file ./alice.aurkey --node 127.0.0.1:18444
cargo run --locked -p aurora-wallet -- send --key-file ./alice.aurkey --to <64-hex-destinataire> --amount 1.25 --fee 0.00000003 --node 127.0.0.1:18444
```

La commande `send` lit le nonce attendu, signe le transfert v3 avec ML-DSA-65 et le soumet au mempool du nœud. Elle imprime le txid. Vérifie ensuite son état et l’inclusion après minage :

```sh
cargo run --locked -p aurora-wallet -- status --node 127.0.0.1:18444 --txid <64-hex-txid>
cargo run --locked -p aurora-wallet -- wait --node 127.0.0.1:18444 --txid <64-hex-txid> --timeout-seconds 120
```

Le statut `Pending` signifie accepté par le mempool de ce nœud. `Confirmed` signifie inclus dans sa chaîne canonique au moment de la requête; ce prototype ne définit pas de profondeur de finalité. Les frais sont comptabilisés par le consensus : 50 % au mineur, le reste brûlé (l’arrondi à l’unité minimale favorise le burn). Le RPC wallet/mempool est réservé au testnet v3.

## Vérifier le scénario multi-nœuds

Depuis la racine du dépôt :

```sh
python3 scripts/testnet_smoke.py
```

Le scénario démarre des processus séparés, crée un wallet, soumet une transaction ML-DSA au mempool, vérifie son inclusion, la preuve Merkle et le partage des frais. Il atteint la hauteur 20 à travers deux retargets, provoque une réorganisation vers une branche plus lourde et teste la reprise après arrêt forcé ou erreur d’écriture simulée. GitHub Actions exécute le même scénario.

## Vérifier, sauvegarder et restaurer les données

Le stockage courant écrit les blocs une seule fois dans `blocks-v5/` et conserve trois index `snapshot-*.auridx`. Après avoir arrêté les commandes qui utilisent le dossier, vérifie les données sans les modifier :

```sh
cargo run --locked -p aurora-node -- testnet verify --data-dir ./aurora-seed
```

Copie ensuite **tout** le dossier vers un autre emplacement et exécute la même commande sur cette copie; compare la hauteur et le hash du sommet. Un fichier index sans `blocks-v5/` ne suffit pas. La procédure complète de sauvegarde, restauration, mise à jour et incident est dans [TESTNET_OPERATIONS.md](TESTNET_OPERATIONS.md).

## Politique monétaire testnet

- Genèse sans préminage; plafond brut de 500 000 000 AUR.
- Émission par bloc : `min(reliquat, max(1, floor(reliquat / 1_000_000)))` en unités minimales; premier bloc de 500 AUR.
- 5 % de l’émission au compte de trésorerie testnet; 95 % au mineur.
- 50 % des frais au mineur, 50 % brûlés, même après le plafond; après l’épuisement de l’émission, les frais sont la seule source de revenu du mineur.
- Le compte trésorerie actuel n’est pas un smart contract et ne comporte pas de mécanisme de dépense.

Détails et limites : [EMISSION_MODEL.md](EMISSION_MODEL.md), [PROTOCOL.md](PROTOCOL.md).


## Wallet et vérification SPV

Le CLI wallet expérimental et la commande de minage multi-pairs sont dans la même workspace. Génère un wallet de test et garde le fichier de clé chiffré en local. Le CLI demande un mot de passe à la création et à chaque accès :

```sh
cargo run --locked -p aurora-wallet -- keygen --out ./wallet.aurkey
cargo run --locked -p aurora-wallet -- address --key-file ./wallet.aurkey
cargo run --locked -p aurora-wallet -- send --key-file ./wallet.aurkey --to <adresse-hex> --amount 1.25 --fee 0.01 --node 127.0.0.1:18444
```

Après l’inclusion d’une transaction, vérifie sa preuve Merkle et ses confirmations avec :

```sh
cargo run --locked -p aurora-wallet -- verify --node 127.0.0.1:18444 --txid <txid-hex> --confirmations 1
```

Le module URI accepte les demandes `aurora://testnet/<adresse-hex>?amount=...&memo=...`; il ne s’agit pas d’une commande de portefeuille autonome. La vérification SPV s’appuie sur le pair interrogé et ne remplace pas une synchronisation complète ni une revue de sécurité.

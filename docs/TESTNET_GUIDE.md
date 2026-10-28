# Lancer une testnet AURORA locale

Cette procédure utilise des pairs connus sur la même machine ou un réseau privé. La testnet est expérimentale, sans audit ni garantie de résistance aux attaques; n’ouvre pas le port à Internet et n’utilise pas de fonds réels.

## Télécharger les binaires testnet

La [Release expérimentale `testnet-v0.0.1-experimental.1`](https://github.com/antoineforterax-cmd/AURORA/releases/tag/testnet-v0.0.1-experimental.1) fournit des archives **non signées** pour Windows x86-64, Linux x86-64 et macOS arm64, construites depuis `421d443370e83a9ffa0c128da9f09d6f99b0ef8f`. Télécharge l’archive correspondant à ton système et à ton architecture ainsi que le fichier externe `SHA256SUMS.txt`. Compare son empreinte SHA-256 à la ligne portant exactement le nom de l’archive avant de l’extraire. Le [guide d’installation détaillé](guide-fr/03-installation.md) montre les commandes.

La Release s’extrait directement et contient `bin/aurora-node` (ou `.exe`), `bin/aurora-wallet`, les guides et un second `SHA256SUMS.txt` portant sur les fichiers internes. Ce n’est pas un installateur. La [CI `testnet-packages`](https://github.com/antoineforterax-cmd/AURORA/actions/workflows/testnet-packages.yml) continue aussi de produire des artefacts expirant après 14 jours; ceux-ci peuvent contenir une archive imbriquée. Ne confonds pas ces artefacts avec les fichiers pérennes de la Release, ni un build plus récent avec la version précise d’un essai.

Ces binaires évitent de compiler Rust localement. Sur Linux ou macOS, rends le binaire exécutable si l’extraction a perdu cette permission (`chmod +x bin/aurora-node bin/aurora-wallet`). Sous Windows, `link.exe` est requis pour **compiler** depuis les sources, pas pour lancer le binaire fourni.

Le logiciel ne contient pas une copie figée de la blockchain. Chaque nœud conserve son propre dossier `--data-dir` et récupère puis valide les blocs depuis une adresse de pair `--peer` configurée. Le protocole P2P courant est `v4` (inventaire de blocs sur 64 bits) : des binaires plus anciens peuvent être incompatibles entre pairs; sauvegarde ton dossier de données avant de les remplacer. Pour l’instant il n’existe pas de pair public officiel; utilise une testnet locale ou des machines d’un même réseau privé selon les exemples ci-dessous.

## Démarrage rapide avec l’archive Windows (PowerShell)

Dans le dossier extrait qui contient `bin`, utilise trois fenêtres PowerShell. Remplace `<adresse-affichée>` par la valeur `address=` affichée par le wallet.

**Fenêtre 1 — wallet et premiers blocs :**

```powershell
.\bin\aurora-wallet.exe keygen --out .\wallet.aurkey
.\bin\aurora-node.exe testnet mine --data-dir .\node-data --miner <adresse-affichée> --count 3
```

**Fenêtre 2 — serveur local (laisse cette fenêtre ouverte) :**

```powershell
.\bin\aurora-node.exe testnet serve --data-dir .\node-data --listen 127.0.0.1:18444
```

**Fenêtre 3 — wallet et second nœud :**

```powershell
.\bin\aurora-wallet.exe balance --key-file .\wallet.aurkey --node 127.0.0.1:18444
.\bin\aurora-node.exe testnet sync --data-dir .\node-peer --peer 127.0.0.1:18444
.\bin\aurora-node.exe testnet verify --data-dir .\node-peer
```

Une erreur Windows `os error 10048` au lancement de `serve` signifie que le port est déjà occupé, souvent par la première fenêtre serveur. Arrête ce serveur avec `Ctrl+C` avant de le relancer. Si tu mines de nouveaux blocs dans `node-data` après avoir lancé le serveur, arrête et redémarre celui-ci pour qu’il recharge le dossier; puis répète `sync` sur `node-peer`. Les archives récentes refusent deux commandes `mine`, `sync` ou `serve` simultanées sur le même dossier avec un message « already in use or cannot be locked ». Le fichier `.aurora.lock` est conservé dans ce dossier; le verrou est libéré automatiquement à l’arrêt, même après un arrêt brutal. Ne supprime pas ce fichier pour contourner le refus. Les anciennes archives, dont `237e87d9a33c`, n’ont pas encore cette protection : arrête d’abord le serveur avant de miner ou synchroniser son dossier.

## Prérequis

- Pour compiler les sources : Rust stable et Cargo, avec accès aux dépendances. Sous Windows, Visual C++ Build Tools fournit l’éditeur de liens `link.exe`. Les archives CI ci-dessus n’exigent pas ces outils.
- Pour le profil testnet natif ML-DSA-65 v3 et l’émission lissée, choisis un nouveau dossier de données. Les anciens réseaux de consensus sont incompatibles; conserve une copie de leurs données. Les snapshots v4 et index v5 du **même** profil peuvent être relus puis enregistrés en v6.
- Le mineur passé à `--miner` est un identifiant public de 32 octets en hexadécimal, pas une clé privée.

Si tu utilises le code source, construis le nœud :

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

Pour configurer un nœud supplémentaire, synchronise d’abord son dossier avec `testnet sync`, puis lance `testnet serve` avec ce dossier et ses `--peer` statiques. Le serveur réinterroge ces pairs en arrière-plan après son démarrage. Le serveur et les commandes `mine`/`sync` utilisent un verrou exclusif de dossier et ne peuvent pas fonctionner simultanément sur le même dossier : chaque commande charge son état au démarrage. Arrête `serve` avant de miner ou synchroniser ce dossier, puis redémarre le serveur pour qu’il recharge le snapshot mis à jour.

Plusieurs destinations `--peer` sont permises pour le mineur. Le nœud a un mempool local borné et, en mode `serve --peer`, rattrape périodiquement les pairs configurés. Il ne maintient pas de minage continu. Les transactions en attente sont interrogées périodiquement auprès des pairs `--peer` configurés et revalidées localement; elles ne sont pas annoncées spontanément et le mempool n’est pas persistant. `sync --discover` découvre au plus un saut de pairs annoncés, depuis un pair de démarrage configuré.

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

La commande `send` lit le nonce attendu, signe le transfert v3 avec ML-DSA-65 et le soumet au mempool du nœud. Elle imprime le txid.

`balance` et `send` utilisent `GET_ACCOUNT` : le nœud rejoue son mempool sur l’état de sa chaîne avant de renvoyer le solde et le prochain nonce. Il s’agit donc d’un solde et d’un nonce projetés avec les transactions en attente, pas d’une preuve de confirmation. La branche de développement nomme le champ `pending_adjusted_balance`; la Release `421d443` affiche encore `confirmed_balance` pour cette même réponse. Le protocole réseau et le calcul ne changent pas avec cette correction de libellé.

Vérifie ensuite le statut de la transaction et son inclusion après minage :

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

Le scénario démarre des processus séparés, crée un wallet, soumet une transaction ML-DSA au mempool, vérifie son inclusion, la preuve Merkle et le partage des frais. Il atteint la hauteur 20 à travers deux retargets, provoque une réorganisation vers une branche plus lourde, vérifie le relais d’une transaction entre nœuds puis son minage par un autre pair, et teste la reprise après arrêt forcé ou erreur d’écriture simulée. GitHub Actions exécute le même scénario.

## Vérifier, sauvegarder et restaurer les données

Le stockage courant écrit les blocs une seule fois dans `blocks-v5/`, les pages d’index dans `index-pages-v6/` et conserve trois manifestes `snapshot-*.auridx`. Cette version v6 a passé la [CI Rust et multi-nœuds](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40485261614). Un [essai de 100 003 blocs connus](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40527921129) a aussi réussi avec branche secondaire, sauvegarde et rechargement. Il ne prouve pas la synchronisation réseau de ce volume ni la reprise après panne matérielle. Après avoir arrêté les commandes qui utilisent le dossier, vérifie les données sans les modifier :

```sh
cargo run --locked -p aurora-node -- testnet verify --data-dir ./aurora-seed
```

Copie ensuite **tout** le dossier vers un autre emplacement et exécute la même commande sur cette copie; compare la hauteur et le hash du sommet. Un manifeste sans `blocks-v5/` et `index-pages-v6/` ne suffit pas. La procédure complète de sauvegarde, restauration, mise à jour et incident est dans [TESTNET_OPERATIONS.md](TESTNET_OPERATIONS.md).

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

Pour créer une sauvegarde indépendante, `backup` vérifie le mot de passe, puis rechiffre la même graine avec un sel et un nonce nouveaux. Place le fichier produit sur un support distinct et conserve le mot de passe séparément. `restore` vérifie la sauvegarde et crée un **nouveau** fichier chiffré; aucune commande ne remplace un fichier de clé existant.

```sh
cargo run --locked -p aurora-wallet -- backup --key-file ./wallet.aurkey --out ./wallet-sauvegarde.aurkey
cargo run --locked -p aurora-wallet -- restore --from ./wallet-sauvegarde.aurkey --out ./wallet-restaure.aurkey
cargo run --locked -p aurora-wallet -- address --key-file ./wallet-restaure.aurkey
```

Compare l’adresse restaurée à l’adresse d’origine. La sauvegarde du seul dossier du nœud ne contient pas la clé du wallet. Sans fichier de clé ou sans mot de passe, la récupération est impossible. `backup` et `restore` exigent le format chiffré AURKEY4; le format en clair AURKEY3 reste réservé aux essais.

Après l’inclusion d’une transaction, vérifie sa preuve Merkle et ses confirmations avec :

```sh
cargo run --locked -p aurora-wallet -- verify --node 127.0.0.1:18444 --txid <txid-hex> --confirmations 1
```

Le module URI accepte les demandes `aurora://testnet/<adresse-hex>?amount=...&memo=...`; il ne s’agit pas d’une commande de portefeuille autonome. La vérification SPV s’appuie sur le pair interrogé et ne remplace pas une synchronisation complète ni une revue de sécurité.

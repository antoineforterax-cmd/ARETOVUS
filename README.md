# AURORA — blockchain expérimentale

> Prototype de testnet uniquement. Le code et le protocole ne sont pas audités et ne doivent jamais recevoir de fonds réels. Il ne s’agit pas d’un mainnet ni d’une promesse de sécurité.

## État actuel

| Composant | Fonctionnement présent |
|---|---|
| `aurora-core` | Montants entiers à 8 décimales, encodage canonique, transferts de comptes signés et vérifiés par Ed25519 (`ed25519-dalek`). |
| `aurora-consensus` | Validation atomique des transactions/blocs, PoW SHA-256, timestamps au-dessus de la médiane des 11 ancêtres, retarget expérimental, travail cumulé u320, choix de branche et réorganisation de l’état/récompenses. |
| `aurora-storage` | Snapshots `AURORSNP` version 3 avec checksum, paramètres de consensus et toutes les branches; chargement par rejeu et revalidation. |
| `aurora-node` | Minage local testnet, persistance à chaque bloc, serveur TCP de synchronisation pull et commande client de synchronisation. |
| `aurora-wallet` | Réservé; aucun portefeuille ou gestion de clés utilisateur. |

## Règles économiques exécutées

Le nœud testnet utilise le profil existant `fast_start_500` : plafond brut de 500 000 000 AUR et 8 décimales. La récompense par bloc est choisie avec l’offre émise au début du bloc : 500 AUR jusqu’à 5 M émis, 100 jusqu’à 25 M, 60 jusqu’à 75 M, 30 jusqu’à 175 M, 10 jusqu’à 350 M, puis 1 AUR jusqu’au plafond. L’émission du dernier bloc est tronquée au reliquat exact.

Cinq pour cent de l’émission vont à la trésorerie (arrondi inférieur en unités minimales); le reste au mineur. La moitié des frais va au mineur (arrondi inférieur), le reste est brûlé. L’identité du mineur est engagée dans le bloc. Les récompenses sont recalculées sur l’état du parent de chaque branche; une réorganisation remplace donc aussi l’offre et les soldes canoniques.

Ces constantes sont exécutées par le profil de testnet du dépôt; elles n’impliquent aucune approbation de lancement ou valeur monétaire. Les constructeurs génériques de chaîne gardent les récompenses désactivées par défaut.

## Testnet expérimentale locale

Prérequis : Rust stable, dépendances disponibles via crates.io. Les commandes ci-dessous sont des nœuds autonomes sur la même machine ou des hôtes joignables.

```bash
# Afficher les commandes disponibles
cargo run -p aurora-node -- --help

# Miner 10 blocs, avec une identité de mineur de 32 octets écrite en hexadécimal
cargo run -p aurora-node -- testnet mine --data-dir ./aurora-node-a --miner 4242424242424242424242424242424242424242424242424242424242424242 --count 10 --peer 127.0.0.1:18444

# Servir la chaîne locale sur TCP
cargo run -p aurora-node -- testnet serve --data-dir ./aurora-node-a --listen 0.0.0.0:18444

# Synchroniser un second répertoire depuis le pair
cargo run -p aurora-node -- testnet sync --data-dir ./aurora-node-b --peer 127.0.0.1:18444

# Vérification CI locale (si l’environnement Rust possède l’éditeur de liens requis)
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
```

Paramètres testnet codés : difficulté initiale 8 bits nuls en tête, cible 60 secondes, ajustement tous les 10 blocs par pas entiers bornés, min 4/max 20 bits; la trésorerie a l’identifiant fixe de test `[0xA0; 32]`. Le minage examine les nonces de manière déterministe. `--miner` est un identifiant public fourni par l’utilisateur, pas une clé privée.

Le protocole TCP est expérimental : empreinte genèse/paramètres, frames de 4 MiB maximum, blocs consensus limités à 4 MiB, pages jusqu’à 64 blocs, timeout d’E/S de 10 secondes et inventaire maximal de 100 000 blocs. `serve` répond aux demandes et accepte les blocs envoyés; `sync` télécharge l’inventaire une fois et accepte plusieurs options `--peer` séquentielles; `mine --peer` pousse chaque bloc miné à un pair configuré. Chaque bloc reçu est revalidé par le consensus. Il n’y a pas encore de découverte automatique, propagation vers une liste durable de pairs, réseau multi-pairs simultané, mempool, reprise automatique, quotas cumulés de session ni suite complète de tests de pairs hostiles. Ne pas exposer ce service à Internet.

Les tests Rust sont exécutés en CI GitHub Actions. Sur l’environnement Windows de développement, la compilation locale peut échouer si `link.exe` (Visual C++ Build Tools) manque; cela ne remplace pas la CI.

## Limites avant de parler d’une blockchain terminée

Il reste notamment : synchronisation et propagation multi-pairs robustes, découverte/reconnexion, messages malveillants et tests DoS, simulation indépendante des retargets et horodatages, tests de crash/coupure disque et migrations de stockage, fuzzing, tests multi-nœuds reproductibles et revue de sécurité indépendante. L’interface portefeuille, l’explorateur Web et le site de présentation sont volontairement hors de cette étape.

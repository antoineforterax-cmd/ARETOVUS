# AURORA — blockchain expérimentale

> Prototype de testnet uniquement. Le code et le protocole ne sont pas audités et ne doivent jamais recevoir de fonds réels. Il ne s’agit pas d’un mainnet ni d’une promesse de sécurité.

## État actuel

Le testnet expérimental utilise désormais des transactions natives ML-DSA-65 v3; le consensus refuse les transactions Ed25519 v1 et hybrides v2. Les anciens formats sont conservés pour Regtest. Le changement de profil est incompatible avec l’ancien testnet: utiliser un nouveau dossier de données et garder les snapshots précédents. Cela ne rend pas SHA-256/PoW post-quantique. Voir `docs/POST_QUANTUM.md`.

| Composant | Fonctionnement présent |
|---|---|
| `aurora-core` | Montants entiers, encodage canonique, signatures ML-DSA-65 natives v3 sur Testnet, compatibilité Ed25519 v1/hybride v2 sur Regtest. |
| `aurora-consensus` | Validation atomique, PoW SHA-256, temps strictement croissants au-dessus du MTP, retarget borné sur le timestamp candidat, travail cumulé et réorganisations. |
| `aurora-storage` | Snapshots `AURORSNP` version 3 avec checksum, paramètres de consensus et toutes les branches; chargement par rejeu et revalidation. |
| `aurora-node` | Minage local testnet, persistance, TCP push/pull vers pairs statiques, découverte opt-in bornée, 8 sessions serveur simultanées, limites cumulées de 256 MiB/16 384 requêtes/15 minutes et 3 tentatives sur erreur réseau transitoire. |
| `aurora-wallet` | CLI testnet ML-DSA-65 v3 : génération/lecture de clés locales, solde/nonce, signature et soumission de transactions, statut mempool/confirmée. Clé en clair, testnet uniquement. |

## Règles économiques exécutées

Le testnet utilise le plafond brut strict de **500 000 000 AUR**, avec 8 décimales et une genèse sans préminage. Pour chaque bloc, l’émission est calculée en unités minimales comme `min(reliquat, max(1, floor(reliquat / 1_000_000)))`. Le premier bloc crée 500 AUR; l’émission décroît ensuite continûment avec l’offre restante et finit par atteindre le plafond grâce au plancher d’une unité. La règle de consensus s’identifie comme `proportional-remaining-1e6-v1`.

Cinq pour cent de chaque émission vont au compte testnet fixe `[0xA0; 32]`, et 95 % au mineur. La moitié des frais va au mineur, l’autre moitié est brûlée; le burn ne libère jamais du plafond. Une fois le plafond atteint, le mineur est rémunéré uniquement par les frais, selon le même partage 50/50.

Le compte de trésorerie est actuellement crédité automatiquement mais n’a pas de logique de smart contract, de gouvernance ou de dépense. Les gardiens/time-locks et le recyclage des comptes inactifs ne sont pas encore implémentés; ils changent les règles de propriété et doivent être spécifiés et testés séparément avant d’entrer au consensus. Voir `docs/EMISSION_MODEL.md`.

Un changement de règle d’émission crée un réseau incompatible. Utilise un nouveau dossier `--data-dir` pour cette version de testnet; conserve ou sauvegarde tes anciens snapshots au lieu de les supprimer.
## Testnet expérimentale locale

Le guide [docs/TESTNET_GUIDE.md](docs/TESTNET_GUIDE.md) explique comment compiler le nœud sous Linux, macOS ou Windows, synchroniser un pair, lancer un serveur et miner en testnet. Sur Windows, Rust Build Tools doit fournir `link.exe`. Utilise seulement des pairs connus sur localhost ou un réseau privé. Arrête `serve` avant d’exécuter `mine` ou `sync` sur le même dossier de données, puis redémarre le serveur pour recharger le snapshot.

Vérification reproductible :

```bash
python3 scripts/testnet_smoke.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
```

Le smoke test démarre des processus séparés, vérifie l’émission exacte des trois premiers blocs, la répartition 95/5, la propagation push, le rechargement d’un snapshot et le téléchargement pull vers un nouveau dossier. GitHub Actions exécute ces vérifications. Le service TCP reste expérimental, sans reconnexion continue ni annonces spontanées multi-sauts; garde-le sur un réseau contrôlé. Le RPC wallet/mempool est disponible seulement sur le testnet.

## Limites avant de parler d’une blockchain terminée

Il reste des tests prolongés de pairs hostiles, reconnexion automatique, fuzzing, retarget indépendant, arrêts brutaux et revue de sécurité indépendante. La trésorerie créditée n’a pas encore de gouvernance ou de règles de dépense; gardiens/time-locks et recyclage des soldes inactifs sont des changements de propriété à spécifier séparément. Le portefeuille CLI expérimental est disponible. URI de paiement, DID/récupération, preuves Merkle/SPV, explorateur et site restent à concevoir; la vente ou cotation vient plus tard, après sécurité et gouvernance.

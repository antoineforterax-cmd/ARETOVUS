# AURORA — blockchain expérimentale

> Prototype de testnet uniquement. Le code et le protocole ne sont pas audités et ne doivent jamais recevoir de fonds réels. Il ne s’agit pas d’un mainnet ni d’une promesse de sécurité.

## Bienvenue : AURORA en une minute

**AURORA est un prototype de blockchain autonome en Rust.** Un portefeuille signe des transactions, des nœuds les vérifient et les mineurs produisent des blocs grâce à une preuve de travail (PoW). Aujourd'hui, le logiciel se teste sur des machines indépendantes ; **il n'existe pas encore de mainnet ni de garantie adaptée à des fonds réels**.

Notre cap : bâtir d'abord un **noyau fiable, facile à vérifier et à maintenir**, puis étudier une confidentialité plus forte, des protections volontaires contre la perte de fonds et des nœuds accessibles sur du matériel modeste. Ce sont des ambitions de recherche, **pas des fonctionnalités déjà livrées**. Voir le [cahier des charges](docs/planning/CAHIER_DES_CHARGES.md).

### Où aller ?

| Vous cherchez… | Commencez ici |
| --- | --- |
| **Découvrir en français**, sans connaître la blockchain | [Guide débutant : AURORA de A à Z](docs/guide-fr/README.md) |
| **Essayer le logiciel** sur Windows, Linux ou macOS | [Installation et premier démarrage](docs/guide-fr/03-installation.md) |
| **Comprendre la direction du projet** et les fonctionnalités futures | [Vision et feuille de route](docs/planning/README.md) |
| **Savoir ce qui est décidé, proposé ou encore à comparer** | [Tableau vivant des décisions](docs/planning/DECISIONS.md) |
| **Vérifier les règles exactes** et les limites du testnet | [Consensus](docs/CONSENSUS.md) · [Rapport de validation](docs/TESTNET_VALIDATION_REPORT.md) |
| **Aider, auditer ou signaler un problème** | [Contribuer](CONTRIBUTING.md) · [Sécurité](SECURITY.md) |

**Important pour l'installation :** la [release expérimentale du 27 octobre 2026](https://github.com/antoineforterax-cmd/AURORA/releases/tag/testnet-v0.0.1-experimental.1) correspond au commit `421d443370e83a9ffa0c128da9f09d6f99b0ef8f` ; elle n'est pas nécessairement identique au code de `main`. Ne mélangez pas les données ou binaires de profils incompatibles. Pour les essais Windows sur deux PC, voir le [guide dédié](docs/TWO_PC_TEST.md).


## État actuel

Le testnet expérimental utilise désormais des transactions natives ML-DSA-65 v3; le consensus refuse les transactions Ed25519 v1 et hybrides v2. Les anciens formats sont conservés pour Regtest. Le changement de profil est incompatible avec l’ancien testnet: utiliser un nouveau dossier de données et garder les snapshots précédents. Cela ne rend pas SHA-256/PoW post-quantique. Voir `docs/POST_QUANTUM.md`.

| Composant | Fonctionnement présent |
|---|---|
| `aurora-core` | Montants entiers, encodage canonique, signatures ML-DSA-65 natives v3 sur Testnet, compatibilité Ed25519 v1/hybride v2 sur Regtest. |
| `aurora-consensus` | Validation atomique, PoW SHA-256, temps strictement croissants au-dessus du MTP, retarget borné sur le timestamp candidat, travail cumulé et réorganisations. |
| `aurora-storage` | Blocs immuables, pages d’index et manifestes checksummés `AURORIDX` v6, trois générations récentes et une ancre de reprise ancienne, toutes les branches; lecture v4/v5 et revalidation au chargement. La [CI du lot réseau/états](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40488607637) a passé formatage, Clippy, compilation, tests Rust et smoke multi-nœuds. Un [essai de stockage de 100 003 blocs connus](https://github.com/antoineforterax-cmd/AURORA/actions/runs/40527921129), avec branche secondaire et rechargement, a réussi; la synchronisation réseau de ce volume reste à démontrer. |
| `aurora-node` | Minage local testnet, persistance, vérification en lecture seule des données, TCP push/pull v4 à inventaire u64 et reprise de session, rattrapage périodique et relais validé des transactions en attente depuis les pairs configurés, découverte opt-in bornée, 8 sessions serveur simultanées avec au plus 4 par IP et handshake de 2 secondes, limites cumulées de 256 MiB/16 384 requêtes/15 minutes et 3 tentatives sur erreur réseau transitoire. |
| `aurora-wallet` | CLI testnet ML-DSA-65 v3 : clés, solde/nonce, envoi signé, suivi, vérification SPV par preuves Merkle et URI `aurora://` via bibliothèque. Nouvelles clés chiffrées par défaut, sauvegarde/restauration chiffrées; testnet uniquement. |

## Règles économiques exécutées

Le testnet utilise le plafond brut strict de **500 000 000 AUR**, avec 8 décimales et une genèse sans préminage. Pour chaque bloc, l’émission est calculée en unités minimales comme `min(reliquat, max(1, floor(reliquat / 1_000_000)))`. Le premier bloc crée 500 AUR; l’émission décroît ensuite continûment avec l’offre restante et finit par atteindre le plafond grâce au plancher d’une unité. La règle de consensus s’identifie comme `proportional-remaining-1e6-v1`.

Cinq pour cent de chaque émission vont au compte testnet fixe `[0xA0; 32]`, et 95 % au mineur. La moitié des frais va au mineur, l’autre moitié est brûlée; le burn ne libère jamais du plafond. Une fois le plafond atteint, le mineur est rémunéré uniquement par les frais, selon le même partage 50/50.

Le compte de trésorerie est actuellement crédité automatiquement mais n’a pas de logique de smart contract, de gouvernance ou de dépense. Les gardiens/time-locks et le recyclage des comptes inactifs ne sont pas encore implémentés; ils changent les règles de propriété et doivent être spécifiés et testés séparément avant d’entrer au consensus. Voir `docs/EMISSION_MODEL.md`.

Un changement de règle d’émission crée un réseau incompatible. Utilise un nouveau dossier `--data-dir` pour cette version de testnet; conserve ou sauvegarde tes anciens snapshots au lieu de les supprimer.

## Testnet expérimentale locale

Le guide [docs/TESTNET_GUIDE.md](docs/TESTNET_GUIDE.md) explique comment télécharger la Release expérimentale pour Linux x86-64, macOS arm64 ou Windows x86-64, ou compiler le nœud, synchroniser un pair, lancer un serveur et miner en testnet. Sur Windows, Rust Build Tools et `link.exe` ne servent qu’à la compilation depuis les sources. Utilise seulement des pairs connus sur localhost ou un réseau privé. Arrête `serve` avant d’exécuter `mine` ou `sync` sur le même dossier de données, puis redémarre le serveur pour recharger le snapshot. Le [parcours débutant](docs/guide-fr/04-reseau-et-minage.md) utilise plutôt des dossiers séparés pour conserver le serveur en fonctionnement pendant le minage.

Vérification reproductible :

```bash
python3 scripts/testnet_smoke.py
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
```

Le smoke test démarre plusieurs processus, vérifie les récompenses, envoie et confirme un paiement wallet avec preuve SPV, traverse deux fenêtres de retarget jusqu’à 20 blocs, redémarre un pair, synchronise un dossier neuf et vérifie le relais d’une transaction en attente jusqu’à son minage par un autre pair. Il force aussi l’arrêt d’un mineur entre deux sauvegardes et simule une erreur d’écriture avec une limite de taille de fichier sous Linux. GitHub Actions exécute ces vérifications. Les tests fuzz déterministes tournent dans la CI principale; les cibles libFuzzer sont dans le workflow séparé et exécutées pour 5 minutes chacune. Voir le [rapport d’essais](docs/TESTNET_VALIDATION_REPORT.md), [guide fuzzing](docs/FUZZING.md), [procédures de sauvegarde et reprise](docs/TESTNET_OPERATIONS.md), [décisions testnet](docs/TESTNET_DECISIONS.md), [feuille de route](docs/ROADMAP_EXECUTION.md) et [fiche complète des travaux restants](docs/FINALIZATION_CHECKLIST.md) pour les résultats, les limites, les contre-revues et le périmètre actuel. Le service TCP rattrape périodiquement ses pairs configurés et leurs transactions en attente, mais n’annonce pas spontanément des blocs à plusieurs sauts; garde-le sur un réseau contrôlé. Le RPC wallet/mempool et les RPC d’en-têtes/preuves sont disponibles seulement sur le testnet.

## Limites avant de parler d’une blockchain terminée

Il reste notamment les simulations approfondies d’[horodatages futurs](https://github.com/antoineforterax-cmd/AURORA/issues/3), la [saturation des workers P2P](https://github.com/antoineforterax-cmd/AURORA/issues/4), la preuve d’une synchronisation réseau au-delà de 100 000 blocs connus ([issue #5](https://github.com/antoineforterax-cmd/AURORA/issues/5)), les campagnes fuzz longues, les essais prolongés de pairs hostiles, une simulation indépendante du retarget, des tests de restauration hors machine et un essai sur volume réellement saturé. Une limite de taille de fichier simule déjà une écriture interrompue sous Linux, mais ce n’est pas une panne électrique. Aucune revue indépendante n’a été réalisée; voir le dossier [docs/INDEPENDENT_REVIEW.md](docs/INDEPENDENT_REVIEW.md). Le DID/récupération a un document de conception, mais n’altère pas les comptes. Le portefeuille CLI expérimental propose maintenant une vérification SPV et une bibliothèque d’URI de paiement; cela ne remplace pas une validation par un nœud complet. La trésorerie n’a pas de gouvernance ni de règles de dépense. Les mécanismes de récupération et de recyclage des soldes restent hors consensus. Explorateur, site, vente simulée et mobile sont hors du périmètre actuel; aucun usage de fonds réels ou présentation de sécurité mainnet n’est justifié sans stabilisation, revue indépendante et analyse juridique de la vente.

## Hors périmètre / pour les repreneurs

Le travail actuel porte sur le nœud, le consensus, le stockage, le réseau, le wallet CLI et leurs preuves de fonctionnement. L’explorateur, le site de présentation, la vente ou l’échange simulé, les interfaces graphiques et le mobile sont reportés. Aucune de ces applications n’est annoncée comme disponible.

## Contribuer et signaler un problème

Lire [ARCHITECTURE.md](ARCHITECTURE.md), [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md) et [PROVENANCE.md](PROVENANCE.md). Les premiers travaux proposés sont dans les issues `good first issue`.

## Licence

Le code AURORA est proposé sous **MIT OR Apache-2.0**, au choix : [LICENSE](LICENSE), [MIT](LICENSE-MIT), [Apache-2.0](LICENSE-APACHE). Les licences des dépendances restent applicables.

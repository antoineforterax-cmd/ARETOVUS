# ARETOVUS V2 / V5 — pages canoniques de blocs (prototype)

Cette première tranche d'explorateur **lit les blocs canoniques d'un snapshot testnet natif validé**. Elle ne démarre pas de réseau, ne mine pas, ne signe pas et ne fait pas confiance à une base indexée externe. Elle ne fournit pas encore la recherche des transactions, des adresses ou des forks.

Après compilation du nœud depuis le même SHA :

    cargo build --locked -p aretovus-node --bin aretovus-node --bin aretovus-explorer
    target/debug/aretovus-explorer --data-dir ./testnet-jetable --start 0 --limit 10

La commande retourne un objet JSON de schéma v1 contenant : `network=testnet`, `status=verified_local`, `source=aretovus-native-explorer`, `read_only=true`, `chain_tip`, `chain_height`, `start`, `blocks`, `next_start`. Chaque bloc contient la hauteur, le hash validé, le parent, l'horodatage, le nonce, le nombre de transactions et la racine Merkle. La pagination n'accepte jamais plus de 20 blocs ; `next_start=null` indique la fin du snapshot consulté. Aucune information privée n'est affichée. Un dossier absent est rejeté, sans être créé.

**Ne pas confondre validation d'un snapshot local et état mondial du réseau.** Une réorganisation peut changer la liste canonique entre deux lectures ; les clients doivent vérifier `chain_tip` et réinitialiser leurs pages si le sommet change. La sortie ne prouve ni la connectivité P2P ni la finalité. Les formats de nombres sont des entiers JSON (la future UI devra les borner à `Number.MAX_SAFE_INTEGER` ou les traiter en BigInt).

**Sécurité et ressources :** une exécution ouvre le snapshot et le revalide, ce qui peut consommer CPU/RAM/disque. Un conteneur desktop devra limiter fréquence, durée et nombre de processus ; l'explorateur n'a aucune connexion réseau ni port RPC. Les erreurs sont des codes constants sans chemins personnels. La chaîne et ses règles ne sont jamais modifiées.

Avant toute fusion : formatage, Clippy, compilation et tests Rust sur les trois OS ; intégration réelle jetable sur Linux avec un bloc miné et pagination d'après la genèse, tests d'options hostiles et de dossier absent. La PR ne constitue ni une interface complète ni un audit indépendant.

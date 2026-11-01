# Architecture d’ARETOVUS

ARETOVUS est un testnet expérimental par comptes. Les montants sont des entiers à huit décimales. Le profil actuel combine transactions v3 ML-DSA-65, en-têtes v1 avec racine Merkle, stockage v6 et transport P2P v4. Ces numéros désignent des formats différents.

| Crate | Responsabilité | Dépendances internes |
| --- | --- | --- |
| `aretovus-core` | Montants, comptes, encodage canonique, domaines de hash, signatures et transitions de compte. | Aucune. |
| `aretovus-consensus` | Validation des blocs, PoW, retarget, horodatages, émission, choix de chaîne, réorganisations et chaîne d’en-têtes. | `aretovus-core`. |
| `aretovus-storage` | Blocs immuables, pages d’index et manifestes v6; lecture v4/v5, sommes de contrôle et rejeu. | Core et consensus. |
| `aretovus-node` | Commandes mine/sync/serve/verify, verrou de dossier, transport TCP, mempool et RPC wallet/SPV. | Core, consensus et stockage. |
| `aretovus-wallet` | Clés chiffrées, sauvegarde/restauration, construction et signature des paiements, RPC, URI et preuves SPV. | Core et consensus. |

Une transaction est décodée et vérifiée avant son admission. Un bloc engage les transactions et le mineur dans son en-tête; son PoW et ses transitions sont vérifiés contre l’état de son parent. La branche au travail cumulé maximal devient canonique, avec départage déterministe par hash. L’état du sommet et des points de contrôle sont conservés; les autres états sont reconstruits à partir des blocs validés. Ce rejeu et les copies de chaîne doivent encore être mesurés sous charge hostile.

Le stockage publie le manifeste après les blocs et pages auxquels il fait référence. Au redémarrage, il vérifie les objets et rejoue les branches. Le P2P échange des pages bornées en ordre d’arrivée, contrôle les paramètres du réseau et revalide les données reçues. Le wallet vérifie les preuves d’inclusion, mais ne remplace pas un nœud complet.

Les règles économiques, le format réseau et les limites sont décrits dans [CONSENSUS](docs/CONSENSUS.md), [PROTOCOL](docs/PROTOCOL.md) et le [rapport d’essais](docs/TESTNET_VALIDATION_REPORT.md). Les interfaces graphiques, le site, l’explorateur, la vente et le mobile sont hors du périmètre actuel.

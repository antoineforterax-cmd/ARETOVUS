# AURORA, de A à Z

Ce guide explique le projet et permet de faire un premier essai sans compiler le code. Sa version de référence est **`testnet-v0.0.1-experimental.1`**, issue du commit **`421d443370e83a9ffa0c128da9f09d6f99b0ef8f`**, publiée le 27 octobre 2026. Le développement peut avancer après cette version : les paramètres d’un essai doivent rester rattachés au binaire utilisé.

AURORA est une blockchain expérimentale. Ses unités de test servent à éprouver le logiciel; elles ne doivent pas être traitées comme des fonds réels. Le projet n’a reçu aucun audit humain indépendant.

## Parcours de lecture

| Pour… | Lire… |
| --- | --- |
| Comprendre le but, les mots et les composants | [1. Comprendre AURORA](01-comprendre.md) |
| Retrouver les règles monétaires exécutées | [2. Économie](02-economie.md) |
| Télécharger et vérifier les programmes | [3. Installation](03-installation.md) |
| Lancer des nœuds et miner sur un PC | [4. Premier réseau local](04-reseau-et-minage.md) |
| Envoyer un paiement et sauvegarder les clés | [5. Portefeuille et paiements](05-portefeuille.md) |
| Vérifier, sauvegarder, restaurer et dépanner | [6. Exploitation](06-exploitation.md) |
| Comprendre les choix techniques et la suite | [7. Technologie et limites](07-technologie-et-suite.md) |

Les exemples principaux utilisent PowerShell sous Windows. Ils fonctionnent avec le contenu d’une archive extraite : `bin/aurora-node.exe` et `bin/aurora-wallet.exe`. Pour Linux et macOS, utiliser les mêmes arguments, des chemins avec `/` et les binaires sans `.exe`.

Le logiciel peut tourner entièrement sur un PC : des processus distincts possèdent chacun leur dossier et leur port. Deux fenêtres ne suffisent pas, à elles seules, à créer deux nœuds indépendants; il faut aussi deux dossiers de données. Un essai local ne démontre pas le comportement entre machines ou sur Internet.

## Sources et état du guide

Rédaction vérifiée le 28 octobre 2026. Les commandes ont été rapprochées des interfaces du [nœud](../../crates/aurora-node/src/main.rs) et du [wallet](../../crates/aurora-wallet/src/main.rs). Les références techniques sont la [spécification](../PROTOCOL.md), les [décisions testnet](../TESTNET_DECISIONS.md) et le [rapport d’essais](../TESTNET_VALIDATION_REPORT.md). Les pages de ce guide résument ces sources; elles ne remplacent pas le code de consensus.

Le code a été largement produit et révisé avec une IA. Les vérifications réalisées et leur portée sont décrites dans [PROVENANCE.md](../../PROVENANCE.md). Pour participer, lire [CONTRIBUTING.md](../../CONTRIBUTING.md); pour signaler une faille, suivre [SECURITY.md](../../SECURITY.md).

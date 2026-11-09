# ARETOVUS — État du projet et source de vérité

> **Photographie documentaire du 9 novembre 2026**, basée sur `main@caa94228b2fb769e3a5ea479ea5fd07485744489`, les fichiers du dépôt et les issues ouvertes. Une mise à jour ultérieure peut avoir changé cet état : **relever le SHA avant toute conclusion**. Ce document n'est ni un audit ni une nouvelle validation du comportement réel.

## La réponse courte

**ARETOVUS est une blockchain expérimentale en Rust, avec une chaîne de test et des outils de minage, portefeuille et réseau. Elle n'est pas un mainnet et n'est pas prête pour des fonds réels.** Elle est largement développée avec assistance IA, sous direction humaine. Le but immédiat est d'en faire un laboratoire documenté, reproductible et ouvert aux contributions. Une monnaie utilisable publiquement avec valeur économique demeure une possibilité future, pas une promesse.

## Ce qui existe dans le code du testnet

| Partie | Capacités documentées | Ce que cela ne prouve pas |
| --- | --- | --- |
| `aretovus-core` | Comptes et nonces, montants entiers, signatures ML-DSA-65 natives du testnet, encodages canoniques | Audit de l'implémentation cryptographique |
| `aretovus-consensus` | Validation et rejeu, PoW SHA-256, retarget, émission, réorganisations | Résistance économique à une attaque de majorité |
| `aretovus-storage` | Blocs, index et manifestes vérifiés, reprises et restaurations locales | Résistance démontrée à toutes les pannes physiques |
| `aretovus-node` | CLI nœud/minage, TCP/P2P avec pairs configurés, relais et synchronisation, RPC testnet | Décentralisation ouverte et robustesse Internet à grande échelle |
| `aretovus-wallet` | CLI de test : clés chiffrées, transactions signées, sauvegardes et preuves SPV | Portefeuille commercial sûr pour des fonds réels |
| `apps/desktop` | Prototype Tauri/React en lecture seule avec état du nœud et pages de blocs, contrôles Windows documentés | Produit desktop complet avec wallet graphique, minage et installation signée |

Sources de référence : [architecture](../ARCHITECTURE.md), [protocole](PROTOCOL.md), [tests](TESTNET_VALIDATION_REPORT.md), [feuille V2](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178). Toute affirmation de test doit être rattachée à un **commit et un run**.

## Paramètres expérimentaux en vigueur dans la documentation technique

- Réseau natif ARETOVUS **incompatible avec l'ancien réseau** ; genèse vide et dossiers de données distincts.
- **Comptes** à solde et nonce, précision de **8 décimales**, signatures **ML-DSA-65 v3** sur Testnet ; anciens profils limités à Regtest.
- **PoW SHA-256**, objectif statistique de **60 secondes par bloc**, difficulté initiale 8 bits, ajustement par fenêtre de 10 blocs (bornes 4–20 bits).
- Plafond **brut d'émission de 500 millions d'unités ARETOVUS**, sans préminage ; émission proportionnelle au reliquat, première récompense brute de 500 ARETOVUS.
- **5 %** de l'émission vont à une adresse de trésorerie **fixe et sans mécanisme de dépense**, 95 % au mineur ; sur les frais, `floor(F/2)` est versé au mineur et le **reste est brûlé**.
- Les choix économiques ci-dessus sont des **règles exécutées du testnet**, **pas des engagements arrêtés pour un futur mainnet**.

Consulter [consensus](CONSENSUS.md), [émission](EMISSION_MODEL.md), [décisions testnet](TESTNET_DECISIONS.md) et surtout le code au SHA choisi pour les formats et invariants précis.

## Ce qui n'est pas livré ou prouvé

- Anonymat fort des transactions, protection des métadonnées avec Tor/I2P, montants cachés : **recherches/propositions**, pas propriétés démontrées du testnet.
- Récupération sociale des accès, gardiens, contrats de dépense du compte de trésorerie, gouvernance effective de ces fonds : **non implémentés**.
- Résistance aux ASIC, sécurité post-quantique de **tout** le protocole : **non démontrées**. ML-DSA-65 ne suffit pas à établir ces deux propriétés.
- Échange financier intégré, wallet mobile abouti, smart contracts généralistes, portefeuille graphique complet : **non livrés**.
- Audit humain externe indépendant approfondi et documenté : **non confirmé**.
- Réseau Internet public éprouvé pendant une longue période entre plusieurs machines indépendantes et lancement avec fonds réels : **non validés**.

## Comment lire l'avancement sans confusion

1. Pour le **développement V2**, suivre [l'issue #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) et les PR ouvertes. Les cases de l'issue reflètent une progression documentée, qu'il faut confronter au SHA et à la CI.
2. Pour les **jalons antérieurs A à E**, conserver [l'issue #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142) comme historique.
3. Pour les **preuves**, consulter [GitHub Actions](https://github.com/antoineforterax-cmd/ARETOVUS/actions) et le [rapport d'essais](TESTNET_VALIDATION_REPORT.md).
4. Pour les **releases**, vérifier qu'un binaire correspond exactement à sa version ; la [release d'octobre](https://github.com/antoineforterax-cmd/ARETOVUS/releases/tag/testnet-v0.0.1-experimental.1) n'est pas automatiquement le dernier code de `main`.
5. Une branche ouverte, une PR en brouillon, une CI verte, un test deux-PC et une revue extérieure sont **des états distincts**.

**Décision de sécurité : NO-GO pour mainnet et fonds réels jusqu'à preuves et décisions spécifiques.** Un futur feu vert nécessitera notamment une revue humaine indépendante, une validation multi-hôtes, une analyse de la sécurité économique et un examen juridique adapté. Voir [conditions de revue](INDEPENDENT_REVIEW.md).
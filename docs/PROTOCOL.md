# AURORA — spécification expérimentale v0

**Version de travail :** 0.1-expérimental. **Branche :** `feature/emission-schedule-v0`. **Usage :** expériences locales/testnet, aucun fonds réel. Les règles sont exécutables et couvertes par CI, mais le protocole n’a pas reçu de revue indépendante et n’est pas prêt pour un mainnet.

Ce document décrit l’implémentation actuelle. Les questions de recherche, menaces et décisions futures restent dans `DECISION_REGISTER.md`; ce brouillon ne transforme pas une hypothèse en garantie de sécurité.

## Profil transaction/état

- En v1, un compte est la clé publique Ed25519 brute de 32 octets. Le profil v2 optionnel exige Ed25519 et ML-DSA-65; l’identifiant engage les deux clés et les deux signatures autorisent le même digest. La signature est de 64 octets et porte sur l’empreinte canonique versionnée, réseau, expéditeur, nonce, destinataire, montant et frais.
- Les montants sont des entiers non signés à 8 décimales. Encodage réseau big-endian et décodage canonique; formats incorrects, données excédentaires et dépassements sont rejetés.
- Une transaction débite montant + frais, exige la signature exacte et le prochain nonce. Le lot de transactions est appliqué atomiquement.
- Le bloc engage la liste ordonnée des identifiants de transactions et l’identité de mineur. Ce n’est pas un arbre de Merkle.

## Entête, PoW et sélection

L’en-tête version 0 porte l’identifiant réseau, le hash parent, l’engagement du bloc, l’horodatage et le nonce. Le hash SHA-256 de l’encodage canonique doit commencer par le nombre requis de bits nuls. Le bloc doit avoir un horodatage strictement supérieur à la médiane des onze ancêtres disponibles.

À chaque hauteur multiple de l’intervalle, le retarget mesure `timestamp_candidat - timestamp_ancêtre(H - intervalle)`, avec un minimum de 1 seconde. Ce delta couvre les espacements de blocs visés et inclut le candidat. Chaque timestamp est strictement supérieur à celui du parent et au temps médian des 11 ancêtres. Aucun plafond lié à l’horloge locale n’est une règle de consensus. L’ajustement reste borné par fenêtre et par difficulté min/max.

## Émission et frais

Le profil testnet sélectionne `fast_start_500` selon l’offre brute déjà émise au début du bloc. Plafond : 500 000 000 AUR. Récompenses : 500 AUR avant 5 M émis; 100 avant 25 M; 60 avant 75 M; 30 avant 175 M; 10 avant 350 M; 1 ensuite jusqu’au plafond. La dernière émission est tronquée au reliquat.

Cinq pour cent de l’émission (arrondi inférieur en unité minimale) vont au compte trésorerie testnet `[0xA0; 32]`; le mineur reçoit le reste. La moitié des frais (arrondi inférieur) va au mineur, le reliquat des frais est brûlé. L’offre brute ne dépasse pas le plafond. Les récompenses sont incluses dans l’état par branche et recalculées lors d’une réorganisation. Les chaînes génériques désactivent les récompenses par défaut.

## Configuration testnet et persistance

Le nœud construit une genèse testnet vide et déterministe. Paramètres actuels : 8 bits initiaux, cible 60 secondes, fenêtre de 10 blocs, ajustement maximal de 2 bits par fenêtre, bornes 4 à 20 bits. La configuration persistée et une empreinte de protocole distinguent ce testnet des configurations incompatibles.

`AURORSNP` version 3 stocke les paramètres et tous les blocs acceptés des branches connues avec un checksum SHA-256. Au chargement, les blocs sont décodés canoniquement, rejoués dans l’ordre topologique et validés à nouveau depuis l’état de genèse déterministe. Un snapshot invalide est ignoré si une sauvegarde valide existe; la reprise choisit ensuite la chaîne valide au travail cumulé maximal. C’est un snapshot simple; journal de base de données, migration formelle, sauvegarde et essais de panne brutale ne sont pas livrés.

## Réseau expérimental

Le transport actuel est TCP push/pull vers des pairs configurés : handshake de fingerprint, frames à longueur préfixée, taille maximale de 4 MiB, blocs consensus au plus à 4 MiB, pages de 64 blocs, inventaire maximal de 100 000 blocs, 256 MiB de trafic cumulé par session, 16 384 requêtes maximum, durée de session de 15 minutes et délai d’E/S de 10 secondes. Le client synchronise les blocs en ordre topologique et n’enregistre que ceux que le consensus accepte; synchronisation et propagation réessaient jusqu’à trois fois les erreurs de connexion transitoires, avec délai croissant. Les erreurs de protocole et consensus ne sont pas répétées. `serve` répond aux pages et accepte les blocs soumis après validation; `sync` récupère l’état annoncé une fois auprès de chaque `--peer` fourni, en séquence; `mine --peer` transmet les blocs fraîchement minés aux pairs indiqués.

Ne sont pas implémentés : découverte automatique de pairs, annonces spontanées et reprise automatique; le serveur accepte plusieurs sessions simultanées mais la synchronisation côté client reste séquentielle. Le mempool n’est pas implémenté; découverte/reconnexion, mais le protocole borne désormais le trafic cumulé, le nombre de demandes et la durée de chaque session; une défense éprouvée contre les attaques DoS reste à faire. Ne pas exposer ce transport à un réseau non fiable.

## Commandes

```bash
cargo run -p aurora-node -- testnet mine --data-dir ./node-a --miner <64 caractères hex> --count 10
cargo run -p aurora-node -- testnet serve --data-dir ./node-a --listen 0.0.0.0:18444
cargo run -p aurora-node -- testnet sync --data-dir ./node-b --peer 127.0.0.1:18444
```

## Travaux de finalisation

1. Compléter les plafonds globaux aux ressources/temps des sessions et tester la cohérence des limites de blocs dans tous les chemins de décodage.
2. Faire des tests d’intégration multi-processus et des tests de pairs adversariaux : frames malformées, inventaire menteur, bloc invalide, coupure et reprise de sync.
3. Compléter découverte, annonces, reconnexion et gestion de plusieurs pairs, ou définir explicitement un testnet supervisé à pairs statiques.
4. Simuler et comparer le retarget avec vecteurs indépendants, dérive temporelle, longues pauses, rafales et réorganisations.
5. Éprouver interruption d’écriture, disque plein, sauvegarde/restauration et migrations de snapshot.
6. Ajouter fuzzing du décodage réseau/transactions/blocs et revue indépendante du consensus et du code.

Portefeuille, explorateur et site de présentation sont une étape ultérieure, hors de ce travail sur le mécanisme blockchain.

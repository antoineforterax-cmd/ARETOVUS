# AURORA — spécification expérimentale v0

**Version de travail :** 0.1-expérimental. **Branche :** `feature/emission-schedule-v0`. **Usage :** expériences locales/testnet, aucun fonds réel. Les règles sont exécutables et couvertes par CI, mais le protocole n’a pas reçu de revue indépendante et n’est pas prêt pour un mainnet.

Ce document décrit l’implémentation actuelle. Les questions de recherche, menaces et décisions futures restent dans `DECISION_REGISTER.md`; ce brouillon ne transforme pas une hypothèse en garantie de sécurité.

## Profil transaction/état

Le passage au profil Testnet v3 est incompatible avec l’ancien testnet hybride. Il faut lancer les nœuds v3 avec un nouveau dossier `--data-dir`; conserver les snapshots antérieurs au cas où une comparaison ou une migration outil serait nécessaire. Aucune migration automatique des soldes ou des identifiants n’est définie. Ce profil remplace l’algorithme de signature des transactions, mais SHA-256 reste utilisé pour les hashes et le PoW: le protocole entier n’est donc pas qualifié de post-quantique.

- Le testnet accepte uniquement les transactions v3: clé publique ML-DSA-65 de 1 952 octets et signature ML-DSA-65 de 3 309 octets; `sender` est le hash domaine-séparé de la clé publique. Le digest signé engage la version, le réseau, l’expéditeur, le nonce, le destinataire, le montant et les frais. Les profils v1 Ed25519 et v2 hybride sont conservés pour Regtest et rejetés par consensus sur Testnet. Le handshake P2P distingue le nouveau profil.
- Les montants sont des entiers non signés à 8 décimales. Encodage réseau big-endian et décodage canonique; formats incorrects, données excédentaires et dépassements sont rejetés.
- Une transaction débite montant + frais, exige la signature exacte et le prochain nonce. Le lot de transactions est appliqué atomiquement.
- Le bloc engage la liste ordonnée des identifiants de transactions et l’identité de mineur. Ce n’est pas un arbre de Merkle.

## Entête, PoW et sélection

L’en-tête version 0 porte l’identifiant réseau, le hash parent, l’engagement du bloc, l’horodatage et le nonce. Le hash SHA-256 de l’encodage canonique doit commencer par le nombre requis de bits nuls. Le bloc doit avoir un horodatage strictement supérieur à la médiane des onze ancêtres disponibles.

À chaque hauteur multiple de l’intervalle, le retarget mesure `timestamp_candidat - timestamp_ancêtre(H - intervalle)`, avec un minimum de 1 seconde. Ce delta couvre les espacements de blocs visés et inclut le candidat. Chaque timestamp est strictement supérieur à celui du parent et au temps médian des 11 ancêtres. Aucun plafond lié à l’horloge locale n’est une règle de consensus. L’ajustement reste borné par fenêtre et par difficulté min/max.

## Émission et frais

Le testnet applique l’émission `proportional-remaining-1e6-v1` : en unités minimales, `R = min(cap - issued, max(1, floor((cap - issued) / 1_000_000)))`. Le plafond est 500 000 000 AUR et la genèse vide ne crée aucun jeton. Le premier bloc émet 500 AUR; ensuite l’émission décroît proportionnellement au reliquat, avec arrondis entiers et un plancher d’une unité.

Cinq pour cent de l’émission (arrondi inférieur en unité minimale) vont au compte trésorerie testnet `[0xA0; 32]`; le mineur reçoit le reste. La moitié des frais (arrondi inférieur) va au mineur, le reliquat des frais est brûlé. L’offre brute ne dépasse pas le plafond. Les récompenses sont incluses dans l’état par branche et recalculées lors d’une réorganisation. Les chaînes génériques désactivent les récompenses par défaut. Une fois le plafond atteint, l’émission est nulle et le revenu du mineur vient des frais; le partage de frais reste 50 % mineur / 50 % brûlé.

## Configuration testnet et persistance

Le nœud construit une genèse testnet vide et déterministe. Paramètres actuels : 8 bits initiaux, cible 60 secondes, fenêtre de 10 blocs, ajustement maximal de 2 bits par fenêtre, bornes 4 à 20 bits. La configuration persistée et une empreinte de protocole distinguent ce testnet des configurations incompatibles.

`AURORSNP` version 3 stocke les paramètres et tous les blocs acceptés des branches connues avec un checksum SHA-256. Au chargement, les blocs sont décodés canoniquement, rejoués dans l’ordre topologique et validés à nouveau depuis l’état de genèse déterministe. Un snapshot invalide est ignoré si une sauvegarde valide existe; la reprise choisit ensuite la chaîne valide au travail cumulé maximal. C’est un snapshot simple; journal de base de données, migration formelle, sauvegarde et essais de panne brutale ne sont pas livrés.

## Réseau expérimental

Le transport actuel est TCP push/pull vers des pairs configurés : handshake de fingerprint, frames à longueur préfixée, taille maximale de 4 MiB, blocs consensus au plus à 4 MiB, pages de 64 blocs, inventaire maximal de 100 000 blocs, 256 MiB de trafic cumulé par session, 16 384 requêtes maximum, durée de session de 15 minutes et délai d’E/S de 10 secondes. Le serveur peut annoncer jusqu’à 32 adresses IP statiques via `GET_PEERS`; la commande `sync --discover` les explore en un seul saut, avec au plus 64 pairs contactés. La découverte est opt-in et n’utilise pas DNS. Chaque pair est soumis au handshake de fingerprint, et chaque bloc reçu est revalidé par le consensus. Le client synchronise les blocs en ordre topologique et n’enregistre que ceux que le consensus accepte; synchronisation et propagation réessaient jusqu’à trois fois les erreurs de connexion transitoires, avec délai croissant. Les erreurs de protocole et consensus ne sont pas répétées. `serve` répond aux pages et accepte les blocs soumis après validation; `sync` récupère l’état annoncé une fois auprès de chaque `--peer` fourni, en séquence; `mine --peer` transmet les blocs fraîchement minés aux pairs indiqués.

La découverte est opt-in, limitée à un saut depuis des bootstrap peers statiques. Il n’existe pas de reconnexion continue, d’annonce spontanée multi-sauts, ni de mempool. Une smoke test multi-processus est exécutée en CI : elle vérifie push, rechargement après arrêt des pairs, pull vers un stockage neuf et égalité de sommet. Ces contrôles ciblés ne remplacent pas les tests longue durée ni une défense éprouvée contre les attaques DoS. Garder cette testnet sur un réseau contrôlé.

## Commandes

```bash
cargo run -p aurora-node -- testnet mine --data-dir ./node-a --miner <64 caractères hex> --count 10
cargo run -p aurora-node -- testnet serve --data-dir ./node-a --listen 0.0.0.0:18444 --peer 192.0.2.20:18444
cargo run -p aurora-node -- testnet sync --data-dir ./node-b --peer 127.0.0.1:18444 --discover
```

## État de préparation

Le réseau expérimental fonctionne pour une testnet supervisée à pairs connus; le push/pull, la reprise du snapshot et la convergence de nœuds distincts passent le scénario multi-processus de CI. Ce jalon ne signifie pas que le réseau est décentralisé, toujours connecté ou résistant aux attaques.

Avant d’élargir les essais, restent à faire :
1. Reconnexion continue et annonces multi-sauts spontanées, si l’usage les exige; le profil actuel suppose un démarrage supervisé avec adresses statiques.
2. Scénarios prolongés de coupure/reprise, inventaires menteurs, connexions lentes et saturation concurrente.
3. Simulation indépendante du retarget avec dérive d’horloge, longues pauses, rafales et réorganisations.
4. Tests d’interruption d’écriture, disque plein, sauvegarde/restauration et migrations de snapshot.
5. Fuzzing du décodage réseau/transactions/blocs et revue indépendante du consensus et du code.

Portefeuille, explorateur et site de présentation sont une étape ultérieure, hors de ce travail sur le mécanisme blockchain.

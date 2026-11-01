# ARETOVUS — spécification expérimentale v0

**Version de travail :** 0.1-expérimental. **Branche :** `feature/emission-schedule-v0`. **Usage :** expériences locales/testnet, aucun fonds réel. Les règles sont exécutables et couvertes par CI, mais le protocole n’a pas reçu de revue indépendante et n’est pas prêt pour un mainnet.

Ce document décrit l’implémentation actuelle. Les choix actifs sont dans [TESTNET_DECISIONS.md](TESTNET_DECISIONS.md); `DECISION_REGISTER.md` conserve les hypothèses historiques et questions de recherche. Le développement expérimental continue sans revue externe immédiate, sans garantie de sécurité.

## Profil transaction/état

Le passage au profil Testnet v3 est incompatible avec l’ancien testnet hybride. Il faut lancer les nœuds v3 avec un nouveau dossier `--data-dir`; conserver les snapshots antérieurs au cas où une comparaison ou une migration outil serait nécessaire. Aucune migration automatique des soldes ou des identifiants n’est définie. Ce profil remplace l’algorithme de signature des transactions, mais SHA-256 reste utilisé pour les hashes et le PoW: le protocole entier n’est donc pas qualifié de post-quantique.

- Le testnet accepte uniquement les transactions v3: clé publique ML-DSA-65 de 1 952 octets et signature ML-DSA-65 de 3 309 octets; `sender` est le hash domaine-séparé de la clé publique. Le digest signé engage la version, le réseau, l’expéditeur, le nonce, le destinataire, le montant et les frais. Les profils v1 Ed25519 et v2 hybride sont conservés pour Regtest et rejetés par consensus sur Testnet. Le handshake P2P distingue le nouveau profil.
- Les montants sont des entiers non signés à 8 décimales. Encodage réseau big-endian et décodage canonique; formats incorrects, données excédentaires et dépassements sont rejetés.
- Une transaction débite montant + frais, exige la signature exacte et le prochain nonce. Le lot de transactions est appliqué atomiquement.
- Le bloc engage la liste ordonnée des identifiants de transactions par une racine Merkle domaine-séparée (feuille, nœud et arbre vide ont des domaines distincts); pour un niveau impair, le dernier nœud est dupliqué. Les preuves bornées incluent l’index et le nombre de feuilles. La racine est incluse dans l’en-tête canonique version 1.

## Entête, PoW et sélection

L’en-tête version 1 porte l’identifiant réseau, le hash parent, l’engagement du bloc, la racine Merkle, l’horodatage et le nonce. Le changement casse la compatibilité avec les anciennes entêtes. Le hash SHA-256 de l’encodage canonique doit commencer par le nombre requis de bits nuls. Le bloc doit avoir un horodatage strictement supérieur à la médiane des onze ancêtres disponibles.

À chaque hauteur multiple de l’intervalle, le retarget mesure `timestamp_candidat - timestamp_ancêtre(H - intervalle)`, avec un minimum de 1 seconde. Ce delta couvre les espacements de blocs visés et inclut le candidat. Chaque timestamp est strictement supérieur à celui du parent et au temps médian des 11 ancêtres. La relecture des blocs reste déterministe et ne dépend pas de l’horloge locale. Pour l’admission sur le testnet, `serve`, `sync` et `mine` refusent un bloc dont l’horodatage dépasse l’heure locale de plus de deux heures. Un bloc ainsi refusé peut être réessayé quand l’horloge le rattrape; la machine doit donc avoir une heure correctement synchronisée. Cette borne réduit le saut temporel exploitable mais ne supprime pas toute manipulation de difficulté à l’intérieur de la fenêtre. L’ajustement reste borné par fenêtre et par difficulté min/max.

## Émission et frais

Le testnet applique l’émission `proportional-remaining-1e6-v1` : en unités minimales, `R = min(cap - issued, max(1, floor((cap - issued) / 1_000_000)))`. Le plafond est 500 000 000 AUR et la genèse vide ne crée aucun jeton. Le premier bloc émet 500 AUR; ensuite l’émission décroît proportionnellement au reliquat, avec arrondis entiers et un plancher d’une unité.

Cinq pour cent de l’émission (arrondi inférieur en unité minimale) vont au compte trésorerie testnet `[0xA0; 32]`; le mineur reçoit le reste. La moitié des frais (arrondi inférieur) va au mineur, le reliquat des frais est brûlé. L’offre brute ne dépasse pas le plafond. Les récompenses sont incluses dans l’état par branche et recalculées lors d’une réorganisation. Les chaînes génériques désactivent les récompenses par défaut. Une fois le plafond atteint, l’émission est nulle et le revenu du mineur vient des frais; le partage de frais reste 50 % mineur / 50 % brûlé.

## Configuration testnet et persistance

Le nœud construit une genèse testnet vide et déterministe. Paramètres actuels : 8 bits initiaux, cible 60 secondes, fenêtre de 10 blocs, ajustement maximal de 2 bits par fenêtre, bornes 4 à 20 bits. La configuration persistée et une empreinte de protocole distinguent ce testnet des configurations incompatibles.

Le stockage v6 de cette branche écrit les blocs acceptés une fois dans `blocks-v5/`, sous leur hash. Il écrit leur ordre d’arrivée dans des pages immuables checksummées dans `index-pages-v6/`, puis publie atomiquement un manifeste `AURORIDX` engageant les pages et les paramètres de consensus. Les trois derniers manifestes sont conservés. Le chargeur lit aussi les index v5 et les snapshots `AURORSNP` v4; il vérifie les hashes, rejoue tous les blocs et choisit la chaîne valide au travail cumulé maximal. La version v6 a passé la [CI Rust et multi-nœuds](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/40485261614); aucun essai de volume ou de panne matérielle n’en découle. Une sauvegarde doit copier le dossier entier, notamment les deux sous-dossiers et les manifestes. Voir [TESTNET_OPERATIONS.md](TESTNET_OPERATIONS.md).

## Réseau expérimental

Le transport actuel est TCP push/pull vers des pairs configurés. Le fingerprint P2P `native-ml-dsa-65-merkle-header/v4` distingue l’inventaire de blocs u64 et la reprise de session du profil `/v3`; les pairs de versions différentes refusent le handshake. Les règles de bloc restent identiques; le stockage v6 est une migration de format. Sauvegarder le dossier complet avant toute mise à niveau. Le mempool RPC et le wallet sont disponibles sur Testnet seulement : handshake de fingerprint, frames à longueur préfixée, taille maximale de 4 MiB, blocs consensus au plus à 4 MiB, pages de 64 blocs, total et curseur de blocs u64, 256 MiB de trafic cumulé par session, 16 384 requêtes maximum, durée de session de 15 minutes et délai d’E/S de 10 secondes. Le serveur peut annoncer jusqu’à 32 adresses IP statiques via `GET_PEERS`; la commande `sync --discover` les explore en un seul saut, avec au plus 64 pairs contactés. La découverte est opt-in et n’utilise pas DNS. Chaque pair est soumis au handshake de fingerprint, et chaque bloc reçu est revalidé par le consensus. Le message `GET_BLOCKS` contient un curseur u64 big-endian; `BLOCKS_REPLY` contient un total u64, un compte u32 puis des blocs canoniques à longueur u32. Le client rouvre une session avant ses quotas et vérifie le hash au dernier curseur; un inventaire décroissant est rejeté. Il synchronise les blocs en ordre d’arrivée et n’enregistre que ceux que le consensus accepte; synchronisation et propagation réessaient jusqu’à trois fois les erreurs de connexion transitoires, avec délai croissant. Les erreurs de protocole et consensus ne sont pas répétées. `serve` répond aux pages, accepte les blocs validés et interroge périodiquement chaque `--peer` configuré (intervalle de 10 secondes après un tour); les branches téléchargées sont fusionnées avec les branches locales avant sauvegarde. `sync` récupère l’état annoncé une fois auprès de chaque `--peer` fourni, en séquence; `mine --peer` transmet les blocs fraîchement minés aux pairs indiqués.

La découverte est opt-in, limitée à un saut depuis des bootstrap peers statiques. Le serveur possède huit workers et limite à quatre les sessions simultanées issues d’une même adresse IP; le handshake dispose de deux secondes, les échanges établis de dix secondes par trame. Cette limitation atténue la saturation par une seule source mais ne garantit pas la disponibilité face à plusieurs adresses hostiles. La lecture et l’écriture d’une trame entière sont chacune bornées par un délai total, même si le pair avance au compte-gouttes. Les blocs d’une page sont validés dans une copie de la chaîne; cette page n’est publiée en mémoire qu’après sauvegarde réussie. La reconnexion périodique ne concerne que les pairs explicitement configurés; il n’y a pas d’annonce spontanée multi-sauts. À chaque tour, le serveur télécharge aussi les pages de transactions en attente de ces pairs et n’accepte que celles qui passent ses propres contrôles de signature, nonce, solde et quotas. Cette propagation par interrogation est dirigée par la configuration des pairs, sans diffusion spontanée; le mempool n’est pas persisté. Une smoke test multi-processus est exécutée en CI : elle vérifie push, rechargement après arrêt des pairs, pull vers un stockage neuf et égalité de sommet. Ces contrôles ciblés ne remplacent pas les tests longue durée ni une défense éprouvée contre les attaques DoS. Garder cette testnet sur un réseau contrôlé.

## Portefeuille et RPC testnet

Le CLI `aretovus-wallet` ne contacte que le protocole wallet du testnet ML-DSA v3. Chaque échange utilise le framing TCP existant : longueur big-endian u32 puis charge utile, limite de 4 MiB, délai d’E/S de 10 secondes. Le client commence par le message d’un octet `WALLET_HELLO=5`; le serveur répond `0x85 || network || fingerprint[32] || height[u64] || tip[32]`. Le client exige le réseau Testnet et compare le fingerprint reçu au profil P2P v4 et aux paramètres de consensus testnet épinglés dans `aretovus-consensus`. Un nœud utilisant d’autres paramètres est refusé avant toute requête de solde ou d’envoi. Le client SPV applique aussi la borne locale de deux heures aux en-têtes reçus; ce contrôle d’admission ne modifie pas le consensus.

Les commandes utilisent les messages suivants :

| Message | Charge après l’opcode | Réponse |
|---|---|---|
| `GET_ACCOUNT=6` | AccountId[32] | `0x86 || next_nonce[u64] || balance_units[u64]` |
| `SUBMIT_TX=7` | Transaction v3 canonique | `0x87 || txid[32] || pending_count[u32]`, ou `0x89 || reason` |
| `GET_MEMPOOL=8` | offset[u32] | `0x88 || count[u16] || (length[u32] || tx)*` |
| `GET_TX_STATUS=9` | txid[32] | `0x8a || status` : 0 inconnu, 1 pending, 2 inclus dans la chaîne canonique |

Le compte et le nonce sont calculés en rejouant les transactions actuellement en attente sur l’état confirmé. L’admission au mempool exige le réseau et la version v3, l’encodage canonique, une signature valide, le prochain nonce et un solde suffisant, après application des transactions déjà en attente. Le mempool est borné à 4 096 transactions et 32 MiB; une page contient au plus 700 transactions et le client avance l’offset jusqu’à une page vide, avec au plus 16 pages par consultation. Il est en mémoire et ne survit pas à un redémarrage. Les serveurs interrogent périodiquement leurs pairs configurés et valident les transactions reçues avant admission. Le mineur récupère les transactions du premier pair `--peer`, vérifie les règles de consensus en construisant le bloc, puis propage le bloc. Le protocole n’a pas de diffusion spontanée, de persistance du mempool ni de sélection économique des frais.

Les commandes `aretovus-wallet verify --txid <hex> --confirmations N` téléchargent des pages d’en-têtes depuis la genèse testnet épinglée, vérifient le PoW, les horodatages, les retargets et le travail cumulé, puis demandent une preuve d’inclusion Merkle au nœud. Le client vérifie que l’en-tête prouvant la transaction est canonique dans la chaîne téléchargée et respecte le nombre minimal de confirmations demandé. C’est une vérification légère dépendante de la vue des pairs; elle ne prouve pas à elle seule que le réseau n’a pas été éclipsé ou censuré.

Le module `aretovus_wallet::uri` encode les paiements comme `aurora://testnet/<adresse-hex>?amount=<AUR>&memo=<texte-percent-encodé>`. Le parseur refuse réseau, paramètres, montants, encodages et mémo invalides. Le montant et le mémo sont optionnels. L’URI ne transporte aucune preuve d’identité ni signature de demande.

Les nouveaux fichiers de clés wallet `AURKEY4` contiennent une graine de 32 octets générée par le CSPRNG du système, chiffrée avec XChaCha20-Poly1305 et une clé dérivée du mot de passe par Argon2id (64 MiB, trois passes). La commande `backup` vérifie la source puis crée un fichier chiffré indépendant avec sel et nonce nouveaux; `restore` vérifie la sauvegarde et recrée une clé sans écraser un fichier existant. L’ancien `AURKEY3` en clair n’est conservé que pour les essais automatisés. L’adresse actuelle est l’identifiant de 32 octets dérivé de la clé publique ML-DSA-65; son affichage est 64 caractères hexadécimaux. Il n’y a pas encore de dérivation hiérarchique, de phrase de récupération, de gardiens/time-locks, de DID intégré aux comptes ou de protection matérielle des clés.

`Confirmed` signifie uniquement que le txid est observé dans les blocs canoniques de ce nœud au moment de la requête; aucune règle de confirmations ou de finalité n’est promise. En cas de réorganisation, une transaction peut perdre cette inclusion. Le portefeuille doit alors réinterroger le statut.

## Commandes

```bash
cargo run -p aretovus-node -- testnet mine --data-dir ./node-a --miner <64 caractères hex> --count 10
cargo run -p aretovus-node -- testnet serve --data-dir ./node-a --listen 0.0.0.0:18444 --peer 192.0.2.20:18444
cargo run -p aretovus-node -- testnet sync --data-dir ./node-b --peer 127.0.0.1:18444 --discover
```

## État de préparation

Le réseau expérimental fonctionne pour une testnet supervisée à pairs connus; le push/pull, la reprise du snapshot et la convergence de nœuds distincts passent le scénario multi-processus de CI. Ce jalon ne signifie pas que le réseau est décentralisé, toujours connecté ou résistant aux attaques.

Avant d’élargir les essais, restent à faire :
1. Découverte et annonces multi-sauts spontanées, si l’usage les exige; le profil actuel garde des adresses statiques et un rattrapage périodique.
2. Scénarios prolongés de coupure/reprise, inventaires menteurs, connexions lentes et saturation concurrente.
3. Simulation indépendante du retarget avec dérive d’horloge, longues pauses, rafales et réorganisations.
4. Essais avec disque réellement plein, panne électrique et sauvegardes hors machine malgré les tests de reprise, copie de sauvegarde et migrations v4→v5 déjà réalisées; la migration vers v6 a passé la CI, mais pas les essais de panne réelle.
5. Campagnes fuzz prolongées et revue indépendante du consensus et du code avant toute prétention de sécurité pour des fonds réels.

Le CLI wallet est expérimental et son essai multi-processus est couvert par le smoke test. DID et récupération restent au stade de conception (`docs/DID_RECOVERY_DESIGN.md`) et n’altèrent pas l’état consensus. L’explorateur, le site de présentation et la vente sont hors de ce jalon.

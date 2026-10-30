# AURORA — protocole d'essai sur deux PC Windows (réseau local privé)

**Statut : préparation d'un essai réel, pas une preuve d'essai déjà effectué.**
Aucun fonds réel, aucun port redirigé sur la box, aucun Internet public.
Ce document concerne le testnet natif ML-DSA-65 et son protocole P2P v4.

## 0. Prérequis bloquants

- Deux PC Windows **64 bits x86-64**, chacun relié au **même réseau privé de confiance**. Si un PC est 32 bits ou ARM, les archives Windows x86-64 ne sont pas utilisables telles quelles. Le Wi-Fi invité peut isoler les appareils.
- Les **deux mêmes binaires** (node et wallet) construits depuis le **même commit** de `main`. Pour la version de test la plus récente, prendre l'artefact Windows du [workflow `testnet-packages`](https://github.com/antoineforterax-cmd/AURORA/actions/workflows/testnet-packages.yml) sur le **commit `main` vert**, et **non** l'ancienne release 421d443. Ces artefacts expirent après 14 jours et contiennent parfois une ZIP imbriquée. La CI du workflow doit être **verte** (Windows, Linux, macOS) avant de les utiliser.
- Dans chaque dossier extrait, on doit trouver `bin/aurora-node.exe`, `bin/aurora-wallet.exe`, `EXPERIMENTAL_TESTNET.txt` (SHA source), `SHA256SUMS.txt`, `scripts/two_pc_preflight.ps1`. Vérifier le SHA-256 de l'archive téléchargée avec une valeur fournie par le workflow ou une source contrôlée; le fichier interne vérifie les fichiers extraits, mais **n'authentifie pas à lui seul l'éditeur**.
- Deux dossiers de données **neufs et indépendants** par machine ; ne réutiliser aucun dossier de vieux testnet. Lancer tous les exemples depuis la racine extraite (le dossier où se trouvent `bin` et `scripts`). Ne jamais partager fichiers `.aurkey`, mots de passe ou graines.
- Noter le SHA complet du workflow, l'heure/date et les versions Windows. Ne pas désactiver Microsoft Defender/SmartScreen ni le pare-feu pour lancer un exécutable non signé.

**PC A** = ordinateur principal, serveur TCP 19444, mineur séparé, portefeuille Alice.
**PC B** = second ordinateur, serveur local 19445, portefeuille Bob.
Remplacer dans les commandes `192.168.1.20` par **l'IPv4 privée réellement affichée sur PC A** (`ipconfig`), différente de `127.0.0.1`. Ne jamais recopier les IP privées dans une issue publique.

## 1. Préflight et réseau (avant de créer des clés)

Sur chaque PC, depuis la racine de l'archive extraite :

```powershell
Get-Content .\EXPERIMENTAL_TESTNET.txt
.\scripts\two_pc_preflight.ps1 -PackageRoot . -Offline
Get-FileHash .\bin\aurora-node.exe -Algorithm SHA256
Get-FileHash .\bin\aurora-wallet.exe -Algorithm SHA256
```

Les deux PC doivent afficher le **même commit** dans `EXPERIMENTAL_TESTNET.txt`, et pour chaque binaire les **mêmes SHA-256**. Le script recalcule chaque somme interne et vérifie le démarrage des deux exécutables avec `--help`. Si une vérification échoue : **stop**, ne pas essayer de connecter les machines.

Sur PC A, dans PowerShell : `ipconfig`. Repérer l'IPv4 de l'interface Wi-Fi/Ethernet réellement connectée. Le réseau Windows doit être *Privé* (et non Public). Ne **pas** ajouter de port forwarding à la box.

La règle de pare-feu éventuelle doit autoriser **uniquement TCP 19444 entrant sur PC A**, **uniquement profil Privé**, avec adresse distante limitée à l'IPv4 de PC B. Ne jamais ouvrir 19444 sur tous les profils ou sur Internet. Si l'utilisateur n'est pas administrateur, faire effectuer cette règle par l'administrateur autorisé, sans désactiver le pare-feu. PC B n'a **aucune** règle entrante à ouvrir pour le parcours initial ; ses connexions vers A sont sortantes.

## 2. PC A — lancer le serveur (terminal laissé ouvert)

```powershell
.\bin\aurora-node.exe testnet serve --data-dir .\two-pc\server-a --listen 0.0.0.0:19444
```

Ne pas utiliser `server-a` dans une autre commande `mine` ou `sync` pendant que ce serveur tourne : le verrou de dossier protège ses écritures.

Sur PC B, une fois PC A démarré :

```powershell
.\scripts\two_pc_preflight.ps1 -PackageRoot . -PeerAddress 192.168.1.20 -PeerPort 19444
Test-NetConnection 192.168.1.20 -Port 19444
```

Attendu : `TcpTestSucceeded : True`. S'il est `False`, vérifier adresse, Wi-Fi isolé et pare-feu **privé** avant de poursuivre. Ne pas lancer le pair B tant que la connexion TCP n'est pas validée.

## 3. PC B — lancer un pair indépendant (terminal laissé ouvert)

```powershell
.\bin\aurora-node.exe testnet serve --data-dir .\two-pc\server-b --listen 127.0.0.1:19445 --peer 192.168.1.20:19444
```

Cette configuration suffit à tester le **rattrapage B vers A**. Elle n'est pas un maillage bidirectionnel : seule B interroge périodiquement A et récupère les transactions du mempool de A. Pour tester les deux directions plus tard, prévoir un second port privé 19445, lancer B sur son IPv4 LAN et configurer A avec `--peer IP_B:19445` après un redémarrage propre. Ne pas activer cette étape tant que le test de base n'est pas vert.

## 4. Wallets et un premier bloc de test (terminaux supplémentaires)

Sur **PC A**, créer une clé Alice chiffrée. Conserver uniquement l'adresse publique dans `$alice` :

```powershell
New-Item -ItemType Directory -Path .\two-pc\cles -Force
.\bin\aurora-wallet.exe keygen --out .\two-pc\cles\alice.aurkey
$alice = .\bin\aurora-wallet.exe address --key-file .\two-pc\cles\alice.aurkey
.\bin\aurora-node.exe testnet mine --data-dir .\two-pc\miner-a --miner $alice --count 1 --peer 127.0.0.1:19444
```

Le mineur a son **propre** dossier `miner-a`, séparé de `server-a`. Un bloc testnet neuf sans transaction émet une récompense testnet ; le portefeuille est uniquement un outil de test.

Sur **PC B**, créer Bob et noter **son adresse publique** :

```powershell
New-Item -ItemType Directory -Path .\two-pc\cles -Force
.\bin\aurora-wallet.exe keygen --out .\two-pc\cles\bob.aurkey
$bob = .\bin\aurora-wallet.exe address --key-file .\two-pc\cles\bob.aurkey
```

Transmettre **uniquement la chaîne publique `$bob`** à PC A, sans copier le fichier de clé. PC B peut consulter son propre pair :

```powershell
.\bin\aurora-wallet.exe balance --key-file .\two-pc\cles\bob.aurkey --node 127.0.0.1:19445
```

## 5. Paiement inter-PC et confirmation

Sur PC A, remplacer la valeur de `$bob` par l'adresse publique exacte donnée par PC B, puis envoyer un petit montant **fictif** :

```powershell
$bob = 'COLLER_ICI_L_ADRESSE_PUBLIQUE_DE_BOB_64_HEX'
.\bin\aurora-wallet.exe send --key-file .\two-pc\cles\alice.aurkey --to $bob --amount 1.25 --fee 0.01 --node 127.0.0.1:19444
```

Copier le `transaction_id` renvoyé (public), puis sur PC A miner un bloc afin de confirmer le transfert :

```powershell
.\bin\aurora-node.exe testnet mine --data-dir .\two-pc\miner-a --miner $alice --count 1 --peer 127.0.0.1:19444
$txid = 'COLLER_ICI_LE_TRANSACTION_ID_64_HEX'
.\bin\aurora-wallet.exe verify --node 127.0.0.1:19444 --txid $txid --confirmations 1
```

Sur PC B, après rattrapage (le poll se fait périodiquement, pas instantanément) :

```powershell
.\bin\aurora-wallet.exe balance --key-file .\two-pc\cles\bob.aurkey --node 127.0.0.1:19445
.\bin\aurora-wallet.exe verify --node 127.0.0.1:19445 --txid $txid --confirmations 1
```

Pour la seconde commande, **recopier seulement** l'identifiant public `$txid` sur PC B. Il faut constater la même inclusion et preuve, pas seulement un solde projeté contenant un paiement en attente.

## 6. Coupure, redémarrage et cohérence des copies

1. Arrêter **uniquement B** avec `Ctrl+C` (laisser A serveur en marche). Miner **deux blocs** supplémentaires depuis `miner-a` sur A.
2. Redémarrer sur B la même commande `testnet serve --data-dir .\two-pc\server-b ...`. Attendre sa convergence ; relever le délai réel, le numéro du nouveau bloc et les éventuelles erreurs P2P.
3. Arrêter proprement les deux serveurs avec `Ctrl+C` avant les contrôles suivants. Après que tous les écrivains sont terminés, dans un terminal sur **chaque PC** :

```powershell
.\bin\aurora-node.exe testnet verify --data-dir .\two-pc\server-a  # PC A uniquement
.\bin\aurora-node.exe testnet verify --data-dir .\two-pc\server-b  # PC B uniquement
```

Les sorties doivent afficher des valeurs identiques de `height`, `tip` **et** `work`. Comparer aussi `known_blocks` si aucune branche latérale n'a été créée. Une même hauteur avec un hash différent est un **échec de convergence**.

4. Pour tester une reprise indépendante, copier le **dossier complet arrêté** de B vers une nouvelle destination (ne pas remplacer A), puis exécuter `testnet verify --data-dir NOUVEAU_DOSSIER`. Le snapshot seul ne suffit pas : conserver `blocks-v5/`, `index-pages-v6/` et les manifestes. Ne pas inclure les clés wallet dans les journaux partagés.
5. Garder un rapport sans secrets contenant : **SHA complet**, SHA256 de chaque binaire, OS/architecture des PC, date/heure, connexion TCP initiale, `height/tip/work/known_blocks` sur A et B, identifiant public du paiement et état SPV, durée de rattrapage, redémarrages, erreurs et consommation disque approximative. Masquer les IPv4, noms d'utilisateur et chemins personnels avant publication.

## Critères de réussite et arrêt

**Réussi pour ce premier essai :** même build et mêmes hashes sur les deux PC ; connectivité TCP privée validée ; propagation de blocs A vers B ; paiement signé validé et confirmé sur les deux pairs ; arrêt/reprise de B sans divergence ; même `height/tip/work` après arrêt ; copie complète restaurée avec le même sommet.

**Échec / stop immédiatement :** archive ou checksum divergents, connexion TCP impossible, refus de fingerprint, perte de snapshot, `height` identique mais `tip` divergent après rattrapage, preuve d'inclusion incohérente, nouvelle règle de pare-feu publique ou un dossier de données utilisé par deux processus.

Cela ne démontre **ni** un mainnet sécurisé, **ni** la stabilité sur 72 h, **ni** 100 000 blocs synchronisés entre machines, **ni** une résistance générale aux pairs malveillants. Ces campagnes demandent des rapports séparés.

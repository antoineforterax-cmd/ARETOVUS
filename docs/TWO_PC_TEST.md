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


## Assistant Windows : connexion automatique sur LAN privé (version expérimentale)

Les nouvelles archives du testnet construites avec cette fonctionnalité incluent **AURORA-LAN.cmd**, une interface Windows sans installation Python ni droit administrateur. Cet assistant **ne remplace pas les vérifications finales de ce protocole** et ne prétend pas réaliser un essai complet sur deux ordinateurs.

1. Extraire sur chaque ordinateur **le même paquet Windows**, vérifier la provenance et exécuter le préflight normal (checksums et SHA identiques). Les checksums dans un ZIP non signé ne prouvent pas, à eux seuls, l'identité de l'éditeur.
2. Sur un **LAN privé de confiance**, double-cliquer AURORA-LAN.cmd sur chaque PC. Il exige un profil Windows **Privé**, sans désactiver le pare-feu.
3. Sur PC A, choisir **« PC A : démarrer le serveur »**. Le programme lance le serveur aurora-node testnet dans sa propre fenêtre, sur TCP **19444** ; il émet une annonce de découverte UDP **19446** sur le réseau local.
4. Sur PC B, cliquer **« PC B : connexion automatique »**. L'assistant écoute l'annonce, compare le hash **annoncé** du binaire A avec son propre binaire, vérifie la disponibilité TCP, puis lance le nœud B sur son adresse loopback avec le pair IP_A:19444. Si la découverte UDP est bloquée, saisir simplement **l'IPv4 privée de PC A** et utiliser le bouton manuel.
5. Si le pare-feu refuse TCP 19444, l'administrateur autorisé doit créer une **règle entrante restreinte au profil Privé et à l'IP de PC B**. Le programme n'ouvre **aucune règle**, ne configure pas de redirection NAT et ne fonctionne pas sur Internet/public Wi-Fi.
6. **Arrêt sûr :** utiliser **Ctrl+C** dans la fenêtre de chaque nœud, attendre sa fermeture, puis effectuer les tests prévus plus haut : transfert entre PC, redémarrage, convergence vérifiée avec testnet verify sur les dossiers arrêtés (height, tip, work), sauvegarde. Le mineur utilise toujours un dossier différent.

**Sécurité :** une annonce UDP n'est pas authentifiée ; un autre appareil du LAN pourrait mentir sur sa version ou son hash. N'utiliser l'assistant que sur un réseau privé fiable et comparer les deux packages par une source de confiance. Les dossiers two-pc/assistant-a et two-pc/assistant-b sont distincts et respectent les verrous de stockage. Ni clés privées, ni mots de passe, ni fonds réels ne sont utilisés par le guide automatique.

**Limites :** le lanceur ne mine pas en continu, ne crée pas de wallet, ne confirme pas automatiquement les transactions et ne prouve pas, à lui seul, la convergence de deux PC physiques. Les assistants GUI plus avancés restent un chantier séparé.


## Centre local AURORA sur macOS et Linux (développement expérimental)

Une future archive construite depuis **la même branche et le même commit** contient désormais :

- **macOS :** `AURORA-LAN.command` ; lancer depuis un Terminal via `sh AURORA-LAN.command` si le système ne traite pas directement le fichier comme exécutable.
- **Linux :** `AURORA-LAN.sh` ; lancer avec `sh AURORA-LAN.sh`.
- **Windows :** `AURORA-LAN.cmd` ; lancer dans un dossier Windows extrait comme décrit précédemment.

Sur macOS et Linux, il faut **Python 3.10 ou supérieur**. L'assistant utilise la bibliothèque standard Python, et **Tkinter quand il est disponible** pour une fenêtre graphique ; sinon, il passe en **mode terminal**. La présence de Python/Tkinter ne découle pas du paquet AURORA : vérifier cette dépendance séparément. Certains extracteurs ZIP perdent le bit d'exécution des binaires Unix : l'assistant vérifie **d'abord** les hashes du paquet, puis restaure uniquement la permission exécutable sur ses deux binaires locaux, sans privilèges administrateur ni changement de contenu. Sur macOS, un fichier téléchargé non signé peut déclencher les protections de Gatekeeper ; ne pas les désactiver globalement. Sous Linux, ne pas désactiver les protections du système ni lancer l'assistant en root.

### Parcours à deux machines

1. Télécharger depuis le **même commit Git**, utiliser sur chaque ordinateur la **bonne archive pour son OS et son architecture**, et comparer le commit source avec une source de confiance. Ne jamais distribuer de clés privées ou de mots de passe.
2. Relier les PC au **même LAN privé de confiance**. L'assistant identifie une seule IPv4 privée active ; en présence de plusieurs interfaces ambiguës, il refuse de lancer le nœud. Les réseaux VPN, isolés, invités ou les pare-feu peuvent bloquer la découverte.
3. Choisir **PC A : serveur**, puis sur l'autre ordinateur **PC B : rechercher et se connecter**. Les assistants partagent une annonce UDP **19446** portant le **commit source**, qui permet désormais de rechercher un pair même entre **Windows, macOS et Linux**. La vérification TCP du port **19444** précède le lancement de B.
4. Si la découverte est bloquée, saisir **l'IPv4 privée** de PC A. Ne jamais ouvrir de port sur le routeur ou désactiver le pare-feu. Une autorisation locale TCP 19444, limitée à l'adresse privée de PC B, peut être requise.
5. Pour arrêter sur macOS/Linux, utiliser **Arrêter mon nœud** (interruption progressive) ou **Ctrl+C** en mode terminal. Comparer ensuite les sorties `testnet verify` des dossiers `two-pc/assistant-a` et `two-pc/assistant-b` **à l'arrêt** : même `height`, `tip`, `work`. Continuer les transferts, restaurations et redémarrages décrits dans le protocole ci-dessus.

### Outils utiles dès maintenant et capacités futures

| Fonction | État sur cette branche |
| --- | --- |
| Vérification des hashes SHA-256 de **tous les fichiers** du package sous macOS/Linux | **Implémentée** (checksum local, pas signature d'éditeur) |
| Détection d'un pair A et lancement d'un nœud B par son IPv4 privée | **Implémentés** (annonce non authentifiée ; pas de preuve de convergence) |
| Arrêt progressif du processus lancé par l'assistant Unix | **Implémenté** (ne supprime aucun verrou) |
| Export d'un diagnostic sans IPv4 privée, nom de compte ou secret | **Implémenté** (`two-pc/assistant-report.txt`) |
| Minage continu, allocation CPU, rendement, suivi temps réel et dashboard | **Prévu, non disponible** |
| Portefeuille graphique, sauvegarde/restauration guidée, état multi-hôte | **Prévu, non disponible** |
| Rapport final de convergence à deux machines et essai 72 heures | **Tests séparés à effectuer**, aucune preuve déduite du lanceur |

Le **protocole d'annonces v2** indique uniquement une révision source commune. Il **n'authentifie pas le pair distant** : un attaquant du LAN peut copier une annonce ou un commit. Les exécutables sur chaque ordinateur doivent être obtenus et vérifiés depuis une source indépendante et fiable ; le protocole P2P continue de valider la chaîne. Ne pas utiliser un réseau Wi-Fi public. Toute ouverture de port pour des machines distantes ou Internet est hors périmètre.

Les assistants restent des outils de **testnet expérimental**. Ils ne sont pas des services de minage public, des wallets sécurisés pour fonds réels, ni des nœuds mainnet validés.

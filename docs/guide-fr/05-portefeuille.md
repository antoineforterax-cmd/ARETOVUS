# 5. Portefeuille, paiements et sauvegarde des clés

[Sommaire](README.md) · [← Réseau et minage](04-reseau-et-minage.md) · [Exploitation →](06-exploitation.md)

Cette page continue le scénario précédent : Alice a reçu une récompense, le nœud A écoute sur `127.0.0.1:19444` et le mineur utilise son propre dossier. Toutes les opérations concernent le testnet.

## Créer un destinataire

Dans un terminal libre, depuis le dossier extrait :

```powershell
.\bin\aurora-wallet.exe keygen --out .\demo\cles\bob.aurkey
$bob = .\bin\aurora-wallet.exe address --key-file .\demo\cles\bob.aurkey
```

Bob possède une clé différente d’Alice. Une adresse est publique et peut être transmise à l’expéditeur. Un mot de passe ou un fichier de clé ne doit pas lui être transmis.

## Lire le solde et envoyer

```powershell
.\bin\aurora-wallet.exe balance --key-file .\demo\cles\alice.aurkey --node 127.0.0.1:19444
.\bin\aurora-wallet.exe send --key-file .\demo\cles\alice.aurkey --to $bob --amount 1.25 --fee 0.01 --node 127.0.0.1:19444
```

Les montants de la commande utilisent un point, sans séparateur de milliers. Le transfert débite 1,26 AUR à Alice : 1,25 AUR pour Bob et 0,01 AUR de frais. Le wallet récupère le nonce auprès du nœud, signe avec ML-DSA-65 puis transmet la transaction. Ne modifiez pas vous-même le nonce dans ce parcours.

La réponse `GET_ACCOUNT`, utilisée par `balance` et `send`, part de l’état de la chaîne du nœud et y rejoue les transactions de son mempool. Le solde et `next_nonce` sont donc projetés avec les transactions en attente de **ce nœud**. Ils peuvent changer après un redémarrage, une inclusion ou une réorganisation; cette réponse n’est pas une preuve de solde confirmé.

La branche de développement affiche désormais ce solde sous le nom `pending_adjusted_balance`. La Release de référence `421d443` et les anciennes archives affichent encore le libellé trompeur `confirmed_balance` pour la même réponse incluant le mempool. Ne déduisez pas une confirmation de ce seul libellé : utilisez le statut de transaction et sa preuve d’inclusion après minage.

La sortie donne un `transaction_id`. Copiez cet identifiant dans une variable, en remplaçant le texte entre guillemets :

```powershell
$txid = 'coller_ici_les_64_caracteres_du_transaction_id'
.\bin\aurora-wallet.exe status --node 127.0.0.1:19444 --txid $txid
```

`Pending` signifie que le nœud conserve le transfert en attente. Cela ne signifie pas qu’il est déjà dans un bloc. Le mempool n’est pas sauvegardé sur disque; un redémarrage peut faire disparaître cette attente.

## Miner et vérifier l’inclusion

Dans le terminal du mineur, où `$alice` contient son adresse :

```powershell
.\bin\aurora-node.exe testnet mine --data-dir .\demo\mineur --miner $alice --count 1 --peer 127.0.0.1:19444
```

Le mineur prend les transactions du premier `--peer` fourni. Revenez ensuite au terminal du wallet :

```powershell
.\bin\aurora-wallet.exe status --node 127.0.0.1:19444 --txid $txid
.\bin\aurora-wallet.exe verify --node 127.0.0.1:19444 --txid $txid --confirmations 1
.\bin\aurora-wallet.exe balance --key-file .\demo\cles\bob.aurkey --node 127.0.0.1:19444
```

Bob doit voir 1,25 AUR dans cet exemple neuf. `verify` contrôle la chaîne d’en-têtes et une preuve d’inclusion Merkle. Cette vérification légère dépend des données du pair interrogé; elle ne prouve pas l’absence d’éclipse ou de censure.

`Confirmed` peut redevenir un autre état après une réorganisation. Demander davantage de confirmations exige de miner d’autres blocs; cela ne crée pas une finalité absolue.

Une commande d’attente est aussi disponible :

```powershell
.\bin\aurora-wallet.exe wait --node 127.0.0.1:19444 --txid $txid --timeout-seconds 120
```

Elle attend l’inclusion mais ne mine rien. Un mineur doit donc travailler dans un autre terminal, sinon l’attente peut expirer normalement.

## Comprendre le fichier de clé

La création ordinaire produit un fichier chiffré `AURKEY4`. Une clé est dérivée du mot de passe par Argon2id et sert à chiffrer la graine ML-DSA-65 avec XChaCha20-Poly1305. Cela protège le fichier au repos, sans garantir la sécurité d’un PC compromis.

Le portefeuille n’utilise pas de phrase de récupération de douze ou vingt-quatre mots. Il faut conserver **le fichier et le mot de passe**. Il n’existe ni bouton de réinitialisation, ni gardien de récupération actif. L’option `--plain-testnet` crée volontairement une clé en clair pour des automatismes de test; ne l’utilisez pas pour ce parcours manuel.

## Sauvegarder et essayer la restauration

```powershell
.\bin\aurora-wallet.exe backup --key-file .\demo\cles\alice.aurkey --out .\demo\cles\alice-sauvegarde.aurkey
.\bin\aurora-wallet.exe restore --from .\demo\cles\alice-sauvegarde.aurkey --out .\demo\cles\alice-restauree.aurkey
.\bin\aurora-wallet.exe address --key-file .\demo\cles\alice-restauree.aurkey
```

L’adresse restaurée doit être exactement celle d’Alice. Le fichier est rechiffré avec de nouveaux paramètres aléatoires, tout en conservant la même graine et le même mot de passe. Les commandes refusent d’écraser un fichier existant.

Copiez ensuite la sauvegarde chiffrée sur un support distinct et conservez le mot de passe séparément. Une seconde copie sur le même disque sert à tester la restauration, mais ne protège pas d’une panne de ce disque. Le dossier de chaîne du nœud ne remplace pas la sauvegarde du wallet.

Les fichiers original et restauré contrôlent le même compte : n’en faites pas deux émetteurs indépendants envoyant simultanément des transactions sans coordination des nonces.

## URI de paiement

La bibliothèque prend en charge une forme telle que `aurora://testnet/<adresse>?amount=1.25&memo=essai`. Ce n’est pas encore une application mobile ni une commande autonome du CLI. Le mémo n’est pas une preuve d’identité, et une URI reçue ne dispense pas de vérifier le destinataire et le montant.

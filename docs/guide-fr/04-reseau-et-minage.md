# 4. Premier réseau local et minage

[Sommaire](README.md) · [← Installation](03-installation.md) · [Portefeuille →](05-portefeuille.md)

Ce scénario utilise un seul PC et plusieurs terminaux PowerShell. Tous les terminaux doivent partir du même dossier extrait, celui qui contient `bin`. Il garde séparés les dossiers du serveur, du mineur et du second nœud. Ne réutilisez pas les dossiers d’un essai déjà en cours.

## 1. Créer un portefeuille de test

Dans un premier terminal :

```powershell
New-Item -ItemType Directory -Path .\demo\cles
.\bin\aretovus-wallet.exe keygen --out .\demo\cles\alice.aretkey
```

Choisissez un mot de passe d’au moins douze caractères et confirmez-le. La saisie est masquée. Conservez l’adresse affichée après `address=`; elle est publique. Le fichier de clé et le mot de passe sont privés.

La commande suivante permet de retrouver l’adresse et de la mettre dans une variable PowerShell pour ce terminal :

```powershell
$alice = .\bin\aretovus-wallet.exe address --key-file .\demo\cles\alice.aretkey
```

Chaque nouvelle fenêtre possède ses propres variables. Si vous changez de fenêtre, refaites cette commande ou recopiez l’adresse publique.

## 2. Lancer le serveur A

Dans une deuxième fenêtre, depuis le dossier extrait :

```powershell
.\bin\aretovus-node.exe testnet serve --data-dir .\demo\noeud-a --listen 127.0.0.1:19444
```

Laissez cette fenêtre ouverte. `127.0.0.1` désigne votre propre PC : aucune ouverture de port sur la box n’est nécessaire pour ce scénario. Un dossier neuf reçoit la genèse de ce testnet.

## 3. Miner le premier bloc

Revenez dans le premier terminal, où `$alice` contient l’adresse :

```powershell
.\bin\aretovus-node.exe testnet mine --data-dir .\demo\mineur --miner $alice --count 1 --peer 127.0.0.1:19444
```

Le mineur synchronise sa copie depuis le serveur, récupère ses transactions en attente, construit un bloc puis l’envoie au serveur. Ici `--count 1` demande un seul bloc et la commande se termine ensuite. Elle ne lance pas un service de minage permanent.

La sortie inclut `height`, `tip`, `difficulty_bits`, `transactions`, `issued_units` et `miner_balance_units`. Les deux derniers nombres sont en unités minimales : diviser par 100 000 000 pour obtenir des ARETOVUS. Sur cette chaîne neuve, le premier bloc sans transaction donne 475 ARETOVUS au compte d’Alice.

Le dossier `demo\mineur` est différent de `demo\noeud-a`. Un serveur et un mineur ne peuvent pas écrire simultanément dans le même dossier : le nœud impose un verrou. Ne supprimez jamais `.aretovus.lock` pour contourner ce verrou.

## 4. Ajouter le nœud B

Dans une troisième fenêtre :

```powershell
.\bin\aretovus-node.exe testnet serve --data-dir .\demo\noeud-b --listen 127.0.0.1:19445 --peer 127.0.0.1:19444
```

B interroge périodiquement A et valide les blocs reçus. La convergence n’est pas instantanée. Le tour de rattrapage est suivi d’un délai de dix secondes; les opérations de synchronisation peuvent ajouter du temps.

Dans un terminal libre, vérifiez les deux copies en lecture seule :

```powershell
.\bin\aretovus-node.exe testnet verify --data-dir .\demo\noeud-a
.\bin\aretovus-node.exe testnet verify --data-dir .\demo\noeud-b
```

Attendez puis recommencez si B est encore en retard. Les valeurs `height` et `tip` doivent être identiques. Une hauteur identique avec un sommet différent signifie que les copies ne sont pas sur la même branche.

## 5. Continuer puis arrêter

Refaire la commande de minage avec le même dossier `demo\mineur` ajoute un bloc. Pour un essai représentatif de la cible, espacez les blocs d’environ une minute. La difficulté de test reste faible et ajustable; il ne s’agit pas d’un réglage destiné à assurer une sécurité économique réelle.

Arrêtez chaque serveur avec `Ctrl+C`. Les données restent dans son dossier. Relancer exactement la même commande de serveur recharge les données et reprend les échanges avec ses pairs configurés.

Pour télécharger une chaîne une seule fois, sans laisser de serveur ouvert, la commande suivante utilise encore un autre dossier :

```powershell
.\bin\aretovus-node.exe testnet sync --data-dir .\demo\copie --peer 127.0.0.1:19444
.\bin\aretovus-node.exe testnet verify --data-dir .\demo\copie
```

## Essai de 72 heures

Le [lanceur d’endurance](../ENDURANCE_RUN.md) automatise plusieurs nœuds locaux, le minage, les redémarrages et les mesures. Le script fourni dans la Release de référence mine des blocs sans transfert utilisateur. La branche de développement ajoute une campagne avec paiements : il faut identifier séparément la version du script et celle des binaires utilisés. Un essai n’est réussi qu’après lecture de ses journaux et de son résumé final, et non parce que le processus a été lancé.

Au moment de la rédaction de ce guide, un essai court de la campagne avec paiements a réussi sur le binaire `421d443` : six blocs, cinq transferts, deux redémarrages et restauration d’une copie en 82,49 secondes. Ce résultat ne valide pas 72 heures de fonctionnement. Les résultats suivants doivent être consignés dans le [rapport d’essais](../TESTNET_VALIDATION_REPORT.md).

Plusieurs processus sur un PC partagent l’alimentation, l’horloge, le système et les interfaces réseau. Ils ne prouvent pas une disponibilité multi-machines pendant 72 heures. Un second PC n’est pas nécessaire pour commencer ce parcours; sa présence ne doit pas être supposée dans les comptes rendus.

# Essai avec deux PC Windows sur le même réseau

Cet essai peut durer une heure ou quelques heures. Il complète l’endurance locale de 72 h, sans être présenté comme une endurance multi-hôtes de 72 h. Utilisez des dossiers neufs et la même archive expérimentale sur les deux PC. Les binaires Windows publiés ciblent x86-64; vérifier le type de système de l’ancien PC et le démarrage de `bin\aurora-node.exe --help` avant l’essai. Aucun portefeuille ou mot de passe existant n’est nécessaire.

Branchez les deux PC sur le même réseau de confiance. Sur le PC principal, `ipconfig` indique l’adresse IPv4 de la carte active. Remplacez `192.168.1.20` dans les exemples par cette adresse. Une adresse `127.0.0.1` ne permet pas à l’autre PC de joindre le nœud. Certains Wi-Fi invités isolent les appareils entre eux.

## PC principal : serveur

Depuis le dossier de l’archive extraite, gardez ce terminal ouvert :

```powershell
.\bin\aurora-node.exe testnet serve --data-dir .\lan-server --listen 0.0.0.0:19444
```

L’écoute sur `0.0.0.0` accepte les connexions sur les interfaces de la machine. Dans le pare-feu Windows, autorisez seulement ce programme/port TCP sur le profil privé et, si vous créez une règle, limitez les adresses distantes au sous-réseau local. Ne désactivez pas le pare-feu. Ne créez pas de redirection de port sur la box. Si Windows indique que le port est occupé, ne relancez pas un second serveur sur le même port.

## Ancien PC : pair

Vérifiez d’abord la liaison, puis gardez le serveur pair ouvert :

```powershell
Test-NetConnection 192.168.1.20 -Port 19444
.\bin\aurora-node.exe testnet serve --data-dir .\lan-peer --listen 127.0.0.1:19445 --peer 192.168.1.20:19444
```

Le pair lance lui-même ses connexions sortantes vers le PC principal. Son écoute locale n’a pas besoin d’être ouverte au réseau pour ce scénario. Le protocole est expérimental et non chiffré; ce guide reste limité à un réseau local de confiance.

## PC principal : mineur séparé

Dans un deuxième terminal, depuis le même dossier extrait :

```powershell
$testMiner = '41' * 32
.\bin\aurora-node.exe testnet mine --data-dir .\lan-miner --miner $testMiner --count 1 --peer 127.0.0.1:19444
```

L’adresse factice reçoit des récompenses de test inutilisables. Le dossier `lan-miner` est distinct de `lan-server` : deux processus écrivains ne partagent jamais un dossier. Refaire la commande après environ une minute pour ajouter un autre bloc.

Après propagation, dans un autre terminal de chaque PC, comparer :

```powershell
# Sur le PC principal
.\bin\aurora-node.exe testnet verify --data-dir .\lan-server
# Sur l’ancien PC
.\bin\aurora-node.exe testnet verify --data-dir .\lan-peer
```

Les valeurs `height` et `tip` doivent être identiques. Le passage de la commande seul ne démontre pas la convergence : conservez les deux sorties avec l’heure et le commit de l’archive.

## Coupure et rattrapage

Arrêtez le pair de l’ancien PC avec Ctrl+C, minez un ou plusieurs blocs sur le principal, puis relancez exactement la commande du pair. Mesurez le délai jusqu’au même `height`/`tip`. Vous pouvez aussi déconnecter temporairement le réseau du seul ancien PC, puis le reconnecter. Ne coupez pas le courant pour ce premier essai.

Conservez la durée réelle, versions Windows, commit, nombre de blocs, résultats de convergence et temps de rattrapage. Ne partagez ni clé privée ni mot de passe, et masquez les informations réseau personnelles avant publication. Le succès de ce scénario ne valide ni Internet public, ni connexions hostiles, ni panne matérielle.

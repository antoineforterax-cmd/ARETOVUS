# Essai d’endurance local

Pour Ubuntu/WSL, suivre [ENDURANCE_WSL.md](ENDURANCE_WSL.md). Le lanceur de la branche accepte désormais `--wallet CHEMIN` pour créer deux comptes jetables et vérifier les transferts, les soldes, les nonces et les frais. L'archive expérimentale.1 contient encore le lanceur initial de blocs vides : ne pas lui attribuer cette option. Les clés du mode avec échanges sont en clair et exclusivement réservées aux tests.

Trois nœuds peuvent fonctionner sur un seul PC. Chacun utilise un dossier et un port locaux distincts; un quatrième dossier sert au mineur. Le lanceur ne touche pas aux dossiers de vos nœuds habituels. Cet essai mesure la convergence et les redémarrages de processus, pas un réseau entre machines ni une panne électrique.

Il faut Python 3.10 ou plus et une archive de binaires issue d’un commit identifié. Vérifiez son SHA-256 avec celui de la Release avant de l’extraire. Gardez le PC alimenté, éveillé et suffisamment ventilé. Ne désactivez pas les protections de Windows. Le lanceur arrête l’essai en cas d’erreur, de trou de suivi supérieur à cinq minutes en plus de l’intervalle prévu, de dépassement de 20 Gio sur disque ou de 4 Gio de mémoire pour un nœud. Ces limites sont modifiables par arguments.

Depuis le dossier extrait, dans PowerShell :

```powershell
python .\scripts\testnet_endurance.py --binary .\bin\aretovus-node.exe --output .\endurance-72h --hours 72
```

Sous Linux/macOS, remplacez les chemins par `./scripts/testnet_endurance.py` et `./bin/aretovus-node`. Le dossier de sortie doit être nouveau. L’essai produit environ un bloc par minute, vérifie que les trois nœuds atteignent le même sommet et tue le pair intermédiaire tous les trente blocs avant de le redémarrer et mesurer son rattrapage. Le temps de calcul s’ajoute si une opération dépasse la cadence prévue. Il utilise une adresse factice sans clé privée : les récompenses de cet essai ne sont pas destinées à être dépensées.

Pour arrêter proprement : créez un fichier vide `STOP` dans le dossier de sortie. Le lanceur finit le tour courant, arrête ses processus, restaure une copie du dossier B et marque l’essai comme arrêté, sans le compter comme 72 h réussies. Une fermeture forcée du lanceur peut laisser ses processus enfants actifs; ne terminez que les PID consignés dans son journal après avoir vérifié leurs chemins et arguments.

## Preuves conservées

- `metadata.json` : heure de début UTC, PID du lanceur, empreinte du binaire, ports et paramètres.
- `measurements.jsonl` : événements horodatés, hauteur et sommet communs, mémoire résidente de chaque serveur, taille totale du dossier, durées des tours et de récupération.
- `a.log`, `b.log`, `c.log` : sorties des serveurs; les données de chaîne restent disponibles.
- `summary.json` : état final, durée, hauteur, résultat de restauration ou erreur. `duration_passed` et `duration_completed: true` ne valent que pour la durée demandée dans `metadata.json`. Vérifier qu’elle était bien de 72 h.

Un essai court avec `--max-rounds 6 --restart-every 3 --interval-seconds 0` sert à vérifier le lanceur. Il est marqué `short_run_passed` et ne vaut jamais un essai de 72 h. Les mesures RSS sont des échantillons pris après convergence, pas une mesure des pointes de mémoire. Les tailles de fichiers excluent l’espace alloué et les métadonnées du système de fichiers. Les arrêts forcés sont programmés entre les tours; ils ne prouvent pas une coupure au milieu de chaque type d’écriture.

La campagne est réussie seulement après lecture du résumé et des mesures, sans erreur ni interruption. Ajouter ensuite le commit, le SHA-256 du binaire, la durée, les hauteurs, les maxima observés et les temps de rattrapage dans `TESTNET_VALIDATION_REPORT.md`. Les essais multi-hôtes, les partitions réseau, la saturation réelle du disque et la coupure matérielle restent distincts.

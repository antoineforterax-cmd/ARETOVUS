# Admission des connexions P2P avant les workers

## Problème et changement

Les huit workers attendaient le premier message TCP de chaque pair. Le plafond
de quatre sessions par IP limitait une seule source, mais huit sources lentes
pouvaient occuper tous les workers pendant le délai de handshake de deux secondes.

Un dispatcher lit désormais les préfaces en mode non bloquant avant de les
confier aux workers. Il conserve au maximum 32 connexions en attente, avec un
buffer fixe de 77 octets chacune. Seules les longueurs 1 (wallet) et 73 (pair)
sont recevables. Les types et longueurs incompatibles sont rejetés sans allouer
une trame de 4 MiB. Le fingerprint et les contrôles de genèse restent validés
dans le traitement de session.

La file des handshakes prêts est limitée à huit éléments et les workers restent
au nombre de huit. Les connexions en attente, en file et en session partagent le
plafond existant de quatre par IP. Une connexion conserve son bail jusqu'à la
fin ou au rejet ; les baux libérés ne laissent pas d'entrée dans le compteur.

Le dispatcher traite au plus 32 acceptations par passage, puis attend 5 ms. Une
connexion muette ou partielle expire deux secondes après son admission. Le
temps passé en file ne prolonge pas ce délai : le worker rejette une tâche
expirée avant de répondre. Une file pleine entraîne la fermeture de la nouvelle
connexion prête, sans allocation ni attente illimitée.

## Budgets

- 32 sockets en préface, 8 en file prête, 8 sessions de travail au maximum,
  hors socket d'écoute, connexions sortantes et backlog TCP du système.
- Préfaces en attente : 32 × 77 = 2464 octets de buffers fixes, plus structures
  Rust et buffers TCP du système. Ce chiffre n'est pas une mesure de RAM totale.
- Huit threads de session et un dispatcher dans le thread serveur ; aucun thread
  par connexion en attente. Le thread existant de synchronisation reste distinct.
- Limites de trame, de session et délais de lecture/écriture inchangés après
  handshake. Aucun format réseau, consensus, compte ou snapshot modifié.

## Essai reproductible

Le workflow `p2p-admission` construit le commit de référence
`b990ce84c06c8a2bd0ef1c6593c595e8b7ab3e61` et le commit proposé, puis utilise le
même script pour les deux :

```sh
python3 scripts/testnet_admission_probe.py --binary target/debug/aurora-node --max-wallet-seconds 1.5 --report target/admission-after.json
```

Le script ne contacte que le serveur qu'il démarre sur la boucle locale. Sous
Linux, il utilise des adresses sources 127/8 distinctes pour tester huit sockets
muettes depuis une IP, huit IP muettes et huit préfaces partielles multi-IP.
Il mesure le handshake wallet, vérifie une synchronisation P2P et vérifie la
récupération après fermeture des connexions. La limite de 1,5 seconde est une
assertion de régression locale, pas un engagement de latence sur Internet.

Quatre tests Rust couvrent fragmentation, conservation de la trame suivante,
expiration sans prolongation, préfaces invalides, libération des baux et maintien
du contrôle de fingerprint. Les tests existants de wallet, sync et endurance
sont toujours requis. Les résultats avant/après sont conservés en artefacts CI
et les liens d'exécution dans la PR.

## Protection supplémentaire contre la saturation après HELLO

Même lorsqu'aucun worker n'attend une préface, les 32 **sessions actives**
peuvent être occupées par des pairs ayant terminé HELLO puis cessé d'envoyer
leurs requêtes. Le cas reproductible utilise huit IP de loopback (quatre
sessions par IP), puis demande un HELLO wallet depuis une neuvième IP.

Quand les 32 places sont occupées, le dispatcher peut désormais libérer
**la session inactive la plus ancienne**, sous les conditions cumulatives :
handshake validé, plus de deux secondes sans progression réelle de lecture
ou d'écriture, aucun travail en vol, aucune requête prête et aucune réponse
à envoyer. Un nouveau client ne devient pas « prioritaire » sur la base
d'une identité réseau invérifiable ; cette protection assure seulement une
possibilité de reprise de capacité sous une saturation par sessions inactives.
Elle peut expulser un wallet légitime resté inactif lors d'une saturation.

Le chronomètre de progression ne rallonge pas le délai **absolu** de lecture
(10 secondes) ni le délai total de session (15 minutes). Les plafonds de
32 sessions et quatre connexions par IP restent inchangés.

Le scénario supplémentaire `saturated_after_hello` dans
`scripts/testnet_admission_probe.py` est exécuté par le workflow
`p2p-admission` ; il vérifie le wallet supplémentaire, une synchronisation,
et la récupération après fermeture des 32 clients. Le premier commit de test
a reproduit le défaut sur le runner CI
([run 41113506674](https://github.com/antoineforterax-cmd/AURORA/actions/runs/41113506674)),
avec une fermeture de la connexion wallet ; il faut vérifier les commits
de correction et leurs propres CI avant de déclarer ce scénario résolu.

## Limites

La protection des préfaces lentes et la récupération des sessions inactives
ne clôturent pas entièrement #4/#8.
Des sessions actives et occupées par des requêtes valides peuvent toujours
saturer les huit workers ; une attaque distribuée peut saturer le quota de 32
sessions, les préfaces ou le backlog. La préemption n'authentifie pas un pair
et peut échouer sous renouvellement adversarial soutenu.
Le plafond par IP peut pénaliser plusieurs utilisateurs derrière un NAT ; aucune
authentification ne permet ici de réserver une capacité à un pair de confiance.
Le sondage toutes les 5 ms a un coût CPU et une latence à mesurer hors CI.
Les essais ne constituent ni une protection générale contre le déni de service,
ni une preuve de résistance multi-hôtes.

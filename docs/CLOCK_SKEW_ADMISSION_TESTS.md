# Écarts d'horloge et admission locale — campagne de tests

## Politique déjà exécutée

- Le consensus reste indépendant de l'heure système : timestamp strictement supérieur au parent et à la médiane des ancêtres ; difficulté et travail cumulé calculés à partir des horodatages validés.
- Le nœud, le mineur et le wallet SPV rejettent **localement** un bloc ou en-tête situé à plus de 7 200 secondes de l'horloge de la machine. Ce contrôle d'admission ne doit **pas** être ajouté au rejeu consensus des snapshots, car le résultat dépendrait du moment du redémarrage.
- Un bloc refusé en raison de l'heure locale peut devenir admissible après correction de l'horloge. Aucun bloc ni état ne doit être publié à partir d'une page invalide.

## Test déterministe de frontière et de réorganisation

Le test `divergent_local_clocks_delay_a_retarget_reorg_without_changing_consensus` du module P2P suit ce scénario :

1. Deux chaînes Testnet avec genèse et PoW identiques progressent de neuf blocs à timestamps réguliers.
2. Au dixième bloc (frontière du retarget), un bloc d'une branche proche reste dans la fenêtre du nœud `now = 1000`, tandis qu'un bloc concurrent à `now + 7201` est rejeté par cette horloge. Le modèle consensus peut toujours vérifier la branche candidate et ses bits de difficulté.
3. Un autre nœud à `now = 1002` admet la branche concurrente et son enfant, dont le travail cumulé devient supérieur. Le nœud en retard conserve son sommet et son état, qui sont vérifiés par sauvegarde et restauration.
4. Après correction locale à `now = 1002`, le premier nœud admet les mêmes bytes, se réorganise vers la même branche, puis restaure le même hash de sommet, travail cumulé et nombre de blocs.

Le test simule la **décision temporelle** avec des horloges injectées, et effectue des validations consensus réelles sur deux chaînes Rust ; il ne simule pas deux systèmes d'exploitation ou sockets TCP avec horloges différentes. Les vecteurs du client SPV sont testés séparément. Le résultat est utilisable seulement après succès de la CI sur le SHA de la PR correspondante.

## Transfert TCP et nouvel essai après rattrapage

Le test `tcp_clock_catchup_retries_an_atomic_future_page_and_restores` passe
par un **vrai socket TCP loopback** et reçoit une page contenant d'abord un
bloc valide, puis un bloc dont l'horodatage dépasse de une seconde la borne
du client. La fonction d'admission existante est injectée avec `now=1000` :
le client doit refuser la page entière, garder la genèse et ne rien écrire
dans le dossier de stockage. Un deuxième essai sur le même serveur avec
`now=1001` doit accepter exactement les deux blocs et restaurer le même
hash et le même travail depuis un snapshot.

Pour assurer la déterminisme du test, le flux TCP est réel mais **l'heure
locale du client est injectée**, sans modifier l'horloge du système ni les
règles de consensus. La voie de production continue d'utiliser
`reject_future_block_timestamp` fondée sur l'heure système, à travers
une fonction d'admission fournie au synchroniseur. Il ne s'agit pas d'une
campagne multi-hôtes sous décalage réel.

## Limites et suites #3

- À tester : désynchronisations de plusieurs minutes/heures sur de vrais processus, horloge reculée pendant un transfert TCP, attente/réessai du peer au lieu d'un abandon définitif, redémarrage sous dérive, page P2P contenant blocs admissibles puis un bloc futur, convergence et preuves SPV sous réorgs.
- La politique de deux heures est **locale** ; la rendre obligatoire en consensus, changer la politique du miner ou l'empreinte du protocole exigerait une décision de versionnement explicite et l'analyse des anciennes données.
- Aucun test ne démontre ici une immunité à la manipulation d'horloges ou à des pairs hostiles répartis sur Internet.

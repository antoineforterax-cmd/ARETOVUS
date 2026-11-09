# Correctifs de disponibilité issus de la revue adversariale

Revue initiale : commit 62fcf0cefb2a78c7c2f60d37a49c1fea646026bb.
Les trois constats ont été confirmés par lecture ; cette PR ajoute les
correctifs et les régressions exécutables. Les résultats effectifs et le SHA
validé sont consignés dans la PR #36.

## Blocs déjà connus

Une recherche BTreeMap du hash précède désormais l'allocation de la chaîne
candidate. Un doublon conserve le reçu usuel sans copie de l'historique ni
sauvegarde supplémentaire. Le test répète cent fois le chemin sans candidat.
L'admission des nouveaux blocs conserve la validation, la sauvegarde et la
publication atomique existantes. La copie candidate des nouveaux blocs reste
un coût à optimiser : ce correctif ne prétend pas supprimer tous les coûts
d'admission ou toutes les attaques de charge.

## Consultation de comptes

Le mempool conserve une seule vue pending calculée pour le tip courant.
Chaque accès mutable au vecteur des transactions invalide cette vue, ainsi
que son remplacement après transition de chaîne. Un changement de tip force
aussi son recalcul. L'admission d'une transaction et la récupération du
mempool peuvent réutiliser l'état déjà calculé pour initialiser le cache.

Une lecture chaude accède au compte par BTreeMap sans copier le registre ni
rejouer le pool. Une lecture froide peut encore construire la vue sous les
verrous ; elle coûte O(comptes + rejeu du mempool). Le cache ajoute au plus
un registre pending conservé par nœud, non persistant. Les tests vérifient
cent lectures avec une construction, les mutations/remplacements, l'inclusion
d'un paiement et une réorganisation qui le détache. Les autres tests de
reconstruction du pool et de panne de sauvegarde restent applicables.

## Connexions après handshake

Les sockets de production restent non bloquants dans le dispatcher. Les huit
workers ne reçoivent que des requêtes complètes et construisent la réponse
avec un transport mémoire. Ils ne lisent et n'écrivent aucun socket client.
Un client inactif, une trame partielle ou un lecteur lent n'immobilise donc
plus un worker de traitement pendant son délai réseau.

La rotation des sessions et un quota de 64 Kio par passage pour chaque lecture
ou écriture empêchent un seul socket de monopoliser la boucle. Les compteurs
de session restent globaux à la connexion malgré le traitement par requête.

Bornes :
- 32 handshakes incomplets, 32 sessions établies, quatre connexions par IP ;
- huit workers, huit requêtes en file, huit résultats en file ;
- une requête en vol par session, trames de 4 Mio au maximum ;
- budgets de session existants : quinze minutes, 16 384 requêtes, 256 Mio ;
- délai de préface de deux secondes, budget de phase d'entrées/sorties
  de dix secondes, sans renouvellement à chaque fragment.

Les buffers et files sont bornés, mais peuvent représenter plusieurs centaines
de Mio dans un cas de grosses trames simultanées, en plus de la chaîne, du
cache pending, des objets temporaires et des buffers du système. Le quota de
lecture peut réduire le débit de gros transferts : mesures longues encore
requises. Une saturation distribuée de toutes les places de connexion ou des
calculs coûteux reste possible ; ce n'est pas une garantie anti-DDoS universelle.

La sonde loopback ajoute huit connexions établies sur deux IP, soit inactives
après une requête valide, soit bloquées au milieu d'une trame annoncée à 4 Mio.
Un neuvième client wallet et une synchronisation doivent rester servis
(latence handshake wallet inférieure à 1,5 s sur le runner). Les mesures sont
conservées par le workflow p2p-admission. La référence historique ne sert
qu'aux anciens cas pré-handshake ; ne pas inventer de comparaison mesurée
avant/après pour les nouveaux cas.

## Compatibilité et limites

Aucune règle monétaire, transaction, bloc, trame ou empreinte de protocole
n'est modifiée. Les clients conservent les connexions multi-requêtes.
Le changement est local à l'ordonnancement, au cache et à l'admission rapide
des doublons. Aucun format de disque ni migration de données n'est ajouté.

CI générale, sonde d'admission et partitions répétées sont les contrôles
prévus. Une campagne multi-hôtes prolongée, une mesure RSS/débit à grande
échelle et une revue indépendante restent nécessaires. Revenir à l'ancien
nœud réintroduit les trois chemins identifiés. PR laissée non fusionnée pour
les essais et la revue, testnet expérimental uniquement.

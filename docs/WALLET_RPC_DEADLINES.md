# Délais totaux des trames RPC du wallet

Les lectures read_exact et écritures write_all utilisaient auparavant uniquement
les délais système par opération de socket. Un pair envoyant périodiquement
quelques octets pouvait prolonger une trame bien au-delà du délai annoncé.

Le wallet applique désormais un budget monotone commun au préfixe de longueur
et à la charge utile, en lecture comme en écriture. Chaque appel reçoit
uniquement le temps restant. Les interruptions ne réinitialisent pas le budget.
Le budget maximal est de dix secondes, ou le délai de socket s'il est plus court.

Les paramètres de socket d'origine sont restaurés après succès ou erreur.
Les limites de trame restent quatre MiB ; aucune modification des messages,
signatures, frais ou règles de consensus.

Quatre tests TCP couvrent une réponse au compte-gouttes, le partage du délai
entre préfixe et charge, les réponses tronquées/surdimensionnées, les transferts
valides et un pair qui n'accepte plus les écritures. Les tests vérifient aussi
la restauration des paramètres de socket. Les contrôles multi-nœuds et wallet
existants vérifient la compatibilité des échanges normaux.

Le budget porte sur chaque trame, pas sur l'ensemble d'une synchronisation SPV
ou d'une commande comportant plusieurs requêtes. Une erreur termine l'échange :
aucune tentative de réutiliser une trame partiellement reçue n'est ajoutée.

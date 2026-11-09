# Réinsertion des transactions annulées par une réorganisation

Le serveur reconstruit le mempool lors d'une transition canonique, après réception
directe d'un bloc ou synchronisation de fond. Il recherche le préfixe commun par
dichotomie, parcourt les blocs détachés dans leur ordre, puis les transactions
déjà en attente. Les paiements détachés passent en premier pour restaurer les
prédécesseurs de nonce dont peuvent dépendre les paiements en attente.

Chaque candidat doit être natif v3 Testnet, absent de la nouvelle chaîne
canonique, non dupliqué et valide contre l'état du nouveau sommet augmenté des
transactions déjà retenues. Les signatures peuvent utiliser le cache ; soldes
et nonces sont systématiquement revérifiés. Les limites existantes de 4096
transactions et 32 MiB s'appliquent à toute reconstruction.

Les doublons, conflits, signatures invalides ou transactions devenues non
financées sont écartés. Un dépassement de budget n'évince pas les transactions
déjà sélectionnées ; les entrées trop grandes sont ignorées. Aucun journal
d'attente illimité n'est créé pour les candidats rejetés.

Le nouveau mempool est calculé avant sauvegarde. Chaîne et mempool ne sont
publiés qu'après réussite du stockage, sous verrous dans l'ordre chaîne puis
mempool. Un échec de sauvegarde conserve les deux états en mémoire.

Cinq tests signés ML-DSA-65 couvrent dépendances de nonce, doublons, limites en
nombre et octets, confirmation/conflit sur la nouvelle branche, erreur de
stockage, synchronisation TCP de fond et réception directe par RPC de bloc.
La difficulté et les récompenses sont neutralisées dans ces tests ciblés.
Le smoke et l'endurance existants restent nécessaires.

Ce mécanisme concerne le mempool du serveur actif. Le mempool reste volatil :
une commande sync hors serveur ou un redémarrage ne restaure pas un mempool
persistant. Le relais périodique existant peut diffuser les paiements réinsérés.
Cette politique de sélection est locale au mempool et ne modifie pas le consensus.

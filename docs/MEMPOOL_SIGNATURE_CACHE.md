# Cache cryptographique du mempool

Le nœud réutilise désormais les réussites de vérification de signature dans les
lectures de comptes, admissions, revalidations et relais du mempool. Les règles
de réseau, format, nonce, financement et arithmétique restent évaluées à chaque
application. Le consensus de bloc utilise toujours la vérification sans cache.

Le cache de processus retient au plus 4096 txids SHA-256 complets, avec éviction
FIFO. Le txid engage réseau, version, champs et autorisation ; changer un champ
ou une signature change la clé. Les erreurs ne sont jamais mémorisées. Les clés
occupent au plus 128 Kio bruts, auxquels s'ajoutent table, file et allocations.
Aucune transaction ou clé privée n'est conservée dans ce cache.

Le verrou est relâché pendant la cryptographie. Deux threads peuvent vérifier
simultanément une même entrée absente ; cela duplique du travail sans permettre
d'admettre une signature non vérifiée. Un verrou empoisonné entraîne une
vérification ordinaire, sans contournement. Le cache disparaît au redémarrage.

Le benchmark release utilise maintenant la véritable implémentation
SignatureCache, initialement vide pour chaque échantillon. Les tests comparent
les transitions avec et sans cache, les mutations, l'éviction, la capacité nulle,
le rejeu après consommation du nonce et la restauration d'un état antérieur.

Les résultats et limites du benchmark initial restent dans MEMPOOL_BENCHMARK.md
comme historique ; les mesures courantes sont liées dans la PR. Les temps du
cache isolé ne sont pas une mesure de latence P2P. La suite smoke vérifie les
transactions, le relais et la réorganisation du nœud avec le cache activé.

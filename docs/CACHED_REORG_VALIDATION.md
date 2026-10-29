# Admission avec cache après réorganisation réelle

Deux tests d'intégration utilisent AccountForkChain, des blocs concurrents et
des transactions natives ML-DSA-65 signées. La sélection de branche est exécutée
par le code de consensus ; aucun registre n'est remplacé manuellement.

Le premier chauffe le cache, confirme un paiement, puis le rend orphelin avec
une branche plus longue. Le paiement doit redevenir admissible avec nonce et
fonds restaurés. La branche initiale reprend ensuite la tête et le paiement doit
être refusé comme rejeu, puis une nouvelle bascule restaure son admissibilité.

Le second confirme un paiement concurrent portant le même nonce sur la nouvelle
branche. L'ancienne transaction reste cryptographiquement valide et absente de
l'index canonique, mais doit être refusée à cause de l'état du compte.

À chaque tentative, validations avec et sans cache doivent rendre le même
résultat et le même état. Une tentative rejetée doit préserver l'état complet.
L'index des transactions canoniques est vérifié lors des bascules.

Commande :
cargo test --locked -p aurora-consensus --test cached_reorg

Les tests isolent les transitions et la sélection de chaîne en utilisant une
difficulté nulle et des récompenses désactivées. Ils ne mesurent ni propagation
P2P, ni réinsertion automatique des transactions orphelines dans le mempool,
ni restauration disque. Ces comportements constituent des périmètres distincts.

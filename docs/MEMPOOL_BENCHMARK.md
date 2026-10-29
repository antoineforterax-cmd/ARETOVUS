# Mesurer les validations répétées du mempool

Première étape de l'issue #7 : mesurer avant d'activer un cache.
Le benchmark se trouve dans `aurora-core/tests/mempool_benchmark.rs`.
Il génère des transactions v3 ML-DSA-65 signées avec une graine publique de test,
puis mesure en release trois tailles de file (1, 64, 256), seize passages et trois
échantillons par taille. Génération des clés, signatures et échauffement sont
hors chronométrage.

## Colonnes

- `verify_ns` : vérifications cryptographiques répétées.
- `txid_ns` : calculs répétés du hash des octets canoniques complets.
- `replay_ns` : copie du registre puis application ordonnée des transactions,
  avec vérification des signatures, soldes et nonces à chaque passage.
- `signature_cache_model_ns` : modèle isolé en mémoire avec calcul du txid à
  chaque accès, vérification sur premier accès puis recherche dans un ensemble.
  Le cache commence vide pour chaque échantillon. Ce modèle ne valide pas l'état.

Les durées sont des totaux en nanosecondes pour taille × seize opérations.
Aucun seuil de vitesse n'est imposé sur les runners partagés. Les deux temps de
signature ne représentent pas une accélération de bout en bout du nœud : le
modèle de cache n'effectue pas les transitions d'état. Le rejeu actuel fournit
une mesure séparée de ce travail complet, sans prétendre isoler chaque coût.

## Reproduction

```sh
cargo test --release --locked -p aurora-core --test mempool_benchmark -- --include-ignored --nocapture --test-threads=1
```

Le workflow `mempool-benchmark` conserve résultats, commit, version du compilateur
et CPU en artefacts pendant quatorze jours. La CI normale exécute le test
d'invariants et ignore seulement la mesure.

## Proposition à examiner après les résultats

Un éventuel cache de signatures doit être borné, indexé par txid canonique
complet et profil, et ne conserver que les réussites cryptographiques.
Le calcul du txid inclut la signature ; toute mutation change la clé du cache.
Un test vérifie les mutations de montant et de signature, le refus d'un rejeu
après consommation du nonce, la restauration de l'éligibilité après rollback
et le refus sur un état non financé.

Aucun cache n'est activé par ce lot. Avant activation, définir une limite
d'entrées/octets et une éviction, éprouver les mutations et réorganisations du
nœud réel, puis mesurer l'admission, les lectures et le relais sous charge.
Le microbenchmark n'exerce ni verrous P2P ni synchronisation concurrente.

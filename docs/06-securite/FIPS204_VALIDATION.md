# Vérification des exemples FIPS 204 / ACVP

Le testnet ARETOVUS v3 utilise déjà la signature native **ML-DSA-65**, fournie par RustCrypto `ml-dsa = 0.1.1`, verrouillée dans `Cargo.lock`. Il n’est pas nécessaire de changer de primitive pour commencer à vérifier sa conformité aux exemples publics du NIST. Aucun paramètre de consensus ni format de transaction n’est modifié par ce lot.

Le fichier `crates/aretovus-core/tests/fips204_vectors.rs` teste la même dépendance que le vérificateur de transactions. Les [données de référence](../tests/vectors-fips204/README.md) proviennent du dépôt officiel NIST ACVP-Server, commit `975de31eb83d87039ec88934fdc47d8c312b892d`. Les fichiers d’origine et les extractions ont des empreintes SHA-256 consignées; la notice NIST est conservée intégralement.

## Contrôles ajoutés

- 25 générations de clés : comparaison exacte de la clé publique dérivée de chaque seed NIST.
- 3 signatures déterministes : comparaison de tous les octets avec le résultat NIST; contextes vide et de 255 octets, message de 1 à 8192 octets. Le cas à contexte vide passe aussi par les traits `Signer` et `Verifier` employés par ARETOVUS.
- 15 vérifications externes « pure ML-DSA » : acceptation des 3 cas valides et rejet des 12 cas invalides publiés. Une signature qui échoue dès le décodage est bien rejetée.
- Rejet d’un contexte modifié et d’un contexte de 256 octets; refus de signer avec un contexte trop long.
- Test supplémentaire, construit à partir d’un exemple valide : refus aux bornes positives et négatives exactes de la norme de `z` (`gamma1 - beta`), avec cas de décodage juste à l’intérieur de la borne. Ce test dérivé est identifié séparément des exemples NIST non modifiés.

L’import de clés privées étendues utilise une API dépréciée uniquement dans le test de signature et seulement sur les exemples NIST fixes. Le portefeuille continue à générer et relire des seeds; aucune entrée utilisateur n’est transmise à cet import de test.

```sh
cargo test --locked -p aretovus-core --test fips204_vectors
```

Au moment de la rédaction, les fichiers de test sont préparés; **la compilation et le résultat des tests doivent être confirmés par la CI du commit qui les contient**. Le poste Windows de développement ne dispose pas d’un environnement de liens MSVC fonctionnel. Aucun succès d’exécution Rust local n’est revendiqué.

## Portée et limites

Ce jeu fournit des comparaisons indépendantes des signatures que la bibliothèque produit elle-même pendant les tests usuels. Il ne couvre pas tous les vecteurs ACVP, les variantes préhachées, toutes les sources d’aléa, les canaux auxiliaires, la sûreté mémoire de toutes les dépendances, ni la sécurité de l’intégration complète. Les tests de transaction, de domaine de signature et de mutation d’état restent nécessaires en complément : les messages NIST ne sont pas des transactions ARETOVUS.

Réussir ces tests ne constitue **ni une certification FIPS, ni une validation ACVTS, ni un audit indépendant**. [FIPS 204](https://csrc.nist.gov/pubs/fips/204/final) décrit ML-DSA comme conçu pour résister aussi aux adversaires disposant d’un grand ordinateur quantique; ce n’est pas une garantie mathématique absolue pour les décennies à venir. La page NIST signale également des corrections potentielles à suivre. Les dépendances, paramètres et procédures de migration doivent donc rester révisables.

Sources : [publication NIST FIPS 204](https://csrc.nist.gov/pubs/fips/204/final), [spécification ACVP ML-DSA](https://pages.nist.gov/ACVP/draft-celi-acvp-ml-dsa.html), [exemples NIST épinglés](https://github.com/usnistgov/ACVP-Server/tree/975de31eb83d87039ec88934fdc47d8c312b892d/gen-val/json-files). La source de la bibliothèque inspectée est le paquet RustCrypto `ml-dsa-0.1.1` téléchargé par Cargo, avec les API `from_seed`, `sign_deterministic`, `verify_with_context` et `Signature::decode`.

# Revue technique initiale — ARETOVUS

> Archive : décrit l’ancien prototype v0.0.1 au moment de sa rédaction. Pour les formats actuels, voir [CONSENSUS.md](CONSENSUS.md) et [PROTOCOL.md](PROTOCOL.md); pour les preuves, voir [TESTNET_VALIDATION_REPORT.md](TESTNET_VALIDATION_REPORT.md).

**Date :** 2026-10-25  
**Périmètre :** lecture du code source et résultat de la CI GitHub du prototype expérimental.  
**Verdict :** prototype de recherche uniquement ; aucune garantie de sécurité des fonds.

## Vérifications réellement observées

La CI GitHub a réussi sur le commit `11c898821e1d983e25c62ec7c0af9efc0d3991e9` :
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo build --workspace --locked`
- `cargo test --workspace --locked`

Cela confirme que le code de ce commit passe ces contrôles automatisés dans l'environnement CI. Cela ne prouve ni la correction du protocole ni sa résistance à des attaques. La revue présente est une inspection du code, pas un audit cryptographique indépendant.

## Points positifs observés

- Rust interdit explicitement `unsafe` au niveau du workspace.
- Les montants sont des entiers bornés ; les additions passent par des opérations vérifiées.
- Le décodeur borne les longueurs et refuse les octets finaux superflus.
- Les identifiants utilisent SHA-256 avec séparation de domaines.
- Le prototype rejette les entrées répétées dans une même transaction, les transactions dupliquées dans un bloc, les engagements incorrects et les blocs dont le parent ne correspond pas au sommet de la chaîne locale.
- Les tests automatisés couvrent plusieurs cas nominaux et négatifs.

## Risques bloquants

| Domaine | Constat | Conséquence |
|---|---|---|
| Autorisation | `auth_data` n'est pas une signature vérifiée | Impossible de prouver le droit de dépenser une sortie |
| État monétaire | Pas de registre des sorties dépensées ni de validation des fonds | Pas de prévention complète des doubles dépenses entre transactions/blocs |
| Émission | Pas de récompense de bloc ni de règle d'émission validée | Aucun approvisionnement monétaire consensuel |
| Consensus | Difficulté passée par l'appelant et constante dans la chaîne locale | Des nœuds peuvent appliquer des règles différentes |
| Chaîne | Pas de branches concurrentes, de réorganisation ni de sélection robuste par travail cumulé | Pas de consensus distribué |
| Temps | Les horodatages ne sont pas validés | Les règles temporelles d'un futur protocole ne sont pas définies |
| Réseau | Pas de réseau pair-à-pair ni de validation des messages distants | Pas de réseau blockchain opérationnel |
| Persistance | Chaîne uniquement en mémoire | Perte de l'état au redémarrage |
| Cryptographie | Aucun audit indépendant ni vecteurs de signature, car les signatures ne sont pas implémentées | Aucune revendication de sécurité cryptographique possible |

## Priorités de développement

1. **Ne pas choisir silencieusement UTXO ou comptes.** Produire un comparatif court et un prototype jetable des deux modèles ; le porteur du projet valide ensuite le choix.
2. **Spécifier l'autorisation de dépense.** Comparer des bibliothèques et formats de signature établis, y compris les options post-quantiques et hybrides ; ne jamais écrire une primitive cryptographique maison.
3. **Créer un validateur d'état déterministe.** Il devra vérifier l'existence des entrées, leur non-répétition dans l'état, la somme des entrées/sorties/frais, et l'émission autorisée. Cette étape dépend du modèle de transaction choisi.
4. **Renforcer la CI.** Ajouter un `Cargo.lock` suivi dans Git, puis `cargo audit` et `cargo deny` après validation de leurs configurations ; ajouter ensuite des tests de propriétés et du fuzzing des décodeurs.
5. **Tester le consensus adversarialement.** Difficulté, horodatages, branches, réorganisations, travail cumulé et divergence entre implémentations.
6. **Revue indépendante et testnet isolé** avant toute expérimentation avec de la valeur réelle.

## Conditions avant toute utilisation avec des fonds

Toutes les fonctions bloquantes ci-dessus doivent être spécifiées, implémentées, testées et revues indépendamment. Le code actuel ne doit pas être présenté comme une blockchain sécurisée ni être utilisé pour stocker ou transférer de vrais fonds.

## Références de méthode

- Guide ANSSI, *Règles de programmation pour le développement d'applications sécurisées en Rust* : https://messervices.cyber.gouv.fr/guides/regles-de-programmation-pour-le-developpement-dapplications-securisees-en-rust
- Checklist Secure Rust Guidelines : https://anssi-fr.github.io/rust-guide/checklist.html

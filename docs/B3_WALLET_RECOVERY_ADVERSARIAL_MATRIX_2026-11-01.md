# ARETOVUS — B3 : matrice adversariale du portefeuille et de la récupération

**Base :** testnet ARETOVUS natif ML-DSA-65 v3, après B2 validée sur `main`. [Plan maître #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142). Cette matrice relie les assertions du code Rust à des menaces réelles ; un test est **prouvé** seulement après GitHub Actions `completed/success` au SHA exact, et un succès CI n'est pas un audit indépendant.

## Matrice

| Menace | Résultat attendu | Test / protection |
| --- | --- | --- |
| Chiffrement ARETOVK4, mauvais mot de passe | Aucun secret rendu, aucune modification du fichier | `keys::tests::encrypted_key_roundtrip_rejects_wrong_password_and_tampering` |
| Manipulation de l'en-tête Argon2id (mémoire / passes / lanes), magic, salt, nonce, ciphertext ou tag | Authentification refusée, aucune réinterprétation de clé, octets source intacts | **Nouveau** `keys::tests::encrypted_wallet_adversarial_headers_and_lengths_never_restore_foreign_bytes` |
| Fichier chiffré tronqué ou allongé | Longueur stricte ARETOVK4 ; échec avant usage de la graine privée | Même nouveau test |
| Backup invalide et destination vierge | La vérification échoue et **ne crée pas** de clé de remplacement | Même nouveau test |
| Backup valide vers une destination préexistante | `create_new` interdit l'écrasement d'un autre fichier même si la source est authentique | Même nouveau test, `keys::tests::encrypted_backup_restores_identity_without_reusing_ciphertext` |
| Copie de sauvegarde légitime | Identité attendue identique et chiffrement différent par sel/nonce neufs | `keys::tests::encrypted_backup_restores_identity_without_reusing_ciphertext` |
| Récupération après perte du fichier/mot de passe | Code de récupération jetable régénère la même identité et permet une signature native ; sauvegarde re-chiffrée | `keys::tests::recovery_survives_original_file_and_password_loss_and_signs` |
| Code de récupération étranger, altéré, tronqué, UTF-8 invalide ou trop long | Erreur, pas de substitution de compte | `keys::tests::recovery_file_rejects_corruption_oversize_and_foreign_identity`, `recovery::tests::every_single_character_change_and_truncation_is_rejected` |
| Anciennes clés d'un autre réseau | Magics historiques ARETOVUS incompatibles rejetés sans écraser l'original | `keys::tests::historical_key_magic_is_rejected_for_plain_and_encrypted_files` |
| Symlink de fichier secret, remplacement avant ouverture | Refuser symlink / reparse point, vérifier identité du fichier, ouvrir O_NOFOLLOW sur Unix ; pas de lecture d'un secret étranger | `keys::tests::wallet_secret_readers_reject_symlinks_but_accept_original_files` (Unix), `wallet_file_swap_to_symlink_of_same_inode_fails_after_preflight`, workflows Windows/macOS/Linux |
| Paquets wallet RPC incomplets, trop longs, réponses trompeuses | Trames bornées, deadline cumulée, reçu TXID exact, refus au mauvais fingerprint | `rpc::tests::trickled_prefix_and_payload_share_one_deadline`, `truncated_or_oversized_frames_restore_read_timeout`, `submission_accepts_only_the_exact_prepared_transaction_id`, `wallet_rejects_a_node_with_different_protocol_fingerprint` |
| En-têtes SPV futurs, preuve Merkle étrangère, confirmations | Vérification de chaîne en-têtes depuis genèse fixe, difficulté, horodatages, égalité du header canonique, Merkle, nombre minimal de confirmations | `spv::tests::spv_future_header_admission_has_exact_boundary`, `admission_vector_tests::independent_admission_vectors_match_local_policy` et `aretovus-core::merkle` ; **tests réseau hostiles de bout en bout encore à renforcer** |

## Portée et risques résiduels

- Le test ARETOVK4 utilise un **seed de synthèse** et un **mot de passe de fixture** sans fonds. Aucun enregistrement de graine, clé privée, code de récupération ou vrai mot de passe ne doit apparaître dans les logs et artefacts.
- Argon2id est configuré à 64 MiB, trois passes et une voie ; l'acceptation de paramètres modifiés n'est pas permise. Ces tests contrôlent des **rejets**, pas la résistance prouvée à une attaque cryptographique ou à un accès local avec privilèges.
- Le fichier de récupération est un **secret en clair** : le garder hors synchronisation et sauvegardes publiques. La récupération actuelle n'est ni sociale ni on-chain.
- Le contrôle SPV nécessite également une validation contre un pair hostile intégral de bout en bout : les tests de parseur RPC, Merkle et horloges ne démontrent pas à eux seuls toutes les attaques d'un pair malveillant.
- Le délai RPC couvre les lectures/écritures par session ; pas de preuve de réseau public hostile, ni de test sur deux PC physiques ou d'endurance 72 heures.
- Aucun changement de protocole, de cryptographie, de formats, de gestion des fonds ni de politique de récupération. Pas de fonds réels et pas de déclaration mainnet.

## Portes de validation

Vérifier au **SHA exact** : `cargo fmt --all -- --check`, Clippy, build/test `--locked`, smoke, workflow wallet Windows/macOS/Linux, tests d'ouverture secrets, sauvegarde cross-runner et régressions RPC/SPV pertinentes. Tout échec/annulation oblige correction et nouvelle CI. Réviser les chemins d'erreur et constater les limites des sous-revues ; si aucun spécialiste indépendant n'est disponible, ne pas prétendre à une revue indépendante. Après fusion autorisée, refaire les tests sur le SHA de fusion et seulement alors cocher B3 dans #142.

# ARETOVUS — B2 : stockage, crash, corruption et reprise

**Portée :** stockage expérimental ARETOVUS (manifeste v6 `ARETOIDX`, pages `ARETOPAG`, blocs immuables). [Plan maître #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142), [PR B2 #157](https://github.com/antoineforterax-cmd/ARETOVUS/pull/157).

Cette matrice énumère des **assertions du dépôt** : elle n'est pas une preuve de succès avant la conclusion `completed/success` des tests sur le SHA exact. Les erreurs d'injection sont testées sur fichiers jetables et runners autorisés, **pas** lors d'une véritable coupure électrique. Les archives et journaux historiques restent immuables.

| Menace/phase de panne | Oracle déterministe | Tests référents |
| --- | --- | --- |
| **Testnet natif : dernier manifeste modifié** | Nouveau tip non adopté, repli exact sur ancien tip, anciens octets inchangés, conflit d'écriture immuable non écrasé ; restauration manuelle de la fixture rétablit le dernier tip | `native_testnet_damaged_latest_manifest_falls_back_without_overwrite` (#157) |
| **Testnet natif : page v6 falsifiée avec checksum interne recalculé** | Le hash contenu/adresse doit refuser la page, ancienne génération lisible, sauvegarde ne répare pas l'objet étranger, ancien manifeste préservé | `native_testnet_damaged_v6_page_falls_back_without_silent_repair` (#157) |
| Écriture partielle, ENOSPC | Aucun nouveau bloc publié, anciennes données et snapshots restaurables, fichier temporaire nettoyé puis retry possible | `injected_partial_write_enospc_preserves_snapshot_without_publishing_block`, `genuine_enospc_tmpfs_preserves_recovery_and_retry` |
| Échec `File::sync_all` pour bloc/manifeste | Erreur conservée, dernier point de reprise valide, nouveau manifeste non publié, retry réussi | `injected_file_sync_failure_preserves_previous_manifest_and_allows_retry`, `manifest_file_sync_failure_preserves_last_valid_generation` |
| Échec rename atomique après sync fichier | Aucun chemin final invalide publié, suppression du temporaire, ancienne génération retenue | `manifest_rename_failure_preserves_last_valid_generation`, `injected_atomic_rename_failure_keeps_previous_snapshot_and_cleans_temp` |
| Échec fsync du dossier **après** rename | Fichier pouvant être visible mais durabilité non garantie ; ancien manifeste conservé ; nouvelle publication synchronisée au retry | `manifest_parent_sync_failure_leaves_visible_generation_recoverable`, `directory_sync_failure_after_rename_does_not_publish_a_new_manifest` |
| Manifeste tronqué ou pages v6 manquantes/corrompues | Repli déterministe vers génération valide ; ne pas publier un tip dont les objets ne sont pas validables | `truncated_latest_manifest_recovers_last_complete_snapshot`, `corrupted_latest_v6_page_recovers_previous_manifest`, `missing_latest_index_page_falls_back_without_publishing_invalid_tip` |
| Corruption de plusieurs générations et élagage | Préserver l'ancre ancienne et l'intégrité du dernier manifeste réellement récupérable | `two_corrupted_newest_manifests_recover_third_generation_and_retry`, `pruning_after_corrupt_manifests_must_preserve_last_valid_recovery_point`, `v6_pages_are_pruned_only_after_last_manifest_reference` |
| Symlink, échange de fichiers, répertoire interne dévié | Rejet de substitution à l'ouverture, jamais suivre un symlink à un objet externe, refuser les répertoires internes non réels | `read_rejects_regular_file_swapped_between_metadata_and_open`, `read_rejects_file_swapped_for_symlink_between_metadata_and_open`, `internal_storage_directory_links_cannot_be_used_for_restore_or_save` |
| Index secondaire/forks, snapshots sauvegardés et restaurés | Tous blocs et branches secondaires restaurés, choix canonique par travail cumulé, cohérence du curseur d'arrivée | `snapshot_roundtrip_keeps_secondary_branches_for_reorganization`, `stored_arrival_hashes_match_block_headers_across_side_branches`, `latest_snapshot_uses_cumulative_work_even_when_winning_chain_is_shorter` |
| Page mise en cache puis contenu sur disque altéré | Cache interdit de faire confiance à une page absente ou changée ; aucune publication fondée sur un contenu obsolète | `cached_complete_v6_page_revalidates_disk_and_exact_arrival_prefix`, `cached_page_io_failure_after_reuse_preserves_previous_manifest` |
| Transport de sauvegarde entre machines virtuelles GitHub | Export/import entre runners et réorganisation, avec données jetables et empreintes | `cross_runner_v6_backup_producer`, `cross_runner_v6_backup_consumer`, workflow `cross-runner-v6-backup` |

## Ordre de durabilité à préserver

`write_immutable` crée un temporaire avec `create_new`, écrit les octets, fait `File::sync_all`, puis rename et sync du répertoire. `save_snapshot_inner` vérifie et synchronise les blocs/pages avant de publier le manifeste v6 ; la suppression des anciennes générations ne démarre qu'après cette publication. Un échec après rename mais avant fsync du répertoire **ne démontre pas** une durabilité de crash : les tests de retry doivent la garantir.

## Validation et limites

- Vérifier Rustfmt, Clippy, build/test `--locked`, suite stockage Windows et Linux, garde contre symlinks/substitution, retry Windows, ENOSPC réel sous tmpfs isolé, sauvegardes inter-runners, P2P et CI du SHA candidat. Corriger toute régression observée avant fusion.
- Après autorisation de fusion : revalider le SHA exact de `main` et produire tableau `SHA → run ID → conclusion → OS → limites`, puis seulement cocher B2.
- Les anciens lecteurs historiques v4/v5 restent une voie de tests spécifiques, **pas** une migration automatique de l'ancienne blockchain vers le nouveau protocole natif.
- L'environnement CI ne prouve ni crash électrique réel, ni deux PC physiques, ni 72 h, ni résistance aux administrateurs hostiles. Les tests Regtest existants ne remplacent pas les deux tests natifs ajoutés.
- Aucune modification de consensus, formats, paramètres économiques ou politique de récupération par cette PR. Pas de fonds réels ni certificat mainnet.

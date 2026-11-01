# ARETOVUS — B1 : matrice adversariale consensus et économie

**Périmètre :** nouveau testnet ARETOVUS (transactions ML-DSA-65 v3, réseau incompatible). Référence : [issue maître #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142), [issue horodatages #3](https://github.com/antoineforterax-cmd/ARETOVUS/issues/3), [PR B1 #155](https://github.com/antoineforterax-cmd/ARETOVUS/pull/155).

Le présent document énumère **des assertions présentes dans le code**, et **non des résultats d'exécution**. Une assertion n'est prouvée que par les workflows GitHub Actions `completed/success` correspondant au **SHA exact** de la PR, puis par les contrôles sur le SHA fusionné. Une anomalie confirmée exigerait test rouge, correction, test vert, nouvelle revue. **Aucune modification du protocole n'est approuvée par cette matrice.**

## Matrice et oracle déterministe

| Risque | Scénario et résultat attendu | Référence de test |
| --- | --- | --- |
| Double dépense intra-bloc, Testnet natif | Deux transactions ML-DSA-65 réellement signées du même expéditeur et nonce 0, bénéficiaires distincts ; bloc refusé sans nouvelle entrée, tip, récompense, offre, balance ni nonce modifiés | `account_fork_chain::tests::native_testnet_double_spend_and_bad_signature_reject_atomically` |
| Signature modifiée après signature | Champ montant changé puis engagement du bloc recalculé ; bloc rejeté sans effet d'état ou de récompense ; transaction intacte reste acceptée ensuite | Même test natif |
| Concurrence de branches, même nonce | Deux dépenses v3 signées sur branches soeurs ; la branche précédemment non canonique devient gagnante après un enfant ; un seul paiement et un seul nonce survivent ; récompenses orphelines retirées ; offre calculée sur la branche gagnante | `account_fork_chain::tests::native_testnet_competing_nonce_spends_reorg_without_double_payout` |
| Nonce rejoué ou sauté dans une même branche | `nonce=0` rejoué après application, `nonce=2` prématuré ; état inchangé au rejet | `state::tests::replayed_or_skipped_nonce_is_rejected`, `account_fork_chain::tests::child_state_rejects_nonce_replay_against_its_parent` |
| Atomicité de lots et fonds insuffisants | Transaction invalide après opération valide dans un lot : aucun débit/crédit partiel ; signature erronée et fonds insuffisants refusés sans mutation | `state::tests::failed_batch_does_not_apply_earlier_transactions`, `invalid_authorization_leaves_state_unchanged`, `insufficient_funds_leaves_state_unchanged` |
| Auto-transfert et limites | Montant nul, auto-transfert, montant+frais au-dessus des limites rejetés | `accounts::tests::rejects_zero_amount_and_same_account`, `rejects_total_above_amount_bound` |
| Compatibilité de signature interdite | Transactions Ed25519 v1 et hybrides v2 sur **Testnet natif** rejetées avant validation et débit ; seule v3 autorisée | `accounts::tests::native_testnet_stateless_rejects_legacy_profiles_but_keeps_v3`, `account_fork_chain::tests::testnet_rejects_legacy_ed25519_and_hybrid_transaction_profiles` |
| Plafond et émission | Émission proportionnelle integer ; à une unité du plafond, exactement une unité émise puis zéro ; aucune réémission ; absence de mineur quand payout exigé : refus sans mutation | `reward::tests::cap_boundary_mints_last_unit_once_and_then_stops`, `reward_errors_leave_the_ledger_atomic_at_supply_cap` |
| Overflow et retour arrière | Débordement de crédit d'émission, récompense de frais vers solde saturé : aucune mutation partielle | `state::tests::issuance_credit_is_bounded_per_account_and_globally`, `reward::tests::reward_errors_leave_the_ledger_atomic_at_supply_cap` |
| Frais/burn/trésorerie | Récompense 95/5 de l'émission (troncature à l'unité) ; frais impairs : moitié entière au mineur, reste brûlé ; émission brute non gonflée par frais | `reward::tests::fee_split_rounds_miner_down_and_burns_remainder`, test intra-bloc natif ci-dessus |
| Émission aux très grandes hauteurs | Récurrence réelle calculée sans blocs réseau : points arithmétiques 10 M, 100 M, 500 M ; pas de simulation P2P revendiquée | `reward::tests::emission_million_height_vectors_match_the_real_reward_function`; `scripts/emission_sim.py` |
| Chaîne au travail cumulé maximal | Réorg vers chaîne de travail cumulé supérieur, pas simplement le nombre de blocs ; état/revenus reconstruits depuis le parent de la branche | `difficulty::tests::cumulative_work_compares_expected_hash_cost_not_height`, `account_fork_chain::tests::reorganization_replays_rewards_from_the_winning_parent_state` |
| Retarget, horodatages extrêmes | Temps strictement croissants ; fenêtres futuristes ne peuvent bouger de plus de 2 bits par intervalle et respectent les bornes min/max, y compris elapsed 0 et `u64::MAX` | `difficulty::tests::retarget_extreme_elapsed_time_remains_bounded_and_deterministic`, `account_fork_chain::tests::future_dated_retarget_windows_cannot_jump_more_than_one_adjustment_bound` |

## Limites et menaces résiduelles

- **Testnet ≠ Regtest :** les tests propres aux anciens profils en Regtest n'établissent pas la sécurité du Testnet natif. Les cas natifs v3 ajoutés dans #155 ciblent les transitions et les réorganisations sous la vraie logique de comptes.
- La borne d'horloge locale de deux heures est un **contrôle d'admission réseau / wallet / SPV**, pas une règle de rejeu consensus. Les scénarios réels de désynchronisation entre plusieurs machines restent à démontrer ([issue #3](https://github.com/antoineforterax-cmd/ARETOVUS/issues/3)).
- La réinsertion et propagation réseau du mempool lors des réorganisations, les forks hostiles WAN, les tests prolongés sur deux PC physiques et l'audit humain indépendant ne sont pas démontrés par la matrice B1. Ils appartiennent également aux phases C et E.
- Les montants 10 M/100 M/500 M sont des simulations arithmétiques et non autant de blocs réellement stockés, minés ou propagés.
- Le test de récompense d'overflow injecte artificiellement un total de frais en API interne après émission maximale pour vérifier **l'atomicité de l'erreur** : cela ne remplace pas un scénario complet où des transactions ont précédemment payé ces frais.
- Toutes les clés/signatures sont synthétiques, sans valeur financière ; aucun fonds réel. Pas d'audit indépendant, pas de feu vert mainnet.

## Critères de clôture

1. CI `cargo fmt --all -- --check`, Clippy, build et tests workspace `--locked`, smoke et suites partitions/fuzz pertinentes **`completed/success` à la tête exacte**.
2. Relecture séparée du diff pour chercher erreurs de fixture, oracle tautologique, confusion Testnet/Regtest et mutations implicites de consensus. Si aucune revue humaine externe n'est disponible, indiquer explicitement la limite.
3. Fusion **seulement après approbation spécifique** et contrôles verts, puis validation de `main` fusionné au SHA exact.
4. Remplir le registre de preuves [#142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142) avant de cocher B1 et passer à B2.

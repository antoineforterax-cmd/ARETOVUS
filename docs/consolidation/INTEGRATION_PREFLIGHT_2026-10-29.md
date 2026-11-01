# ARETOVUS — Contrôle préalable à l'intégration des branches

_Date du relevé : 29 octobre 2026. Ce document présente des constats GitHub vérifiés ; il ne vaut pas autorisation de fusion._

## Résultat immédiat

**Aucun changement de consensus et aucune fusion ne sont nécessaires pour documenter ce constat.** Les fondations du testnet avancé existent sur `feature/emission-schedule-v0`, mais restent séparées de `main`. Plusieurs PR se recouvrent par ascendance : les traiter toutes comme des patches indépendants introduirait des risques de conflit et des tests mal attribués.

À ce relevé, **23 PR sont ouvertes et en brouillon** (#1, #2, #12-#29, #35-#37). Le SHA de `feature/emission-schedule-v0` est `0fbf9545a5bd60319d08ee64c9932e5ce04c2543`.

## Constatations vérifiées par comparaison de branches

| Comparaison | Résultat GitHub | Conséquence |
| --- | --- | --- |
| `main` → `docs/phase-0-protocol` (#1) | `diverged` : 23 commits d'avance, 1 de retard | Le README propre à `main` n'est pas dans la base de #1 ; préparer une intégration conservant les deux |
| `docs/phase-0-protocol` → `feature/emission-schedule-v0` (#2) | `ahead` : 479 commits, 0 de retard | Le testnet est un descendant de la fondation historique #1 |
| `docs/phase-0-protocol` → `build/reproducible-foundation` (#37) | `ahead` : 2 commits, 0 de retard | #37 ajoute un verrou Cargo, un toolchain fixe et des contrôles de reproductibilité à #1 |
| `build/reproducible-foundation` → `feature/emission-schedule-v0` | `diverged` : 479 commits d'avance, 2 de retard | Les deux commits propres à #37 ne sont pas dans le testnet actuel |
| `test/repeated-network-partitions` (#24) → `integration/two-pc-testnet` (#25) | `ahead` : 21 commits, 0 de retard | #25 contient déjà les commits de #24 |
| `fix/bounded-p2p-handshake-admission` (#16) → #25 | `ahead` : 32 commits, 0 de retard | #25 contient déjà les commits de #16 |
| `test/signed-v3-self-transfer` (#13) → #25 | `ahead` : 32 commits, 0 de retard | #25 contient déjà les commits de #13 |
| `feature/emission-schedule-v0` → #25 | `diverged` : 34 commits d'avance, 16 de retard | #25 n'intègre pas les 16 derniers commits du testnet de référence |
| `docs/aurora-master-plan-2026-10` (#12) → #25 | `diverged` : 23 commits d'avance, 6 de retard | La documentation stratégique doit être réconciliée séparément |
| `feat/offline-payment-signing` (#29) → `fix/adversarial-node-dos` (#36) | `ahead` : 12 commits, 0 de retard | #36 dépend de la chaîne de protection des fonds |

_« Ahead » indique une ascendance des commits, pas un succès d'exécution ni une absence de conflit après combinaison avec d'autres branches._

## Travaux déjà vérifiés, à ne pas recommencer inutilement

La [PR #37](https://github.com/antoineforterax-cmd/ARETOVUS/pull/37) a déjà construit une carte détaillée, verrouillé les dépendances de la fondation et exécuté ses contrôles sur `e0254493e93adce6080ba129ce32f6683f772ae3`. Les workflows `ci` et `foundation-check` associés sont marqués **success**. Sa description indique également un contrôle distinct de #2, sans fusion combinée. La preuve ne doit pas être interprétée comme une validation intégrée de #37 et #2.

Les workflows `ci`, `p2p-admission` et `network-partitions` de la [PR #36](https://github.com/antoineforterax-cmd/ARETOVUS/pull/36) sur `8227d97436497120c5fd590095a9364d162a038a` sont également marqués **success**. Aucun de ces succès ne prouve le comportement d'une fusion avec les changements les plus récents de `feature/emission-schedule-v0`.

## Prochain protocole d'intégration (sans fusion automatique)

1. **Préparer une simulation isolée de #1 + #37 vers `main`** : garder le README actualisé de `main`, épingler les SHAs, vérifier les fichiers réellement modifiés, puis exécuter les tests du prototype. Ne pas réécrire `main` directement.
2. **Simuler l'ajout de #2** : traiter explicitement `Cargo.lock`, `rust-toolchain.toml`, `.github/workflows/ci.yml` et `AGENTS.md`, qui existent dans les branches pertinentes. Tester la combinaison et conserver les résultats associés à son SHA de simulation.
3. **Évaluer #25 comme point de regroupement** : ne pas réappliquer aveuglément #13-#24. Comparer la version intégrée à la dernière branche de référence, puis conserver les modifications documentaires et techniques manquantes.
4. **Intégrer les chaînes sensibles par revue** : #26-#29, #35-#36 nécessitent une analyse de sécurité wallet, de récupération d'accès et de résistance aux attaques ; #19 (fuzzing) est une branche parallèle, pas un maillon ordinaire de #20.
5. **Contre-revues de sécurité distinctes** : consensus et monnaie ; P2P/DoS ; wallet/clés/restauration ; stockage et intégration. Chaque anomalie exige reproduction et test de non-régression.
6. **Tests sur la combinaison finale** : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace --locked`, `cargo test --workspace --locked`, `python3 scripts/testnet_smoke.py`, plus les scénarios TCP, partitions, récupération, fuzz, reorgs et deux PC réels adaptés à la branche.
7. **Décision humaine de fusion** : fournir la liste des modifications, les SHAs, les risques et les résultats ; ne fusionner dans `main` que sur autorisation explicite du responsable.

## Risques non clos

- **Intégration non prouvée** : les CI séparées ne remplacent pas une CI sur le commit combiné.
- **Résilience à grande échelle** : synchronisation réelle de plus de 100 000 blocs, réseau multi-machines, saturation prolongée, disque réellement plein, résultats finaux d'endurance 72 h à confirmer.
- **Sécurité des fonds** : chaque mécanisme de récupération ou de signature hors ligne doit être testé face aux erreurs d'utilisateur, à la compromission et aux incohérences de sauvegarde.
- **Audit extérieur** : aucune revue qualifiée, indépendante et documentée couvrant le système final n'est confirmée. Les propres essais du responsable et les revues automatisées restent utiles, sans remplacer cet examen.

## Conditions de clôture de ce chantier de cartographie

- [x] Relever les PR ouvertes et les SHAs de leurs branches.
- [x] Vérifier les liens de dépendance et les recouvrements importants par l'API GitHub de comparaison.
- [x] Examiner les workflows de #37 et #36 sur leurs SHAs précis.
- [x] Identifier l'écart de 16 commits entre la base testnet actuelle et #25.
- [ ] Exécuter une simulation Git de fusion #1 / #37 / #2 dans un environnement de compilation isolé.
- [ ] Soumettre la combinaison à une CI complète et à une revue adversariale distincte.
- [ ] Faire valider les fusions, puis recontrôler l'historique intégré.

**Statut : inventaire et précontrôle terminés ; aucune intégration ou preuve de compatibilité combinée annoncée.**

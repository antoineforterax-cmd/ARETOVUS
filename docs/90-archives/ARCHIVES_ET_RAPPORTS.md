# Archives techniques et rapports — comment retrouver une preuve

**Les anciens documents restent accessibles, mais ne doivent pas être confondus avec le protocole courant.** Leur date, leur branche et le SHA de référence priment. Cette page est un **catalogue**, pas un effacement, un audit ni une nouvelle feuille de route.

Pour commencer : [index documentaire](../README.md) · [état actuel](../01-demarrer/ETAT_ACTUEL.md) · [contradictions réconciliées](../03-protocole/DECISIONS_CLARIFIEES.md) · [feuille V2](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178).

## Références à consulter en priorité (pas des archives)

- [CONSENSUS.md](../CONSENSUS.md), [PROTOCOL.md](../PROTOCOL.md), [TESTNET_DECISIONS.md](../TESTNET_DECISIONS.md) : spécification et décisions du **profil testnet** documenté ; confronter au SHA.
- [EMISSION_MODEL.md](../EMISSION_MODEL.md) : détails économiques et simulations, pas un modèle mainnet approuvé.
- [TESTNET_GUIDE.md](../TESTNET_GUIDE.md), [TESTNET_OPERATIONS.md](../TESTNET_OPERATIONS.md), [guide français](../guide-fr/README.md) : procédures datées et environnements expérimentaux.
- [INDEPENDENT_REVIEW.md](../06-securite/INDEPENDENT_REVIEW.md), [THREAT_MODEL.md](../04-securite/THREAT_MODEL.md), [FUZZING.md](../04-securite/FUZZING.md) : exigences de sécurité et méthodes.
- [TESTNET_VALIDATION_REPORT.md](../TESTNET_VALIDATION_REPORT.md) : preuves et limites **historiques**, vérifier le SHA et les workflows.
- [Provenance](../../PROVENANCE.md), [licences](../05-contribuer/LICENCES_ET_DROITS.md), [contribution](../../CONTRIBUTING.md) : attribution, réutilisation et participation.

## 1. Anciennes décisions, roadmaps et états figés

| Documents | À quoi ils servent désormais |
| --- | --- |
| [ancienne baseline (Git)](https://github.com/antoineforterax-cmd/ARETOVUS/blob/caa94228b2fb769e3a5ea479ea5fd07485744489/docs/DECISION_BASELINE.md), [DECISION_REGISTER.md](DECISION_REGISTER.md) | Hypothèses et arbitrages initiaux, **pas** paramètres du testnet courant |
| [ancienne feuille d'exécution (Git)](https://github.com/antoineforterax-cmd/ARETOVUS/blob/caa94228b2fb769e3a5ea479ea5fd07485744489/docs/ROADMAP_EXECUTION.md), [FINALIZATION_CHECKLIST.md](FINALIZATION_CHECKLIST.md) | Anciennes séquences de travail et bilan d'octobre, non feuille de route vivante |
| [planning/DECISIONS.md](../planning/DECISIONS.md), [ancienne feuille de planification (Git)](https://github.com/antoineforterax-cmd/ARETOVUS/blob/caa94228b2fb769e3a5ea479ea5fd07485744489/docs/planning/ROADMAP.md), [planning/ARCHITECTURE_CIBLE.md](../planning/ARCHITECTURE_CIBLE.md) | Propositions et rapprochements de conception datés |
| [consolidation/STATE_2026-10-29.md](../consolidation/STATE_2026-10-29.md), [consolidation/state-2026-10-29.json](../consolidation/state-2026-10-29.json), [consolidation/INTEGRATION_PREFLIGHT_2026-10-29.md](../consolidation/INTEGRATION_PREFLIGHT_2026-10-29.md) | Photographes de consolidation de fin octobre |
| [REVIEW_2026_10_29.md](REVIEW_2026_10_29.md), [SECURITY_REVIEW_2026-10-25.md](SECURITY_REVIEW_2026-10-25.md), [POST_MERGE_VALIDATION_GATES_2026-10-31.md](POST_MERGE_VALIDATION_GATES_2026-10-31.md) | Relectures et critères d'intégration datés |
| [E3_PRELIMINARY_NO_GO_2026-11-02.md](E3_PRELIMINARY_NO_GO_2026-11-02.md) | Motifs d'un NO-GO historique ; il ne se transforme pas en GO par vieillissement |
| [PROJECT_HISTORY.md](PROJECT_HISTORY.md), [BRANDING_COMPATIBILITY.md](BRANDING_COMPATIBILITY.md), [D4_LEGACY_PR_BRANDING_INVENTORY_2026-11-02.md](D4_LEGACY_PR_BRANDING_INVENTORY_2026-11-02.md) | Historique des identités, archives et renommage |

## 2. Rapports sur le consensus, les comptes et l'économie

- [ACCOUNT_STATE_FUZZING.md](../04-securite/ACCOUNT_STATE_FUZZING.md), [B1_CONSENSUS_ADVERSARIAL_MATRIX_2026-11-01.md](../06-rapports/B1_CONSENSUS_ADVERSARIAL_MATRIX_2026-11-01.md), [CACHED_REORG_VALIDATION.md](../06-rapports/CACHED_REORG_VALIDATION.md).
- [RETARGET_SIMULATION.md](../03-protocole/RETARGET_SIMULATION.md), [CLOCK_SKEW_ADMISSION_TESTS.md](../06-rapports/CLOCK_SKEW_ADMISSION_TESTS.md), [STATE_MODEL_COMPARISON.md](../03-protocole/STATE_MODEL_COMPARISON.md).
- [FIPS204_VALIDATION.md](../03-protocole/FIPS204_VALIDATION.md), [POST_QUANTUM.md](../03-protocole/POST_QUANTUM.md) : tests et limites cryptographiques ; **pas certification** du protocole.
- [ORPHAN_TRANSACTION_REINSERTION.md](../06-rapports/ORPHAN_TRANSACTION_REINSERTION.md), [HEADER_PAGINATION_REVIEW.md](../06-rapports/HEADER_PAGINATION_REVIEW.md).

## 3. Réseau, P2P, stockage et endurance

- [ADVERSARIAL_DOS_FIXES.md](../04-securite/ADVERSARIAL_DOS_FIXES.md), [P2P_ADMISSION.md](../06-rapports/P2P_ADMISSION.md), [P2P_COUNTER_REVIEW_2026-10-27.md](../06-rapports/P2P_COUNTER_REVIEW_2026-10-27.md), [C1_P2P_ADMISSION_RED_TEST_TRIAGE_2026-11-02.md](../06-rapports/C1_P2P_ADMISSION_RED_TEST_TRIAGE_2026-11-02.md).
- [P2P_VOLUME_CAMPAIGN.md](../06-rapports/P2P_VOLUME_CAMPAIGN.md), [NETWORK_PARTITIONS.md](../06-rapports/NETWORK_PARTITIONS.md), [MEMPOOL_BENCHMARK.md](../06-rapports/MEMPOOL_BENCHMARK.md), [MEMPOOL_SIGNATURE_CACHE.md](../06-rapports/MEMPOOL_SIGNATURE_CACHE.md).
- [STORAGE_V6_DESIGN.md](../03-protocole/STORAGE_V6_DESIGN.md), [B2_STORAGE_CORRUPTION_MATRIX_2026-11-01.md](../06-rapports/B2_STORAGE_CORRUPTION_MATRIX_2026-11-01.md), [D2_LARGE_VOLUME_FEASIBILITY_2026-11-02.md](../06-rapports/D2_LARGE_VOLUME_FEASIBILITY_2026-11-02.md).
- [ENDURANCE_RUN.md](../02-utiliser/ENDURANCE_RUN.md), [ENDURANCE_WSL.md](../02-utiliser/ENDURANCE_WSL.md), [TWO_PC_TEST.md](../02-utiliser/TWO_PC_TEST.md), [INTEGRATION_TWO_PC.md](../02-utiliser/INTEGRATION_TWO_PC.md).

## 4. Portefeuille, protection, récupération et recherches futures

- [ESSENTIAL_WALLET_CLEANUPS.md](ESSENTIAL_WALLET_CLEANUPS.md), [PERSISTENT_WALLET_LIMITS.md](../02-utiliser/PERSISTENT_WALLET_LIMITS.md), [WALLET_FUND_PROTECTION.md](../04-securite/WALLET_FUND_PROTECTION.md), [WALLET_RPC_DEADLINES.md](../04-securite/WALLET_RPC_DEADLINES.md).
- [B3_WALLET_RECOVERY_ADVERSARIAL_MATRIX_2026-11-01.md](../06-rapports/B3_WALLET_RECOVERY_ADVERSARIAL_MATRIX_2026-11-01.md), [OFFLINE_PAYMENTS.md](../02-utiliser/OFFLINE_PAYMENTS.md), [OFFLINE_RECOVERY.md](../02-utiliser/OFFLINE_RECOVERY.md).
- [DID_RECOVERY_DESIGN.md](../03-protocole/DID_RECOVERY_DESIGN.md), [SECURITY_CUSTODY_THREAT_MODEL.md](../04-securite/SECURITY_CUSTODY_THREAT_MODEL.md) : recherche et menaces, **pas** récupération effective des avoirs.
- [AI_COUNTER_REVIEW.md](../04-securite/AI_COUNTER_REVIEW.md), [HUMAN_REVIEW_CALL.md](../04-securite/HUMAN_REVIEW_CALL.md) : pistes de contre-revue et appel à une vérification humaine, sans présumer qu'elle a eu lieu.

## 5. Conceptions et prototypes V2

Les documents de `docs/planning/` consacrés au desktop (ADR, architecture, IPC, erreurs, pagination, frontières de confiance et labo Windows) documentent des **décisions et expériences propres à une version donnée**. Consulter d'abord [apps/desktop/README.md](../../apps/desktop/README.md) et la [feuille V2](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178).

## Documents retirés de la branche et archives conservées

Les anciens guides `guide-fr/07-technologie-et-suite.md` et `guide-fr/08-interface-locale.md`, ainsi que `DECISION_BASELINE.md`, `ROADMAP_EXECUTION.md` et `planning/ROADMAP.md`, ont été sélectionnés pour retrait de cette branche documentaire. Ils restent accessibles via [leur version Git de référence](https://github.com/antoineforterax-cmd/ARETOVUS/blob/caa94228b2fb769e3a5ea479ea5fd07485744489/docs/guide-fr/07-technologie-et-suite.md), dans l'historique et les PR. Les liens dans les guides actifs ont été mis à jour. Les comptes rendus d'essais, bilans adversariaux, registres de décisions et sources de preuve demeurent conservés dans l'arbre : les effacer ferait perdre du contexte et pourrait casser des liens historiques.

**Règle d'archivage :** conserver le texte et sa date, ajouter un lien vers la documentation courante, ne jamais réécrire un rapport ancien pour faire croire qu'il validait un protocole ultérieur.

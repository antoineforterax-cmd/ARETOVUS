# Centre documentaire AURORA

**Un seul point d'entrée pour retrouver les documents existants** — inventaire effectué sur `main` au commit `b76e0cff6d7e2ee6527cac2506ad005d7360ec4f`, le **31 octobre 2026**. Les documents de développement continuent d'évoluer ; vérifier leur SHA et leur date. **AURORA reste un testnet expérimental non audité pour les fonds réels.**

## Si vous ne devez lire qu'une page

| Je veux… | Lire d'abord |
| --- | --- |
| Découvrir ce qu'est AURORA et essayer un wallet | [Le guide pour débutants](guide-fr/README.md) |
| Comprendre la **vision et l'ordre de développement** | [Vision et navigation de projet](planning/README.md) |
| Comparer **tous les arbitrages restants avec A / B / C** | [Questionnaire complet (68 décisions)](planning/DECISION_OPTIONS.md) |
| Connaître les règles **actives** | [Consensus](CONSENSUS.md), [paramètres testnet](TESTNET_DECISIONS.md) |
| Suivre les preuves, risques et tâches | [Rapport d'essais](TESTNET_VALIDATION_REPORT.md), [critères de finalisation](FINALIZATION_CHECKLIST.md) |
| Relire, contribuer ou signaler une faille | [CONTRIBUTING](../CONTRIBUTING.md), [SECURITY](../SECURITY.md) |
| Vérifier les licences et l'origine du code | [Licence double MIT/Apache](../LICENSE), [provenance](../PROVENANCE.md) |

## Qui fait autorité ?

| Question | Document de référence | Ce qu'il ne faut pas confondre |
| --- | --- | --- |
| **Comportement réellement exécuté** | Code source + tests sur un **commit identifié** | Un fichier Markdown ne remplace pas une vérification du code |
| **Formats, règles et économie du testnet** | [PROTOCOL](PROTOCOL.md), [CONSENSUS](CONSENSUS.md), [TESTNET_DECISIONS](TESTNET_DECISIONS.md) | Pas une garantie de sécurité mainnet |
| **Choix de développement et décisions ouvertes** | [Décisions actuelles](planning/DECISIONS.md) et [questionnaire](planning/DECISION_OPTIONS.md) | Un choix proposé n'est pas approuvé |
| **Avancement réel et validation** | [Rapport d'essais](TESTNET_VALIDATION_REPORT.md) et CI attachée au SHA | CI verte ≠ audit externe |
| **Vision et recherche future** | [Cahier des charges](planning/CAHIER_DES_CHARGES.md) et [roadmap #123](https://github.com/antoineforterax-cmd/AURORA/issues/123) | Ambition ≠ fonctionnalité livrée |
| **Anciens choix, revue et notes datées** | [Registre d'archives](DECISION_REGISTER.md), [ancienne baseline](DECISION_BASELINE.md) | Historiquement utile, mais souvent dépassé |

## Principe de rangement

Les **67 fichiers de documentation existants** ne sont pas 67 documents à lire pour démarrer : on les classe ci-dessous selon leur *fonction*. Tous les originaux restent accessibles et l'historique Git est préservé. Le nombre de fichiers n'est pas en lui-même un défaut : **les doublons normatifs, les contradictions non datées et l'absence de navigation le sont**.

1. **Une seule source normative par sujet.** Un guide débutant renvoie aux règles ; il ne les recopie pas dans une nouvelle « spécification ».
2. **Tests et archives sont conservés.** Les comptes rendus de sessions ne doivent pas être écrasés par un résumé marketing.
3. **Toute référence porte sa provenance.** Réseau, date, commit, version, limites.
4. **Les anciennes PR et anciens statuts ne sont pas l'état présent.** Ne jamais confondre ancien prototype, PR ouverte, `main`, binaires publiés et mainnet.
5. **Pas de déplacement massif aveugle.** La première phase crée une carte vérifiable en conservant **tous les chemins existants**. Les déplacements éventuels se feront par lots, avec mise à jour de tous les liens (y compris liens externes et scripts) et redirections de compatibilité si nécessaire.

## Catalogue complet (inventaire du 31 octobre 2026)

Chaque document apparaît dans **une seule catégorie**. Déplier un thème pour voir ses fichiers, sans perdre les détails des travaux de recherche et des tests.

<details>
<summary><strong>1 · Comprendre et essayer</strong> · 8 documents</summary>

- [guide-fr/README.md](guide-fr/README.md) — Point de départ pour un débutant
- [guide-fr/01-comprendre.md](guide-fr/01-comprendre.md)
- [guide-fr/02-economie.md](guide-fr/02-economie.md)
- [guide-fr/03-installation.md](guide-fr/03-installation.md)
- [guide-fr/04-reseau-et-minage.md](guide-fr/04-reseau-et-minage.md)
- [guide-fr/05-portefeuille.md](guide-fr/05-portefeuille.md)
- [guide-fr/06-exploitation.md](guide-fr/06-exploitation.md)
- [guide-fr/07-technologie-et-suite.md](guide-fr/07-technologie-et-suite.md)

</details>


<details>
<summary><strong>2 · Spécifications et règles exécutées</strong> · 7 documents</summary>

- [CONSENSUS.md](CONSENSUS.md) — Règles de consensus du profil testnet
- [PROTOCOL.md](PROTOCOL.md) — Spécification protocolaire détaillée
- [TESTNET_DECISIONS.md](TESTNET_DECISIONS.md) — Choix monétaires effectivement appliqués
- [EMISSION_MODEL.md](EMISSION_MODEL.md)
- [POST_QUANTUM.md](POST_QUANTUM.md)
- [STORAGE_V6_DESIGN.md](STORAGE_V6_DESIGN.md)
- [PERSISTENT_WALLET_LIMITS.md](PERSISTENT_WALLET_LIMITS.md)

</details>


<details>
<summary><strong>3 · Vision, décisions et recherche</strong> · 12 documents</summary>

- [planning/README.md](planning/README.md) — Vision et navigation de la planification
- [planning/CAHIER_DES_CHARGES.md](planning/CAHIER_DES_CHARGES.md)
- [planning/DECISIONS.md](planning/DECISIONS.md) — Tableau d'état des choix et des orientations
- [planning/ARCHITECTURE_CIBLE.md](planning/ARCHITECTURE_CIBLE.md)
- [planning/ROADMAP.md](planning/ROADMAP.md)
- [planning/IMPLEMENTATION_POLICY.md](planning/IMPLEMENTATION_POLICY.md)
- [ROADMAP_EXECUTION.md](ROADMAP_EXECUTION.md)
- [STATE_MODEL_COMPARISON.md](STATE_MODEL_COMPARISON.md)
- [DID_RECOVERY_DESIGN.md](DID_RECOVERY_DESIGN.md)
- [WALLET_FUND_PROTECTION.md](WALLET_FUND_PROTECTION.md)
- [OFFLINE_PAYMENTS.md](OFFLINE_PAYMENTS.md)
- [OFFLINE_RECOVERY.md](OFFLINE_RECOVERY.md)

</details>


<details>
<summary><strong>4 · Exécuter et exploiter les nœuds</strong> · 7 documents</summary>

- [TESTNET_GUIDE.md](TESTNET_GUIDE.md)
- [TESTNET_OPERATIONS.md](TESTNET_OPERATIONS.md)
- [TWO_PC_TEST.md](TWO_PC_TEST.md)
- [INTEGRATION_TWO_PC.md](INTEGRATION_TWO_PC.md)
- [ENDURANCE_RUN.md](ENDURANCE_RUN.md)
- [ENDURANCE_WSL.md](ENDURANCE_WSL.md)
- [validation/MULTI_HOST_72H_RUNBOOK_2026-10-31.md](validation/MULTI_HOST_72H_RUNBOOK_2026-10-31.md)

</details>


<details>
<summary><strong>5 · Sécurité, contre-revue et audit</strong> · 6 documents</summary>

- [SECURITY_CUSTODY_THREAT_MODEL.md](SECURITY_CUSTODY_THREAT_MODEL.md)
- [AI_COUNTER_REVIEW.md](AI_COUNTER_REVIEW.md)
- [INDEPENDENT_REVIEW.md](INDEPENDENT_REVIEW.md)
- [HUMAN_REVIEW_CALL.md](HUMAN_REVIEW_CALL.md)
- [ADVERSARIAL_DOS_FIXES.md](ADVERSARIAL_DOS_FIXES.md)
- [P2P_COUNTER_REVIEW_2026-10-27.md](P2P_COUNTER_REVIEW_2026-10-27.md)

</details>


<details>
<summary><strong>6 · Résultats et méthodes de validation</strong> · 18 documents</summary>

- [FINALIZATION_CHECKLIST.md](FINALIZATION_CHECKLIST.md)
- [POST_MERGE_VALIDATION_GATES_2026-10-31.md](POST_MERGE_VALIDATION_GATES_2026-10-31.md)
- [TESTNET_VALIDATION_REPORT.md](TESTNET_VALIDATION_REPORT.md) — Preuves et limites des tests au SHA indiqué
- [ACCOUNT_STATE_FUZZING.md](ACCOUNT_STATE_FUZZING.md)
- [CACHED_REORG_VALIDATION.md](CACHED_REORG_VALIDATION.md)
- [CLOCK_SKEW_ADMISSION_TESTS.md](CLOCK_SKEW_ADMISSION_TESTS.md)
- [FIPS204_VALIDATION.md](FIPS204_VALIDATION.md)
- [FUZZING.md](FUZZING.md)
- [HEADER_PAGINATION_REVIEW.md](HEADER_PAGINATION_REVIEW.md)
- [MEMPOOL_BENCHMARK.md](MEMPOOL_BENCHMARK.md)
- [MEMPOOL_SIGNATURE_CACHE.md](MEMPOOL_SIGNATURE_CACHE.md)
- [NETWORK_PARTITIONS.md](NETWORK_PARTITIONS.md)
- [ORPHAN_TRANSACTION_REINSERTION.md](ORPHAN_TRANSACTION_REINSERTION.md)
- [RETARGET_SIMULATION.md](RETARGET_SIMULATION.md) — Vecteurs et comparaison de retarget
- [P2P_ADMISSION.md](P2P_ADMISSION.md)
- [P2P_VOLUME_CAMPAIGN.md](P2P_VOLUME_CAMPAIGN.md)
- [WALLET_RPC_DEADLINES.md](WALLET_RPC_DEADLINES.md)
- [validation/P2P_STORAGE_COST_MODEL_2026-10-31.md](validation/P2P_STORAGE_COST_MODEL_2026-10-31.md)

</details>


<details>
<summary><strong>7 · Archives et constats datés</strong> · 9 documents</summary>

- [DECISION_BASELINE.md](DECISION_BASELINE.md)
- [DECISION_REGISTER.md](DECISION_REGISTER.md) — Registre d'origine, pas une liste de décisions encore ouvertes
- [REVIEW_2026_10_29.md](REVIEW_2026_10_29.md)
- [SECURITY_REVIEW_2026-10-25.md](SECURITY_REVIEW_2026-10-25.md)
- [THREAT_MODEL.md](THREAT_MODEL.md) — Ancien modèle de menace explicitement archivé
- [ESSENTIAL_WALLET_CLEANUPS.md](ESSENTIAL_WALLET_CLEANUPS.md)
- [consolidation/INTEGRATION_PREFLIGHT_2026-10-29.md](consolidation/INTEGRATION_PREFLIGHT_2026-10-29.md)
- [consolidation/STATE_2026-10-29.md](consolidation/STATE_2026-10-29.md)
- [consolidation/state-2026-10-29.json](consolidation/state-2026-10-29.json) — État historique brut horodaté

</details>


## Évolutions documentaires restantes

- **Étape 1 (présente PR)** : inventaire complet, parcours lisible, questionnaire unique d'arbitrages, zéro suppression.
- **Étape 2 (PR séparées)** : vérifier les liens sortants réels, classer les doublons *de contenu* (pas les simples intitulés), corriger les versions historiques mal signalées et limiter les copies de spécifications.
- **Étape 3 (si utile)** : déplacer les pièces purement historiques dans des sous-répertoires par thème avec compatibilité, **sans casser leurs références**. Les documents de référence activement liés peuvent rester à leur emplacement stable.
- **Étape 4** : ajouter un contrôle automatique des liens/documentation aux PR une fois les règles de classement stabilisées.

**Attention :** les archives `LICENSE`, `LICENSE-MIT` et `LICENSE-APACHE` ne sont pas des doublons arbitraires : elles représentent le même choix explicite `MIT OR Apache-2.0`. Aucun fichier de licence n'est touché par ce rangement.

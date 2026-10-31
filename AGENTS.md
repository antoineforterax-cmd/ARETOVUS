# Instructions pour Codex — ARETOVUS

Ce fichier s'applique aux travaux sur cette branche. Guide complet de consolidation : [politique d'implémentation](https://github.com/antoineforterax-cmd/AURORA/blob/docs/aurora-master-plan-2026-10/docs/planning/IMPLEMENTATION_POLICY.md). **Lire le guide complet avant toute action Git susceptible de modifier les bases ou de fusionner des branches.** Une mise à jour du guide n'est pas une approbation automatique de ses propositions.

## Objet

ARETOVUS est une blockchain Rust **expérimentale** ; aucun fonds réel ni annonce de sécurité mainnet. Le responsable fixe la vision, les orientations architecturales et les décisions de protocole. Codex accélère l'implémentation, les tests, la revue et la documentation. Ne jamais inventer une approbation, une preuve, un test réussi ou un audit indépendant.

## Situation et branches

- `main` : prototype historique ; PR #1 : `docs/phase-0-protocol` vers `main`.
- PR #2 : `feature/emission-schedule-v0` vers `docs/phase-0-protocol`, fondation du testnet avancé.
- PR #12–#16 : travaux indépendants depuis le testnet.
- Chaîne A : #17 → #18 → #20 → #21 → #22 → #23 → #24 ; #19 part de #18 en parallèle de #20.
- Chaîne B : #25 → #26 → #27 → #28 → #29 → #35 → #36.
- État relevé le 29 octobre 2026 ; **reconsulter GitHub**, car bases, SHA, statuts et nombre de PR peuvent changer.

## Procédure obligatoire, dans cet ordre

1. **Inspecter** `README.md`, `CONTRIBUTING.md`, `docs/CONSENSUS.md`, `docs/TESTNET_DECISIONS.md`, `docs/FINALIZATION_CHECKLIST.md` et `docs/TESTNET_VALIDATION_REPORT.md`. Lister les PR ouvertes, leur base/tête, SHA, dépendances, conflits et CI.
2. **Choisir un seul chantier vérifiable**. Donner objectif, comportement actuel, résultat attendu, risques et invariants ; signaler les choix de protocole au responsable avant de coder.
3. **Créer ou utiliser une branche isolée** et un test de reproduction/régression. Ne pas effectuer de refonte opportuniste ou de changement silencieux du consensus.
4. **Implémenter petit**, avec des commits lisibles, puis lancer les contrôles ci-dessous et des essais ciblés.
5. **Contre-revue indépendante** : confier si possible à des sous-agents distincts les examens consensus/économie, réseau/DoS, wallet/stockage et intégration. Leur faire chercher activement les défaillances et écrire des reproductions, uniquement sur des instances locales ou autorisées. Ne pas exécuter de modifications concurrentes sur les mêmes fichiers. Sans sous-agents, effectuer une seconde revue séparée et déclarer la limite.
6. **Corriger et retester après chaque modification**. Toute preuve doit pointer vers le SHA exact ; distinguer test écrit, exécuté, réussi, non exécuté ou échoué.
7. **Ouvrir ou mettre à jour une PR en brouillon** avec code, tests, risques, compatibilité, références CI, dépendances et compte rendu des contre-revues.
8. **S'arrêter avant fusion**. Ne pas fusionner de PR, supprimer de branche, réécrire l'historique ou annoncer une release sans demande explicite du responsable. Après accord et intégration, relancer les tests sur le commit intégré.

## Contrôles de base

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
python3 scripts/testnet_smoke.py
```

Ajouter selon le chantier fuzzing, essais TCP adversariaux, partitions, wallet RPC, signature ML-DSA-65, stockage/reprise, économie, retarget, benchmarks et deux ordinateurs distincts. Ne jamais utiliser de clés ou fonds réels. Contrôler les arguments des scripts avant exécution.

**Gates critiques :** conservation des montants, limites de l'émission, nonces, signatures, rejets sans mutation, forks/reorgs, horodatages, résistance à la saturation, délais RPC, restauration après crash, compatibilité des versions, absence de régression. Une CI verte n'est pas un audit.

## Limites non négociables

Pas de modification non approuvée de genèse, émission, frais, consensus, propriété des fonds, profils de signature, encodage, versions incompatibles ou politique de récupération. Pas de fusion en cascade « parce que les PR précédentes sont vertes ». Signaler les vulnérabilités via `SECURITY.md`. Les concepts de confidentialité avancée, récupération sociale et gouvernance ne sont pas des fonctions mainnet livrées.

## Compte rendu de fin de tâche

Donner : `branche/commit de départ ; fichiers ; résultat ; nouvelle branche/commit ; PR et dépendances ; tests + SHA et liens ; sous-revues et reproductions ; limites et risques ; statut (sur branche / en revue / fusionné) ; prochaine étape proposée au responsable`.

La référence reste le **comportement réel du code sur le SHA ciblé**, et non une description historique ou une affirmation d'agent.

## Renommage de marque ARETOVUS (2026-10-31)

Ne pas modifier les séparateurs de domaine cryptographiques `AURORA/`, les constantes `AURORIDX`, les identités de genèse, les empreintes réseau, les anciens schémas `aurora://`, les chemins de stockage historiques ou les symboles monétaires de testnet sans migration de protocole distincte. Le dépôt et les branches historiques peuvent garder `AURORA` dans leurs URLs pour conserver les liens existants. Voir `docs/BRANDING_COMPATIBILITY.md`.

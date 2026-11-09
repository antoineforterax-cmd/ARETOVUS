# Instructions pour Codex — ARETOVUS

Ce fichier s'applique aux travaux sur cette branche. Guide complet de consolidation : [politique d'implémentation](docs/planning/IMPLEMENTATION_POLICY.md). **Lire le guide complet avant toute action Git susceptible de modifier les bases ou de fusionner des branches.** Une mise à jour du guide n'est pas une approbation automatique de ses propositions.

## Objet

ARETOVUS est une blockchain Rust **expérimentale** ; aucun fonds réel ni annonce de sécurité mainnet. Le responsable fixe la vision, les orientations architecturales et les décisions de protocole. Codex accélère l'implémentation, les tests, la revue et la documentation. Ne jamais inventer une approbation, une preuve, un test réussi ou un audit indépendant.

## Situation et branches — état expérimental natif à revalider

**Navigation :** [état du testnet](docs/01-demarrer/ETAT_ACTUEL.md), [décisions clarifiées](docs/05-pilotage/DECISIONS_CLARIFIEES.md), [feuille V2 #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178), [jalons historiques #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142). Toujours vérifier le SHA actuel.

- `main` contient le réseau expérimental **ARETOVUS natif** depuis la migration A3 du 1er novembre 2026. Cette identité de protocole est volontairement incompatible avec les anciennes données et clés.
- Les jalons A1–A4, B1–B4 et C1 disposent de preuves conservées dans le [suivi historique #142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142). Les workflows C2 [#166](https://github.com/antoineforterax-cmd/ARETOVUS/pull/166), C3 [#167](https://github.com/antoineforterax-cmd/ARETOVUS/pull/167) et le packaging partiel E2 [#168](https://github.com/antoineforterax-cmd/ARETOVUS/pull/168) ont été fusionnés le 2 novembre ; leur **périmètre borné** ne clôt pas automatiquement toute la phase.
- La décision préliminaire E3 [#169](https://github.com/antoineforterax-cmd/ARETOVUS/pull/169) est **NO-GO** pour un testnet public élargi, un mainnet ou de vrais fonds. La **revue E1 par une personne qualifiée extérieure au projet** est réservée à l'ultime étape à la demande du responsable ; elle n'a pas eu lieu. Les relectures du responsable et les contre-revues des agents sont des vérifications internes, à conserver distinctement.
- Les PR historiques #108/#109/#112/#115/#116/#124–#129/#131 ne doivent pas être fusionnées en cascade. Vérifier base, tête, SHA, conflits et CI avant de reconstruire les composants utiles, notamment les assistants LAN obsolètes.
- Toute affirmation de statut dans ce fichier est datée : **la version réellement sur `main`, ses tests et les issues actuelles priment**. Ne jamais substituer une preuve de branche ancienne à une preuve du réseau natif.

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

## Identité native ARETOVUS (intégrée au testnet expérimental)

La nouvelle identité ARETOVUS est intégrée dans `main` depuis A3. Elle distingue domaines de hachage, genèse, marqueurs de stockage, URI, wallet et outils du réseau antérieur. **Cette intégration ne signifie ni migration des soldes, ni sécurité mainnet.** Ne pas réécrire l'historique Git, supprimer les archives ou importer les clés/soldes de l'ancien réseau ; employer des dossiers de données neufs. Les décisions ultérieures de consensus, formats ou monnaie restent soumises à une approbation dédiée, des tests et une revue avant fusion. Lire [la migration de protocole](docs/03-recherche/PROTOCOL_IDENTITY_MIGRATION.md).

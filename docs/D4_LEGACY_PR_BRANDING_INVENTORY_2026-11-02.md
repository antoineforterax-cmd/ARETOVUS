# D4 — inventaire des PR historiques, identité active et archivage

**Date de référence : 2 novembre 2026.** Dépôt : `antoineforterax-cmd/ARETOVUS`. Analyse effectuée sur `main@059f2f07498ecd457edfdd6964c108d8136b8c64`, après #165, #166, #167 et #168. Ce document est un **inventaire de revue**, et non une fusion ou une fermeture de toutes les branches ci-dessous. Les SHA, bases et CI doivent être recontrôlés avant toute reprise.

## Identité et provenance

L'identité native opérationnelle est **ARETOVUS** : dépôt, crates, binaires, genèse, empreintes et formats réseau distincts du projet précédent (voir `docs/PROTOCOL_IDENTITY_MIGRATION.md` et `scripts/test_branding_compatibility.py`). Le libellé descriptif de `LICENSE` a été corrigé de l'ancien projet vers ARETOVUS ; **les mentions de copyright `AURORA contributors` dans `LICENSE` et `LICENSE-MIT` sont conservées délibérément** pour ne pas effacer des attributions historiques. Les droits d'origine doivent être examinés avant tout changement de crédits. Ne pas réécrire les commits Git, archives de tests ou états datés de 2026-10-29.

La recherche indexée GitHub montre encore des références anciennes dans `docs/consolidation/` ; il s'agit d'archives datées et non de noms d'outils natifs. La présence de l'ancien nom dans les textes d'une PR **non fusionnée** n'implique pas que le code opérationnel l'utilise. Les scripts actifs sont contrôlés par le test de branding. Cet inventaire ne prétend pas une disparition de chaque mention historique.

## PR encore ouvertes : disposition recommandée, pas de fusion automatique

| PR | État de base/dépendance au pointage | Disposition prudente |
| --- | --- | --- |
| [#108](https://github.com/antoineforterax-cmd/ARETOVUS/pull/108) | Conflit de fusion avec `main`, optimisation storage ancienne | **Bloquer** ; mesurer sur le code natif actuel et revérifier durabilité / symlinks avant éventuelle reconstruction. |
| [#109](https://github.com/antoineforterax-cmd/ARETOVUS/pull/109) | Projet de conception storage incremental, base `main`, sans exécution | **Conserver en étude** ; ne pas interpréter une note de conception comme un correctif. |
| [#112](https://github.com/antoineforterax-cmd/ARETOVUS/pull/112) | 32 768 blocs TCP, base `test/storage-cache-symlink-guard-20261031` | **Précurseur historique**, pas fusionnable en l'état dans le flux courant. |
| [#115](https://github.com/antoineforterax-cmd/ARETOVUS/pull/115) | 65 536 blocs TCP, même branche de base ancienne | **Précurseur historique**, éviter les duplications de workflow. |
| [#116](https://github.com/antoineforterax-cmd/ARETOVUS/pull/116) | Validation TCP 100 001 blocs sur ancien état, base `main` | **Supplantée pour l'identité native** par [#167](https://github.com/antoineforterax-cmd/ARETOVUS/pull/167) fusionnée ; préserver les logs originaux. |
| [#124](https://github.com/antoineforterax-cmd/ARETOVUS/pull/124) | Guide contributeurs/licences daté ; références à l'ancienne marque à relire | **Porter sélectivement** des clarifications après audit des attributions et test de marque. |
| [#125](https://github.com/antoineforterax-cmd/ARETOVUS/pull/125) | Navigation débutants, ancien texte de marque | **Rebaser/revoir** plutôt que fusionner tel quel. |
| [#126](https://github.com/antoineforterax-cmd/ARETOVUS/pull/126) | Empilée sur #125 ; inclut des instructions d'agent | **Bloquer fusion directe** : relire chaque règle d'agent, préserver les décisions de protocole natives. |
| [#127](https://github.com/antoineforterax-cmd/ARETOVUS/pull/127) | Assistant LAN Windows, utilise d'anciens noms de fichiers `AURORA-LAN` | **D3 en attente** ; adapter binaires/chemins ARETOVUS, vérifier commandes sans secrets. |
| [#128](https://github.com/antoineforterax-cmd/ARETOVUS/pull/128) | Empilée sur #127, assistant LAN macOS/Linux | **D3 en attente** ; reconstruire après #127 plutôt qu'intégrer directement. |
| [#129](https://github.com/antoineforterax-cmd/ARETOVUS/pull/129) | Campagne physique de 4 h inconclusive et diagnostics Windows | **Conserver les échecs** ; ne pas comptabiliser comme validation d'endurance C4. |
| [#131](https://github.com/antoineforterax-cmd/ARETOVUS/pull/131) | Empilée sur #129 ; supervision CPU/RAM 4 h | **Instrumentation candidate C4/D1**, à tester sur nouvelle base ; ne pas inventer 4 ou 72 h de résultats. |

## Critères de sortie D4 et contrôles

- [x] Distinguer nom actif et archives, et corriger le libellé public du fichier `LICENSE` sans effacer les copyrights historiques.
- [x] Inventorier les anciennes PR et leurs bases, documenter les conflits, la chaîne d'empilement et les remplacements connus.
- [ ] Réconcilier ou fermer les PR dépassées uniquement après décision et revalidation, sans suppression d'archives ni de commits.
- [ ] Harmoniser les assistants LAN #127/#128 et remplacer les noms opérationnels obsolètes après qualification multi-OS D3.
- [ ] Réviser `AGENTS.md` et la politique d'implémentation pour séparer clairement l'historique du 29 octobre des règles actuelles, avec approbation explicite des changements de protocole.
- [ ] Vérifier exhaustivement les références AURORA dans tous les **documents actifs** ; préserver les citations d'histoire, les licences et les résultats originaux.

**D4 ne peut donc pas être cochée comme terminée sur la seule base de cette PR.** Aucun changement de consensus, format natif, émission, clé ou règle de réseau. Tester les modifications documentaires par `scripts/test_branding_compatibility.py` et la CI au SHA exact. L'étape E3 demeure **NO-GO pour extension publique** tant que les lacunes C4/D1–D4/E1/E2 pertinentes ne sont pas levées.

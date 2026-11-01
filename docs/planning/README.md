# ARETOVUS — Documentation du projet (mise à jour le 29 octobre 2026)

**Statut : proposition documentaire pour revue.** Référence : branche expérimentale `feature/emission-schedule-v0`. Aucune modification de consensus ou de code n'est incluse.

Le [README public de `main`](https://github.com/antoineforterax-cmd/ARETOVUS) présente désormais l'état général du projet. Le testnet avancé reste sur `feature/emission-schedule-v0`, et les améliorations des PR #13 à #22 ne sont pas encore fusionnées. Les comptes rendus techniques et les résultats de CI se trouvent sur ces PR ; les tests réussis sur une branche ne prouvent pas que `main` contient la fonctionnalité.

## Ordre de lecture
1. [CAHIER_DES_CHARGES.md](CAHIER_DES_CHARGES.md) — finalité, exigences, limites.
2. [DECISIONS.md](DECISIONS.md) — décisions historiques, profil implémenté, propositions.
3. [ARCHITECTURE_CIBLE.md](ARCHITECTURE_CIBLE.md) — composants, dépendances, options de confidentialité.
4. [ROADMAP.md](ROADMAP.md) — séquence, critères de sortie, blocages.
5. [IMPLEMENTATION_POLICY.md](IMPLEMENTATION_POLICY.md) — méthode de modification contrôlée.

## Hiérarchie documentaire
- **Comportement réellement exécuté** : code et tests sur un commit identifié ; `docs/PROTOCOL.md` décrit ce comportement, sans le remplacer.
- **Paramètres testnet** : `docs/TESTNET_DECISIONS.md`, `docs/EMISSION_MODEL.md`.
- **Historique** : `docs/DECISION_REGISTER.md`, non à effacer.
- **Programme et propositions** : présent dossier. Il ne rend aucune proposition normative par sa seule existence.
- **Opérations et preuves** : `docs/TESTNET_OPERATIONS.md`, `docs/TESTNET_VALIDATION_REPORT.md`, CI et tickets.

En cas de divergence, ouvrir une tâche de réconciliation : ne jamais réécrire l'histoire ni faire passer une aspiration mainnet pour une règle du testnet. Le document Word historique évoqué dans les échanges n'a pas été retrouvé ; la comparaison mot à mot reste donc **à faire**, sans inventer son contenu.

## Situation des branches
`main` est un prototype plus ancien ; PR #1 est un brouillon vers `main` ; PR #2 (`feature/emission-schedule-v0`) est un brouillon vers la branche de PR #1. Le contenu de PR #2 ne doit pas être présenté comme déjà fusionné à `main`.

## Limites
Testnet expérimental uniquement, sans fonds réels, sans audit humain indépendant. Les essais locaux, CI et simulations ne démontrent ni sécurité mainnet ni anonymat absolu.

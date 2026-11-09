# ARETOVUS — Dossier de planification (historique et propositions)

> **Commencer ailleurs si vous découvrez le projet :** [documentation générale](../README.md), [état actuel du testnet](../01-demarrer/ETAT_ACTUEL.md), [décisions clarifiées](../05-pilotage/DECISIONS_CLARIFIEES.md), [licences](../02-participer/LICENCES_ET_DROITS.md). Ce dossier **ne crée aucune règle de consensus** et contient des documents datés. La feuille opérationnelle V2 est [#178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) ; l'issue [#142](https://github.com/antoineforterax-cmd/ARETOVUS/issues/142) conserve les précédents jalons.

**Statut : dossier de planification.** Les propositions anciennes et les règles du réseau natif actuel doivent être distinguées par leur date et leur commit. Cette page ne modifie aucune règle de consensus.

Le [README public de `main`](https://github.com/antoineforterax-cmd/ARETOVUS) et [le suivi V2 actif](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) présentent l'état actuel. Le réseau natif ARETOVUS est intégré à `main` depuis le 1er novembre 2026. Les résultats de CI historiques sur une ancienne branche ne prouvent pas le comportement du commit courant.

## Ordre de lecture
1. [CAHIER_DES_CHARGES.md](CAHIER_DES_CHARGES.md) — finalité, exigences, limites.
2. [DECISIONS.md](DECISIONS.md) — décisions historiques, profil implémenté, propositions.
3. [ARCHITECTURE_CIBLE.md](ARCHITECTURE_CIBLE.md) — composants, dépendances, options de confidentialité.
4. [Feuille V2 #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178) — travaux en cours et critères de sortie ; [ancienne feuille Git](https://github.com/antoineforterax-cmd/ARETOVUS/blob/caa94228b2fb769e3a5ea479ea5fd07485744489/docs/planning/ROADMAP.md) pour l'historique.
5. [IMPLEMENTATION_POLICY.md](IMPLEMENTATION_POLICY.md) — méthode de modification contrôlée.

## Hiérarchie documentaire
- **Comportement réellement exécuté** : code et tests sur un commit identifié ; `docs/PROTOCOL.md` décrit ce comportement, sans le remplacer.
- **Paramètres testnet** : `docs/TESTNET_DECISIONS.md`, `docs/EMISSION_MODEL.md`.
- **Historique** : `docs/DECISION_REGISTER.md`, non à effacer.
- **Programme et propositions** : présent dossier. Il ne rend aucune proposition normative par sa seule existence.
- **Opérations et preuves** : `docs/TESTNET_OPERATIONS.md`, `docs/TESTNET_VALIDATION_REPORT.md`, CI et tickets.

En cas de divergence, ouvrir une tâche de réconciliation : ne jamais réécrire l'histoire ni faire passer une aspiration mainnet pour une règle du testnet. Le document Word historique évoqué dans les échanges n'a pas été retrouvé ; la comparaison mot à mot reste donc **à faire**, sans inventer son contenu.

## Situation des branches
Les PR #1 et #2 ainsi que la branche `feature/emission-schedule-v0` sont des jalons historiques de l'ancien réseau. Vérifier le SHA et la compatibilité du protocole avant de reprendre leur code ; le réseau natif actuel est sur `main`.

## Limites
Testnet expérimental uniquement, sans fonds réels, sans revue E1 par une personne extérieure au projet. Les essais locaux, CI et simulations ne démontrent ni sécurité mainnet ni anonymat absolu.

# AURORA — Feuille de route à portes de validation

**Règle** : les phases peuvent avoir des recherches parallèles, mais une fonctionnalité critique ne franchit pas sa porte de validation sur la base d'une démonstration superficielle.

| Phase | Travaux | Porte de sortie |
|---|---|---|
| P0 — Réconciliation | Consolider documents, confronter le Word historique lorsqu'il est disponible, classifier chaque décision, résoudre contradictions sans toucher au consensus | Sources et statuts traçables |
| P1 — Noyau | Tests d'invariants économiques, signatures, horodatages, difficulté, réorganisations, décodage hostile | CI reproductible, incidents documentés |
| P2 — Résilience | Saturation workers (#4), horodatages adversariaux (#3), partitions, stockage plein/pannes, sauvegarde hors machine | Scénarios adversariaux passés et rapports |
| P3 — Réseau long | 100k+ synchronisation réseau (#5), plusieurs machines, endurance 72 h, convergence et mesures CPU/RAM/disque | Résultats mesurés, pas extrapolés |
| P4 — Confidentialité R&D | Modèle de menace, comparaison des modèles d'état et preuves, transports anonymisants, compatibilité PQ | Rapport de faisabilité et prototype isolé |
| P5 — Proposition de protocole | Version, invariants, migration, impacts économiques, activation et plan de retour | Décision explicite avant intégration |
| P6 — Testnet d'intégration | Implémentation isolée, tests unitaires/propriétés/fuzz, simulations multi-nœuds, réorganisations, benchmarks | Tests et revue du changement |
| P7 — Préparation mainnet éventuelle | Revue humaine indépendante, correction des constats, gouvernance, opérations, sécurité wallet et cadre juridique | Aucune annonce mainnet sans prérequis |

## Prochaines tâches prioritaires
1. Comparer les paramètres du testnet aux anciennes propositions et documenter les écarts.
2. Terminer les essais adversariaux #3 et #4 et l'essai réseau #5.
3. Faire des campagnes de fuzz plus longues, avec rapport de couverture.
4. Définir un protocole de mesure pour endurance 72 h, synchronisation, stockage et reorgs.
5. Ouvrir une étude de confidentialité indépendante du consensus existant.
6. Évaluer les limites de la trésorerie et de la récupération sans les activer.

## Ce qui n'est pas un succès de phase
- CI verte seule ≠ audit.
- 100 003 blocs rejoués depuis stockage ≠ 100k blocs synchronisés sur un réseau réel.
- 3 nœuds locaux quelques minutes ≠ réseau public stable.
- Signature ML-DSA ≠ blockchain entièrement post-quantique.
- Transport Tor ≠ transactions anonymes.

## Livrables attendus par phase
Spécification, commit/branche, tests reproductibles, métriques, limites connues, rapport de revue et décision de poursuite.

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


## Point de situation vérifié — 29 octobre 2026

**À lire avec la branche et le commit :** le README de `main` a été actualisé pour expliquer l'avancement, mais le code avancé du testnet reste sur `feature/emission-schedule-v0` et dans des PR empilées non fusionnées. Une CI verte sur une PR ne valide pas automatiquement `main`.

| Sujet | Réalisé / démontré | Encore à faire ou confirmer |
| --- | --- | --- |
| Base testnet | Comptes, signatures ML-DSA-65 v3, PoW SHA-256, émission plafonnée, stockage v6, TCP P2P, wallet CLI | Stabilisation, revue et intégration des branches |
| Publication | Guide français et release expérimentale Windows/Linux/macOS datée du 27/10 | Binaires signés, installation guidée, version stable, communication accessible |
| Horodatages et difficulté | Simulateur indépendant et vecteurs testés sur PR #14 | Horloges divergentes en réseau, partitions, réessais, clôture de #3 |
| Admission P2P | Dispatcher de préfaces et mesures hostiles sur PR #16 | Saturation après handshake, NAT partagés, essais Internet, clôture de #4/#8 |
| Signatures et mempool | Benchmark PR #17, cache borné PR #18, fuzz comptes PR #19, réorganisations PR #20/#21 | Revue, intégration et mesures de bout en bout ; mempool non persistant |
| Adresses | Parseur hexadécimal strict et tests sur PR #22 | Intégration et tests sur la branche de référence |
| Gros volumes | Test de stockage et rechargement de 100 003 blocs | Sync P2P 100k+ et charge prolongée sur machines réelles (#5) |
| Endurance | Test court réussi ; campagne 72 h lancée le 28/10 | Rapport final de 72 h et réplication multi-hôtes |
| Sécurité | CI, tests de propriétés, fuzz court, revue IA | Audit humain indépendant, campagnes longues, pannes réelles, tests de pairs hostiles |
| Confidentialité | Exigences et menaces documentées | Prototype isolé, comparaison cryptographique, mesures, décisions de consensus |
| Anti-perte / anti-arnaque | Wallet chiffré ; confirmation/aperçu #26, secours hors ligne #27, plafonds persistants #28, signature hors ligne #29 sur PR non fusionnées | Intégration, essais physiques, délais révocables, notifications, récupération sociale et tests de collusion |
| Gouvernance | 5 % d'émission crédités à la trésorerie de test | Contrôle, dépense, audit, politique et responsabilité |
| Applications | Wallet CLI | GUI, explorateur, site complet, mobile et intégrations |

### Ordre d'intégration conseillé

1. **Réconcilier les branches et leurs dépendances** : #1, #2, #12 et les PR #13 à #22 ; vérifier les bases exactes et les conflits avant chaque fusion.
2. **Tester l'ensemble intégré** : formatage, Clippy, compilation, tests Rust, smoke multi-nœuds, essais de réorganisation et de récupération, puis comparer les résultats à chaque PR.
3. **Clore les risques P0 par preuves** : #3, #4 et #5, endurance, partitions, stockage et réseau sous charge.
4. **Geler un testnet de référence** : version, commit, compatibilité, binaires et instructions d'installation vérifiables.
5. **Étudier sans activer prématurément** : confidentialité native, protection des fonds, gouvernance, évolution du PoW et post-quantique complet.
6. **Après validation des dépendances** : explorer, interface graphique, applications et réseau public expérimental.
7. **Avant toute annonce mainnet** : audit humain indépendant, corrections vérifiées, opérations et cadre juridique.

### Points de vigilance documentaires

- Le numéro historique `v0.0.1` ne désigne pas la maturité actuelle du projet. La release `testnet-v0.0.1-experimental.1` reste une archive datée et liée à son commit, à ne pas supprimer ni présenter comme dernière version du code.
- Ne pas confondre **fonction implémentée sur une PR**, **fonction fusionnée**, **test passé sur un commit**, **fonctionnalité disponible dans la release** et **fonctionnalité simplement proposée**.
- Les anciennes décisions de comparaison UTXO/comptes et les anciennes règles monétaires restent des traces historiques, pas la spécification du testnet actuel.
- Toute fonctionnalité qui modifie les droits de propriété, l'offre, la validité des transactions ou les formats réseau nécessite décision explicite, versionnement et essais de migration.

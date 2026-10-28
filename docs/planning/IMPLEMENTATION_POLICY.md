# AURORA — Politique d'implémentation contrôlée

## 1. Préparer
- Une issue par changement cohérent : problème, périmètre, risques, invariants et critères d'acceptation.
- Identifier si la modification touche le consensus, le stockage, le P2P, la crypto, l'économie ou seulement la documentation.
- Pour un changement de consensus : proposition écrite et accord explicite **avant** de changer les règles.
- Épingler branche et SHA de référence ; sauvegarder les données de test.

## 2. Construire
- Branche isolée, petits commits lisibles, interfaces et formats versionnés.
- Pas de refonte générale opportuniste ; pas de dépendance cryptographique ajoutée sans analyse.
- Ne jamais modifier les montants, signatures, émissions ou règles d'admission silencieusement.
- Ne pas mélanger expérimentation de confidentialité et corrections de stabilité dans une même PR.
- Pour tout nouveau profil incompatible : réseau et data-dir distincts, plan de migration explicite.

## 3. Vérifier
- `cargo fmt --check`, `cargo clippy`, `cargo test`, build, tests d'intégration et multi-nœuds pertinents.
- Tests de propriétés et cas adversariaux (fuzz, overflow, encodage non canonique, reorg, timestamp, partitions, saturation).
- Mesures de CPU, RAM, stockage, bande passante et temps de vérification quand les coûts changent.
- Comparaison avant/après et preuve qu'aucune règle hors périmètre n'a changé.
- Tests de sauvegarde, reprise et compatibilité lorsque stockage ou réseau sont touchés.

## 4. Revoir et fusionner
- PR avec résumé, risques, résultats CI, métriques, limites, liens vers décisions.
- Revue humaine spécialisée pour toute prétention de sécurité cryptographique ou financière.
- Validation du porteur pour toute décision de protocole ; ne pas inférer cette validation d'un commit ou d'un message ambigu.
- Ne pas fusionner une PR en brouillon ; traiter les échecs avant la fusion.
- Actualiser le registre et les documents de protocole **après** confirmation du comportement.

## 5. Règles de publication
Testnet = expérimental, aucun fonds réels. Pas de promesse d'anonymat, d'invulnérabilité, de rendement ou de résistance quantique totale. Une version mainnet exige un examen indépendant et des critères opérationnels explicites.

## 6. Traçabilité des changements
Modèle d'issue : `Objectif / Décision concernée / Fichiers / Invariants / Compatibilité / Menaces / Tests / Benchmarks / Résultats / Revue / Statut`.

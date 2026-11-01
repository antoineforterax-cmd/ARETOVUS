# THREAT_MODEL.md — ARETOVUS

> Archive : décrit l’ancien prototype v0.0.1 au moment de sa rédaction. Pour les formats actuels, voir [CONSENSUS.md](CONSENSUS.md) et [PROTOCOL.md](PROTOCOL.md); pour les preuves, voir [TESTNET_VALIDATION_REPORT.md](TESTNET_VALIDATION_REPORT.md).

**Statut : version initiale, aucune évaluation chiffrée.** Le cahier des charges interdit
d'attribuer un score sans méthode de notation documentée. Les colonnes « Probabilité » et « Impact »
sont donc à remplir **après** validation de l'échelle ci-dessous, et chaque ligne reste « Non évalué »
d'ici là. Aucune mesure listée ici n'est démontrée efficace : une mesure « prévue » n'est pas une mesure « testée ».

## 1. Échelle proposée (à valider par le porteur)

Probabilité et impact : **Faible / Moyen / Élevé**, justifiés par un scénario écrit. Risque = lecture
croisée des deux, pas un nombre. Réévaluation à chaque phase et après chaque incident ou résultat de test.

## 2. Menaces du cahier (§21)

Colonnes : mesure prévue · test requis · état de la mesure.

| Menace | Mesure prévue | Test requis | État |
|---|---|---|---|
| Double dépense | Consensus, confirmations, travail cumulé | Tests de forks et de réorganisations | Non implémenté |
| Création monétaire excessive | Validation des récompenses, plafond vérifié | Recalcul indépendant de l'émission | Non implémenté (calcul indicatif seulement) |
| Signature falsifiée | Bibliothèque établie, vecteurs officiels | Vecteurs ; signature altérée rejetée | Non implémenté |
| Compromission d'une clé | Stockage sécurisé, sauvegardes | Revue ; tests de restauration | Non implémenté |
| Attaque de majorité PoW | Simulation, confirmations, diversité du minage | Simulations 10/25/40/50 % | Non implémenté |
| Manipulation des horodatages | Règles temporelles déterministes | Tests d'horodatages manipulés | Non implémenté |
| Déni de service | Quotas, limites avant calcul coûteux | Tests adversariaux, fuzzing | Partiel : bornes de longueur dans le codec (écrites, non exécutées) |
| Spam transactionnel | Frais, quotas | Tests adversariaux | Non implémenté |
| Faux nœud distant | Authentification, vérification indépendante | Tests d'accès | Non implémenté |
| Vol du téléphone | Verrouillage, permissions, révocation | Tests de révocation | Non implémenté |
| Compromission du PC | Isolation, droits minimaux, mises à jour | Revue | Non implémenté |
| Fuite des journaux | Expurgation des secrets | Revue ; recherche de secrets | Non implémenté |
| Corruption du stockage | Vérification, sauvegardes | Tests de panne | Non implémenté |
| Dépendance vulnérable | `cargo audit`, `cargo deny`, suivi | Exécution en CI | Prévu (aucune CI exécutée) ; dépendance unique `sha2` |
| Migration mal gérée | Procédure formelle, testnet | Essai sur testnet | Non implémenté |
| Récupération frauduleuse | Conditions explicites, multisignature | Analyse + tests | Non implémenté |
| Phishing | Vérification d'adresse, avertissements | Tests d'interface | Non implémenté |
| Erreur de portefeuille léger | Preuves vérifiables, avertissements | Tests | Non implémenté |

## 3. Menaces propres au prototype v0.0.1

| Menace | Mesure dans le code | Test écrit (non exécuté) | Limite |
|---|---|---|---|
| Longueur annoncée mensongère (allocation démesurée) | Longueurs bornées ; octets vérifiés avant copie | `lying_length_does_not_allocate_or_succeed`, `excessive_length_claims_are_rejected` | Bornes arbitraires (FEE-02/03) |
| Encodage non canonique (malléabilité) | Largeur fixe, rejet des octets en trop et des valeurs inconnues | `mutated_encodings_stay_canonical_or_fail`, `roundtrip_random_transactions` | Prouve peu : générateur maison, pas de réduction des contre-exemples |
| Débordement de montant | Addition vérifiée, plafond de garde | tests `amount` | Le plafond n'est pas une règle d'émission |
| Confusion entre réseaux | Octet de réseau dans transaction et bloc | `network_mismatch_is_rejected` | Aucune signature ne lie encore le réseau (CRY-01 ouvert) |
| Collision de domaines de hachage | Tag de domaine préfixé avec sa longueur | `same_data_different_domain_gives_different_hash` | Construction non revue (TX-03 ouvert) |
| Autorisation absente | — | — | **`auth_data` n'est pas vérifié : n'importe qui « dépenserait » n'importe quoi.** Aucune sécurité des fonds. |
| Engagement ambigu sur les transactions | Nombre de transactions et identifiants de taille fixe | `adding_a_transaction_after_build_is_detected`, `order_of_transactions_changes_the_commitment` | Pas de preuve d'appartenance (pas de Merkle) |
| Erreur de logique non détectée | Vecteurs indépendants Python | `tests/vectors.rs` | Le code n'a **pas été compilé ni testé** |

## 4. Risques résiduels connus

Tout le consensus, la cryptographie de signature, le réseau, le stockage et le portefeuille sont absents. Le
prototype ne protège aucun fonds et ne doit jamais en contenir.

## 5. Journal

| Date | Changement |
|---|---|
| 2026-10-25 | Création : menaces du cahier + menaces du prototype, sans scores |

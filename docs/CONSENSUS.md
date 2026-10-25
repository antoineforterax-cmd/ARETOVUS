# CONSENSUS.md — AURORA

**Statut : aucune règle de consensus n'est validée.** Ce document liste ce qui est défini dans le
prototype v0.0.1 (hypothèses), ce qui reste à spécifier, et les invariants qu'un futur consensus
devra respecter. Les décisions sont suivies dans `DECISION_REGISTER.md` ; en cas de désaccord, le
registre fait foi.

## 1. Ce que le code fait aujourd'hui

| Sujet | Comportement du prototype | Statut | Référence |
|---|---|---|---|
| Montants | Entiers 64 bits non signés, 8 décimales, plafond de garde 105 000 000 AUR | Hypothèse | TX-06, TX-07, MON-01 |
| Encodage | Entiers big-endian largeur fixe ; longueurs bornées ; rejet des octets en trop | Hypothèse | TX-02 |
| Hachage | SHA-256 de `longueur(tag) ‖ tag ‖ données`, trois domaines | Hypothèse | TX-03 |
| Identifiant de transaction | Hachage de l'encodage complet, `auth_data` inclus | Hypothèse | TX-05 |
| Engagement du bloc | Hachage de `nombre ‖ identifiants` (pas un Merkle) | Hypothèse | TX-04 |
| Validation d'un bloc | **Sans état seulement** : réseau identique, transactions valides sans état, pas de doublon, engagement correct | Hypothèse | — |

## 2. Ce que le code ne fait PAS (donc aucun consensus)

Preuve de travail, difficulté, cible, travail cumulé, sélection de chaîne, horodatages, récompenses,
émission, maturité, existence des fonds (état), signatures, doubles dépenses entre blocs, frais,
taille maximale en octets, réorganisations, réseau.

## 3. Règles à spécifier (par ordre de la chaîne de dépendances du registre)

1. Modèle de transaction (TX-01) puis format canonique définitif (TX-02), hachage (TX-03), Merkle (TX-04).
2. Signatures et format d'adresse (CRY-01 à CRY-06, TX-08).
3. Poids et capacité (FEE-01 à FEE-03).
4. PoW et en-tête (POW-01 à POW-03).
5. Difficulté et règles temporelles (POW-04 à POW-07) ; sélection de chaîne (POW-08) ; maturité (POW-09).
6. Unité, calendrier d'émission, arrondis, plafond (TX-06, MON-01 à MON-04) ; genèse (MON-05).

Pour chaque règle : formule exacte, bornes, arrondis, cas limites, vecteurs de test indépendants.

## 4. Invariants que tout consensus futur doit garantir (à transformer en tests)

- **Déterminisme :** deux nœuds avec le même historique valide calculent le même état et la même difficulté.
- **Aucune création monétaire hors règle :** la somme émise ne dépasse jamais le calendrier d'émission.
- **Canonicité :** si une donnée est acceptée, son encodage canonique est exactement les octets reçus.
- **Aucune dépense non autorisée, aucune double dépense** dans la chaîne active.
- **Chaîne choisie :** la chaîne valide au plus grand travail cumulé, pas la plus longue.
- **Séparation des réseaux :** une donnée d'un réseau n'est pas valide sur un autre.

## 5. Critères de décision du PoW avant tout testnet public

Voir `DECISION_REGISTER.md`, fiche F-02 (six conditions). Une simulation ne garantit pas la sécurité réelle.

## 6. Journal

| Date | Changement |
|---|---|
| 2026-10-25 | Création : état du prototype v0.0.1, aucune règle validée |

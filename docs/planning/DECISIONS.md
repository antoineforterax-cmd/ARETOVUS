# AURORA — Registre de consolidation des décisions

**État au 29 octobre 2026 :** le testnet avancé se trouve sur `feature/emission-schedule-v0`. Les PR #13 à #22 apportent des tests et correctifs supplémentaires encore en attente de fusion. `Implémenté testnet` ne signifie pas `validé mainnet`. Ce document ne change pas le statut d'approbation du porteur de projet. Les décisions historiques demeurent dans `docs/DECISION_REGISTER.md`.

| ID | Sujet | État vérifiable | Statut de gouvernance |
|---|---|---|---|
| D-01 | Rust, quatre crates principales | Présents dans le dépôt | Implémenté expérimentalement |
| D-02 | Modèle comptes + nonce | Testnet v3 | Implémenté expérimentalement |
| D-03 | Signatures ML-DSA-65 | Testnet v3 ; SHA-256 reste utilisé | Implémenté expérimentalement |
| D-04 | PoW SHA-256, 60 s, retarget borné | Testnet | Implémenté expérimentalement |
| D-05 | 500 M AUR, 8 décimales, genèse vide | Testnet | Implémenté expérimentalement ; mainnet non arrêté |
| D-06 | Émission proportionnelle décroissante | Formule `min(reliquat,max(1,floor(reliquat/1_000_000)))` en unités minimales | Implémenté expérimentalement |
| D-07 | 5 % trésorerie / 95 % mineur | Compte testnet fixe sans politique de dépense | Implémenté expérimentalement |
| D-08 | Frais 50 % mineur / 50 % brûlés | Burn sans réouverture du plafond | Implémenté expérimentalement |
| D-09 | TCP, pairs configurés, découverte limitée | Testnet | Implémenté expérimentalement |
| D-10 | DID, gardiens, récupération | Conception documentaire | Non implémenté |
| D-11 | Tor, I2P, Dandelion++ | Idées d'architecture | À étudier |
| D-12 | Adresses furtives, montants cachés, ZK | Idées d'architecture incompatibles avec une simple extension du modèle de comptes actuel | À étudier |
| D-13 | Minage plus accessible, PoW memory-hard | Recherche antérieure | À étudier |
| D-14 | Wallet matériel, mobile, explorer | Ambitions et travaux futurs | Non livrés |
| D-15 | Revue indépendante | Non effectuée ; développement expérimental poursuivi | Prérequis avant sécurité mainnet |
| D-16 | 100k+ synchronisation réseau, endurance 72 h | Non démontrés par les essais locaux bornés | À vérifier |
| D-17 | Confidentialité obligatoire ou facultative | Non tranchée | Ouvert, ne pas supposer |

## Règles de tenue
Pour toute nouvelle décision : date, auteur/approbateur, problème, alternatives, impacts de consensus, sécurité, performances, compatibilité, plan de test, état (`proposé`, `à expérimenter`, `implémenté expérimentalement`, `validé`, `abandonné`) et liens vers commits/tests. Seul un accord explicite peut porter une proposition au statut `validé`.

## Incohérences à traiter
1. Anciennes ambitions de plafond (dont ~105 M) versus 500 M codés dans le testnet.
2. Objectif post-quantique versus SHA-256 et composants classiques du protocole.
3. Objectif confidentialité forte versus état actuel public par comptes.
4. Objectif mineurs particuliers versus PoW SHA-256 non spécifiquement anti-ASIC.
5. Trésorerie créditée versus absence de politique de dépense.
6. Préparation mainnet versus absence de revue indépendante et d'endurance multi-hôtes.

**Ne pas résoudre ces conflits en modifiant silencieusement le code.**

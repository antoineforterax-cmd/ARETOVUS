# AURORA — vision, orientation et documentation

**État documentaire au 31 octobre 2026**, à partir de `main` au commit `b76e0cff6d7e2ee6527cac2506ad005d7360ec4f`. Cette page est une **navigation**, pas une nouvelle règle de consensus. Toujours vérifier le SHA consulté : le dépôt continue d'évoluer.

## En quelques mots

Nous développons un **testnet de blockchain indépendant en Rust**, avec preuve de travail, transactions signées, stockage persistant, réseau P2P et portefeuille en ligne de commande. Le testnet avancé est **déjà intégré à `main`** ; les anciennes présentations le décrivant comme uniquement présent sur `feature/emission-schedule-v0` sont datées. Ce n'est pas un mainnet sûr pour des fonds réels.

## Notre direction : trois objectifs dans l'ordre

| Ordre | But | Où en est-on ? |
| --- | --- | --- |
| **1. Fiabilité et autonomie** | Consensus déterministe, stockage récupérable, P2P robuste, wallet testable, contrôles et auditabilité | Base expérimentale présente ; campagnes adversariales et audit indépendants encore nécessaires |
| **2. Utilisation simple** | Documentation accessible, installations reproductibles, explorateur en lecture seule, puis wallet graphique | Guide français et CLI présents ; explorateur et GUI non livrés |
| **3. Différenciation** | Vie privée du réseau et des transactions ; récupération volontaire ; possibilité de participer avec du matériel courant | **Recherche à mener**, pas de confidentialité native ni de coffre consensuel aujourd'hui |

**Règle de qualité :** un noyau plus petit, lisible et mesurable est préférable à beaucoup de code insuffisamment revu. Ne pas élargir le consensus pour faire une démonstration.

## Quel document lire ?

| Besoin | Source de référence |
| --- | --- |
| Première découverte | [Guide débutant en français](../guide-fr/README.md) |
| Pourquoi AURORA et quelles ambitions | [Cahier des charges](CAHIER_DES_CHARGES.md) |
| Quel travail ensuite | [Feuille de route technique](ROADMAP.md) et [issue produit #123](https://github.com/antoineforterax-cmd/AURORA/issues/123) |
| Qu'est-ce qui est choisi / encore ouvert ? | [Décisions actuelles et arbitrages](DECISIONS.md) |
| Comment le code est organisé | [Architecture du dépôt](../../ARCHITECTURE.md) et [architecture cible de recherche](ARCHITECTURE_CIBLE.md) |
| Quelles sont les **règles exécutées** | [Protocole](../PROTOCOL.md), [consensus](../CONSENSUS.md), [paramètres économiques](../TESTNET_DECISIONS.md) |
| Qu'est-ce qui est réellement prouvé | [Rapport de validation](../TESTNET_VALIDATION_REPORT.md), [critères de finalisation](../FINALIZATION_CHECKLIST.md) |
| Comment contribuer sans casser le projet | [CONTRIBUTING](../../CONTRIBUTING.md), [AGENTS](../../AGENTS.md), [politique d'implémentation](IMPLEMENTATION_POLICY.md) |
| Historique des propositions | [Registre historique](../DECISION_REGISTER.md) et [anciennes hypothèses](../DECISION_BASELINE.md) |

## Comment lire les décisions et les preuves ?

**Une idée** n'est pas une fonction, **une fonction codée** n'est pas nécessairement testée, **une CI verte** n'est pas un audit, et **un choix de testnet** n'est pas une décision de mainnet. Les anciens documents datés sont conservés comme archives : leurs sections évoquant les PR #1/#2 et la branche `feature/emission-schedule-v0` ne représentent plus à elles seules l'état de `main`.

Pour tout changement au consensus, à l'émission, à l'identité des comptes ou à la propriété des fonds : fiche motivée, tests et simulations, compatibilité/migration, revue, puis **accord explicite du responsable**. Pas de décision implicite à partir d'une documentation.

## Ce qu'on ne promet pas

Pas d'anonymat absolu, de résistance totale aux attaques, de minage rentable garanti, de fonds récupérables après n'importe quel vol, ni de date garantie de mainnet. L'audit humain indépendant et les preuves en conditions hostiles restent essentiels.

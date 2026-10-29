# Comparaison préliminaire des modèles d'état — AURORA

**Statut : comparaison historique de phase 0, conservée pour référence.**

**État vérifié le 29 octobre 2026 :** le testnet de la branche
`feature/emission-schedule-v0` utilise des comptes avec solde et nonce, des
transactions v3 ML-DSA-65 et des transitions atomiques. La mention historique
« TX-01 À COMPARER » ne décrit donc plus le code exécuté. Voir
[CONSENSUS.md](CONSENSUS.md), [ARCHITECTURE.md](../ARCHITECTURE.md) et
[le registre de comptes](../crates/aurora-core/src/state.rs).

Cette date est celle du constat documentaire, pas une date inventée de décision
du porteur. La comparaison ci-dessous reste utile pour une recherche future,
notamment sur la confidentialité ; elle n'autorise aucune migration du modèle
d'état ni modification du consensus actuel.

## 1. Ce qu'on compare

Un modèle d'état décrit comment le réseau sait quels fonds existent, à qui ils appartiennent et s'ils ont déjà été dépensés. Le consensus doit appliquer exactement les mêmes règles sur chaque nœud.

Deux familles courantes sont comparées :

- **UTXO** : les fonds sont représentés par des sorties non dépensées. Une transaction consomme des sorties existantes et en crée de nouvelles. C'est la famille utilisée par Bitcoin.
- **Comptes** : le réseau conserve un état par compte, notamment un solde et généralement un compteur de séquence (*nonce*) pour ordonner les transactions. C'est la famille utilisée par Ethereum pour les comptes externes.

Ces descriptions sont simplifiées : il existe des variantes et des modèles hybrides. Ce document ne présume pas qu'une variante est adaptée à AURORA.

## 2. Comparaison fonctionnelle

| Critère | UTXO | Comptes |
|---|---|---|
| Représentation des fonds | Ensemble de sorties dépensables | Solde enregistré par compte |
| Dépense double | Rejeter une sortie déjà consommée ; vérifier aussi les conflits au sein d'un bloc et de la chaîne | Débiter un solde valide une seule fois et vérifier le nonce ; gérer correctement l'ordre des transactions |
| Transactions concurrentes | Deux dépenses d'UTXO distinctes peuvent souvent être validées indépendamment | Transactions d'un même compte sont généralement liées par le nonce |
| Modèle mental du portefeuille | Plusieurs sorties peuvent composer le solde affiché | Solde directement associé au compte |
| Rendu de monnaie | La transaction crée des sorties pour le destinataire et la monnaie rendue | Le débit et le crédit modifient les soldes |
| Confidentialité de base | Peut limiter certains liens si les sorties sont bien gérées, mais le graphe des transactions reste observable | Les mouvements associés à un compte sont faciles à relier entre eux |
| Taille / complexité de l'état | Indexer les sorties non dépensées ; les sorties dépensées peuvent être retirées de l'ensemble actif | Conserver les comptes et leurs soldes/nonces |
| Preuves d'état | Des structures d'engagement sur les UTXO sont possibles, avec des compromis de taille et de preuve | Des arbres d'état de comptes sont possibles, avec des compromis de taille et de preuve |
| Frais | Peut dépendre des octets, des entrées et du travail de validation | Peut dépendre des octets et du travail d'exécution/validation |
| Récupération et portefeuille | Le portefeuille doit retrouver et suivre les sorties contrôlées par ses clés | Le portefeuille suit les comptes et leur état |
| Contrats programmables | Possible, mais nécessite un modèle de script/exécution défini séparément | Souvent associé à une machine d'exécution par comptes ; ajoute une grande surface de complexité |
| Risques d'implémentation | Gestion des entrées, sorties, monnaie rendue, conflits et historique | Gestion des nonces, des soldes, des ordres et des mises à jour atomiques de l'état |

Aucun modèle ne garantit à lui seul la confidentialité, des frais faibles, une meilleure sécurité ou une plus grande capacité. Ces propriétés dépendent aussi des formats, règles de consensus, structures de données, signatures et politiques réseau.

## 3. Questions de sécurité à résoudre pour les deux modèles

1. **Autorisation** : quelle signature ou preuve démontre que l'émetteur a le droit de dépenser ?
2. **Validité des fonds** : comment le nœud vérifie-t-il qu'ils existent et sont disponibles ?
3. **Double dépense** : comment les conflits sont-ils rejetés dans une même transaction, entre transactions d'un bloc, entre blocs concurrents et après une réorganisation ?
4. **Conservation monétaire** : comment prouver que la valeur créée ne dépasse jamais la valeur autorisée, en tenant compte des frais et des récompenses ?
5. **Rejeu** : comment empêcher qu'une transaction soit rejouée sur un autre réseau ou dans un autre contexte ?
6. **Déterminisme** : deux nœuds qui partent du même état et reçoivent les mêmes blocs doivent calculer exactement le même état.
7. **Récupération après incident** : comment reconstruire ou vérifier l'état après redémarrage, corruption, synchronisation ou réorganisation ?
8. **Tests adversariaux** : quelles propriétés et invariants sont vérifiés par des tests de cas limites, tests différentiels et fuzzing ?

## 4. Proposition historique de prototype comparatif

La proposition initiale était de construire deux petites simulations séparées avant de retenir un modèle. Elle est conservée comme piste de recherche, sans constituer une tâche préalable au développement du testnet actuel :

- **Simulation UTXO** : créer des fonds de départ fictifs, transférer une partie, rendre la monnaie, essayer une double dépense et réorganiser un petit historique.
- **Simulation comptes** : créer les mêmes soldes fictifs, effectuer les transferts correspondants, tester les nonces invalides, les soldes insuffisants et une réorganisation.
- **Scénarios communs** : transactions valides/invalides, overflow, frais, conflits, charge de validation, taille de l'état, reconstruction après redémarrage simulé.
- **Mesures** : durée et mémoire sur une machine précisée, taille sérialisée, coût des vérifications, complexité de la logique et nombre de cas limites à couvrir.
- **Limites** : les mesures d'un micro-prototype ne prouvent ni la sécurité d'un protocole complet ni la performance d'un réseau réel.

Les résultats doivent être reproductibles : code, commandes, paramètres, matériel et résultats sont publiés ensemble. Ne pas ajouter de signature maison ni de dépendance cryptographique expérimentale pour cette comparaison.

## 5. Ce que ce document ne décide pas

Ce document ne choisit pas :
- UTXO, comptes ou une variante hybride ;
- les signatures, l'adresse, le hachage ou l'émission ;
- les frais, la confidentialité ou les contrats ;
- les paramètres de consensus.

**Suite actuelle :** éprouver les invariants du modèle par comptes déjà implémenté. Toute alternative UTXO ou hybride nécessite une étude séparée, une décision explicite et un plan de compatibilité ; ce document historique ne remet pas ce choix en attente.

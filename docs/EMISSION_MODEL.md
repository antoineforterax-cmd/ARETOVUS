# AURORA — profil d’émission `fast_start_500`

**État : implémenté pour la testnet expérimentale de la branche `feature/emission-schedule-v0`.** Ce document décrit les constantes exécutées dans `aurora-consensus/src/reward.rs`; il ne constitue pas une promesse de valeur ou une décision de lancement mainnet.

## Unités et plafond

- Précision : 8 décimales; 1 AUR = 100 000 000 unités.
- Plafond brut total : 500 000 000 AUR.
- L’offre brute historique compte la genèse et les récompenses d’émission; une destruction de frais ne libère jamais de nouveau plafond.
- La récompense est calculée à partir de l’offre brute au début du bloc et tronquée au reliquat exact. Si le plafond est atteint, aucune nouvelle émission n’a lieu.

## Calendrier par offre émise

| Offre brute avant bloc | Émission brute maximale |
|---:|---:|
| moins de 5 000 000 AUR | 500 AUR/bloc |
| moins de 25 000 000 AUR | 100 AUR/bloc |
| moins de 75 000 000 AUR | 60 AUR/bloc |
| moins de 175 000 000 AUR | 30 AUR/bloc |
| moins de 350 000 000 AUR | 10 AUR/bloc |
| moins de 500 000 000 AUR | 1 AUR/bloc |
| 500 000 000 AUR ou plus | 0 AUR |

Le seuil est évalué en unités entières; le palier s’applique au premier bloc dont l’offre initiale est égale ou supérieure au seuil. Le dernier bloc est limité au reliquat et ne peut dépasser le plafond.

## Répartition et frais

Pour une émission `R` en unités minimales :

```text
treasury = floor(R / 20)   # 5 %
miner_issuance = R - treasury
```

Les unités résiduelles de la division restent au mineur. Pour les frais validés `F` :

```text
miner_fees = floor(F / 2)
burned_fees = F - miner_fees
```

Le compte trésorerie testnet est fixe (`[0xA0; 32]`). Le mineur est une identité publique engagée dans l’en-tête via l’engagement de bloc. Les récompenses et frais sont appliqués atomiquement après validation du lot de transactions; un paiement est requis lorsqu’une récompense ou des frais sont dus. Les frais brûlés ne sont pas crédités à l’offre brute.

## Réorganisations et contrôles

Chaque bloc est évalué depuis l’état de son parent, y compris l’offre brute et la récompense correspondante. Une branche secondaire porte son propre état; lors d’une réorganisation, le nouveau chemin gagnant détermine soldes et offre canoniques. Il n’y a pas de maturité de récompense dans le prototype.

Les tests couvrent les seuils, le plafond et la dernière unité, le partage des frais impairs, l’engagement du mineur, l’absence de paiement de mineur et la réorganisation. L’intégration n’est pas une validation indépendante de la politique monétaire; le profil reste limité aux expériences testnet.

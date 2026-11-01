# 2. Les règles économiques exécutées

[Sommaire](README.md) · [← Comprendre](01-comprendre.md) · [Installation →](03-installation.md)

Les règles ci-dessous sont celles du testnet de référence. Elles sont exprimées en nombres entiers; le code n’utilise pas de montant monétaire à virgule flottante.

## Plafond et émission

La genèse ne distribue aucun jeton : il n’y a pas de préminage dans ce profil. Le plafond d’émission brute est de **500 000 000 ARETOVUS**. Un ARETOVUS vaut **100 000 000 unités minimales**; la plus petite unité vaut donc `0.00000001 ARETOVUS`.

À chaque bloc, le protocole calcule l’émission à partir du reliquat jamais émis :

```text
reliquat = plafond - émission_brute_cumulée
récompense = min(reliquat, max(1, floor(reliquat / 1_000_000)))
```

Tous ces nombres sont en unités minimales. Le premier bloc émet **500 ARETOVUS**. Le second en émet **499,99950000 ARETOVUS**. La récompense diminue avec le reliquat et les arrondis entiers; il n’y a pas de halving programmé. Le plancher d’une unité permet de finir par épuiser le reliquat. Quand celui-ci vaut zéro, la récompense est zéro.

Il faut distinguer l’**émission brute cumulée** des unités encore présentes dans les soldes. Le burn réduit ces dernières, mais ne rouvre jamais de place sous le plafond d’émission. Une unité brûlée n’est pas réémise grâce à ce burn.

## Mineur, trésorerie et frais

| Flux | Règle |
| --- | --- |
| Nouvelle émission | `floor(récompense / 20)` pour la trésorerie, le reste pour le mineur. |
| Frais de transaction | `floor(frais / 2)` pour le mineur, le reste détruit. |
| Après le plafond | Aucune nouvelle émission; le mineur reçoit toujours sa part des frais. |

Pour le premier bloc sans transaction, le mineur reçoit **475 ARETOVUS** et la trésorerie **25 ARETOVUS**. Avec des frais de `0.01 ARETOVUS`, le mineur reçoit `0.005 ARETOVUS` et `0.005 ARETOVUS` sont détruits. Si des frais valent trois unités minimales, une unité va au mineur et deux sont brûlées. L’arrondi se fait à l’unité minimale.

Le burn est une opération comptable du protocole. Les unités ne sont pas mises en attente chez un acheteur et ne sont pas versées dans une réserve échangeable. Son taux est fixe dans ce profil; aucun taux dynamique lié à la charge n’est actif.

La trésorerie est actuellement un compte fixe de test : `[0xA0; 32]`. Le protocole le crédite, mais ne définit ni contrat de gouvernance, ni signataires, ni voie de dépense. Ces unités ne constituent donc pas un budget disponible pour financer le projet. Une règle de dépense demanderait une spécification et une activation distinctes.

## Soldes et branches concurrentes

Chaque branche est validée à partir de l’état de son parent. Lorsqu’un nœud change de branche, les transactions, les récompenses, les frais et le burn suivent la branche retenue. Additionner les récompenses de toutes les branches serait incorrect : des blocs connus peuvent ne pas appartenir à la chaîne canonique.

Le prototype ne prévoit pas de période de maturité différée pour les récompenses. Cela n’en fait pas un système destiné à des fonds réels : une récompense peut appartenir à une branche ensuite abandonnée.

## Ce que ces règles ne promettent pas

Un plafond et un burn ne garantissent ni rareté économique utile, ni hausse de prix, ni revenu suffisant pour les mineurs. Ces résultats dépendraient d’usages, de demande, de coûts et de sécurité qui ne sont pas établis par le testnet.

Aucun mécanisme actif ne confisque ou recycle les comptes inactifs. Aucun gardien ne peut récupérer une clé perdue. Ces propositions touchent aux droits de propriété et à l’autorisation des transferts; elles restent séparées du consensus actuel.

Références : [modèle d’émission](../EMISSION_MODEL.md), [décisions testnet](../TESTNET_DECISIONS.md), [règles de récompense](../../crates/aretovus-consensus/src/reward.rs).

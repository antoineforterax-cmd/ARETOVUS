# AURORA — Modèle d'émission par paliers (prototype de spécification)

**Statut : À SIMULER — proposition de travail, non validée pour le protocole.**  
**Périmètre :** modèle monétaire uniquement ; aucun changement du code de consensus dans ce document.

## 1. Objectif

Tester une émission qui démarre avec une récompense élevée, diminue rapidement vers un palier intermédiaire, puis ralentit progressivement à long terme. La récompense doit tenir compte à la fois :

- de la hauteur du bloc ;
- de l'offre brute déjà créée ;
- du plafond strict de 500 000 000 AUR ;
- des arrondis en unités entières de 10^-8 AUR.

L'intervalle cible proposé est de 120 secondes. Il ne sert qu'à convertir les hauteurs de bloc en durées indicatives : la règle de consensus dépend de la hauteur, pas de l'heure murale.

## 2. Invariant absolu

Soit :

- `CAP_UNITS = 500_000_000 × 100_000_000` ;
- `issued_units` : total brut déjà créé, en unités minimales ;
- `scheduled_reward_units(height, issued_units)` : récompense brute calculée par une fonction déterministe.

La récompense autorisée est toujours bornée par le reliquat :

```text
remaining = CAP_UNITS - issued_units
reward = min(scheduled_reward_units, remaining)
```

Si `issued_units >= CAP_UNITS`, la récompense vaut zéro. Les frais brûlés réduisent l'offre nette mais ne réaugmentent jamais le plafond brut. Aucune règle de récupération de fonds ne modifie l'émission.

## 3. Courbe hybride candidate

La règle candidate combine deux plafonds de récompense :

1. **Calendrier par hauteur** : une table de paliers définit la récompense maximale selon le numéro de bloc.
2. **Frein lié à l'offre restante** : la récompense ne peut pas dépasser une fraction définie du reliquat d'émission.

```text
reward_units = min(
    height_tier_reward_units(height),
    remaining_supply_limit_units(issued_units),
    CAP_UNITS - issued_units
)
```

Cette architecture garantit que les deux dimensions sont prises en compte. Elle ne détermine pas encore les valeurs des paliers ni la fraction du reliquat : ces paramètres doivent être comparés par simulation avant d'être retenus.

### Paliers à comparer

Les récompenses de départ proposées pour les scénarios sont **500 AUR**, puis une réduction vers **100 AUR**, ensuite **50–60 AUR**, puis des paliers plus bas tels que **30 AUR**, **10 AUR** et **1 AUR**. Ce sont des points d'essai, pas encore un calendrier complet : les hauteurs de transition doivent être déterminées par la simulation et inscrites explicitement avant toute implémentation de consensus.

Ne pas supposer que ces paliers sont des valeurs définitives, ni interpoler implicitement entre eux. Chaque transition doit avoir une règle entière et déterministe.

## 4. Trésorerie et frais : comptabilité séparée

Hypothèse de travail, à tester :

- récompense brute d'émission : `R` ;
- trésorerie : `floor(R / 20)` (5 %) ;
- mineur : `R - treasury` (95 %), de sorte que les deux allocations totalisent exactement `R` ;
- frais : partage 50/50 avec une règle d'arrondi publiée séparément ;
- brûlage des frais : diminue l'offre nette, pas le total brut historiquement créé.

Pour une récompense brute impaire en unités minimales, la trésorerie reçoit `floor(R/20)` et le mineur reçoit le reliquat. Cette règle est proposée uniquement pour permettre les tests ; elle doit être comparée à d'autres conventions d'arrondi avant validation.

## 5. Propriétés requises

- Le calcul utilise uniquement des entiers ; aucun flottant ne participe au consensus.
- Deux nœuds ayant la même hauteur et le même état monétaire calculent exactement la même récompense.
- La récompense ne dépasse jamais le reliquat du plafond.
- Une récompense calculée à zéro ne provoque aucune création monétaire.
- Les allocations mineur + trésorerie sont exactement égales à la récompense brute.
- Une réduction par palier ne peut jamais augmenter la récompense.
- Un changement d'offre créée ne peut pas contourner la borne de plafond.
- Les valeurs limites et les opérations arithmétiques sont vérifiées contre les dépassements.
- Le calendrier ne dépend pas de l'horloge locale du nœud.

## 6. Tests de simulation à réaliser

1. Tester les récompenses juste avant, sur et après chaque transition de palier.
2. Simuler des hauteurs de bloc normales et des émissions plus rapides/lentes que prévu.
3. Comparer l'effet du seul calendrier par hauteur, du seul frein d'offre et de la combinaison hybride.
4. Publier l'offre brute cumulée, l'offre restante, les récompenses par bloc, les allocations de trésorerie et le nombre de blocs où la récompense devient nulle.
5. Tester le plafond avec une offre restante plus petite que la récompense calculée.
6. Tester les arrondis à 0, 1, 19, 20, 21 unités et autour des seuils de paliers.
7. Tester que le brûlage des frais n'augmente pas l'offre brute disponible.
8. Vérifier les propriétés avec des tests génératifs et comparer la sortie à un calcul de référence indépendant.

## 7. Ce qui reste ouvert

- Hauteurs exactes des paliers.
- Frein lié à l'offre restante et son coefficient.
- Convention exacte de partage 50/50 des frais pour un nombre impair d'unités.
- Traitement de la récompense zéro et de la fin d'émission.
- Simulation économique des revenus des mineurs et de la trésorerie.
- Acceptation explicite du propriétaire après examen des résultats.

**Règle de processus :** ne pas modifier le consensus ni marquer cette courbe comme validée avant simulation reproductible, tests automatisés, revue des cas limites et confirmation explicite du propriétaire.

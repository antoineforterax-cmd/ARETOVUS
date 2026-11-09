# Pagination des en-têtes — 28 octobre 2026

La relecture statique signale `start as usize` dans `GET_HEADERS` (P0-10).
La conversion est remplacée par une conversion vérifiée et une lecture de la
tranche canonique, avant la résolution des blocs. Une hauteur non représentable
ou hors de la chaîne donne une page vide au niveau de cette API interne.
Le serveur conserve son rejet des hauteurs supérieures au sommet + 1.

Il s'agit d'un durcissement : la borne existante du serveur empêchait déjà une
hauteur arbitraire d'atteindre la conversion sur une petite chaîne. Aucune
exploitation sur 32 bits n'a été reproduite. Aucun gain de latence n'est annoncé :
l'itérateur précédent pouvait déjà optimiser le saut des éléments.

Deux tests de régression couvrent :

- l'équivalence avec la branche sélectionnée, avant et après une réorganisation,
  les limites de page et les hauteurs supérieures à `u32::MAX`, dont `u64::MAX` ;
- les octets du protocole TCP pour la genèse, la page vide après le sommet et le
  rejet d'une hauteur hors limites.

Validation locale Ubuntu : les 31 tests de la bibliothèque du nœud et les
3 tests de son exécutable passent. La CI du commit publié reste la référence
pour les vérifications complètes. Aucun changement de consensus, de données
persistantes, de binaires de la campagne de 72 heures ou de protocole réseau.

Ce correctif ne résout pas l'ensemble de P0-10 : l'inventaire des paniques et
la gestion des verrous empoisonnés restent ouverts. Il ne constitue pas une
revue indépendante du protocole.

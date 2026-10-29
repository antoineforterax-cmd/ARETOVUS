# Simulateur indépendant du retarget testnet

## Périmètre

Ce lot traite l'issue #9 et une partie des preuves demandées par #3.
Les paramètres restent ceux du testnet : 60 secondes par bloc, fenêtre de
10 blocs, difficulté initiale 8, bornes 4..20, variation maximale de deux bits.
Aucune règle de consensus, genèse, empreinte réseau ou donnée persistante ne change.

Le simulateur Python utilise uniquement sa bibliothèque standard. Il n'appelle
pas le Rust. Pour une fenêtre de 600 secondes, il compte les seuils strictement
franchis parmi les facteurs 2 et 4, puis borne le résultat. Il utilise des entiers
de précision arbitraire ; les entrées sont limitées au domaine u64 du testnet.
Il constitue une implémentation distincte, pas une revue humaine indépendante.

## Reproduction

```sh
python3 scripts/retarget_sim.py
python3 scripts/retarget_sim.py --check
cargo test --locked -p aurora-consensus --test retarget_vectors
cargo test --locked -p aurora-node independent_admission_vectors
cargo test --locked -p aurora-wallet independent_admission_vectors
```

Sans option, le simulateur imprime un bilan JSON sans modifier de fichier.
`--check` régénère les résultats en mémoire et refuse toute divergence avec les
fixtures versionnées. `--write` les remplace explicitement pour un changement
revu ; ne jamais régénérer les attentes uniquement pour faire passer un test.

## Cas et assertions

- 323 vecteurs numériques : chaque difficulté parente de 4 à 20, avec 19 durées
  couvrant zéro, les seuils exacts et leurs voisins, une longue pause et u64::MAX.
- Sept traces de 30 blocs : cadence régulière, rapide, lente, pause d'un jour,
  dérive à 61 secondes, alternance rapide/lente/normale, avance future répétée.
- Pour chaque bloc : difficulté annoncée par le mineur, difficulté validée,
  hauteur, horodatage et travail cumulé comparés au modèle.
- Nœud complet et chaîne d'en-têtes SPV reçoivent les mêmes blocs avec une preuve
  de travail exacte, afin qu'un excès de travail ne masque pas un seuil erroné.
- À chaque retarget, une preuve insuffisante est rejetée sans changer sommet,
  travail ou nombre de blocs connus ; rejeu indépendant de l'horloge.
- Trois branches concurrentes soumises dans trois ordres : sélection vérifiée
  après chaque insertion selon le travail, puis le hash en cas d'égalité.
- Huit vecteurs d'admission locale partagés entre P2P et wallet SPV : limite
  inclusive à +7200 secondes, refus à +7201, acceptation après avance de
  l'horloge, horloge en retard et addition saturante proche de u64::MAX.

Les traces utilisent la genèse testnet et les paramètres réels de difficulté,
mais désactivent les récompenses pour isoler le retarget ; les tests économiques
existants restent nécessaires. Le minage des tests est borné à dix millions de
nonces par bloc. Les bornes hautes sont couvertes numériquement, sans miner
systématiquement des blocs à 20 bits.

## Interprétation

À difficulté parente 8, une fenêtre de 149 secondes donne 10 bits, 150 secondes
donne 9, 300 donne 8, 1200 donne 8, 1201 donne 7, 2400 donne 7 et 2401 donne 6.
Ces seuils sont stricts. Le candidat ferme la fenêtre depuis son ancêtre H−10 ;
la médiane des ancêtres intervient séparément dans la validité temporelle.

Une chaîne future peut être valide au sens du consensus déterministe tout en
étant refusée à l'entrée du réseau. Les vecteurs distinguent volontairement ces
deux décisions. La borne locale ne démontre pas à elle seule la résistance à
toutes les stratégies de manipulation des horodatages.

## Limites et suites

Ce lot ne clôt pas #3 : partitions répétées, stratégie de réessai en réseau,
horloges divergentes de plusieurs machines et campagne prolongée restent à
éprouver. Il ne prouve ni la résistance à 51 %, ni la rentabilité du minage.
Les résultats d'exécution sont liés dans la PR ; les fixtures seules ne sont pas
une preuve que la CI a réussi.

Priorités suivantes : saturation multi-IP des workers (#4/#8), synchronisation
réseau au-delà de 100 000 blocs (#5), puis campagne multi-hôtes de 72 heures.

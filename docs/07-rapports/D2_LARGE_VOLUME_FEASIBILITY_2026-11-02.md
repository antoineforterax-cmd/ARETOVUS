# D2 — faisabilité théorique des volumes ARETOVUS (aucun essai physique implicite)

Cette étude du 2 novembre 2026 complète [le diagnostic des snapshots](../validation/P2P_STORAGE_COST_MODEL_2026-10-31.md). **Aucun bloc parmi les volumes 1 M à 500 M n'est créé, propagé ou stocké par le script : les résultats ci-dessous sont calculés à partir de la politique de pagination actuelle.**

## Modèle vérifiable

Pour `N` blocs ajoutés après une genèse unique, des pages de `P=64` blocs et un snapshot après chaque page, on note `m = ceil(N/P)` :

- Nombre de snapshots supposés : `m`.
- Minimum de vérifications des chemins de blocs déjà enregistrés : `m + P*m*(m-1)/2 + N`. Ce calcul tient compte de la dernière page partielle.
- Hypothèses : chaîne linéaire sans forks, un contrôle de chemin de chacun des blocs visibles à chaque snapshot. Les branches latérales et opérations supplémentaires ne sont pas représentées.

| Blocs **hypothétiques** | Snapshots du modèle | Vérifications minimales de chemins **calculées** |
| ---: | ---: | ---: |
| 1 000 000 | 15 625 | 7 813 015 625 |
| 10 000 000 | 156 250 | 781 255 156 250 |
| 100 000 000 | 1 562 500 | 78 125 051 562 500 |
| 500 000 000 | 7 812 500 | 1 953 125 257 812 500 |

Ces nombres révèlent le risque d'un coût de scan **quadratique** avec la hauteur, si la stratégie de vérification complète et la fréquence de snapshots restent inchangées. Ils ne permettent de déduire ni temps d'exécution fiable, ni consommation RAM/CPU/disque, ni performance P2P. N'appliquer aucun coefficient de débit CPU ou de disque sans benchmark mesuré et reproductible au SHA pertinent.

## Provenance et limites

Le code `scripts/scale_cost_model.py` calcule seulement des entiers, et `scripts/test_scale_cost_model.py` vérifie les exemples 8 192 et 100 001 blocs déjà documentés ainsi que les quatre grands paliers. Le workflow `d2-storage-scale-model` conserve JSON, SHA GitHub Actions et tests, mais **ce n'est pas un test de gros volumes réels**.

Il existe un test de stockage un million de blocs conçu pour lancement manuel (`.github/workflows/million-block-storage-probe.yml`), cependant sa présence ne vaut pas succès ; il faut vérifier une conclusion CI sur le SHA exact, la taille disque, le temps et la restauration. Le test TCP 100 001 blocs de [#167](https://github.com/antoineforterax-cmd/ARETOVUS/pull/167) demeure un palier réel distinct et ne prouve rien à 1 M, 10 M, 100 M ou 500 M.

## Décisions laissées ouvertes

La décision d'espacer les snapshots ou d'éviter certaines revérifications modifie la granularité de récupération après panne, voire la surface d'attaque par substitution de fichiers. **Ne pas modifier cette politique sans revue de durabilité, crash/restart, ENOSPC et symlinks.** La fin de D2 demandera un budget réaliste de disque et de temps sur plusieurs hôtes, et éventuellement une preuve physique sur des volumes représentatifs. Aucun fonds réel.

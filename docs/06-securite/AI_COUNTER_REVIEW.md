# Contre-revue technique par un second agent IA

**Périmètre lu :** branche `feature/emission-schedule-v0`, PR #2, commit `ed3a97176ab9b5aa39d3e824f0ce04727c53ea15`.  
**Méthode :** lecture séparée du code de consensus, stockage et P2P par un second agent; aucune modification ni essai d’exploitation effectués par cet agent. Le responsable a ensuite vérifié les chemins de code et créé des tests ou tâches de reproduction.  
**Statut :** cette contre-revue aide à trouver des défauts, mais ne constitue pas une revue humaine indépendante ni une assurance pour des fonds réels.

## Constats

| Constat | Risque | Suivi |
| --- | --- | --- |
| Les horodatages futurs n’ont pas de borne d’admission liée à l’horloge locale et alimentent le retarget. | Un mineur peut tenter de faire baisser la difficulté et d’avancer l’horloge de la chaîne. L’exploitation n’a pas été reproduite. | [Issue #3](https://github.com/antoineforterax-cmd/ARETOVUS/issues/3) : vecteurs de test, politique temporelle et versionnement. |
| Huit connexions lentes ou silencieuses peuvent occuper les huit workers P2P. | Déni de service du nœud sans grand volume de données; essai de saturation non exécuté. | [Issue #4](https://github.com/antoineforterax-cmd/ARETOVUS/issues/4) : test de neuf connexions et admission équitable. |
| Le stockage et le transport arrêtent la chaîne à 100 000 blocs connus, branches comprises. | Refus des nouveaux blocs après saturation; essai à l’échelle réelle non exécuté. | [Issue #5](https://github.com/antoineforterax-cmd/ARETOVUS/issues/5) : pagination, stockage et test de franchissement. |
| Les demandes de statut, de preuve et de pages parcouraient toute la chaîne sous le verrou global. | Un pair pouvait retarder le traitement des blocs avec de petites requêtes répétées. Le coût réel n’a pas été mesuré. | Index canonique des transactions avec mise à jour lors des réorganisations; ordre de blocs mémorisé et pages bornées. [CI du correctif](https://github.com/antoineforterax-cmd/ARETOVUS/actions/runs/40402855391) verte; essais prolongés de cette révision en cours. |

## Limite de cette démarche

Les quatre constats proviennent d’une inspection statique. Les scénarios d’attaque 1 à 3 restent à reproduire et corriger. Pour le quatrième, les tests de réorganisation, de portefeuille/SPV et de pagination servent de régression fonctionnelle; une mesure de latence sous charge hostile reste à faire. Après ces corrections, une personne extérieure devra encore examiner le protocole et produire son propre rapport avant toute affirmation de sécurité mainnet.

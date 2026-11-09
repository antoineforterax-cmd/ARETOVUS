# Licences, droits, contributions et réutilisation d'ARETOVUS

> **Guide explicatif, pas avis juridique.** Pour une publication avec enjeux financiers, une revue juridique professionnelle des droits, des dépendances et des obligations applicables reste nécessaire.

## Quelles licences sont actuellement annoncées ?

Le dépôt annonce **MIT OR Apache-2.0** : une personne qui reçoit une copie peut utiliser **l'une ou l'autre** licence, sous réserve des droits effectivement détenus et des conditions de cette licence.

Les textes officiels à conserver **sans réécriture improvisée** :
- [LICENSE](../../LICENSE) : texte combiné et indication du double choix.
- [LICENSE-MIT](../../LICENSE-MIT) : texte MIT.
- [LICENSE-APACHE](../../LICENSE-APACHE) : texte Apache-2.0.
- [Cargo.toml](../../Cargo.toml) et les manifests des crates : mentions de licence des paquets, à vérifier lors de chaque distribution.

**Ces trois fichiers ne sont pas des « doublons inutiles » à supprimer machinalement :** ils servent à différents lecteurs et outils. Les licences des **dépendances tierces** s'ajoutent aux conditions applicables ; notre licence ne permet pas d'ignorer leurs obligations.

## Que peut faire un autre développeur ?

En principe, selon les textes applicables, il peut **lire, exécuter, modifier, redistribuer** et même intégrer du code à un projet commercial. Une contribution au dépôt principal est **encouragée**, mais pas rendue juridiquement obligatoire.

- **MIT** demande notamment de conserver l'avis de copyright et la permission de licence dans les copies substantielles.
- **Apache-2.0** impose ses propres obligations de conservation des avis/licences et prévoit notamment des dispositions sur les brevets ; les avis `NOTICE` doivent être traités correctement **s'il en existe et si applicable**.
- Un **fork** peut exister même si ARETOVUS reste le projet de référence. Empêcher tous les forks ne serait pas compatible avec l'objectif open source.
- Le **nom ARETOVUS et son logo** soulèvent des questions distinctes de la licence du code ; aucune exclusivité de marque déposée n'est affirmée ici.

## « L'IA a écrit le code, donc je ne possède rien » ?

Non : ce raisonnement confond le contrôle du dépôt, les droits sur des contributions humaines, la licence de distribution, les droits éventuellement transférés par les outils d'IA et la protection par droit d'auteur des sorties entièrement générées automatiquement.

Le projet a été **largement développé à l'aide d'IA** ; cela n'annule pas mécaniquement tous les droits du porteur. **En revanche, il serait imprudent de garantir une propriété exclusive sur chaque ligne ou une chaîne de droits parfaite sans revue juridique.**

Pour être transparent, distinguer dans [PROVENANCE.md](../../PROVENANCE.md) le rôle de direction humaine, les contributions et outils effectivement documentés, et les limites de ce que l'historique prouve. L'en-tête de copyright `ARETOVUS contributors` n'est pas, à lui seul, un certificat de propriété ou de cession de tous les droits.

## Comment contribuer proprement ?

1. N'inclure que des contributions sur lesquelles vous avez les autorisations nécessaires.
2. Respecter les licences des bibliothèques, extraits copiés et ressources graphiques.
3. Signaler les parties générées ou substantiellement assistées par IA et les vérifications effectuées.
4. Proposer des changements sous **MIT OR Apache-2.0** selon les [règles de contribution](../../CONTRIBUTING.md), sans inventer de transfert obligatoire de droits supplémentaires.
5. En cas d'incertitude sur l'origine ou la licence d'un composant, **ne pas le redistribuer sans clarification**.

## Peut-on changer la licence plus tard ?

Un changement peut concerner les futures contributions et distributions, mais il faut examiner les droits de chaque contributeur. Les autorisations déjà accordées pour les versions publiées **ne disparaissent pas simplement parce qu'on change un fichier**. Ne promettons donc pas de « récupérer » rétroactivement des forks déjà autorisés.

## Et un futur mainnet ?

La licence libre n'autorise pas, à elle seule, la vente d'actifs, une activité financière ou l'exploitation d'un réseau public dans n'importe quelle juridiction. Avant des fonds réels, il faudra une analyse distincte de la sécurité, de l'économie, des conditions de distribution et du droit applicable.

**Position du projet : open source expérimental, pas une offre financière.**
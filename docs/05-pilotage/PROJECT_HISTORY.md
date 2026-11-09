# ARETOVUS — Historique et provenance de l'identité

> Document de référence unique pour comprendre l'ancien nom, le changement de marque et la naissance de la chaîne native. **Ce document ne modifie aucun fait historique, aucun hash de bloc, aucune licence ni aucune règle de consensus.**

## Origine et changement de nom

Le prototype Rust s'appelait initialement **AURORA**. Il a été renommé **ARETOVUS** à partir du 31 octobre 2026, avec la [PR de rebranding #137](https://github.com/antoineforterax-cmd/ARETOVUS/pull/137). Le dépôt courant est [antoineforterax-cmd/ARETOVUS](https://github.com/antoineforterax-cmd/ARETOVUS).

Il ne s'agit pas seulement d'une modification d'affichage. La [migration d'identité native](PROTOCOL_IDENTITY_MIGRATION.md) sépare la nouvelle chaîne de l'ancien testnet : nouvelle genèse, domaines cryptographiques, marques de stockage, formats de clés et URI. **Les anciens soldes, clés, snapshots et données ne sont pas transférés par renommage** ; il faut utiliser un répertoire de données neuf.

L'historique Git, les PR, les rapports d'essais et les preuves de la chaîne expérimentale précédente restent conservés. Leurs noms, SHA et identifiants sont des faits historiques et non des noms utilisables dans les binaires actuels.

## Liens vers les preuves historiques originales

| Preuve | Rôle |
| --- | --- |
| [Inventaire des PR anciennes](D4_LEGACY_PR_BRANDING_INVENTORY_2026-11-02.md) | Identifie les propositions réutilisables et celles à ne pas fusionner |
| [État du 29 octobre](consolidation/STATE_2026-10-29.md) | Photographie datée des branches, commits et dépendances |
| [Précontrôle d'intégration du 29 octobre](consolidation/INTEGRATION_PREFLIGHT_2026-10-29.md) | Comparaisons des branches telles qu'elles existaient alors |
| [État Git JSON du 29 octobre](consolidation/state-2026-10-29.json) | Preuve machine des noms de branches et des SHA d'origine |
| [Décisions de migration de protocole](PROTOCOL_IDENTITY_MIGRATION.md) | Identité technique du nouveau réseau et incompatibilités explicites |
| [Rapport de validation du testnet](TESTNET_VALIDATION_REPORT.md) | Limites des tests, résultats et preuves disponibles |
| [Traçabilité des auteurs et de l'assistance IA](../PROVENANCE.md) | Origine des contributions et limites de la revue |

L'ancien identifiant de branche `docs/aurora-master-plan-2026-10` est reproduit ici **uniquement comme référence historique exacte** ; il ne doit pas être renommé à l'intérieur des relevés d'octobre, car ceux-ci documentent une réalité Git ancienne.

## Notices de copyright et licences

La notice des premières versions mentionnait exactement `Copyright (c) 2026 AURORA contributors`. Elle apparaît notamment dans le [commit d'origine des licences](https://github.com/antoineforterax-cmd/ARETOVUS/commit/421d443370e83a9ffa0c128da9f09d6f99b0ef8f). La désignation actuelle `ARETOVUS contributors` vise le même collectif historique et ne doit pas être comprise comme effaçant un titulaire ou transférant des droits.

**Réserve juridique :** le renommage d'une notice de copyright n'est acceptable que si les attributions obligatoires restent respectées et que les titulaires concernés y consentent lorsque cela est nécessaire. Les textes MIT et Apache-2.0 sont distincts des noms commerciaux ; les obligations de conservation de notices s'appliquent également aux redistributions. Le présent document ne remplace pas une validation juridique des droits.

## Conventions de maintenance

- Le code exécutable, les nouveaux fichiers, les noms de crate, les URI et la documentation opérationnelle utilisent uniquement **ARETOVUS**.
- Les anciennes références sont autorisées dans **ce document d'histoire**, les snapshots datés et les corpus d'essais immuables ; les données de preuves ne doivent pas être corrigées artificiellement.
- L'identité native est surveillée par [le test de compatibilité du renommage](../scripts/test_branding_compatibility.py), exécuté en CI.
- Aucune archive de l'ancien testnet ne peut être utilisée comme preuve de sécurité ou de compatibilité d'une version actuelle. Le statut **NO-GO** pour un réseau public à fonds réels n'est pas levé par ce nettoyage.

## Périmètre du nettoyage

La [PR #193](https://github.com/antoineforterax-cmd/ARETOVUS/pull/193) consolide le renommage des sources actives et les exceptions historiques documentées. Ce dossier est une porte d'entrée **unique pour le lecteur**, pas une suppression des pièces justificatives ni une réécriture de l'historique Git.

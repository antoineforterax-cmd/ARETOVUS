# Provenance et vérifications

Pour une explication des droits et licences, lire [LICENCES_ET_DROITS.md](docs/02-participer/LICENCES_ET_DROITS.md). Un historique de commits ou une mention de copyright ne garantit pas à lui seul l'identité de l'auteur humain de chaque ligne ni une chaîne de droits incontestable.

Le prototype initial d’ARETOVUS a été largement écrit avec une IA à la demande du porteur du dépôt `antoineforterax-cmd`. La première phase a été menée très rapidement, sur environ une journée. L’historique Git commence le 25 octobre 2026 et contient des modifications supplémentaires les 26 et 27 octobre; cela ne représente pas des années de développement ou d’audit.

Le porteur du projet définit la vision d’ARETOVUS, propose les orientations d’architecture, fixe les priorités et pilote activement le développement. La production de code est **très largement assistée par IA**, y compris pour des parties techniques importantes. Le porteur a également rapporté des modifications et vérifications personnelles, notamment sur des scripts Python et des parties Rust ; la répartition exacte des lignes d'origine humaine ou IA **n'a pas été mesurée ni auditée**. Les outils d'IA servent à générer, expliquer, tester et contre-relire les modifications ; les résultats doivent être vérifiés, et une revue par une autre IA ne vaut pas un audit humain indépendant. Les décisions et priorités sont définies par le porteur.


## Continuité des attributions après le renommage

La mention `ARETOVUS contributors` utilisée dans les licences désigne le collectif des contributeurs du projet, y compris ceux intervenus avant son changement de nom. Le renommage de cette désignation ne constitue ni un transfert de droits, ni une suppression de contributions ou une validation de l'appartenance juridique des droits. Les textes MIT/Apache restent inchangés et les notices de licence antérieures demeurent consultables dans l'historique Git, notamment au commit `421d443370e83a9ffa0c128da9f09d6f99b0ef8f`. Consulter [l'histoire du projet](docs/05-pilotage/PROJECT_HISTORY.md) pour les notices originelles et les références archivées.

## Ce qui a réellement été vérifié

- GitHub Actions exécute formatage, Clippy, compilation, tests Rust, smoke multi-nœuds, empaquetage et campagnes bornées de fuzzing/soak. Les commits et liens sont dans [TESTNET_VALIDATION_REPORT.md](docs/TESTNET_VALIDATION_REPORT.md).
- Le porteur a fourni des sorties Windows de création de wallet, minage, lecture de solde et synchronisation de trois blocs. Il a aussi rapporté des essais macOS/Linux; leur convergence multi-hôtes n’est pas démontrée par ces messages.
- Une contre-revue par un autre agent IA a signalé des risques consignés dans [AI_COUNTER_REVIEW.md](docs/04-securite/AI_COUNTER_REVIEW.md). Une nouvelle tentative du 27 octobre n’a produit aucun rapport à cause d’une limite d’usage.
- Le scan Gitleaks de l’historique disponible du 27 octobre n’a signalé aucun secret; sa portée exacte est consignée dans le rapport.

Le porteur a personnellement examiné et corrigé du code, notamment en Python, en complément des essais manuels et automatisés. Ces vérifications internes comptent dans le développement, sans constituer une revue extérieure indépendante.

À ce jour, aucune revue de sécurité approfondie et documentée par une personne extérieure qualifiée n’est confirmée. Il peut s’agir d’un développeur expérimenté, d’un étudiant compétent ou d’un passionné bénévole : l’important est la qualité de la méthode et des constats, pas le statut professionnel. Les tests réussis et les contre-revues automatisées ne garantissent pas l’absence de défauts. Le [dossier de revue](docs/06-securite/INDEPENDENT_REVIEW.md) précise les vérifications attendues avant tout usage avec de vrais fonds.

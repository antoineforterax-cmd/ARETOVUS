# Provenance et vérifications

Le prototype initial d’AURORA a été largement écrit avec une IA à la demande du porteur du dépôt `antoineforterax-cmd`. La première phase a été menée très rapidement, sur environ une journée. L’historique Git commence le 25 octobre 2026 et contient des modifications supplémentaires les 26 et 27 octobre; cela ne représente pas des années de développement ou d’audit.

Le porteur du projet définit la vision d’AURORA, propose les orientations d’architecture, fixe les priorités et pilote activement le développement. Il a personnellement écrit et corrigé des pans entiers du code (notamment les scripts de test et d'automatisation en Python, ainsi que des contributions directes sur le Rust). L’IA est utilisée comme un outil d’assistance technique à la demande pour accélérer la mise en œuvre, générer certaines structures ou aider à la recherche de défauts. Les propositions et le code restent soumis aux décisions du porteur et à ses vérifications.

## Ce qui a réellement été vérifié

- GitHub Actions exécute formatage, Clippy, compilation, tests Rust, smoke multi-nœuds, empaquetage et campagnes bornées de fuzzing/soak. Les commits et liens sont dans [TESTNET_VALIDATION_REPORT.md](docs/TESTNET_VALIDATION_REPORT.md).
- Le porteur a fourni des sorties Windows de création de wallet, minage, lecture de solde et synchronisation de trois blocs. Il a aussi rapporté des essais macOS/Linux; leur convergence multi-hôtes n’est pas démontrée par ces messages.
- Une contre-revue par un autre agent IA a signalé des risques consignés dans [AI_COUNTER_REVIEW.md](docs/AI_COUNTER_REVIEW.md). Une nouvelle tentative du 27 octobre n’a produit aucun rapport à cause d’une limite d’usage.
- Le scan Gitleaks de l’historique disponible du 27 octobre n’a signalé aucun secret; sa portée exacte est consignée dans le rapport.

Le porteur a personnellement examiné et corrigé du code, notamment en Python, en complément des essais manuels et automatisés. Ces vérifications internes comptent dans le développement, sans constituer une revue extérieure indépendante.

À ce jour, aucune revue de sécurité approfondie et documentée par une personne extérieure qualifiée n’est confirmée. Il peut s’agir d’un développeur expérimenté, d’un étudiant compétent ou d’un passionné bénévole : l’important est la qualité de la méthode et des constats, pas le statut professionnel. Les tests réussis et les contre-revues automatisées ne garantissent pas l’absence de défauts. Le [dossier de revue](docs/INDEPENDENT_REVIEW.md) précise les vérifications attendues avant tout usage avec de vrais fonds.

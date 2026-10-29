# Provenance et vérifications

Le prototype initial d’AURORA a été largement écrit avec une IA à la demande du porteur du dépôt `antoineforterax-cmd`. La première phase a été menée très rapidement, sur environ une journée. L’historique Git commence le 25 octobre 2026 et contient des modifications supplémentaires les 26 et 27 octobre; cela ne représente pas des années de développement ou d’audit.

Le porteur du projet définit la vision d’AURORA, propose les orientations d’architecture, fixe les priorités et les choix économiques, et dirige les étapes de développement. Il pilote les décisions et les essais du projet. L’IA est utilisée comme outil d’assistance pour accélérer la mise en œuvre de ces orientations : des agents Codex ont produit ou modifié une grande partie du code, des tests et de la documentation, et ont également aidé à la recherche de défauts et aux revues techniques. Les propositions et le code restent soumis aux décisions du porteur et aux vérifications nécessaires. Ce texte décrit le processus.

## Ce qui a réellement été vérifié

- GitHub Actions exécute formatage, Clippy, compilation, tests Rust, smoke multi-nœuds, empaquetage et campagnes bornées de fuzzing/soak. Les commits et liens sont dans [TESTNET_VALIDATION_REPORT.md](docs/TESTNET_VALIDATION_REPORT.md).
- Le porteur a fourni des sorties Windows de création de wallet, minage, lecture de solde et synchronisation de trois blocs. Il a aussi rapporté des essais macOS/Linux; leur convergence multi-hôtes n’est pas démontrée par ces messages.
- Une contre-revue par un autre agent IA a signalé des risques consignés dans [AI_COUNTER_REVIEW.md](docs/AI_COUNTER_REVIEW.md). Une nouvelle tentative du 27 octobre n’a produit aucun rapport à cause d’une limite d’usage.
- Le scan Gitleaks de l’historique disponible du 27 octobre n’a signalé aucun secret; sa portée exacte est consignée dans le rapport.

Aucun reviewer humain indépendant n’est désigné et aucun audit humain n’a été reçu. Une revue par IA et des tests verts peuvent manquer des défauts de conception. Le [dossier de revue](docs/INDEPENDENT_REVIEW.md) explique ce qui doit être examiné avant toute prétention de sécurité au-delà du testnet.

# ARETOVUS — identité publique et compatibilité historique

ARETOVUS est le nom de marque qui remplace AURORA pour le projet Rust de blockchain expérimentale. Ce changement **ne constitue pas** un nouveau réseau, un nouveau protocole ou une réinitialisation des données. Aucun fonds réel ne doit être utilisé.

## Identité visuelle

L'emblème officiel fourni par le responsable est conservé dans [`assets/aretovus-logo.svg`](../assets/aretovus-logo.svg) (version vectorisée fidèle au visuel fourni, en ivoire sur fond noir). Le logo figure dans le [README principal](../README.md) et dans le [guide d'accueil](guide-fr/README.md). Son affichage n'intervient dans aucune donnée de consensus, fichier de clés ou format réseau.

## Identifiants conservés intentionnellement

- Les préfixes de séparation de domaine de hachage et de signature `AURORA/exp/...` et `AURORA/testnet/...` sont des entrées cryptographiques : **les modifier change les identifiants et signatures**.
- Les octets magiques `AURORIDX` (index v6), la genèse, les paramètres de réseau, l'empreinte de réseau et les formats de stockage restent identiques.
- Le schéma URI `aurora://testnet/`, le nom de verrou local `.aurora.lock`, les anciens fichiers de sauvegarde, les tags/releases, les anciennes archives, et les messages de preuve archivés restent reconnus comme historiques.
- Les anciens marqueurs `network=AURORA-TESTNET`, la confirmation manuelle `SEND AURORA-TESTNET`, les en-têtes des politiques wallet et les variables de tests `AURORA_*` restent des identifiants de compatibilité. Les workflows les utilisent encore intentionnellement ; les modifier demande des tests de migration ciblés.
- Les scripts de comparaison utilisent volontairement `aurora-node` lorsqu'ils compilent un commit historique, mais la version courante expose `aretovus-node`.
- Le symbole économique expérimental `AUR` est conservé, car un nouveau ticker et sa migration n'ont pas été définis ni validés. Ce n'est **pas** une assurance de disponibilité du ticker.
- Les anciennes URLs `github.com/antoineforterax-cmd/AURORA`, les références et SHA historiques sont conservés pour l'intégrité des preuves. Le dépôt GitHub est désormais nommé `antoineforterax-cmd/ARETOVUS` ; les URLs historiques peuvent rediriger vers ce dépôt. Les SHA, tags et références de preuve restent immuables.

## Périmètre de la PR

Changement de marque affichée, noms de crates et binaires d'outillage, noms d'archives et documentation. Aucune altération voulue de consensus, signature, économie, émission, genèse, URI wire ou format de stockage.

## Après intégration

Contrôler `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace --locked`, `cargo test --workspace --locked`, `python3 scripts/testnet_smoke.py` et packaging sur Windows/Linux/macOS. Les anciens binaires AURORA ne sont pas renommés rétroactivement. Les automatisations externes, URLs d'artefacts, scripts utilisateur et README anciens doivent être vérifiés lors du passage de la marque.

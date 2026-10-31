# ARETOVUS — identité publique et compatibilité historique

ARETOVUS est le nom de marque qui remplace AURORA pour le projet Rust de blockchain expérimentale. Ce changement **ne constitue pas** un nouveau réseau, un nouveau protocole ou une réinitialisation des données. Aucun fonds réel ne doit être utilisé.

## Identifiants conservés intentionnellement

- Les préfixes de séparation de domaine de hachage et de signature `AURORA/exp/...` et `AURORA/testnet/...` sont des entrées cryptographiques : **les modifier change les identifiants et signatures**.
- Les octets magiques `AURORIDX` (index v6), la genèse, les paramètres de réseau, l'empreinte de réseau et les formats de stockage restent identiques.
- Le schéma URI `aurora://testnet/`, le nom de verrou local `.aurora.lock`, les anciens fichiers de sauvegarde, les tags/releases, les anciennes archives, et les messages de preuve archivés restent reconnus comme historiques.
- Le symbole économique expérimental `AUR` est conservé, car un nouveau ticker et sa migration n'ont pas été définis ni validés. Ce n'est **pas** une assurance de disponibilité du ticker.
- Les anciennes URLs `github.com/antoineforterax-cmd/AURORA`, les références et SHA historiques sont conservés pour l'intégrité des preuves. Le renommage GitHub du dépôt est une opération administrative séparée après revue des liens/actions.

## Périmètre de la PR

Changement de marque affichée, noms de crates et binaires d'outillage, noms d'archives et documentation. Aucune altération voulue de consensus, signature, économie, émission, genèse, URI wire ou format de stockage.

## Après intégration

Contrôler `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --workspace --locked`, `cargo test --workspace --locked`, `python3 scripts/testnet_smoke.py` et packaging sur Windows/Linux/macOS. Les anciens binaires AURORA ne sont pas renommés rétroactivement. Les automatisations externes, URLs d'artefacts, scripts utilisateur et README anciens doivent être vérifiés lors du passage de la marque.

# ARETOVUS V2 — premier pont IPC Rust (lecture seule)

Statut : **prototype local sur branche de développement, non mainnet**. Suivi : #178 et #180. N'utiliser que des données testnet jetables.

## Surface réellement implémentée

Le binaire Rust aretovus-local-bridge est compilé avec aretovus-node. Il accepte **une seule requête** sur stdin, traite un snapshot existant validé et écrit une seule réponse JSON sur stdout. Aucun socket HTTP/TCP, aucun accès wallet, aucune signature, aucune création de répertoire, aucun minage et aucune écriture de snapshot.

Exemple après compilation et génération d'un snapshot testnet expérimental :

    printf '{"schema_version":1,"method":"status"}' | target/debug/aretovus-local-bridge --data-dir ./testnet-data

Requête acceptée (octets canoniques UTF-8 uniquement ; CRLF/LF final facultatif) :

    {"schema_version":1,"method":"status"}

Le protocole refuse tout autre message : plusieurs requêtes, JSON avec champs supplémentaires, méthodes d'écriture, version inconnue ou payload de plus de 256 octets. Ce choix simplifie le premier contrat ; il **ne constitue pas un parseur JSON général** et ne doit pas être traité comme tel par un client.

Succès : schema_version=1, network=testnet, status=verified, height, tip, known_blocks, chain_work_display, source, read_only=true. Le travail cumulé est uniquement destiné à l'affichage diagnostique. Le succès est une vérification **locale** du snapshot, pas une preuve de connectivité P2P, de synchronisation mondiale ou de sécurité financière. Échec : code stable sans chemin ni exception interne, sortie processus 2. La GUI ne doit pas traiter un code de sortie non nul comme une validation.

## Modèle de sécurité et limitations

- Le conteneur desktop devra lancer le binaire connu lui-même, en utilisant un chemin contrôlé et le dossier choisi explicitement par l'utilisateur. Pas de commande shell, pas d'exécutable fourni par une page web.
- L'OS limite l'accès au canal stdio au parent et à son enfant ; il n'y a **aucun token de session HTTP car aucun service HTTP n'existe**. L'authentification/les vérifications Host et Origin seront nécessaires si une future API socket est décidée dans une ADR distincte.
- Le processus lit les snapshots vérifiés avec le décodeur Rust existant et vérifie la genèse, la difficulté initiale et les paramètres de récompense natifs. Ne jamais adapter ce pont en autorité de consensus ou indexeur de soldes.
- La lecture peut coûter du CPU/disque sur une chaîne volumineuse. Le processus parent doit imposer un timeout, contrôler les chemins et éviter les appels fréquents/concurrents. L'entrée est bornée, mais **le temps de chargement snapshot ne l'est pas dans ce binaire**.
- Pas de gestion des pairs, pas de statut live, pas de contrôle du nœud, pas d'API write, pas d'interface graphique : ces étapes viennent ensuite.
- Ne pas y connecter de wallet réel ni utiliser de fonds réels. Le schéma d'erreur ne divulgue pas les chemins privés.

## Contrôles exigés

cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo build --workspace --locked, cargo test --workspace --locked, smoke test habituel et workflow v2-local-stdio-ipc.yml. L'intégration native doit produire un vrai bloc testnet jetable, puis prouver la réponse JSON du pont, le rejet d'une méthode d'écriture et l'absence d'initialisation d'un dossier inexistant.

Dépendances fonctionnelles UI : #179 (adaptateur JSON Python) et #191 (snapshot HTML) sont des travaux séparés ; le binaire Rust ne doit pas requérir leur Python pour fonctionner. La suite #181 devra choisir une intégration IPC de GUI sécurisée et des mises à jour de l'état, sans promettre qu'un instantané est un flux live.

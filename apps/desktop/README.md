# ARETOVUS Desktop Lab — première interface réellement connectée (V2)

Travail expérimental pour les tickets #181 et #178 ; dépend de **PR #192** et de son binaire Rust `aretovus-local-bridge`. Aucun fonds réel, aucun wallet/signature ni contrôle du minage. Interface React/Vite/TypeScript, hôte local Tauri v2. L'hôte lance un binaire natif placé **à côté de son propre exécutable**, transmet une requête unique et affiche une réponse validée. Aucun navigateur web ordinaire ne peut obtenir ces données via une API publique.

## Compilation de développement (non distribuable)

1. Compiler le nœud et le pont depuis la racine ARETOVUS : `cargo build --locked -p aretovus-node --bin aretovus-node --bin aretovus-local-bridge`.
2. Installer les dépendances JS : `cd apps/desktop && npm install`.
3. Compiler/lancer la partie Tauri : `npm run desktop` (dépendances Tauri/WebView du système requises).
4. **Important :** placer le binaire `aretovus-local-bridge` construit à côté du binaire `aretovus-desktop` produit par Tauri (ou configurer le même dossier cible Cargo de façon maîtrisée). Pas de résolution par PATH ni de binaire téléchargeable depuis la webview.
5. Dans l'application, indiquer un chemin de **snapshot testnet natif déjà créé** puis connecter. La hauteur, le tip et le travail proviennent du vrai nœud via le pont ; la vue se rafraîchit toutes les 20 secondes, avec erreur explicite si le pont est indisponible.

L'absence de package-lock et d'icônes/bundles signés bloque encore un build reproductible/distribuable. Cette étape ne revendique **pas** un installateur utilisable par le public. Le build CI frontend valide typage/tests, la compilation hôte Tauri et le packaging multi-OS restent à vérifier séparément.

## Sécurité

- Frontend **lecture seule**, ne conserve aucune clé, mot de passe ou seed dans JS.
- Un unique invoke Rust asynchrone : `read_node_status` ; pas de shell, HTTP, plugin shell ou permission filesystem webview.
- Chemin choisi par l'utilisateur canonisé côté hôte ; binaire fixe adjacent à l'exécutable (pas PATH). Entrée et sortie limitées, timeout du processus à 30 s ; stderr supprimé et codes d'erreurs stables.
- Les sorties du bridge sont vérifiées à nouveau dans l'hôte et dans le frontend. Un navigateur seul, même avec un JSON similaire, ne constitue pas un nœud validé.
- Attention : le prototype dépend encore de l'intégrité des exécutables installés et peut consommer du CPU/disque à chaque relecture complète. Aucune garantie de synchronisation globale/P2P active. Distinguer la lecture du snapshot d'un affichage de consensus de réseau.

## Reste à faire

Pin du lockfile npm, intégration packaging binaire côte-à-côte, test hôte Windows/Linux/macOS, cas de processus suspendu, chemin verrouillé, corruption de snapshot, refresh en cas de réorganisation et interface professionnelle. Ensuite wallet graphique testnet, minage et explorateur, via contrats dédiés et revue de sécurité.

## Pages de blocs canoniques (PR empilée #196, dépendance #195)

Le nouveau panneau « Explorateur de blocs » appelle `read_canonical_page` depuis l'hôte Tauri. L'hôte démarre uniquement le binaire Rust compagnon `aretovus-explorer` (PR #195) placé à côté de son propre exécutable, avec un dossier existant, un départ borné et une limite d'au plus 20 blocs. Il capture la sortie sans risque de blocage si elle dépasse la capacité du pipe, impose un timeout de 30 s, borne la réponse à 16 KiB, et **rejette** JSON malformé, discontinuité des parents, nombres JS non sûrs, faux tip et curseurs invalides. Les champs natifs timestamp et nonce (u64) passent dans la webview sous forme de **chaînes décimales** pour éviter tout arrondi JavaScript.

La page est **déclenchée à la demande**, et non à chaque rafraîchissement du nœud, afin de limiter la charge CPU/disque. La vue refuse des pages provenant d'un autre sommet que celui affiché dans le statut local. Il faut rafraîchir le statut puis la page après une réorganisation : l'identité du sommet ne garantit pas une finalité globale. La limite de 10 blocs par page utilisée côté UI est volontairement inférieure à celle du moteur Rust.

**Prérequis supplémentaire :** construire `aretovus-explorer` depuis un commit où la PR #195 est intégrée et placer ce binaire dans le même répertoire approuvé que `aretovus-desktop` et `aretovus-local-bridge`. Le workflow Windows de cette PR ne prouve pas encore le packaging multi-OS, les ACL du dossier d'installation, ni la présence du compagnon chez les utilisateurs. Aucun wallet réel, clé, transaction, minage ou service HTTP n'est ajouté.

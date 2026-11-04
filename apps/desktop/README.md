# ARETOVUS Desktop Lab — première interface réellement connectée (V2)

Travail expérimental pour les tickets #181 et #178 ; dépend de **PR #192** et de son binaire Rust `aretovus-local-bridge`. Aucun fonds réel, aucun wallet/signature ni contrôle du minage. Interface React/Vite/TypeScript, hôte local Tauri v2. L'hôte lance un binaire natif placé **à côté de son propre exécutable**, transmet une requête unique et affiche une réponse validée. Aucun navigateur web ordinaire ne peut obtenir ces données via une API publique.

## Compilation de développement (non distribuable)

1. Compiler le nœud et le pont depuis la racine ARETOVUS : `cargo build --locked -p aretovus-node --bin aretovus-node --bin aretovus-local-bridge`.
2. Installer les dépendances JavaScript exactement comme la CI : `cd apps/desktop && npm ci`.
3. Compiler/lancer la partie Tauri : `npm run desktop` (dépendances Tauri/WebView du système requises).
4. **Important :** placer le binaire `aretovus-local-bridge` construit à côté du binaire `aretovus-desktop` produit par Tauri (ou configurer le même dossier cible Cargo de façon maîtrisée). Pas de résolution par PATH ni de binaire téléchargeable depuis la webview.
5. Dans l'application, indiquer un chemin de **snapshot testnet natif déjà créé** puis connecter. La hauteur, le tip et le travail proviennent du vrai nœud via le pont ; la vue se rafraîchit toutes les 20 secondes, avec erreur explicite si le pont est indisponible.

Les fichiers de verrouillage npm et Cargo sont désormais présents sur cette branche, mais la reproductibilité des binaires et la signature des installateurs restent à démontrer. Cette étape ne revendique **pas** un installateur utilisable par le public. Le build CI frontend valide typage/tests, la compilation hôte Tauri et le packaging multi-OS restent à vérifier séparément.

## Sécurité

- Frontend **lecture seule**, ne conserve aucune clé, mot de passe ou seed dans JS.
- Un unique invoke Rust asynchrone : `read_node_status` ; pas de shell, HTTP, plugin shell ou permission filesystem webview.
- Chemin choisi par l'utilisateur canonisé côté hôte ; binaire fixe adjacent à l'exécutable (pas PATH). Entrée et sortie limitées, timeout du processus à 30 s ; stderr supprimé et codes d'erreurs stables.
- Les sorties du bridge sont vérifiées à nouveau dans l'hôte et dans le frontend. Un navigateur seul, même avec un JSON similaire, ne constitue pas un nœud validé.
- Attention : le prototype dépend encore de l'intégrité des exécutables installés et peut consommer du CPU/disque à chaque relecture complète. Aucune garantie de synchronisation globale/P2P active. Distinguer la lecture du snapshot d'un affichage de consensus de réseau.

## Reste à faire

Audit et actualisation contrôlée des lockfiles, intégration packaging binaire côte-à-côte, test hôte Windows/Linux/macOS, cas de processus suspendu, chemin verrouillé, corruption de snapshot, refresh en cas de réorganisation et interface professionnelle. Ensuite wallet graphique testnet, minage et explorateur, via contrats dédiés et revue de sécurité.

## Pages de blocs canoniques (PR empilée #196, dépendance #195)

Le nouveau panneau « Explorateur de blocs » appelle `read_canonical_page` depuis l'hôte Tauri. L'hôte démarre uniquement le binaire Rust compagnon `aretovus-explorer` (PR #195) placé à côté de son propre exécutable, avec un dossier existant, un départ borné et une limite d'au plus 20 blocs. Il capture la sortie sans risque de blocage si elle dépasse la capacité du pipe, impose un timeout de 30 s, borne la réponse à 16 KiB, et **rejette** JSON malformé, discontinuité des parents, nombres JS non sûrs, faux tip et curseurs invalides. Les champs natifs timestamp et nonce (u64) passent dans la webview sous forme de **chaînes décimales** pour éviter tout arrondi JavaScript.

La page est **déclenchée à la demande**, et non à chaque rafraîchissement du nœud, afin de limiter la charge CPU/disque. La vue refuse des pages provenant d'un autre sommet que celui affiché dans le statut local. Il faut rafraîchir le statut puis la page après une réorganisation : l'identité du sommet ne garantit pas une finalité globale. La limite de 10 blocs par page utilisée côté UI est volontairement inférieure à celle du moteur Rust.

**Prérequis supplémentaire :** construire `aretovus-explorer` depuis un commit où la PR #195 est intégrée et placer ce binaire dans le même répertoire approuvé que `aretovus-desktop` et `aretovus-local-bridge`. Le workflow Windows de cette PR ne prouve pas encore le packaging multi-OS, les ACL du dossier d'installation, ni la présence du compagnon chez les utilisateurs. Aucun wallet réel, clé, transaction, minage ou service HTTP n'est ajouté.


## Apparence glassmorphism — amélioration visuelle facultative

Le tableau de bord conserve une identité bleu nuit/or discret, avec des panneaux à effet verre dépoli **lorsque le moteur WebView prend en charge `backdrop-filter`**. Les fonds de remplacement restent suffisamment opaques pour lire les informations sans cet effet ; le texte, les valeurs du nœud et les contrôles restent des éléments React, et non des images.

Principes : aucun CDN, image distante, import CSS externe ou police à télécharger ; bordures et focus clavier visibles, typographie et tableaux adaptés aux petits écrans, `prefers-reduced-motion`, `prefers-reduced-transparency` et `forced-colors` pris en compte au mieux. Le test `python apps/desktop/scripts/test_glass_style.py` vérifie ces contrats statiques et les avertissements de testnet, en complément des tests `npm test` et de la compilation `npm run build`.

Ces contrôles ne remplacent **ni audit WCAG, ni vérification visuelle sur les différents WebView/écrans**. Le thème est purement cosmétique, sans modification de l'IPC Rust, du consensus ou des politiques de portefeuille. 

## Archive Windows portable de laboratoire — PR #199

Une CI GitHub Windows dédiée prépare un **ZIP temporaire non signé**, sans installateur ni accès mainnet. Il comprend `aretovus-desktop.exe`, `aretovus-local-bridge.exe`, `aretovus-explorer.exe` et le `aretovus-node.exe` de testnet. Ces quatre exécutables sont construits depuis **le même SHA Git** et conservés côte à côte afin que l'application ne dépende pas du `PATH` pour ses compagnons. L'archive comporte un `manifest.json`, une liste SHA-256 et des avertissements obligatoires. Les hachages du ZIP ne sont **pas** des signatures numériques.

Le workflow teste la lecture réelle d'une chaîne testnet jetable, crée le ZIP, vérifie sa structure, puis lance de nouveau les compagnons **extraits de l'archive** depuis un dossier temporaire avec espaces. Les tests refusent les fausses signatures PE, les architectures autres qu'AMD64, les archives modifiées ou les erreurs de concurrence sur la destination. Le démarrage réel de la fenêtre dans le ZIP a été ajouté ensuite par la PR #200 ; l'installation WebView2 sur une machine extérieure reste non vérifiée.

Voir [la spécification et ses limites](../../docs/planning/V2_WINDOWS_PORTABLE_LAB.md) et [la PR #199](https://github.com/antoineforterax-cmd/ARETOVUS/pull/199). N'ouvrez pas de fonds réels, ne désactivez pas les protections Windows contre un programme non signé, et n'interprétez pas cet artefact comme une release approuvée.

## Test de lancement de la fenêtre Windows sur le ZIP extrait

La CI `v2-portable-windows-lab` compile **le desktop Tauri en mode `release`**, pour embarquer les ressources frontend React plutôt que de nécessiter un serveur Vite de développement, puis réalise un **smoke test graphique réel** sous Windows, après avoir vérifié les compagnons Rust et l'archive portable. Le script `apps/desktop/scripts/test_desktop_window.py` démarre `aretovus-desktop.exe` **depuis le ZIP extrait** et recherche, au niveau Win32, une fenêtre visible appartenant au PID lancé, avec un titre exact et une stabilité minimale de deux secondes. Il vérifie que les quatre binaires et le contenu du dossier n'ont pas été modifiés par le lancement ; aucune clé, donnée réelle ou commande réseau n'est demandée.

Un processus vivant sans fenêtre, une autre application avec le même titre ou un démarrage sans environnement graphique sont **des échecs, pas des validations**. Les tests du script de surveillance couvrent titre falsifié, fermeture du processus et délai dépassé. Les différences entre les runners GitHub Actions et un bureau Windows personnel restent à vérifier.

Ce test ne valide **ni le rendu WebView2/React ni des clics utilisateurs** ; il n'est pas une attestation d'accessibilité, de signature logicielle ou de sécurité pour fonds réels. Il ne constitue pas un installeur.

## Test E2E React et WebView2 sur un vrai snapshot jetable

Une extension du workflow Windows ajoute `scripts/test_webview_accessibility.ps1` : Windows UI Automation **sur le véritable Tauri extrait de l'archive**. Le test confirme la présence de l'interface React, interagit avec son champ de chemin et ses boutons, contrôle le refus d'un dossier inexistant par la commande native, puis charge un snapshot testnet miné sur place et ouvre le tableau de blocs via le binaire explorateur Rust.

Le résultat doit être visible dans l'arbre d'accessibilité Windows ; un processus simplement ouvert ou une réponse JSON inventée par le script ne comptent pas. Pour ce test Windows seulement, le lanceur active l'accessibilité du renderer WebView2 via le drapeau de diagnostic `--force-renderer-accessibility`. Les jetons GitHub et tout autre argument WebView2 hérité sont retirés ; ce drapeau n'est jamais ajouté aux exécutables du ZIP ni à une configuration de production. Les données sont créées dans un répertoire temporaire, le GUI n'a aucun accès à une clé de portefeuille réelle, aucun nœud public n'est démarré et aucun port de débogage du navigateur n'est ouvert. Cette vérification dépend du provider UI Automation/WebView2 sur le runner GitHub : **si celui-ci n'expose pas les éléments, la CI doit échouer et le chantier reste à corriger**, jamais se marquer réussi par défaut.

À ne pas confondre avec un contrôle visuel pixel-perfect, une mesure du contraste, une validation clavier complète, un E2E Windows physique, un test P2P live ou un audit indépendant. Cette étape n'autorise toujours pas les fonds réels ni le mainnet.

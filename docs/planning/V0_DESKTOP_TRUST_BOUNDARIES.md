# V0/P0 — Frontières de confiance du desktop et des futures API

Statut : **modèle de menace du prototype**, établi sur `main@105e610e3188bf8b73ca244ffdd6e47ba44c1638` le 4 novembre 2026. Références : [feuille V2 #178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178), [ADR V2-001](ADR_V2_DESKTOP_TAURI.md), [politique d'implémentation](IMPLEMENTATION_POLICY.md). Ce document n'autorise ni service réseau, ni signature, ni mainnet. Les observations de CI ne sont pas un audit indépendant.

## État vérifiable et actifs

| Composant | Entrée réellement exposée | Autorité et sortie | Limite |
| --- | --- | --- | --- |
| Nœud Rust | Données de chaîne et P2P testnet, via ses propres CLI/protocoles | Valide selon le consensus existant et écrit son état | Un snapshot local ne prouve pas la finalité du réseau. |
| React dans Tauri | Chemin de dossier et demandes de lecture de page | Affiche une réponse reçue par IPC ; aucune clé brute | Le contenu WebView est moins fiable que l'hôte ; validation UI seule insuffisante. |
| Hôte Tauri Rust | Commandes `read_node_status` et `read_canonical_page` | Canonise le chemin, appelle des exécutables fixes voisins, borne/valide les réponses | Une installation modifiable par un attaquant ne fournit pas de racine de confiance. |
| Compagnons Rust | Requête `status` sur stdin ; page bornée par arguments CLI ; réponses sur stdout | Relisent les données locales testnet en lecture seule | Leur exactitude dépend du binaire, du snapshot et du contrôle des processus. |
| ZIP Windows de laboratoire | Exécutables et manifeste à empreintes SHA-256 | Détecte des altérations accidentelles à la vérification | Non signé : une archive et ses empreintes peuvent être remplacées ensemble. |

Actifs à préserver : clés privées et mots de passe **absents de ce prototype**, intégrité des données du nœud, frontière du répertoire d'installation, ressources CPU/disque et vérité des informations montrées. L'attaquant pertinent peut contrôler le contenu affiché dans une WebView compromise, proposer un chemin local, fournir un snapshot malformé, remplacer des fichiers dans un emplacement où il possède déjà des droits ou envoyer des requêtes à un éventuel futur port local. Il ne doit pas acquérir par l'UI un droit d'écriture, de signature ou de lancement de programme arbitraire.

## Flux autorisés aujourd'hui

1. L'utilisateur choisit un dossier de données **testnet**. React transmet son chemin à Tauri via `invoke`. L'hôte valide/canonise le chemin, ouvre uniquement le binaire voisin `aretovus-local-bridge`, puis valide à nouveau le JSON avant de le rendre à React. La réponse annonce sa provenance locale ; elle ne prouve ni P2P actif ni consensus global.
2. Après un statut validé, l'utilisateur peut demander une page de blocs. `read_canonical_page` appelle uniquement `aretovus-explorer` avec une limite de page et valide la cohérence interne du sommet, la continuité et les bornes. Le frontend `ExplorerPanel` compare ensuite le sommet retourné au `verifiedTip` du statut avant affichage ; cette comparaison dans JS n'est pas une autorisation native. Une réorganisation peut rendre une ancienne page caduque ; l'index dérivé n'est jamais source de vérité du consensus.
3. Le nœud, le wallet CLI et les données de portefeuille ne sont **pas** accessibles par commande Tauri de signature, de transfert, de minage, de lecture de clé ou par socket HTTP du desktop. Le binaire de nœud présent dans le ZIP de laboratoire est séparé de ces deux commandes GUI.

## Scénarios adversariaux et règles de passage

| Scénario | Mesure actuelle | Condition avant extension |
| --- | --- | --- |
| WebView compromise demande un autre chemin ou répète l'IPC | Hôte recanonise ; tailles, délais et concurrence bornés ; réponses et erreurs contrôlées. | Tester jonctions/reparse points, symlinks, répertoire modifié entre contrôle et lecture, saturation et annulation ; aucune sortie brute ni chemin privé dans les logs. |
| Sidecar remplacé ou dossier d'installation inscriptible | Nom fixe voisin de l'exécutable, pas de résolution par `PATH` ; archive de laboratoire vérifiée en CI. | Installation à ACL adaptées, provenance et signature des binaires ; une empreinte stockée dans le même dossier ne suffit pas. |
| JSON, hauteur ou curseur forgés | Hôte et frontend vérifient schéma, bornes et cohérence ; lecture seule. | Ajouter une version de contrat node/UI, erreurs stables, tests de fuzz et reorg ; si une commande native dépend du sommet affiché, valider ce sommet dans Rust également, sans dépendre de JS ; ne jamais convertir un état pending en finalité. |
| Site web externe contacte un futur service localhost | **Pas de socket desktop actuellement.** | Avant de créer le service : liaison loopback seulement, secret éphémère lié à la session, vérification `Host` et `Origin`, refus du DNS rebinding, CORS strict, protection CSRF, quotas et tests de requêtes hostiles. Un port loopback seul n'authentifie pas l'appelant. |
| Wallet GUI ou minage ajoutés | **Aucune commande d'écriture actuellement.** | Signer exclusivement dans Rust sous contrôle local, ne jamais transmettre seed/clé en JS, isoler signer/observer, confirmation utilisateur et verrouillage ; revue dédiée et tests de transfert jetable avant ouverture. |
| CI voit une fenêtre sans React | La PR #201 teste la fenêtre et bloque sur l'absence de vrais contrôles React accessibles. | Résoudre ou qualifier le rendu et faire passer l'interaction réelle ; la fenêtre seule ne valide pas le produit. |

## Vérification et décisions encore ouvertes

- Pour chaque nouvelle commande IPC, documenter : acteur, requête/réponse JSON versionnée, droits, schéma d'erreur, budget temps/taille/concurrence, données sensibles, état après échec et test adversarial. Le frontend ne décide jamais seul de l'autorisation native.
- Avant toute API réseau locale, produire un ADR et des tests négatifs `Origin`/`Host`/token/CSRF/DNS rebinding et refus d'écoute sur interface publique. La présente PR ne crée aucune API socket ni session.
- Avant un wallet graphique : définir le modèle de menace du signer Rust, du stockage chiffré et de la restauration ; aucune récupération de clé définitivement perdue ne peut être promise sans mécanisme de sauvegarde préexistant et consenti.
- Vérifier les flux Windows, Linux et macOS sur de vraies applications. #201 a depuis été fusionnée avec un test d'interaction Windows réussi sur son périmètre jetable ; aucun essai physique sur deux machines, aucune revue E1 par une personne extérieure au projet et aucune décision de distribution ne sont acquis.

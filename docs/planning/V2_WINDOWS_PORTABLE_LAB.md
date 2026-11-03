# ARETOVUS V2 — archive Windows portable de laboratoire (non signée)

**Statut : prototype testnet expérimental, pas un installateur, une release publique, un wallet, ou une autorisation d'utiliser des fonds réels.**

## Objectif borné

Permettre à l'équipe de télécharger depuis **une CI GitHub réussie et identifiée** un seul ZIP contenant exactement quatre exécutables **Windows**, placés côte à côte :

- `aretovus-desktop.exe` : interface Tauri/React avec statut de snapshot local et explorateur de blocs.
- `aretovus-local-bridge.exe` : pont stdio natif de statut, lecture seule.
- `aretovus-explorer.exe` : lecteur natif de blocs canoniques, lecture seule.
- `aretovus-node.exe` : CLI testnet Rust à exécuter **explicitement en dehors de l'interface** pour les scénarios jetables ; ses fonctions de minage ne sont pas exposées par le GUI.

Aucun changement de consensus, de crypto, de clé, de stockage, d'API P2P, ni d'autorisation de service HTTP. L'application n'écrit pas de snapshot ni ne lance automatiquement le nœud réseau.

## Preuve de build prévue

Workflow [`v2-portable-windows-lab.yml`](../../.github/workflows/v2-portable-windows-lab.yml) :

1. Windows GitHub Actions isolé avec permissions `contents: read`, checkout sans identifiants persistants ; aucun secret de release ni publication de version.
2. `npm ci` à partir du lockfile, `npm test`, `npm run build`, vérifications statiques du thème, `cargo test --locked` du desktop.
3. Builds Rust **debug** verrouillés de Tauri et des trois binaires natifs, même checkout/commit.
4. Sur un dossier testnet temporaire avec espaces : miner un bloc jetable avec le vrai nœud Rust, vérifier hauteur/tip par le pont stdio et blocs 0/1 par l'explorateur ; rejeter méthodes de signature, limites de page et chemins absents.
5. Assembleur Python : contrôle présence, type PE Windows x64 (signature DOS `MZ`, signature `PE`, architecture AMD64 et en-tête PE32+, vérification structurelle uniquement), taille maximale et absence de liens symboliques ; ZIP à structure fixe sans chemins arbitraires ; SHA-256 de chaque binaire et `manifest.json` avec SHA exact du checkout, avertissements explicites `signed:false` et `release_approved:false`.
6. Vérification indépendante du ZIP par le script de contrôle, **extraction uniquement après vérification de l'inventaire fixe**, puis exécution des trois vrais binaires de testnet extraits dans un dossier temporaire contenant des espaces. On vérifie de nouveau le bloc miné, le pont IPC, l'explorateur et l'absence de modification des binaires ou de création de fichiers à côté de l'application. Le binaire GUI est présent et haché, **mais aucun test interactif de fenêtre Tauri n'est revendiqué**.
7. Dépôt de l'artefact GitHub (rétention **3 jours**) uniquement si les étapes précédentes passent. Le test adversarial refuse fichiers ajoutés, doublons, données modifiées, manifestes `mainnet`, limites dépassées, SHA incorrect, entrée non Windows et remplacement d'archive.

Exécuter une vérification locale sans extraire ni exécuter les fichiers :

```powershell
py -3 apps/desktop/scripts/package_windows_lab.py verify --archive chemin\vers\aretovus-desktop-Windows-lab-UNSIGNED.zip
```

Le script d'archive refuse également une **course à la création du fichier ZIP** : si un autre processus crée le fichier de destination en premier, son contenu n'est jamais supprimé pendant le nettoyage. Des tests de régression couvrent cette situation et les faux binaires portant uniquement le préfixe `MZ`.

## Limites / risques

- Archive **non signée**. Les checksums présents dans le même ZIP détectent la corruption involontaire et certaines modifications incohérentes, **pas** une attaque modifiant à la fois le binaire et son manifeste. Aucune attestation de build ni authentification cryptographique d'éditeur.
- Fichiers compilés en mode **debug**, peut augmenter taille et RAM ; aucune preuve de build bit-à-bit reproductible d'un runner à l'autre.
- `WebView2`, bibliothèques et runtime Windows peuvent manquer ; aucune vérification de lancement d'interface graphique réelle sur un bureau Windows dans ce workflow.
- Le pont et l'explorateur sont cherchés par chemin adjacent au binaire Tauri. Le dossier d'installation doit être privé et non modifiable par un autre utilisateur ; l'archive ne crée **pas** d'ACL, de sandbox d'installateur, de mise à jour automatique ou de vérification de signature des exécutables compagnons.
- La validation d'un snapshot local ne prouve pas une connexion P2P globale ni une finalité. Ne pas publier les dossiers de données, logs sensibles, seeds ou clés de portefeuille.
- Les tests physiques deux PC, endurance, signatures de release et E1 audit indépendant restent des phases distinctes non réalisées.

**Gate de fusion :** PR associée, CI exacte `completed/success` y compris le job Windows de création et de vérification ; examen du rapport de sécurité, puis tests de la CI du nouveau `main`. Aucune release publique automatiquement créée.

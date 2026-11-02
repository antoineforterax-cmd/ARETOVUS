# ARETOVUS — première interface de consultation du testnet (V2.0)

**Prototype local en lecture seule, pas un wallet graphique terminé.** Deux scripts Python de la V2 produisent une **page HTML à partir d'un vrai dossier de nœud vérifié**. Elle affiche hauteur, blocs connus, tip et travail cumulé sous sa forme de diagnostic, jamais des soldes inventés.

## Prérequis

- Utiliser un dossier de données **ARETOVUS natif**, distinct de celui de l'ancien réseau ; le nœud doit être arrêté ou dans un état que `testnet verify` peut relire sûrement.
- Compiler le nœud sur Windows/Linux/macOS depuis un SHA Git connu : `cargo build --locked -p aretovus-node`.
- Python 3.10+ ; aucun npm, navigateur distant, fichier de clé wallet ou service cloud requis pour cette page.

## Générer la page (Linux/macOS)

```sh
python3 scripts/aretovus_status_json.py \
  --node-binary ./target/debug/aretovus-node \
  --data-dir ./mon-noeud-testnet > aretovus-status.json

python3 scripts/aretovus_snapshot_page.py \
  --status-json ./aretovus-status.json \
  --output ./aretovus-etat-local.html
```

Sous Windows PowerShell, remplacer `python3` par `py -3` et utiliser `target/debug/aretovus-node.exe` comme binaire, avec `--data-dir` pointant sur le vrai dossier testnet.

Ouvrir `aretovus-etat-local.html` directement dans son navigateur. Cette page contient uniquement HTML/CSS statiques, une politique CSP refusant les scripts/ressources réseau, et ne demande aucun compte. Elle n'ouvre pas un serveur HTTP et **ne peut pas envoyer de transaction ou démarrer un mineur**.

## Limites et intégrations futures

- Il faut **régénérer** les deux fichiers pour voir de nouvelles données : aucun rafraîchissement automatique ni réseau P2P en JavaScript.
- Si le JSON est produit directement par l'adaptateur officiel à partir du vrai nœud local, `verified` signifie que ce processus de lecture a réussi. **La page HTML seule ne sait pas authentifier un fichier JSON** : un fichier modifié à la main pourrait mentir, même avec un schéma valide. Elle affiche donc un statut *déclaré*, sans garantir la provenance, le meilleur tip du réseau, ni l'absence d'attaque P2P ou de réorganisation.
- Les fonctionnalités wallet graphique, minage, recherche de transactions/blocs/adresses et carte des pairs seront ajoutées via l'**API locale sécurisée** proposée dans [#180](https://github.com/antoineforterax-cmd/ARETOVUS/issues/180), pas par un accès direct depuis cette page à un socket P2P.
- Cette version ne doit jamais afficher de clés, seed, mot de passe, chemin personnel ou autres données de wallet. **Aucun fonds réel**.

Feuille de route : [#178](https://github.com/antoineforterax-cmd/ARETOVUS/issues/178). Architecture : [APP_V2_ARCHITECTURE.md](../planning/APP_V2_ARCHITECTURE.md).

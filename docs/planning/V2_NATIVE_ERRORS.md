# Erreurs du desktop local — contrat courant

Les commandes Tauri `read_node_status` et `read_canonical_page` renvoient un
objet d'erreur JSON v1, par exemple `{"schema_version":1,"code":"invalid_data_dir"}`.
Rust choisit le code dans une liste fermée. Le frontend associe les codes reconnus
à des messages fixes et garde la compatibilité avec les anciens codes texte.
Une version inconnue, un champ supplémentaire ou un code non reconnu donne le
message générique. Le message affiché ne reprend aucun détail natif.

| Codes | Action présentée |
| --- | --- |
| `invalid_data_dir` | Vérifier le chemin du dossier testnet |
| `bridge_busy` | Attendre la lecture en cours puis réessayer |
| `bridge_timeout`, `explorer_timeout` | Réessayer dans un instant |
| `bridge_unavailable`, `explorer_unavailable` | Vérifier les fichiers de l'application |
| `verification_failed` | Vérifier le dossier et conserver les sauvegardes |
| `explorer_failed` | Actualiser le nœud puis réessayer |
| Autres codes ou valeurs | Message générique propre à la vue |

Les erreurs n'établissent pas un diagnostic de corruption ou de perte de fonds.
Les réponses périmées restent filtrées par les vues. Ce contrat porte sur la
frontière entre Tauri et la WebView. Les réponses de succès et le protocole
stdio du binaire compagnon suivent leurs contrats respectifs.

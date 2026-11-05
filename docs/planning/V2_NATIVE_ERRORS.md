# Erreurs du desktop local — contrat courant

Incrément V0/V2 de la feuille de route #178. Les commandes Tauri en lecture seule
renvoient actuellement un code chaîne en cas d'échec. Le frontend ne présente que
des messages fixes pour les codes reconnus ; tout objet, diagnostic inconnu,
chemin ou trace conserve le message générique. Aucun détail natif n'est interpolé.

| Codes | Action présentée |
| --- | --- |
| `invalid_data_dir` | Vérifier le chemin du dossier testnet |
| `bridge_busy` | Attendre la lecture en cours puis réessayer |
| `bridge_timeout`, `explorer_timeout` | Réessayer lorsque le disque est moins sollicité |
| `bridge_unavailable`, `explorer_unavailable` | Vérifier les fichiers de l'application |
| `verification_failed` | Vérifier le dossier et conserver les sauvegardes |
| `explorer_failed` | Actualiser le nœud puis réessayer |
| Autres codes ou valeurs | Message générique propre à la vue |

Les erreurs n'établissent pas un diagnostic de corruption ou de perte de fonds.
Les réponses périmées restent filtrées par les vues. Le contrat versionné JSON
du processus stdio reste distinct : cet incrément ne livre pas encore un modèle
d'erreur structuré et versionné pour toutes les futures API V0/V1.

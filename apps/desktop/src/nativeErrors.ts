const messages: Readonly<Record<string, string>> = Object.freeze({
  invalid_data_dir: "Dossier introuvable ou inaccessible. Vérifie le chemin du dossier testnet.",
  bridge_busy: "Une lecture est déjà en cours. Attends sa fin, puis réessaie.",
  bridge_timeout: "La vérification prend trop de temps. Réessaie lorsque le disque est moins sollicité.",
  explorer_timeout: "La lecture des blocs prend trop de temps. Réessaie lorsque le disque est moins sollicité.",
  bridge_unavailable: "La vérification locale ne peut pas démarrer. Vérifie que l’application et ses fichiers sont installés ensemble.",
  explorer_unavailable: "L’explorateur local ne peut pas démarrer. Vérifie que l’application et ses fichiers sont installés ensemble.",
  verification_failed: "Le snapshot ne peut pas être validé. Vérifie le dossier testnet et conserve tes sauvegardes.",
  explorer_failed: "Les blocs ne peuvent pas être relus. Actualise le nœud, puis réessaie.",
});

// Native diagnostics are untrusted display input. Only exact allowlisted codes
// may select text; never interpolate exception messages, paths or stack traces.
export function nativeErrorMessage(error: unknown, fallback: string): string {
  return typeof error === "string" && Object.hasOwn(messages, error)
    ? messages[error]
    : fallback;
}

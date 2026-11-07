const messages: Readonly<Record<string, string>> = Object.freeze({
  invalid_data_dir: "Dossier introuvable ou inaccessible. Vérifie le chemin du dossier testnet.",
  bridge_busy: "Une lecture est déjà en cours. Attends sa fin, puis réessaie.",
  bridge_timeout: "La vérification prend trop de temps. Réessaie dans un instant.",
  explorer_timeout: "La lecture des blocs prend trop de temps. Réessaie dans un instant.",
  bridge_unavailable: "La vérification locale ne peut pas démarrer. Vérifie que l’application et ses fichiers sont installés ensemble.",
  explorer_unavailable: "L’explorateur local ne peut pas démarrer. Vérifie que l’application et ses fichiers sont installés ensemble.",
  verification_failed: "Le snapshot ne peut pas être validé. Vérifie le dossier testnet et conserve tes sauvegardes.",
  explorer_failed: "Les blocs ne peuvent pas être relus. Actualise le nœud, puis réessaie.",
});

function versionedErrorCode(error: unknown): unknown {
  if (error === null || typeof error !== "object" || Array.isArray(error)) return undefined;
  if (Object.keys(error).length !== 2) return undefined;
  const version = Object.getOwnPropertyDescriptor(error, "schema_version");
  const code = Object.getOwnPropertyDescriptor(error, "code");
  return version?.value === 1 ? code?.value : undefined;
}

// The old string codes remain readable when developing against an older host.
// Only fixed messages reach the view, even when the payload is malformed.
export function nativeErrorMessage(error: unknown, fallback: string): string {
  const code = typeof error === "string" ? error : versionedErrorCode(error);
  return typeof code === "string" && Object.hasOwn(messages, code)
    ? messages[code]
    : fallback;
}

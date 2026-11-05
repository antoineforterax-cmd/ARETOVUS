import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseCanonicalPage, type CanonicalPage } from "./explorer";

type Props = { dataDir: string; verifiedTip: string | null };
const LIMIT = 10;

export function ExplorerPanel({ dataDir, verifiedTip }: Props) {
  const [page, setPage] = useState<CanonicalPage | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const busy = useRef(false);
  const generation = useRef(0);

  useEffect(() => {
    // Never retain blocks from another user-selected directory or tip.
    generation.current++;
    setPage(null);
    setError("");
  }, [dataDir, verifiedTip]);

  const load = useCallback(async (start: number) => {
    if (!verifiedTip || !dataDir || busy.current) return;
    const expectedGeneration = generation.current;
    busy.current = true;
    setLoading(true);
    try {
      const raw = await invoke<unknown>("read_canonical_page", {
        dataDir, start, limit: LIMIT,
      });
      const validated = parseCanonicalPage(raw, verifiedTip, start, LIMIT);
      if (expectedGeneration === generation.current) {
        setPage(validated);
        setError("");
      }
    } catch {
      if (expectedGeneration === generation.current) {
        setPage(null);
        setError("Lecture des blocs indisponible ou sommet modifié. Actualise le nœud, puis réessaie.");
      }
    } finally {
      busy.current = false;
      // Only one native request can own the bridge. Even when its response is
      // stale, releasing that request must enable the new tip's load button.
      setLoading(false);
    }
  }, [dataDir, verifiedTip]);

  return <section className="explorer">
    <div className="explorer-head">
      <div>
        <h2>Explorateur de blocs</h2>
        <p>Blocs canoniques du snapshot local, pas une vue P2P en direct.</p>
      </div>
      <button type="button" className="refresh"
        disabled={!dataDir || !verifiedTip || loading}
        onClick={() => void load(0)}>
        {loading ? "Chargement…" : "Lire les blocs"}
      </button>
    </div>
    {error && <p className="warning" role="alert">{error}</p>}
    {page && <>
      <div className="explorer-meta">
        <span>Snapshot vérifié localement</span>
        <span>Hauteur {page.chain_height.toLocaleString("fr-FR")}</span>
      </div>
      <div className="explorer-table-scroll">
        <table className="explorer-table">
          <thead><tr><th>Hauteur</th><th>Hash du bloc</th><th>Transactions</th><th>Timestamp (s)</th></tr></thead>
          <tbody>{page.blocks.map(block => <tr key={block.hash}>
            <td>{block.height.toLocaleString("fr-FR")}</td>
            <td><code title={block.hash}>{block.hash}</code></td>
            <td>{block.tx_count}</td>
            <td><code>{block.timestamp_decimal}</code></td>
          </tr>)}</tbody>
        </table>
      </div>
      {page.blocks.length === 0 && <p>Aucun bloc pour cette page du snapshot.</p>}
      <div className="explorer-pagination">
        <button type="button" disabled={loading || page.start === 0}
          onClick={() => void load(Math.max(0, page.start - LIMIT))}>
          Page précédente
        </button>
        <span>Départ : {page.start.toLocaleString("fr-FR")}</span>
        <button type="button" disabled={loading || page.next_start === null}
          onClick={() => { if (page.next_start !== null) void load(page.next_start); }}>
          Page suivante
        </button>
      </div>
      <p className="explorer-note">Le sommet d'une chaîne peut changer après une réorganisation. Les pages sont liées au même hash de sommet et refusées en cas d'écart. Aucune transaction n'est envoyée.</p>
    </>}
    {!page && !error && <p className="explorer-empty">Connecte ton nœud, puis lis une page de ses véritables blocs.</p>}
  </section>;
}

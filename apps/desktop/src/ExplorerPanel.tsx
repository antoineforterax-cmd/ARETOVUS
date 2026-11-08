import { useCallback, useId, useLayoutEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseCanonicalPage, type CanonicalPage } from "./explorer";
import { nativeErrorMessage } from "./nativeErrors";
import type { VerifiedNodeStatus } from "./status";

type Props = {
  dataDir: string;
  snapshot: Pick<VerifiedNodeStatus, "tip" | "height"> | null;
};
const LIMIT = 10;

export function ExplorerPanel({ dataDir, snapshot }: Props) {
  const verifiedTip = snapshot?.tip;
  const verifiedHeight = snapshot?.height;
  const [page, setPage] = useState<CanonicalPage | null>(null);
  const [heightQuery, setHeightQuery] = useState("");
  const [foundHeight, setFoundHeight] = useState<number | null>(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const busy = useRef(false);
  const generation = useRef(0);
  const heightInputId = useId();
  const heightHintId = useId();
  const hasSnapshot = !!dataDir && !!verifiedTip && verifiedHeight !== undefined;
  const enteredHeight = /^[0-9]{1,16}$/.test(heightQuery) ? Number(heightQuery) : NaN;
  const validHeight = verifiedHeight !== undefined && Number.isSafeInteger(enteredHeight)
    && enteredHeight <= verifiedHeight;

  useLayoutEffect(() => {
    // Clear the old snapshot before it can paint or anchor a new request.
    generation.current++;
    setPage(null);
    setFoundHeight(null);
    setError("");
  }, [dataDir, verifiedTip, verifiedHeight]);

  const load = useCallback(async (start: number, targetHeight: number | null = null) => {
    if (!verifiedTip || verifiedHeight === undefined || !dataDir || busy.current) return;
    const expectedGeneration = generation.current;
    busy.current = true;
    setLoading(true);
    try {
      const raw = await invoke<unknown>("read_canonical_page", {
        dataDir, start, limit: LIMIT,
      });
      const validated = parseCanonicalPage(raw, verifiedTip, start, LIMIT);
      if (validated.chain_height !== verifiedHeight) throw new Error("snapshot_height_mismatch");
      if (expectedGeneration === generation.current) {
        // Each cursor has already been checked against its page's block count.
        if (page && (
          (page.next_start === validated.start && page.blocks.at(-1)?.hash !== validated.blocks[0]?.parent)
          || (validated.next_start === page.start && validated.blocks.at(-1)?.hash !== page.blocks[0]?.parent)
        )) {
          throw new Error("page_parent_mismatch");
        }
        setPage(validated);
        setFoundHeight(targetHeight);
        setError("");
      }
    } catch (error) {
      if (expectedGeneration === generation.current) {
        setPage(null);
        setError(nativeErrorMessage(error, "Lecture des blocs indisponible ou sommet modifié. Actualise le nœud, puis réessaie."));
      }
    } finally {
      busy.current = false;
      // Only one native request can own the bridge. Even when its response is
      // stale, releasing that request must enable the new tip's load button.
      setLoading(false);
    }
  }, [dataDir, page, verifiedTip, verifiedHeight]);

  return <section className="explorer">
    <div className="explorer-head">
      <div>
        <h2>Explorateur de blocs</h2>
        <p>Blocs canoniques du snapshot local, pas une vue P2P en direct.</p>
      </div>
      <button type="button" className="refresh"
        disabled={!hasSnapshot || loading}
        onClick={() => void load(0)}>
        {loading ? "Chargement…" : "Lire les blocs"}
      </button>
    </div>
    {hasSnapshot && <form className="explorer-search" onSubmit={event => {
      event.preventDefault();
      if (!validHeight || loading) return;
      // Keep every jump aligned with the existing previous/next page buttons.
      void load(enteredHeight - enteredHeight % LIMIT, enteredHeight);
    }}>
      <label htmlFor={heightInputId}>Hauteur du bloc</label>
      <div className="formline">
        <input id={heightInputId} type="text" inputMode="numeric" value={heightQuery}
          disabled={loading} autoComplete="off"
          aria-describedby={heightHintId} aria-invalid={heightQuery !== "" && !validHeight}
          onChange={event => setHeightQuery(event.target.value)} />
        <button type="submit" disabled={!validHeight || loading}>Aller au bloc</button>
      </div>
      <small id={heightHintId}>Saisis une hauteur entière de 0 à {verifiedHeight?.toLocaleString("fr-FR")}.</small>
    </form>}
    {error && <p className="warning" role="alert">{error}</p>}
    {page && <>
      <div className="explorer-meta">
        <span>Snapshot vérifié localement</span>
        <span>Hauteur {page.chain_height.toLocaleString("fr-FR")}</span>
      </div>
      {foundHeight !== null && <p role="status">Bloc {foundHeight.toLocaleString("fr-FR")} trouvé dans le snapshot.</p>}
      <div className="explorer-table-scroll">
        <table className="explorer-table">
          <thead><tr><th>Hauteur</th><th>Hash du bloc</th><th>Transactions</th><th>Timestamp (s)</th></tr></thead>
          <tbody>{page.blocks.map(block => <tr key={block.hash}>
            <td>{block.height.toLocaleString("fr-FR")}{block.height === foundHeight && <strong> · Recherché</strong>}</td>
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
      <p className="explorer-note">Le sommet d'une chaîne peut changer après une réorganisation. Les pages doivent correspondre au sommet vérifié et deux pages voisines doivent se raccorder. Aucune transaction n'est envoyée.</p>
    </>}
    {!page && !error && <p className="explorer-empty">Connecte ton nœud, puis lis une page de ses véritables blocs.</p>}
  </section>;
}

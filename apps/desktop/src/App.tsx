import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { parseNodeStatus, type VerifiedNodeStatus } from "./status";
import { ExplorerPanel } from "./ExplorerPanel";
import { nativeErrorCode, nativeErrorMessage } from "./nativeErrors";

export function App() {
  const [folder, setFolder] = useState("");
  const [selected, setSelected] = useState("");
  const [data, setData] = useState<VerifiedNodeStatus | null>(null);
  const [error, setError] = useState("");
  const [updated, setUpdated] = useState<Date | null>(null);
  const busy = useRef(false);
  const activeDir = useRef("");
  const pendingDir = useRef<string | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async function verifyLocalNode(dataDir: string): Promise<void> {
    if (busy.current) {
      // Coalesce requests: only the currently selected directory may win.
      pendingDir.current = dataDir;
      return;
    }
    busy.current = true;
    setLoading(true);
    try {
      const response = await invoke<unknown>("read_node_status", { dataDir });
      const verified = parseNodeStatus(response);
      if (activeDir.current === dataDir) {
        setData(verified);
        setError("");
        setUpdated(new Date());
      }
    } catch (error) {
      if (activeDir.current === dataDir) {
        setError(nativeErrorMessage(error, "Lecture impossible : dossier, binaire ou snapshot invalide. Vérifie le testnet local."));
        // A busy bridge has not verified a newer snapshot. Keep the last
        // verified tip so the explorer read holding the lease can finish.
        if (nativeErrorCode(error) !== "bridge_busy") {
          setData(null);
          setUpdated(null);
        }
      }
    } finally {
      busy.current = false;
      if (activeDir.current === dataDir) setLoading(false);
      const next = pendingDir.current;
      pendingDir.current = null;
      if (next && next === activeDir.current) {
        void verifyLocalNode(next);
      } else if (activeDir.current && activeDir.current !== dataDir) {
        void verifyLocalNode(activeDir.current);
      } else if (!activeDir.current) {
        setLoading(false);
      }
    }
  }, []);

  const connect = (nextDir: string) => {
    if (!nextDir) return;
    if (nextDir === selected) {
      void refresh(nextDir);
      return;
    }
    // Mark the old asynchronous response stale *before* React rerenders.
    activeDir.current = nextDir;
    setData(null);
    setUpdated(null);
    setError("");
    setSelected(nextDir);
  };

  useEffect(() => {
    activeDir.current = selected;
    if (!selected) return;
    void refresh(selected);
    // Each refresh invokes native verification; never a public HTTP API.
    const timer = window.setInterval(() => void refresh(selected), 20_000);
    return () => window.clearInterval(timer);
  }, [selected, refresh]);

  return <div className="app">
    <header className="top"><div className="mark">A</div><div className="brand">ARETOVUS <span>DESKTOP / LAB</span></div><div className="network">TESTNET EXPÉRIMENTAL</div></header>
    <main>
      <section className="hero"><div className="eyebrow">NŒUD LOCAL • LECTURE SEULE</div><h1>Ton réseau.<br /><em>Ton nœud.</em></h1>
        <p>Affichage de l'état réel relu par le pont Rust. Un instantané local n'est ni une connexion P2P prouvée ni une garantie de finalité.</p></section>
      <form className="connect" onSubmit={e => { e.preventDefault(); connect(folder.trim()); }}>
        <label htmlFor="path">Dossier de données ARETOVUS (testnet uniquement)</label>
        <div className="formline"><input id="path" value={folder} maxLength={1024} onChange={e => setFolder(e.target.value)} placeholder="Chemin local du dossier testnet" autoComplete="off" spellCheck={false}/>
        <button disabled={!folder.trim() || loading} type="submit">{loading ? "Vérification…" : "Connecter le nœud"}</button></div>
        <small>Le chemin reste dans l'application locale. Aucun wallet, clé ou mot de passe n'est demandé.</small>
      </form>
      <div className="statusline" role="status">{data ? <span className="ok">● Snapshot local validé</span> : <span className="muted">● {loading ? "Lecture locale en cours" : error ? "Nœud indisponible" : "Aucun nœud connecté"}</span>}
      {updated && <time>Dernière lecture : {updated.toLocaleTimeString("fr-FR")}</time>}</div>
      {error && <p className="warning" role="alert">{error}</p>}
      <div className="cards">
        <article><span className="hint">Hauteur validée</span><strong>{data?.height.toLocaleString("fr-FR") ?? "—"}</strong><small>Chaîne locale</small></article>
        <article><span className="hint">Blocs connus</span><strong>{data?.known_blocks.toLocaleString("fr-FR") ?? "—"}</strong><small>Branches incluses</small></article>
        <article><span className="hint">Réseau</span><strong>{data?.network.toUpperCase() ?? "—"}</strong><small>Pas de mesure de peers</small></article>
      </div>
      <section className="details"><h2>État vérifié par Rust</h2><div className="row"><span>Sommet (tip)</span><code>{data?.tip ?? "Non disponible"}</code></div>
      <div className="row"><span>Travail cumulé (diagnostic)</span><code>{data?.chain_work_display ?? "Non disponible"}</code></div>
      <button className="refresh" type="button" disabled={!selected || loading} onClick={() => void refresh(selected)}>Actualiser maintenant</button></section>
      <ExplorerPanel dataDir={selected} verifiedTip={data?.tip ?? null} />
      <section className="roadmap"><h2>À intégrer ensuite</h2><div className="next"><div>Wallet graphique <small>Non disponible</small></div><div>Minage contrôlé <small>Non disponible</small></div><div>Visualisation des pairs P2P <small>Non disponible</small></div></div></section>
    </main><footer>ARETOVUS V2 — prototype de laboratoire. Pas de fonds réels. Aucune action de minage, signature ou transaction dans cette version.</footer>
  </div>;
}

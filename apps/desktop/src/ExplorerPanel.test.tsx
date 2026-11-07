// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { App } from "./App";
import { ExplorerPanel } from "./ExplorerPanel";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const FIRST_TIP = "a".repeat(64);
const NEXT_TIP = "b".repeat(64);
let root: Root | null = null;
let host: HTMLDivElement | null = null;

afterEach(async () => {
  if (root) await act(async () => root!.unmount());
  root = null;
  host?.remove();
  host = null;
  vi.resetAllMocks();
  vi.useRealTimers();
});

it("keeps a verified explorer request when the status refresh finds the bridge busy", async () => {
  vi.useFakeTimers();
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);

  let resolvePage!: (value: unknown) => void;
  vi.mocked(invoke)
    .mockResolvedValueOnce({
      schema_version: 1, network: "testnet", status: "verified", read_only: true,
      source: "aretovus-native-stdio", height: 0, known_blocks: 1, tip: FIRST_TIP,
      chain_work_display: "ChainWork([0, 1])",
    })
    .mockImplementationOnce(() => new Promise(resolve => { resolvePage = resolve; }))
    .mockRejectedValueOnce({ schema_version: 1, code: "bridge_busy" })
    .mockRejectedValueOnce({ schema_version: 1, code: "verification_failed" });

  await act(async () => root!.render(<App />));
  const input = host.querySelector<HTMLInputElement>("#path")!;
  const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!;
  await act(async () => {
    setValue.call(input, "testnet-a");
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  await act(async () => host!.querySelector<HTMLButtonElement>("form button")!.click());
  expect(host.textContent).toContain("Snapshot local validé");
  await act(async () => host!.querySelector<HTMLButtonElement>(".explorer-head button")!.click());

  await act(async () => vi.advanceTimersByTimeAsync(20_000));
  expect(vi.mocked(invoke).mock.calls.map(([command]) => command)).toEqual([
    "read_node_status", "read_canonical_page", "read_node_status",
  ]);
  expect(host.textContent).toContain("Snapshot local validé");
  expect(host.textContent).toContain("Une lecture est déjà en cours");
  await act(async () => resolvePage({
    schema_version: 1, network: "testnet", status: "verified_local", read_only: true,
    source: "aretovus-native-explorer", chain_tip: FIRST_TIP, chain_height: 0,
    start: 0, next_start: null, blocks: [{
      height: 0, hash: FIRST_TIP, parent: "0".repeat(64), merkle_root: "0".repeat(64),
      timestamp_decimal: "1", nonce_decimal: "0", tx_count: 0,
    }],
  }));
  expect(host.querySelector("table")?.textContent).toContain(FIRST_TIP);

  await act(async () => vi.advanceTimersByTimeAsync(20_000));
  expect(host.textContent).not.toContain("Snapshot local validé");
  expect(host.querySelector("table")).toBeNull();
});

it("hides a stale versioned error and displays the error of the current request", async () => {
  // React needs a real DOM to exercise effects, UI state and the click path.
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
  let rejectRequest!: (reason: unknown) => void;
  vi.mocked(invoke).mockImplementation(() => new Promise((_, reject) => {
    rejectRequest = reject;
  }));

  await act(async () => root!.render(<ExplorerPanel dataDir="testnet-a" verifiedTip={FIRST_TIP} />));
  const button = () => host!.querySelector<HTMLButtonElement>(".explorer-head button")!;
  await act(async () => button().click());
  expect(button().disabled).toBe(true);

  await act(async () => root!.render(<ExplorerPanel dataDir="testnet-b" verifiedTip={NEXT_TIP} />));
  expect(button().disabled).toBe(true); // One native request still owns the bridge.
  await act(async () => rejectRequest({ schema_version: 1, code: "explorer_timeout" }));
  expect(button().disabled).toBe(false);
  expect(button().textContent).toBe("Lire les blocs");
  expect(host!.querySelector('[role="alert"]')).toBeNull();
  await act(async () => button().click());
  expect(vi.mocked(invoke)).toHaveBeenCalledTimes(2);
  expect(vi.mocked(invoke)).toHaveBeenLastCalledWith("read_canonical_page", {
    dataDir: "testnet-b", start: 0, limit: 10,
  });
  expect(button().disabled).toBe(true);
  await act(async () => rejectRequest({ schema_version: 1, code: "explorer_timeout" }));
  expect(button().disabled).toBe(false);
  expect(host!.querySelector('[role="alert"]')?.textContent).toContain("prend trop de temps");
});

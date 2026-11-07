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

  await act(async () => root!.render(<ExplorerPanel dataDir="testnet-a" snapshot={{ tip: FIRST_TIP, height: 0 }} />));
  const button = () => host!.querySelector<HTMLButtonElement>(".explorer-head button")!;
  await act(async () => button().click());
  expect(button().disabled).toBe(true);

  await act(async () => root!.render(<ExplorerPanel dataDir="testnet-b" snapshot={{ tip: NEXT_TIP, height: 0 }} />));
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

it("jumps to a verified block height and keeps pages aligned", async () => {
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);

  const tipHeight = 12;
  const hashAt = (height: number) => height === tipHeight ? FIRST_TIP : height.toString(16).padStart(64, "0");
  const pageAt = (start: number) => ({
    schema_version: 1, network: "testnet", status: "verified_local", read_only: true,
    source: "aretovus-native-explorer", chain_tip: FIRST_TIP, chain_height: tipHeight,
    start, next_start: start === 0 ? 10 : null,
    blocks: Array.from({ length: Math.min(10, tipHeight + 1 - start) }, (_, index) => {
      const height = start + index;
      return {
        height, hash: hashAt(height), parent: hashAt(Math.max(0, height - 1)),
        timestamp_decimal: "1", nonce_decimal: "0", tx_count: 0, merkle_root: hashAt(0),
      };
    }),
  });
  vi.mocked(invoke)
    .mockResolvedValueOnce(pageAt(10))
    .mockResolvedValueOnce(pageAt(0))
    .mockResolvedValueOnce({ ...pageAt(0), chain_height: tipHeight + 1 });

  await act(async () => root!.render(<ExplorerPanel dataDir="testnet-a" snapshot={null} />));
  expect(host.querySelector(".explorer-search")).toBeNull();
  await act(async () => root!.render(
    <ExplorerPanel dataDir="testnet-a" snapshot={{ tip: FIRST_TIP, height: tipHeight }} />,
  ));
  const form = host.querySelector<HTMLFormElement>(".explorer-search")!;
  const input = form.querySelector("input")!;
  const button = form.querySelector("button")!;
  const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!;
  const typeHeight = async (value: string) => act(async () => {
    setValue.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  expect(form.querySelector("label")?.htmlFor).toBe(input.id);
  for (const invalid of ["13", "-1", "1.2", "1e1", "9007199254740992"]) {
    await typeHeight(invalid);
    expect(input.getAttribute("aria-invalid")).toBe("true");
    expect(button.disabled).toBe(true);
    await act(async () => { form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true })); });
  }
  expect(invoke).not.toHaveBeenCalled();

  await typeHeight("12");
  expect(button.disabled).toBe(false);
  await act(async () => button.click());
  expect(invoke).toHaveBeenLastCalledWith("read_canonical_page", { dataDir: "testnet-a", start: 10, limit: 10 });
  expect(host.querySelector('[role="status"]')?.textContent).toContain("Bloc 12 trouvé");
  expect(host.querySelector("tr:last-child td")?.textContent).toContain("12 · Recherché");

  const [previous, next] = host.querySelectorAll<HTMLButtonElement>(".explorer-pagination button");
  expect(next.disabled).toBe(true);
  await act(async () => previous.click());
  expect(invoke).toHaveBeenLastCalledWith("read_canonical_page", { dataDir: "testnet-a", start: 0, limit: 10 });
  expect(host.querySelector('[role="status"]')).toBeNull();

  await act(async () => host!.querySelector<HTMLButtonElement>(".explorer-head button")!.click());
  expect(host.querySelector("table")).toBeNull();
  expect(host.querySelector('[role="alert"]')?.textContent).toContain("Lecture des blocs indisponible");
});

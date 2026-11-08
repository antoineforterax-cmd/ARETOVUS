// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { App } from "./App";
import { parseCanonicalPage } from "./explorer";
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

const CHAIN_HEIGHT = 24;
const FOREIGN_HASH = "f".repeat(64);

function linkedPage(start: number, tip = FIRST_TIP) {
  const hashAt = (height: number) => height === CHAIN_HEIGHT ? tip : (height + 1).toString(16).padStart(64, "0");
  return {
    schema_version: 1, network: "testnet", status: "verified_local", read_only: true,
    source: "aretovus-native-explorer", chain_tip: tip, chain_height: CHAIN_HEIGHT,
    start, next_start: start + 10 <= CHAIN_HEIGHT ? start + 10 : null,
    blocks: Array.from({ length: Math.min(10, CHAIN_HEIGHT - start + 1) }, (_, index) => {
      const height = start + index;
      return {
        height, hash: hashAt(height), parent: height === 0 ? "0".repeat(64) : hashAt(height - 1),
        timestamp_decimal: "1", nonce_decimal: "0", tx_count: 0, merkle_root: "0".repeat(64),
      };
    }),
  };
}

async function renderExplorer(tip = FIRST_TIP) {
  if (!root) {
    Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
    host = document.createElement("div");
    document.body.appendChild(host);
    root = createRoot(host);
  }
  await act(async () => root!.render(
    <ExplorerPanel dataDir="testnet-a" snapshot={{ tip, height: CHAIN_HEIGHT }} />,
  ));
}

async function clickExplorerButton(label: string) {
  const button = Array.from(host!.querySelectorAll("button")).find(button => button.textContent === label);
  expect(button?.disabled).toBe(false);
  await act(async () => button!.click());
}

async function searchHeight(height: number) {
  const input = host!.querySelector<HTMLInputElement>(".explorer-search input")!;
  const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!;
  await act(async () => {
    setValue.call(input, String(height));
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  await clickExplorerButton("Aller au bloc");
}

it.each(["next", "previous", "search", "hash"] as const)("rejects a broken adjacent parent via %s", async route => {
  const start = route === "previous" ? 0 : 10;
  const response = linkedPage(start);
  if (route === "previous") {
    response.blocks.at(-1)!.hash = FOREIGN_HASH;
  } else {
    response.blocks[0].parent = FOREIGN_HASH;
  }
  expect(parseCanonicalPage(response, FIRST_TIP, start, 10).start).toBe(start);
  vi.mocked(invoke).mockResolvedValueOnce(linkedPage(route === "previous" ? 10 : 0))
    .mockResolvedValueOnce(response);

  await renderExplorer();
  if (route === "previous") {
    await searchHeight(10);
  } else {
    await clickExplorerButton("Lire les blocs");
  }
  expect(host!.querySelector("table")).not.toBeNull();

  if (route === "hash") {
    const blockHash = response.blocks[2].hash;
    await enterHash(blockHash);
    await clickExplorerButton("Chercher par hash");
    expect(invoke).toHaveBeenLastCalledWith("read_canonical_page_by_hash", {
      dataDir: "testnet-a", blockHash, limit: 10, expectedTip: FIRST_TIP, expectedHeight: CHAIN_HEIGHT,
    });
  } else if (route === "search") {
    await searchHeight(12);
  } else {
    await clickExplorerButton(route === "next" ? "Page suivante" : "Page précédente");
  }
  if (route !== "hash") {
    expect(invoke).toHaveBeenLastCalledWith("read_canonical_page", { dataDir: "testnet-a", start, limit: 10 });
  }
  expect(host!.querySelector("table")).toBeNull();
  expect(host!.querySelector('[role="alert"]')?.textContent).toContain("Lecture des blocs indisponible");
});

it("allows a valid round trip between adjacent pages and a nonadjacent jump", async () => {
  const lastPage = linkedPage(20);
  lastPage.blocks[0].parent = FOREIGN_HASH;
  vi.mocked(invoke).mockResolvedValueOnce(linkedPage(0))
    .mockResolvedValueOnce(linkedPage(10))
    .mockResolvedValueOnce(linkedPage(0))
    .mockResolvedValueOnce(lastPage);

  await renderExplorer();
  await clickExplorerButton("Lire les blocs");
  await clickExplorerButton("Page suivante");
  expect(host!.querySelector("tbody tr td")?.textContent).toBe("10");
  await clickExplorerButton("Page précédente");
  expect(host!.querySelector("tbody tr td")?.textContent).toBe("0");
  await searchHeight(24);
  expect(host!.querySelector("tbody tr td")?.textContent).toBe("20");
  expect(host!.querySelector('[role="status"]')?.textContent).toContain("Bloc 24 trouvé");
  expect(host!.querySelector('[role="alert"]')).toBeNull();
});

it("ignores a pending adjacent response after the verified snapshot changes", async () => {
  let resolveOldPage!: (value: unknown) => void;
  const oldPage = linkedPage(10);
  oldPage.blocks[0].parent = FOREIGN_HASH;
  vi.mocked(invoke).mockResolvedValueOnce(linkedPage(0))
    .mockImplementationOnce(() => new Promise(resolve => { resolveOldPage = resolve; }))
    .mockResolvedValueOnce(linkedPage(20, NEXT_TIP));

  await renderExplorer();
  await clickExplorerButton("Lire les blocs");
  await clickExplorerButton("Page suivante");
  await renderExplorer(NEXT_TIP);
  await act(async () => resolveOldPage(oldPage));

  expect(host!.querySelector("table")).toBeNull();
  expect(host!.querySelector('[role="alert"]')).toBeNull();
  await searchHeight(24);
  expect(host!.querySelector("tbody")?.textContent).toContain(NEXT_TIP);
});

async function enterHash(value: string) {
  const input = host!.querySelector<HTMLInputElement>(".explorer-hash-search input");
  if (!input) throw new Error("The explorer hash input is missing");
  const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")!.set!;
  await act(async () => {
    setValue.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
  });
  return input;
}

it("only submits complete ASCII hashes and locates the matching block", async () => {
  const foundPage = linkedPage(10);
  const target = foundPage.blocks[3];
  vi.mocked(invoke).mockResolvedValueOnce(foundPage);
  await renderExplorer();

  for (const malformed of [target.hash.slice(1), target.hash + "0", target.hash + " ",
    "Ｆ".repeat(64), "--limit=20"]) {
    const input = await enterHash(malformed);
    const form = input.form!;
    expect(input.getAttribute("aria-invalid")).toBe("true");
    expect(form.querySelector("button")?.disabled).toBe(true);
    await act(async () => { form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true })); });
  }
  expect(invoke).not.toHaveBeenCalled();

  const input = await enterHash(target.hash.toUpperCase());
  expect(input.form?.querySelector("label")?.htmlFor).toBe(input.id);
  await clickExplorerButton("Chercher par hash");
  expect(invoke).toHaveBeenCalledExactlyOnceWith("read_canonical_page_by_hash", {
    dataDir: "testnet-a", blockHash: target.hash, limit: 10, expectedTip: FIRST_TIP,
    expectedHeight: CHAIN_HEIGHT,
  });
  expect(host!.querySelector('[role="status"]')?.textContent).toContain("Bloc 13 trouvé");
  expect(host!.querySelector("tbody strong")?.closest("tr")?.textContent).toContain(target.hash);
  expect(host!.querySelector('[role="alert"]')).toBeNull();
});

it.each([
  ["missing hash", linkedPage(10), FOREIGN_HASH],
  ["misaligned start", linkedPage(11), linkedPage(11).blocks[2].hash],
  ["different tip", linkedPage(10, NEXT_TIP), linkedPage(10).blocks[3].hash],
  ["different height", { ...linkedPage(10), chain_height: CHAIN_HEIGHT + 1 }, linkedPage(10).blocks[3].hash],
  ["non-numeric start", { ...linkedPage(10), start: "10" }, linkedPage(10).blocks[3].hash],
] as const)("rejects a hash lookup response with %s", async (_case, response, target) => {
  vi.mocked(invoke).mockResolvedValueOnce(response);
  await renderExplorer();
  await enterHash(target);
  await clickExplorerButton("Chercher par hash");
  expect(host!.querySelector("table")).toBeNull();
  expect(host!.querySelector('[role="status"]')).toBeNull();
  expect(host!.querySelector('[role="alert"]')?.textContent).toContain("Lecture des blocs indisponible");
});

it("displays the fixed not-found error from the native hash lookup", async () => {
  vi.mocked(invoke).mockRejectedValueOnce({ schema_version: 1, code: "explorer_block_not_found" });
  await renderExplorer();
  await enterHash(FOREIGN_HASH);
  await clickExplorerButton("Chercher par hash");
  expect(host!.querySelector("table")).toBeNull();
  expect(host!.querySelector('[role="alert"]')?.textContent).toContain("Bloc introuvable");
});

it("discards a pending hash lookup after the verified snapshot changes", async () => {
  let resolveOldPage!: (value: unknown) => void;
  const oldPage = linkedPage(10);
  vi.mocked(invoke).mockImplementationOnce(() => new Promise(resolve => { resolveOldPage = resolve; }))
    .mockResolvedValueOnce(linkedPage(20, NEXT_TIP));

  await renderExplorer();
  await enterHash(oldPage.blocks[3].hash);
  await clickExplorerButton("Chercher par hash");
  await renderExplorer(NEXT_TIP);
  expect(host!.querySelector<HTMLButtonElement>(".explorer-hash-search button")?.disabled).toBe(true);
  await act(async () => resolveOldPage(oldPage));
  expect(host!.querySelector("table")).toBeNull();
  expect(host!.querySelector('[role="alert"]')).toBeNull();

  await enterHash(NEXT_TIP);
  await clickExplorerButton("Chercher par hash");
  expect(host!.querySelector('[role="status"]')?.textContent).toContain("Bloc 24 trouvé");
});

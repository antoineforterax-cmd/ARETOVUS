// @vitest-environment jsdom
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
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
  vi.clearAllMocks();
});

it("re-enables block reading when an obsolete request settles after a tip change", async () => {
  // React needs a real DOM to exercise effects, UI state and the click path.
  Object.assign(globalThis, { IS_REACT_ACT_ENVIRONMENT: true });
  host = document.createElement("div");
  document.body.appendChild(host);
  root = createRoot(host);
  let resolveOld!: (value: unknown) => void;
  vi.mocked(invoke).mockImplementation(() => new Promise(resolve => { resolveOld = resolve; }));

  await act(async () => root!.render(<ExplorerPanel dataDir="testnet-a" verifiedTip={FIRST_TIP} />));
  const button = () => host!.querySelector<HTMLButtonElement>(".explorer-head button")!;
  await act(async () => button().click());
  expect(button().disabled).toBe(true);

  await act(async () => root!.render(<ExplorerPanel dataDir="testnet-b" verifiedTip={NEXT_TIP} />));
  expect(button().disabled).toBe(true); // One native request still owns the bridge.
  await act(async () => resolveOld(null));
  expect(button().disabled).toBe(false);
  expect(button().textContent).toBe("Lire les blocs");
  expect(host!.textContent).not.toContain("Lecture des blocs indisponible");
  await act(async () => button().click());
  expect(vi.mocked(invoke)).toHaveBeenCalledTimes(2);
  expect(vi.mocked(invoke)).toHaveBeenLastCalledWith("read_canonical_page", {
    dataDir: "testnet-b", start: 0, limit: 10,
  });
  expect(button().disabled).toBe(true);
  await act(async () => resolveOld(null));
  expect(button().disabled).toBe(false);
});

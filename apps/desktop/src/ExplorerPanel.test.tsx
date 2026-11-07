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

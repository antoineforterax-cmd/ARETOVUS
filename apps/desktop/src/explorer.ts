/** The desktop explorer may only render data returned by the native local Rust
 * explorer, validated by the Tauri host and tied to the currently visible tip. */
export type CanonicalBlock = {
  height: number;
  hash: string;
  parent: string;
  timestamp_decimal: string;
  nonce_decimal: string;
  tx_count: number;
  merkle_root: string;
};
export type CanonicalPage = {
  schema_version: 1;
  network: "testnet";
  status: "verified_local";
  read_only: true;
  source: "aretovus-native-explorer";
  chain_tip: string;
  chain_height: number;
  start: number;
  next_start: number | null;
  blocks: CanonicalBlock[];
};
const HEX = /^[0-9a-f]{64}$/;
const DECIMAL = /^(0|[1-9][0-9]{0,19})$/;
const MAX_U64 = 18_446_744_073_709_551_615n;
const uint64Decimal = (value: unknown): value is string =>
  typeof value === "string" && DECIMAL.test(value) && BigInt(value) <= MAX_U64;
const safe = (v: unknown): v is number =>
  typeof v === "number" && Number.isSafeInteger(v) && v >= 0;

export function parseCanonicalPage(
  raw: unknown,
  expectedTip: string,
  expectedStart: number,
  requestedLimit = 10,
): CanonicalPage {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) throw Error("invalid_page");
  if (!HEX.test(expectedTip) || !safe(expectedStart) ||
      !Number.isInteger(requestedLimit) || requestedLimit < 1 || requestedLimit > 20) {
    throw Error("invalid_request");
  }
  const page = raw as Record<string, unknown>;
  if (page.schema_version !== 1 || page.network !== "testnet" ||
      page.status !== "verified_local" || page.read_only !== true ||
      page.source !== "aretovus-native-explorer" ||
      page.chain_tip !== expectedTip || !safe(page.chain_height) ||
      page.start !== expectedStart || !Array.isArray(page.blocks)) {
    throw Error("invalid_page");
  }
  const height = page.chain_height;
  const expectedCount = Math.min(requestedLimit, Math.max(0, height - expectedStart + 1));
  if (page.blocks.length !== expectedCount) {
    throw Error("invalid_page");
  }
  let previousHash: string | null = null;
  const hashes = new Set<string>();
  const entries: CanonicalBlock[] = page.blocks.map((entry: unknown, i: number) => {
    if (!entry || typeof entry !== "object" || Array.isArray(entry)) throw Error("invalid_page");
    const block = entry as Record<string, unknown>;
    const expectedHeight = expectedStart + i;
    if (!safe(expectedHeight) || !safe(block.height) ||
        block.height !== expectedHeight || expectedHeight > height ||
        typeof block.hash !== "string" || !HEX.test(block.hash) ||
        hashes.has(block.hash) || (block.hash === expectedTip) !== (expectedHeight === height) ||
        typeof block.parent !== "string" || !HEX.test(block.parent) ||
        typeof block.merkle_root !== "string" || !HEX.test(block.merkle_root) ||
        !uint64Decimal(block.timestamp_decimal) ||
        !uint64Decimal(block.nonce_decimal) ||
        !safe(block.tx_count) || (previousHash !== null && block.parent !== previousHash)) {
      throw Error("invalid_page");
    }
    hashes.add(block.hash);
    previousHash = block.hash;
    return block as CanonicalBlock;
  });
  const first = entries[0];
  if (first && expectedStart > 0 && (first.parent === expectedTip || hashes.has(first.parent))) {
    throw Error("invalid_page");
  }
  const next = expectedStart + entries.length;
  if (page.next_start !== (next <= height ? next : null)) {
    throw Error("invalid_page");
  }
  return { ...page, blocks: entries } as CanonicalPage;
}

export function parseCanonicalPageByHash(
  raw: unknown,
  expectedTip: string,
  blockHash: string,
  limit: number,
): { page: CanonicalPage; height: number } {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) throw Error("invalid_page");
  const start: unknown = Object.getOwnPropertyDescriptor(raw, "start")?.value;
  if (!safe(start)) throw Error("invalid_page");
  const page = parseCanonicalPage(raw, expectedTip, start, limit);
  const block = page.blocks.find(block => block.hash === blockHash);
  if (!block || start !== block.height - block.height % limit) throw Error("invalid_hash_page");
  return { page, height: block.height };
}

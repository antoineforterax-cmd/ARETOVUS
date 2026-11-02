import { describe, expect, it } from "vitest";
import { parseCanonicalPage } from "./explorer";

const ZERO = "0".repeat(64);
const ONE = "a".repeat(64);
const TWO = "b".repeat(64);
function sample() {
  return {
    schema_version: 1,
    network: "testnet",
    status: "verified_local",
    read_only: true,
    source: "aretovus-native-explorer",
    chain_tip: ONE,
    chain_height: 1,
    start: 0,
    next_start: null,
    blocks: [
      { height: 0, hash: ZERO, parent: ZERO, timestamp_decimal: "1", nonce_decimal: "0", tx_count: 0, merkle_root: ZERO },
      { height: 1, hash: ONE, parent: ZERO, timestamp_decimal: "2", nonce_decimal: "18446744073709551615", tx_count: 0, merkle_root: ZERO },
    ],
  };
}
describe("native explorer page trust boundary", () => {
  it("accepts validated canonical blocks and preserves unsigned u64 decimal fields", () => {
    const page = parseCanonicalPage(sample(), ONE, 0, 10);
    expect(page.blocks.map(b => b.height)).toEqual([0, 1]);
    expect(page.blocks[1].nonce_decimal).toBe("18446744073709551615");
  });
  it("rejects malformed source, unsafe integers, broken parent links and reorg/stale tip", () => {
    const p = sample();
    const variants = [
      { ...p, status: "unverified" },
      { ...p, network: "mainnet" },
      { ...p, source: "web-api" },
      { ...p, chain_tip: TWO },
      { ...p, chain_height: Number.MAX_SAFE_INTEGER + 1 },
      { ...p, start: 1 },
      { ...p, next_start: 1 },
      { ...p, read_only: false },
      { ...p, blocks: [p.blocks[0], { ...p.blocks[1], parent: TWO }] },
      { ...p, blocks: [p.blocks[0], { ...p.blocks[1], height: 4 }] },
      { ...p, blocks: [p.blocks[0], { ...p.blocks[1], hash: '<script>' }] },
      { ...p, blocks: [p.blocks[0], { ...p.blocks[1], nonce_decimal: "01" }] },
      { ...p, blocks: [p.blocks[0], { ...p.blocks[1], tx_count: -1 }] },
      { ...p, blocks: [p.blocks[0], { ...p.blocks[1], timestamp_decimal: 123 }] },
    ];
    for (const forged of variants) expect(() => parseCanonicalPage(forged, ONE, 0, 10)).toThrow();
    expect(() => parseCanonicalPage(p, TWO, 0, 10)).toThrow();
    expect(() => parseCanonicalPage(p, ONE, -1, 10)).toThrow();
    expect(() => parseCanonicalPage(p, ONE, 0, 21)).toThrow();
    expect(() => parseCanonicalPage(null, ONE, 0, 10)).toThrow();
    expect(() => parseCanonicalPage({ ...p, blocks: Array(21).fill(p.blocks[0]) }, ONE, 0, 20)).toThrow();
  });
  it("requires exact next-page cursor and a matching tip at the last height", () => {
    const page = sample();
    page.chain_height = 2;
    page.next_start = 2;
    page.chain_tip = TWO;
    expect(parseCanonicalPage(page, TWO, 0, 2).next_start).toBe(2);
    expect(() => parseCanonicalPage({ ...page, next_start: 3 }, TWO, 0, 2)).toThrow();
    const end = { ...page, start: 2, next_start: null, blocks: [
      { ...page.blocks[1], height: 2, hash: TWO },
    ] };
    expect(parseCanonicalPage(end, TWO, 2, 2).blocks[0].hash).toBe(TWO);
    expect(() => parseCanonicalPage({ ...end, blocks: [{ ...end.blocks[0], hash: ONE }] }, TWO, 2, 2)).toThrow();
  });
});

import { describe, expect, it } from "vitest";
import { parseNodeStatus } from "./status";
const valid = () => ({
  schema_version: 1, network: "testnet", status: "verified", height: 1,
  known_blocks: 2, tip: "ab".repeat(32),
  chain_work_display: "ChainWork([256, 0, 0, 0, 0])",
  source: "aretovus-native-stdio", read_only: true,
});
describe("desktop status trust boundary", () => {
  it("accepts a constrained native reply", () => expect(parseNodeStatus(valid()).height).toBe(1));
  it("rejects forged fields, unknown versions and unsafe numeric states", () => {
    for (const [key, value] of Object.entries({
      status: "pending", network: "regtest", source: "internet", read_only: false,
      tip: "<script>", height: 1.5, known_blocks: -1, schema_version: 2,
      chain_work_display: "not-work",
    })) expect(() => parseNodeStatus({ ...valid(), [key]: value })).toThrow();
    expect(() => parseNodeStatus(null)).toThrow();
    expect(() => parseNodeStatus([valid()])).toThrow();
    expect(() => parseNodeStatus({ ...valid(), height: Number.MAX_SAFE_INTEGER, known_blocks: Number.MAX_SAFE_INTEGER })).toThrow();
  });
});

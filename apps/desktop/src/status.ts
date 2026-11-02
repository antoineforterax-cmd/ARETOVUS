export type VerifiedNodeStatus = {
  schema_version: 1;
  network: "testnet";
  status: "verified";
  height: number;
  known_blocks: number;
  tip: string;
  chain_work_display: string;
  source: "aretovus-native-stdio";
  read_only: true;
};
export function parseNodeStatus(raw: unknown): VerifiedNodeStatus {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) throw Error("invalid_response");
  const s = raw as Record<string, unknown>;
  if (s.schema_version !== 1 || s.network !== "testnet" || s.status !== "verified" ||
      s.source !== "aretovus-native-stdio" || s.read_only !== true ||
      typeof s.height !== "number" || !Number.isSafeInteger(s.height) || s.height < 0 ||
      typeof s.known_blocks !== "number" || !Number.isSafeInteger(s.known_blocks) ||
      s.known_blocks < s.height + 1 || typeof s.tip !== "string" ||
      !/^[0-9a-f]{64}$/.test(s.tip) || typeof s.chain_work_display !== "string" ||
      !/^ChainWork\(\[[0-9, ]{1,240}\]\)$/.test(s.chain_work_display)) {
    throw Error("invalid_response");
  }
  return s as VerifiedNodeStatus;
}

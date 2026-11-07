import { expect, it } from "vitest";
import { nativeErrorMessage } from "./nativeErrors";

it("gives an action for busy, unavailable, timeout and invalid directory errors", () => {
  for (const code of ["bridge_busy", "bridge_unavailable", "explorer_unavailable",
    "bridge_timeout", "explorer_timeout", "invalid_data_dir", "verification_failed", "explorer_failed"]) {
    expect(nativeErrorMessage(code, "fallback")).not.toBe("fallback");
  }
});

it("never displays unknown native text, paths, objects or inherited property names", () => {
  for (const error of ["/private/alice/wallet.key", "bridge_timeout\nsecret", "toString",
    "__proto__", new Error("private path"), { code: "bridge_busy" }, null, 42]) {
    expect(nativeErrorMessage(error, "Lecture indisponible.")).toBe("Lecture indisponible.");
  }
});

it("maps a v1 native error to the same fixed message as the legacy code", () => {
  expect(nativeErrorMessage({ schema_version: 1, code: "invalid_data_dir" }, "fallback"))
    .toBe(nativeErrorMessage("invalid_data_dir", "fallback"));
});

it("falls back for unknown versions, malformed codes and extra diagnostics", () => {
  const getter = {
    schema_version: 1,
    get code(): string { throw new Error("The parser must not evaluate this getter"); },
  };
  for (const error of [
    { schema_version: 2, code: "bridge_busy" },
    { schema_version: "1", code: "bridge_busy" },
    { schema_version: 1, code: "toString" },
    { schema_version: 1, code: "bridge_busy", message: "/private/path" },
    { schema_version: 1, code: { path: "/private/path" } },
    Object.assign(Object.create({ code: "bridge_busy" }), { schema_version: 1, other: true }),
    getter,
  ]) {
    expect(nativeErrorMessage(error, "Lecture indisponible.")).toBe("Lecture indisponible.");
  }
});

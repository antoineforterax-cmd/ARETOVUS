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

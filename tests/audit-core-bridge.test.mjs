import { describe, expect, it } from "vitest";

import { runAuditCoreJson } from "../_lib/audit-core.mjs";

describe("audit-core JS bridge output policy", () => {
  it("ACB1. rejects repository-sized manifest commands on the stdout bridge", () => {
    for (const subcommand of [
      "manifest-root-with-evidence",
      "manifest-lifecycle-evidence-refresh",
      "manifest-evidence-summary-with-reads",
      "manifest-evidence-refresh-with-reads",
    ]) {
      expect(() =>
        runAuditCoreJson([subcommand], "stdoutBridgePolicy"),
      ).toThrow(
        `${subcommand} can emit repository-sized JSON and must use runAuditCoreJsonResultFile`,
      );
    }
  });
});

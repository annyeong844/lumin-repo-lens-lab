import { describe, expect, it } from "vitest";
import { chmodSync, writeFileSync } from "node:fs";
import path from "node:path";

import { runAuditCoreJson } from "../_lib/audit-core.mjs";
import * as auditManifest from "../_lib/audit-manifest.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

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

  it("ACB2. rejects stale helpers that lack result-output support before result-file calls", () => {
    if (process.platform === "win32") {
      expect(process.platform).toBe("win32");
      return;
    }

    const fixture = createTempRepoFixture({
      prefix: "audit-core-stale-result-output-",
    });
    const previous = process.env.LUMIN_AUDIT_CORE_BIN;
    try {
      const fakeBinary = path.join(fixture.root, "stale-audit-core");
      writeFileSync(
        fakeBinary,
        `#!/usr/bin/env node
const command = process.argv[2];
if (process.argv.includes("--result-output")) {
  console.error(\`\${command}: unknown argument '--result-output'\`);
  process.exit(1);
}
const messages = {
  "producer-performance-runtime-artifact": "producer-performance-runtime-artifact: missing --input",
  "producer-performance-audit-run-artifact": "producer-performance-audit-run-artifact: missing --input",
  "manifest-companion-update": "manifest-companion-update: missing --input",
  "manifest-root-with-evidence": "manifest-root-with-evidence: missing --input <path|->",
  "manifest-evidence-refresh-with-reads": "manifest-evidence-refresh-with-reads: missing --root <repo>",
  "manifest-lifecycle-evidence-refresh": "manifest-lifecycle-evidence-refresh: missing --input <path|->",
  "manifest-evidence-summary-with-reads": "manifest-evidence-summary-with-reads: missing --root <repo>",
  "manifest-closeout-update": "manifest-closeout-update: missing --input",
  "manifest-artifacts-produced-update": "manifest-artifacts-produced-update: missing --output <dir>",
  "manifest-write": "manifest-write: missing --output <dir>",
  "manifest-closeout-write": "manifest-closeout-write: missing --input <path|->",
  "finalize-audit-run": "finalize-audit-run: missing --input <path|->",
};
console.error(messages[command] ?? \`\${command}: unknown command\`);
process.exit(1);
`,
      );
      chmodSync(fakeBinary, 0o755);
      process.env.LUMIN_AUDIT_CORE_BIN = fakeBinary;

      const evidence = auditManifest.buildManifestEvidence({
        root: fixture.root,
        outDir: fixture.output,
        includeTests: true,
        production: false,
      });

      expect(evidence.scanRange).toMatchObject({
        includeTests: true,
        production: false,
      });
    } finally {
      fixture.cleanup();
      if (previous === undefined) delete process.env.LUMIN_AUDIT_CORE_BIN;
      else process.env.LUMIN_AUDIT_CORE_BIN = previous;
    }
  }, 30000);
});

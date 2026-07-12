import { describe, expect, it } from "vitest";
import { chmodSync, utimesSync, writeFileSync } from "node:fs";
import path from "node:path";

import { runAuditCoreJson } from "../_lib/audit-core.mjs";
import * as auditManifest from "../_lib/audit-manifest.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function writeFakeAuditCore(binaryPath, { resultMode }) {
  const validResult = `
function validResult(command) {
  if (command === "manifest-root-with-evidence" || command === "manifest-lifecycle-evidence-refresh") {
    return {
      manifest: {
        scanRange: { files: 999, includeTests: true, production: false },
        lifecycle: { ranCount: 0 }
      },
      artifactReads: { reads: [] }
    };
  }
  return {
    evidence: {
      scanRange: { files: 999, includeTests: true, production: false }
    },
    artifactReads: { reads: [] }
  };
}

function emptyBodyResult(command) {
  if (command === "manifest-root-with-evidence" || command === "manifest-lifecycle-evidence-refresh") {
    return { manifest: {}, artifactReads: { reads: [] } };
  }
  return { evidence: {}, artifactReads: { reads: [] } };
}
`;
  const resultExpression = {
    valid: "validResult(command)",
    emptyBody: "emptyBodyResult(command)",
  }[resultMode] ?? "{}";
  writeFileSync(
    binaryPath,
    `#!/usr/bin/env node
const { writeFileSync } = require("node:fs");
const command = process.argv[2];
const resultIndex = process.argv.indexOf("--result-output");
if (resultIndex !== -1) {
  const resultPath = process.argv[resultIndex + 1];
  const result = ${resultExpression};
  writeFileSync(resultPath, JSON.stringify(result));
  process.exit(0);
}
const messages = {
  "producer-performance-runtime-artifact": "producer-performance-runtime-artifact: missing --input",
  "producer-performance-audit-run-artifact": "producer-performance-audit-run-artifact: missing --input",
  "manifest-companion-update": "manifest-companion-update: missing --input",
  "manifest-root-with-evidence": "manifest-root-with-evidence: missing --input <path|->",
  "manifest-evidence-refresh": "manifest-evidence-refresh: missing --root <repo>",
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
${validResult}
`,
  );
  chmodSync(binaryPath, 0o755);
  const touchTime = new Date(Date.now() + 2000);
  utimesSync(binaryPath, touchTime, touchTime);
}

function writeMinimalManifestArtifacts(fixture) {
  fixture.writeJson(
    "triage.json",
    {
      shape: { totalFiles: 2, tsFiles: 1, rsFiles: 1 },
      byLanguage: { rs: 1 },
    },
    { to: "output" },
  );
  fixture.writeJson(
    "symbols.json",
    {
      uses: {
        external: 0,
        resolvedInternal: 0,
        unresolvedInternal: 0,
        unresolvedInternalRatio: 0,
      },
    },
    { to: "output" },
  );
}

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

  it("ACB2. rejects stale helpers that write empty result-output bodies", () => {
    if (process.platform === "win32") {
      expect(process.platform).toBe("win32");
      return;
    }

    const fixture = createTempRepoFixture({
      prefix: "audit-core-stale-result-output-",
    });
    writeMinimalManifestArtifacts(fixture);
    const previous = process.env.LUMIN_AUDIT_CORE_BIN;
    try {
      const fakeBinary = path.join(fixture.root, "stale-audit-core");
      writeFakeAuditCore(fakeBinary, { resultMode: "emptyBody" });
      process.env.LUMIN_AUDIT_CORE_BIN = fakeBinary;

      const evidence = auditManifest.buildManifestEvidence({
        root: fixture.root,
        outDir: fixture.output,
        includeTests: true,
        production: false,
      });

      expect(evidence.scanRange).toMatchObject({
        files: 2,
        includeTests: true,
        production: false,
      });
    } finally {
      fixture.cleanup();
      if (previous === undefined) delete process.env.LUMIN_AUDIT_CORE_BIN;
      else process.env.LUMIN_AUDIT_CORE_BIN = previous;
    }
  }, 30000);

  it("ACB3. rechecks a repaired override before returning a cached fallback", () => {
    if (process.platform === "win32") {
      expect(process.platform).toBe("win32");
      return;
    }

    const fixture = createTempRepoFixture({
      prefix: "audit-core-repaired-override-",
    });
    writeMinimalManifestArtifacts(fixture);
    const previous = process.env.LUMIN_AUDIT_CORE_BIN;
    try {
      const fakeBinary = path.join(fixture.root, "repairable-audit-core");
      writeFakeAuditCore(fakeBinary, { resultMode: "emptyBody" });
      process.env.LUMIN_AUDIT_CORE_BIN = fakeBinary;

      const fallbackEvidence = auditManifest.buildManifestEvidence({
        root: fixture.root,
        outDir: fixture.output,
        includeTests: true,
        production: false,
      });
      expect(fallbackEvidence.scanRange.files).toBe(2);

      writeFakeAuditCore(fakeBinary, { resultMode: "valid" });
      const overrideEvidence = auditManifest.buildManifestEvidence({
        root: fixture.root,
        outDir: fixture.output,
        includeTests: true,
        production: false,
      });
      expect(overrideEvidence.scanRange.files).toBe(999);
    } finally {
      fixture.cleanup();
      if (previous === undefined) delete process.env.LUMIN_AUDIT_CORE_BIN;
      else process.env.LUMIN_AUDIT_CORE_BIN = previous;
    }
  }, 30000);

  it("ACB4. revalidates a cached helper path after the binary is replaced", () => {
    if (process.platform === "win32") {
      expect(process.platform).toBe("win32");
      return;
    }

    const fixture = createTempRepoFixture({
      prefix: "audit-core-replaced-result-output-",
    });
    writeMinimalManifestArtifacts(fixture);
    const previous = process.env.LUMIN_AUDIT_CORE_BIN;
    try {
      const fakeBinary = path.join(fixture.root, "replaceable-audit-core");
      writeFakeAuditCore(fakeBinary, { resultMode: "valid" });
      process.env.LUMIN_AUDIT_CORE_BIN = fakeBinary;

      const fakeEvidence = auditManifest.buildManifestEvidence({
        root: fixture.root,
        outDir: fixture.output,
        includeTests: true,
        production: false,
      });
      expect(fakeEvidence.scanRange.files).toBe(999);

      writeFakeAuditCore(fakeBinary, { resultMode: "placeholder" });
      const realEvidence = auditManifest.buildManifestEvidence({
        root: fixture.root,
        outDir: fixture.output,
        includeTests: true,
        production: false,
      });
      expect(realEvidence.scanRange.files).toBe(2);
    } finally {
      fixture.cleanup();
      if (previous === undefined) delete process.env.LUMIN_AUDIT_CORE_BIN;
      else process.env.LUMIN_AUDIT_CORE_BIN = previous;
    }
  }, 30000);
});

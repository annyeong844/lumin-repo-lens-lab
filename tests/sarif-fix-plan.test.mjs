import { execFileSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");

function createFixPlan(root) {
  return {
    meta: {
      generated: new Date().toISOString(),
      root,
      tool: "rank-fixes.mjs",
      inputs: {
        "dead-classify.json": true,
        "runtime-evidence.json": false,
        "staleness.json": false,
        "symbols.json": false,
      },
      resolverBlindness: null,
    },
    summary: { SAFE_FIX: 1, REVIEW_FIX: 1, DEGRADED: 1, MUTED: 1, total: 4 },
    safeFixes: [
      {
        finding: {
          id: "x",
          file: "src/safe.ts",
          line: 10,
          symbol: "SafeSym",
          kind: "FunctionDeclaration",
          bucket: "C",
          action: "remove",
        },
        evidence: {
          runtime: {
            status: "dead-confirmed",
            grounding: "grounded",
            confidence: "high",
            hitsInSymbol: 0,
          },
          staleness: {
            tier: "fossil",
            grounding: "grounded",
            lineLastTouchedDaysAgo: 900,
          },
          policy: { excluded: false },
        },
        tier: "SAFE_FIX",
        reason:
          "AST-dead + runtime-dead-confirmed + staleness-fossil + bucket-C",
      },
    ],
    reviewFixes: [
      {
        finding: {
          id: "x",
          file: "src/review.ts",
          line: 20,
          symbol: "ReviewSym",
          kind: "FunctionDeclaration",
          bucket: "A",
          action: "demote",
          fileInternalUses: 2,
        },
        evidence: { policy: { excluded: false } },
        tier: "REVIEW_FIX",
        reason: "bucket-A; missing: no-runtime, no-staleness",
      },
    ],
    degraded: [
      {
        finding: {
          id: "x",
          file: "src/deg.ts",
          line: 30,
          symbol: "DegSym",
          kind: "FunctionDeclaration",
          bucket: "C",
          action: "remove",
        },
        evidence: {
          runtime: {
            status: "executed",
            grounding: "grounded",
            confidence: "high",
            hitsInSymbol: 7,
          },
          policy: { excluded: false },
        },
        tier: "DEGRADED",
        reason: "runtime-executed (7 hits)",
      },
    ],
    muted: [
      {
        finding: {
          id: "x",
          file: "eslint.config.mjs",
          line: 1,
          symbol: "default",
          kind: "default",
          bucket: "excluded",
          action: "Policy-excluded: config_FP22",
        },
        evidence: { policy: { excluded: true, reason: "config_FP22" } },
        tier: "MUTED",
        reason: "policy-excluded: config_FP22",
      },
    ],
  };
}

function withSarifFromFixPlan(fn) {
  const fixture = mkdtempSync(path.join(tmpdir(), "vitest-sarif-fixplan-"));
  const output = path.join(fixture, "artifacts");
  try {
    mkdirSync(output, { recursive: true });
    writeFileSync(
      path.join(fixture, "package.json"),
      JSON.stringify({ name: "fixture", type: "module" }),
    );
    writeFileSync(
      path.join(output, "fix-plan.json"),
      JSON.stringify(createFixPlan(fixture), null, 2),
    );

    execFileSync(
      process.execPath,
      ["emit-sarif.mjs", "--root", fixture, "--output", output],
      {
        cwd: REPO_ROOT,
        stdio: ["ignore", "pipe", "pipe"],
      },
    );

    const sarif = JSON.parse(
      readFileSync(path.join(output, "lumin-repo-lens-lab.sarif"), "utf8"),
    );
    const results = sarif.runs[0].results.filter(
      (result) => result.ruleId === "GA001",
    );
    return fn({ sarif, results });
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
}

describe("SARIF fix-plan output", () => {
  it("S1-S3. uses fix-plan tier properties and excludes MUTED entries", () => {
    withSarifFromFixPlan(({ results }) => {
      expect(
        results.every((result) => result.properties?.tier !== undefined),
      ).toBe(true);
      expect(results).toHaveLength(3);
      expect(
        results.find((result) => result.properties?.tier === "MUTED"),
      ).toBeUndefined();
    });
  });

  it("S4-S6 and S9. maps SAFE_FIX to warning and REVIEW_FIX/DEGRADED to notes", () => {
    withSarifFromFixPlan(({ results }) => {
      const safe = results.find(
        (result) => result.properties?.tier === "SAFE_FIX",
      );
      const review = results.find(
        (result) => result.properties?.tier === "REVIEW_FIX",
      );
      const degraded = results.find(
        (result) => result.properties?.tier === "DEGRADED",
      );
      const byLevel = { warning: 0, note: 0, error: 0 };
      for (const result of results) {
        byLevel[result.level] = (byLevel[result.level] ?? 0) + 1;
      }

      expect(safe?.level).toBe("warning");
      expect(review?.level).toBe("note");
      expect(degraded?.level).toBe("note");
      expect(byLevel).toMatchObject({ warning: 1, note: 2, error: 0 });
    });
  });

  it("S7-S8 and S10. carries proposal bucket, ranking reason, and runtime hits", () => {
    withSarifFromFixPlan(({ results }) => {
      const safe = results.find(
        (result) => result.properties?.tier === "SAFE_FIX",
      );
      const review = results.find(
        (result) => result.properties?.tier === "REVIEW_FIX",
      );
      const degraded = results.find(
        (result) => result.properties?.tier === "DEGRADED",
      );

      expect(safe?.properties?.proposalBucket).toBe("C");
      expect(review?.properties?.proposalBucket).toBe("A");
      expect(degraded?.properties?.proposalBucket).toBe("C");
      expect(safe?.properties?.reason).toContain("runtime-dead-confirmed");
      expect(degraded?.properties?.hitsInSymbol).toBe(7);
    });
  });
});

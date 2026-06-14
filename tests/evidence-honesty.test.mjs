import { execFileSync } from "node:child_process";
import { copyFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

const fixtures = [];
let compare;
let asymmetricCompare;
let docRefsValidRun;
let docRefsMissingRun;
let docRefsLibRun;

function createFixture(prefix) {
  const fixture = createTempRepoFixture({ prefix });
  fixtures.push(fixture);
  return fixture;
}

function runNode(args, options = {}) {
  try {
    const out = execFileSync(process.execPath, args, {
      cwd: options.cwd ?? ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    return { ok: true, out };
  } catch (error) {
    return {
      ok: false,
      out: `${error.stdout ?? ""}${error.stderr ?? error.message}`,
    };
  }
}

function writeCompareArtifacts(
  fixture,
  {
    files,
    loc,
    safeFixes,
    reviewFixes,
    degraded,
    muted,
    totalDefs,
    deadInProd,
  },
) {
  fixture.writeJson("triage.json", {
    summary: { files, loc, buildSystem: "vite" },
  });
  fixture.writeJson("fix-plan.json", {
    meta: { resolverBlindness: { gate: "ok" } },
    summary: {
      SAFE_FIX: safeFixes,
      REVIEW_FIX: reviewFixes,
      DEGRADED: degraded,
      MUTED: muted,
      total: safeFixes + reviewFixes + degraded + muted,
    },
    safeFixes: [],
    reviewFixes: [],
    degraded: [],
    muted: [],
  });
  fixture.writeJson("symbols.json", {
    files,
    totalDefs,
    deadInProd,
    uses: {
      resolvedInternal: files * 20,
      external: files * 5,
      unresolvedInternal: 0,
      unresolvedInternalRatio: 0,
    },
  });
}

function runCompare(left, right, out, extraArgs = []) {
  const run = runNode([
    "compare-repos.mjs",
    "--left",
    left.root,
    "--right",
    right.root,
    "--output",
    out.root,
    ...extraArgs,
  ]);
  expect(run.ok).toBe(true);
  return out.readJson("compare.json");
}

function prepareDocRefFixture(fixture) {
  fixture.mkdir("scripts");
  fixture.mkdir("_lib");
  fixture.mkdir("tests");
  fixture.mkdir("templates");
  copyFileSync(
    path.join(ROOT, "scripts/check-doc-script-refs.mjs"),
    fixture.path("scripts/check-doc-script-refs.mjs"),
  );
  fixture.write("templates/report-template.md", "# Report\n");
  fixture.write("tests/README.md", "# Tests\n");
}

function runDocRefGuard(fixture) {
  return runNode(["scripts/check-doc-script-refs.mjs"], { cwd: fixture.root });
}

beforeAll(() => {
  const left = createFixture("fx-vitest-evidence-left-");
  const right = createFixture("fx-vitest-evidence-right-");
  const out = createFixture("fx-vitest-evidence-out-");
  writeCompareArtifacts(left, {
    files: 10,
    loc: 1000,
    safeFixes: 3,
    reviewFixes: 5,
    degraded: 1,
    muted: 2,
    totalDefs: 80,
    deadInProd: 4,
  });
  writeCompareArtifacts(right, {
    files: 15,
    loc: 1500,
    safeFixes: 7,
    reviewFixes: 3,
    degraded: 2,
    muted: 1,
    totalDefs: 120,
    deadInProd: 6,
  });
  compare = runCompare(left, right, out, [
    "--left-label",
    "L",
    "--right-label",
    "R",
  ]);

  const rightMissingFixPlan = createFixture("fx-vitest-evidence-right2-");
  const outMissingFixPlan = createFixture("fx-vitest-evidence-out2-");
  rightMissingFixPlan.writeJson("triage.json", {
    summary: { files: 15, loc: 1500 },
  });
  asymmetricCompare = runCompare(left, rightMissingFixPlan, outMissingFixPlan);

  const docRefs = createFixture("fx-vitest-docrefs-");
  prepareDocRefFixture(docRefs);
  docRefs.write("real-tool.mjs", "// stub\n");
  docRefs.write("SKILL.md", "# Skill\n\nRun `real-tool.mjs` for this.\n");
  docRefsValidRun = runDocRefGuard(docRefs);

  docRefs.write("SKILL.md", "# Skill\n\nRun `ghost-tool.mjs` for this.\n");
  docRefsMissingRun = runDocRefGuard(docRefs);

  docRefs.write("_lib/helper.mjs", "// stub\n");
  docRefs.write("SKILL.md", "# Skill\n\nInternal: `helper.mjs`.\n");
  docRefsLibRun = runDocRefGuard(docRefs);
});

afterAll(() => {
  for (const fixture of fixtures.reverse()) {
    fixture.cleanup();
  }
});

describe("compare-repos artifact deltas", () => {
  it("C1. compare-repos exits 0 on valid inputs", () => {
    expect(compare.meta?.tool).toBe("compare-repos.mjs");
  });

  it("C2. deltas.files = +5 (15 - 10)", () => {
    expect(compare.deltas?.files).toBe(5);
  });

  it("C3. deltas.safeFixes = +4 (7 - 3)", () => {
    expect(compare.deltas?.safeFixes).toBe(4);
  });

  it("C4. deltas.degraded = +1 (2 - 1)", () => {
    expect(compare.deltas?.degraded).toBe(1);
  });

  it("C5. both sides list fix-plan.json, symbols.json, and triage.json", () => {
    expect(compare.left?.artifactsFound?.sort()).toEqual([
      "fix-plan.json",
      "symbols.json",
      "triage.json",
    ]);
    expect(compare.right?.artifactsFound?.sort()).toEqual([
      "fix-plan.json",
      "symbols.json",
      "triage.json",
    ]);
  });

  it("C6. missingArtifacts flags artifacts that were not present", () => {
    expect(compare.missingArtifacts?.left).toEqual(
      expect.arrayContaining(["runtime-evidence.json", "staleness.json"]),
    );
  });

  it("C7. asymmetric missing artifact makes the affected delta null", () => {
    expect(asymmetricCompare.deltas?.safeFixes).toBeNull();
  });
});

describe("doc script reference guard", () => {
  it("D1. guard exits 0 when every referenced .mjs exists", () => {
    expect(docRefsValidRun.ok).toBe(true);
    expect(docRefsValidRun.out).toContain("resolve on disk");
  });

  it("D2. guard exits non-zero when a referenced .mjs is missing", () => {
    expect(docRefsMissingRun.ok).toBe(false);
    expect(docRefsMissingRun.out).toContain("ghost-tool.mjs");
  });

  it("D3. guard error message suggests remediation", () => {
    expect(docRefsMissingRun.out).toContain("create");
    expect(docRefsMissingRun.out).toContain("remove");
  });

  it("D4. files under _lib/ count as present", () => {
    expect(docRefsLibRun.ok).toBe(true);
  });
});

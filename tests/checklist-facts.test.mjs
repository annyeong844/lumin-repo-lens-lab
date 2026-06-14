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

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function createFixture(prefix = "vitest-checklist-facts-") {
  const root = mkdtempSync(path.join(tmpdir(), prefix));
  const output = mkdtempSync(path.join(tmpdir(), `${prefix}out-`));
  return {
    root,
    output,
    cleanup() {
      rmSync(root, { recursive: true, force: true });
      rmSync(output, { recursive: true, force: true });
    },
  };
}

function runProducer(scriptName, root, output, extraArgs = []) {
  execFileSync(
    process.execPath,
    [
      path.join(REPO_ROOT, scriptName),
      "--root",
      root,
      "--output",
      output,
      ...extraArgs,
    ],
    {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function readChecklist(output) {
  return JSON.parse(
    readFileSync(path.join(output, "checklist-facts.json"), "utf8"),
  );
}

function longFunctionBody(lines = 160) {
  return Array.from({ length: lines }, (_, i) => `  const x${i} = ${i};`).join(
    "\n",
  );
}

describe("checklist-facts producer artifact", () => {
  it("degrades cleanly when upstream artifacts are missing while keeping AST-backed facts", () => {
    const fixture = createFixture("vitest-checklist-bare-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-bare", type: "module" }),
      );
      write(fixture.root, "src/ok.ts", "export const trivial = 1;\n");

      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const cf = readChecklist(fixture.output);

      expect(typeof cf.meta?.schemaVersion).toBe("number");
      expect(cf.meta.schemaVersion).toBeGreaterThanOrEqual(2);
      expect(cf.A2_function_size).toMatchObject({
        gate: "ok",
        buckets: expect.objectContaining({ big: 0 }),
      });
      expect(cf.A5_decoupling_ratio).toMatchObject({
        available: false,
        gate: "unknown",
      });
      expect(cf.A6_circular_deps.available).toBe(false);
      expect(cf.B3_dead_code.available).toBe(false);
      expect(cf.C5_lint_enforcement.available).toBe(false);
      expect(cf.C7_barrel_amplification.available).toBe(false);
      expect(cf.B1B2_shape_drift).toMatchObject({
        available: false,
        gate: "unknown",
      });
      expect(cf.E2_silent_catch).toMatchObject({
        count: 0,
        gate: "ok",
        analysis: "oxc-ast-catch-clause",
      });
      expect(cf._not_computed.length).toBeGreaterThanOrEqual(20);
      expect(cf.A2_function_size._citation_hint).toMatch(/^\[grounded,/);
      expect(cf.A6_circular_deps._context_check_required).toBe(false);
      expect(cf.A2_function_size._context_check_required).toBe(true);
      expect(cf.A5_decoupling_ratio._citation_hint).toContain("확인 불가");
      expect(cf.A5_decoupling_ratio._citation_hint).toContain("scan range");
    } finally {
      fixture.cleanup();
    }
  });

  it("records oversized function evidence by production, test, and script role", () => {
    const fixture = createFixture("vitest-checklist-a2-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-a2", type: "module" }),
      );
      const body = longFunctionBody();
      write(
        fixture.root,
        "src/huge.ts",
        `export function huge() {\n${body}\n  return 0;\n}\n`,
      );
      write(
        fixture.root,
        "tests/huge.test.ts",
        `export function testHuge() {\n${body}\n  return 0;\n}\n`,
      );
      write(
        fixture.root,
        "scripts/huge-smoke.mjs",
        `export function scriptHuge() {\n${body}\n  return 0;\n}\n`,
      );

      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const cf = readChecklist(fixture.output);
      const huge = cf.A2_function_size.oversized.find(
        (entry) => entry.name === "huge",
      );

      expect(cf.A2_function_size.oversized.length).toBeGreaterThanOrEqual(1);
      expect(huge?.loc).toBeGreaterThan(150);
      expect(["watch", "fix"]).toContain(cf.A2_function_size.gate);
      expect(
        cf.A2_function_size.oversizedByRole.production.some(
          (entry) => entry.name === "huge",
        ),
      ).toBe(true);
      expect(
        cf.A2_function_size.oversizedByRole.test.some(
          (entry) => entry.name === "testHuge",
        ),
      ).toBe(true);
      expect(
        cf.A2_function_size.oversizedByRole.script.some(
          (entry) => entry.name === "scriptHuge",
        ),
      ).toBe(true);
      expect(cf.A2_function_size.roleBuckets.production.big).toBe(1);
      expect(cf.A2_function_size.roleBuckets.test.big).toBe(1);
      expect(cf.A2_function_size.roleBuckets.script.big).toBe(1);
    } finally {
      fixture.cleanup();
    }
  });

  it("uses full cross-submodule edges and downgrades healthy layered flow only", () => {
    const fixture = createFixture("vitest-checklist-a5-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-a5", type: "module" }),
      );
      write(fixture.root, "src/ok.ts", "export const ok = 1;\n");
      write(
        fixture.output,
        "topology.json",
        JSON.stringify({
          summary: { internalEdges: 100 },
          crossSubmoduleEdges: [
            { from: "root", to: "_lib", count: 60 },
            { from: "tests", to: "_lib", count: 20 },
          ],
          crossSubmoduleTop: [{ edge: "root -> _lib", count: 60 }],
          sccs: [],
        }),
      );

      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const cf = readChecklist(fixture.output);

      expect(cf.A5_decoupling_ratio).toMatchObject({
        crossSubmoduleEdgeSource: "full-list",
        crossSubmoduleEdgesSum: 80,
        rawGate: "fix",
        gate: "ok",
        reviewedEdgesSum: 0,
      });
    } finally {
      fixture.cleanup();
    }
  });

  it("keeps inverted engine-to-root cross-submodule flow as a fix gate", () => {
    const fixture = createFixture("vitest-checklist-a5-inv-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-a5-inv", type: "module" }),
      );
      write(fixture.root, "src/ok.ts", "export const ok = 1;\n");
      write(
        fixture.output,
        "topology.json",
        JSON.stringify({
          summary: { internalEdges: 100 },
          crossSubmoduleEdges: [{ from: "_lib", to: "root", count: 60 }],
          sccs: [],
        }),
      );

      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const cf = readChecklist(fixture.output);

      expect(cf.A5_decoupling_ratio).toMatchObject({
        rawGate: "fix",
        gate: "fix",
        reviewedEdgesSum: 60,
      });
    } finally {
      fixture.cleanup();
    }
  });

  it("surfaces exact shape drift as watch evidence while leaving broader B1/B2 as judgment", () => {
    const fixture = createFixture("vitest-checklist-shape-exact-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-b1b2", type: "module" }),
      );
      write(
        fixture.root,
        "src/web.ts",
        "export interface SubagentActivityState { id: string; status: 'idle' | 'running' }\n",
      );
      write(
        fixture.root,
        "src/daemon.ts",
        "export type DaemonActivityView = { status: 'idle' | 'running'; id: string };\n",
      );
      write(
        fixture.root,
        "src/other.ts",
        "export interface DifferentShape { id: number; status: string }\n",
      );

      runProducer("build-shape-index.mjs", fixture.root, fixture.output);
      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const cf = readChecklist(fixture.output);
      const shape = cf.B1B2_shape_drift;

      expect(cf.meta.inputsPresent["shape-index.json"]).toBe(true);
      expect(shape).toMatchObject({
        gate: "watch",
        exactDuplicateGroups: 1,
        duplicateIdentityCount: 2,
      });
      expect(shape.topGroups[0].identities).toEqual(
        expect.arrayContaining([
          "src/web.ts::SubagentActivityState",
          "src/daemon.ts::DaemonActivityView",
        ]),
      );
      expect(shape.topGroups[0].fieldNames).toEqual(
        expect.arrayContaining(["id", "status"]),
      );
      expect(shape._citation_hint).toContain(
        "B1B2_shape_drift.exactDuplicateGroups = 1",
      );
      expect(shape._citation_hint).toContain("nearShapeCandidateCount");
      expect(shape._context_check_required).toBe(true);
      expect(
        cf._not_computed.some(
          (item) =>
            item.item === "B1" && item.reason.includes("function clone cues"),
        ),
      ).toBe(true);
      expect(
        cf._not_computed.some(
          (item) =>
            item.item === "B2" && item.reason.includes("domain/vocab judgment"),
        ),
      ).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("surfaces near shape candidates as review cues, not proof", () => {
    const fixture = createFixture("vitest-checklist-shape-near-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-b1b2-near", type: "module" }),
      );
      write(
        fixture.root,
        "src/activity-state.ts",
        [
          "export interface SubagentActivityState {",
          "  id: string;",
          "  status: 'idle' | 'running';",
          "  updatedAt: string;",
          "}",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/activity-view.ts",
        [
          "export interface SubagentActivityView {",
          "  id: string;",
          "  status: 'idle' | 'running';",
          "  label: string;",
          "}",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/unrelated.ts",
        [
          "export interface BuildResult {",
          "  ok: boolean;",
          "  durationMs: number;",
          "}",
          "",
        ].join("\n"),
      );

      runProducer("build-shape-index.mjs", fixture.root, fixture.output);
      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const shape = readChecklist(fixture.output).B1B2_shape_drift;
      const near = shape.nearShapeCandidates[0];

      expect(shape).toMatchObject({
        gate: "watch",
        exactDuplicateGroups: 0,
        nearShapeCandidateCount: 1,
      });
      expect(near.identities).toEqual(
        expect.arrayContaining([
          "src/activity-state.ts::SubagentActivityState",
          "src/activity-view.ts::SubagentActivityView",
        ]),
      );
      expect(near.sharedFieldNames).toEqual(
        expect.arrayContaining(["id", "status"]),
      );
      expect(near.fieldJaccard).toBeGreaterThanOrEqual(0.5);
      expect(near.reason).toMatch(/review cue/);
      expect(near.reason).toMatch(/not proof/);
    } finally {
      fixture.cleanup();
    }
  });

  it("surfaces function clone structure groups as grounded review-only observations", () => {
    const fixture = createFixture("vitest-checklist-fn-structure-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-b1-fn-clones", type: "module" }),
      );
      write(
        fixture.root,
        "src/a.ts",
        [
          "export function formatCurrencyCents(cents: number, currency = 'USD') {",
          "  const dollars = cents / 100;",
          "  return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(dollars);",
          "}",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/b.ts",
        [
          "export function renderPaymentTotal(value: number, unit = 'USD') {",
          "  const amount = value / 100;",
          "  return new Intl.NumberFormat('en-US', { style: 'currency', currency: unit }).format(amount);",
          "}",
          "",
        ].join("\n"),
      );

      runProducer(
        "build-function-clone-index.mjs",
        fixture.root,
        fixture.output,
      );
      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const cf = readChecklist(fixture.output);
      const clones = cf.B1_duplicate_implementation;

      expect(cf.meta.inputsPresent["function-clones.json"]).toBe(true);
      expect(clones).toMatchObject({
        gate: "watch",
        structureGroupCandidates: 1,
        candidateIdentityCount: 2,
      });
      expect(clones.topStructureGroups[0].identities).toEqual(
        expect.arrayContaining([
          "src/a.ts::formatCurrencyCents",
          "src/b.ts::renderPaymentTotal",
        ]),
      );
      expect(clones.topStructureGroups[0].reason).toMatch(
        /not proof of semantic equivalence/,
      );
      expect(clones._citation_hint).toContain(
        "B1_duplicate_implementation.exactBodyGroups",
      );
      expect(clones._citation_hint).toContain("structureGroupCandidates");
      expect(clones._context_check_required).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("keeps near function candidates distinct from exact and structure clone groups", () => {
    const fixture = createFixture("vitest-checklist-fn-near-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-b1-fn-near", type: "module" }),
      );
      write(
        fixture.root,
        "src/date-a.ts",
        [
          "export function formatDate(value: Date) {",
          "  const formatter = new Intl.DateTimeFormat('en-US', { dateStyle: 'medium' });",
          "  return formatter.format(value);",
          "}",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/date-b.ts",
        [
          "export function dateFormat(input: Date) {",
          "  return new Intl.DateTimeFormat('en-US', { dateStyle: 'medium' }).format(input);",
          "}",
          "",
        ].join("\n"),
      );

      runProducer(
        "build-function-clone-index.mjs",
        fixture.root,
        fixture.output,
      );
      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const clones = readChecklist(fixture.output).B1_duplicate_implementation;
      const near = clones.topNearFunctionCandidates[0];

      expect(clones).toMatchObject({
        gate: "watch",
        exactBodyGroups: 0,
        structureGroupCandidates: 0,
        nearFunctionCandidates: 1,
      });
      expect(near.identities).toEqual(
        expect.arrayContaining([
          "src/date-a.ts::formatDate",
          "src/date-b.ts::dateFormat",
        ]),
      );
      expect(near.risk).toBe("review-only");
      expect(near.reason).toMatch(/not proof of semantic equivalence/);
      expect(clones._citation_hint).toContain("nearFunctionCandidates");
      expect(clones._context_check_required).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("counts silent catches while separating documented empty catch sites", () => {
    const fixture = createFixture("vitest-checklist-e2-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-e2", type: "module" }),
      );
      write(
        fixture.root,
        "src/a.ts",
        [
          "export function a() {",
          "  try { JSON.parse('x'); } catch {}",
          "  try { JSON.parse('y'); } catch (e) {}",
          "  try { JSON.parse('z'); } catch { /* intentionally optional */ }",
          "  try { JSON.parse('q'); } catch (e) {",
          "    // intentionally optional",
          "  }",
          "}",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/b.ts",
        [
          "// Non-silent — should NOT be counted:",
          "export function b() {",
          "  try { JSON.parse('z'); } catch (e) { console.error(e); }",
          "}",
          "",
        ].join("\n"),
      );

      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const catchFacts = readChecklist(fixture.output).E2_silent_catch;

      expect(catchFacts.count).toBe(2);
      expect(catchFacts.sites.some((site) => site.file.endsWith("b.ts"))).toBe(
        false,
      );
      expect(catchFacts.gate).toBe("watch");
      expect(catchFacts.documentedCount).toBe(2);
      expect(
        catchFacts.documentedSites.every((site) => site.file.endsWith("a.ts")),
      ).toBe(true);
      expect(catchFacts.anonymousCount).toBe(2);
      expect(catchFacts.nonEmptyAnonymousCount).toBe(0);
      expect(catchFacts.analysis).toBe("oxc-ast-catch-clause");
      expect(
        catchFacts.sites.every(
          (site) =>
            site.fileRole === "production" &&
            typeof site.bodyStatementCount === "number",
        ),
      ).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("surfaces non-empty anonymous catches without inflating empty silent count", () => {
    const fixture = createFixture("vitest-checklist-e2-anon-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-e2-anon", type: "module" }),
      );
      write(
        fixture.root,
        "src/a.ts",
        [
          "export function a(raw: string) {",
          "  try { return JSON.parse(raw); } catch { return null; }",
          "}",
          "",
        ].join("\n"),
      );

      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const catchFacts = readChecklist(fixture.output).E2_silent_catch;

      expect(catchFacts.count).toBe(0);
      expect(catchFacts.nonEmptyAnonymousCount).toBe(1);
      expect(catchFacts.anonymousCount).toBe(1);
      expect(catchFacts.gate).toBe("watch");
    } finally {
      fixture.cleanup();
    }
  });

  it("surfaces unused catch parameters while ignoring used catch parameters", () => {
    const fixture = createFixture("vitest-checklist-e2-unused-param-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-e2-unused-param", type: "module" }),
      );
      write(
        fixture.root,
        "src/a.ts",
        [
          "export function ignored(raw: string) {",
          "  try { return JSON.parse(raw); } catch (err) { return null; }",
          "}",
          "export function logged(raw: string) {",
          "  try { return JSON.parse(raw); } catch (err) { console.error(err); return null; }",
          "}",
          "",
        ].join("\n"),
      );

      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const catchFacts = readChecklist(fixture.output).E2_silent_catch;

      expect(catchFacts.count).toBe(0);
      expect(catchFacts.unusedParamCount).toBe(1);
      expect(catchFacts.unusedParamSites[0]).toMatchObject({
        paramName: "err",
        line: 2,
      });
      expect(catchFacts.gate).toBe("watch");
      expect(catchFacts.unusedParamSites).toHaveLength(1);
      expect(catchFacts._citation_hint).toContain("unusedParamCount = 1");
      expect(catchFacts._citation_hint).toContain(
        "analysis = oxc-ast-catch-clause",
      );
    } finally {
      fixture.cleanup();
    }
  });

  it("grounds C5 lint boundary evidence from no-restricted-imports", () => {
    const fixture = createFixture("vitest-checklist-c5-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-c5", type: "module" }),
      );
      write(
        fixture.root,
        "eslint.config.mjs",
        [
          "export default [{",
          "  rules: {",
          "    'no-restricted-imports': ['error', { patterns: ['../*.mjs'] }],",
          "  },",
          "}];",
          "",
        ].join("\n"),
      );
      write(fixture.root, "src/ok.ts", "export const ok = 1;\n");

      runProducer("triage-repo.mjs", fixture.root, fixture.output);
      runProducer("checklist-facts.mjs", fixture.root, fixture.output);
      const triage = JSON.parse(
        readFileSync(path.join(fixture.output, "triage.json"), "utf8"),
      );
      const cf = readChecklist(fixture.output);

      expect(
        triage.boundaries.some(
          (boundary) =>
            boundary.rule === "no-restricted-imports" &&
            boundary.file === "eslint.config.mjs",
        ),
      ).toBe(true);
      expect(cf.C5_lint_enforcement).toMatchObject({
        gate: "ok",
        boundaryRulePresent: true,
      });
    } finally {
      fixture.cleanup();
    }
  });

  it("populates artifact-backed checklist facts and input bits from the producer pipeline", () => {
    const fixture = createFixture("vitest-checklist-pipeline-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "cf-pipe", type: "module" }),
      );
      write(
        fixture.root,
        "src/entry.ts",
        [
          "import { helper } from './helper.js';",
          "export const live = helper();",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/helper.ts",
        [
          "export function helper() { return 1; }",
          "export const maybeDead = 2;",
          "",
        ].join("\n"),
      );

      for (const script of [
        "triage-repo.mjs",
        "measure-topology.mjs",
        "build-symbol-graph.mjs",
        "classify-dead-exports.mjs",
        "rank-fixes.mjs",
        "check-barrel-discipline.mjs",
        "build-shape-index.mjs",
        "checklist-facts.mjs",
      ]) {
        runProducer(script, fixture.root, fixture.output);
      }

      const cf = readChecklist(fixture.output);

      expect(cf.A5_decoupling_ratio.available).not.toBe(false);
      expect(typeof cf.A5_decoupling_ratio.ratioLowerBound).toBe("number");
      expect(cf.A6_circular_deps).toMatchObject({
        sccCount: 0,
        gate: "ok",
      });
      expect(typeof cf.B3_dead_code.total).toBe("number");
      expect(cf.C5_lint_enforcement.available).not.toBe(false);
      expect(cf.C7_barrel_amplification.gate).toBe("ok");
      expect(cf.meta.inputsPresent).toMatchObject({
        "topology.json": true,
        "fix-plan.json": true,
        "triage.json": true,
        "barrels.json": true,
        "shape-index.json": true,
      });
    } finally {
      fixture.cleanup();
    }
  });
});

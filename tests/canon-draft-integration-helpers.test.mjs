import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterEach, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const CANON_CLI = path.join(ROOT, "generate-canon-draft.mjs");
const fixtures = [];

function createFixture(prefix) {
  const fixture = createTempRepoFixture({ prefix });
  fixtures.push(fixture);
  return fixture;
}

function runCanon(fixture) {
  execFileSync(
    process.execPath,
    [
      CANON_CLI,
      "--root",
      fixture.root,
      "--output",
      fixture.output,
      "--source",
      "helper-registry",
    ],
    { stdio: "ignore" },
  );
  return fixture.read("canonical-draft/helper-registry.md");
}

function parseHelperTableRows(md) {
  const lines = md.split("\n");
  const start = lines.findIndex((line) =>
    line.startsWith("| Name | Identity | Owner | Signature"),
  );
  if (start < 0) return [];
  const rows = [];
  for (let i = start + 2; i < lines.length; i++) {
    const line = lines[i];
    if (!line.startsWith("|")) break;
    const cells = line
      .split("|")
      .slice(1, -1)
      .map((cell) => cell.trim());
    if (cells.length < 7) continue;
    rows.push({
      name: cells[0],
      identity: cells[1],
      owner: cells[2],
      signature: cells[3],
      fanIn: cells[4],
      status: cells[5],
      tags: cells[6],
      anySignal: cells[7] ?? "",
    });
  }
  return rows;
}

afterEach(() => {
  while (fixtures.length > 0) {
    fixtures.pop().cleanup();
  }
});

describe("helper-registry canon draft integration", () => {
  it("F1. central helpers render with fan-in and owner identity", () => {
    const fixture = createFixture("vitest-cdh-i-central-");
    fixture.write(
      "src/util.ts",
      "export function tryParseJson(raw: string) { try { return JSON.parse(raw) } catch { return null } }\n",
    );
    fixture.write(
      "src/c1.ts",
      "import { tryParseJson } from './util';\n" +
        "export const a = tryParseJson('1');\n",
    );
    fixture.write(
      "src/c2.ts",
      "import { tryParseJson } from './util';\n" +
        "export const b = tryParseJson('2');\n",
    );
    fixture.write(
      "src/c3.ts",
      "import { tryParseJson } from './util';\n" +
        "export const c = tryParseJson('3');\n",
    );

    const row = parseHelperTableRows(runCanon(fixture)).find(
      (entry) => entry.name === "`tryParseJson`",
    );

    expect(row).toBeDefined();
    expect(row.fanIn).toBe("3");
    expect(row.status).toContain("central-helper");
    expect(row.identity).toContain("src/util.ts::tryParseJson");
  }, 30_000);

  it("F2. fan-in counts distinct consumer files instead of repeated call sites", () => {
    const fixture = createFixture("vitest-cdh-i-fanin-");
    fixture.write(
      "src/util.ts",
      "export function doWork(x: number) { return x * 2 }\n",
    );
    fixture.write(
      "src/c.ts",
      "import { doWork } from './util';\n" +
        "export const a = doWork(1);\n" +
        "export const b = doWork(2);\n" +
        "export const c = doWork(3);\n" +
        "export const d = doWork(4);\n" +
        "export const e = doWork(5);\n" +
        "export const f = doWork(6);\n",
    );

    const row = parseHelperTableRows(runCanon(fixture)).find(
      (entry) => entry.name === "`doWork`",
    );

    expect(row).toBeDefined();
    expect(row.fanIn).toBe("1");
    expect(row.status).toContain("shared-helper");
  }, 30_000);

  it("F3. exported-never-called helpers stay visible as zero-internal-fan-in", () => {
    const fixture = createFixture("vitest-cdh-i-orphan-");
    fixture.write(
      "src/public.ts",
      "export function unusedButPublic(x: number) { return x + 1 }\n",
    );

    const row = parseHelperTableRows(runCanon(fixture)).find(
      (entry) => entry.name === "`unusedButPublic`",
    );

    expect(row).toBeDefined();
    expect(row.fanIn).toBe("0");
    expect(row.status).toContain("zero-internal-fan-in-helper");
  }, 30_000);

  it("F4. callback-only helper consumption is captured through import resolution", () => {
    const fixture = createFixture("vitest-cdh-i-callback-");
    fixture.write(
      "src/util.ts",
      "export function parseOne(raw: string) { return raw.trim() }\n",
    );
    fixture.write(
      "src/c.ts",
      "import { parseOne } from './util';\n" +
        "export const all = ['a', 'b', 'c'].map(parseOne);\n",
    );

    const row = parseHelperTableRows(runCanon(fixture)).find(
      (entry) => entry.name === "`parseOne`",
    );

    expect(row).toBeDefined();
    expect(row.fanIn).toBe("1");
  }, 30_000);

  it("F5. cross-file duplicate helpers share the strong duplicate group label", () => {
    const fixture = createFixture("vitest-cdh-i-dup-");
    fixture.write(
      "src/a.ts",
      "export function renderThing(x: number) { return x * 2 }\n",
    );
    fixture.write(
      "src/b.ts",
      "export function renderThing(x: string) { return x.toUpperCase() }\n",
    );
    fixture.write(
      "src/c1.ts",
      "import { renderThing } from './a'; export const x = renderThing(1);\n",
    );
    fixture.write(
      "src/c2.ts",
      "import { renderThing } from './a'; export const y = renderThing(2);\n",
    );
    fixture.write(
      "src/c3.ts",
      "import { renderThing } from './a'; export const z = renderThing(3);\n",
    );
    fixture.write(
      "src/d1.ts",
      "import { renderThing } from './b'; export const x = renderThing('hi');\n",
    );
    fixture.write(
      "src/d2.ts",
      "import { renderThing } from './b'; export const y = renderThing('ho');\n",
    );
    fixture.write(
      "src/d3.ts",
      "import { renderThing } from './b'; export const z = renderThing('ha');\n",
    );

    const rows = parseHelperTableRows(runCanon(fixture)).filter(
      (entry) => entry.name === "`renderThing`",
    );
    const labels = new Set(rows.map((row) => row.status.split(" ")[0]));

    expect(rows).toHaveLength(2);
    expect(labels).toEqual(new Set(["HELPER_DUPLICATE_STRONG"]));
  }, 30_000);

  it("F6. const-var arrow helpers surface with fan-in", () => {
    const fixture = createFixture("vitest-cdh-i-const-");
    fixture.write(
      "src/util.ts",
      "export const mapThing = (x: number) => x + 1;\n",
    );
    fixture.write(
      "src/c.ts",
      "import { mapThing } from './util'; export const v = mapThing(1);\n",
    );

    const row = parseHelperTableRows(runCanon(fixture)).find(
      (entry) => entry.name === "`mapThing`",
    );

    expect(row).toBeDefined();
    expect(row.fanIn).toBe("1");
  }, 30_000);

  it("F7. empty repos still render a helper-registry draft with zero rows", () => {
    const fixture = createFixture("vitest-cdh-i-empty-");

    const md = runCanon(fixture);
    const rows = parseHelperTableRows(md);

    expect(md).toContain("# Helper registry draft");
    expect(rows).toHaveLength(0);
  }, 30_000);

  it("F8. call-graph cross-check diagnostics surface in notes with owner identity", () => {
    const fixture = createFixture("vitest-cdh-i-crosscheck-");
    fixture.write(
      "src/reflective.ts",
      "export function viaReflection() { return 'hi' }\n",
    );
    fixture.writeJson(
      "call-graph.json",
      {
        meta: {
          generated: "2026-05-15T00:00:00Z",
          root: fixture.root,
          tool: "build-call-graph.mjs",
        },
        summary: {},
        topCallees: [
          { file: "src/reflective.ts", name: "viaReflection", count: 8 },
        ],
      },
      { to: "output" },
    );

    const md = runCanon(fixture);

    expect(md).toMatch(
      /call-graph-evidence-but-no-ast-consumers|call-graph-cross-check/,
    );
    expect(md).toContain("src/reflective.ts::viaReflection");
  }, 30_000);

  it("F9. fixture parser recovers rows whose statuses are canonical helper labels", () => {
    const fixture = createFixture("vitest-cdh-i-labels-");
    fixture.write(
      "src/a.ts",
      "export function alpha(x: number) { return x }\n",
    );
    fixture.write("src/b.ts", "export function beta(x: number) { return x }\n");

    const rows = parseHelperTableRows(runCanon(fixture));
    const labels = rows.map((row) => row.status.split(" ")[0]);
    const canonicalHelperLabels = new Set([
      "HELPER_DUPLICATE_STRONG",
      "HELPER_DUPLICATE_REVIEW",
      "HELPER_LOCAL_COMMON",
      "ANY_COLLISION_HELPER",
      "severely-any-contaminated-helper",
      "central-helper",
      "shared-helper",
      "zero-internal-fan-in-helper",
      "low-signal-helper-name",
    ]);

    expect(rows).toHaveLength(2);
    expect(rows.map((row) => row.name)).toEqual(
      expect.arrayContaining(["`alpha`", "`beta`"]),
    );
    expect(labels.every((label) => canonicalHelperLabels.has(label))).toBe(
      true,
    );
  }, 30_000);
});

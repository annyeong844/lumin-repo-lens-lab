import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterEach, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const SYMBOLS_CLI = path.join(ROOT, "build-symbol-graph.mjs");
const CANON_CLI = path.join(ROOT, "generate-canon-draft.mjs");
const fixtures = [];

function createFixture(prefix) {
  const fixture = createTempRepoFixture({ prefix });
  fixtures.push(fixture);
  return fixture;
}

function runSymbols(fixture) {
  execFileSync(
    process.execPath,
    [SYMBOLS_CLI, "--root", fixture.root, "--output", fixture.output],
    { stdio: "ignore" },
  );
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
      "type-ownership",
    ],
    { stdio: "ignore" },
  );
}

function runPipeline(fixture) {
  runSymbols(fixture);
  runCanon(fixture);
  return fixture.read("canonical-draft/type-ownership.md");
}

function parseTableRows(md) {
  const lines = md.split("\n");
  const start = lines.findIndex((line) => line.startsWith("| Name | Identity"));
  if (start < 0) return [];
  const headerCells = lines[start]
    .split("|")
    .slice(1, -1)
    .map((cell) => cell.trim());
  const index = Object.fromEntries(
    headerCells.map((cell, columnIndex) => [cell, columnIndex]),
  );
  const rows = [];
  for (let i = start + 2; i < lines.length; i++) {
    const line = lines[i];
    if (!line.startsWith("|")) break;
    const cells = line
      .split("|")
      .slice(1, -1)
      .map((cell) => cell.trim());
    if (cells.length < 5) continue;
    rows.push({
      name: cells[0],
      identity: cells[1],
      owner: cells[2],
      fanIn: cells[index["Fan-in"]],
      status: cells[index.Status],
    });
  }
  return rows;
}

afterEach(() => {
  while (fixtures.length > 0) {
    fixtures.pop().cleanup();
  }
});

describe("type-ownership canon draft integration", () => {
  it("F1. same type name across files emits one row per owner with a group classification", () => {
    const fixture = createFixture("vitest-cdi-grp-");
    fixture.write("src/a.ts", "export type Result = { ok: true };\n");
    fixture.write("src/b.ts", "export type Result = { err: string };\n");
    fixture.write(
      "src/consumer1.ts",
      "import { Result as R1 } from './a';\n" +
        "import { Result as R2 } from './b';\n" +
        "export const x: R1 = { ok: true };\n" +
        "export const y: R2 = { err: '' };\n",
    );
    fixture.write(
      "src/consumer2.ts",
      "import { Result } from './a';\n" +
        "export const z: Result = { ok: true };\n",
    );
    fixture.write(
      "src/consumer3.ts",
      "import { Result } from './a';\n" +
        "export const w: Result = { ok: true };\n",
    );

    const rows = parseTableRows(runPipeline(fixture));
    const resultRows = rows.filter((row) => row.name === "`Result`");
    const labels = new Set(resultRows.map((row) => row.status.split(" ")[0]));

    expect(resultRows).toHaveLength(2);
    expect(labels.size).toBe(1);
    expect([
      "DUPLICATE_STRONG",
      "DUPLICATE_REVIEW",
      "LOCAL_COMMON_NAME",
      "ANY_COLLISION",
    ]).toContain([...labels][0]);
  }, 30_000);

  it("F2. distinct exported type names keep distinct identities and single-identity labels", () => {
    const fixture = createFixture("vitest-cdi-cross-");
    fixture.write("src/api.ts", "export interface User { id: string }\n");
    fixture.write("src/blog.ts", "export type Post = { id: string };\n");
    fixture.write(
      "src/use.ts",
      "import { User } from './api';\n" +
        "import { Post } from './blog';\n" +
        "export const u: User = { id: '' };\n" +
        "export const p: Post = { id: '' };\n",
    );

    const rows = parseTableRows(runPipeline(fixture));
    const userRow = rows.find((row) => row.name === "`User`");
    const postRow = rows.find((row) => row.name === "`Post`");

    expect(userRow).toBeDefined();
    expect(postRow).toBeDefined();
    expect(userRow.identity).not.toBe(postRow.identity);
    expect(userRow.status).toMatch(
      /single-owner-(strong|weak)|zero-internal-fan-in|low-signal-type-name/,
    );
  }, 30_000);

  it("F3. re-export chains retain the terminal owner identity instead of the barrel", () => {
    const fixture = createFixture("vitest-cdi-reexp-");
    fixture.write("src/y.ts", "export type X = { v: number };\n");
    fixture.write("src/index.ts", "export { X } from './y';\n");
    fixture.write(
      "src/consumer.ts",
      "import { X } from './index';\n" + "export const v: X = { v: 1 };\n",
    );

    const rows = parseTableRows(runPipeline(fixture));
    const xRow = rows.find((row) => row.name === "`X`");

    expect(xRow).toBeDefined();
    expect(xRow.identity).toContain("src/y.ts::X");
    expect(xRow.identity).not.toContain("src/index.ts::X");
  }, 30_000);

  it("F4. emitted Markdown round-trips to the expected type row names", () => {
    const fixture = createFixture("vitest-cdi-rt-");
    fixture.write("src/one.ts", "export type A = number;\n");
    fixture.write("src/two.ts", "export interface B { s: string };\n");

    const rows = parseTableRows(runPipeline(fixture));

    expect(rows).toHaveLength(2);
    expect(new Set(rows.map((row) => row.name)).size).toBe(2);
    expect(rows.some((row) => row.name === "`A`")).toBe(true);
    expect(rows.some((row) => row.name === "`B`")).toBe(true);
  }, 30_000);
});

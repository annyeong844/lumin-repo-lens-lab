import { execFileSync, spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterEach, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const CANON_CLI = path.join(ROOT, "generate-canon-draft.mjs");
const TOPOLOGY_CLI = path.join(ROOT, "measure-topology.mjs");
const TRIAGE_CLI = path.join(ROOT, "triage-repo.mjs");
const fixtures = [];

function createFixture(prefix) {
  const fixture = createTempRepoFixture({ prefix });
  fixtures.push(fixture);
  return fixture;
}

function runProducersAndCanon(fixture, extraCanonFlags = []) {
  execFileSync(
    process.execPath,
    [TRIAGE_CLI, "--root", fixture.root, "--output", fixture.output],
    { stdio: "ignore" },
  );
  execFileSync(
    process.execPath,
    [TOPOLOGY_CLI, "--root", fixture.root, "--output", fixture.output],
    { stdio: "ignore" },
  );
  execFileSync(
    process.execPath,
    [
      CANON_CLI,
      "--root",
      fixture.root,
      "--output",
      fixture.output,
      "--source",
      "topology",
      ...extraCanonFlags,
    ],
    { stdio: "ignore" },
  );
  return fixture.read("canonical-draft/topology.md");
}

function parseInventoryRows(md) {
  const lines = md.split("\n");
  const start = lines.findIndex((line) =>
    line.startsWith("| Submodule | Files"),
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
      submodule: cells[0],
      files: Number(cells[1]),
      loc: Number(cells[2]),
      inEdges: Number(cells[3]),
      outEdges: Number(cells[4]),
      scc: cells[5],
      status: cells[6],
      tags: cells[7] ?? "",
    });
  }
  return rows;
}

afterEach(() => {
  while (fixtures.length > 0) {
    fixtures.pop().cleanup();
  }
});

describe("topology canon draft integration", () => {
  it("F1. inventory rows match distinct submodules and topology node totals", () => {
    const fixture = createFixture("vitest-cdt-i-3sub-");
    fixture.write("_lib/a.mjs", "export const a = 1;\n");
    fixture.write("_lib/b.mjs", "export const b = 2;\n");
    fixture.write(
      "src/main.mjs",
      "import { a } from '../_lib/a.mjs'; export const x = a;\n",
    );
    fixture.write(
      "src/util.mjs",
      "import { b } from '../_lib/b.mjs'; export const y = b;\n",
    );
    fixture.write(
      "tests/smoke.mjs",
      "import { a } from '../_lib/a.mjs'; export const z = a;\n",
    );

    const rows = parseInventoryRows(runProducersAndCanon(fixture));
    const topology = fixture.readJson("topology.json", { from: "output" });
    const filesSum = rows.reduce((sum, row) => sum + row.files, 0);
    const names = new Set(rows.map((row) => row.submodule));

    expect(rows).toHaveLength(3);
    expect(filesSum).toBe(topology.summary.files);
    expect(filesSum).toBe(Object.keys(topology.nodes).length);
    expect(names).toEqual(new Set(["`_lib`", "`src`", "`tests`"]));
  }, 30_000);

  it("F2. SCC fixtures render forbidden-cycle evidence and cyclic submodule status", () => {
    const fixture = createFixture("vitest-cdt-i-scc-");
    fixture.write(
      "core/a.mjs",
      "import { b } from './b.mjs';\n" +
        "export function a() { return b() + 1 }\n",
    );
    fixture.write(
      "core/b.mjs",
      "import { c } from './c.mjs';\n" +
        "export function b() { return c() + 1 }\n",
    );
    fixture.write(
      "core/c.mjs",
      "import { a } from './a.mjs';\n" +
        "export function c() { return 1 + (Math.random() > 2 ? a() : 0) }\n",
    );

    const md = runProducersAndCanon(fixture);
    const rows = parseInventoryRows(md);
    const core = rows.find((row) => row.submodule === "`core`");

    expect(md).toContain("forbidden-cycle");
    expect(md).toContain("`core/a.mjs`");
    expect(md).toContain("`core/b.mjs`");
    expect(md).toContain("`core/c.mjs`");
    expect(md).toContain("❌ Cycles observed");
    expect(core.status).toContain("cyclic-submodule");
  }, 30_000);

  it("F3. oversize fixtures distinguish extreme and ordinary oversize files", () => {
    const fixture = createFixture("vitest-cdt-i-over-");
    const bigLines =
      Array.from({ length: 1200 }, (_, i) => `export const x${i} = ${i};`).join(
        "\n",
      ) + "\n";
    const midLines =
      Array.from({ length: 500 }, (_, i) => `export const y${i} = ${i};`).join(
        "\n",
      ) + "\n";
    fixture.write("huge/h.mjs", bigLines);
    fixture.write("mid/m.mjs", midLines);

    const md = runProducersAndCanon(fixture);

    expect(md).toContain("## 4. Oversize files");
    expect(md).toContain("`huge/h.mjs`");
    expect(md).toContain("extreme-oversize");
    expect(md).toMatch(/`mid\/m\.mjs`[^\n]*\soversize\s/);
  }, 30_000);

  it("F4. acyclic fixtures render an explicit no-cycle banner", () => {
    const fixture = createFixture("vitest-cdt-i-acyc-");
    fixture.write("_lib/util.mjs", "export const x = 1;\n");
    fixture.write(
      "src/main.mjs",
      "import { x } from '../_lib/util.mjs'; export const y = x;\n",
    );

    const md = runProducersAndCanon(fixture);

    expect(md).toContain("✅ No submodule-level cycles observed");
  }, 30_000);

  it("F5. missing topology.json exits 2 and points at measure-topology", () => {
    const fixture = createFixture("vitest-cdt-i-notopo-");
    fixture.write("src/x.mjs", "export const x = 1;\n");

    const result = spawnSync(
      process.execPath,
      [
        CANON_CLI,
        "--root",
        fixture.root,
        "--output",
        fixture.output,
        "--source",
        "topology",
      ],
      { encoding: "utf8" },
    );

    expect(result.status).toBe(2);
    expect(result.stderr).toMatch(/measure-topology\.mjs/);
  }, 30_000);

  it("F6. missing triage.json exits 0 and omits workspace boundaries", () => {
    const fixture = createFixture("vitest-cdt-i-notriage-");
    fixture.write("src/x.mjs", "export const x = 1;\n");
    execFileSync(
      process.execPath,
      [TOPOLOGY_CLI, "--root", fixture.root, "--output", fixture.output],
      { stdio: "ignore" },
    );

    const result = spawnSync(
      process.execPath,
      [
        CANON_CLI,
        "--root",
        fixture.root,
        "--output",
        fixture.output,
        "--source",
        "topology",
      ],
      { encoding: "utf8" },
    );
    const md = fixture.read("canonical-draft/topology.md");

    expect(result.status).toBe(0);
    expect(md).not.toContain("## 5. Workspace boundaries");
  }, 30_000);

  it("F7. emitted submodule statuses stay canonical and high fan-in becomes shared", () => {
    const fixture = createFixture("vitest-cdt-i-labels-");
    fixture.write("hub/index.mjs", "export const h = 1;\n");
    for (let i = 0; i < 5; i++) {
      fixture.write(
        `leaf${i}/x.mjs`,
        `import { h } from '../hub/index.mjs'; export const x${i} = h;\n`,
      );
    }

    const rows = parseInventoryRows(runProducersAndCanon(fixture));
    const hub = rows.find((row) => row.submodule === "`hub`");

    expect(
      rows.every((row) =>
        [
          "cyclic-submodule",
          "isolated-submodule",
          "shared-submodule",
          "leaf-submodule",
          "scoped-submodule",
        ].some((label) => row.status.includes(label)),
      ),
    ).toBe(true);
    expect(hub.status).toContain("shared-submodule");
  }, 30_000);

  it("F8. paths with spaces and dollar signs survive the full pipeline", () => {
    const fixture = createFixture("my $root-");
    fixture.write("_lib/util.mjs", "export const x = 1;\n");
    fixture.write(
      "src/main.mjs",
      "import { x } from '../_lib/util.mjs'; export const y = x;\n",
    );

    const md = runProducersAndCanon(fixture);

    expect(md).toContain("# Topology draft");
    expect(md).toContain("## 1. Submodule inventory");
  }, 30_000);
});

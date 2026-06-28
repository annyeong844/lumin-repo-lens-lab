import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterEach, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const CLI = path.join(ROOT, "generate-canon-draft.mjs");
const TOPOLOGY_CLI = path.join(ROOT, "measure-topology.mjs");
const fixtures = [];

function createFixture(prefix, options = {}) {
  const fixture = createTempRepoFixture({ prefix, ...options });
  fixtures.push(fixture);
  return fixture;
}

function writeTopologyFixture(fixture) {
  fixture.write(
    "package.json",
    JSON.stringify({ name: "topo-fx", type: "module" }),
  );
  fixture.write("_lib/util.mjs", "export function helper() { return 1 }\n");
  fixture.write(
    "src/app.mjs",
    "import { helper } from '../_lib/util.mjs';\nexport const x = helper();\n",
  );
}

function runMeasureTopology(fixture) {
  execFileSync(process.execPath, [
    TOPOLOGY_CLI,
    "--root",
    fixture.root,
    "--output",
    fixture.output,
  ]);
}

function runCli(fixture, args = [], options = {}) {
  return spawnSync(process.execPath, [CLI, "--root", fixture.root, ...args], {
    cwd: ROOT,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    ...options,
  });
}

function execCli(fixture, args = []) {
  execFileSync(process.execPath, [CLI, "--root", fixture.root, ...args], {
    cwd: ROOT,
    stdio: "ignore",
  });
}

afterEach(() => {
  while (fixtures.length > 0) {
    fixtures.pop().cleanup();
  }
});

describe("generate-canon-draft topology CLI", () => {
  it("T1/T11/T13. emits a topology draft with full-list confidence, production scope, and stderr summary", () => {
    const fixture = createFixture("fx-vitest-canon-topology-");
    writeTopologyFixture(fixture);
    runMeasureTopology(fixture);

    const result = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "topology",
      "--production",
    ]);
    const md = fixture.read("canonical-draft/topology.md");

    expect(result.status).toBe(0);
    expect(md).toContain("# Topology draft");
    expect(md).toContain("## 1. Submodule inventory");
    expect(md).toContain("## 3. Cycles");
    expect(md).toContain("CrossEdgeSource: full-list");
    expect(md).toContain("ClassificationConfidence: high");
    expect(md).toContain("TS/JS production files");
    expect(result.stderr).toMatch(/submodule/i);
    expect(result.stderr).toMatch(/CrossEdgeSource: full-list|full-list/);
  }, 30_000);

  it("T2/T3/T14. rejects unknown sources, distinguishes missing topology, and rejects missing root", () => {
    const fixture = createFixture("fx-vitest-canon-topology-errors-");
    writeTopologyFixture(fixture);

    const badSource = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "foobar",
    ]);
    const missingTopology = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "topology",
    ]);
    const missingRoot = spawnSync(
      process.execPath,
      [CLI, "--output", fixture.output, "--source", "topology"],
      { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    );

    expect(badSource.status).toBe(1);
    expect(badSource.stderr).toMatch(/type-ownership/);
    expect(badSource.stderr).toMatch(/helper-registry/);
    expect(badSource.stderr).toMatch(/topology/);
    expect(badSource.stderr).toMatch(/naming/);
    expect(missingTopology.status).toBe(2);
    expect(missingTopology.stderr).toMatch(/measure-topology\.mjs/);
    expect(missingRoot.status).toBe(1);
  }, 30_000);

  it("T4/T5/T6/T7. preserves other sources, versioning, existing-canon header, and canon-output override", () => {
    const fixture = createFixture("fx-vitest-canon-topology-io-");
    writeTopologyFixture(fixture);
    fixture.write("src/types.ts", "export type User = { id: string };\n");
    fixture.writeJson(
      "symbols.json",
      {
        meta: {
          tool: "build-symbol-graph.mjs",
          generated: "2026-04-21T00:00:00Z",
          root: fixture.root,
          supports: { identityFanIn: true },
        },
        defIndex: {
          "src/types.ts": {
            User: { name: "User", kind: "TSTypeAliasDeclaration", line: 1 },
          },
        },
        fanInByIdentity: {},
        reExportsByFile: {},
      },
      { to: "output" },
    );
    runMeasureTopology(fixture);

    expect(
      runCli(fixture, [
        "--output",
        fixture.output,
        "--source",
        "type-ownership",
      ]).status,
    ).toBe(0);
    expect(
      runCli(fixture, [
        "--output",
        fixture.output,
        "--source",
        "helper-registry",
      ]).status,
    ).toBe(0);

    execCli(fixture, ["--output", fixture.output, "--source", "topology"]);
    const first = fixture.read("canonical-draft/topology.md");
    fixture.write("new/extra.mjs", "export const extra = 1;\n");
    runMeasureTopology(fixture);
    execCli(fixture, ["--output", fixture.output, "--source", "topology"]);
    expect(readdirSync(fixture.path("canonical-draft"))).toContain(
      "topology.v2.md",
    );
    expect(fixture.read("canonical-draft/topology.md")).toBe(first);

    const existing = createFixture("fx-vitest-canon-topology-existing-");
    writeTopologyFixture(existing);
    runMeasureTopology(existing);
    existing.write("canonical/topology.md", "# Existing canon\n");
    execCli(existing, ["--output", existing.output, "--source", "topology"]);
    expect(existing.read("canonical-draft/topology.md")).toContain(
      "Existing canon detected",
    );

    const custom = createFixture("fx-vitest-canon-topology-custom-");
    writeTopologyFixture(custom);
    runMeasureTopology(custom);
    const customOut = custom.path("custom-output");
    execCli(custom, [
      "--output",
      custom.output,
      "--canon-output",
      customOut,
      "--source",
      "topology",
    ]);
    expect(existsSync(path.join(customOut, "topology.md"))).toBe(true);
    expect(existsSync(custom.path("canonical-draft"))).toBe(false);
  }, 60_000);

  it("T8/T9/T10/T12. handles shell-sensitive paths, missing triage, incomplete topology, and degraded cross-edge evidence", () => {
    const fixture = createFixture("my $root-topology-", {
      prefix: "my $root-topology-",
    });
    writeTopologyFixture(fixture);
    runMeasureTopology(fixture);
    expect(
      runCli(fixture, ["--output", fixture.output, "--source", "topology"])
        .status,
    ).toBe(0);

    const synthetic = createFixture("fx-vitest-canon-topology-degraded-");
    synthetic.writeJson(
      "topology.json",
      {
        meta: { complete: false, generated: "2026-05-15T00:00:00.000Z" },
        summary: { lens: "runtime" },
        nodes: {
          "a/x.mjs": { loc: 100 },
          "b/y.mjs": { loc: 100 },
        },
        crossSubmoduleTop: [{ edge: "a → b", count: 5 }],
        sccs: [],
        largestFiles: [],
      },
      { to: "output" },
    );
    const result = runCli(synthetic, [
      "--output",
      synthetic.output,
      "--source",
      "topology",
    ]);
    const md = synthetic.read("canonical-draft/topology.md");

    expect(result.status).toBe(0);
    expect(md).not.toContain("## 5. Workspace boundaries");
    expect(md).toContain("TopologyComplete: false");
    expect(md).toContain("CrossEdgeSource: top-30-only");
    expect(md).toContain("ClassificationConfidence: medium");
    expect(md).toContain("top-30 cross-edge lens");
  }, 30_000);
});

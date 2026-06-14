import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  utimesSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { afterAll, describe, expect, it } from "vitest";

const DIR = path.resolve(import.meta.dirname, "..");
const CLI = path.join(DIR, "check-canon.mjs");
const cleanup = [];
const CLI_TEST_TIMEOUT_MS = 30000;

afterAll(() => {
  for (const root of cleanup) rmSync(root, { recursive: true, force: true });
});

function runCli(args, { cwd } = {}) {
  const result = spawnSync(process.execPath, [CLI, ...args], {
    cwd: cwd ?? DIR,
    encoding: "utf8",
  });
  return {
    exit: result.status ?? -1,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

function makeTypeFixture({ canonical, symbols }) {
  const root = mkdtempSync(path.join(tmpdir(), "vitest-check-canon-cli-type-"));
  mkdirSync(path.join(root, "canonical"), { recursive: true });
  mkdirSync(path.join(root, "audit-output"), { recursive: true });
  if (canonical !== null) {
    writeFileSync(
      path.join(root, "canonical", "type-ownership.md"),
      canonical,
      "utf8",
    );
  }
  if (symbols) {
    writeFileSync(
      path.join(root, "audit-output", "symbols.json"),
      JSON.stringify(symbols, null, 2),
      "utf8",
    );
  }
  cleanup.push(root);
  return root;
}

function basicSymbols({ typeDefs = [] } = {}) {
  const defIndex = {};
  const fanInByIdentity = {};
  for (const definition of typeDefs) {
    defIndex[definition.ownerFile] ??= {};
    defIndex[definition.ownerFile][definition.name] = {
      kind: definition.kind ?? "TSInterfaceDeclaration",
      line: definition.line,
      anyContamination: null,
    };
    fanInByIdentity[`${definition.ownerFile}::${definition.name}`] =
      definition.fanIn;
  }
  return {
    meta: { scope: "fixture", supports: { identityFanIn: true } },
    defIndex,
    fanInByIdentity,
    reExportsByFile: {},
  };
}

function makeHash(ch) {
  return `sha256:${ch.repeat(64)}`;
}

function makeShapeIndex(facts, { complete = true } = {}) {
  const groupsByHash = {};
  for (const fact of facts) {
    groupsByHash[fact.hash] ??= [];
    groupsByHash[fact.hash].push(fact.identity);
  }
  for (const ids of Object.values(groupsByHash)) ids.sort();
  return {
    schemaVersion: "shape-index.v1",
    meta: { complete },
    facts: facts.map((fact) => ({ ...fact })),
    groupsByHash,
  };
}

function makeHelperFixture({ canonical, symbols, callGraph, srcFiles = [] }) {
  const root = mkdtempSync(
    path.join(tmpdir(), "vitest-check-canon-cli-helper-"),
  );
  mkdirSync(path.join(root, "canonical"), { recursive: true });
  mkdirSync(path.join(root, "audit-output"), { recursive: true });
  mkdirSync(path.join(root, "src"), { recursive: true });
  if (canonical !== null) {
    writeFileSync(
      path.join(root, "canonical", "helper-registry.md"),
      canonical,
      "utf8",
    );
  }
  if (symbols !== undefined) {
    writeFileSync(
      path.join(root, "audit-output", "symbols.json"),
      typeof symbols === "string" ? symbols : JSON.stringify(symbols, null, 2),
      "utf8",
    );
  }
  if (callGraph !== undefined) {
    writeFileSync(
      path.join(root, "audit-output", "call-graph.json"),
      typeof callGraph === "string"
        ? callGraph
        : JSON.stringify(callGraph, null, 2),
      "utf8",
    );
  }
  for (const file of srcFiles) {
    writeFileSync(path.join(root, "src", file.name), file.content, "utf8");
  }
  cleanup.push(root);
  return root;
}

function makeTopologyFixture({ canonical, topology, triage }) {
  const root = mkdtempSync(
    path.join(tmpdir(), "vitest-check-canon-cli-topology-"),
  );
  mkdirSync(path.join(root, "canonical"), { recursive: true });
  mkdirSync(path.join(root, "audit-output"), { recursive: true });
  if (canonical !== null) {
    writeFileSync(
      path.join(root, "canonical", "topology.md"),
      canonical,
      "utf8",
    );
  }
  if (topology !== undefined) {
    writeFileSync(
      path.join(root, "audit-output", "topology.json"),
      typeof topology === "string"
        ? topology
        : JSON.stringify(topology, null, 2),
      "utf8",
    );
  }
  if (triage !== undefined) {
    writeFileSync(
      path.join(root, "audit-output", "triage.json"),
      typeof triage === "string" ? triage : JSON.stringify(triage, null, 2),
      "utf8",
    );
  }
  cleanup.push(root);
  return root;
}

function makeNamingFixture({
  canonical,
  srcFiles = [],
  pkg = { name: "vitest-check-canon-naming", type: "module" },
}) {
  const root = mkdtempSync(
    path.join(tmpdir(), "vitest-check-canon-cli-naming-"),
  );
  mkdirSync(path.join(root, "canonical"), { recursive: true });
  mkdirSync(path.join(root, "audit-output"), { recursive: true });
  mkdirSync(path.join(root, "src"), { recursive: true });
  writeFileSync(path.join(root, "package.json"), JSON.stringify(pkg), "utf8");
  if (canonical !== null) {
    writeFileSync(path.join(root, "canonical", "naming.md"), canonical, "utf8");
  }
  for (const file of srcFiles) {
    const fullPath = path.join(root, file.rel);
    mkdirSync(path.dirname(fullPath), { recursive: true });
    writeFileSync(fullPath, file.content, "utf8");
  }
  cleanup.push(root);
  return root;
}

const TYPE_HEADER =
  "| Name | Identity | Owner | Fan-in | Status | Tags |\n" +
  "|------|----------|-------|-------:|--------|------|\n";

const HELPER_HEADER =
  "| Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |\n" +
  "|------|----------|-------|-----------|-------:|--------|------|----------------------|\n";

const TOPO_CLEAN_CANON = [
  "## 1. Submodule inventory",
  "",
  "| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |",
  "|-----------|------:|----:|---------:|----------:|-----|--------|------|",
  "| `src` | 1 | 10 | 0 | 0 | — | leaf-submodule ✅ | |",
  "",
  "## 3. Cycles (SCCs)",
  "",
  "✅ No submodule-level cycles observed. Repo is acyclic at submodule granularity.",
  "",
].join("\n");

const TOPO_CLEAN_TOPOLOGY = {
  meta: { complete: true, generated: "2026-04-22T00:00:00Z" },
  nodes: { "src/a.ts": { loc: 10 } },
  sccs: [],
  crossSubmoduleEdges: [],
  largestFiles: [],
};

describe("check-canon CLI dispatch and exit policy", () => {
  it(
    "rejects missing or unknown source without stack traces",
    () => {
      const missing = runCli([]);
      expect(missing.exit).toBe(2);
      expect(missing.stderr).toMatch(/--source/);

      const unknown = runCli(["--source", "xyz-bogus"]);
      expect(unknown.exit).toBe(2);
      expect(unknown.stderr).toMatch(/unknown|unsupported|source/i);
    },
    CLI_TEST_TIMEOUT_MS,
  );

  it(
    "keeps type-ownership strict input and write-anyway contracts",
    () => {
      const missingSymbols = makeTypeFixture({
        canonical:
          "| Name | Identity | Owner | Fan-in | Status | Tags |\n|--|--|--|--:|--|--|\n",
        symbols: null,
      });
      const strictMissing = runCli([
        "--source",
        "type-ownership",
        "--root",
        missingSymbols,
        "--output",
        path.join(missingSymbols, "audit-output"),
      ]);
      expect(strictMissing.exit).toBe(2);
      expect(strictMissing.stderr).toMatch(/symbols\.json/i);

      const missingCanon = makeTypeFixture({
        canonical: null,
        symbols: basicSymbols(),
      });
      const skipped = runCli([
        "--source",
        "type-ownership",
        "--root",
        missingCanon,
        "--output",
        path.join(missingCanon, "audit-output"),
      ]);
      expect(skipped.exit).toBe(2);
      const skippedJson = JSON.parse(
        readFileSync(
          path.join(missingCanon, "audit-output", "canon-drift.json"),
          "utf8",
        ),
      );
      expect(skippedJson.perSource["type-ownership"].status).toBe(
        "skipped-missing-canon",
      );
      expect(
        existsSync(
          path.join(
            missingCanon,
            "audit-output",
            "canon-drift.type-ownership.md",
          ),
        ),
      ).toBe(false);

      const cleanRoot = makeTypeFixture({
        canonical:
          TYPE_HEADER +
          "| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:10` | 3 | single-owner-strong ✅ | |\n",
        symbols: basicSymbols({
          typeDefs: [
            { name: "Foo", ownerFile: "src/foo.ts", line: 10, fanIn: 3 },
          ],
        }),
      });
      const clean = runCli([
        "--source",
        "type-ownership",
        "--root",
        cleanRoot,
        "--output",
        path.join(cleanRoot, "audit-output"),
      ]);
      expect(clean.exit).toBe(0);
      expect(clean.stdout).toMatch(/clean/i);

      const driftRoot = makeTypeFixture({
        canonical:
          TYPE_HEADER +
          "| `Foo` | `src/foo.ts::Foo` | `src/foo.ts:10` | 3 | single-owner-strong ✅ | |\n",
        symbols: basicSymbols({ typeDefs: [] }),
      });
      const drift = runCli([
        "--source",
        "type-ownership",
        "--root",
        driftRoot,
        "--output",
        path.join(driftRoot, "audit-output"),
      ]);
      expect(drift.exit).toBe(1);
      expect(drift.stdout).toMatch(/drift/i);
      expect(
        existsSync(
          path.join(driftRoot, "audit-output", "canon-drift.type-ownership.md"),
        ),
      ).toBe(true);

      const corruptRoot = makeTypeFixture({
        canonical:
          "| Name | Identity | Owner | Fan-in | Status | Tags |\n|--|--|--|--:|--|--|\n",
        symbols: null,
      });
      writeFileSync(
        path.join(corruptRoot, "audit-output", "symbols.json"),
        "not json {{\n",
        "utf8",
      );
      const corrupt = runCli([
        "--source",
        "type-ownership",
        "--root",
        corruptRoot,
        "--output",
        path.join(corruptRoot, "audit-output"),
      ]);
      expect(corrupt.exit).toBe(2);
      expect(corrupt.stderr).toContain("[check-canon]");
      expect(corrupt.stderr).not.toContain("at JSON.parse");
    },
    CLI_TEST_TIMEOUT_MS,
  );

  it(
    "uses shape-index to upgrade only the grounded type owner-change pair",
    () => {
      const root = makeTypeFixture({
        canonical:
          TYPE_HEADER +
          "| `X` | `src/a.ts::X` | `src/a.ts:1` | 1 | single-owner-weak ✅ | |\n",
        symbols: basicSymbols({
          typeDefs: [
            { name: "X", ownerFile: "src/b.ts", line: 1, fanIn: 1 },
            { name: "X", ownerFile: "src/c.ts", line: 1, fanIn: 1 },
          ],
        }),
      });
      writeFileSync(
        path.join(root, "audit-output", "shape-index.json"),
        JSON.stringify(
          makeShapeIndex([
            { identity: "src/a.ts::X", hash: makeHash("a") },
            { identity: "src/b.ts::X", hash: makeHash("a") },
            { identity: "src/c.ts::X", hash: makeHash("b") },
          ]),
          null,
          2,
        ),
        "utf8",
      );
      const result = runCli([
        "--source",
        "type-ownership",
        "--root",
        root,
        "--output",
        path.join(root, "audit-output"),
      ]);
      const json = JSON.parse(
        readFileSync(
          path.join(root, "audit-output", "canon-drift.json"),
          "utf8",
        ),
      );
      expect(result.exit).toBe(1);
      expect(json.drifts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            category: "owner-changed",
            canon: expect.objectContaining({ identity: "src/a.ts::X" }),
            fresh: expect.objectContaining({ identity: "src/b.ts::X" }),
          }),
          expect.objectContaining({
            category: "identity-added",
            identity: "src/c.ts::X",
          }),
        ]),
      );
      expect(
        json.drifts.filter((drift) => drift.category === "identity-removed"),
      ).toHaveLength(0);
    },
    CLI_TEST_TIMEOUT_MS,
  );

  it(
    "keeps helper-registry enrichment non-strict but type-ownership strict",
    () => {
      const missingCanon = makeHelperFixture({
        canonical: null,
        srcFiles: [{ name: "foo.ts", content: "export function doFoo() {}\n" }],
      });
      const skipped = runCli([
        "--source",
        "helper-registry",
        "--root",
        missingCanon,
        "--output",
        path.join(missingCanon, "audit-output"),
      ]);
      expect(skipped.exit).toBe(2);
      const skippedJson = JSON.parse(
        readFileSync(
          path.join(missingCanon, "audit-output", "canon-drift.json"),
          "utf8",
        ),
      );
      expect(skippedJson.perSource["helper-registry"].status).toBe(
        "skipped-missing-canon",
      );

      const missingSymbols = makeHelperFixture({
        canonical:
          HELPER_HEADER +
          "| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:1` | () => void | 0 | zero-internal-fan-in-helper ⚠ | | |\n",
        srcFiles: [{ name: "foo.ts", content: "export function doFoo() {}\n" }],
      });
      const advisory = runCli([
        "--source",
        "helper-registry",
        "--root",
        missingSymbols,
        "--output",
        path.join(missingSymbols, "audit-output"),
      ]);
      expect([0, 1]).toContain(advisory.exit);
      const advisoryJson = JSON.parse(
        readFileSync(
          path.join(missingSymbols, "audit-output", "canon-drift.json"),
          "utf8",
        ),
      );
      expect(
        advisoryJson.perSource["helper-registry"].diagnostics.map(
          (d) => d.kind,
        ),
      ).toContain("helper-contamination-enrichment-unavailable");

      const corruptSymbols = makeHelperFixture({
        canonical:
          HELPER_HEADER +
          "| `doFoo` | `src/foo.ts::doFoo` | `src/foo.ts:1` | () => void | 0 | zero-internal-fan-in-helper ⚠ | | |\n",
        symbols: "not json {{",
        callGraph: "not json {{",
        srcFiles: [{ name: "foo.ts", content: "export function doFoo() {}\n" }],
      });
      const helper = runCli([
        "--source",
        "helper-registry",
        "--root",
        corruptSymbols,
        "--output",
        path.join(corruptSymbols, "audit-output"),
      ]);
      expect([0, 1]).toContain(helper.exit);

      writeFileSync(
        path.join(corruptSymbols, "canonical", "type-ownership.md"),
        TYPE_HEADER,
        "utf8",
      );
      const type = runCli([
        "--source",
        "type-ownership",
        "--root",
        corruptSymbols,
        "--output",
        path.join(corruptSymbols, "audit-output"),
      ]);
      expect(type.exit).toBe(2);
    },
    CLI_TEST_TIMEOUT_MS,
  );

  it(
    "keeps topology source dispatch strict while stale warnings remain non-fatal",
    () => {
      const cleanRoot = makeTopologyFixture({
        canonical: TOPO_CLEAN_CANON,
        topology: TOPO_CLEAN_TOPOLOGY,
      });
      const clean = runCli([
        "--source",
        "topology",
        "--root",
        cleanRoot,
        "--output",
        path.join(cleanRoot, "audit-output"),
      ]);
      expect(clean.exit).toBe(0);
      expect(clean.stdout).toMatch(/clean/i);

      const driftRoot = makeTopologyFixture({
        canonical: TOPO_CLEAN_CANON,
        topology: {
          ...TOPO_CLEAN_TOPOLOGY,
          nodes: { "src/a.ts": { loc: 10 }, "lib/b.ts": { loc: 20 } },
        },
      });
      const drift = runCli([
        "--source",
        "topology",
        "--root",
        driftRoot,
        "--output",
        path.join(driftRoot, "audit-output"),
      ]);
      expect(drift.exit).toBe(1);
      expect(
        existsSync(
          path.join(driftRoot, "audit-output", "canon-drift.topology.md"),
        ),
      ).toBe(true);

      const missingTopology = makeTopologyFixture({
        canonical: TOPO_CLEAN_CANON,
        topology: undefined,
      });
      const missing = runCli([
        "--source",
        "topology",
        "--root",
        missingTopology,
        "--output",
        path.join(missingTopology, "audit-output"),
      ]);
      expect(missing.exit).toBe(2);
      expect(missing.stderr).toMatch(/topology\.json/i);

      const corruptTopology = makeTopologyFixture({
        canonical: TOPO_CLEAN_CANON,
        topology: "not json {{",
      });
      const corrupt = runCli([
        "--source",
        "topology",
        "--root",
        corruptTopology,
        "--output",
        path.join(corruptTopology, "audit-output"),
      ]);
      expect(corrupt.exit).toBe(2);
      expect(corrupt.stderr).toContain("[check-canon]");
      expect(corrupt.stderr).not.toContain("at JSON.parse");

      const staleRoot = makeTopologyFixture({
        canonical: TOPO_CLEAN_CANON,
        topology: TOPO_CLEAN_TOPOLOGY,
      });
      mkdirSync(path.join(staleRoot, "src"), { recursive: true });
      writeFileSync(
        path.join(staleRoot, "src", "newer.ts"),
        "export const X = 1;\n",
      );
      const topologyPath = path.join(
        staleRoot,
        "audit-output",
        "topology.json",
      );
      const oneHourAgo = Date.now() / 1000 - 3600;
      utimesSync(topologyPath, oneHourAgo, oneHourAgo);
      const stale = runCli([
        "--source",
        "topology",
        "--root",
        staleRoot,
        "--output",
        path.join(staleRoot, "audit-output"),
      ]);
      expect(stale.stderr).toMatch(/stale|older|refresh/i);
      expect(stale.stderr).toMatch(/topology\.json/i);
      expect([0, 1]).toContain(stale.exit);
    },
    CLI_TEST_TIMEOUT_MS,
  );

  it(
    "keeps naming and all-source aggregation exit semantics",
    () => {
      const namingClean = makeNamingFixture({
        canonical: [
          "## 1. File-naming cohorts",
          "",
          "| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |",
          "|--------------------|------:|--------------------|----------------:|--------------:|--------|",
          "",
          "## 2. Symbol-naming cohorts",
          "",
          "| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |",
          "|--------------------------|------:|--------------------|----------------:|--------------:|--------|",
          "",
        ].join("\n"),
      });
      const clean = runCli([
        "--source",
        "naming",
        "--root",
        namingClean,
        "--output",
        path.join(namingClean, "audit-output"),
      ]);
      expect(clean.exit).toBe(0);

      const namingMissing = makeNamingFixture({ canonical: null });
      const missing = runCli([
        "--source",
        "naming",
        "--root",
        namingMissing,
        "--output",
        path.join(namingMissing, "audit-output"),
      ]);
      expect(missing.exit).toBe(2);
      const missingJson = JSON.parse(
        readFileSync(
          path.join(namingMissing, "audit-output", "canon-drift.json"),
          "utf8",
        ),
      );
      expect(missingJson.perSource.naming.status).toBe("skipped-missing-canon");

      const allMissing = mkdtempSync(
        path.join(tmpdir(), "vitest-check-canon-all-"),
      );
      cleanup.push(allMissing);
      mkdirSync(path.join(allMissing, "audit-output"), { recursive: true });
      writeFileSync(
        path.join(allMissing, "package.json"),
        JSON.stringify({ name: "all-missing", type: "module" }),
        "utf8",
      );
      writeFileSync(
        path.join(allMissing, "audit-output", "symbols.json"),
        JSON.stringify({
          meta: { scope: "fix" },
          defIndex: {},
          fanInByIdentity: {},
        }),
        "utf8",
      );
      writeFileSync(
        path.join(allMissing, "audit-output", "topology.json"),
        JSON.stringify({
          meta: {},
          nodes: {},
          sccs: [],
          crossSubmoduleEdges: [],
          largestFiles: [],
        }),
        "utf8",
      );
      const all = runCli([
        "--source",
        "all",
        "--root",
        allMissing,
        "--output",
        path.join(allMissing, "audit-output"),
      ]);
      expect(all.exit).toBe(2);
      const allJson = JSON.parse(
        readFileSync(
          path.join(allMissing, "audit-output", "canon-drift.json"),
          "utf8",
        ),
      );
      expect(Object.keys(allJson.perSource).sort()).toEqual([
        "helper-registry",
        "naming",
        "topology",
        "type-ownership",
      ]);

      const oneClean = mkdtempSync(
        path.join(tmpdir(), "vitest-check-canon-all-clean-"),
      );
      cleanup.push(oneClean);
      mkdirSync(path.join(oneClean, "audit-output"), { recursive: true });
      mkdirSync(path.join(oneClean, "canonical"), { recursive: true });
      writeFileSync(
        path.join(oneClean, "package.json"),
        JSON.stringify({ name: "all-one-clean", type: "module" }),
        "utf8",
      );
      writeFileSync(
        path.join(oneClean, "canonical", "type-ownership.md"),
        TYPE_HEADER,
        "utf8",
      );
      writeFileSync(
        path.join(oneClean, "audit-output", "symbols.json"),
        JSON.stringify({
          meta: { scope: "fix" },
          defIndex: {},
          fanInByIdentity: {},
        }),
        "utf8",
      );
      writeFileSync(
        path.join(oneClean, "audit-output", "topology.json"),
        JSON.stringify({
          meta: {},
          nodes: {},
          sccs: [],
          crossSubmoduleEdges: [],
          largestFiles: [],
        }),
        "utf8",
      );
      expect(
        runCli([
          "--source",
          "all",
          "--root",
          oneClean,
          "--output",
          path.join(oneClean, "audit-output"),
        ]).exit,
      ).toBe(0);
    },
    CLI_TEST_TIMEOUT_MS,
  );
});

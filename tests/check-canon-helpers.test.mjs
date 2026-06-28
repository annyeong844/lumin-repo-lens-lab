import { describe, expect, it } from "vitest";

import { detectHelperRegistryDrift } from "../_lib/check-canon-helpers.mjs";
import { HELPER_LABEL_SET } from "../_lib/check-canon-utils.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function withFixture(testFn) {
  const fixture = createTempRepoFixture({ prefix: "fx-vitest-check-canon-" });
  try {
    return testFn(fixture);
  } finally {
    fixture.cleanup();
  }
}

function writeCanon(fixture, relPath, rows) {
  let md =
    "| Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |\n";
  md +=
    "|------|----------|-------|-----------|-------:|--------|------|----------------------|\n";
  for (const row of rows) {
    md += `| \`${row.name}\` | \`${row.identity}\` | \`${row.owner}\` | ${
      row.signature ?? ""
    } | ${row.fanIn} | ${row.label} ✅ | | |\n`;
  }
  fixture.write(relPath, md);
  return fixture.path(relPath);
}

function makeExtractStub(byFile, failingPaths = new Set()) {
  return (absFile) => {
    if (failingPaths.has(absFile)) {
      throw new Error(`stub extractor forced-throw on ${absFile}`);
    }
    return byFile.get(absFile) ?? { defs: [], uses: [], reExports: [] };
  };
}

function makeResolveSpecifier(specToPath) {
  return (_fromFile, spec) => specToPath.get(spec) ?? null;
}

function buildScanContext({
  root,
  files,
  defs,
  uses = new Map(),
  spec = new Map(),
  symbols = null,
  callGraph = null,
  failingPaths = new Set(),
}) {
  const defsByFile = new Map();
  for (const [file, fileDefs] of defs) {
    defsByFile.set(file, {
      defs: fileDefs,
      uses: uses.get(file) ?? [],
      reExports: [],
    });
  }
  return {
    files,
    root,
    extractFn: makeExtractStub(defsByFile, failingPaths),
    resolveSpecifier: makeResolveSpecifier(spec),
    symbols,
    callGraph,
  };
}

function fixtureDefs(entries) {
  const defs = new Map();
  const files = [];
  for (const entry of entries) {
    if (!defs.has(entry.absFile)) {
      defs.set(entry.absFile, []);
      files.push(entry.absFile);
    }
    defs.get(entry.absFile).push({
      name: entry.name,
      kind: entry.kind ?? "FunctionDeclaration",
      line: entry.line ?? 1,
    });
  }
  return { defs, files };
}

function detect({ fixture, canonPath, scanContext }) {
  return detectHelperRegistryDrift({
    canonPath,
    scanContext,
    canonLabelSet: HELPER_LABEL_SET,
  });
}

describe("helper-registry drift engine", () => {
  it("H-1. missing canon skips without fabricating drift", () =>
    withFixture((fixture) => {
      const result = detect({
        fixture,
        canonPath: fixture.path("missing.md"),
        scanContext: buildScanContext({
          root: fixture.root,
          files: [],
          defs: new Map(),
        }),
      });

      expect(result).toMatchObject({
        status: "skipped-missing-canon",
        drifts: [],
        reportMarkdown: null,
      });
    }));

  it("H-2/H-3. added and removed helpers stay separate without helper-owner-changed upgrade", () =>
    withFixture((fixture) => {
      const canonPath = writeCanon(fixture, "canon.md", [
        {
          name: "doFoo",
          identity: "src/a.ts::doFoo",
          owner: "src/a.ts:10",
          fanIn: 3,
          label: "central-helper",
        },
      ]);
      const bAbs = fixture.path("src/b.ts");
      const { defs, files } = fixtureDefs([
        { absFile: bAbs, name: "doFoo", line: 1 },
      ]);
      const result = detect({
        fixture,
        canonPath,
        scanContext: buildScanContext({
          root: fixture.root,
          files,
          defs,
        }),
      });

      expect(
        result.drifts.filter((d) => d.category === "helper-added"),
      ).toHaveLength(1);
      expect(
        result.drifts.filter((d) => d.category === "helper-removed"),
      ).toHaveLength(1);
      expect(
        result.drifts.some((d) => d.category === "helper-owner-changed"),
      ).toBe(false);
    }));

  it("H-4. contamination-changed requires per-identity enrichment evidence", () =>
    withFixture((fixture) => {
      const canonPath = writeCanon(fixture, "canon.md", [
        {
          name: "doX",
          identity: "src/x.ts::doX",
          owner: "src/x.ts:1",
          fanIn: 3,
          label: "severely-any-contaminated-helper",
        },
      ]);
      const xAbs = fixture.path("src/x.ts");
      const consumers = ["src/c1.ts", "src/c2.ts", "src/c3.ts"].map((rel) =>
        fixture.path(rel),
      );
      const { defs } = fixtureDefs([{ absFile: xAbs, name: "doX", line: 1 }]);
      for (const consumer of consumers) defs.set(consumer, []);
      const uses = new Map(
        consumers.map((consumer) => [
          consumer,
          [{ fromSpec: "./x", name: "doX", kind: "import" }],
        ]),
      );
      const baseContext = {
        root: fixture.root,
        files: [xAbs, ...consumers],
        defs,
        uses,
        spec: new Map([["./x", xAbs]]),
      };

      const available = detect({
        fixture,
        canonPath,
        scanContext: buildScanContext({
          ...baseContext,
          symbols: {
            helperOwnersByIdentity: {
              "src/x.ts::doX": {
                anyContamination: null,
              },
            },
          },
        }),
      });
      const unavailable = detect({
        fixture,
        canonPath,
        scanContext: buildScanContext({
          ...baseContext,
          symbols: null,
        }),
      });

      expect(
        available.drifts.filter((d) => d.category === "contamination-changed"),
      ).toHaveLength(1);
      expect(
        unavailable.drifts.filter(
          (d) => d.category === "contamination-changed",
        ),
      ).toHaveLength(0);
      expect(
        unavailable.drifts.filter((d) => d.category === "label-changed"),
      ).toHaveLength(1);
      expect(unavailable.diagnostics).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            kind: "helper-contamination-enrichment-unavailable",
          }),
        ]),
      );
    }));

  it("H-4c/H-7. fan-in-tier-changed is not gated by callGraph availability", () =>
    withFixture((fixture) => {
      const canonPath = writeCanon(fixture, "canon.md", [
        {
          name: "doZ",
          identity: "src/z.ts::doZ",
          owner: "src/z.ts:1",
          fanIn: 2,
          label: "shared-helper",
        },
      ]);
      const zAbs = fixture.path("src/z.ts");
      const consumers = ["src/e1.ts", "src/e2.ts", "src/e3.ts"].map((rel) =>
        fixture.path(rel),
      );
      const { defs } = fixtureDefs([{ absFile: zAbs, name: "doZ", line: 1 }]);
      for (const consumer of consumers) defs.set(consumer, []);
      const uses = new Map(
        consumers.map((consumer) => [
          consumer,
          [{ fromSpec: "./z", name: "doZ", kind: "import" }],
        ]),
      );
      const result = detect({
        fixture,
        canonPath,
        scanContext: buildScanContext({
          root: fixture.root,
          files: [zAbs, ...consumers],
          defs,
          uses,
          spec: new Map([["./z", zAbs]]),
          callGraph: null,
        }),
      });

      expect(
        result.drifts.filter((d) => d.category === "fan-in-tier-changed"),
      ).toHaveLength(1);
    }));

  it("H-5. extractor throws promote to parse-error diagnostics, not empty drift", () =>
    withFixture((fixture) => {
      const canonPath = writeCanon(fixture, "canon.md", [
        {
          name: "doFoo",
          identity: "src/foo.ts::doFoo",
          owner: "src/foo.ts:1",
          fanIn: 1,
          label: "shared-helper",
        },
      ]);
      const badAbs = fixture.path("src/bad.ts");
      const result = detect({
        fixture,
        canonPath,
        scanContext: buildScanContext({
          root: fixture.root,
          files: [badAbs],
          defs: new Map([[badAbs, []]]),
          failingPaths: new Set([badAbs]),
        }),
      });

      expect(result).toMatchObject({
        status: "parse-error",
        drifts: [],
        reportMarkdown: null,
      });
      expect(result.diagnostics).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            kind: "parse-error",
            target: "src/bad.ts",
          }),
        ]),
      );
    }));

  it("H-8/H-10/H-12. clean and contamination reports keep expected markdown and kind shape", () =>
    withFixture((fixture) => {
      const cleanCanon = writeCanon(fixture, "clean.md", [
        {
          name: "doClean",
          identity: "src/clean.ts::doClean",
          owner: "src/clean.ts:1",
          fanIn: 1,
          label: "shared-helper",
        },
      ]);
      const cleanAbs = fixture.path("src/clean.ts");
      const { defs, files } = fixtureDefs([
        { absFile: cleanAbs, name: "doClean", line: 1 },
      ]);
      const cleanConsumer = fixture.path("src/clean-consumer.ts");
      defs.set(cleanConsumer, []);
      const clean = detect({
        fixture,
        canonPath: cleanCanon,
        scanContext: buildScanContext({
          root: fixture.root,
          files: [...files, cleanConsumer],
          defs,
          uses: new Map([
            [
              cleanConsumer,
              [{ fromSpec: "./clean", name: "doClean", kind: "import" }],
            ],
          ]),
          spec: new Map([["./clean", cleanAbs]]),
        }),
      });

      expect(clean.status).toBe("clean");
      expect(clean.drifts).toHaveLength(0);
      expect(clean.reportMarkdown).not.toContain("###");

      const contaminatedCanon = writeCanon(fixture, "contaminated.md", [
        {
          name: "doBad",
          identity: "src/bad.ts::doBad",
          owner: "src/bad.ts:1",
          fanIn: 3,
          label: "severely-any-contaminated-helper",
        },
      ]);
      const badAbs = fixture.path("src/bad.ts");
      const consumers = ["src/b1.ts", "src/b2.ts", "src/b3.ts"].map((rel) =>
        fixture.path(rel),
      );
      const { defs: badDefs } = fixtureDefs([
        { absFile: badAbs, name: "doBad", line: 1 },
      ]);
      for (const consumer of consumers) badDefs.set(consumer, []);
      const badUses = new Map(
        consumers.map((consumer) => [
          consumer,
          [{ fromSpec: "./bad", name: "doBad", kind: "import" }],
        ]),
      );
      const contamination = detect({
        fixture,
        canonPath: contaminatedCanon,
        scanContext: buildScanContext({
          root: fixture.root,
          files: [badAbs, ...consumers],
          defs: badDefs,
          uses: badUses,
          spec: new Map([["./bad", badAbs]]),
          symbols: {
            helperOwnersByIdentity: {
              "src/bad.ts::doBad": {
                anyContamination: null,
              },
            },
          },
        }),
      });

      expect(contamination.drifts.every((d) => d.kind === "helper-drift")).toBe(
        true,
      );
      expect(contamination.reportMarkdown).toContain("Canon signal");
      expect(contamination.reportMarkdown).toContain("Fresh signal");
      expect(contamination.reportMarkdown).not.toContain("Canon fan-in");
    }));

  it("H-9. HELPER_LABEL_SET stays the canonical nine-label set", () => {
    expect([...HELPER_LABEL_SET].sort()).toEqual(
      [
        "ANY_COLLISION_HELPER",
        "HELPER_DUPLICATE_REVIEW",
        "HELPER_DUPLICATE_STRONG",
        "HELPER_LOCAL_COMMON",
        "central-helper",
        "low-signal-helper-name",
        "severely-any-contaminated-helper",
        "shared-helper",
        "zero-internal-fan-in-helper",
      ].sort(),
    );
  });
});

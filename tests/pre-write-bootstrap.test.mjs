import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

const ROOT = path.resolve(import.meta.dirname, "..");
const NODE = process.execPath;

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

async function importModule(relPath) {
  return import(`file://${path.join(ROOT, relPath).replace(/\\/g, "/")}`);
}

function runSymbolsOnFixture(fixtureRoot, output) {
  execFileSync(
    NODE,
    [
      path.join(ROOT, "build-symbol-graph.mjs"),
      "--root",
      fixtureRoot,
      "--output",
      output,
    ],
    { stdio: ["ignore", "pipe", "pipe"] },
  );
  return JSON.parse(readFileSync(path.join(output, "symbols.json"), "utf8"));
}

function hasConformingAnnotation(annotation) {
  return (
    annotation &&
    typeof annotation === "object" &&
    typeof annotation.label === "string" &&
    Array.isArray(annotation.labels) &&
    annotation.measurements &&
    typeof annotation.measurements === "object"
  );
}

describe("pre-write bootstrap prerequisites", () => {
  it("keeps required pre-write dependency modules importable with named exports", async () => {
    const requiredExports = {
      "_lib/cli.mjs": ["parseCliArgs"],
      "_lib/artifacts.mjs": ["loadIfExists"],
      "_lib/resolver-core.mjs": ["makeResolver"],
      "_lib/alias-map.mjs": ["extractStringTarget", "mapOutputToSource"],
      "_lib/finding-provenance.mjs": ["specifierCouldMatchFile"],
      "_lib/classify-facts.mjs": ["countFileReferencesAst"],
      "_lib/test-paths.mjs": ["isTestLikePath"],
      "_lib/vocab.mjs": ["EVIDENCE", "TAINT"],
      "_lib/pre-write-canonical-parser.mjs": [
        "parseCanonicalFile",
        "findCanonicalOwnerClaim",
      ],
    };

    for (const [relPath, exports] of Object.entries(requiredExports)) {
      expect(existsSync(path.join(ROOT, relPath)), relPath).toBe(true);
      const mod = await importModule(relPath);
      for (const exportName of exports) {
        expect(mod[exportName], `${relPath} exports ${exportName}`).not.toBe(
          undefined,
        );
      }
    }
  });

  it("emits support flags and conforming any-contamination owner facts", () => {
    const fixtureRoot = mkdtempSync(path.join(tmpdir(), "p1-0-bootstrap-"));
    const output = mkdtempSync(path.join(tmpdir(), "p1-0-bootstrap-out-"));
    try {
      write(
        fixtureRoot,
        "package.json",
        JSON.stringify({ name: "fx", type: "module" }),
      );
      write(
        fixtureRoot,
        "src/a.ts",
        "export const formatDate = (d) => d.toString();\n",
      );
      write(
        fixtureRoot,
        "src/dirty.ts",
        "export interface DirtyType { payload: any }\n" +
          "export function parsePayload(payload: any) { return payload as any; }\n",
      );
      write(
        fixtureRoot,
        "src/jsdoc.mjs",
        "/** @type {any} */\nexport const fromJsdoc = readValue();\n",
      );
      write(
        fixtureRoot,
        "src/b.ts",
        "import { formatDate } from './a';\nexport const useFmt = () => formatDate(new Date());\n",
      );

      const symbols = runSymbolsOnFixture(fixtureRoot, output);
      expect(symbols.meta?.supports).toBeTruthy();
      expect(symbols.meta.schemaVersion).toBeGreaterThanOrEqual(3);
      expect(symbols.meta.supports.anyContamination).toBe(true);

      expect(
        hasConformingAnnotation(
          symbols.defIndex?.["src/dirty.ts"]?.DirtyType?.anyContamination,
        ),
      ).toBe(true);
      expect(
        hasConformingAnnotation(
          symbols.defIndex?.["src/dirty.ts"]?.parsePayload?.anyContamination,
        ),
      ).toBe(true);
      expect(
        symbols.defIndex?.["src/a.ts"]?.formatDate?.anyContamination,
      ).toBeUndefined();
      expect(
        symbols.helperOwnersByIdentity?.["src/a.ts::formatDate"]
          ?.anyContamination,
      ).toBeNull();
      expect(
        hasConformingAnnotation(
          symbols.helperOwnersByIdentity?.["src/dirty.ts::parsePayload"]
            ?.anyContamination,
        ),
      ).toBe(true);
      expect(
        hasConformingAnnotation(
          symbols.typeOwnersByIdentity?.["src/dirty.ts::DirtyType"]
            ?.anyContamination,
        ),
      ).toBe(true);
      expect(
        hasConformingAnnotation(
          symbols.helperOwnersByIdentity?.["src/jsdoc.mjs::fromJsdoc"]
            ?.anyContamination,
        ),
      ).toBe(true);
    } finally {
      rmSync(fixtureRoot, { recursive: true, force: true });
      rmSync(output, { recursive: true, force: true });
    }
  });

  it("rejects legacy flat any-contamination shape and exposes identity fan-in", () => {
    const fixtureRoot = mkdtempSync(path.join(tmpdir(), "p1-0-fanin-"));
    const output = mkdtempSync(path.join(tmpdir(), "p1-0-fanin-out-"));
    try {
      write(
        fixtureRoot,
        "package.json",
        JSON.stringify({ name: "fx", type: "module" }),
      );
      write(
        fixtureRoot,
        "src/a.ts",
        "export const formatDate = (d) => d.toString();\n",
      );
      write(
        fixtureRoot,
        "src/b.ts",
        "import { formatDate } from './a';\nexport const useFmt = () => formatDate(new Date());\n",
      );

      const flatLegacy = {
        label: "any-contaminated",
        anyFieldRatio: 0.5,
        totalFields: 2,
        anyFields: 1,
      };
      expect(hasConformingAnnotation(flatLegacy)).toBe(false);

      const symbols = runSymbolsOnFixture(fixtureRoot, output);
      expect(symbols.meta.supports.identityFanIn).toBe(true);
      expect(symbols.fanInByIdentity).toEqual(expect.any(Object));
      expect(symbols.fanInByIdentity["src/a.ts::formatDate"]).toBe(1);
      expect(["symbol-level", "file-level", "absent"]).toContain(
        symbols.meta.supports.reExportRecords,
      );
    } finally {
      rmSync(fixtureRoot, { recursive: true, force: true });
      rmSync(output, { recursive: true, force: true });
    }
  });

  it("keeps FP_BUDGET declared as zero for downstream exit criteria", () => {
    const corpusPath = path.join(ROOT, "tests", "test-corpus.mjs");
    expect(existsSync(corpusPath)).toBe(true);

    const corpusText = readFileSync(corpusPath, "utf8");
    const match = corpusText.match(/FP_BUDGET\s*=\s*(\d+)/);
    expect(match).not.toBeNull();
    expect(Number(match?.[1])).toBe(0);
  });
});

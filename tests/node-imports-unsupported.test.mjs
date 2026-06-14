import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { buildAliasMap } from "../_lib/alias-map.mjs";
import { detectRepoMode } from "../_lib/repo-mode.mjs";
import { makeResolver } from "../_lib/resolver-core.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

function runProducer(fixture, producer) {
  execFileSync(
    process.execPath,
    [
      path.join(ROOT, producer),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
    ],
    { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] },
  );
}

function buildResolverArtifacts(fixture) {
  runProducer(fixture, "build-symbol-graph.mjs");
  runProducer(fixture, "build-resolver-diagnostics.mjs");
  return {
    symbols: fixture.readJson("symbols.json", { from: "output" }),
    diagnostics: fixture.readJson("resolver-diagnostics.json", {
      from: "output",
    }),
  };
}

function resolveFromFixture(fixture, importer, specifier) {
  const aliasMap = buildAliasMap(fixture.root, detectRepoMode(fixture.root));
  const resolve = makeResolver(fixture.root, aliasMap);
  return resolve(fixture.path(importer), specifier);
}

describe("Node #imports unsupported-family diagnostics", () => {
  it("keeps package-local #imports without a supported imports map diagnostic-only", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-node-imports-unsupported-",
      packageJson: {
        name: "fx-node-imports-unsupported",
        type: "module",
      },
    });

    try {
      fixture.write(
        "src/app.ts",
        "import { config } from '#app/config';\n" +
          "export function boot() { return config; }\n",
      );

      const { symbols, diagnostics } = buildResolverArtifacts(fixture);
      const direct = resolveFromFixture(fixture, "src/app.ts", "#app/config");
      const record = symbols.unresolvedInternalSpecifierRecords?.find(
        (item) => item.specifier === "#app/config",
      );
      const unresolvedImport = diagnostics.unresolvedImports?.find(
        (item) => item.specifier === "#app/config",
      );
      const unsupportedImport = diagnostics.unsupportedImports?.find(
        (item) => item.specifier === "#app/config",
      );
      const blindZone = diagnostics.blindZones?.find(
        (item) => item.specifier === "#app/config",
      );
      const graphEdge = symbols.resolvedInternalEdges?.find(
        (edge) => edge.source === "#app/config",
      );

      expect(direct).toBe("UNRESOLVED_INTERNAL");
      expect(record).toMatchObject({
        reason: "hash-imports-unsupported",
        resolverStage: "hash-imports",
        outputLevel: "unsupported",
        unsupportedFamily: "node-imports",
      });
      expect(graphEdge).toBeUndefined();
      expect(symbols.uses).toMatchObject({
        resolvedInternal: 0,
        unresolvedInternal: 1,
      });
      expect(unresolvedImport).toMatchObject({
        family: "node-imports",
        reason: "hash-imports-unsupported",
        outputLevel: "unsupported",
        createsGraphEdge: false,
      });
      expect(diagnostics.summary).toMatchObject({
        unsupportedImportCount: 1,
      });
      expect(unsupportedImport).toMatchObject({
        family: "node-imports",
        reason: "hash-imports-unsupported",
        outputLevel: "unsupported",
      });
      expect(blindZone).toMatchObject({
        family: "node-imports",
        outputLevel: "unsupported",
        blocksAbsenceClaims: true,
        blockingScope: "repo-confidence-limited",
      });
      expect(blindZone.targetCandidates).toBeUndefined();
    } finally {
      fixture.cleanup();
    }
  });

  it("keeps unsupported condition-profile #imports maps diagnostic-only with candidates", () => {
    const fixture = createTempRepoFixture({
      prefix: "fx-vitest-node-imports-condition-ambiguous-",
      packageJson: {
        name: "fx-node-imports-condition-ambiguous",
        type: "module",
        imports: {
          "#env": {
            browser: "./src/browser.ts",
            "react-native": "./src/native.ts",
          },
        },
      },
    });

    try {
      fixture.write("src/browser.ts", 'export const env = "browser";\n');
      fixture.write("src/native.ts", 'export const env = "native";\n');
      fixture.write(
        "src/app.ts",
        "import { env } from '#env';\n" +
          "export function boot() { return env; }\n",
      );

      const { symbols, diagnostics } = buildResolverArtifacts(fixture);
      const direct = resolveFromFixture(fixture, "src/app.ts", "#env");
      const record = symbols.unresolvedInternalSpecifierRecords?.find(
        (item) => item.specifier === "#env",
      );
      const unsupportedImport = diagnostics.unsupportedImports?.find(
        (item) => item.specifier === "#env",
      );
      const blindZone = diagnostics.blindZones?.find(
        (item) => item.specifier === "#env",
      );
      const graphEdge = symbols.resolvedInternalEdges?.find(
        (edge) => edge.source === "#env",
      );

      expect(direct).toBe("UNRESOLVED_INTERNAL");
      expect(record).toMatchObject({
        reason: "condition-profile-ambiguous",
        resolverStage: "hash-imports",
        outputLevel: "unsupported",
        unsupportedFamily: "node-imports",
      });
      expect(record.targetCandidates).toEqual(
        expect.arrayContaining(["src/browser.ts", "src/native.ts"]),
      );
      expect(graphEdge).toBeUndefined();
      expect(symbols.uses).toMatchObject({
        resolvedInternal: 0,
        unresolvedInternal: 1,
      });
      expect(diagnostics.summary).toMatchObject({
        unsupportedImportCount: 1,
      });
      expect(unsupportedImport).toMatchObject({
        family: "node-imports",
        outputLevel: "unsupported",
        reason: "condition-profile-ambiguous",
      });
      expect(unsupportedImport.targetCandidates).toEqual(
        expect.arrayContaining(["src/browser.ts", "src/native.ts"]),
      );
      expect(blindZone).toMatchObject({
        family: "node-imports",
        reason: "condition-profile-ambiguous",
        outputLevel: "unsupported",
        blocksAbsenceClaims: true,
        blockingScope: "candidate-relevant",
      });
      expect(blindZone.targetCandidates).toEqual(
        expect.arrayContaining(["src/browser.ts", "src/native.ts"]),
      );
    } finally {
      fixture.cleanup();
    }
  });
});

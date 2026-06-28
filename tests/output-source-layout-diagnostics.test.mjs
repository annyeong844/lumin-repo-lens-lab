import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

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

function resolveFromFixture(fixture, importer, specifier) {
  const aliasMap = buildAliasMap(fixture.root, detectRepoMode(fixture.root));
  const resolve = makeResolver(fixture.root, aliasMap);
  return resolve(fixture.path(importer), specifier);
}

function buildOutputSourceLayoutFixture() {
  const fixture = createTempRepoFixture({
    prefix: "fx-vitest-output-source-layout-",
    packageJson: {
      name: "fx-output-source-layout",
      type: "module",
      workspaces: ["apps/*", "packages/*"],
    },
  });

  fixture.writeJson("apps/web/package.json", {
    name: "@fixture/web",
    type: "module",
    dependencies: {
      "@fixture/weird": "workspace:*",
    },
  });
  fixture.writeJson("packages/weird/package.json", {
    name: "@fixture/weird",
    type: "module",
    exports: {
      "./*": "./compiled/*.js",
    },
  });
  fixture.write(
    "apps/web/src/app.ts",
    "import { value } from '@fixture/weird/foo';\n" +
      "export const appValue = value;\n",
  );
  fixture.write(
    "packages/weird/main/foo.ts",
    "export const value = 1;\n" + "export const sibling = 2;\n",
  );

  runProducer(fixture, "build-symbol-graph.mjs");
  runProducer(fixture, "build-resolver-diagnostics.mjs");

  return {
    fixture,
    direct: resolveFromFixture(
      fixture,
      "apps/web/src/app.ts",
      "@fixture/weird/foo",
    ),
    symbols: fixture.readJson("symbols.json", { from: "output" }),
    diagnostics: fixture.readJson("resolver-diagnostics.json", {
      from: "output",
    }),
  };
}

describe("output-to-source layout unsupported diagnostics", () => {
  let fixture;
  let direct;
  let symbols;
  let diagnostics;

  beforeAll(() => {
    ({ fixture, direct, symbols, diagnostics } =
      buildOutputSourceLayoutFixture());
  });

  afterAll(() => {
    fixture?.cleanup();
  });

  it("keeps unsupported package output/source layouts from faking resolved edges", () => {
    const graphEdge = symbols.resolvedInternalEdges?.find(
      (edge) => edge.source === "@fixture/weird/foo",
    );

    expect(direct).toBe("UNRESOLVED_INTERNAL");
    expect(graphEdge).toBeUndefined();
  });

  it("records the unsupported output-to-source mapping in symbols", () => {
    const record = symbols.unresolvedInternalSpecifierRecords?.find(
      (item) => item.specifier === "@fixture/weird/foo",
    );

    expect(record).toMatchObject({
      reason: "output-source-layout-unsupported",
      resolverStage: "wildcard-alias",
      outputLevel: "unsupported",
      unsupportedFamily: "output-to-source-mapping",
      source: "exports",
    });
    expect(record?.targetCandidates).toContain(
      "packages/weird/compiled/foo.js",
    );
  });

  it("surfaces the unsupported output layout through resolver diagnostics", () => {
    const unsupportedImport = diagnostics.unsupportedImports?.find(
      (item) => item.specifier === "@fixture/weird/foo",
    );

    expect(diagnostics.summary).toMatchObject({
      unsupportedImportCount: 1,
    });
    expect(unsupportedImport).toMatchObject({
      family: "output-to-source-mapping",
      outputLevel: "unsupported",
      reason: "output-source-layout-unsupported",
    });
  });

  it("keeps the output layout blind zone candidate-scoped", () => {
    const blindZone = diagnostics.blindZones?.find(
      (item) => item.specifier === "@fixture/weird/foo",
    );

    expect(blindZone).toMatchObject({
      family: "output-to-source-mapping",
      outputLevel: "unsupported",
      blocksAbsenceClaims: true,
      blockingScope: "candidate-relevant",
      affectedPackageScope: "packages/weird",
    });
  });

  it("points blocked candidate hints at the affected package surface", () => {
    const blockedHint = diagnostics.blockedCandidateHints?.find(
      (item) => item.specifier === "@fixture/weird/foo",
    );

    expect(blockedHint).toMatchObject({
      family: "output-to-source-mapping",
      reason: "output-source-layout-unsupported",
      affectedPackageScope: "packages/weird",
      candidatePath: "packages/weird/compiled/foo.js",
    });
  });
});

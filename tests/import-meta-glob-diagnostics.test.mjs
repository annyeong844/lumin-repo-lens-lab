import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

function runProducer(fixture, producer, extraArgs = []) {
  execFileSync(
    process.execPath,
    [
      path.join(ROOT, producer),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
      ...extraArgs,
    ],
    { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] },
  );
}

function buildImportMetaGlobFixture() {
  const fixture = createTempRepoFixture({
    prefix: "fx-vitest-import-meta-glob-",
    packageJson: {
      name: "fx-import-meta-glob",
      type: "module",
    },
  });

  fixture.write(
    "src/app.ts",
    "const routes = import.meta.glob('./routes/*.ts');\n" +
      "const missing = import.meta.glob('./missing/*.ts');\n" +
      "const pattern = './routes/*.ts';\n" +
      "const dynamicRoutes = import.meta.glob(pattern);\n" +
      "const many = import.meta.glob('./many/*.ts');\n" +
      "export function routeCount() { return Object.keys(routes).length; }\n",
  );
  fixture.write("src/routes/home.ts", "export const home = true;\n");
  fixture.write("src/routes/about.ts", "export const about = true;\n");
  fixture.write("src/routes/hidden.ts", "export const hidden = true;\n");
  for (let i = 0; i < 65; i++) {
    fixture.write(`src/many/route-${i}.ts`, `export const route${i} = true;\n`);
  }

  runProducer(fixture, "build-symbol-graph.mjs", [
    "--exclude",
    "src/routes/hidden.ts",
  ]);
  runProducer(fixture, "build-resolver-diagnostics.mjs");

  return {
    fixture,
    symbols: fixture.readJson("symbols.json", { from: "output" }),
    diagnostics: fixture.readJson("resolver-diagnostics.json", {
      from: "output",
    }),
  };
}

describe("import.meta.glob dynamic-module diagnostics", () => {
  let fixture;
  let symbols;
  let diagnostics;

  beforeAll(() => {
    ({ fixture, symbols, diagnostics } = buildImportMetaGlobFixture());
  });

  afterAll(() => {
    fixture?.cleanup();
  });

  it("expands supported literal globs into concrete dynamic graph edges", () => {
    const routeEdges = symbols.resolvedInternalEdges
      ?.filter((edge) => edge.source === "./routes/*.ts")
      .map((edge) => ({ to: edge.to, kind: edge.kind }))
      .sort((a, b) => a.to.localeCompare(b.to));
    const supportedRecord = symbols.unresolvedInternalSpecifierRecords?.find(
      (item) => item.specifier === "./routes/*.ts",
    );

    expect(routeEdges).toEqual([
      { to: "src/routes/about.ts", kind: "dynamic-import-meta-glob" },
      { to: "src/routes/home.ts", kind: "dynamic-import-meta-glob" },
    ]);
    expect(supportedRecord).toBeUndefined();
  });

  it("honors the scanned file set instead of broad filesystem globbing", () => {
    const hiddenEdge = symbols.resolvedInternalEdges?.find(
      (edge) => edge.to === "src/routes/hidden.ts",
    );

    expect(hiddenEdge).toBeUndefined();
  });

  it("marks supported glob targets as broad consumers instead of true dead exports", () => {
    const routeFanInSpace = {
      about: symbols.fanInByIdentitySpace?.["src/routes/about.ts::about"],
      home: symbols.fanInByIdentitySpace?.["src/routes/home.ts::home"],
    };
    const trulyDeadRoute = symbols.deadProdList?.find(
      (entry) =>
        entry.file === "src/routes/about.ts" ||
        entry.file === "src/routes/home.ts",
    );

    expect(routeFanInSpace.about).toMatchObject({ broad: 1 });
    expect(routeFanInSpace.home).toMatchObject({ broad: 1 });
    expect(trulyDeadRoute).toBeUndefined();
  });

  it("keeps zero-match globs as unsupported dynamic-module diagnostics", () => {
    const record = symbols.unresolvedInternalSpecifierRecords?.find(
      (item) => item.specifier === "./missing/*.ts",
    );

    expect(record).toMatchObject({
      reason: "import-meta-glob-zero-matches",
      resolverStage: "import-meta-glob",
      outputLevel: "unsupported",
      unsupportedFamily: "dynamic-modules",
      matchCount: 0,
      affectedPackageScope: "src/missing",
    });
  });

  it("keeps non-literal globs unsupported without creating graph edges", () => {
    const record = symbols.unresolvedInternalSpecifierRecords?.find(
      (item) => item.specifier === "import.meta.glob(<nonliteral>)",
    );

    expect(record).toMatchObject({
      reason: "import-meta-glob-nonliteral-unsupported",
      outputLevel: "unsupported",
    });
  });

  it("caps large glob expansions without producing partial edges", () => {
    const record = symbols.unresolvedInternalSpecifierRecords?.find(
      (item) => item.specifier === "./many/*.ts",
    );
    const manyEdges = symbols.resolvedInternalEdges?.filter(
      (edge) => edge.source === "./many/*.ts",
    );

    expect(record).toMatchObject({
      reason: "import-meta-glob-match-cap-exceeded",
      matchCount: 65,
      cap: 64,
    });
    expect(manyEdges).toEqual([]);
  });

  it("surfaces only unsupported import.meta.glob shapes through resolver diagnostics", () => {
    const unsupportedImports = Object.fromEntries(
      (diagnostics.unsupportedImports ?? []).map((item) => [item.specifier, item]),
    );

    expect(diagnostics.summary).toMatchObject({
      unsupportedImportCount: 3,
    });
    expect(unsupportedImports["./routes/*.ts"]).toBeUndefined();
    expect(unsupportedImports["./missing/*.ts"]).toMatchObject({
      family: "dynamic-modules",
      reason: "import-meta-glob-zero-matches",
      outputLevel: "unsupported",
    });
    expect(unsupportedImports["import.meta.glob(<nonliteral>)"]).toMatchObject({
      family: "dynamic-modules",
      reason: "import-meta-glob-nonliteral-unsupported",
      outputLevel: "unsupported",
    });
    expect(unsupportedImports["./many/*.ts"]).toMatchObject({
      family: "dynamic-modules",
      reason: "import-meta-glob-match-cap-exceeded",
      outputLevel: "unsupported",
    });
  });

  it("keeps unsupported glob blind zones scoped where scope is known", () => {
    const blindZones = Object.fromEntries(
      (diagnostics.blindZones ?? []).map((item) => [item.specifier, item]),
    );

    expect(blindZones["./missing/*.ts"]).toMatchObject({
      family: "dynamic-modules",
      reason: "import-meta-glob-zero-matches",
      outputLevel: "unsupported",
      blocksAbsenceClaims: true,
      blockingScope: "candidate-relevant",
      affectedPackageScope: "src/missing",
    });
    expect(blindZones["./many/*.ts"]).toMatchObject({
      affectedPackageScope: "src/many",
    });
  });
});

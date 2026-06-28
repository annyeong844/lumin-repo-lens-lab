import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import {
  buildAliasMap,
  mapOutputPatternToSource,
  mapOutputPatternToSourceCandidates,
} from "../_lib/alias-map.mjs";
import { detectRepoMode } from "../_lib/repo-mode.mjs";
import { makeResolver } from "../_lib/resolver-core.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

function createHashImportFixture({ prefix = "fx-vitest-hash-imports-", packageJson, files }) {
  const fixture = createTempRepoFixture({
    prefix,
    packageJson,
  });

  for (const [relPath, content] of Object.entries(files)) {
    fixture.write(relPath, content);
  }

  return fixture;
}

function resolveInFixture(fixture, consumer, specifier) {
  const mode = detectRepoMode(fixture.root);
  const aliasMap = buildAliasMap(fixture.root, mode);
  const resolve = makeResolver(fixture.root, aliasMap);
  return resolve(fixture.path(consumer), specifier);
}

function runSymbolGraph(fixture) {
  execFileSync(
    process.execPath,
    [
      path.join(ROOT, "build-symbol-graph.mjs"),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
    ],
    { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] },
  );
}

function expectResolvedScenario(scenario) {
  const fixture = createHashImportFixture(scenario);

  try {
    expect(resolveInFixture(fixture, scenario.consumer, scenario.specifier)).toBe(
      fixture.path(scenario.expected),
    );
  } finally {
    fixture.cleanup();
  }
}

describe("Node #imports exact output-to-source mapping", () => {
  it("maps exact runtime output entries to supported authored source files", () => {
    const scenarios = [
      {
        packageJson: {
          name: "hash-exact-mjs",
          type: "module",
          imports: { "#entry": "./dist/entry.mjs" },
        },
        files: {
          "src/entry.ts": "export const E = 1;\n",
          "src/consumer.ts": "import { E } from '#entry'; export const c = E;\n",
        },
        consumer: "src/consumer.ts",
        specifier: "#entry",
        expected: "src/entry.ts",
      },
      {
        packageJson: {
          name: "hash-exact-cjs",
          type: "module",
          imports: { "#util": "./dist/util.cjs" },
        },
        files: {
          "src/util.ts": "export const U = 1;\n",
          "src/consumer.ts": "import { U } from '#util'; export const c = U;\n",
        },
        consumer: "src/consumer.ts",
        specifier: "#util",
        expected: "src/util.ts",
      },
      {
        packageJson: {
          name: "hash-exact-js",
          type: "module",
          imports: { "#legacy": "./dist/legacy.js" },
        },
        files: {
          "src/legacy.ts": "export const L = 1;\n",
          "src/consumer.ts": "import { L } from '#legacy'; export const c = L;\n",
        },
        consumer: "src/consumer.ts",
        specifier: "#legacy",
        expected: "src/legacy.ts",
      },
      {
        packageJson: {
          name: "hash-exact-jsx",
          type: "module",
          imports: { "#button": "./dist/ui/Button.jsx" },
        },
        files: {
          "src/ui/Button.tsx": "export const Button = () => null;\n",
          "src/consumer.tsx": "import { Button } from '#button'; export const c = Button;\n",
        },
        consumer: "src/consumer.tsx",
        specifier: "#button",
        expected: "src/ui/Button.tsx",
      },
      {
        packageJson: {
          name: "hash-exact-lib",
          type: "module",
          imports: { "#helpers": "./dist/helpers.js" },
        },
        files: {
          "lib/helpers.ts": "export const H = 1;\n",
          "lib/consumer.ts": "import { H } from '#helpers'; export const c = H;\n",
        },
        consumer: "lib/consumer.ts",
        specifier: "#helpers",
        expected: "lib/helpers.ts",
      },
    ];

    for (const scenario of scenarios) {
      expectResolvedScenario(scenario);
    }
  });
});

describe("Node #imports wildcard output-to-source mapping", () => {
  it("maps wildcard runtime output entries and keeps missing targets unresolved internal", () => {
    const fixture = createHashImportFixture({
      packageJson: {
        name: "hash-wild-mjs",
        type: "module",
        imports: { "#feat/*": "./dist/features/*.mjs" },
      },
      files: {
        "src/features/alpha.ts": "export const A = 1;\n",
        "src/features/beta.ts": "export const B = 1;\n",
        "src/consumer.ts": "import { A } from '#feat/alpha'; export const c = A;\n",
      },
    });

    try {
      expect(resolveInFixture(fixture, "src/consumer.ts", "#feat/alpha")).toBe(
        fixture.path("src/features/alpha.ts"),
      );
      expect(resolveInFixture(fixture, "src/consumer.ts", "#feat/beta")).toBe(
        fixture.path("src/features/beta.ts"),
      );
      expect(resolveInFixture(fixture, "src/consumer.ts", "#feat/alpha.js")).toBe(
        fixture.path("src/features/alpha.ts"),
      );
      expect(resolveInFixture(fixture, "src/consumer.ts", "#feat/gamma")).toBe(
        "UNRESOLVED_INTERNAL",
      );
    } finally {
      fixture.cleanup();
    }
  });

  it("maps wildcard jsx output patterns to authored tsx source files", () => {
    expectResolvedScenario({
      packageJson: {
        name: "hash-wild-jsx",
        type: "module",
        imports: { "#ui/*": "./dist/ui/*.jsx" },
      },
      files: {
        "src/ui/Button.tsx": "export const B = 1;\n",
        "src/consumer.ts": "import { B } from '#ui/Button'; export const c = B;\n",
      },
      consumer: "src/consumer.ts",
      specifier: "#ui/Button",
      expected: "src/ui/Button.tsx",
    });
  });
});

describe("Node #imports suffix wildcard graph protection", () => {
  it("protects type and value exports reached through a suffix wildcard import", () => {
    const fixture = createHashImportFixture({
      packageJson: {
        name: "hash-wild-suffix-js",
        type: "module",
        imports: { "#web/request/*.js": "./src/adapter/web/request/*.ts" },
      },
      files: {
        "src/adapter/web/request/project-scope.ts":
          "export interface ProjectScopeRegistry { ok: boolean; }\n" +
          "export function readProjectScope(): ProjectScopeRegistry { return { ok: true }; }\n",
        "src/consumer.ts":
          "import { readProjectScope, type ProjectScopeRegistry } from '#web/request/project-scope.js';\n" +
          "export const value = readProjectScope();\n" +
          "export type ConsumerScope = ProjectScopeRegistry;\n",
      },
    });

    try {
      expect(resolveInFixture(fixture, "src/consumer.ts", "#web/request/project-scope.js")).toBe(
        fixture.path("src/adapter/web/request/project-scope.ts"),
      );

      runSymbolGraph(fixture);

      const symbols = fixture.readJson("symbols.json", { from: "output" });
      const dead = new Set(
        (symbols.deadProdList ?? []).map((item) => `${item.file}::${item.symbol}`),
      );

      expect(dead.has("src/adapter/web/request/project-scope.ts::ProjectScopeRegistry")).toBe(
        false,
      );
      expect(dead.has("src/adapter/web/request/project-scope.ts::readProjectScope")).toBe(
        false,
      );
    } finally {
      fixture.cleanup();
    }
  });
});

describe("Node #imports output pattern helper contract", () => {
  it("keeps output pattern rewrites and candidate ordering stable", () => {
    expect(mapOutputPatternToSource("./dist/*.mjs")).toBe("src/*.ts");
    expect(mapOutputPatternToSource("./dist/*.cjs")).toBe("src/*.ts");
    expect(mapOutputPatternToSource("./dist/*.js")).toBe("src/*.ts");
    expect(mapOutputPatternToSource("./dist/*.jsx")).toBe("src/*.tsx");
    expect(mapOutputPatternToSource("./lib/*.mjs")).toBe("lib/*.ts");
    expect(mapOutputPatternToSource("./esm/features/*.mjs")).toBe(
      "src/features/*.ts",
    );
    expect(mapOutputPatternToSource("./pattern.mjs")).toBe("pattern.ts");
    expect(mapOutputPatternToSourceCandidates("./dist/features/*.js").slice(0, 3)).toEqual([
      "src/features/*.js",
      "src/features/*.ts",
      "src/features/*.tsx",
    ]);
  });
});

describe("Node #imports authored JS and directory targets", () => {
  it("preserves authored JS source targets", () => {
    expectResolvedScenario({
      packageJson: {
        name: "hash-wild-js-source",
        type: "module",
        imports: { "#internal/*": "./src/internal/*.js" },
      },
      files: {
        "src/internal/util.js": "export const used = 1;\n",
        "src/consumer.js": "import { used } from '#internal/util'; export const c = used;\n",
      },
      consumer: "src/consumer.js",
      specifier: "#internal/util",
      expected: "src/internal/util.js",
    });
  });

  it("resolves wildcard directory targets to index files and protects exports", () => {
    const fixture = createHashImportFixture({
      packageJson: {
        name: "hash-wild-dir-index",
        type: "module",
        imports: { "#internal/*": "./src/internal/*" },
      },
      files: {
        "src/internal/util/index.js": "export const used = 1;\n",
        "src/consumer.js": "import { used } from '#internal/util'; export const c = used;\n",
      },
    });

    try {
      expect(resolveInFixture(fixture, "src/consumer.js", "#internal/util")).toBe(
        fixture.path("src/internal/util/index.js"),
      );

      runSymbolGraph(fixture);

      const symbols = fixture.readJson("symbols.json", { from: "output" });
      const dead = new Set(
        (symbols.deadProdList ?? []).map((item) => `${item.file}::${item.symbol}`),
      );

      expect(dead.has("src/internal/util/index.js::used")).toBe(false);
    } finally {
      fixture.cleanup();
    }
  });
});

describe("Node #imports malformed workspace package resilience", () => {
  it("skips one malformed workspace package without poisoning valid packages", () => {
    const fixture = createHashImportFixture({
      packageJson: {
        name: "root",
        private: true,
        workspaces: ["pkgs/good", "pkgs/bad"],
      },
      files: {
        "pkgs/good/package.json": JSON.stringify({
          name: "@m/good",
          exports: { ".": "./src/index.ts" },
        }),
        "pkgs/good/src/index.ts": "export const ok = 1;\n",
        "pkgs/bad/package.json": "{ this is not valid JSON - could be a half-written edit }",
      },
    });

    try {
      let aliasMap;

      expect(() => {
        aliasMap = buildAliasMap(fixture.root, detectRepoMode(fixture.root));
      }).not.toThrow();

      fixture.write("pkgs/good/src/consumer.ts", "export const x = 1;\n");
      const resolve = makeResolver(fixture.root, aliasMap);

      expect(resolve(fixture.path("pkgs/good/src/consumer.ts"), "@m/good")).toBe(
        fixture.path("pkgs/good/src/index.ts"),
      );
    } finally {
      fixture.cleanup();
    }
  });
});

import path from "node:path";

import { describe, expect, it } from "vitest";

import { buildAliasMap } from "../_lib/alias-map.mjs";
import { detectRepoMode } from "../_lib/repo-mode.mjs";
import { makeResolver } from "../_lib/resolver-core.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function createWildcardFixture({ prefix, packageJson, files }) {
  const fixture = createTempRepoFixture({ prefix, packageJson });
  for (const [relPath, content] of Object.entries(files)) {
    fixture.write(relPath, content);
  }
  return fixture;
}

function resolveInFixture(fixture, fromRelFile, specifier) {
  const mode = detectRepoMode(fixture.root);
  const aliasMap = buildAliasMap(fixture.root, mode);
  const resolve = makeResolver(fixture.root, aliasMap);
  return resolve(fixture.path(fromRelFile), specifier);
}

function withWildcardFixture(options, fn) {
  const fixture = createWildcardFixture(options);
  try {
    return fn(fixture);
  } finally {
    fixture.cleanup();
  }
}

describe("package exports wildcard subpath resolution", () => {
  it('resolves simple "./*" wildcard package subpaths to source files', () => {
    withWildcardFixture(
      {
        prefix: "fx-vitest-wildcard-star-",
        packageJson: {
          name: "@scope/a",
          type: "module",
          exports: { "./*": "./src/*.ts" },
        },
        files: {
          "src/leaf.ts": "export const L = 1;\n",
          "src/consumer.ts":
            "import { L } from '@scope/a/leaf'; export const x = L;\n",
        },
      },
      (fixture) => {
        expect(
          resolveInFixture(fixture, "src/consumer.ts", "@scope/a/leaf"),
        ).toBe(fixture.path("src/leaf.ts"));
      },
    );
  });

  it('resolves nested "./features/*" wildcard subpaths', () => {
    withWildcardFixture(
      {
        prefix: "fx-vitest-wildcard-subpath-",
        packageJson: {
          name: "@scope/b",
          type: "module",
          exports: { "./features/*": "./src/features/*.ts" },
        },
        files: {
          "src/features/alpha.ts": "export const ALPHA = 1;\n",
          "src/consumer.ts":
            "import { ALPHA } from '@scope/b/features/alpha'; export const x = ALPHA;\n",
        },
      },
      (fixture) => {
        expect(
          resolveInFixture(
            fixture,
            "src/consumer.ts",
            "@scope/b/features/alpha",
          ),
        ).toBe(fixture.path("src/features/alpha.ts"));
      },
    );
  });

  it("prefers the deepest matching wildcard prefix when multiple exports match", () => {
    withWildcardFixture(
      {
        prefix: "fx-vitest-wildcard-specificity-",
        packageJson: {
          name: "@scope/c",
          type: "module",
          exports: {
            "./*": "./src/*.ts",
            "./features/*": "./src/features/*.ts",
          },
        },
        files: {
          "src/root-thing.ts": "export const R = 1;\n",
          "src/features/specific.ts": "export const S = 2;\n",
          "src/consumer.ts": "export const x = 0;\n",
        },
      },
      (fixture) => {
        expect(
          resolveInFixture(fixture, "src/consumer.ts", "@scope/c/root-thing"),
        ).toBe(fixture.path("src/root-thing.ts"));
        expect(
          resolveInFixture(
            fixture,
            "src/consumer.ts",
            "@scope/c/features/specific",
          ),
        ).toBe(fixture.path("src/features/specific.ts"));
      },
    );
  });

  it("maps dist wildcard targets back to authored source files", () => {
    withWildcardFixture(
      {
        prefix: "fx-vitest-wildcard-dist-",
        packageJson: {
          name: "@scope/d",
          type: "module",
          exports: { "./*": "./dist/*.js" },
        },
        files: {
          "src/worker.ts": "export const W = 1;\n",
          "src/consumer.ts": "export const x = 0;\n",
        },
      },
      (fixture) => {
        expect(
          resolveInFixture(fixture, "src/consumer.ts", "@scope/d/worker"),
        ).toBe(fixture.path("src/worker.ts"));
      },
    );
  });

  it("resolves deeply nested wildcard export subpaths", () => {
    withWildcardFixture(
      {
        prefix: "fx-vitest-wildcard-nested-",
        packageJson: {
          name: "@scope/e",
          type: "module",
          exports: { "./ui/components/*": "./src/ui/components/*.ts" },
        },
        files: {
          "src/ui/components/button.ts": "export const B = 1;\n",
          "src/consumer.ts": "export const x = 0;\n",
        },
      },
      (fixture) => {
        expect(
          resolveInFixture(
            fixture,
            "src/consumer.ts",
            "@scope/e/ui/components/button",
          ),
        ).toBe(fixture.path("src/ui/components/button.ts"));
      },
    );
  });

  it("keeps unmatched packages external and matched missing targets unresolved internal", () => {
    withWildcardFixture(
      {
        prefix: "fx-vitest-wildcard-negative-",
        packageJson: {
          name: "@scope/f",
          type: "module",
          exports: { "./features/*": "./src/features/*.ts" },
        },
        files: {
          "src/features/a.ts": "export const A = 1;\n",
          "src/consumer.ts": "export const x = 0;\n",
        },
      },
      (fixture) => {
        expect(resolveInFixture(fixture, "src/consumer.ts", "lodash")).toBe(
          "EXTERNAL",
        );
        expect(
          resolveInFixture(fixture, "src/consumer.ts", "@other/pkg/features/x"),
        ).toBe("EXTERNAL");
        expect(
          resolveInFixture(
            fixture,
            "src/consumer.ts",
            "@scope/f/features/missing",
          ),
        ).toBe("UNRESOLVED_INTERNAL");
      },
    );
  });

  it("keeps exact package exports working beside wildcard support", () => {
    withWildcardFixture(
      {
        prefix: "fx-vitest-wildcard-exact-",
        packageJson: {
          name: "@scope/g",
          type: "module",
          exports: {
            ".": "./src/index.ts",
            "./specific": "./src/specific.ts",
          },
        },
        files: {
          "src/index.ts": "export const I = 1;\n",
          "src/specific.ts": "export const S = 1;\n",
          "src/consumer.ts": "export const x = 0;\n",
        },
      },
      (fixture) => {
        expect(resolveInFixture(fixture, "src/consumer.ts", "@scope/g")).toBe(
          fixture.path("src/index.ts"),
        );
        expect(
          resolveInFixture(fixture, "src/consumer.ts", "@scope/g/specific"),
        ).toBe(fixture.path("src/specific.ts"));
      },
    );
  });
});

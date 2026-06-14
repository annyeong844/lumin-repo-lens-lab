import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { detectRepoMode } from "../_lib/repo-mode.mjs";
import {
  collectHtmlModuleEntrypointFiles,
  collectPackagePublicSurfaceFiles,
  collectScriptEntrypointFiles,
  collectScriptEntrypoints,
} from "../_lib/public-surface.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function write(root, relPath, content) {
  const full = path.join(root, relPath);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function withFixture(prefix, fn) {
  const fixture = createTempRepoFixture({ prefix });
  try {
    return fn(fixture);
  } finally {
    fixture.cleanup();
  }
}

function collect(root) {
  return collectPackagePublicSurfaceFiles({
    root,
    repoMode: detectRepoMode(root),
  });
}

function collectScripts(root) {
  return collectScriptEntrypointFiles({
    root,
    repoMode: detectRepoMode(root),
  });
}

function collectScriptSurface(root) {
  return collectScriptEntrypoints({
    root,
    repoMode: detectRepoMode(root),
  });
}

function collectHtml(root) {
  return collectHtmlModuleEntrypointFiles({
    root,
    repoMode: detectRepoMode(root),
  });
}

describe("public surface collection", () => {
  it("PS-1. collects package root, bare main/types/bin, and direct declaration surfaces", () => {
    withFixture("vitest-public-surface-root-", (fixture) => {
      fixture.write("pnpm-workspace.yaml", "packages:\n  - examples/*\n");
      fixture.writeJson("package.json", {
        name: "root-public",
        type: "module",
        exports: {
          ".": {
            types: "./dist/index.d.ts",
            default: "./dist/index.js",
          },
          "./types": {
            types: "./types/index.d.ts",
          },
        },
        main: "./dist/index.cjs",
        module: "./dist/index.js",
        types: "./dist/index.d.ts",
        bin: { "root-public": "./dist/cli.js" },
      });
      fixture.write("src/index.ts", "export const publicValue = 1;\n");
      fixture.write("src/cli.ts", "export const cli = 1;\n");
      fixture.write("types/index.d.ts", "export interface PublicOptions {}\n");
      fixture.writeJson("examples/app/package.json", { name: "example-app" });

      const files = new Set(collect(fixture.root).map((entry) => entry.file));

      expect(files).toContain("src/index.ts");
      expect(files).toContain("types/index.d.ts");
      expect(files).toContain("src/cli.ts");
    });

    withFixture("vitest-public-surface-bare-fields-", (fixture) => {
      fixture.writeJson("package.json", {
        name: "bare-field-targets",
        type: "module",
        main: "server.js",
        types: "types/index.d.ts",
        bin: { "bare-field-targets": "bin/cli.js" },
      });
      fixture.write("server.js", "export const server = 1;\n");
      fixture.write("bin/cli.js", "export const cli = 1;\n");
      fixture.write("types/index.d.ts", "export interface PublicTypes {}\n");

      const files = new Set(collect(fixture.root).map((entry) => entry.file));

      expect(files).toContain("server.js");
      expect(files).toContain("bin/cli.js");
      expect(files).toContain("types/index.d.ts");
    });
  });

  it("PS-2. collects condition, dist/source, and wildcard export surfaces with evidence", () => {
    withFixture("vitest-public-surface-conditions-", (fixture) => {
      fixture.writeJson("package.json", {
        name: "all-conditions",
        type: "module",
        exports: {
          ".": {
            import: {
              types: "./dist/import.d.ts",
              default: "./dist/import.js",
            },
            require: {
              types: "./dist/require.d.cts",
              default: "./dist/require.cjs",
            },
          },
        },
      });
      fixture.write("src/import.ts", "export const importPublic = 1;\n");
      fixture.write("src/require.ts", "export const requirePublic = 1;\n");

      const entries = collect(fixture.root);
      const files = new Set(entries.map((entry) => entry.file));
      const importEvidence =
        entries.find((entry) => entry.file === "src/import.ts")?.evidence ?? {};
      const requireEvidence =
        entries.find((entry) => entry.file === "src/require.ts")?.evidence ??
        {};

      expect(files).toContain("src/import.ts");
      expect(files).toContain("src/require.ts");
      expect(importEvidence.conditionPath).toBe("import.types");
      expect(requireEvidence.conditionPath).toBe("require.types");
    });

    withFixture("vitest-public-surface-dist-source-", (fixture) => {
      fixture.writeJson("package.json", {
        name: "dist-source",
        type: "module",
        exports: {
          ".": {
            types: "./dist/index.d.ts",
            import: "./dist/index.js",
          },
        },
      });
      fixture.write("dist/index.js", "export const compiled = 1;\n");
      fixture.write(
        "dist/index.d.ts",
        "export declare const compiled: number;\n",
      );
      fixture.write("src/index.ts", "export const authored = 1;\n");

      const entries = collect(fixture.root);
      const files = new Set(entries.map((entry) => entry.file));
      const sourceEvidence =
        entries.find((entry) => entry.file === "src/index.ts")?.evidence ?? {};

      expect(files).toContain("src/index.ts");
      expect(files).not.toContain("dist/index.js");
      expect(files).not.toContain("dist/index.d.ts");
      expect(["./dist/index.d.ts", "./dist/index.js"]).toContain(
        sourceEvidence.target,
      );
    });

    withFixture("vitest-public-surface-wildcard-", (fixture) => {
      fixture.writeJson("package.json", {
        name: "wild-public",
        type: "module",
        exports: {
          "./features/*": "./src/features/*.ts",
        },
      });
      fixture.write("src/features/alpha.ts", "export const alpha = 1;\n");
      fixture.write("src/features/beta.ts", "export const beta = 1;\n");
      fixture.write("src/private.ts", "export const privateValue = 1;\n");

      const entries = collect(fixture.root);
      const files = new Set(entries.map((entry) => entry.file));
      const alphaEvidence =
        entries.find((entry) => entry.file === "src/features/alpha.ts")
          ?.evidence ?? {};

      expect(files).toContain("src/features/alpha.ts");
      expect(files).toContain("src/features/beta.ts");
      expect(files).not.toContain("src/private.ts");
      expect(alphaEvidence).toMatchObject({
        source: "package.exports",
        subpath: "./features/*",
        sourcePattern: "src/features/*.ts",
        wildcard: true,
      });
    });

    withFixture("vitest-public-surface-wildcard-js-", (fixture) => {
      fixture.writeJson("package.json", {
        name: "wild-public-js",
        type: "module",
        exports: {
          "./features/*": "./src/features/*.js",
        },
      });
      fixture.write("src/features/alpha.js", "export const alpha = 1;\n");
      fixture.write("src/features/beta.js", "export const beta = 1;\n");
      fixture.write("src/features/gamma.ts", "export const gamma = 1;\n");
      fixture.write("src/private.js", "export const privateValue = 1;\n");

      const entries = collect(fixture.root);
      const files = new Set(entries.map((entry) => entry.file));
      const alphaEvidence =
        entries.find((entry) => entry.file === "src/features/alpha.js")
          ?.evidence ?? {};

      expect(files).toContain("src/features/alpha.js");
      expect(files).toContain("src/features/beta.js");
      expect(files).toContain("src/features/gamma.ts");
      expect(files).not.toContain("src/private.js");
      expect(alphaEvidence).toMatchObject({
        source: "package.exports",
        subpath: "./features/*",
        sourcePattern: "src/features/*.js",
        wildcard: true,
      });
    });
  });

  it("PS-3. collects package script entrypoints without treating string mentions as commands", () => {
    withFixture("vitest-public-surface-scripts-", (fixture) => {
      fixture.writeJson("package.json", {
        name: "script-entrypoints",
        type: "module",
        scripts: {
          build: "rimraf dist && esno scripts/build.ts",
          bundle: "tsup src/direct.ts --format esm",
        },
      });
      fixture.write("src/direct.ts", "export const direct = 1;\n");
      fixture.write(
        "src/client/dev/react.ts",
        "export const useRegisterSW = () => null;\n",
      );
      fixture.write("src/not-entry.ts", "export const notEntry = 1;\n");
      fixture.write(
        "scripts/build.ts",
        [
          "export const commands = [",
          "  'npx tsup src/client/dev/react.ts --external react --target esnext',",
          "  'this mentions src/not-entry.ts but is not a tsup command',",
          "]",
          "",
        ].join("\n"),
      );

      const entries = collectScripts(fixture.root);
      const scriptSurface = collectScriptSurface(fixture.root);
      const files = new Set(entries.map((entry) => entry.file));
      const reactEvidence =
        entries.find((entry) => entry.file === "src/client/dev/react.ts")
          ?.evidence ?? {};

      expect(files).toContain("src/direct.ts");
      expect(files).toContain("src/client/dev/react.ts");
      expect(files).not.toContain("src/not-entry.ts");
      expect(
        scriptSurface.unsupported.some(
          (entry) =>
            entry.source === "script-string-literal" ||
            entry.targetCandidates?.includes("src/not-entry.ts"),
        ),
      ).toBe(false);
      expect(reactEvidence).toMatchObject({
        source: "script-string-literal",
        scriptFile: "scripts/build.ts",
        tool: "tsup",
      });
    });

    withFixture("vitest-public-surface-rollup-esbuild-", (fixture) => {
      fixture.writeJson("package.json", {
        name: "script-entrypoint-tools",
        type: "module",
        scripts: {
          rollupExplicit: "rollup --input src/explicit.ts --format esm",
          rollupDynamic: "rollup -c rollup.config.js --input",
          esbuildBundle:
            "esbuild --bundle ./src/esbuild-entry.ts --outfile=dist/out.js",
        },
      });
      fixture.write("src/explicit.ts", "export const explicit = 1;\n");
      fixture.write("src/esbuild-entry.ts", "export const esbuildEntry = 1;\n");
      fixture.write("zod-full.ts", "export const schema = 1;\n");
      fixture.write("rollup.config.js", "export default {};\n");
      fixture.write("src/internal.ts", "export const internal = 1;\n");

      const entries = collectScripts(fixture.root);
      const files = new Set(entries.map((entry) => entry.file));
      const dynamicEvidence =
        entries.find((entry) => entry.file === "zod-full.ts")?.evidence ?? {};

      expect(files).toContain("src/explicit.ts");
      expect(files).toContain("src/esbuild-entry.ts");
      expect(files).toContain("zod-full.ts");
      expect(files).not.toContain("rollup.config.js");
      expect(files).not.toContain("src/internal.ts");
      expect(dynamicEvidence).toMatchObject({
        tool: "rollup",
        dynamicInput: true,
        scriptName: "rollupDynamic",
      });
    });
  });

  it("PS-4. collects HTML module script entrypoints and ignores non-module scripts", () => {
    withFixture("vitest-public-surface-html-", (fixture) => {
      fixture.writeJson("package.json", {
        name: "html-entrypoints",
        type: "module",
      });
      fixture.write(
        "index.html",
        [
          '<div id="app"></div>',
          '<script type="module" src="/src/main.ts"></script>',
          '<script src="/src/legacy.ts"></script>',
          "",
        ].join("\n"),
      );
      fixture.write("src/main.ts", "export default {};\n");
      fixture.write("src/legacy.ts", "export default {};\n");

      const entries = collectHtml(fixture.root);
      const files = new Set(entries.map((entry) => entry.file));
      const evidence =
        entries.find((entry) => entry.file === "src/main.ts")?.evidence ?? {};

      expect(files).toContain("src/main.ts");
      expect(files).not.toContain("src/legacy.ts");
      expect(evidence).toMatchObject({
        source: "html-module-script",
        htmlFile: "index.html",
      });
    });
  });
});

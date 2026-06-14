import { execFileSync } from "node:child_process";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const ROOT = path.resolve(import.meta.dirname, "..");
const ENTRY_SURFACE_TEST_TIMEOUT_MS = 30_000;

function runScript(script, fixture, args = []) {
  execFileSync(
    process.execPath,
    [
      path.join(ROOT, script),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
      ...args,
    ],
    { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
  );
}

function writeFiles(fixture, files) {
  for (const [file, content] of Object.entries(files))
    fixture.write(file, content);
}

function createBaseEntrySurfaceFixture(prefix) {
  const fixture = createTempRepoFixture({
    prefix,
    packageJson: {
      name: "entry-surface-fixture",
      private: true,
      exports: { ".": "./src/index.ts" },
      scripts: { build: "tsup src/cli.ts" },
      dependencies: { next: "15.0.0" },
    },
  });

  writeFiles(fixture, {
    "index.html": '<script type="module" src="./src/browser.ts"></script>\n',
    "vite.config.ts": "export default { plugins: [] };\n",
    "src/index.ts": 'export { feature } from "./feature";\n',
    "src/feature.ts": "export const feature = 1;\n",
    "src/cli.ts": "export function cli() {}\n",
    "src/browser.ts": "export const browser = true;\n",
    "src/internal.ts": "export const internal = true;\n",
    "src/app/dashboard/page.tsx":
      "export default function Page() { return null; }\n",
    "cloudflare/worker/wrangler.toml": 'main = "src/index.js"\n',
    "cloudflare/worker/src/index.js":
      'export default { async fetch() { return new Response("ok"); } };\n',
  });

  return fixture;
}

function runEntrySurface(fixture, args = ["--production"]) {
  runScript("build-symbol-graph.mjs", fixture, args);
  runScript("build-entry-surface.mjs", fixture, args);
  return fixture.readJson("entry-surface.json", { from: "output" });
}

function runQuickAudit(fixture, args = ["--profile", "quick", "--production"]) {
  runScript("audit-repo.mjs", fixture, args);
  return {
    manifest: fixture.readJson("manifest.json", { from: "output" }),
    entrySurface: fixture.readJson("entry-surface.json", { from: "output" }),
  };
}

function asSet(value) {
  return new Set(value ?? []);
}

describe("entry-surface artifact", () => {
  it(
    "E1-E11. records public, script, HTML, framework, config, and completeness evidence",
    () => {
      const fixture = createBaseEntrySurfaceFixture("vitest-entry-surface-");
      try {
        const artifact = runEntrySurface(fixture);
        const publicApiFiles = asSet(artifact.publicApiFiles);
        const scriptEntrypointFiles = asSet(artifact.scriptEntrypointFiles);
        const htmlEntrypointFiles = asSet(artifact.htmlEntrypointFiles);
        const frameworkEntrypointFiles = asSet(
          artifact.frameworkEntrypointFiles,
        );
        const configEntrypointFiles = asSet(artifact.configEntrypointFiles);
        const entryFiles = asSet(artifact.entryFiles);

        expect(artifact.meta?.tool).toBe("build-entry-surface.mjs");
        expect(publicApiFiles.has("src/index.ts")).toBe(true);
        expect(publicApiFiles.has("src/feature.ts")).toBe(true);
        expect(scriptEntrypointFiles.has("src/cli.ts")).toBe(true);
        expect(htmlEntrypointFiles.has("src/browser.ts")).toBe(true);
        expect(frameworkEntrypointFiles.has("src/app/dashboard/page.tsx")).toBe(
          true,
        );
        expect(
          frameworkEntrypointFiles.has("cloudflare/worker/src/index.js"),
        ).toBe(true);
        expect(configEntrypointFiles.has("vite.config.ts")).toBe(true);
        expect(entryFiles.has("src/index.ts")).toBe(true);
        expect(entryFiles.has("src/feature.ts")).toBe(true);
        expect(entryFiles.has("src/cli.ts")).toBe(true);
        expect(entryFiles.has("src/browser.ts")).toBe(true);
        expect(entryFiles.has("src/app/dashboard/page.tsx")).toBe(true);
        expect(entryFiles.has("vite.config.ts")).toBe(true);
        expect(entryFiles.has("src/internal.ts")).toBe(false);
        expect(artifact.evidenceByFile?.["src/feature.ts"]).toEqual(
          expect.arrayContaining([
            expect.objectContaining({ source: "public-reexport" }),
          ]),
        );
        expect(artifact.globalCompleteness).toBe("high");
        expect(artifact.completenessBySubmodule?.root).toBe("high");
        expect(artifact.completenessBySubmodule?.src).toBe("high");
        expect(
          Object.values(artifact.completenessBySubmodule ?? {}).every(
            (value) => value === "high",
          ),
        ).toBe(true);
      } finally {
        fixture.cleanup();
      }
    },
    ENTRY_SURFACE_TEST_TIMEOUT_MS,
  );

  it(
    "E12-E14. quick audit runs build-entry-surface and keeps public API evidence",
    () => {
      const fixture = createBaseEntrySurfaceFixture("vitest-entry-audit-");
      try {
        const audit = runQuickAudit(fixture);

        expect(
          audit.manifest.commandsRun?.some(
            (step) =>
              step.step === "build-entry-surface.mjs" && step.status === "ok",
          ),
        ).toBe(true);
        expect(audit.manifest.artifactsProduced).toContain(
          "entry-surface.json",
        );
        expect(audit.entrySurface.publicApiFiles).toContain("src/feature.ts");
      } finally {
        fixture.cleanup();
      }
    },
    ENTRY_SURFACE_TEST_TIMEOUT_MS,
  );

  it(
    "E15-E20. missing static-server HTML roots become scoped blind-zone review evidence",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "vitest-entry-static-mismatch-",
        packageJson: {
          name: "entry-surface-static-root-mismatch",
          private: true,
          type: "module",
        },
      });
      try {
        writeFiles(fixture, {
          "index.html":
            '<script type="module" src="/assets/app.js"></script>\n',
          "server.ts": [
            'import path from "node:path";',
            'export const STATIC_ROOT = path.join(process.cwd(), "public");',
          ].join("\n"),
          "public/assets/app.js": [
            'import { boot } from "../../src/boot.js";',
            "export function createTerminalInputMessage(value) {",
            '  return { type: "input", value };',
            "}",
            "export function serializeTerminalInputMessage(value) {",
            "  return JSON.stringify(createTerminalInputMessage(value));",
            "}",
            "boot();",
            'serializeTerminalInputMessage("hello");',
          ].join("\n"),
          "src/boot.js": "export function boot() {}\n",
          "src/unused.js": "export const unused = true;\n",
        });

        runScript("audit-repo.mjs", fixture, [
          "--profile",
          "quick",
          "--production",
        ]);
        const entrySurface = fixture.readJson("entry-surface.json", {
          from: "output",
        });
        const reachability = fixture.readJson("module-reachability.json", {
          from: "output",
        });
        const manifest = fixture.readJson("manifest.json", { from: "output" });
        const fixPlan = fixture.readJson("fix-plan.json", { from: "output" });

        expect(entrySurface.htmlEntrypointFiles).not.toContain("assets/app.js");
        expect(reachability.reachableFiles).not.toContain("assets/app.js");
        expect(entrySurface.unresolvedHtmlEntrypoints).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              htmlFile: "index.html",
              src: "/assets/app.js",
              reason: "html-module-script-target-missing",
            }),
          ]),
        );
        expect(entrySurface.globalCompleteness).toBe("medium");
        expect(manifest.blindZones).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              area: "html-entry-surface",
              details: expect.objectContaining({
                unresolvedHtmlEntrypoints: 1,
              }),
            }),
          ]),
        );
        expect(
          fixPlan.safeFixes?.some(
            (score) => score.finding?.file === "public/assets/app.js",
          ),
        ).toBe(false);
        expect(
          fixPlan.reviewFixes?.filter(
            (score) =>
              score.finding?.file === "public/assets/app.js" &&
              score.reason === "html-entry-surface-blind-zone" &&
              score.blockedPromotion === true &&
              score.blockedBy?.[0]?.area === "html-entry-surface",
          ),
        ).toHaveLength(2);
      } finally {
        fixture.cleanup();
      }
    },
    ENTRY_SURFACE_TEST_TIMEOUT_MS,
  );

  it(
    "E21-E23. nested HTML app roots resolve against the HTML directory without phantom probes",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "vitest-entry-nested-html-",
        packageJson: {
          name: "entry-surface-nested-html-app",
          private: true,
          type: "module",
        },
      });
      try {
        writeFiles(fixture, {
          "apps/web/index.html":
            '<script type="module" src="/src/main.tsx"></script>\n',
          "apps/web/src/main.tsx": "export function mountApp() {}\n",
          "src/main.tsx": "export function wrongRoot() {}\n",
        });

        const entrySurface = runEntrySurface(fixture);

        expect(entrySurface.htmlEntrypointFiles).toContain(
          "apps/web/src/main.tsx",
        );
        expect(entrySurface.htmlEntrypointFiles).not.toContain("src/main.tsx");
        expect(entrySurface.htmlEntrypointFiles).not.toContain(
          "apps/web/src/main.jsx",
        );
        expect(entrySurface.entryFiles).not.toContain("apps/web/src/main.jsx");
        expect(entrySurface.evidenceByFile?.["apps/web/src/main.jsx"]).toBe(
          undefined,
        );
        expect(entrySurface.unresolvedHtmlEntrypoints ?? []).toHaveLength(0);
        expect(entrySurface.globalCompleteness).toBe("high");
      } finally {
        fixture.cleanup();
      }
    },
    ENTRY_SURFACE_TEST_TIMEOUT_MS,
  );

  it(
    "E24. excluded HTML files do not create unresolved entry-surface blind zones",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "vitest-entry-excluded-html-",
        packageJson: {
          name: "entry-surface-excluded-html",
          private: true,
          type: "module",
        },
      });
      try {
        writeFiles(fixture, {
          "src/main.ts": "export const main = 1;\n",
          "output/corpus/sample/index.html":
            '<script type="module" src="/src/missing.ts"></script>\n',
        });

        const entrySurface = runEntrySurface(fixture, [
          "--exclude",
          "output/corpus",
          "--production",
        ]);

        expect(entrySurface.unresolvedHtmlEntrypoints ?? []).toHaveLength(0);
        expect(entrySurface.globalCompleteness).toBe("high");
      } finally {
        fixture.cleanup();
      }
    },
    ENTRY_SURFACE_TEST_TIMEOUT_MS,
  );

  it(
    "E25. package runtime scripts seed module reachability",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "vitest-entry-runtime-script-",
        packageJson: {
          name: "entry-surface-runtime-script",
          private: true,
          type: "module",
          scripts: {
            start: "tsx src/server.ts",
          },
        },
      });
      try {
        writeFiles(fixture, {
          "src/server.ts": [
            'import { app } from "./app";',
            "app.listen();",
          ].join("\n"),
          "src/app.ts": "export const app = { listen() {} };\n",
          "src/isolated.ts": "export const isolated = true;\n",
        });

        runScript("build-symbol-graph.mjs", fixture, ["--production"]);
        runScript("build-entry-surface.mjs", fixture, ["--production"]);
        runScript("build-module-reachability.mjs", fixture, ["--production"]);

        const entrySurface = fixture.readJson("entry-surface.json", {
          from: "output",
        });
        const reachability = fixture.readJson("module-reachability.json", {
          from: "output",
        });

        expect(entrySurface.scriptEntrypointFiles).toContain("src/server.ts");
        expect(entrySurface.entryFiles).toContain("src/server.ts");
        expect(entrySurface.evidenceByFile?.["src/server.ts"]).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              source: "package.scripts",
              scriptName: "start",
              tool: "tsx",
              runtime: true,
            }),
          ]),
        );
        expect(reachability.runtimeReachableFiles).toContain("src/server.ts");
        expect(reachability.unreachableFiles).not.toContain("src/server.ts");
        expect(reachability.unreachableFiles).toContain("src/isolated.ts");
      } finally {
        fixture.cleanup();
      }
    },
    ENTRY_SURFACE_TEST_TIMEOUT_MS,
  );

  it(
    "E26. unknown script wrappers do not create runtime entry evidence",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "vitest-entry-unknown-script-wrapper-",
        packageJson: {
          name: "entry-surface-unknown-script-wrapper",
          private: true,
          type: "module",
          scripts: {
            start: "custom-runner src/server.ts",
          },
        },
      });
      try {
        writeFiles(fixture, {
          "src/server.ts": "export function listen() {}\n",
        });

        const entrySurface = runEntrySurface(fixture);

        expect(entrySurface.scriptEntrypointFiles).not.toContain(
          "src/server.ts",
        );
        expect(entrySurface.entryFiles).not.toContain("src/server.ts");
        expect(entrySurface.evidenceByFile?.["src/server.ts"]).toBeUndefined();
        expect(entrySurface.meta?.supports?.unsupportedScriptEntrypoints).toBe(
          true,
        );
        expect(entrySurface.unsupportedScriptEntrypointCount).toBe(1);
        expect(entrySurface.unsupportedScriptEntrypointSampleLimit).toBe(50);
        expect(entrySurface.unsupportedScriptEntrypoints).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              reason: "unknown-script-wrapper",
              scriptName: "start",
              tool: "custom-runner",
              targetCandidates: expect.arrayContaining(["src/server.ts"]),
              confidence: "advisory",
            }),
          ]),
        );
      } finally {
        fixture.cleanup();
      }
    },
    ENTRY_SURFACE_TEST_TIMEOUT_MS,
  );

  it(
    "E27. runtime script argv tokens do not become entry evidence",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "vitest-entry-runtime-script-argv-",
        packageJson: {
          name: "entry-surface-runtime-script-argv",
          private: true,
          type: "module",
          scripts: {
            start: "node src/main.ts src/config.ts",
          },
        },
      });
      try {
        writeFiles(fixture, {
          "src/main.ts": "export function main() {}\n",
          "src/config.ts": "export const config = {};\n",
        });

        const entrySurface = runEntrySurface(fixture);

        expect(entrySurface.scriptEntrypointFiles).toContain("src/main.ts");
        expect(entrySurface.scriptEntrypointFiles).not.toContain(
          "src/config.ts",
        );
        expect(entrySurface.entryFiles).toContain("src/main.ts");
        expect(entrySurface.entryFiles).not.toContain("src/config.ts");
        expect(entrySurface.evidenceByFile?.["src/config.ts"]).toBeUndefined();
      } finally {
        fixture.cleanup();
      }
    },
    ENTRY_SURFACE_TEST_TIMEOUT_MS,
  );

  it(
    "E28. package script wrappers record scoped unsupported diagnostics without entry evidence",
    () => {
      const fixture = createTempRepoFixture({
        prefix: "vitest-entry-script-recursion-",
        packageJson: {
          name: "entry-surface-script-recursion",
          private: true,
          type: "module",
          scripts: {
            start: "npm run server",
          },
        },
      });
      try {
        writeFiles(fixture, {
          "src/server.ts": "export function listen() {}\n",
        });

        const entrySurface = runEntrySurface(fixture);

        expect(entrySurface.scriptEntrypointFiles).not.toContain(
          "src/server.ts",
        );
        expect(entrySurface.entryFiles).not.toContain("src/server.ts");
        expect(entrySurface.unsupportedScriptEntrypointCount).toBe(1);
        expect(entrySurface.unsupportedScriptEntrypoints).toEqual(
          expect.arrayContaining([
            expect.objectContaining({
              reason: "package-script-recursion-unsupported",
              scriptName: "start",
              tool: "npm",
              targetScript: "server",
              confidence: "advisory",
            }),
          ]),
        );
      } finally {
        fixture.cleanup();
      }
    },
    ENTRY_SURFACE_TEST_TIMEOUT_MS,
  );
});

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");

function runTopologyDynamicFixture() {
  const fixture = createTempRepoFixture({
    prefix: "vitest-topo-dynamic-",
    packageJson: { name: "topo-fx", type: "module" },
  });

  try {
    fixture.write(
      "src/a.ts",
      [
        "export async function lazy() {",
        "  const m = await import('./target');",
        "  return m;",
        "}",
        "",
      ].join("\n"),
    );
    fixture.write(
      "src/b.ts",
      [
        "export async function cond(flag) {",
        "  if (flag) {",
        "    return import('./plugin');",
        "  }",
        "  return null;",
        "}",
        "",
      ].join("\n"),
    );
    fixture.write(
      "src/c.ts",
      [
        "export function loadLater() {",
        "  return import('./utils').then((m) => m.default);",
        "}",
        "",
      ].join("\n"),
    );
    fixture.write(
      "src/d.ts",
      [
        "export const routes = {",
        "  home: () => import('./home-page'),",
        "  about: () => import('./about-page'),",
        "};",
        "",
      ].join("\n"),
    );
    fixture.write("src/target.ts", "export const T = 1;\n");
    fixture.write("src/plugin.ts", "export const P = 2;\n");
    fixture.write("src/utils.ts", "export default {};\n");
    fixture.write("src/home-page.ts", "export const H = 3;\n");
    fixture.write("src/about-page.ts", "export const A = 4;\n");
    fixture.write(
      "src/control.ts",
      ["import { T } from './target';", "export const x = T;", ""].join("\n"),
    );
    fixture.write(
      "src/fallback.ts",
      [
        "export function cjs() {",
        "  const c = require('./target');",
        "  return c;",
        "}",
        "",
      ].join("\n"),
    );

    execFileSync(
      "node",
      [
        path.join(REPO_ROOT, "measure-topology.mjs"),
        "--root",
        fixture.root,
        "--output",
        fixture.output,
      ],
      { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
    );

    return JSON.parse(
      readFileSync(path.join(fixture.output, "topology.json"), "utf8"),
    );
  } finally {
    fixture.cleanup();
  }
}

function fanMap(entries) {
  return Object.fromEntries(
    (entries ?? []).map((entry) => [entry.file, entry.count]),
  );
}

describe("topology literal dynamic import edge detection", () => {
  it("counts static and literal dynamic imports as internal topology edges", () => {
    const topology = runTopologyDynamicFixture();

    expect(topology.summary.internalEdges).toBe(6);
    expect(topology.summary.parseErrors).toBe(0);
    expect(fanMap(topology.topFanIn)["src/target.ts"]).toBeGreaterThanOrEqual(
      2,
    );
  });

  it("records fan-out for object-literal, conditional, and awaited dynamic imports", () => {
    const topology = runTopologyDynamicFixture();
    const fanOut = fanMap(topology.topFanOut);

    expect(fanOut["src/d.ts"]).toBe(2);
    expect(fanOut["src/b.ts"]).toBe(1);
    expect(fanOut["src/a.ts"]).toBe(1);
  });

  it("keeps scanner fallback and parser counters visible for unsupported require calls", () => {
    const topology = runTopologyDynamicFixture();
    const perf = topology.summary.performance;

    expect(perf.filesCollected).toBe(11);
    expect(perf.changedFiles).toBe(11);
    expect(perf.unchangedFiles).toBe(0);
    expect(perf.droppedFiles).toBe(0);
    expect(perf.jsFilesProcessed).toBe(11);
    expect(perf.scannerPolicyVersion).toBe("module-edge-scanner-v1");
    expect(perf.scannerFilesAttempted).toBe(11);
    expect(perf.scannerAcceptedFiles).toBe(10);
    expect(perf.scannerFallbackFiles).toBe(1);
    expect(perf.scannerRiskCounts?.["require-call"]).toBe(1);
    expect(perf.oxcParseCalls).toBe(1);
    expect(perf.oxcParseErrors).toBe(0);
    expect(typeof perf.resolverMemoHits).toBe("number");
    expect(typeof perf.resolverMemoMisses).toBe("number");
    expect(typeof perf.resolverMemoSize).toBe("number");
  });
});

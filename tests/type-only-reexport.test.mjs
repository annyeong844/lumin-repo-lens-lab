import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");

function runTypeOnlyReexportFixture() {
  const fixture = createTempRepoFixture({
    prefix: "vitest-type-only-reexport-",
    packageJson: { name: "typeonly", type: "module" },
  });

  try {
    fixture.write(
      "src/a.ts",
      [
        "export type { BType } from './b';",
        'export type AType = { tag: "a" };',
        "export const aRuntime = 1;",
        "",
      ].join("\n"),
    );
    fixture.write(
      "src/b.ts",
      [
        "export type { AType } from './a';",
        'export type BType = { tag: "b" };',
        "export const bRuntime = 2;",
        "",
      ].join("\n"),
    );
    fixture.write(
      "src/c.ts",
      [
        "export { runtimeD, type DType } from './d';",
        "export const runtimeC = 1;",
        "",
      ].join("\n"),
    );
    fixture.write(
      "src/d.ts",
      [
        "export { runtimeC } from './c';",
        "export const runtimeD = 2;",
        'export type DType = { tag: "d" };',
        "",
      ].join("\n"),
    );
    fixture.write(
      "src/types.ts",
      [
        "export type Foo = { x: number };",
        "export const runtimeValue = 42;",
        "",
      ].join("\n"),
    );
    fixture.write("src/kind2.ts", "export type * from './types';\n");
    fixture.write("src/kind3.ts", "export { type Foo } from './types';\n");

    for (const script of ["build-symbol-graph.mjs", "measure-topology.mjs"]) {
      execFileSync(
        "node",
        [
          path.join(REPO_ROOT, script),
          "--root",
          fixture.root,
          "--output",
          fixture.output,
        ],
        { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
      );
    }

    return {
      topology: JSON.parse(
        readFileSync(path.join(fixture.output, "topology.json"), "utf8"),
      ),
      symbols: JSON.parse(
        readFileSync(path.join(fixture.output, "symbols.json"), "utf8"),
      ),
    };
  } finally {
    fixture.cleanup();
  }
}

function cycleFilesFrom(topology) {
  const summary = topology.summary ?? topology;
  const sccs = summary.topSccs ?? summary.sccs ?? topology.sccs ?? [];
  const first = sccs[0];
  return first
    ? (first.members ?? first.files ?? first.nodes ?? []).map((member) =>
        typeof member === "string" ? member : (member.file ?? ""),
      )
    : [];
}

describe("topology runtime lens for type-only re-exports", () => {
  it("counts type-only re-export forms without treating them as runtime cycles", () => {
    const { topology } = runTypeOnlyReexportFixture();
    const summary = topology.summary ?? topology;

    expect(typeof summary.typeOnlyEdges).toBe("number");
    expect(summary.typeOnlyEdges).toBeGreaterThanOrEqual(4);
    expect(summary.sccCount ?? 0).toBe(1);
  });

  it("keeps the mixed/runtime c-d cycle and erases the pure type-only a-b cycle", () => {
    const { topology } = runTypeOnlyReexportFixture();
    const cycleFiles = cycleFilesFrom(topology);

    expect(cycleFiles.some((file) => file.endsWith("c.ts"))).toBe(true);
    expect(cycleFiles.some((file) => file.endsWith("d.ts"))).toBe(true);
    expect(cycleFiles.some((file) => file.endsWith("a.ts"))).toBe(false);
    expect(cycleFiles.some((file) => file.endsWith("b.ts"))).toBe(false);
  });

  it("tracks the exact re-exporting files in symbols.json", () => {
    const { symbols } = runTypeOnlyReexportFixture();
    const files = Object.keys(symbols.reExportsByFile ?? {}).sort();

    expect(files).toEqual([
      "src/a.ts",
      "src/b.ts",
      "src/c.ts",
      "src/d.ts",
      "src/kind2.ts",
      "src/kind3.ts",
    ]);
  });
});

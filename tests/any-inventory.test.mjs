import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const ROOT = path.resolve(".");
const CLI = path.join(ROOT, "any-inventory.mjs");
const CANON_ESCAPE_KINDS = [
  "explicit-any",
  "as-any",
  "angle-any",
  "as-unknown-as-T",
  "rest-any-args",
  "index-sig-any",
  "generic-default-any",
  "ts-ignore",
  "ts-expect-error",
  "no-explicit-any-disable",
  "jsdoc-any",
];

function runInventory(fixture, extraArgs = []) {
  execFileSync(
    "node",
    [CLI, "--root", fixture.root, "--output", fixture.output, ...extraArgs],
    {
      cwd: ROOT,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function writeManual(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

describe("any-inventory producer emits type escape evidence", () => {
  it("emits every canonical escape kind from a full fixture", () => {
    const fixture = createTempRepoFixture({ prefix: "fx-vitest-any-full-" });
    try {
      fixture.write(
        "src/all.ts",
        "type A = any;\n" +
          "const b = (x as any);\n" +
          "const c = (<any>x);\n" +
          "const d = (x as unknown as Foo);\n" +
          "function e(...args: any[]) {}\n" +
          "type F = { [k: string]: any };\n" +
          "type G<T = any> = T;\n" +
          "// @ts-ignore reason\nconst h = 1;\n" +
          "// @ts-expect-error reason\nconst i = 1;\n" +
          "// eslint-disable-next-line no-explicit-any\nconst j = 1;\n" +
          "/** @type {any} */\nconst k = readValue();\n",
      );

      runInventory(fixture);
      const inventory = fixture.readJson("any-inventory.json", {
        from: "output",
      });
      const emittedKinds = new Set(
        inventory.typeEscapes.map((entry) => entry.escapeKind),
      );

      expect(inventory.typeEscapes).toHaveLength(11);
      for (const kind of CANON_ESCAPE_KINDS) {
        expect(emittedKinds, kind).toContain(kind);
      }
    } finally {
      fixture.cleanup();
    }
  });

  it("populates complete metadata and canonical escape-kind order on clean runs", () => {
    const fixture = createTempRepoFixture({ prefix: "fx-vitest-any-meta-" });
    try {
      fixture.write("src/a.ts", "export const foo = x as any;\n");

      runInventory(fixture);
      const inventory = fixture.readJson("any-inventory.json", {
        from: "output",
      });

      expect(inventory.meta).toMatchObject({
        tool: "any-inventory.mjs",
        complete: true,
        scope: "TS/JS including tests",
        includeTests: true,
      });
      expect(inventory.meta.supports?.typeEscapes).toBe(true);
      expect(inventory.meta.supports?.escapeKinds).toEqual(CANON_ESCAPE_KINDS);
      expect(inventory.meta.fileCount).toBeGreaterThanOrEqual(1);
      expect(inventory.meta.exclude).toEqual(expect.any(Array));
    } finally {
      fixture.cleanup();
    }
  });

  it("marks parse-error runs incomplete while preserving clean-file facts", () => {
    const fixture = createTempRepoFixture({ prefix: "fx-vitest-any-parse-" });
    try {
      fixture.write("src/bad.ts", "const x = ;;;broken\n");
      fixture.write("src/good.ts", "const y = z as any;\n");

      runInventory(fixture);
      const inventory = fixture.readJson("any-inventory.json", {
        from: "output",
      });
      const [firstError] = inventory.meta.filesWithParseErrors ?? [];

      expect(inventory.meta.complete).toBe(false);
      expect(
        inventory.meta.filesWithParseErrors?.length,
      ).toBeGreaterThanOrEqual(1);
      expect(firstError).toMatchObject({
        file: expect.any(String),
        message: expect.any(String),
      });
      expect(
        inventory.typeEscapes.some((entry) => /bad\.ts$/.test(entry.file)),
      ).toBe(false);
      expect(
        inventory.typeEscapes.some(
          (entry) =>
            /good\.ts$/.test(entry.file) && entry.escapeKind === "as-any",
        ),
      ).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("scans tests by default and excludes tests with --production", () => {
    const fixture = createTempRepoFixture({ prefix: "fx-vitest-any-scope-" });
    try {
      fixture.write("src/a.ts", "const x = 1;\n");
      fixture.write("tests/sample.test.ts", "const y = z as any;\n");

      runInventory(fixture);
      const defaultInventory = fixture.readJson("any-inventory.json", {
        from: "output",
      });

      expect(defaultInventory.meta.includeTests).toBe(true);
      expect(defaultInventory.meta.scope).toBe("TS/JS including tests");
      expect(
        defaultInventory.typeEscapes.some((entry) =>
          /sample\.test\.ts$/.test(entry.file),
        ),
      ).toBe(true);

      const production = createTempRepoFixture({
        prefix: "fx-vitest-any-scope-prod-",
      });
      try {
        production.write("src/a.ts", "const x = 1;\n");
        production.write("tests/sample.test.ts", "const y = z as any;\n");
        runInventory(production, ["--production"]);
        const productionInventory = production.readJson("any-inventory.json", {
          from: "output",
        });

        expect(productionInventory.meta.includeTests).toBe(false);
        expect(productionInventory.meta.scope).toBe("TS/JS production files");
        expect(
          productionInventory.typeEscapes.some((entry) =>
            /sample\.test\.ts$/.test(entry.file),
          ),
        ).toBe(false);
      } finally {
        production.cleanup();
      }
    } finally {
      fixture.cleanup();
    }
  });

  it("handles shell-sensitive fixture and output paths", () => {
    const parent = mkdtempSync(path.join(tmpdir(), "fx-vitest-any-shell-"));
    const root = path.join(parent, "my $fixture");
    const output = path.join(parent, "my $output");
    try {
      mkdirSync(root, { recursive: true });
      mkdirSync(output, { recursive: true });
      writeManual(root, "package.json", '{"name":"fixture","type":"module"}\n');
      writeManual(root, "src/a.ts", "const x = y as any;\n");

      execFileSync("node", [CLI, "--root", root, "--output", output], {
        cwd: ROOT,
        stdio: ["ignore", "pipe", "pipe"],
      });
      const inventory = JSON.parse(
        readFileSync(path.join(output, "any-inventory.json"), "utf8"),
      );

      expect(inventory.meta.complete).toBe(true);
      expect(inventory.typeEscapes.length).toBeGreaterThanOrEqual(1);
    } finally {
      rmSync(parent, { recursive: true, force: true });
    }
  });

  it("emits required typeEscape fields with exported identity", () => {
    const fixture = createTempRepoFixture({ prefix: "fx-vitest-any-fields-" });
    try {
      fixture.write("src/a.ts", "export const foo = () => x as any;\n");

      runInventory(fixture);
      const inventory = fixture.readJson("any-inventory.json", {
        from: "output",
      });
      const hit = inventory.typeEscapes.find(
        (entry) => entry.escapeKind === "as-any",
      );

      expect(hit).toMatchObject({
        file: expect.any(String),
        line: expect.any(Number),
        escapeKind: "as-any",
        codeShape: expect.any(String),
        normalizedCodeShape: expect.any(String),
        occurrenceKey: expect.stringMatching(/^sha256:[a-f0-9]{64}$/),
      });
      expect(hit?.insideExportedIdentity).toMatch(/::foo$/);
    } finally {
      fixture.cleanup();
    }
  });

  it("writes only the requested custom artifact for --artifact-name", () => {
    const fixture = createTempRepoFixture({ prefix: "fx-vitest-any-custom-" });
    try {
      fixture.write("src/a.ts", "export const foo = () => x as any;\n");

      runInventory(fixture, ["--artifact-name", "any-inventory.pre.test.json"]);
      const customPath = fixture.outputPath("any-inventory.pre.test.json");
      const inventory = fixture.readJson("any-inventory.pre.test.json", {
        from: "output",
      });

      expect(existsSync(customPath)).toBe(true);
      expect(existsSync(fixture.outputPath("any-inventory.json"))).toBe(false);
      expect(
        inventory.typeEscapes.some((entry) => entry.escapeKind === "as-any"),
      ).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });
});

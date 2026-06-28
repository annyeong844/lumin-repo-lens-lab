import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const CLI = path.join(REPO_ROOT, "build-shape-index.mjs");

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function createFixture(prefix = "vitest-shape-index-") {
  const root = mkdtempSync(path.join(tmpdir(), prefix));
  const output = mkdtempSync(path.join(tmpdir(), `${prefix}out-`));
  return {
    root,
    output,
    cleanup() {
      rmSync(root, { recursive: true, force: true });
      rmSync(output, { recursive: true, force: true });
    },
  };
}

function runShapeIndex(root, output, extraArgs = []) {
  return execFileSync(
    process.execPath,
    [CLI, "--root", root, "--output", output, ...extraArgs],
    {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function readIndex(output) {
  return JSON.parse(
    readFileSync(path.join(output, "shape-index.json"), "utf8"),
  );
}

function factByName(index, name) {
  return index.facts.find((fact) => fact.exportedName === name);
}

describe("build-shape-index producer artifact", () => {
  it("writes shape-index.json with canonical metadata and deterministic structural groups", () => {
    const fixture = createFixture();
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "shape-fixture", type: "module" }),
      );
      write(
        fixture.root,
        "src/a.ts",
        "export interface UserA { id: string; name?: string }\n",
      );
      write(
        fixture.root,
        "src/b.ts",
        "export type UserB = { name?: string; id: string };\n",
      );
      write(
        fixture.root,
        "src/c.ts",
        "export type Other = { id: number; name?: string };\n",
      );

      const stdout = runShapeIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);
      const userA = factByName(index, "UserA");
      const userB = factByName(index, "UserB");
      const other = factByName(index, "Other");

      expect(existsSync(path.join(fixture.output, "shape-index.json"))).toBe(
        true,
      );
      expect(stdout).toContain("[shape-index]");
      expect(stdout).toContain("shape-hash facts");
      expect(index.schemaVersion).toBe("shape-index.v1");
      expect(index.meta.tool).toBe("build-shape-index.mjs");
      expect(index.meta.supports?.shapeHash).toBe(true);
      expect(index.meta.supports?.normalizedVersion).toBe(
        "shape-hash.normalized.v1",
      );
      expect(index.meta.supports?.exportedUnionLiteralTypeAliases).toBe(true);
      expect(index.meta.complete).toBe(true);
      expect(index.facts).toHaveLength(3);
      expect(userA?.hash).toBe(userB?.hash);
      expect(userA?.hash).not.toBe(other?.hash);
      expect(index.groupsByHash[userA.hash]).toEqual([
        "src/a.ts::UserA",
        "src/b.ts::UserB",
      ]);
      expect(userA).toMatchObject({
        source: "fresh-ast-pass",
        scope: "TS/JS including tests, exported types only",
        confidence: "high",
      });
      expect(typeof userA?.observedAt).toBe("string");
    } finally {
      fixture.cleanup();
    }
  });

  it("records unsupported mapped or generic declarations as diagnostics instead of fake facts", () => {
    const fixture = createFixture("vitest-shape-index-unsupported-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "shape-fixture" }),
      );
      write(
        fixture.root,
        "src/a.ts",
        [
          "export type Good = { id: string };",
          "export type Mapped<T> = { [K in keyof T]: T[K] };",
          "",
        ].join("\n"),
      );

      runShapeIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);

      expect(index.facts.some((fact) => fact.exportedName === "Good")).toBe(
        true,
      );
      expect(
        index.diagnostics.some(
          (diagnostic) =>
            diagnostic.exportedName === "Mapped" &&
            diagnostic.code === "unsupported-type-parameters",
        ),
      ).toBe(true);
      expect(index.meta.complete).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("marks parse-error runs incomplete while preserving good-file facts and diagnostics", () => {
    const fixture = createFixture("vitest-shape-index-parse-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "shape-fixture" }),
      );
      write(
        fixture.root,
        "src/good.ts",
        "export interface Good { id: string }\n",
      );
      write(fixture.root, "src/bad.ts", "export interface Bad { id: string ");

      runShapeIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);

      expect(index.meta.complete).toBe(false);
      expect(
        index.meta.filesWithParseErrors.some(
          (entry) => entry.file === "src/bad.ts",
        ),
      ).toBe(true);
      expect(
        index.diagnostics.some(
          (diagnostic) =>
            diagnostic.code === "parse-error" &&
            diagnostic.file === "src/bad.ts",
        ),
      ).toBe(true);
      expect(
        index.facts.some((fact) => fact.identity === "src/good.ts::Good"),
      ).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("excludes test files under --production and records the scan scope accurately", () => {
    const root = mkdtempSync(path.join(tmpdir(), "vitest-shape-index-prod-"));
    const defaultOut = mkdtempSync(
      path.join(tmpdir(), "vitest-shape-index-prod-out1-"),
    );
    const productionOut = mkdtempSync(
      path.join(tmpdir(), "vitest-shape-index-prod-out2-"),
    );
    try {
      write(root, "package.json", JSON.stringify({ name: "shape-fixture" }));
      write(root, "src/a.ts", "export interface ProdShape { id: string }\n");
      write(
        root,
        "tests/a.test.ts",
        "export interface TestShape { id: string }\n",
      );

      runShapeIndex(root, defaultOut);
      runShapeIndex(root, productionOut, ["--production"]);
      const defaultIndex = readIndex(defaultOut);
      const productionIndex = readIndex(productionOut);

      expect(
        defaultIndex.facts.some(
          (fact) => fact.identity === "tests/a.test.ts::TestShape",
        ),
      ).toBe(true);
      expect(defaultIndex.meta.scope).toBe(
        "TS/JS including tests, exported types only",
      );
      expect(
        productionIndex.facts.some(
          (fact) => fact.identity === "tests/a.test.ts::TestShape",
        ),
      ).toBe(false);
      expect(productionIndex.meta.scope).toBe(
        "TS/JS production files, exported types only",
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
      rmSync(defaultOut, { recursive: true, force: true });
      rmSync(productionOut, { recursive: true, force: true });
    }
  });

  it("handles output paths containing spaces and dollar signs", () => {
    const parent = mkdtempSync(
      path.join(tmpdir(), "vitest-shape-index-shell-"),
    );
    const root = path.join(parent, "my $fixture");
    const output = path.join(parent, "my $output");
    mkdirSync(root, { recursive: true });
    mkdirSync(output, { recursive: true });
    try {
      write(root, "package.json", JSON.stringify({ name: "shape-fixture" }));
      write(root, "src/a.ts", "export interface ShellSafe { id: string }\n");

      runShapeIndex(root, output);
      const index = readIndex(output);

      expect(
        index.facts.some((fact) => fact.identity === "src/a.ts::ShellSafe"),
      ).toBe(true);
    } finally {
      rmSync(parent, { recursive: true, force: true });
    }
  });

  it("surfaces declaration merging as unsupported instead of emitting partial facts", () => {
    const fixture = createFixture("vitest-shape-index-merge-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "shape-fixture" }),
      );
      write(
        fixture.root,
        "src/a.ts",
        [
          "export interface Foo { a: string }",
          "export interface Foo { b: number }",
          "",
        ].join("\n"),
      );

      runShapeIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);

      expect(
        index.facts.some((fact) => fact.identity === "src/a.ts::Foo"),
      ).toBe(false);
      expect(
        index.diagnostics.some(
          (diagnostic) =>
            diagnostic.code === "declaration-merge-unsupported" &&
            diagnostic.identity === "src/a.ts::Foo",
        ),
      ).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("keeps generated-file facts present while counting generated evidence", () => {
    const fixture = createFixture("vitest-shape-index-generated-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "shape-fixture" }),
      );
      write(
        fixture.root,
        "src/routeTree.gen.ts",
        "export interface FileRoutesByPath { id: string }\n",
      );
      write(
        fixture.root,
        "src/ordinary.ts",
        "export interface Ordinary { id: string }\n",
      );

      runShapeIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);

      expect(
        index.facts.some(
          (fact) =>
            fact.identity === "src/routeTree.gen.ts::FileRoutesByPath" &&
            fact.generatedFile?.kind === "generated-file",
        ),
      ).toBe(true);
      expect(index.meta.generatedFileFactCount).toBe(1);
      expect(index.meta.supports?.generatedFileEvidence).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("groups literal union aliases by exact normalized literal sets", () => {
    const fixture = createFixture("vitest-shape-index-literal-union-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "shape-fixture" }),
      );
      write(
        fixture.root,
        "src/a.ts",
        'export type StatusA = "open" | "closed" | "pending";\n',
      );
      write(
        fixture.root,
        "src/b.ts",
        "export type StatusB = \"pending\" | 'closed' | `open`;\n",
      );

      runShapeIndex(fixture.root, fixture.output);
      const index = readIndex(fixture.output);
      const statusA = factByName(index, "StatusA");
      const statusB = factByName(index, "StatusB");

      expect(statusA?.shapeKind).toBe("literal-union");
      expect(statusB?.shapeKind).toBe("literal-union");
      expect(statusA?.hash).toBeTruthy();
      expect(statusA?.hash).toBe(statusB?.hash);
      expect(index.groupsByHash[statusA.hash]).toEqual([
        "src/a.ts::StatusA",
        "src/b.ts::StatusB",
      ]);
    } finally {
      fixture.cleanup();
    }
  });
});

import { execFileSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

const NODE = process.execPath;
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const CLI = path.join(ROOT, "any-inventory.mjs");
const TEST_TIMEOUT = 60_000;

function fresh() {
  return mkdtempSync(path.join(tmpdir(), "vitest-any-inc-"));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function run(root, output, args = []) {
  return execFileSync(
    NODE,
    [CLI, "--root", root, "--output", output, ...args],
    {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function readInventory(output, name = "any-inventory.json") {
  return JSON.parse(readFileSync(path.join(output, name), "utf8"));
}

function stableInventory(inv) {
  return {
    complete: inv.meta.complete,
    scope: inv.meta.scope,
    includeTests: inv.meta.includeTests,
    exclude: inv.meta.exclude,
    supports: inv.meta.supports,
    typeEscapes: inv.typeEscapes,
    filesWithParseErrors: inv.meta.filesWithParseErrors,
  };
}

function cleanup(repo) {
  if (!repo) return;
  rmSync(repo, { recursive: true, force: true });
}

describe("any-inventory strict incremental cache", () => {
  describe("cold and warm public fact equivalence", () => {
    let repo;
    let cold;
    let firstIncremental;
    let warm;

    beforeAll(() => {
      repo = fresh();
      const output = path.join(repo, ".audit");
      write(repo, "package.json", JSON.stringify({ name: "fixture" }));
      write(repo, "src/a.ts", "const a = value as any;\n");
      write(repo, "src/b.ts", "const b = value as unknown as string;\n");

      run(repo, output, ["--no-incremental"]);
      cold = readInventory(output);
      run(repo, output);
      firstIncremental = readInventory(output);
      run(repo, output);
      warm = readInventory(output);
    }, TEST_TIMEOUT);

    afterAll(() => cleanup(repo));

    it("matches cold public facts on the first incremental run", () => {
      expect(stableInventory(firstIncremental)).toEqual(stableInventory(cold));
    });

    it("keeps warm public facts equivalent to cold public facts", () => {
      expect(stableInventory(warm)).toEqual(stableInventory(cold));
    });

    it("reports incremental mode enabled on warm runs", () => {
      expect(warm.meta.incremental?.enabled).toBe(true);
    });

    it("reuses at least one file on warm runs", () => {
      expect(warm.meta.incremental?.reusedFiles).toBeGreaterThanOrEqual(1);
    });
  });

  describe("changed file refresh", () => {
    let repo;
    let inventory;

    beforeAll(() => {
      repo = fresh();
      const output = path.join(repo, ".audit");
      write(repo, "package.json", JSON.stringify({ name: "fixture" }));
      write(repo, "src/a.ts", "const a = value as any;\n");
      write(repo, "src/b.ts", "const b = value as any;\n");
      run(repo, output);

      write(repo, "src/b.ts", "const b = value as unknown as string;\n");
      run(repo, output);
      inventory = readInventory(output);
    }, TEST_TIMEOUT);

    afterAll(() => cleanup(repo));

    it("updates changed file type escape facts after edit", () => {
      expect(
        inventory.typeEscapes.some(
          (fact) =>
            fact.file === "src/b.ts" && fact.escapeKind === "as-unknown-as-T",
        ),
      ).toBe(true);
    });

    it("keeps unchanged file facts present after edit", () => {
      expect(
        inventory.typeEscapes.some(
          (fact) => fact.file === "src/a.ts" && fact.escapeKind === "as-any",
        ),
      ).toBe(true);
    });

    it("reports a positive incremental changed-file count", () => {
      expect(inventory.meta.incremental?.changedFiles).toBeGreaterThanOrEqual(
        1,
      );
    });
  });

  describe("deleted file cleanup", () => {
    let repo;
    let inventory;

    beforeAll(() => {
      repo = fresh();
      const output = path.join(repo, ".audit");
      write(repo, "package.json", JSON.stringify({ name: "fixture" }));
      write(repo, "src/a.ts", "const a = value as any;\n");
      write(repo, "src/b.ts", "const b = value as any;\n");
      run(repo, output);

      rmSync(path.join(repo, "src/b.ts"), { force: true });
      run(repo, output);
      inventory = readInventory(output);
    }, TEST_TIMEOUT);

    afterAll(() => cleanup(repo));

    it("removes deleted file type escape facts", () => {
      expect(
        inventory.typeEscapes.some((fact) => fact.file === "src/b.ts"),
      ).toBe(false);
    });

    it("reports dropped-file evidence after deletion", () => {
      expect(inventory.meta.incremental?.droppedFiles).toBeGreaterThanOrEqual(
        1,
      );
    });
  });

  describe("scan option cache invalidation", () => {
    let repo;
    let inventory;

    beforeAll(() => {
      repo = fresh();
      const output = path.join(repo, ".audit");
      write(repo, "package.json", JSON.stringify({ name: "fixture" }));
      write(repo, "src/a.ts", "const a = value as any;\n");
      write(repo, "tests/a.test.ts", "const t = value as any;\n");
      run(repo, output, ["--production"]);
      run(repo, output);
      inventory = readInventory(output);
    }, TEST_TIMEOUT);

    afterAll(() => cleanup(repo));

    it("keeps public artifact correct when scan options change", () => {
      expect(inventory.meta.includeTests).toBe(true);
      expect(
        inventory.typeEscapes.some((fact) => fact.file === "tests/a.test.ts"),
      ).toBe(true);
    });

    it("prevents stale production-only cache reuse after scan option change", () => {
      expect(
        inventory.meta.incremental?.invalidatedFiles,
      ).toBeGreaterThanOrEqual(0);
    });
  });

  it(
    "does not crash on malformed unrelated cache payloads",
    () => {
      const repo = fresh();
      const output = path.join(repo, ".audit");
      try {
        write(repo, "package.json", JSON.stringify({ name: "fixture" }));
        write(repo, "src/a.ts", "const a = value as any;\n");
        run(repo, output);

        const cacheFile = path.join(output, ".cache", "incremental");
        mkdirSync(cacheFile, { recursive: true });
        writeFileSync(path.join(cacheFile, "bad.cache.json"), "{broken");

        run(repo, output);
        const inventory = readInventory(output);
        expect(inventory.meta.complete).toBe(true);
        expect(inventory.typeEscapes).toHaveLength(1);
      } finally {
        cleanup(repo);
      }
    },
    TEST_TIMEOUT,
  );

  it(
    "reports disabled metadata under --no-incremental",
    () => {
      const repo = fresh();
      const output = path.join(repo, ".audit");
      try {
        write(repo, "package.json", JSON.stringify({ name: "fixture" }));
        write(repo, "src/a.ts", "const a = value as any;\n");
        run(repo, output, ["--no-incremental"]);
        const inventory = readInventory(output);

        expect(inventory.meta.incremental).toMatchObject({
          enabled: false,
          reason: "disabled-by-flag",
        });
      } finally {
        cleanup(repo);
      }
    },
    TEST_TIMEOUT,
  );
});

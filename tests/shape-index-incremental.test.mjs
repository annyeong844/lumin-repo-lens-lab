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

import { describe, expect, it } from "vitest";

const NODE = process.execPath;
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const CLI = path.join(ROOT, "build-shape-index.mjs");
const AUDIT = path.join(ROOT, "audit-repo.mjs");

function fresh() {
  return mkdtempSync(path.join(tmpdir(), "vitest-shape-inc-"));
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

function runAudit(root, output, args = []) {
  return execFileSync(
    NODE,
    [AUDIT, "--root", root, "--output", output, ...args],
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

function stableIndex(index) {
  const facts = (index.facts ?? []).map(
    ({ observedAt: _observedAt, ...fact }) => fact,
  );
  const { meta, ...rest } = index;
  const {
    generated: _generated,
    observedAt: _metaObservedAt,
    incremental: _incremental,
    ...stableMeta
  } = meta ?? {};
  return {
    meta: stableMeta,
    ...rest,
    facts,
  };
}

function setupRepo(repo) {
  write(
    repo,
    "package.json",
    JSON.stringify({ name: "fixture", private: true }),
  );
  write(
    repo,
    "src/a.ts",
    "export interface UserA { id: string; name?: string }\n",
  );
  write(
    repo,
    "src/b.ts",
    "export type UserB = { name?: string; id: string };\n",
  );
  write(
    repo,
    "src/c.ts",
    "export type Other = { id: number; name?: string };\n",
  );
}

describe("shape index strict incremental cache", () => {
  it("matches cold public facts, reports strict mode, reuses unchanged facts, and restamps reused facts", () => {
    const repo = fresh();
    const output = path.join(repo, ".audit");
    try {
      setupRepo(repo);

      run(repo, output, ["--no-incremental"]);
      const cold = readIndex(output);
      run(repo, output);
      const firstIncremental = readIndex(output);
      run(repo, output);
      const warm = readIndex(output);

      expect(stableIndex(firstIncremental)).toEqual(stableIndex(cold));
      expect(stableIndex(warm)).toEqual(stableIndex(cold));
      expect(warm.meta.incremental).toMatchObject({
        enabled: true,
        identityMode: "strict-content-hash",
      });
      expect(warm.meta.incremental.reusedFiles).toBeGreaterThanOrEqual(3);
      expect(
        warm.facts.every((fact) => fact.observedAt === warm.meta.observedAt),
      ).toBe(true);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("refreshes changed file facts while reusing unchanged files", () => {
    const repo = fresh();
    const output = path.join(repo, ".audit");
    try {
      setupRepo(repo);
      run(repo, output);
      run(repo, output);

      write(
        repo,
        "src/b.ts",
        "export type UserB = { name?: string; id: number };\n",
      );
      run(repo, output);
      const index = readIndex(output);
      const userA = index.facts.find(
        (fact) => fact.identity === "src/a.ts::UserA",
      );
      const userB = index.facts.find(
        (fact) => fact.identity === "src/b.ts::UserB",
      );

      expect(userA?.hash).toBeTruthy();
      expect(userB?.hash).toBeTruthy();
      expect(userA.hash).not.toBe(userB.hash);
      expect(index.meta.incremental.changedFiles).toBeGreaterThanOrEqual(1);
      expect(index.meta.incremental.reusedFiles).toBeGreaterThanOrEqual(1);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("drops deleted file facts and increments dropped-file evidence", () => {
    const repo = fresh();
    const output = path.join(repo, ".audit");
    try {
      setupRepo(repo);
      run(repo, output);
      run(repo, output);

      rmSync(path.join(repo, "src/b.ts"), { force: true });
      run(repo, output);
      const index = readIndex(output);

      expect(index.facts.some((fact) => fact.ownerFile === "src/b.ts")).toBe(
        false,
      );
      expect(index.meta.incremental.droppedFiles).toBeGreaterThanOrEqual(1);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("reports disabled cache metadata under --no-incremental", () => {
    const repo = fresh();
    const output = path.join(repo, ".audit");
    try {
      setupRepo(repo);
      run(repo, output, ["--no-incremental"]);
      const index = readIndex(output);

      expect(index.meta.incremental).toMatchObject({
        enabled: false,
        reason: "disabled-by-flag",
      });
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("forwards audit-repo --no-incremental and --cache-root to build-shape-index", () => {
    const repo = fresh();
    const output = path.join(repo, ".audit");
    try {
      setupRepo(repo);
      runAudit(repo, output, ["--profile", "full", "--no-incremental"]);
      const coldIndex = readIndex(output);
      expect(coldIndex.meta.incremental).toMatchObject({
        enabled: false,
        reason: "disabled-by-flag",
      });

      const cacheRoot = path.join(repo, ".shape-cache");
      runAudit(repo, output, ["--profile", "full", "--cache-root", cacheRoot]);
      runAudit(repo, output, ["--profile", "full", "--cache-root", cacheRoot]);
      const warmIndex = readIndex(output);

      expect(warmIndex.meta.incremental).toMatchObject({
        enabled: true,
      });
      expect(path.resolve(warmIndex.meta.incremental.cacheRoot)).toBe(
        path.resolve(cacheRoot),
      );
      expect(warmIndex.meta.incremental.reusedFiles).toBeGreaterThanOrEqual(3);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  }, 60_000);
});

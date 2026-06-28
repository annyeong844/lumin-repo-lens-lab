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
const CLI = path.join(ROOT, "build-function-clone-index.mjs");

function fresh() {
  return mkdtempSync(path.join(tmpdir(), "vitest-fn-clone-inc-"));
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

function readIndex(output) {
  return JSON.parse(
    readFileSync(path.join(output, "function-clones.json"), "utf8"),
  );
}

function stripRunMetadata(value) {
  if (Array.isArray(value)) return value.map(stripRunMetadata);
  if (value && typeof value === "object") {
    const out = {};
    for (const [key, child] of Object.entries(value)) {
      if (key === "generated" || key === "observedAt" || key === "incremental")
        continue;
      out[key] = stripRunMetadata(child);
    }
    return out;
  }
  return value;
}

function stableIndex(index) {
  return stripRunMetadata(index);
}

function setupRepo(repo) {
  write(
    repo,
    "package.json",
    JSON.stringify({ name: "fixture", private: true }),
  );
  write(
    repo,
    "src/money-a.ts",
    `export function formatCurrencyCents(cents: number, currency = 'USD') {\n` +
      `  const dollars = cents / 100;\n` +
      `  return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(dollars);\n` +
      `}\n`,
  );
  write(
    repo,
    "src/money-b.ts",
    `export function renderPaymentTotal(value: number, unit = 'USD') {\n` +
      `  const amount = value / 100;\n` +
      `  return new Intl.NumberFormat('en-US', { style: 'currency', currency: unit }).format(amount);\n` +
      `}\n`,
  );
  write(
    repo,
    "src/exact-a.ts",
    `export const parseOne = (raw: string) => {\n` +
      `  const value = raw.trim();\n` +
      `  return value.toUpperCase();\n` +
      `};\n`,
  );
  write(
    repo,
    "src/exact-b.ts",
    `const local = (raw: string) => {\n` +
      `  const value = raw.trim();\n` +
      `  return value.toUpperCase();\n` +
      `};\n` +
      `export { local as parseTwo };\n`,
  );
}

describe("function clone strict incremental cache", () => {
  it("matches cold artifacts, reports strict mode, reuses unchanged payloads, and restamps reused facts", () => {
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
      expect(warm.meta.incremental.reusedFiles).toBeGreaterThanOrEqual(4);
      expect(
        warm.facts.every((fact) => fact.observedAt === warm.meta.observedAt),
      ).toBe(true);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("refreshes changed file facts, reuses unchanged files, and matches a cold artifact after the same change", () => {
    const repo = fresh();
    const output = path.join(repo, ".audit");
    try {
      setupRepo(repo);
      run(repo, output);
      run(repo, output);

      write(
        repo,
        "src/money-b.ts",
        `export function renderPaymentTotal(value: number, unit = 'USD') {\n` +
          `  const amount = value / 100;\n` +
          `  return new Intl.NumberFormat('en-GB', { style: 'currency', currency: unit }).format(amount);\n` +
          `}\n`,
      );
      run(repo, output);
      const index = readIndex(output);
      const changed = index.facts.find(
        (fact) => fact.identity === "src/money-b.ts::renderPaymentTotal",
      );
      const unchanged = index.facts.find(
        (fact) => fact.identity === "src/money-a.ts::formatCurrencyCents",
      );

      expect(changed?.exactBodyHash).toBeTruthy();
      expect(unchanged?.exactBodyHash).toBeTruthy();
      expect(changed.exactBodyHash).not.toBe(unchanged.exactBodyHash);
      expect(index.meta.incremental.changedFiles).toBeGreaterThanOrEqual(1);
      expect(index.meta.incremental.reusedFiles).toBeGreaterThanOrEqual(1);
      expect(index.meta.incremental.droppedFiles).toBe(0);

      const coldAfterChangeOutput = path.join(repo, ".audit-cold-after-change");
      run(repo, coldAfterChangeOutput, ["--no-incremental"]);
      const coldAfterChange = readIndex(coldAfterChangeOutput);
      expect(stableIndex(index)).toEqual(stableIndex(coldAfterChange));
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("rebuilds exact clone groups from mixed fresh and reused facts", () => {
    const repo = fresh();
    const output = path.join(repo, ".audit");
    try {
      setupRepo(repo);
      run(repo, output);
      run(repo, output);

      write(
        repo,
        "src/new-exact-c.ts",
        `export const parseThree = (raw: string) => {\n` +
          `  const value = raw.trim();\n` +
          `  return value.toUpperCase();\n` +
          `};\n`,
      );
      run(repo, output);
      const index = readIndex(output);
      const matchingGroup = (index.exactBodyGroups ?? []).find(
        (group) =>
          (group.identities ?? []).includes("src/exact-a.ts::parseOne") &&
          (group.identities ?? []).includes("src/new-exact-c.ts::parseThree"),
      );

      expect(matchingGroup).toBeTruthy();
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

      rmSync(path.join(repo, "src/exact-b.ts"), { force: true });
      run(repo, output);
      const index = readIndex(output);

      expect(
        index.facts.some((fact) => fact.ownerFile === "src/exact-b.ts"),
      ).toBe(false);
      expect(index.meta.incremental.droppedFiles).toBeGreaterThanOrEqual(1);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("treats a same-content file move as changed under relPath identity", () => {
    const repo = fresh();
    const output = path.join(repo, ".audit");
    try {
      setupRepo(repo);
      run(repo, output);
      run(repo, output);

      rmSync(path.join(repo, "src/money-a.ts"), { force: true });
      write(
        repo,
        "src/moved-money-a.ts",
        `export function formatCurrencyCents(cents: number, currency = 'USD') {\n` +
          `  const dollars = cents / 100;\n` +
          `  return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(dollars);\n` +
          `}\n`,
      );
      run(repo, output);
      const index = readIndex(output);

      expect(index.meta.incremental.changedFiles).toBeGreaterThanOrEqual(1);
      expect(index.meta.incremental.droppedFiles).toBeGreaterThanOrEqual(1);
      expect(
        index.facts.some(
          (fact) =>
            fact.identity === "src/moved-money-a.ts::formatCurrencyCents",
        ),
      ).toBe(true);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("clears incremental cache before a run when requested", () => {
    const repo = fresh();
    const output = path.join(repo, ".audit");
    try {
      setupRepo(repo);
      run(repo, output);
      run(repo, output);
      run(repo, output, ["--clear-incremental-cache"]);
      const index = readIndex(output);

      expect(index.meta.incremental).toMatchObject({
        enabled: true,
        reusedFiles: 0,
      });
      expect(index.meta.incremental.changedFiles).toBeGreaterThanOrEqual(4);
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
});

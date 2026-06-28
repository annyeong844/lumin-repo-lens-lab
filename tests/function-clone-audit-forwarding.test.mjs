import { it } from "vitest";
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

const NODE = process.execPath;
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const AUDIT_REPO = path.join(ROOT, "audit-repo.mjs");

function assert(label, ok, detail = "") {
  it(label, () => {
    if (!ok) {
      throw new Error(detail ? String(detail) : `Assertion failed: ${label}`);
    }
  });
}

function fresh() {
  return mkdtempSync(path.join(tmpdir(), "lumin-fn-clone-audit-forward-"));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
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
    `export const runA = () => {\n` +
      `  const value = 'a'.trim();\n` +
      `  return value.toUpperCase();\n` +
      `};\n`,
  );
  write(
    repo,
    "src/b.ts",
    `export const runB = () => {\n` +
      `  const value = 'b'.trim();\n` +
      `  return value.toUpperCase();\n` +
      `};\n`,
  );
}

function runAudit(root, output, args = []) {
  execFileSync(
    NODE,
    [AUDIT_REPO, "--root", root, "--output", output, ...args],
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

{
  const repo = fresh();
  const output = path.join(repo, ".audit");
  try {
    setupRepo(repo);
    runAudit(repo, output, ["--profile", "full", "--no-incremental"]);
    const index = readIndex(output);
    assert(
      "audit-repo forwards --no-incremental to function clone producer",
      index.meta.incremental?.enabled === false &&
        index.meta.incremental?.reason === "disabled-by-flag",
      JSON.stringify(index.meta.incremental),
    );
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, ".audit");
  try {
    setupRepo(repo);
    const cacheRoot = path.join(repo, "cache root with spaces");
    runAudit(repo, output, ["--profile", "full", "--cache-root", cacheRoot]);
    runAudit(repo, output, ["--profile", "full", "--cache-root", cacheRoot]);
    const warmIndex = readIndex(output);
    assert(
      "audit-repo forwards --cache-root to function clone producer",
      warmIndex.meta.incremental?.enabled === true &&
        path.resolve(warmIndex.meta.incremental.cacheRoot) ===
          path.resolve(cacheRoot) &&
        warmIndex.meta.incremental.reusedFiles >= 2,
      JSON.stringify(warmIndex.meta.incremental),
    );
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, ".audit");
  try {
    setupRepo(repo);
    const cacheRoot = path.join(repo, "cache root with spaces");
    runAudit(repo, output, ["--profile", "full", "--cache-root", cacheRoot]);
    runAudit(repo, output, ["--profile", "full", "--cache-root", cacheRoot]);
    runAudit(repo, output, [
      "--profile",
      "full",
      "--cache-root",
      cacheRoot,
      "--clear-incremental-cache",
    ]);
    const clearedIndex = readIndex(output);
    assert(
      "audit-repo clears shared incremental cache once before supported producers run",
      clearedIndex.meta.incremental?.enabled === true &&
        clearedIndex.meta.incremental.reusedFiles === 0 &&
        clearedIndex.meta.incremental.changedFiles >= 2,
      JSON.stringify(clearedIndex.meta.incremental),
    );
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

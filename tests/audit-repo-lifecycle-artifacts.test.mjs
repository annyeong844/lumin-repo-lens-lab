import { describe, expect, it } from "vitest";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const NODE = process.execPath;
const AUDIT_REPO = path.join(ROOT, "audit-repo.mjs");

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function runAudit(args) {
  const result = spawnSync(NODE, [AUDIT_REPO, ...args], {
    cwd: ROOT,
    encoding: "utf8",
  });

  expect(result.status, result.stderr || result.stdout).toBe(0);
  return result;
}

function readJson(file) {
  return JSON.parse(readFileSync(file, "utf8"));
}

describe("audit-repo lifecycle artifact collection split track", () => {
  it("O12. lists lifecycle artifacts only after opt-in modes run", () => {
    const repo = mkdtempSync(path.join(tmpdir(), "lumin-lifecycle-artifacts-"));
    const out = path.join(repo, "audit-out");
    const intent = path.join(repo, "intent.json");

    try {
      write(
        repo,
        "package.json",
        JSON.stringify({ name: "fx-lifecycle", type: "module" }),
      );
      write(repo, "src/a.ts", "export const live = 1;\n");
      write(
        repo,
        "intent.json",
        JSON.stringify({
          names: ["newHelper"],
          shapes: [],
          files: ["src/new-helper.ts"],
          dependencies: [],
          plannedTypeEscapes: [],
        }),
      );

      runAudit([
        "--root",
        repo,
        "--output",
        out,
        "--profile",
        "quick",
        "--pre-write",
        "--intent",
        intent,
        "--check-canon",
        "--sources",
        "all",
      ]);

      const manifest = readJson(path.join(out, "manifest.json"));
      const artifacts = manifest.artifactsProduced ?? [];

      expect(artifacts).toContain("pre-write-advisory.latest.json");
      expect(
        artifacts.some((name) => /^any-inventory\.pre\..+\.json$/.test(name)),
      ).toBe(true);
      expect(artifacts).toContain("canon-drift.json");
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  }, 30000);
});

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

describe("audit-repo scan range and self-audit exclusions split track", () => {
  it("O9. forwards user excludes and generated-artifact mode into producer evidence", () => {
    const repo = mkdtempSync(path.join(tmpdir(), "lumin-scan-range-exclude-"));
    const out = path.join(repo, "audit-out");

    try {
      write(
        repo,
        "package.json",
        JSON.stringify({
          name: "fx",
          type: "module",
          workspaces: ["apps/*", "packages/*"],
        }),
      );
      write(
        repo,
        "apps/web/package.json",
        JSON.stringify({ name: "web", type: "module" }),
      );
      write(
        repo,
        "packages/prisma/package.json",
        JSON.stringify({
          name: "@scope/prisma",
          type: "module",
          main: "index.ts",
          bin: { "prisma-enum-generator": "./run-enum-generator.js" },
          scripts: { generate: "prisma generate" },
          dependencies: { "@prisma/client": "1.0.0" },
        }),
      );
      write(repo, "src/a.ts", "export const live = 1;\n");
      write(repo, "packages/prisma/index.ts", "export const prismaRoot = 1;\n");
      write(
        repo,
        "apps/web/src/consumer.ts",
        "import { BookingStatus } from '@scope/prisma/enums';\n" +
          "export const status = BookingStatus.ACCEPTED;\n",
      );
      write(repo, "output/corpus/leak.ts", "export const leaked = 1;\n");

      runAudit([
        "--root",
        repo,
        "--output",
        out,
        "--profile",
        "quick",
        "--exclude",
        "output",
        "--generated-artifacts",
        "prepared",
      ]);

      const manifest = readJson(path.join(out, "manifest.json"));
      const symbols = readJson(path.join(out, "symbols.json"));
      const defFiles = Object.keys(symbols.defIndex ?? {});

      expect(defFiles.some((file) => file.startsWith("output/"))).toBe(false);
      expect(manifest.scanRange?.excludes).toContain("output");
      expect(manifest.generatedArtifacts).toMatchObject({
        mode: "prepared",
        executedGenerators: false,
      });
      expect(symbols.generatedConsumerBlindZones).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            specifier: "@scope/prisma/enums",
            mode: "prepared",
            staleStatus: "unknown",
            staleReason: "generator-input-hash-not-recorded",
          }),
        ]),
      );
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  }, 30000);

  it("O13. records maintainer self-audit auto-excludes and keeps mirror definitions out", () => {
    const repo = mkdtempSync(path.join(tmpdir(), "lumin-self-audit-excludes-"));
    const out = path.join(repo, "audit-out");

    try {
      write(
        repo,
        "package.json",
        JSON.stringify({
          name: "lumin-repo-lens-lab-scripts",
          type: "module",
        }),
      );
      mkdirSync(path.join(repo, "_lib"), { recursive: true });
      write(repo, "audit-repo.mjs", "export const rootEntrypoint = 1;\n");
      write(repo, "src/live.ts", "export const live = 1;\n");
      write(repo, "p6-corpus/leak.ts", "export const p6Leak = 1;\n");
      write(repo, "output/corpus/leak.ts", "export const outputLeak = 1;\n");
      write(
        repo,
        "skills/lumin-repo-lens-lab/_engine/leak.mjs",
        "export const engineLeak = 1;\n",
      );
      write(
        repo,
        "skills/lumin-repo-lens-lab/scripts/leak.mjs",
        "export const scriptLeak = 1;\n",
      );
      write(
        repo,
        "test-harness/lib/leak.mjs",
        "export const harnessLeak = 1;\n",
      );

      runAudit([
        "--root",
        repo,
        "--output",
        out,
        "--profile",
        "quick",
        "--production",
      ]);

      const manifest = readJson(path.join(out, "manifest.json"));
      const symbols = readJson(path.join(out, "symbols.json"));
      const defFiles = Object.keys(symbols.defIndex ?? {});

      expect(manifest.scanRange?.autoExcludes).toEqual(
        expect.arrayContaining([
          "p6-corpus",
          "output/corpus",
          "skills/lumin-repo-lens-lab/_engine",
          "skills/lumin-repo-lens-lab/scripts",
          "test-harness",
        ]),
      );
      expect(defFiles).toContain("src/live.ts");
      expect(
        defFiles.some(
          (file) =>
            file.startsWith("p6-corpus/") ||
            file.startsWith("output/corpus/") ||
            file.startsWith("skills/lumin-repo-lens-lab/_engine/") ||
            file.startsWith("skills/lumin-repo-lens-lab/scripts/") ||
            file.startsWith("test-harness/"),
        ),
      ).toBe(false);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  }, 30000);

  it("O11. treats production scan aliases as test-file exclusions across triage and symbols", () => {
    const flags = [
      ["--production"],
      ["--no-tests"],
      ["--exclude-tests"],
      ["--include-tests=false"],
    ];

    for (const flagArgs of flags) {
      const repo = mkdtempSync(path.join(tmpdir(), "lumin-scan-scope-"));
      const out = path.join(repo, "audit-out");

      try {
        write(
          repo,
          "package.json",
          JSON.stringify({ name: "fx-scope", type: "module" }),
        );
        write(repo, "src/a.ts", "export const prodOnly = 1;\n");
        write(repo, "src/a.test.ts", "export const testOnly = 1;\n");

        runAudit([
          "--root",
          repo,
          "--output",
          out,
          "--profile",
          "quick",
          ...flagArgs,
        ]);

        const manifest = readJson(path.join(out, "manifest.json"));
        const triage = readJson(path.join(out, "triage.json"));
        const symbols = readJson(path.join(out, "symbols.json"));
        const defFiles = Object.keys(symbols.defIndex ?? {});

        expect(manifest.scanRange).toMatchObject({
          includeTests: false,
          production: true,
        });
        expect(manifest.scanRange?.languages).toContain("ts");
        expect(triage.shape).toMatchObject({
          testFiles: 0,
          totalFiles: 1,
        });
        expect(defFiles.some((file) => file.includes(".test."))).toBe(false);
      } finally {
        rmSync(repo, { recursive: true, force: true });
      }
    }
  }, 60000);
});

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

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? ROOT,
    encoding: "utf8",
  });

  expect(
    result.status,
    [
      `${command} ${args.join(" ")}`,
      `stdout:\n${result.stdout}`,
      `stderr:\n${result.stderr}`,
    ].join("\n\n"),
  ).toBe(0);

  return result;
}

function readJson(file) {
  return JSON.parse(readFileSync(file, "utf8"));
}

describe("audit-repo full-profile staleness split track", () => {
  it("O10. keeps staleness and full-profile evidence when root is a git subdirectory", () => {
    const fixture = mkdtempSync(path.join(tmpdir(), "lumin-full-profile-"));
    const repoRoot = path.join(fixture, "workspace", "tool");
    const out = path.join(fixture, "audit-out");

    try {
      write(
        repoRoot,
        "package.json",
        JSON.stringify({ name: "fx-subdir", type: "module" }),
      );
      write(
        repoRoot,
        "src/a.ts",
        [
          "export const live = 1;",
          "export const unused = 2;",
          "export function dirtyHelper(a: any, b: any, c: any) { return a; }",
          "export interface DirtyShape { a: any; b: any; c: any }",
          "",
        ].join("\n"),
      );
      write(
        repoRoot,
        "src/web.ts",
        "export interface WebState { id: string; status: 'idle' | 'running' }\n",
      );
      write(
        repoRoot,
        "src/daemon.ts",
        "export type DaemonState = { status: 'idle' | 'running'; id: string };\n",
      );

      run("git", ["init"], { cwd: fixture });
      run("git", ["config", "user.email", "test@example.com"], {
        cwd: fixture,
      });
      run("git", ["config", "user.name", "Test User"], { cwd: fixture });
      run("git", ["add", "."], { cwd: fixture });
      run("git", ["commit", "-m", "fixture"], { cwd: fixture });

      run(
        NODE,
        [
          AUDIT_REPO,
          "--root",
          repoRoot,
          "--output",
          out,
          "--profile",
          "full",
          "--production",
        ],
        { cwd: ROOT },
      );

      const manifest = readJson(path.join(out, "manifest.json"));
      const checklist = readJson(path.join(out, "checklist-facts.json"));
      const summary = readFileSync(
        path.join(out, "audit-summary.latest.md"),
        "utf8",
      );
      const reviewPack = readFileSync(
        path.join(out, "audit-review-pack.latest.md"),
        "utf8",
      );
      const steps = manifest.commandsRun.map((command) => command.step);
      const artifacts = manifest.artifactsProduced ?? [];

      expect(steps).toContain("measure-staleness.mjs");
      expect(artifacts).toContain("staleness.json");

      expect(steps).toEqual(
        expect.arrayContaining([
          "build-call-graph.mjs",
          "check-barrel-discipline.mjs",
          "build-shape-index.mjs",
          "build-function-clone-index.mjs",
        ]),
      );
      expect(artifacts).toEqual(
        expect.arrayContaining([
          "call-graph.json",
          "barrels.json",
          "shape-index.json",
          "function-clones.json",
        ]),
      );

      expect(artifacts).toContain("audit-review-pack.latest.md");
      expect(manifest.reviewPack?.path).toMatch(
        /audit-review-pack\.latest\.md$/,
      );
      expect(reviewPack).toContain("Audit Review Pack");
      expect(reviewPack).toContain("never calls external APIs");
      expect(reviewPack).toContain("Claude Code");
      expect(reviewPack).toContain("main-controller artifact brief");
      expect(reviewPack).toContain("codebase-reading assignment");
      expect(reviewPack).toContain("Subagent rule:");
      expect(reviewPack).not.toContain(
        "paste one whole lane into each chosen reviewer",
      );

      expect(checklist.B1B2_shape_drift?.exactDuplicateGroups).toBe(1);
      expect(summary).toContain("Shape drift: exact groups 1");
      expect(summary).toContain("shape-index.json");

      expect(
        typeof checklist.B1_duplicate_implementation?.structureGroupCandidates,
      ).toBe("number");
      expect(
        typeof checklist.B1_duplicate_implementation?.nearFunctionCandidates,
      ).toBe("number");
      expect(summary).toContain("JS/TS function clone cues:");
      expect(summary).toContain("near-function cues");
      expect(summary).toContain("function-clones.json");
      expect(reviewPack).toContain("JS/TS function clone cues:");
      expect(reviewPack).toContain("near-function cues");

      expect(summary).toContain("Exported any-contamination:");
      expect(summary).toContain("symbols.json.typeOwnersByIdentity");
      expect(reviewPack).toContain("Identity-level anyContamination:");
      expect(reviewPack).toContain("Inspect symbols.json owner maps");
    } finally {
      rmSync(fixture, { recursive: true, force: true });
    }
  }, 120000);
});

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

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");

function write(root, rel, text) {
  const p = path.join(root, rel);
  mkdirSync(path.dirname(p), { recursive: true });
  writeFileSync(p, text);
}

function runFixture() {
  const root = mkdtempSync(path.join(tmpdir(), "vitest-call-parse-root-"));
  const out = mkdtempSync(path.join(tmpdir(), "vitest-call-parse-out-"));
  try {
    write(
      root,
      "src/good.mjs",
      ["export function live() {", "  return 1;", "}", "live();", ""].join(
        "\n",
      ),
    );
    write(
      root,
      "src/bad.mjs",
      ["export function broken() {", "  if (", "}", ""].join("\n"),
    );

    execFileSync(
      process.execPath,
      ["build-call-graph.mjs", "--root", root, "--output", out],
      {
        cwd: REPO_ROOT,
        stdio: ["ignore", "pipe", "pipe"],
      },
    );

    return JSON.parse(readFileSync(path.join(out, "call-graph.json"), "utf8"));
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

describe("call graph parse-error diagnostics", () => {
  it("T1-T3. marks the artifact incomplete and emits parse-error warning counts", () => {
    const artifact = runFixture();
    const warning = (artifact.meta?.warnings ?? []).find(
      (w) => w.code === "call-graph-parse-errors",
    );

    expect(artifact.meta?.complete).toBe(false);
    expect(artifact.meta?.parseErrors).toBe(1);
    expect(warning?.count).toBe(1);
  });

  it("T4-T5. preserves malformed file and parser message evidence", () => {
    const artifact = runFixture();
    const warning = (artifact.meta?.warnings ?? []).find(
      (w) => w.code === "call-graph-parse-errors",
    );

    expect(warning?.files?.[0]?.file).toBe("src/bad.mjs");
    expect(warning?.files?.[0]?.message).toEqual(expect.any(String));
    expect(warning.files[0].message.length).toBeGreaterThan(0);
  });
});

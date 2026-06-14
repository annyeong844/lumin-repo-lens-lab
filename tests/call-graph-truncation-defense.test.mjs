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
  const root = mkdtempSync(path.join(tmpdir(), "vitest-pcef-call-root-"));
  const out = mkdtempSync(path.join(tmpdir(), "vitest-pcef-call-out-"));
  try {
    const exported = [];
    const imports = [];
    const calls = [];
    for (let i = 0; i < 102; i++) {
      exported.push(`export function fn${i}() { return ${i}; }`);
      imports.push(`fn${i}`);
      calls.push(`fn${i}();`);
    }
    write(root, "src/lib.ts", `${exported.join("\n")}\n`);
    write(
      root,
      "src/consumer.ts",
      [`import { ${imports.join(", ")} } from './lib';`, ...calls, ""].join(
        "\n",
      ),
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

describe("call graph truncation defense", () => {
  it("T1-T3. keeps topCallees as display slice while full fan-in retains hidden identities", () => {
    const artifact = runFixture();
    const topNames = new Set((artifact.topCallees ?? []).map((c) => c.name));

    expect(artifact.topCallees ?? []).toHaveLength(100);
    expect(topNames.has("fn101")).toBe(false);
    expect(artifact.callFanInByIdentity?.["src/lib.ts::fn101"]).toBe(1);
  });
});

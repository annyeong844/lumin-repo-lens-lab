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

function runCallGraph(files) {
  const root = mkdtempSync(path.join(tmpdir(), "vitest-call-bounded-root-"));
  const out = mkdtempSync(path.join(tmpdir(), "vitest-call-bounded-out-"));
  try {
    for (const [rel, text] of Object.entries(files)) {
      write(root, rel, text);
    }
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

describe("call graph bounded member-call evidence", () => {
  it("B1. maps default exported object member calls to referenced functions", () => {
    const artifact = runCallGraph({
      "src/lib.ts": [
        "export function actualRun() { return 1; }",
        "export default { run: actualRun };",
        "",
      ].join("\n"),
      "src/consumer.ts": ["import api from './lib';", "api.run();", ""].join(
        "\n",
      ),
    });

    expect(artifact.callFanInByIdentity?.["src/lib.ts::actualRun"]).toBe(1);
    expect(artifact.meta?.supports?.boundedMemberCallResolution).toBe(true);
  });

  it("B2-B3. maps named exported object member calls only for known function properties", () => {
    const artifact = runCallGraph({
      "src/lib.ts": [
        "export function actualRun() { return 1; }",
        "export const count = 1;",
        "export const tools = { run: actualRun, inline() { return 2; }, value: 1, count };",
        "",
      ].join("\n"),
      "src/consumer.ts": [
        "import { tools } from './lib';",
        "tools.run();",
        "tools.inline();",
        "tools.value();",
        "tools.count();",
        "",
      ].join("\n"),
    });

    expect(artifact.callFanInByIdentity?.["src/lib.ts::actualRun"]).toBe(1);
    expect(artifact.callFanInByIdentity?.["src/lib.ts::inline"]).toBe(1);
    expect(artifact.callFanInByIdentity?.["src/lib.ts::value"]).toBeUndefined();
    expect(artifact.callFanInByIdentity?.["src/lib.ts::count"]).toBe(0);
    expect(artifact.boundedOutMemberCallsByFile?.["src/consumer.ts"]).toBe(2);
    expect(artifact.memberCallsByFile?.["src/consumer.ts"]).toBe(4);
  });

  it("B4. bounds out depth-2 imported object member calls", () => {
    const artifact = runCallGraph({
      "src/lib.ts": [
        "export function actualRun() { return 1; }",
        "export default { run: actualRun };",
        "",
      ].join("\n"),
      "src/consumer.ts": [
        "import api from './lib';",
        "api.run.deep();",
        "",
      ].join("\n"),
    });

    expect(artifact.callFanInByIdentity?.["src/lib.ts::actualRun"]).toBe(0);
    expect(artifact.boundedOutMemberCallsByFile?.["src/consumer.ts"]).toBe(1);
    expect(artifact.memberCallsByFile?.["src/consumer.ts"]).toBe(1);
  });
});

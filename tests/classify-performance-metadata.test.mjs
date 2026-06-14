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

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const TEST_TIMEOUT = 60_000;

function fresh(prefix) {
  return mkdtempSync(path.join(tmpdir(), prefix));
}

function write(root, rel, content) {
  const target = path.join(root, rel);
  mkdirSync(path.dirname(target), { recursive: true });
  writeFileSync(target, content);
}

function run(script, args) {
  execFileSync(process.execPath, [path.join(ROOT, script), ...args], {
    cwd: ROOT,
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function readClassifyArtifact(output) {
  return JSON.parse(
    readFileSync(path.join(output, "dead-classify.json"), "utf8"),
  );
}

function cleanup(...roots) {
  for (const root of roots) {
    if (!root) continue;
    rmSync(root, { recursive: true, force: true });
  }
}

function writeMainFixture(root) {
  write(
    root,
    "package.json",
    JSON.stringify({ name: "classify-perf", type: "module", private: true }),
  );
  write(
    root,
    "src/dead.ts",
    [
      "export const Alpha = 1;",
      "export const Beta = 2;",
      "const gamma = Alpha + Beta;",
      "",
    ].join("\n"),
  );
  write(
    root,
    "src/text-zero.ts",
    ["export const Gamma = 3;", "export const Delta = 4;", ""].join("\n"),
  );
}

function writeTextZeroFixture(root) {
  write(
    root,
    "package.json",
    JSON.stringify({
      name: "classify-perf-text-zero",
      type: "module",
      private: true,
    }),
  );
  write(
    root,
    "src/text-zero.ts",
    ["export const Gamma = 3;", "export const Delta = 4;", ""].join("\n"),
  );
}

function buildAndClassify(root, output, classifyArgs = []) {
  run("build-symbol-graph.mjs", ["--root", root, "--output", output]);
  run("classify-dead-exports.mjs", [
    "--root",
    root,
    "--output",
    output,
    ...classifyArgs,
  ]);
  return readClassifyArtifact(output);
}

describe("classify-dead-exports performance metadata", () => {
  describe("default performance metadata", () => {
    let root;
    let output;
    let artifact;
    let performance;

    beforeAll(() => {
      root = fresh("vitest-classify-perf-fx-");
      output = fresh("vitest-classify-perf-out-");
      writeMainFixture(root);
      artifact = buildAndClassify(root, output);
      performance = artifact.summary?.performance;
    }, TEST_TIMEOUT);

    afterAll(() => cleanup(root, output));

    it("carries performance metadata in the classify summary", () => {
      expect(performance).toEqual(expect.any(Object));
    });

    it("records processed dead candidates", () => {
      expect(performance?.deadCandidatesProcessed).toBe(4);
    });

    it("AST-counts same-file candidates through one file batch", () => {
      expect(performance?.astFilesParsed).toBe(1);
    });

    it("does not apply candidate caps by default", () => {
      expect(performance?.candidateLimitApplied).toBe(false);
    });

    it("keeps file-size degradation opt-in rather than default policy", () => {
      expect(performance?.maxFileBytes).toBe(0);
      expect(performance?.astFilesSkippedBySize).toBe(0);
    });

    it("skips AST for text-zero candidates without degrading accuracy", () => {
      expect(performance?.textZeroCandidates).toBe(2);
      expect(performance?.textZeroFiles).toBe(1);
    });

    it("caches provenance work per file rather than repeating per symbol", () => {
      expect(performance?.provenanceCacheEntries).toBe(2);
    });
  });

  it(
    "finishes all-text-zero batches without parsing candidate files",
    () => {
      const root = fresh("vitest-classify-perf-text-zero-fx-");
      const output = fresh("vitest-classify-perf-text-zero-out-");
      try {
        writeTextZeroFixture(root);
        const artifact = buildAndClassify(root, output);
        const performance = artifact.summary?.performance;

        expect(performance?.textZeroCandidates).toBe(2);
        expect(performance?.astFilesParsed).toBe(0);
      } finally {
        cleanup(root, output);
      }
    },
    TEST_TIMEOUT,
  );

  describe("candidate limit degradation", () => {
    let root;
    let output;
    let artifact;
    let performance;

    beforeAll(() => {
      root = fresh("vitest-classify-perf-limited-fx-");
      output = fresh("vitest-classify-perf-limited-out-");
      writeMainFixture(root);
      artifact = buildAndClassify(root, output, [
        "--classify-candidate-limit",
        "1",
      ]);
      performance = artifact.summary?.performance;
    }, TEST_TIMEOUT);

    afterAll(() => cleanup(root, output));

    it("marks the classify artifact incomplete when a candidate cap applies", () => {
      expect(artifact.summary?.incomplete).toBe(true);
      expect(performance?.candidateLimitApplied).toBe(true);
    });

    it("records total versus processed candidate counts under the cap", () => {
      expect(performance?.deadCandidatesTotal).toBe(4);
      expect(performance?.deadCandidatesProcessed).toBe(1);
    });
  });

  describe("time budget degradation", () => {
    let root;
    let output;
    let artifact;

    beforeAll(() => {
      root = fresh("vitest-classify-perf-budget-fx-");
      output = fresh("vitest-classify-perf-budget-out-");
      writeMainFixture(root);
      artifact = buildAndClassify(root, output, [
        "--classify-time-budget-ms",
        "1",
      ]);
    }, TEST_TIMEOUT);

    afterAll(() => cleanup(root, output));

    it("marks the classify artifact incomplete when the time budget is exceeded", () => {
      expect(artifact.summary?.incomplete).toBe(true);
      expect(artifact.summary?.performance?.timeBudgetExceeded).toBe(true);
    });

    it("materializes time-budgeted candidates as degraded proposals", () => {
      expect(artifact.proposal_DEGRADED_unprocessed).toEqual(expect.any(Array));
      expect(artifact.proposal_DEGRADED_unprocessed.length).toBeGreaterThan(0);
    });
  });

  it(
    "degrades oversized candidate files instead of AST-counting them",
    () => {
      const root = fresh("vitest-classify-perf-sized-fx-");
      const output = fresh("vitest-classify-perf-sized-out-");
      try {
        writeMainFixture(root);
        const artifact = buildAndClassify(root, output, [
          "--classify-max-file-bytes",
          "10",
        ]);

        expect(artifact.summary?.performance?.astFilesSkippedBySize).toBe(2);
        expect(artifact.proposal_DEGRADED_unprocessed).toHaveLength(4);
      } finally {
        cleanup(root, output);
      }
    },
    TEST_TIMEOUT,
  );
});

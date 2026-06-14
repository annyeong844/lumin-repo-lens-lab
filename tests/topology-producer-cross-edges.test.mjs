import { spawnSync } from "node:child_process";
import { chmodSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const AUDIT_CLI = path.join(REPO_ROOT, "audit-repo.mjs");

function runTopologyWithStderr(fixture, { output = fixture.output, args = [] } = {}) {
  const result = spawnSync(
    process.execPath,
    [
      path.join(REPO_ROOT, "measure-topology.mjs"),
      "--root",
      fixture.root,
      "--output",
      output,
      ...args,
    ],
    { encoding: "utf8" },
  );
  if (result.status !== 0) {
    throw new Error(
      [
        `measure-topology exited with ${result.status}`,
        result.stdout,
        result.stderr,
      ].join("\n"),
    );
  }
  return {
    stderr: result.stderr,
    topology: JSON.parse(
      readFileSync(path.join(output, "topology.json"), "utf8"),
    ),
  };
}

function runTopology(fixture) {
  return runTopologyWithStderr(fixture).topology;
}

function runAudit(fixture, output, args = []) {
  const result = spawnSync(
    process.execPath,
    [
      AUDIT_CLI,
      "--root",
      fixture.root,
      "--output",
      output,
      "--profile",
      "quick",
      ...args,
    ],
    { encoding: "utf8" },
  );
  if (result.status !== 0) {
    throw new Error(
      [
        `audit-repo exited with ${result.status}`,
        result.stdout,
        result.stderr,
      ].join("\n"),
    );
  }
  return JSON.parse(readFileSync(path.join(output, "topology.json"), "utf8"));
}

function createCrossEdgeFixture() {
  const fixture = createTempRepoFixture({
    prefix: "vitest-topology-cross-edges-",
    packageJson: { name: "tpcx-fx", type: "module" },
  });

  fixture.write("util/helper.mjs", "export function helper() { return 1; }\n");
  fixture.write(
    "lib/a.mjs",
    [
      "import { helper } from '../util/helper.mjs';",
      "export function a() { return helper(); }",
      "",
    ].join("\n"),
  );
  fixture.write(
    "lib/b.mjs",
    [
      "import { helper } from '../util/helper.mjs';",
      "export function b() { return helper() + 1; }",
      "",
    ].join("\n"),
  );
  fixture.write(
    "app/main.mjs",
    [
      "import { a } from '../lib/a.mjs';",
      "import { helper } from '../util/helper.mjs';",
      "export function main() { return a() + helper(); }",
      "",
    ].join("\n"),
  );

  return fixture;
}

function writeFakeRustTopologySidecar(dir) {
  mkdirSync(dir, { recursive: true });
  const script = path.join(dir, "sidecar.mjs");
  const command = path.join(
    dir,
    process.platform === "win32" ? "sidecar.cmd" : "sidecar.sh",
  );
  writeFileSync(
    script,
    `let input = "";
process.stdin.on("data", (chunk) => { input += chunk; });
process.stdin.on("end", () => {
  const req = JSON.parse(input);
  process.stdout.write(JSON.stringify({
    schemaVersion: 1,
    policyVersion: req.policyVersion,
    files: req.files.map((file) => ({
      file,
      ok: true,
      loc: 1,
      edges: [],
      risk: []
    })),
    timing: { files: req.files.length, elapsedMs: 1 }
  }));
});
`,
    "utf8",
  );
  if (process.platform === "win32") {
    writeFileSync(command, `@echo off\r\n"${process.execPath}" "%~dp0\\sidecar.mjs"\r\n`, "utf8");
  } else {
    writeFileSync(command, `#!/usr/bin/env sh\n"${process.execPath}" "$(dirname "$0")/sidecar.mjs"\n`, "utf8");
    chmodSync(command, 0o755);
  }
  return command;
}

describe("topology producer cross-submodule edge artifact", () => {
  it("emits full structured crossSubmoduleEdges while preserving legacy display top shape", () => {
    const fixture = createCrossEdgeFixture();
    try {
      const topology = runTopology(fixture);

      expect(Array.isArray(topology.crossSubmoduleEdges)).toBe(true);
      expect(
        topology.crossSubmoduleEdges.every(
          (edge) =>
            edge &&
            typeof edge.from === "string" &&
            edge.from.length > 0 &&
            typeof edge.to === "string" &&
            edge.to.length > 0 &&
            typeof edge.count === "number" &&
            edge.count >= 1,
        ),
      ).toBe(true);
      expect(
        topology.crossSubmoduleEdges.every(
          (edge) => !("edge" in edge) && !edge.from.includes("→"),
        ),
      ).toBe(true);

      const pairs = new Set(
        topology.crossSubmoduleEdges.map((edge) => `${edge.from}→${edge.to}`),
      );
      expect(topology.crossSubmoduleEdges).toHaveLength(3);
      expect(pairs).toEqual(new Set(["lib→util", "app→lib", "app→util"]));
      expect(
        topology.crossSubmoduleEdges.find(
          (edge) => edge.from === "lib" && edge.to === "util",
        )?.count,
      ).toBe(2);

      expect(Array.isArray(topology.crossSubmoduleTop)).toBe(true);
      expect(
        topology.crossSubmoduleTop.every(
          (edge) =>
            edge &&
            typeof edge.edge === "string" &&
            edge.edge.includes(" → ") &&
            typeof edge.count === "number",
        ),
      ).toBe(true);
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("emits crossSubmoduleEdges as an empty array when no cross edges exist", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-cross-edges-zero-",
      packageJson: { name: "tpcx-zero-fx", type: "module" },
    });
    try {
      fixture.write("lib/a.mjs", "export const a = 1;\n");
      fixture.write(
        "lib/b.mjs",
        "import { a } from './a.mjs'; export const b = a + 1;\n",
      );

      const topology = runTopology(fixture);

      expect("crossSubmoduleEdges" in topology).toBe(true);
      expect(topology.crossSubmoduleEdges).toEqual([]);
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("skips Python and Go availability probes for JS-only scans", () => {
    const fixture = createCrossEdgeFixture();
    try {
      const { stderr } = runTopologyWithStderr(fixture);

      expect(stderr).toContain("python=skipped, 0 .py");
      expect(stderr).toContain("go=skipped, 0 .go");
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("records capped scanner fallback examples by risk reason", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-scanner-fallback-examples-",
      packageJson: { name: "scanner-fallback-fx", type: "module" },
    });
    try {
      fixture.write(
        "lib/uses-require.mjs",
        [
          "const fs = require('node:fs');",
          "export const value = fs.existsSync('.');",
          "",
        ].join("\n"),
      );

      const topology = runTopology(fixture);

      expect(topology.summary.performance.scannerRiskCounts).toEqual({
        "require-call": 1,
      });
      expect(topology.summary.performance.scannerFallbackExamples).toEqual({
        "require-call": ["lib/uses-require.mjs"],
      });
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("reuses topology cache across output directories through cache-root", () => {
    const fixture = createCrossEdgeFixture();
    try {
      const cacheRoot = fixture.mkdir("cache-root");
      const coldOutput = fixture.mkdir("topology-cold");
      const warmOutput = fixture.mkdir("topology-warm");

      const cold = runTopologyWithStderr(fixture, {
        output: coldOutput,
        args: ["--cache-root", cacheRoot],
      }).topology;
      const warm = runTopologyWithStderr(fixture, {
        output: warmOutput,
        args: ["--cache-root", cacheRoot],
      }).topology;

      expect(cold.summary.performance.changedFiles).toBeGreaterThan(0);
      expect(cold.summary.performance.unchangedFiles).toBe(0);
      expect(warm.summary.performance.changedFiles).toBe(0);
      expect(warm.summary.performance.unchangedFiles).toBeGreaterThan(0);
      expect(warm.summary.performance.scannerFilesAttempted).toBe(0);
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("forwards audit-repo cache-root to measure-topology", () => {
    const fixture = createCrossEdgeFixture();
    try {
      const cacheRoot = fixture.mkdir("audit-cache-root");
      const coldOutput = fixture.mkdir("audit-cold");
      const warmOutput = fixture.mkdir("audit-warm");

      const cold = runAudit(fixture, coldOutput, ["--cache-root", cacheRoot]);
      const warm = runAudit(fixture, warmOutput, ["--cache-root", cacheRoot]);

      expect(cold.summary.performance.changedFiles).toBeGreaterThan(0);
      expect(warm.summary.performance.changedFiles).toBe(0);
      expect(warm.summary.performance.unchangedFiles).toBeGreaterThan(0);
      expect(warm.summary.performance.scannerFilesAttempted).toBe(0);
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("records Rust topology scanner comparison metadata in compare mode", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-compare-",
      packageJson: { name: "rust-compare-fx", type: "module" },
    });
    try {
      fixture.write("src/empty.mjs", "export const value = 1;\n");
      const sidecar = writeFakeRustTopologySidecar(fixture.mkdir("fake-sidecar"));

      const topology = runTopologyWithStderr(fixture, {
        args: [
          "--no-incremental",
          "--rust-topology-scanner",
          "compare",
          "--rust-topology-scanner-bin",
          sidecar,
          "--rust-topology-timeout-ms",
          "1000",
        ],
      }).topology;

      expect(topology.meta.rustTopologyScanner).toMatchObject({
        attempted: true,
        mode: "compare",
        status: "matched",
        timeoutMs: 1000,
        mismatches: 0,
      });
      expect(topology.meta.rustTopologyScanner.filesCompared).toBeGreaterThan(0);
      expect(topology.meta.rustTopologyScanner.sidecarTiming).toMatchObject({
        files: topology.meta.rustTopologyScanner.filesCompared,
        elapsedMs: 1,
      });
    } finally {
      fixture.cleanup();
    }
  }, 30000);
});

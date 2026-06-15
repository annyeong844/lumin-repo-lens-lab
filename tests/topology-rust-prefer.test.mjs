import { spawnSync } from "node:child_process";
import { chmodSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { MODULE_EDGE_SCANNER_POLICY_VERSION } from "../_lib/js-module-edge-scanner.mjs";
import { hashFileSha256 } from "../_lib/rust-topology-prefer.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const FAKE_RUST_SIDECAR_COMMIT = "87116819c23d1e1adfbfca5def44552856e4f464";

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
      loc: 2,
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

function writeWrongLocRustTopologySidecar(dir) {
  mkdirSync(dir, { recursive: true });
  const script = path.join(dir, "sidecar-wrong-loc.mjs");
  const command = path.join(
    dir,
    process.platform === "win32" ? "sidecar-wrong-loc.cmd" : "sidecar-wrong-loc.sh",
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
      loc: 999,
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
    writeFileSync(command, `@echo off\r\n"${process.execPath}" "%~dp0\\sidecar-wrong-loc.mjs"\r\n`, "utf8");
  } else {
    writeFileSync(command, `#!/usr/bin/env sh\n"${process.execPath}" "$(dirname "$0")/sidecar-wrong-loc.mjs"\n`, "utf8");
    chmodSync(command, 0o755);
  }
  return command;
}

function cleanQuorumEvidence() {
  const requiredCorpora = [
    "geulbat-phase1",
    "lab-self",
    "stable-source-clean",
    "nuxt-main",
  ];
  const policyVersion = MODULE_EDGE_SCANNER_POLICY_VERSION;
  const rustSidecarSourceCommit = FAKE_RUST_SIDECAR_COMMIT;
  const runs = Object.fromEntries(
    requiredCorpora.map((corpus) => [
      corpus,
      [0, 1, 2].map((index) => ({
        labSourceCommit: `lab-${index}`,
        rustSidecarSourceCommit,
        rustSidecarBinary: "experiments/rust-sidecar/topology-scanner/target/release/lumin-topology-scanner.exe",
        command: `node measure-topology.mjs --rust-topology-prefer-gate-corpus ${corpus}`,
        corpusRoot: `C:/corpora/${corpus}`,
        cacheMode: "no-incremental",
        fileCount: 1,
        filesCompared: 1,
        mismatches: 0,
        commandWallElapsedMs: 10 + index,
        scannerBridgeElapsedMs: 2 + index,
        sidecarElapsedMs: 1,
        sidecarStatus: "matched",
        policyVersion,
        machineOs: "Microsoft Windows NT 10.0.26200.0",
        collector: {
          workingTreeClean: true,
          sourceDirty: false,
          labWorkingTreeClean: true,
          rustSidecarWorkingTreeClean: true,
        },
      })),
    ]),
  );
  return {
    schemaVersion: 1,
    requiredCorpora,
    policyVersion,
    rustSidecarSourceCommit,
    runs,
  };
}

function normalizeTopologyForGateContract(topology) {
  const normalized = structuredClone(topology);
  delete normalized.meta.rustTopologyPreferGate;
  normalized.meta.generated = "<generated>";
  if (normalized.meta.rustTopologyScanner) {
    normalized.meta.rustTopologyScanner.elapsedMs = "<elapsed>";
  }
  normalized.summary.performance.scannerMs = "<scannerMs>";
  return normalized;
}

describe("topology Rust scanner and prefer integration", () => {
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

  it("records dry-run Rust prefer gate metadata without changing topology artifacts", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-prefer-gate-",
      packageJson: { name: "rust-prefer-gate-fx", type: "module" },
    });
    try {
      fixture.write("src/empty.mjs", "export const value = 1;\n");
      const sidecar = writeFakeRustTopologySidecar(path.join(fixture.output, "fake-sidecar"));
      const quorumPath = path.join(fixture.root, "rust-topology-prefer-quorum.json");
      writeFileSync(quorumPath, JSON.stringify(cleanQuorumEvidence(), null, 2));

      const commonArgs = [
        "--no-incremental",
        "--rust-topology-scanner",
        "compare",
        "--rust-topology-scanner-bin",
        sidecar,
        "--rust-topology-timeout-ms",
        "1000",
      ];
      const gateOffOutput = fixture.mkdir("gate-off");
      const gateOnOutput = fixture.mkdir("gate-on");

      const gateOff = runTopologyWithStderr(fixture, {
        output: gateOffOutput,
        args: commonArgs,
      }).topology;
      const gateOn = runTopologyWithStderr(fixture, {
        output: gateOnOutput,
        args: [
          ...commonArgs,
          "--rust-topology-prefer-gate",
          "--rust-topology-prefer-gate-corpus",
          "lab-self",
          "--rust-topology-prefer-quorum",
          quorumPath,
        ],
      }).topology;

      expect(gateOn.meta.rustTopologyPreferGate).toMatchObject({
        status: "eligible",
        mode: "compare",
        scope: "run",
        preferEnabled: false,
        jsRemainsOracle: true,
        reason: "all-required-corpora-matched",
        currentCorpus: "lab-self",
        currentCorpusSource: "cli",
        cacheMode: "no-incremental",
        mismatches: 0,
        filesCompared: 1,
        sidecarStatus: "matched",
      });
      expect(normalizeTopologyForGateContract(gateOn)).toEqual(
        normalizeTopologyForGateContract(gateOff),
      );
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("records blocked Rust prefer gate metadata when quorum evidence is missing", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-prefer-gate-missing-quorum-",
      packageJson: { name: "rust-prefer-gate-missing-quorum-fx", type: "module" },
    });
    try {
      fixture.write("src/empty.mjs", "export const value = 1;\n");
      const sidecar = writeFakeRustTopologySidecar(path.join(fixture.output, "fake-sidecar"));
      const missingQuorumPath = path.join(fixture.root, "missing-quorum.json");

      const topology = runTopologyWithStderr(fixture, {
        args: [
          "--no-incremental",
          "--rust-topology-scanner",
          "compare",
          "--rust-topology-scanner-bin",
          sidecar,
          "--rust-topology-timeout-ms",
          "5000",
          "--rust-topology-prefer-gate",
          "--rust-topology-prefer-gate-corpus",
          "lab-self",
          "--rust-topology-prefer-quorum",
          missingQuorumPath,
        ],
      }).topology;

      expect(topology.meta.rustTopologyScanner.status).toBe("matched");
      expect(topology.meta.rustTopologyPreferGate).toMatchObject({
        status: "blocked-corpus-quorum",
        reason: "quorum-evidence-missing",
        currentCorpus: "lab-self",
        preferEnabled: false,
        jsRemainsOracle: true,
      });
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("uses Rust for explicit prefer when gate is eligible and artifact guard passes", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-prefer-happy-",
      packageJson: { name: "rust-prefer-happy-fx", type: "module" },
    });
    try {
      fixture.write("src/empty.mjs", "export const value = 1;\n");
      const sidecar = writeFakeRustTopologySidecar(path.join(fixture.output, "fake-sidecar"));
      const quorumPath = path.join(fixture.root, "rust-topology-prefer-quorum.json");
      writeFileSync(quorumPath, JSON.stringify(cleanQuorumEvidence(), null, 2));

      const topology = runTopologyWithStderr(fixture, {
        args: [
          "--no-incremental",
          "--clear-incremental-cache",
          "--rust-topology-scanner",
          "prefer",
          "--rust-topology-scanner-bin",
          sidecar,
          "--rust-topology-timeout-ms",
          "1000",
          "--rust-sidecar-source-commit",
          FAKE_RUST_SIDECAR_COMMIT,
          "--rust-sidecar-binary-sha256",
          hashFileSha256(sidecar),
          "--rust-topology-prefer-gate",
          "--rust-topology-prefer-gate-corpus",
          "lab-self",
          "--rust-topology-prefer-quorum",
          quorumPath,
        ],
      }).topology;

      expect(topology.meta.rustTopologyPrefer).toMatchObject({
        requested: true,
        mode: "prefer",
        status: "used-rust",
        usedRust: true,
        fallbackUsed: false,
        reason: "gate-eligible-artifact-guard-passed",
        rustSidecarSourceCommit: FAKE_RUST_SIDECAR_COMMIT,
      });
      expect(topology.summary.files).toBe(1);
      expect(topology.edges).toEqual([]);
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("falls back to JS topology when the prefer sidecar binary is missing", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-prefer-missing-binary-",
      packageJson: { name: "rust-prefer-missing-binary-fx", type: "module" },
    });
    try {
      fixture.write("src/dep.mjs", "export const dep = 1;\n");
      fixture.write("src/main.mjs", "import { dep } from './dep.mjs'; export const value = dep;\n");
      const quorumPath = path.join(fixture.root, "rust-topology-prefer-quorum.json");
      writeFileSync(quorumPath, JSON.stringify(cleanQuorumEvidence(), null, 2));

      const topology = runTopologyWithStderr(fixture, {
        args: [
          "--no-incremental",
          "--clear-incremental-cache",
          "--rust-topology-scanner",
          "prefer",
          "--rust-topology-scanner-bin",
          path.join(fixture.root, "missing-sidecar.exe"),
          "--rust-sidecar-source-commit",
          FAKE_RUST_SIDECAR_COMMIT,
          "--rust-topology-prefer-gate",
          "--rust-topology-prefer-gate-corpus",
          "lab-self",
          "--rust-topology-prefer-quorum",
          quorumPath,
        ],
      }).topology;

      expect(topology.meta.rustTopologyPrefer).toMatchObject({
        status: "fallback-js",
        usedRust: false,
        fallbackUsed: true,
        reason: "blocked-binary-not-found",
      });
      expect(topology.edges.length).toBeGreaterThan(0);
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("falls back visibly when prefer quorum evidence is missing", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-prefer-missing-quorum-",
      packageJson: { name: "rust-prefer-missing-quorum-fx", type: "module" },
    });
    try {
      fixture.write("src/empty.mjs", "export const value = 1;\n");
      const sidecar = writeFakeRustTopologySidecar(path.join(fixture.output, "fake-sidecar"));

      const topology = runTopologyWithStderr(fixture, {
        args: [
          "--no-incremental",
          "--clear-incremental-cache",
          "--rust-topology-scanner",
          "prefer",
          "--rust-topology-scanner-bin",
          sidecar,
          "--rust-sidecar-source-commit",
          FAKE_RUST_SIDECAR_COMMIT,
          "--rust-sidecar-binary-sha256",
          hashFileSha256(sidecar),
          "--rust-topology-prefer-gate",
          "--rust-topology-prefer-gate-corpus",
          "lab-self",
          "--rust-topology-prefer-quorum",
          path.join(fixture.root, "missing-quorum.json"),
        ],
      }).topology;

      expect(topology.meta.rustTopologyPreferGate).toMatchObject({
        status: "blocked-corpus-quorum",
        reason: "quorum-evidence-missing",
      });
      expect(topology.meta.rustTopologyPrefer).toMatchObject({
        status: "fallback-js",
        reason: "blocked-quorum-missing",
        usedRust: false,
      });
      expect(topology.summary.files).toBe(1);
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("falls back when the M3 prefer gate is not eligible", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-prefer-ineligible-gate-",
      packageJson: { name: "rust-prefer-ineligible-gate-fx", type: "module" },
    });
    try {
      fixture.write("src/empty.mjs", "export const value = 1;\n");
      const sidecar = writeFakeRustTopologySidecar(path.join(fixture.output, "fake-sidecar"));
      const quorum = cleanQuorumEvidence();
      quorum.runs["nuxt-main"] = quorum.runs["nuxt-main"].slice(0, 2);
      const quorumPath = path.join(fixture.root, "rust-topology-prefer-quorum.json");
      writeFileSync(quorumPath, JSON.stringify(quorum, null, 2));

      const topology = runTopologyWithStderr(fixture, {
        args: [
          "--no-incremental",
          "--clear-incremental-cache",
          "--rust-topology-scanner",
          "prefer",
          "--rust-topology-scanner-bin",
          sidecar,
          "--rust-sidecar-source-commit",
          FAKE_RUST_SIDECAR_COMMIT,
          "--rust-sidecar-binary-sha256",
          hashFileSha256(sidecar),
          "--rust-topology-prefer-gate",
          "--rust-topology-prefer-gate-corpus",
          "lab-self",
          "--rust-topology-prefer-quorum",
          quorumPath,
        ],
      }).topology;

      expect(topology.meta.rustTopologyPreferGate.status).toBe("blocked-corpus-quorum");
      expect(topology.meta.rustTopologyPrefer).toMatchObject({
        status: "fallback-js",
        reason: "blocked-gate-ineligible",
      });
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("falls back when scanner parity passes but the artifact guard catches LOC drift", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-prefer-artifact-guard-",
      packageJson: { name: "rust-prefer-artifact-guard-fx", type: "module" },
    });
    try {
      fixture.write("src/empty.mjs", "export const value = 1;\n");
      const sidecar = writeWrongLocRustTopologySidecar(path.join(fixture.output, "fake-sidecar"));
      const quorumPath = path.join(fixture.root, "rust-topology-prefer-quorum.json");
      writeFileSync(quorumPath, JSON.stringify(cleanQuorumEvidence(), null, 2));

      const topology = runTopologyWithStderr(fixture, {
        args: [
          "--no-incremental",
          "--clear-incremental-cache",
          "--rust-topology-scanner",
          "prefer",
          "--rust-topology-scanner-bin",
          sidecar,
          "--rust-sidecar-source-commit",
          FAKE_RUST_SIDECAR_COMMIT,
          "--rust-sidecar-binary-sha256",
          hashFileSha256(sidecar),
          "--rust-topology-prefer-gate",
          "--rust-topology-prefer-gate-corpus",
          "lab-self",
          "--rust-topology-prefer-quorum",
          quorumPath,
        ],
      }).topology;

      expect(topology.meta.rustTopologyScanner.status).toBe("matched");
      expect(topology.meta.rustTopologyPrefer).toMatchObject({
        status: "fallback-js",
        reason: "blocked-artifact-contract",
        usedRust: false,
      });
      expect(topology.nodes["src/empty.mjs"].loc).toBe(2);
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("falls back without running Rust when prefer is requested with incremental cache coverage", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-prefer-cache-mode-",
      packageJson: { name: "rust-prefer-cache-mode-fx", type: "module" },
    });
    try {
      fixture.write("src/empty.mjs", "export const value = 1;\n");

      const topology = runTopologyWithStderr(fixture, {
        args: [
          "--rust-topology-scanner",
          "prefer",
          "--rust-topology-prefer-gate-corpus",
          "lab-self",
        ],
      }).topology;

      expect(topology.meta.rustTopologyPrefer).toMatchObject({
        status: "fallback-js",
        reason: "blocked-cache-mode",
        usedRust: false,
      });
      expect(topology.summary.files).toBe(1);
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("falls back before Rust ownership when prefer is requested for a non-required corpus", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-prefer-corpus-scope-",
      packageJson: { name: "rust-prefer-corpus-scope-fx", type: "module" },
    });
    try {
      fixture.write("src/empty.mjs", "export const value = 1;\n");

      const topology = runTopologyWithStderr(fixture, {
        args: [
          "--no-incremental",
          "--clear-incremental-cache",
          "--rust-topology-scanner",
          "prefer",
          "--rust-topology-prefer-gate-corpus",
          "random-repo",
        ],
      }).topology;

      expect(topology.meta.rustTopologyPrefer).toMatchObject({
        status: "fallback-js",
        reason: "blocked-corpus-scope",
        usedRust: false,
      });
      expect(topology.summary.files).toBe(1);
    } finally {
      fixture.cleanup();
    }
  }, 30000);

  it("keeps off and compare rollback paths quiet about Rust ownership", () => {
    const fixture = createTempRepoFixture({
      prefix: "vitest-topology-rust-prefer-rollback-",
      packageJson: { name: "rust-prefer-rollback-fx", type: "module" },
    });
    try {
      fixture.write("src/empty.mjs", "export const value = 1;\n");
      const sidecar = writeFakeRustTopologySidecar(path.join(fixture.output, "fake-sidecar"));

      const off = runTopologyWithStderr(fixture, {
        output: fixture.mkdir("prefer-off"),
        args: [],
      }).topology;
      const compare = runTopologyWithStderr(fixture, {
        output: fixture.mkdir("prefer-compare"),
        args: [
          "--no-incremental",
          "--clear-incremental-cache",
          "--rust-topology-scanner",
          "compare",
          "--rust-topology-scanner-bin",
          sidecar,
        ],
      }).topology;

      expect(off.meta.rustTopologyPrefer).toBeUndefined();
      expect(compare.meta.rustTopologyPrefer).toBeUndefined();
      expect(compare.meta.rustTopologyScanner.status).toBe("matched");
    } finally {
      fixture.cleanup();
    }
  }, 30000);
});

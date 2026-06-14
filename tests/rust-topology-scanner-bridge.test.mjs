import { chmodSync, mkdirSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import {
  compareRustTopologyScanner,
  normalizeScannerPath,
} from "../_lib/rust-topology-scanner.mjs";
import { MODULE_EDGE_SCANNER_POLICY_VERSION } from "../_lib/js-module-edge-scanner.mjs";

function tempDir(name) {
  const dir = path.join(
    os.tmpdir(),
    `${name}-${Date.now()}-${Math.random().toString(16).slice(2)}`,
  );
  mkdirSync(dir, { recursive: true });
  return dir;
}

function writeFakeSidecar(dir, body) {
  const file = path.join(
    dir,
    process.platform === "win32" ? "fake-sidecar.cmd" : "fake-sidecar.sh",
  );
  const script = path.join(dir, "fake-sidecar.mjs");
  writeFileSync(script, body, "utf8");
  if (process.platform === "win32") {
    const node = process.execPath;
    writeFileSync(file, `@echo off\r\n"${node}" "%~dp0\\fake-sidecar.mjs"\r\n`, "utf8");
  } else {
    writeFileSync(file, `#!/usr/bin/env sh\n"${process.execPath}" "$(dirname "$0")/fake-sidecar.mjs"\n`, "utf8");
    chmodSync(file, 0o755);
  }
  return file;
}

const jsResults = [
  {
    file: "C:/repo/src/a.ts",
    ok: true,
    loc: 2,
    edges: [
      {
        source: "./dep",
        line: 1,
        dynamic: false,
        typeOnly: false,
        reExport: false,
      },
    ],
    risk: [],
  },
];

function runSidecar(binary, overrides = {}) {
  return compareRustTopologyScanner({
    mode: "compare",
    binary,
    root: "C:/repo",
    files: ["C:/repo/src/a.ts"],
    jsResults,
    timeoutMs: 1000,
    ...overrides,
  });
}

describe("Rust topology scanner bridge", () => {
  it("normalizes Windows paths to slash form", () => {
    expect(normalizeScannerPath("C:\\repo\\src\\a.ts")).toBe("C:/repo/src/a.ts");
  });

  it("reports matched output while keeping JS as the oracle", () => {
    const dir = tempDir("lumin-rust-sidecar-matched");
    const bin = writeFakeSidecar(
      dir,
      `process.stdin.resume();
process.stdin.on('end', () => {
  process.stdout.write(JSON.stringify({
    schemaVersion: 1,
    policyVersion: '${MODULE_EDGE_SCANNER_POLICY_VERSION}',
    files: [{
      file: 'C:/repo/src/a.ts',
      ok: true,
      loc: 2,
      edges: [{ source: './dep', line: 1, dynamic: false, typeOnly: false, reExport: false }],
      risk: []
    }],
    timing: { files: 1, elapsedMs: 1 }
  }));
});
`,
    );

    const result = runSidecar(bin);
    expect(result.metadata.status).toBe("matched");
    expect(result.metadata.filesCompared).toBe(1);
    expect(result.metadata.mismatches).toBe(0);
    expect(result.metadata.sidecarTiming.elapsedMs).toBe(1);
    expect(result.useRust).toBe(false);
  });

  it("reports invalid-json-output", () => {
    const dir = tempDir("lumin-rust-sidecar-invalid-json");
    const bin = writeFakeSidecar(dir, "process.stdout.write('{bad json');\n");
    const result = runSidecar(bin);
    expect(result.metadata.status).toBe("invalid-json-output");
    expect(result.useRust).toBe(false);
  });

  it("reports non-zero-exit", () => {
    const dir = tempDir("lumin-rust-sidecar-non-zero");
    const bin = writeFakeSidecar(dir, "process.exit(7);\n");
    const result = runSidecar(bin);
    expect(result.metadata.status).toBe("non-zero-exit");
    expect(result.metadata.exitCode).toBe(7);
    expect(result.useRust).toBe(false);
  });

  it("reports timeout with the effective timeoutMs", () => {
    const dir = tempDir("lumin-rust-sidecar-timeout");
    const bin = writeFakeSidecar(dir, "setTimeout(() => {}, 5000);\n");
    const result = runSidecar(bin, { timeoutMs: 50 });
    expect(result.metadata.status).toBe("timeout");
    expect(result.metadata.timeoutMs).toBe(50);
    expect(result.useRust).toBe(false);
  });

  it("reports edge-mismatch with capped samples", () => {
    const dir = tempDir("lumin-rust-sidecar-edge-mismatch");
    const bin = writeFakeSidecar(
      dir,
      `process.stdin.resume();
process.stdin.on('end', () => {
  process.stdout.write(JSON.stringify({
    schemaVersion: 1,
    policyVersion: '${MODULE_EDGE_SCANNER_POLICY_VERSION}',
    files: [{
      file: 'C:/repo/src/a.ts',
      ok: true,
      loc: 2,
      edges: [{ source: './different', line: 1, dynamic: false, typeOnly: false, reExport: false }],
      risk: []
    }],
    timing: { files: 1, elapsedMs: 1 }
  }));
});
`,
    );

    const result = runSidecar(bin);
    expect(result.metadata.status).toBe("edge-mismatch");
    expect(result.metadata.mismatches).toBe(1);
    expect(result.metadata.mismatchSamples.length).toBeLessThanOrEqual(10);
    expect(result.useRust).toBe(false);
  });

  it("reports count-mismatch when the sidecar omits or adds files", () => {
    const dir = tempDir("lumin-rust-sidecar-count-mismatch");
    const bin = writeFakeSidecar(
      dir,
      `process.stdin.resume();
process.stdin.on('end', () => {
  process.stdout.write(JSON.stringify({
    schemaVersion: 1,
    policyVersion: '${MODULE_EDGE_SCANNER_POLICY_VERSION}',
    files: [{ file: 'C:/repo/src/extra.ts', ok: true, loc: 1, edges: [], risk: [] }],
    timing: { files: 1, elapsedMs: 1 }
  }));
});
`,
    );

    const result = runSidecar(bin);
    expect(result.metadata.status).toBe("count-mismatch");
    expect(result.metadata.mismatchSamples[0].jsOnlyFiles).toEqual([
      "C:/repo/src/a.ts",
    ]);
    expect(result.metadata.mismatchSamples[0].rustOnlyFiles).toEqual([
      "C:/repo/src/extra.ts",
    ]);
    expect(result.useRust).toBe(false);
  });

  it("rejects mismatched scanner policyVersion before comparison", () => {
    const dir = tempDir("lumin-rust-sidecar-policy-mismatch");
    const bin = writeFakeSidecar(
      dir,
      `process.stdin.resume();
process.stdin.on('end', () => {
  process.stdout.write(JSON.stringify({
    schemaVersion: 1,
    policyVersion: 'wrong-policy',
    files: [],
    timing: { files: 0, elapsedMs: 1 }
  }));
});
`,
    );

    const result = runSidecar(bin);
    expect(result.metadata.status).toBe("invalid-json-output");
    expect(result.metadata.reason).toBe("policy-version-mismatch");
    expect(result.metadata.rustPolicyVersion).toBe("wrong-policy");
    expect(result.useRust).toBe(false);
  });
});

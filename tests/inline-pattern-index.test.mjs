// Mirrors tests/test-inline-pattern-index.mjs.
//
// This suite protects review-only repeated inline catch-block evidence. It
// keeps grouping, source citations, noise suppression, and deterministic output
// visible without treating structural repetition as safe extraction proof.

import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const CLI = path.join(REPO_ROOT, "build-inline-pattern-index.mjs");

function write(root, relPath, content) {
  const full = path.join(root, relPath);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function run(root, output, extraArgs = []) {
  return execFileSync(
    process.execPath,
    [CLI, "--root", root, "--output", output, ...extraArgs],
    {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function readIndex(output) {
  return JSON.parse(
    readFileSync(path.join(output, "inline-patterns.json"), "utf8"),
  );
}

function writeRepeatedCatchDestroyFixture(root) {
  write(
    root,
    "package.json",
    JSON.stringify(
      {
        name: "inline-pattern-fixture",
        type: "module",
        private: true,
      },
      null,
      2,
    ),
  );

  write(
    root,
    "src/server.ts",
    `function send(socket: { send(value: string): void }, payload: string) { socket.send(payload); }\n` +
      `export function a(connection: { socket: { send(value: string): void, destroy(): void } }, payload: string) {\n` +
      `  try {\n` +
      `    send(connection.socket, payload);\n` +
      `  } catch {\n` +
      `    connection.socket.destroy();\n` +
      `  }\n` +
      `}\n` +
      `export function b(client: { socket: { send(value: string): void, destroy(): void } }, payload: string) {\n` +
      `  try {\n` +
      `    send(client.socket, payload);\n` +
      `  } catch {\n` +
      `    client.socket.destroy();\n` +
      `  }\n` +
      `}\n` +
      `export function c(peer: { socket: { send(value: string): void, destroy(): void } }, payload: string) {\n` +
      `  try {\n` +
      `    send(peer.socket, payload);\n` +
      `  } catch {\n` +
      `    peer.socket.destroy();\n` +
      `  }\n` +
      `}\n` +
      `export function d(target: { socket: { send(value: string): void, destroy(): void } }, payload: string) {\n` +
      `  try {\n` +
      `    send(target.socket, payload);\n` +
      `  } catch {\n` +
      `    target.socket.destroy();\n` +
      `  }\n` +
      `}\n`,
  );
}

function writeNoisyCatchFixture(root) {
  write(
    root,
    "package.json",
    JSON.stringify(
      {
        name: "inline-pattern-noisy-fixture",
        type: "module",
        private: true,
      },
      null,
      2,
    ),
  );

  write(
    root,
    "src/noisy.ts",
    `export function a() { try { work(); } catch { console.error('failed'); } }\n` +
      `export function b() { try { work(); } catch { console.error('failed'); } }\n` +
      `export function c() { try { work(); } catch { console.error('failed'); } }\n` +
      `export function d() { try { work(); } catch { console.error('failed'); } }\n` +
      `export function e() { try { work(); } catch { return; } }\n` +
      `export function f() { try { work(); } catch { return; } }\n` +
      `export function g() { try { work(); } catch { return; } }\n`,
  );
}

function stableGroupKey(group) {
  return [
    group.size,
    group.patternHash,
    ...(group.occurrences ?? []).map(
      (occ) =>
        `${occ.file}:${occ.line}:${occ.endLine}:${occ.enclosingFunction}`,
    ),
  ].join("|");
}

it("groups repeated catch-destroy blocks as review-only inline evidence", () => {
  const fx = mkdtempSync(path.join(tmpdir(), "inline-pattern-"));
  const out = mkdtempSync(path.join(tmpdir(), "inline-pattern-out-"));
  try {
    writeRepeatedCatchDestroyFixture(fx);

    const stdout = run(fx, out, ["--production"]);
    const index = readIndex(out);
    const group = index.groups?.[0];

    expect(existsSync(path.join(out, "inline-patterns.json"))).toBe(true);
    expect(stdout).toContain("[inline-patterns]");
    expect(stdout).toContain("groups");
    expect(index.meta).toMatchObject({
      schemaVersion: "inline-patterns.v1",
      supports: {
        catchBlockPatterns: true,
        statementSequencePatterns: false,
      },
    });
    expect(index.meta?.thresholdPolicies).toContainEqual(
      expect.objectContaining({
        policyId: "inline-pattern-policy",
        policyVersion: "inline-pattern-policy-v1",
        policyClass: "review",
        thresholds: expect.objectContaining({
          minOccurrences: 3,
          maxCatchStatements: 2,
        }),
      }),
    );
    expect(index.groups).toHaveLength(1);
    expect(group).toMatchObject({
      kind: "catch-block",
      size: 4,
      normalizedPattern: "catch { <id>.socket.destroy(); }",
    });
    expect(group?.occurrences).toHaveLength(4);
    expect(group?.occurrences).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          file: "src/server.ts",
          enclosingFunction: expect.any(String),
        }),
      ]),
    );
    expect(group?.occurrences?.every((occ) => Number.isInteger(occ.line))).toBe(
      true,
    );
    expect(
      group?.occurrences?.every((occ) => Number.isInteger(occ.endLine)),
    ).toBe(true);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
});

it("does not group generic logging or control-flow-only catch bodies", () => {
  const fx = mkdtempSync(path.join(tmpdir(), "inline-pattern-noisy-"));
  const out = mkdtempSync(path.join(tmpdir(), "inline-pattern-noisy-out-"));
  try {
    writeNoisyCatchFixture(fx);
    run(fx, out, ["--production"]);
    const index = readIndex(out);

    expect(index.groups ?? []).toHaveLength(0);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
});

it("keeps inline pattern group and occurrence ordering deterministic", () => {
  const fx = mkdtempSync(path.join(tmpdir(), "inline-pattern-stable-"));
  const outA = mkdtempSync(path.join(tmpdir(), "inline-pattern-stable-a-"));
  const outB = mkdtempSync(path.join(tmpdir(), "inline-pattern-stable-b-"));
  try {
    writeRepeatedCatchDestroyFixture(fx);
    run(fx, outA, ["--production"]);
    run(fx, outB, ["--production"]);
    const a = readIndex(outA);
    const b = readIndex(outB);

    expect((a.groups ?? []).map(stableGroupKey)).toEqual(
      (b.groups ?? []).map(stableGroupKey),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(outA, { recursive: true, force: true });
    rmSync(outB, { recursive: true, force: true });
  }
});

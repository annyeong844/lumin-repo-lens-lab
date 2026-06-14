// Tests for build-inline-pattern-index.mjs — repeated inline statement cues.

import { execFileSync } from 'node:child_process';
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(DIR, 'build-inline-pattern-index.mjs');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function run(root, output, extraArgs = []) {
  return execFileSync(NODE, [CLI, '--root', root, '--output', output, ...extraArgs], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function readIndex(output) {
  return JSON.parse(readFileSync(path.join(output, 'inline-patterns.json'), 'utf8'));
}

function writeRepeatedCatchDestroyFixture(root) {
  write(root, 'package.json', JSON.stringify({
    name: 'inline-pattern-fixture',
    type: 'module',
    private: true,
  }, null, 2));

  write(root, 'src/server.ts',
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
    `}\n`);
}

function writeNoisyCatchFixture(root) {
  write(root, 'package.json', JSON.stringify({
    name: 'inline-pattern-noisy-fixture',
    type: 'module',
    private: true,
  }, null, 2));

  write(root, 'src/noisy.ts',
    `export function a() { try { work(); } catch { console.error('failed'); } }\n` +
    `export function b() { try { work(); } catch { console.error('failed'); } }\n` +
    `export function c() { try { work(); } catch { console.error('failed'); } }\n` +
    `export function d() { try { work(); } catch { console.error('failed'); } }\n` +
    `export function e() { try { work(); } catch { return; } }\n` +
    `export function f() { try { work(); } catch { return; } }\n` +
    `export function g() { try { work(); } catch { return; } }\n`);
}

function stableGroupKey(group) {
  return [
    group.size,
    group.patternHash,
    ...(group.occurrences ?? []).map((occ) =>
      `${occ.file}:${occ.line}:${occ.endLine}:${occ.enclosingFunction}`)
  ].join('|');
}

// T1. Four repeated catch-destroy blocks produce one review-only inline group.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'inline-pattern-'));
  const out = mkdtempSync(path.join(tmpdir(), 'inline-pattern-out-'));
  try {
    writeRepeatedCatchDestroyFixture(fx);

    const stdout = run(fx, out, ['--production']);
    const index = readIndex(out);
    const group = index.groups?.[0];

    assert('IP1a. CLI writes inline-patterns.json',
      existsSync(path.join(out, 'inline-patterns.json')));
    assert('IP1b. stdout summarizes inline pattern run',
      stdout.includes('[inline-patterns]') && stdout.includes('groups'),
      stdout);
    assert('IP1c. schema and support flags are declared',
      index.meta?.schemaVersion === 'inline-patterns.v1' &&
        index.meta?.supports?.catchBlockPatterns === true &&
        index.meta?.supports?.statementSequencePatterns === false,
      JSON.stringify(index.meta));
    assert('IP1c2. inline pattern thresholds are exposed as policy metadata',
      index.meta?.thresholdPolicies?.some((policy) =>
        policy.policyId === 'inline-pattern-policy' &&
        policy.policyVersion === 'inline-pattern-policy-v1' &&
        policy.policyClass === 'review' &&
        policy.thresholds?.minOccurrences === 3 &&
        policy.thresholds?.maxCatchStatements === 2),
      JSON.stringify(index.meta?.thresholdPolicies, null, 2));
    assert('IP1d. repeated catch-destroy blocks are grouped',
      index.groups?.length === 1 &&
        group?.kind === 'catch-block' &&
        group?.size === 4 &&
        group?.normalizedPattern === 'catch { <id>.socket.destroy(); }',
      JSON.stringify(index.groups, null, 2));
    assert('IP1e. all occurrences are cited with source ranges',
      group?.occurrences?.length === 4 &&
        group.occurrences.every((occ) =>
          occ.file === 'src/server.ts' &&
          Number.isInteger(occ.line) &&
          Number.isInteger(occ.endLine) &&
          typeof occ.enclosingFunction === 'string'),
      JSON.stringify(group?.occurrences, null, 2));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T2. Generic logging and control-flow-only catch bodies are not default groups.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'inline-pattern-noisy-'));
  const out = mkdtempSync(path.join(tmpdir(), 'inline-pattern-noisy-out-'));
  try {
    writeNoisyCatchFixture(fx);
    run(fx, out, ['--production']);
    const index = readIndex(out);

    assert('IP2. generic noisy catch bodies stay out of default groups',
      index.groups?.length === 0,
      JSON.stringify(index.groups, null, 2));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T3. Artifact ordering is deterministic across repeated runs.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'inline-pattern-stable-'));
  const outA = mkdtempSync(path.join(tmpdir(), 'inline-pattern-stable-a-'));
  const outB = mkdtempSync(path.join(tmpdir(), 'inline-pattern-stable-b-'));
  try {
    writeRepeatedCatchDestroyFixture(fx);
    run(fx, outA, ['--production']);
    run(fx, outB, ['--production']);
    const a = readIndex(outA);
    const b = readIndex(outB);

    assert('IP3. deterministic group and occurrence ordering',
      JSON.stringify((a.groups ?? []).map(stableGroupKey)) ===
        JSON.stringify((b.groups ?? []).map(stableGroupKey)),
      JSON.stringify({ a: a.groups, b: b.groups }, null, 2));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(outA, { recursive: true, force: true });
    rmSync(outB, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

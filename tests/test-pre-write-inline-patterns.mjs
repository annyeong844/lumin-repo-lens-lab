// Integration tests for pre-write inline extraction cues.

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.join(DIR, '..');
const PREWRITE = path.join(ROOT, 'pre-write.mjs');
const NODE = process.execPath;

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function makeFixture() {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lumin-prewrite-inline-'));
  mkdirSync(path.join(dir, 'src'), { recursive: true });
  writeFileSync(path.join(dir, 'package.json'), JSON.stringify({
    name: 'inline-fixture',
    type: 'module',
  }, null, 2));
  writeFileSync(path.join(dir, 'src', 'server.ts'), `export function server(connection, payload) {
  try {
    writeWebSocketTextMessage(connection.socket, payload);
  } catch {
    connection.socket.destroy();
  }
  try {
    writeWebSocketTextMessage(connection.socket, payload);
  } catch {
    connection.socket.destroy();
  }
  try {
    writeWebSocketTextMessage(connection.socket, payload);
  } catch {
    connection.socket.destroy();
  }
  try {
    writeWebSocketTextMessage(connection.socket, payload);
  } catch {
    connection.socket.destroy();
  }
}
`);
  return dir;
}

function writeIntent(dir) {
  const intentPath = path.join(dir, 'intent.json');
  writeFileSync(intentPath, JSON.stringify({
    names: ['writeOrDestroyConnection', 'WriteOrDestroyResult'],
    shapes: [],
    files: ['src/connection-write.ts'],
    dependencies: [],
    plannedTypeEscapes: [],
    refactorSources: [{
      file: 'src/server.ts',
      lines: [4, 9, 14, 19],
      why: 'extract repeated catch-destroy handling',
    }],
  }, null, 2));
  return intentPath;
}

function runPreWrite({ root, out, intentPath, extraArgs = [] }) {
  return execFileSync(NODE, [
    PREWRITE,
    '--root', root,
    '--output', out,
    '--intent', intentPath,
    ...extraArgs,
  ], {
    cwd: ROOT,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function readLatest(out) {
  return JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
}

{
  const fx = makeFixture();
  const out = path.join(fx, '.audit');
  const intentPath = writeIntent(fx);
  try {
    const stdout = runPreWrite({ root: fx, out, intentPath });
    const latest = readLatest(out);
    const inlineCue = latest.cueCards
      .flatMap((card) => card.cues ?? [])
      .find((cue) => cue.evidenceLane === 'inline-extraction');

    assert('T1. cold pre-write creates inline-patterns artifact',
      existsSync(path.join(out, 'inline-patterns.json')));
    assert('T1b. pre-write JSON carries inline extraction review cue',
      inlineCue?.cueTier === 'AGENT_REVIEW_CUE' &&
      inlineCue?.claim === 'repeated inline statement pattern',
      JSON.stringify(latest.cueCards, null, 2));
    assert('T1c. markdown renders agent review cue wording',
      stdout.includes('Agent review cues') &&
      stdout.includes('repeated inline statement pattern'),
      stdout);
    assert('T1d. markdown does not claim safe extraction or semantic duplicate',
      !stdout.includes('Safe to extract') &&
      !stdout.includes('Duplicate behavior found'),
      stdout);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

{
  const fx = makeFixture();
  const out = path.join(fx, '.audit');
  const intentPath = writeIntent(fx);
  try {
    const stdout = runPreWrite({
      root: fx,
      out,
      intentPath,
      extraArgs: ['--no-fresh-audit'],
    });
    const latest = readLatest(out);
    assert('T2. no-fresh pre-write reports inline lane unavailable',
      latest.unavailableEvidence.some((item) =>
        item.evidenceLane === 'inline-extraction' &&
        item.status === 'UNAVAILABLE' &&
        item.artifact === 'inline-patterns.json'),
      JSON.stringify(latest.unavailableEvidence, null, 2));
    assert('T2b. missing artifact does not invent inline review cue',
      !latest.cueCards.flatMap((card) => card.cues ?? []).some((cue) => cue.evidenceLane === 'inline-extraction') &&
      !stdout.includes('repeated inline statement pattern'),
      stdout);
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

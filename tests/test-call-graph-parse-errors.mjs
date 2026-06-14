// Call graph parse-error diagnostics.
//
// build-call-graph may continue after a malformed source file, but the public
// artifact must not look complete when one or more scanned files failed to
// parse.

import { execFileSync } from 'node:child_process';
import {
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

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function write(root, rel, text) {
  const p = path.join(root, rel);
  mkdirSync(path.dirname(p), { recursive: true });
  writeFileSync(p, text);
}

const root = mkdtempSync(path.join(tmpdir(), 'call-parse-root-'));
const out = mkdtempSync(path.join(tmpdir(), 'call-parse-out-'));

try {
  write(root, 'src/good.mjs', [
    'export function live() {',
    '  return 1;',
    '}',
    'live();',
    '',
  ].join('\n'));
  write(root, 'src/bad.mjs', [
    'export function broken() {',
    '  if (',
    '}',
    '',
  ].join('\n'));

  execFileSync('node', ['build-call-graph.mjs', '--root', root, '--output', out], {
    cwd: DIR,
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  const artifact = JSON.parse(readFileSync(path.join(out, 'call-graph.json'), 'utf8'));
  const warning = (artifact.meta?.warnings ?? []).find((w) => w.code === 'call-graph-parse-errors');

  assert('T1. call graph marks artifact incomplete when any file fails to parse',
    artifact.meta?.complete === false,
    JSON.stringify(artifact.meta, null, 2));
  assert('T2. call graph exposes parse error count in meta',
    artifact.meta?.parseErrors === 1,
    JSON.stringify(artifact.meta, null, 2));
  assert('T3. call graph emits a parse-error warning record',
    warning?.count === 1,
    JSON.stringify(artifact.meta?.warnings, null, 2));
  assert('T4. parse-error warning names the malformed file',
    warning?.files?.[0]?.file === 'src/bad.mjs',
    JSON.stringify(warning, null, 2));
  assert('T5. parse-error warning preserves a human-readable parser message',
    typeof warning?.files?.[0]?.message === 'string' && warning.files[0].message.length > 0,
    JSON.stringify(warning, null, 2));
} finally {
  rmSync(root, { recursive: true, force: true });
  rmSync(out, { recursive: true, force: true });
}

console.log(`\n[test-call-graph-parse-errors] passed=${passed} failed=${failed}`);
if (failed) process.exit(1);

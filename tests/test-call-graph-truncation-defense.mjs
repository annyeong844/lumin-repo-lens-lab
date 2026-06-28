// PCEF P3 call graph truncation defense.
//
// `topCallees` is a display slice. Full ranking evidence must come from full
// fan-in maps, so identities outside the top-100 display list still need exact
// zero/non-zero data.

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

const root = mkdtempSync(path.join(tmpdir(), 'pcef-call-root-'));
const out = mkdtempSync(path.join(tmpdir(), 'pcef-call-out-'));
try {
  const exported = [];
  const imports = [];
  const calls = [];
  for (let i = 0; i < 102; i++) {
    exported.push(`export function fn${i}() { return ${i}; }`);
    imports.push(`fn${i}`);
    calls.push(`fn${i}();`);
  }
  write(root, 'src/lib.ts', `${exported.join('\n')}\n`);
  write(root, 'src/consumer.ts', [
    `import { ${imports.join(', ')} } from "./lib";`,
    ...calls,
    '',
  ].join('\n'));

  execFileSync('node', ['build-call-graph.mjs', '--root', root, '--output', out], {
    cwd: DIR,
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  const artifact = JSON.parse(readFileSync(path.join(out, 'call-graph.json'), 'utf8'));
  const topNames = new Set((artifact.topCallees ?? []).map((c) => c.name));

  assert('T1. topCallees remains a display slice',
    (artifact.topCallees ?? []).length === 100,
    JSON.stringify(artifact.topCallees?.length));
  assert('T2. generated fixture places fn101 outside display slice',
    !topNames.has('fn101'),
    JSON.stringify([...topNames].slice(-10)));
  assert('T3. full callFanInByIdentity retains fn101 despite display truncation',
    artifact.callFanInByIdentity?.['src/lib.ts::fn101'] === 1,
    JSON.stringify(artifact.callFanInByIdentity?.['src/lib.ts::fn101']));
} finally {
  rmSync(root, { recursive: true, force: true });
  rmSync(out, { recursive: true, force: true });
}

console.log(`\n[test-call-graph-truncation-defense] passed=${passed} failed=${failed}`);
if (failed) process.exit(1);

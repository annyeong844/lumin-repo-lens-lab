// P4-3 integration: pre-write shape lookup consumes shape-index.json by exact hash.

import { execFileSync } from 'node:child_process';
import {
  writeFileSync,
  readFileSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const BUILD_SHAPE_INDEX = path.join(DIR, 'build-shape-index.mjs');
const PRE_WRITE = path.join(DIR, 'pre-write.mjs');

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

{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-shape-index-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-shape-index-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'shape-fixture', type: 'module' }));
    write(fx, 'src/a.ts', `export interface CalendarA { year: number; month: number }\n`);
    write(fx, 'src/b.ts', `export type CalendarB = { month: number; year: number };\n`);

    execFileSync(NODE, [BUILD_SHAPE_INDEX, '--root', fx, '--output', out], {
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    const shapeIndex = JSON.parse(readFileSync(path.join(out, 'shape-index.json'), 'utf8'));
    const hash = shapeIndex.facts.find((f) => f.exportedName === 'CalendarA')?.hash;

    const intent = {
      names: [],
      shapes: [{ fields: [], typeLiteral: '{ month: number; year: number }' }],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = execFileSync(NODE, [
      PRE_WRITE,
      '--root', fx,
      '--output', out,
      '--intent', intentPath,
      '--no-fresh-audit',
    ], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    assert('PSI1. pre-write renders grounded shape cue from shape-index',
      stdout.includes('### Grounded facts') &&
      stdout.includes('same normalized type shape') &&
      stdout.includes('shape-index.json'),
      stdout);
    assert('PSI2. pre-write renders both matching identities',
      stdout.includes('src/a.ts::CalendarA') && stdout.includes('src/b.ts::CalendarB'),
      stdout);

    const latest = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    const shapeLookup = latest.lookups.find((l) => l.kind === 'shape');
    assert('PSI3. advisory JSON shape lookup result is SHAPE_MATCH',
      shapeLookup?.result === 'SHAPE_MATCH',
      JSON.stringify(shapeLookup));
    assert('PSI4. advisory JSON carries two exact-hash matches',
      shapeLookup?.matches?.length === 2 &&
      shapeLookup.matches.every((m) => m.hash === hash),
      JSON.stringify(shapeLookup?.matches));
    assert('PSI5. advisory JSON records hash derived from typeLiteral',
      shapeLookup?.shapeHash === hash && shapeLookup?.shapeHashSource === 'typeLiteral',
      JSON.stringify(shapeLookup));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

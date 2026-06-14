// P3-3-pre producer shape pin for `measure-topology.mjs::crossSubmoduleEdges`.
//
// Per docs/history/phases/p3/p3-3.md v3 PF-6: topology canon classification rides on the FULL
// untruncated cross-submodule edge list, not the top-30 display truncation.
// This file is the forcing function against future edits that silently
// narrow or remove the field.
//
// Pinning rules:
//   P1. `crossSubmoduleEdges` field present on output.
//   P2. Each element shape: { from: string, to: string, count: number }.
//   P3. Structured keys, NOT a stringified "a → b" edge label.
//   P4. Full list — NOT truncated like `crossSubmoduleTop` (top 30).
//   P5. Zero cross-edges fixture → `crossSubmoduleEdges: []` (empty array,
//       not missing key).
//   P6. `crossSubmoduleTop` still present, unchanged shape — regression
//       safety for existing consumers.

import { execFileSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const TOPO_CLI = path.join(DIR, 'measure-topology.mjs');

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

// ═══ P1–P4. Fixture with 3 submodules + several cross-edges ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'tpcx-multi-'));
  const out = mkdtempSync(path.join(tmpdir(), 'tpcx-multi-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'tpcx-fx', type: 'module' }));
    // Submodules: `lib/`, `app/`, `util/`. Cross-edges:
    //   lib/a.mjs → util/helper.mjs   (lib → util)
    //   lib/b.mjs → util/helper.mjs   (lib → util, same edge type)
    //   app/main.mjs → lib/a.mjs       (app → lib)
    //   app/main.mjs → util/helper.mjs (app → util)
    // Expected cross-submodule edges (3 distinct from→to pairs):
    //   lib → util    count 2
    //   app → lib     count 1
    //   app → util    count 1
    write(fx, 'util/helper.mjs', `export function helper() { return 1; }\n`);
    write(fx, 'lib/a.mjs',
      `import { helper } from '../util/helper.mjs';\n` +
      `export function a() { return helper(); }\n`);
    write(fx, 'lib/b.mjs',
      `import { helper } from '../util/helper.mjs';\n` +
      `export function b() { return helper() + 1; }\n`);
    write(fx, 'app/main.mjs',
      `import { a } from '../lib/a.mjs';\n` +
      `import { helper } from '../util/helper.mjs';\n` +
      `export function main() { return a() + helper(); }\n`);

    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    const topology = JSON.parse(readFileSync(path.join(out, 'topology.json'), 'utf8'));

    // P1. field present
    assert('P1. topology.json.crossSubmoduleEdges field present',
      Array.isArray(topology.crossSubmoduleEdges));

    // P2. shape per element
    const allWellShaped = topology.crossSubmoduleEdges.every((e) =>
      e !== null && typeof e === 'object' &&
      typeof e.from === 'string' && e.from.length > 0 &&
      typeof e.to === 'string' && e.to.length > 0 &&
      typeof e.count === 'number' && e.count >= 1
    );
    assert('P2. every element is { from: string, to: string, count: number }',
      allWellShaped, `edges=${JSON.stringify(topology.crossSubmoduleEdges)}`);

    // P3. structured keys, NOT stringified "a → b"
    const noStringifiedEdge = topology.crossSubmoduleEdges.every((e) =>
      !('edge' in e) && typeof e.from === 'string' && !e.from.includes('→'));
    assert('P3. structured keys (no stringified "a → b" label)',
      noStringifiedEdge);

    // P4. full list — 3 distinct (from, to) pairs expected from the fixture
    const pairs = new Set(topology.crossSubmoduleEdges.map((e) => `${e.from}→${e.to}`));
    assert('P4a. fixture with 3 distinct cross-submodule edge pairs yields 3 entries',
      topology.crossSubmoduleEdges.length === 3,
      `got ${topology.crossSubmoduleEdges.length}: ${JSON.stringify(topology.crossSubmoduleEdges)}`);
    assert('P4b. all 3 expected pairs present (lib→util, app→lib, app→util)',
      pairs.has('lib→util') && pairs.has('app→lib') && pairs.has('app→util'),
      `pairs=${[...pairs].join(',')}`);

    const libToUtil = topology.crossSubmoduleEdges.find((e) => e.from === 'lib' && e.to === 'util');
    assert('P4c. lib → util count aggregated to 2 (both lib/a.mjs and lib/b.mjs import util/helper.mjs)',
      libToUtil?.count === 2,
      `got=${libToUtil?.count}`);

    // P6. crossSubmoduleTop still present, unchanged shape
    assert('P6a. crossSubmoduleTop preserved',
      Array.isArray(topology.crossSubmoduleTop));
    const topShapeOk = topology.crossSubmoduleTop.every((e) =>
      e !== null && typeof e.edge === 'string' && e.edge.includes(' → ') &&
      typeof e.count === 'number');
    assert('P6b. crossSubmoduleTop retains legacy { edge: "a → b", count } shape',
      topShapeOk);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ P5. Zero cross-edges → empty array (not missing key) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'tpcx-zero-'));
  const out = mkdtempSync(path.join(tmpdir(), 'tpcx-zero-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'tpcx-zero-fx', type: 'module' }));
    // Two files in the same submodule → zero CROSS-submodule edges.
    write(fx, 'lib/a.mjs', `export const a = 1;\n`);
    write(fx, 'lib/b.mjs', `import { a } from './a.mjs'; export const b = a + 1;\n`);

    execFileSync(NODE, [TOPO_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    const topology = JSON.parse(readFileSync(path.join(out, 'topology.json'), 'utf8'));

    assert('P5a. crossSubmoduleEdges key still present even with zero cross edges',
      'crossSubmoduleEdges' in topology && Array.isArray(topology.crossSubmoduleEdges));
    assert('P5b. empty array (not undefined/null)',
      topology.crossSubmoduleEdges.length === 0);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

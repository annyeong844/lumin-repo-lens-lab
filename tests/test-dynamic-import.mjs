// Tests for dynamic import() edge detection in measure-topology.mjs
//
// Fixture setup: enriched fixture with multiple dynamic-import patterns
// to verify the recursive AST walker handles all of them.
import { execSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SCRIPT = path.resolve(__dirname, '../measure-topology.mjs');
const FIXTURE = '/tmp/fx-topo-dynamic';
const OUT = '/tmp/topo-test';

// Clean + set up fixture
rmSync(FIXTURE, { recursive: true, force: true });
mkdirSync(path.join(FIXTURE, 'src'), { recursive: true });

writeFileSync(path.join(FIXTURE, 'package.json'), JSON.stringify({ name: 'topo-fx', type: 'module' }));
// a.ts: top-level await import
writeFileSync(path.join(FIXTURE, 'src/a.ts'), `
export async function lazy() {
  const m = await import('./target');
  return m;
}
`);
// b.ts: conditional import inside if-else
writeFileSync(path.join(FIXTURE, 'src/b.ts'), `
export async function cond(flag) {
  if (flag) {
    return import('./plugin');
  }
  return null;
}
`);
// c.ts: import().then() chain
writeFileSync(path.join(FIXTURE, 'src/c.ts'), `
export function loadLater() {
  return import('./utils').then((m) => m.default);
}
`);
// d.ts: nested inside arrow in object literal
writeFileSync(path.join(FIXTURE, 'src/d.ts'), `
export const routes = {
  home: () => import('./home-page'),
  about: () => import('./about-page'),
};
`);
// target.ts, plugin.ts, utils.ts, home-page.ts, about-page.ts — targets
writeFileSync(path.join(FIXTURE, 'src/target.ts'), `export const T = 1;`);
writeFileSync(path.join(FIXTURE, 'src/plugin.ts'), `export const P = 2;`);
writeFileSync(path.join(FIXTURE, 'src/utils.ts'), `export default {};`);
writeFileSync(path.join(FIXTURE, 'src/home-page.ts'), `export const H = 3;`);
writeFileSync(path.join(FIXTURE, 'src/about-page.ts'), `export const A = 4;`);
// control.ts: regular static import (existing behavior must still work)
writeFileSync(path.join(FIXTURE, 'src/control.ts'), `
import { T } from './target';
export const x = T;
`);
// fallback.ts: require() is intentionally outside the fast scanner's accepted
// module-edge subset. Topology must fall back to the existing Oxc path.
writeFileSync(path.join(FIXTURE, 'src/fallback.ts'), `
export function cjs() {
  const c = require('./target');
  return c;
}
`);

// Run topology
rmSync(OUT, { recursive: true, force: true });
execSync(`node ${SCRIPT} --root ${FIXTURE} --output ${OUT}`, { stdio: 'inherit' });

const art = JSON.parse(readFileSync(path.join(OUT, 'topology.json'), 'utf8'));

let passed = 0, failed = 0;
function assert(label, ok, detail) {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// Extract edge list (from topFanIn / topFanOut / file adjacencies)
// Topology artifact doesn't expose per-file edges directly. Use summary and fan counts.
const summary = art.summary;
const fanIn = Object.fromEntries((art.topFanIn || []).map((x) => [x.file, x.count]));
const fanOut = Object.fromEntries((art.topFanOut || []).map((x) => [x.file, x.count]));

console.log('\n  observed summary:', JSON.stringify(summary));
console.log('  fanIn:', JSON.stringify(fanIn));
console.log('  fanOut:', JSON.stringify(fanOut));

// ── T1: total internal edges must include all dynamic + static edges ─
// Expected:
//   control.ts → target.ts          (static)                            = 1
//   a.ts      → target.ts           (dynamic)                           = 1
//   b.ts      → plugin.ts           (dynamic)                           = 1
//   c.ts      → utils.ts            (dynamic)                           = 1
//   d.ts      → home-page.ts        (dynamic)                           = 1
//   d.ts      → about-page.ts       (dynamic)                           = 1
// Total: 6 internal edges
assert(
  'T1. total internal edges = 6 (1 static + 5 dynamic)',
  summary.internalEdges === 6,
  `got internalEdges=${summary.internalEdges}, expected 6`,
);

// ── T2: target.ts has fanIn >= 2 (control.ts static + a.ts dynamic) ─
assert(
  'T2. src/target.ts has fanIn ≥ 2',
  (fanIn['src/target.ts'] || 0) >= 2,
  `got fanIn['src/target.ts']=${fanIn['src/target.ts']}`,
);

// ── T3: d.ts has fanOut = 2 (two dynamic imports inline) ─
assert(
  'T3. src/d.ts has fanOut = 2 (object-literal dynamic imports)',
  (fanOut['src/d.ts'] || 0) === 2,
  `got fanOut['src/d.ts']=${fanOut['src/d.ts']}`,
);

// ── T4: b.ts has fanOut = 1 (conditional dynamic import) ─
assert(
  'T4. src/b.ts has fanOut = 1 (dynamic inside if-branch)',
  (fanOut['src/b.ts'] || 0) === 1,
  `got fanOut['src/b.ts']=${fanOut['src/b.ts']}`,
);

// ── T5: a.ts has fanOut = 1 (top-level await import) ─
assert(
  'T5. src/a.ts has fanOut = 1 (await import)',
  (fanOut['src/a.ts'] || 0) === 1,
  `got fanOut['src/a.ts']=${fanOut['src/a.ts']}`,
);

// ── T6: no parse errors ─
assert(
  'T6. no parse errors on any file',
  summary.parseErrors === 0,
  `got parseErrors=${summary.parseErrors}`,
);

// ── T7: topology emits parser/resolver performance counters ─
assert(
  'T7. topology summary records scanner fallback and parser-call counters',
  summary.performance?.filesCollected === 11 &&
    summary.performance?.changedFiles === 11 &&
    summary.performance?.unchangedFiles === 0 &&
    summary.performance?.droppedFiles === 0 &&
    summary.performance?.jsFilesProcessed === 11 &&
    summary.performance?.scannerPolicyVersion === 'module-edge-scanner-v1' &&
    summary.performance?.scannerFilesAttempted === 11 &&
    summary.performance?.scannerAcceptedFiles === 10 &&
    summary.performance?.scannerFallbackFiles === 1 &&
    summary.performance?.scannerRiskCounts?.['require-call'] === 1 &&
    summary.performance?.oxcParseCalls === 1 &&
    summary.performance?.oxcParseErrors === 0 &&
    typeof summary.performance?.resolverMemoHits === 'number' &&
    typeof summary.performance?.resolverMemoMisses === 'number' &&
    typeof summary.performance?.resolverMemoSize === 'number',
  `performance=${JSON.stringify(summary.performance)}`,
);

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

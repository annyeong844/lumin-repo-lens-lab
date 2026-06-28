// Regression guard for v1.8.3 type-only re-export filtering.
//
// TypeScript has three forms that emit NOTHING at runtime:
//   (1) export type { X } from './y'       ← node.exportKind === 'type'
//   (2) export type * from './y'           ← ExportAllDeclaration,
//                                              exportKind === 'type'
//   (3) export { type X, type Y } from ... ← every spec exportKind='type'
// Mixed (4) `export { X, type Y } from ...` is still a runtime re-export
// because X is runtime, so the edge must survive.
//
// v1.8.4 strengthening: the previous fixture was a star graph with no
// cycles at all, so the "runtime lens reports 0 SCCs" assertion passed
// trivially — it would have passed even without the fix. The new
// fixture contains:
//
//   • a.ts ⇄ b.ts via `export type { }` — a REAL type-only cycle.
//     Pre-fix runtime lens would report SCC of size 2; post-fix SCC=0.
//
//   • c.ts ⇄ d.ts via mixed/runtime re-exports — a REAL runtime cycle.
//     Both pre-fix and post-fix must report a runtime SCC here; the fix
//     must NOT over-filter and erase legitimate runtime cycles.

import { execSync } from 'node:child_process';
import { writeFileSync, mkdirSync, rmSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const FX = '/tmp/fx-type-only-reexport';
const OUT = '/tmp/out-type-only-reexport';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

rmSync(FX, { recursive: true, force: true });
rmSync(OUT, { recursive: true, force: true });
mkdirSync(path.join(FX, 'src'), { recursive: true });
writeFileSync(path.join(FX, 'package.json'),
  '{"name":"typeonly","type":"module"}');

// ── Part A. Type-only cycle (a ⇄ b via `export type { }`) ─────
writeFileSync(path.join(FX, 'src/a.ts'),
  "export type { BType } from './b';\n" +
  'export type AType = { tag: "a" };\n' +
  'export const aRuntime = 1;\n'
);
writeFileSync(path.join(FX, 'src/b.ts'),
  "export type { AType } from './a';\n" +
  'export type BType = { tag: "b" };\n' +
  'export const bRuntime = 2;\n'
);

// ── Part B. Runtime cycle (c ⇄ d, with one mixed edge) ────────
// c.ts pulls a runtime value AND a type from d in one re-export
// (mixed form — edge must survive). d.ts re-exports a runtime value
// from c. Edge graph: c → d (runtime, mixed), d → c (runtime pure).
writeFileSync(path.join(FX, 'src/c.ts'),
  "export { runtimeD, type DType } from './d';\n" +
  'export const runtimeC = 1;\n'
);
writeFileSync(path.join(FX, 'src/d.ts'),
  "export { runtimeC } from './c';\n" +
  'export const runtimeD = 2;\n' +
  'export type DType = { tag: "d" };\n'
);

// ── Part C. Non-cycle type-only edges (sanity check) ──────────
writeFileSync(path.join(FX, 'src/types.ts'),
  'export type Foo = { x: number };\n' +
  'export const runtimeValue = 42;\n'
);
writeFileSync(path.join(FX, 'src/kind2.ts'),
  "export type * from './types';\n"  // form (2)
);
writeFileSync(path.join(FX, 'src/kind3.ts'),
  "export { type Foo } from './types';\n"  // form (3)
);

// v1.8.5: helpful failure message if the pipeline crashes — most often
// because oxc-parser isn't installed. Without this the test looks like
// it's failing on the artifact read (ENOENT), which obscures the real
// cause.
function runChecked(cmd) {
  try {
    return execSync(cmd, { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'], encoding: 'utf8' });
  } catch (e) {
    console.error(`[${path.basename(process.argv[1])}] pipeline step failed:`);
    console.error(`  cmd: ${cmd}`);
    if (e.stdout) console.error(`  stdout: ${String(e.stdout).slice(0, 500)}`);
    if (e.stderr) console.error(`  stderr: ${String(e.stderr).slice(0, 500)}`);
    console.error(`\nHint: if this is "Cannot find package 'oxc-parser'", run \`npm install\` first.`);
    process.exit(1);
  }
}

runChecked(`node build-symbol-graph.mjs --root ${FX} --output ${OUT}`);
runChecked(`node measure-topology.mjs --root ${FX} --output ${OUT}`);

const topo = JSON.parse(readFileSync(path.join(OUT, 'topology.json'), 'utf8'));
const summary = topo.summary ?? topo;

// T1: basic plumbing
assert('T1. topology.json exposes typeOnlyEdges counter',
  typeof summary.typeOnlyEdges === 'number',
  `summary keys: ${Object.keys(summary).slice(0, 12).join(', ')}`);

// T2: all three type-only forms counted.
// Expected type-only edges: a→b + b→a (Part A) + kind2→types + kind3→types
// = 4 minimum.
assert('T2. all type-only re-export forms contribute to typeOnlyEdges (>=4)',
  (summary.typeOnlyEdges ?? 0) >= 4,
  `typeOnlyEdges: ${summary.typeOnlyEdges}`);

// T3: THE core regression. Pre-1.8.3 would report a runtime SCC
// containing {a.ts, b.ts} because type-only re-exports were treated as
// runtime edges. Post-fix, the only runtime cycle is {c.ts, d.ts}.
const runtimeSccCount = summary.sccCount ?? 0;
assert('T3. runtime lens sees exactly ONE SCC (the real runtime cycle)',
  runtimeSccCount === 1,
  `sccCount: ${runtimeSccCount}, lens: ${summary.lens ?? 'unknown'}, ` +
  `topSCC: ${JSON.stringify(summary.topSccs?.slice(0, 1) ?? summary.sccs?.slice(0, 1) ?? 'n/a')}`);

// T4: the surviving runtime cycle should be the c⇄d pair, not the
// a⇄b pair which is entirely type-only.
const sccs = summary.topSccs ?? summary.sccs ?? topo.sccs ?? [];
const runtimeCycle = sccs[0];
const cycleFiles = runtimeCycle
  ? (runtimeCycle.members ?? runtimeCycle.files ?? runtimeCycle.nodes ?? [])
      .map((m) => (typeof m === 'string' ? m : m.file ?? ''))
  : [];
const hasRuntimeC = cycleFiles.some((f) => f.endsWith('c.ts'));
const hasRuntimeD = cycleFiles.some((f) => f.endsWith('d.ts'));
const hasTypeOnlyA = cycleFiles.some((f) => f.endsWith('a.ts'));
const hasTypeOnlyB = cycleFiles.some((f) => f.endsWith('b.ts'));

assert('T4. surviving SCC contains c.ts and d.ts (runtime pair)',
  hasRuntimeC && hasRuntimeD,
  `cycleFiles: ${JSON.stringify(cycleFiles)}`);
assert('T5. surviving SCC does NOT contain a.ts or b.ts (type-only pair erased)',
  !hasTypeOnlyA && !hasTypeOnlyB,
  `cycleFiles: ${JSON.stringify(cycleFiles)}`);

// T6: symbols.json still surfaces re-export source files (shape check).
// Note: reExportsByFile does NOT currently carry `typeOnly` per
// re-export — that's a follow-up improvement flagged in reviewer
// feedback (symbol-level vs file-level public API precision).
//
// v1.8.5: check the exact expected set rather than a length threshold.
// The previous `files.length >= 5` would have passed even if a.ts
// dropped out and some unrelated file leaked in. Exact-set check
// localizes the failure to the specific file that went missing.
const syms = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));
const reMap = syms.reExportsByFile ?? {};
const files = Object.keys(reMap).sort();
const expected = [
  'src/a.ts', 'src/b.ts', 'src/c.ts', 'src/d.ts',
  'src/kind2.ts', 'src/kind3.ts',
];
const missing = expected.filter((f) => !files.includes(f));
const extra = files.filter((f) => !expected.includes(f));
assert('T6. symbols.json.reExportsByFile tracks the exact expected re-exporting files',
  missing.length === 0 && extra.length === 0,
  `missing=${JSON.stringify(missing)}, extra=${JSON.stringify(extra)}, got=${JSON.stringify(files)}`);

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

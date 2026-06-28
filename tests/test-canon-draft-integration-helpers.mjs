// Integration tests for P3-2 helper-registry — P3-2 Step 4.
//
// End-to-end: fixture repo → full generate-canon-draft.mjs (real extractor +
// resolver) → parse emitted Markdown.
//
// Parser scope limit inherited from P3-1 `test-canon-draft-integration.mjs`
// (reviewer P1-3): the Markdown splitter here is fixture-controlled (known
// table layout, known row count). It is NOT a general-purpose Markdown
// parser. If a future case needs richer parsing, the renderer output format
// has drifted — fix the renderer, not this test.

import { execFileSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CANON_CLI = path.join(DIR, 'generate-canon-draft.mjs');

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

// Fixture-controlled table parser — helper table has 8 columns:
// | Name | Identity | Owner | Signature | Fan-in | Status | Tags | Any / unknown signal |
function parseHelperTableRows(md) {
  const lines = md.split('\n');
  const start = lines.findIndex((l) => l.startsWith('| Name | Identity | Owner | Signature'));
  if (start < 0) return [];
  const rows = [];
  for (let i = start + 2; i < lines.length; i++) {
    const line = lines[i];
    if (!line.startsWith('|')) break;
    const cells = line.split('|').slice(1, -1).map((c) => c.trim());
    if (cells.length < 7) continue;
    rows.push({
      name: cells[0],
      identity: cells[1],
      owner: cells[2],
      signature: cells[3],
      fanIn: cells[4],
      status: cells[5],
      tags: cells[6],
      anySignal: cells[7] ?? '',
    });
  }
  return rows;
}

// ═══ F1. Single central helper (3 distinct consumers) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-i-central-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-i-central-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'central-fx', type: 'module' }));
    write(fx, 'src/util.ts',
      `export function tryParseJson(raw: string) { try { return JSON.parse(raw) } catch { return null } }\n`);
    write(fx, 'src/c1.ts',
      `import { tryParseJson } from './util';\n` +
      `export const a = tryParseJson('1');\n`);
    write(fx, 'src/c2.ts',
      `import { tryParseJson } from './util';\n` +
      `export const b = tryParseJson('2');\n`);
    write(fx, 'src/c3.ts',
      `import { tryParseJson } from './util';\n` +
      `export const c = tryParseJson('3');\n`);
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    const rows = parseHelperTableRows(md);
    const row = rows.find((r) => r.name === '`tryParseJson`');
    assert('F1a. tryParseJson row present',
      row !== undefined);
    assert('F1b. fan-in = 3 (three distinct consumer files)',
      row && row.fanIn === '3');
    assert('F1c. status contains central-helper',
      row && row.status.includes('central-helper'));
    assert('F1d. identity uses ownerFile::exportedName format',
      row && row.identity.includes('src/util.ts::tryParseJson'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F2. Call-site count vs consumer-file count (PF-4) ═══
//
// One consumer file calls the helper multiple times in its body.
// Fan-in should be 1 (distinct consumer file count), not N (call-site count).

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-i-fanin-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-i-fanin-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fanin-fx', type: 'module' }));
    write(fx, 'src/util.ts',
      `export function doWork(x: number) { return x * 2 }\n`);
    write(fx, 'src/c.ts',
      `import { doWork } from './util';\n` +
      `export const a = doWork(1);\n` +
      `export const b = doWork(2);\n` +
      `export const c = doWork(3);\n` +
      `export const d = doWork(4);\n` +
      `export const e = doWork(5);\n` +
      `export const f = doWork(6);\n`);
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    const rows = parseHelperTableRows(md);
    const row = rows.find((r) => r.name === '`doWork`');
    assert('F2a. doWork row present',
      row !== undefined);
    assert('F2b. fan-in = 1 despite 6 call-sites in a single consumer (PF-4)',
      row && row.fanIn === '1', `row=${JSON.stringify(row)}`);
    assert('F2c. status is shared-helper (fanIn=1, non-low-info)',
      row && row.status.includes('shared-helper'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F3. Exported-never-called → zero-internal-fan-in-helper (PF-3) ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-i-orphan-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-i-orphan-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'orphan-fx', type: 'module' }));
    write(fx, 'src/public.ts',
      `export function unusedButPublic(x: number) { return x + 1 }\n`);
    // No consumers anywhere.
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    const rows = parseHelperTableRows(md);
    const row = rows.find((r) => r.name === '`unusedButPublic`');
    assert('F3a. exported-never-called helper appears in registry (PF-3)',
      row !== undefined);
    assert('F3b. fan-in = 0',
      row && row.fanIn === '0');
    assert('F3c. status = zero-internal-fan-in-helper',
      row && row.status.includes('zero-internal-fan-in-helper'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F4. Callback-passed helper still gets fan-in via import-resolve ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-i-callback-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-i-callback-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'callback-fx', type: 'module' }));
    write(fx, 'src/util.ts',
      `export function parseOne(raw: string) { return raw.trim() }\n`);
    write(fx, 'src/c.ts',
      `import { parseOne } from './util';\n` +
      `export const all = ['a', 'b', 'c'].map(parseOne);\n`);  // callback, not direct call
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    const rows = parseHelperTableRows(md);
    const row = rows.find((r) => r.name === '`parseOne`');
    assert('F4a. parseOne row present despite callback-only consumption',
      row !== undefined);
    assert('F4b. fan-in = 1 (import-resolve lens captures callback consumer)',
      row && row.fanIn === '1');
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F5. Cross-file duplicate helpers → group classification ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-i-dup-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-i-dup-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'dup-fx', type: 'module' }));
    write(fx, 'src/a.ts',
      `export function renderThing(x: number) { return x * 2 }\n`);
    write(fx, 'src/b.ts',
      `export function renderThing(x: string) { return x.toUpperCase() }\n`);
    // 3 distinct consumers of a.ts renderThing + 3 of b.ts renderThing = both central
    write(fx, 'src/c1.ts', `import { renderThing } from './a'; export const x = renderThing(1);\n`);
    write(fx, 'src/c2.ts', `import { renderThing } from './a'; export const y = renderThing(2);\n`);
    write(fx, 'src/c3.ts', `import { renderThing } from './a'; export const z = renderThing(3);\n`);
    write(fx, 'src/d1.ts', `import { renderThing } from './b'; export const x = renderThing('hi');\n`);
    write(fx, 'src/d2.ts', `import { renderThing } from './b'; export const y = renderThing('ho');\n`);
    write(fx, 'src/d3.ts', `import { renderThing } from './b'; export const z = renderThing('ha');\n`);
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    const rows = parseHelperTableRows(md);
    const dupRows = rows.filter((r) => r.name === '`renderThing`');
    assert('F5a. two renderThing rows emitted (one per owner)',
      dupRows.length === 2);
    // Both carry the SAME group classification label.
    const labels = new Set(dupRows.map((r) => r.status.split(' ')[0]));
    assert('F5b. both rows carry HELPER_DUPLICATE_STRONG (each owner has fanIn 3)',
      labels.size === 1 && [...labels][0] === 'HELPER_DUPLICATE_STRONG');
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F6. const-var (arrow helper) classification works ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-i-arrow-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-i-arrow-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'arrow-fx', type: 'module' }));
    write(fx, 'src/util.ts',
      `export const arrowHelper = (x: number) => x + 1;\n`);
    write(fx, 'src/c.ts',
      `import { arrowHelper } from './util';\n` +
      `export const a = arrowHelper(1);\n`);
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    const rows = parseHelperTableRows(md);
    const row = rows.find((r) => r.name === '`arrowHelper`');
    assert('F6a. const-var (arrow) helper surfaces in registry',
      row !== undefined);
    assert('F6b. fan-in = 1',
      row && row.fanIn === '1');
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F7. Empty repo still emits a well-formed draft ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-i-empty-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-i-empty-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'empty-fx', type: 'module' }));
    // No source files — only package.json.
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    assert('F7a. empty repo → draft header present',
      md.includes('# Helper registry draft'));
    const rows = parseHelperTableRows(md);
    assert('F7b. empty repo → zero data rows',
      rows.length === 0);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F8. call-graph cross-check: topCallees evidence without AST consumer ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-i-xcheck-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-i-xcheck-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'xcheck-fx', type: 'module' }));
    write(fx, 'src/reflective.ts',
      `export function viaReflection() { return 'hi' }\n`);
    // NO consumer imports it — AST fan-in will be 0.
    // But we plant a call-graph.json saying it was called.
    writeFileSync(path.join(out, 'call-graph.json'), JSON.stringify({
      meta: { generated: new Date().toISOString(), root: fx, tool: 'build-call-graph.mjs' },
      summary: {},
      topCallees: [{ file: 'src/reflective.ts', name: 'viaReflection', count: 8 }],
    }));
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    assert('F8a. cross-check diagnostic surfaces in Notes',
      md.includes('call-graph-evidence-but-no-ast-consumers') ||
      md.includes('call-graph-cross-check'),
      `md tail=${md.slice(-600)}`);
    assert('F8b. identity target of cross-check is the owner',
      md.includes('src/reflective.ts::viaReflection'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F9. JSON round-trip pin — emitted Markdown parses consistently ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdh-i-rt-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdh-i-rt-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'rt-fx', type: 'module' }));
    write(fx, 'src/one.ts', `export function helperA() { return 1 }\n`);
    write(fx, 'src/two.ts', `export const helperB = () => 2;\n`);
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'helper-registry'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'helper-registry.md'), 'utf8');
    const rows = parseHelperTableRows(md);
    assert('F9a. fixture-controlled parser recovers 2 rows',
      rows.length === 2);
    const names = new Set(rows.map((r) => r.name));
    assert('F9b. both helper names recovered',
      names.has('`helperA`') && names.has('`helperB`'));
    // Every emitted row's status label is a canonical §10 helper label.
    const CANONICAL_HELPER_LABELS = new Set([
      'HELPER_DUPLICATE_STRONG', 'HELPER_DUPLICATE_REVIEW', 'HELPER_LOCAL_COMMON',
      'ANY_COLLISION_HELPER', 'severely-any-contaminated-helper',
      'central-helper', 'shared-helper',
      'zero-internal-fan-in-helper', 'low-signal-helper-name',
    ]);
    const allCanonical = rows.every((r) => {
      const firstToken = r.status.split(' ')[0];
      return CANONICAL_HELPER_LABELS.has(firstToken);
    });
    assert('F9c. every row status is a canonical §10 helper label',
      allCanonical, `rows=${JSON.stringify(rows.map((r) => r.status))}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

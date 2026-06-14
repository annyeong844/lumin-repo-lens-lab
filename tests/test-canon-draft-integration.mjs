// Integration tests — P3-1 Step 4.
//
// End-to-end: fixture → build-symbol-graph → generate-canon-draft →
// parse emitted Markdown.
//
// Parser scope limit (reviewer P1-3): the Markdown splitter here is
// fixture-controlled (known table layout, known row count). It is NOT
// a general-purpose Markdown parser. If a future case needs richer
// parsing, the renderer output format has drifted — fix the renderer,
// not this test.

import { execFileSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const SYMBOLS_CLI = path.join(DIR, 'build-symbol-graph.mjs');
const CANON_CLI   = path.join(DIR, 'generate-canon-draft.mjs');

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

// Fixture-controlled table parser. Knows the renderer emits a type-ownership
// table with Name/Identity/Owner/Fan-in/Status columns. It indexes by header
// name so display-only columns such as Fan-in space do not affect assertions.
// + separator + data rows until a blank line. Returns array of { name,
// identity, fanIn, status }.
function parseTableRows(md) {
  const lines = md.split('\n');
  const start = lines.findIndex((l) => l.startsWith('| Name | Identity'));
  if (start < 0) return [];
  const headerCells = lines[start].split('|').slice(1, -1).map((c) => c.trim());
  const index = Object.fromEntries(headerCells.map((cell, i) => [cell, i]));
  const rows = [];
  for (let i = start + 2; i < lines.length; i++) {
    const line = lines[i];
    if (!line.startsWith('|')) break;
    const cells = line.split('|').slice(1, -1).map((c) => c.trim());
    if (cells.length < 5) continue;
    rows.push({
      name: cells[0],
      identity: cells[1],
      owner: cells[2],
      fanIn: cells[index['Fan-in']],
      status: cells[index.Status],
    });
  }
  return rows;
}

// ═══ F1. Same-file duplicate → group classification ═══
//
// Two exported types with the same name in the SAME file cannot occur in
// TS (redeclaration error). Same-name in different files is the real
// case; test both identities' label is the group classification.

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdi-grp-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdi-grp-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'grp-fx', type: 'module' }));
    write(fx, 'src/a.ts', `export type Result = { ok: true };\n`);
    write(fx, 'src/b.ts', `export type Result = { err: string };\n`);
    // Heavy consumers to push Rule 1.
    write(fx, 'src/consumer1.ts',
      `import { Result as R1 } from './a';\n` +
      `import { Result as R2 } from './b';\n` +
      `export const x: R1 = { ok: true };\n` +
      `export const y: R2 = { err: '' };\n`
    );
    write(fx, 'src/consumer2.ts',
      `import { Result } from './a';\n` +
      `export const z: Result = { ok: true };\n`
    );
    write(fx, 'src/consumer3.ts',
      `import { Result } from './a';\n` +
      `export const w: Result = { ok: true };\n`
    );
    execFileSync(NODE, [SYMBOLS_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'type-ownership'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'type-ownership.md'), 'utf8');
    const rows = parseTableRows(md);

    const resultRows = rows.filter((r) => r.name === '`Result`');
    assert('F1a. two `Result` rows emitted (one per owner)',
      resultRows.length === 2);
    // Both should carry the group classification (not individual).
    const labels = new Set(resultRows.map((r) => r.status.split(' ')[0]));
    assert('F1b. both Result rows carry a group classification label',
      labels.size === 1 &&
      ['DUPLICATE_STRONG', 'DUPLICATE_REVIEW', 'LOCAL_COMMON_NAME', 'ANY_COLLISION'].some((l) =>
        [...labels][0].includes(l)));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F2. Cross-file distinct names → single-identity labels ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdi-cross-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdi-cross-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'cross-fx', type: 'module' }));
    write(fx, 'src/api.ts', `export interface User { id: string }\n`);
    write(fx, 'src/blog.ts', `export type Post = { id: string };\n`);
    write(fx, 'src/use.ts',
      `import { User } from './api';\n` +
      `import { Post } from './blog';\n` +
      `export const u: User = { id: '' };\n` +
      `export const p: Post = { id: '' };\n`
    );
    execFileSync(NODE, [SYMBOLS_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'type-ownership'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'type-ownership.md'), 'utf8');
    const rows = parseTableRows(md);

    const userRow = rows.find((r) => r.name === '`User`');
    const postRow = rows.find((r) => r.name === '`Post`');
    assert('F2a. User row present',
      userRow !== undefined);
    assert('F2b. Post row present',
      postRow !== undefined);
    assert('F2c. identities are distinct',
      userRow && postRow && userRow.identity !== postRow.identity);
    // Each should carry a single-identity label.
    assert('F2d. User row carries single-identity label',
      userRow && /single-owner-(strong|weak)|zero-internal-fan-in|low-signal-type-name/.test(userRow.status));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F3. Re-export chain — owner retains identity ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdi-reexp-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdi-reexp-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'reexp-fx', type: 'module' }));
    write(fx, 'src/y.ts', `export type X = { v: number };\n`);
    write(fx, 'src/index.ts', `export { X } from './y';\n`);
    write(fx, 'src/consumer.ts',
      `import { X } from './index';\n` +
      `export const v: X = { v: 1 };\n`
    );
    execFileSync(NODE, [SYMBOLS_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'type-ownership'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'type-ownership.md'), 'utf8');
    const rows = parseTableRows(md);

    const xRow = rows.find((r) => r.name === '`X`');
    assert('F3a. X row present',
      xRow !== undefined);
    assert('F3b. terminal identity is the owner (src/y.ts::X), NOT the barrel (src/index.ts::X)',
      xRow && xRow.identity.includes('src/y.ts::X') && !xRow.identity.includes('src/index.ts::X'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ F4. JSON round-trip pin — emitted Markdown parses and matches aggregate ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'cdi-rt-'));
  const out = mkdtempSync(path.join(tmpdir(), 'cdi-rt-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'rt-fx', type: 'module' }));
    write(fx, 'src/one.ts', `export type A = number;\n`);
    write(fx, 'src/two.ts', `export interface B { s: string };\n`);
    execFileSync(NODE, [SYMBOLS_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
    execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'type-ownership'], { stdio: 'ignore' });
    const md = readFileSync(path.join(fx, 'canonical-draft', 'type-ownership.md'), 'utf8');
    const rows = parseTableRows(md);

    assert('F4a. Markdown table parses back to 2 rows',
      rows.length === 2);
    assert('F4b. row names correspond to exported type names',
      new Set(rows.map((r) => r.name)).size === 2 &&
      rows.some((r) => r.name === '`A`') && rows.some((r) => r.name === '`B`'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

// Synthetic corpus for classification label emission.
//
// This is intentionally end-to-end: tiny TS repo → build-symbol-graph →
// generate-canon-draft → emitted Markdown table. Unit tests already pin the
// classifier predicates; this suite proves the public producer path can still
// surface the canonical labels in real artifacts.

import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const NODE = process.execPath;
const SYMBOLS_CLI = path.join(ROOT, 'build-symbol-graph.mjs');
const CANON_CLI = path.join(ROOT, 'generate-canon-draft.mjs');

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

function parseTypeOwnershipRows(md) {
  const lines = md.split('\n');
  const start = lines.findIndex((line) => line.startsWith('| Name | Identity'));
  if (start < 0) return [];
  const headerCells = lines[start].split('|').slice(1, -1).map((cell) => cell.trim());
  const index = Object.fromEntries(headerCells.map((cell, i) => [cell, i]));
  const rows = [];
  for (let i = start + 2; i < lines.length; i++) {
    const line = lines[i];
    if (!line.startsWith('|')) break;
    const cells = line.split('|').slice(1, -1).map((cell) => cell.trim());
    if (cells.length < 6) continue;
    rows.push({
      name: cells[0].replace(/^`|`$/g, ''),
      identity: cells[1].replace(/^`|`$/g, ''),
      owner: cells[2].replace(/^`|`$/g, ''),
      fanIn: Number(cells[index['Fan-in']]),
      status: cells[index.Status],
      tags: cells[index.Tags],
    });
  }
  return rows;
}

function rowsFor(rows, name) {
  return rows.filter((row) => row.name === name);
}

function statusesFor(rows, name) {
  return new Set(rowsFor(rows, name).map((row) => row.status.split(/\s+/)[0]));
}

function expectUniformLabel(rows, name, expected, expectedCount) {
  const hits = rowsFor(rows, name);
  const statuses = statusesFor(rows, name);
  assert(`${name}: emits ${expectedCount} row(s)`,
    hits.length === expectedCount,
    JSON.stringify(hits, null, 2));
  assert(`${name}: all rows carry ${expected}`,
    statuses.size === 1 && statuses.has(expected),
    `statuses=${JSON.stringify([...statuses])}\nrows=${JSON.stringify(hits, null, 2)}`);
  return hits;
}

const fx = mkdtempSync(path.join(tmpdir(), 'label-emission-corpus-'));
const out = mkdtempSync(path.join(tmpdir(), 'label-emission-out-'));

try {
  write(fx, 'package.json', JSON.stringify({ name: 'label-emission-corpus', type: 'module' }, null, 2));

  // Rule 1: Result is low-info, but high fan-in must win over LOCAL_COMMON_NAME.
  write(fx, 'src/dup-strong-a.ts', 'export type Result = { ok: true };\n');
  write(fx, 'src/dup-strong-b.ts', 'export type Result = { ok: false; reason: string };\n');
  for (let i = 1; i <= 3; i++) {
    write(fx, `src/use-result-a${i}.ts`,
      `import { Result } from './dup-strong-a';\nexport const resultA${i}: Result = { ok: true };\n`);
    write(fx, `src/use-result-b${i}.ts`,
      `import { Result } from './dup-strong-b';\nexport const resultB${i}: Result = { ok: false, reason: '${i}' };\n`);
  }

  // Rule 2: low-info local names with low fan-in stay local/common.
  write(fx, 'src/card-props.ts', 'export interface Props { cardId: string }\n');
  write(fx, 'src/dialog-props.ts', 'export interface Props { open: boolean }\n');
  write(fx, 'src/use-card-props.ts',
    "import { Props } from './card-props';\nexport const cardProps: Props = { cardId: 'c' };\n");
  write(fx, 'src/use-dialog-props.ts',
    "import { Props } from './dialog-props';\nexport const dialogProps: Props = { open: true };\n");

  // Rule 3: duplicate non-low-info names with low fan-in need review.
  write(fx, 'src/api-envelope.ts', 'export type Envelope = { id: string };\n');
  write(fx, 'src/ui-envelope.ts', 'export type Envelope = { id: string; title: string };\n');
  write(fx, 'src/use-api-envelope.ts',
    "import { Envelope } from './api-envelope';\nexport const apiEnvelope: Envelope = { id: 'a' };\n");
  write(fx, 'src/use-ui-envelope.ts',
    "import { Envelope } from './ui-envelope';\nexport const uiEnvelope: Envelope = { id: 'u', title: 't' };\n");

  // Rule 0: all contaminated same-name owners collapse to ANY_COLLISION.
  write(fx, 'src/opaque-a.ts', 'export type Opaque = { a: any; b: any; c: any };\n');
  write(fx, 'src/opaque-b.ts', 'export type Opaque = { left: any; right: any; tag: any };\n');

  // Single-owner Rule 2: fan-in >= 3 promotes to single-owner-strong.
  write(fx, 'src/session.ts', 'export interface Session { id: string }\n');
  for (let i = 1; i <= 3; i++) {
    write(fx, `src/use-session-${i}.ts`,
      `import { Session } from './session';\nexport const session${i}: Session = { id: '${i}' };\n`);
  }

  execFileSync(NODE, [SYMBOLS_CLI, '--root', fx, '--output', out], { stdio: 'ignore' });
  execFileSync(NODE, [CANON_CLI, '--root', fx, '--output', out, '--source', 'type-ownership'], { stdio: 'ignore' });

  const symbols = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
  const md = readFileSync(path.join(fx, 'canonical-draft/type-ownership.md'), 'utf8');
  const rows = parseTypeOwnershipRows(md);

  assert('C1. symbols producer declares anyContamination support',
    symbols.meta?.supports?.anyContamination === true,
    JSON.stringify(symbols.meta?.supports));
  assert('C2. emitted type-ownership table parses',
    rows.length > 0,
    md);

  const resultRows = expectUniformLabel(rows, 'Result', 'DUPLICATE_STRONG', 2);
  assert('C3. Result fan-in comes from real import consumers',
    resultRows.every((row) => row.fanIn >= 3),
    JSON.stringify(resultRows, null, 2));

  expectUniformLabel(rows, 'Props', 'LOCAL_COMMON_NAME', 2);
  expectUniformLabel(rows, 'Envelope', 'DUPLICATE_REVIEW', 2);

  const opaqueRows = expectUniformLabel(rows, 'Opaque', 'ANY_COLLISION', 2);
  assert('C4. ANY_COLLISION rows carry contamination tags from real type escapes',
    opaqueRows.every((row) => /contamination:severely-any-contaminated/.test(row.tags)),
    JSON.stringify(opaqueRows, null, 2));

  const sessionRows = expectUniformLabel(rows, 'Session', 'single-owner-strong', 1);
  assert('C5. single-owner-strong row fan-in is grounded in three consumer files',
    sessionRows[0]?.fanIn === 3,
    JSON.stringify(sessionRows, null, 2));
} finally {
  rmSync(fx, { recursive: true, force: true });
  rmSync(out, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

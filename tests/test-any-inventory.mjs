// Tests for any-inventory.mjs producer — P2-0 step 3.
//
// Pinning rules from docs/history/phases/p2/p2-0.md §4.2 + §5.4:
//   - meta.supports.typeEscapes === true AND meta.complete === true on clean run.
//   - meta.supports.escapeKinds[] equals canonical §3.9 exactly.
//   - meta.scope / includeTests / exclude / fileCount populated.
//   - Parse-error file → filesWithParseErrors has an entry AND
//     meta.complete === false AND no typeEscapes from that file.
//   - Shell safety: path with spaces + $ works.
//   - --include-tests flips includeTests + scans test files.

import { execFileSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync, existsSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(DIR, 'any-inventory.mjs');

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

function run(fx, out, extraArgs = []) {
  return execFileSync(NODE, [CLI, '--root', fx, '--output', out, ...extraArgs], {
    stdio: ['ignore', 'pipe', 'pipe'], encoding: 'utf8',
  });
}

function readInv(out) {
  return JSON.parse(readFileSync(path.join(out, 'any-inventory.json'), 'utf8'));
}

function readNamedInv(out, name) {
  return JSON.parse(readFileSync(path.join(out, name), 'utf8'));
}

const CANON_ESCAPE_KINDS = [
  'explicit-any', 'as-any', 'angle-any', 'as-unknown-as-T',
  'rest-any-args', 'index-sig-any', 'generic-default-any',
  'ts-ignore', 'ts-expect-error', 'no-explicit-any-disable',
  'jsdoc-any',
];

// ═══ T1. Happy path — one escape of each kind ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'ai-happy-'));
  const out = mkdtempSync(path.join(tmpdir(), 'ai-happy-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'ai-fx', type: 'module' }));
    write(fx, 'src/all.ts',
      `type A = any;\n` +
      `const b = (x as any);\n` +
      `const c = (<any>x);\n` +
      `const d = (x as unknown as Foo);\n` +
      `function e(...args: any[]) {}\n` +
      `type F = { [k: string]: any };\n` +
      `type G<T = any> = T;\n` +
      `// @ts-ignore reason\nconst h = 1;\n` +
      `// @ts-expect-error reason\nconst i = 1;\n` +
      `// eslint-disable-next-line no-explicit-any\nconst j = 1;\n` +
      `/** @type {any} */\nconst k = readValue();\n`
    );
    run(fx, out);
    const inv = readInv(out);

    assert('T1. inventory emits 11 type-escapes on full fixture',
      inv.typeEscapes.length === 11,
      `got ${inv.typeEscapes.length}: ${inv.typeEscapes.map((e) => e.escapeKind).join(',')}`);
    const emittedKinds = new Set(inv.typeEscapes.map((e) => e.escapeKind));
    for (const k of CANON_ESCAPE_KINDS) {
      assert(`T1b. emits ${k}`, emittedKinds.has(k));
    }
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T2. meta fields populated ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'ai-meta-'));
  const out = mkdtempSync(path.join(tmpdir(), 'ai-meta-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fx' }));
    write(fx, 'src/a.ts', `export const foo = x as any;\n`);
    run(fx, out);
    const inv = readInv(out);

    assert('T2. meta.tool === any-inventory.mjs',
      inv.meta.tool === 'any-inventory.mjs');
    assert('T2b. meta.supports.typeEscapes === true',
      inv.meta.supports?.typeEscapes === true);
    assert('T2c. meta.complete === true on clean run',
      inv.meta.complete === true);
    // P0 fix (2026-04-21): scope reflects actual scan range, not a hardcoded
    // string. Default (includeTests=true) → "TS/JS including tests"; with
    // --production → "TS/JS production files". Previous hardcoded string
    // was internally inconsistent and broke scan-range parity downstream.
    assert('T2d. meta.scope === "TS/JS including tests" when tests included (default)',
      inv.meta.scope === 'TS/JS including tests');
    assert('T2e. meta.includeTests === true by default (codebase convention)',
      inv.meta.includeTests === true);
    assert('T2f. meta.fileCount is number',
      typeof inv.meta.fileCount === 'number' && inv.meta.fileCount >= 1);
    assert('T2g. meta.exclude is array',
      Array.isArray(inv.meta.exclude));
    assert('T2h. meta.supports.escapeKinds equals canonical order',
      JSON.stringify(inv.meta.supports.escapeKinds) === JSON.stringify(CANON_ESCAPE_KINDS));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T3. Parse-error handling ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'ai-parse-err-'));
  const out = mkdtempSync(path.join(tmpdir(), 'ai-parse-err-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fx' }));
    write(fx, 'src/bad.ts', `const x = ;;;broken\n`);
    write(fx, 'src/good.ts', `const y = z as any;\n`);
    run(fx, out);
    const inv = readInv(out);

    assert('T3. meta.filesWithParseErrors has at least one entry',
      Array.isArray(inv.meta.filesWithParseErrors) &&
      inv.meta.filesWithParseErrors.length >= 1,
      `got: ${JSON.stringify(inv.meta.filesWithParseErrors)}`);
    assert('T3b. meta.complete === false when parse errors exist',
      inv.meta.complete === false);
    const errored = inv.meta.filesWithParseErrors[0];
    assert('T3c. parse-error entry shape: {file, message, line?}',
      typeof errored.file === 'string' && typeof errored.message === 'string');
    assert('T3d. bad file contributes no typeEscapes',
      !inv.typeEscapes.some((e) => /bad\.ts$/.test(e.file)));
    assert('T3e. good file still emits its typeEscape',
      inv.typeEscapes.some((e) => /good\.ts$/.test(e.file) && e.escapeKind === 'as-any'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T4. --production opts out of test scanning ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'ai-inc-tests-'));
  const out = mkdtempSync(path.join(tmpdir(), 'ai-inc-tests-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fx' }));
    write(fx, 'src/a.ts', `const x = 1;\n`);
    write(fx, 'tests/sample.test.ts', `const y = z as any;\n`);

    // Default: tests ARE scanned (codebase-wide convention).
    run(fx, out);
    const invDefault = readInv(out);
    assert('T4. default scans test files (codebase convention)',
      invDefault.typeEscapes.some((e) => /sample\.test\.ts$/.test(e.file)),
      `typeEscapes files: ${invDefault.typeEscapes.map((e) => e.file).join(', ')}`);
    assert('T4b. default meta.includeTests === true',
      invDefault.meta.includeTests === true);

    // Scope string reflects default scan range.
    assert('T4b2. default meta.scope === "TS/JS including tests"',
      invDefault.meta.scope === 'TS/JS including tests');

    // With --production: tests excluded.
    const out2 = mkdtempSync(path.join(tmpdir(), 'ai-inc-tests-out2-'));
    try {
      run(fx, out2, ['--production']);
      const invProd = readInv(out2);
      assert('T4c. --production excludes test files',
        !invProd.typeEscapes.some((e) => /sample\.test\.ts$/.test(e.file)));
      assert('T4c2. --production meta.scope === "TS/JS production files"',
        invProd.meta.scope === 'TS/JS production files');
      assert('T4d. --production meta.includeTests === false',
        invProd.meta.includeTests === false);
    } finally {
      rmSync(out2, { recursive: true, force: true });
    }
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T5. Shell safety — space + $ path ═══

{
  const parent = mkdtempSync(path.join(tmpdir(), 'ai-shell-'));
  const fx = path.join(parent, 'my $fixture');
  const out = path.join(parent, 'my $output');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fx' }));
    write(fx, 'src/a.ts', `const x = y as any;\n`);
    run(fx, out);
    const inv = readInv(out);
    assert('T5. space + $ path: inventory produced',
      inv.typeEscapes.length >= 1 && inv.meta.complete === true);
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// ═══ T6. typeEscape facts carry all required fields ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'ai-fields-'));
  const out = mkdtempSync(path.join(tmpdir(), 'ai-fields-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fx' }));
    write(fx, 'src/a.ts', `export const foo = () => x as any;\n`);
    run(fx, out);
    const inv = readInv(out);
    const hit = inv.typeEscapes.find((e) => e.escapeKind === 'as-any');
    assert('T6. fact has file, line, escapeKind, codeShape, normalizedCodeShape, insideExportedIdentity, occurrenceKey',
      !!hit && typeof hit.file === 'string' && typeof hit.line === 'number' &&
      typeof hit.codeShape === 'string' && typeof hit.normalizedCodeShape === 'string' &&
      typeof hit.occurrenceKey === 'string' &&
      /^sha256:[a-f0-9]{64}$/.test(hit.occurrenceKey));
    assert('T6b. insideExportedIdentity resolves to <file>::foo',
      hit.insideExportedIdentity && hit.insideExportedIdentity.endsWith('::foo'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// ═══ T7. --artifact-name writes an invocation-specific artifact ═══

{
  const fx = mkdtempSync(path.join(tmpdir(), 'ai-artifact-name-'));
  const out = mkdtempSync(path.join(tmpdir(), 'ai-artifact-name-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'fx' }));
    write(fx, 'src/a.ts', `export const foo = () => x as any;\n`);

    const customName = 'any-inventory.pre.test-invocation.json';
    run(fx, out, ['--artifact-name', customName]);
    const customPath = path.join(out, customName);
    const inv = existsSync(customPath) ? readNamedInv(out, customName) : null;

    assert('T7. --artifact-name writes the requested artifact',
      inv?.typeEscapes?.some((e) => e.escapeKind === 'as-any') &&
      existsSync(customPath),
      JSON.stringify(inv));
    assert('T7b. --artifact-name does not also write shared any-inventory.json',
      !existsSync(path.join(out, 'any-inventory.json')),
      `files: ${existsSync(path.join(out, 'any-inventory.json'))}`);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

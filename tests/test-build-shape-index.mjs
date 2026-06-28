// Tests for build-shape-index.mjs - P4-2 shape-index producer.

import { execFileSync } from 'node:child_process';
import {
  writeFileSync,
  readFileSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  existsSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(DIR, 'build-shape-index.mjs');

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

function run(root, output, extraArgs = []) {
  return execFileSync(NODE, [CLI, '--root', root, '--output', output, ...extraArgs], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function readIndex(output) {
  return JSON.parse(readFileSync(path.join(output, 'shape-index.json'), 'utf8'));
}

function factByName(index, name) {
  return index.facts.find((f) => f.exportedName === name);
}

// T1. Happy path producer shape and grouping.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'shape-idx-'));
  const out = mkdtempSync(path.join(tmpdir(), 'shape-idx-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'shape-fixture', type: 'module' }));
    write(fx, 'src/a.ts', `export interface UserA { id: string; name?: string }\n`);
    write(fx, 'src/b.ts', `export type UserB = { name?: string; id: string };\n`);
    write(fx, 'src/c.ts', `export type Other = { id: number; name?: string };\n`);

    const stdout = run(fx, out);
    const indexPath = path.join(out, 'shape-index.json');
    const index = readIndex(out);
    const a = factByName(index, 'UserA');
    const b = factByName(index, 'UserB');
    const other = factByName(index, 'Other');

    assert('BI1a. CLI writes shape-index.json',
      existsSync(indexPath));
    assert('BI1b. stdout summarizes shape-index run',
      stdout.includes('[shape-index]') && stdout.includes('shape-hash facts'),
      stdout);
    assert('BI1c. schemaVersion is shape-index.v1',
      index.schemaVersion === 'shape-index.v1');
    assert('BI1d. meta.tool is build-shape-index.mjs',
      index.meta.tool === 'build-shape-index.mjs');
    assert('BI1e. meta supports shapeHash + normalizedVersion',
      index.meta.supports?.shapeHash === true &&
      index.meta.supports?.normalizedVersion === 'shape-hash.normalized.v1' &&
      index.meta.supports?.exportedUnionLiteralTypeAliases === true);
    assert('BI1f. complete clean run',
      index.meta.complete === true);
    assert('BI1g. three facts emitted',
      index.facts.length === 3,
      JSON.stringify(index.facts));
    assert('BI1h. same structural shape groups together',
      a?.hash === b?.hash && a?.hash !== other?.hash,
      `a=${a?.hash} b=${b?.hash} other=${other?.hash}`);
    assert('BI1i. groupsByHash lists both matching identities sorted',
      JSON.stringify(index.groupsByHash[a.hash]) ===
      JSON.stringify(['src/a.ts::UserA', 'src/b.ts::UserB']),
      JSON.stringify(index.groupsByHash));
    assert('BI1j. fact carries canonical metadata',
      a?.source === 'fresh-ast-pass' &&
      a?.scope === 'TS/JS including tests, exported types only' &&
      a?.confidence === 'high' &&
      typeof a?.observedAt === 'string');
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T2. Unsupported shapes are diagnostics, not fake facts.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'shape-idx-unsupported-'));
  const out = mkdtempSync(path.join(tmpdir(), 'shape-idx-unsupported-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'shape-fixture' }));
    write(fx, 'src/a.ts',
      `export type Good = { id: string };\n` +
      `export type Mapped<T> = { [K in keyof T]: T[K] };\n`);

    run(fx, out);
    const index = readIndex(out);

    assert('BI2a. supported fact still emitted',
      index.facts.some((f) => f.exportedName === 'Good'));
    assert('BI2b. unsupported mapped/generic declaration emits diagnostic',
      index.diagnostics.some((d) =>
        d.exportedName === 'Mapped' && d.code === 'unsupported-type-parameters'),
      JSON.stringify(index.diagnostics));
    assert('BI2c. unsupported declaration does not make run incomplete',
      index.meta.complete === true);
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T3. Parse errors are file errors, while good files still emit facts.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'shape-idx-parse-'));
  const out = mkdtempSync(path.join(tmpdir(), 'shape-idx-parse-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'shape-fixture' }));
    write(fx, 'src/good.ts', `export interface Good { id: string }\n`);
    write(fx, 'src/bad.ts', `export interface Bad { id: string `);

    run(fx, out);
    const index = readIndex(out);

    assert('BI3a. parse-error run is incomplete',
      index.meta.complete === false);
    assert('BI3b. parse error recorded in meta.filesWithParseErrors',
      index.meta.filesWithParseErrors.some((e) => e.file === 'src/bad.ts'),
      JSON.stringify(index.meta.filesWithParseErrors));
    assert('BI3c. parse-error diagnostic present',
      index.diagnostics.some((d) => d.code === 'parse-error' && d.file === 'src/bad.ts'),
      JSON.stringify(index.diagnostics));
    assert('BI3d. good file still emits its fact',
      index.facts.some((f) => f.identity === 'src/good.ts::Good'));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T4. --production excludes test files and records scope accurately.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'shape-idx-prod-'));
  const outDefault = mkdtempSync(path.join(tmpdir(), 'shape-idx-prod-out1-'));
  const outProd = mkdtempSync(path.join(tmpdir(), 'shape-idx-prod-out2-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'shape-fixture' }));
    write(fx, 'src/a.ts', `export interface ProdShape { id: string }\n`);
    write(fx, 'tests/a.test.ts', `export interface TestShape { id: string }\n`);

    run(fx, outDefault);
    run(fx, outProd, ['--production']);
    const defaultIndex = readIndex(outDefault);
    const prodIndex = readIndex(outProd);

    assert('BI4a. default includes tests',
      defaultIndex.facts.some((f) => f.identity === 'tests/a.test.ts::TestShape'));
    assert('BI4b. default scope says including tests',
      defaultIndex.meta.scope === 'TS/JS including tests, exported types only');
    assert('BI4c. --production excludes tests',
      !prodIndex.facts.some((f) => f.identity === 'tests/a.test.ts::TestShape'));
    assert('BI4d. --production scope says production files',
      prodIndex.meta.scope === 'TS/JS production files, exported types only');
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(outDefault, { recursive: true, force: true });
    rmSync(outProd, { recursive: true, force: true });
  }
}

// T5. Shell safety: spaces + $ in paths.
{
  const parent = mkdtempSync(path.join(tmpdir(), 'shape-idx-shell-'));
  const fx = path.join(parent, 'my $fixture');
  const out = path.join(parent, 'my $output');
  mkdirSync(fx, { recursive: true });
  mkdirSync(out, { recursive: true });
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'shape-fixture' }));
    write(fx, 'src/a.ts', `export interface ShellSafe { id: string }\n`);
    run(fx, out);
    const index = readIndex(out);
    assert('BI5. path with spaces + $ produces shape index',
      index.facts.some((f) => f.identity === 'src/a.ts::ShellSafe'));
  } finally {
    rmSync(parent, { recursive: true, force: true });
  }
}

// T6. Declaration merging is surfaced as unsupported, not partial facts.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'shape-idx-merge-'));
  const out = mkdtempSync(path.join(tmpdir(), 'shape-idx-merge-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'shape-fixture' }));
    write(fx, 'src/a.ts',
      `export interface Foo { a: string }\n` +
      `export interface Foo { b: number }\n`);

    run(fx, out);
    const index = readIndex(out);

    assert('BI6a. merged interface emits no partial Foo fact',
      !index.facts.some((f) => f.identity === 'src/a.ts::Foo'),
      JSON.stringify(index.facts));
    assert('BI6b. declaration-merge-unsupported diagnostic present',
      index.diagnostics.some((d) =>
        d.code === 'declaration-merge-unsupported' &&
        d.identity === 'src/a.ts::Foo'),
      JSON.stringify(index.diagnostics));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T7. Generated-file evidence is counted but facts remain present.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'shape-idx-generated-'));
  const out = mkdtempSync(path.join(tmpdir(), 'shape-idx-generated-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'shape-fixture' }));
    write(fx, 'src/routeTree.gen.ts',
      `export interface FileRoutesByPath { id: string }\n`);
    write(fx, 'src/ordinary.ts',
      `export interface Ordinary { id: string }\n`);

    run(fx, out);
    const index = readIndex(out);

    assert('BI7a. generated shape fact remains in facts[]',
      index.facts.some((f) =>
        f.identity === 'src/routeTree.gen.ts::FileRoutesByPath' &&
        f.generatedFile?.kind === 'generated-file'),
      JSON.stringify(index.facts));
    assert('BI7b. meta counts generated-file facts',
      index.meta.generatedFileFactCount === 1 &&
      index.meta.supports?.generatedFileEvidence === true,
      JSON.stringify(index.meta));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// T8. Literal union aliases participate in grouping.
{
  const fx = mkdtempSync(path.join(tmpdir(), 'shape-idx-literal-union-'));
  const out = mkdtempSync(path.join(tmpdir(), 'shape-idx-literal-union-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'shape-fixture' }));
    write(fx, 'src/a.ts', `export type StatusA = "open" | "closed" | "pending";\n`);
    write(fx, 'src/b.ts', `export type StatusB = "pending" | 'closed' | \`open\`;\n`);

    run(fx, out);
    const index = readIndex(out);
    const a = factByName(index, 'StatusA');
    const b = factByName(index, 'StatusB');

    assert('BI8a. literal union facts are emitted',
      a?.shapeKind === 'literal-union' && b?.shapeKind === 'literal-union',
      JSON.stringify(index.facts));
    assert('BI8b. literal unions group by exact normalized literal set',
      a?.hash && a.hash === b?.hash &&
      JSON.stringify(index.groupsByHash[a.hash]) ===
        JSON.stringify(['src/a.ts::StatusA', 'src/b.ts::StatusB']),
      JSON.stringify(index.groupsByHash));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

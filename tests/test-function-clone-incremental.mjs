// Tests for strict incremental caching in build-function-clone-index.mjs.

import { execFileSync } from 'node:child_process';
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const NODE = process.execPath;
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const CLI = path.join(ROOT, 'build-function-clone-index.mjs');

let passed = 0;
let failed = 0;

function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}\n        ${detail}`);
  }
}

function fresh() {
  return mkdtempSync(path.join(tmpdir(), 'lumin-fn-clone-inc-'));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function run(root, output, args = []) {
  return execFileSync(NODE, [CLI, '--root', root, '--output', output, ...args], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function readIndex(output) {
  return JSON.parse(readFileSync(path.join(output, 'function-clones.json'), 'utf8'));
}

function stripRunMetadata(value) {
  if (Array.isArray(value)) return value.map(stripRunMetadata);
  if (value && typeof value === 'object') {
    const out = {};
    for (const [key, child] of Object.entries(value)) {
      if (key === 'generated' || key === 'observedAt' || key === 'incremental') continue;
      out[key] = stripRunMetadata(child);
    }
    return out;
  }
  return value;
}

function stableIndex(index) {
  return stripRunMetadata(index);
}

function setupRepo(repo) {
  write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
  write(repo, 'src/money-a.ts',
    `export function formatCurrencyCents(cents: number, currency = 'USD') {\n` +
    `  const dollars = cents / 100;\n` +
    `  return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(dollars);\n` +
    `}\n`);
  write(repo, 'src/money-b.ts',
    `export function renderPaymentTotal(value: number, unit = 'USD') {\n` +
    `  const amount = value / 100;\n` +
    `  return new Intl.NumberFormat('en-US', { style: 'currency', currency: unit }).format(amount);\n` +
    `}\n`);
  write(repo, 'src/exact-a.ts',
    `export const parseOne = (raw: string) => {\n` +
    `  const value = raw.trim();\n` +
    `  return value.toUpperCase();\n` +
    `};\n`);
  write(repo, 'src/exact-b.ts',
    `const local = (raw: string) => {\n` +
    `  const value = raw.trim();\n` +
    `  return value.toUpperCase();\n` +
    `};\n` +
    `export { local as parseTwo };\n`);
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);

    run(repo, output, ['--no-incremental']);
    const cold = readIndex(output);
    run(repo, output);
    const firstIncremental = readIndex(output);
    run(repo, output);
    const warm = readIndex(output);

    assert('function-clones incremental equals cold public artifact',
      JSON.stringify(stableIndex(firstIncremental)) === JSON.stringify(stableIndex(cold)));
    assert('warm function-clones equals cold public artifact',
      JSON.stringify(stableIndex(warm)) === JSON.stringify(stableIndex(cold)));
    assert('warm function-clones reports strict incremental enabled',
      warm.meta.incremental?.enabled === true &&
        warm.meta.incremental?.identityMode === 'strict-content-hash',
      JSON.stringify(warm.meta.incremental));
    assert('warm function-clones reused unchanged file payloads',
      warm.meta.incremental?.reusedFiles >= 4,
      JSON.stringify(warm.meta.incremental));
    assert('warm reused facts are stamped with current artifact observedAt',
      warm.facts.every((fact) => fact.observedAt === warm.meta.observedAt),
      JSON.stringify(warm.facts.map((fact) => ({
        identity: fact.identity,
        factObservedAt: fact.observedAt,
        metaObservedAt: warm.meta.observedAt,
      }))));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output);
    run(repo, output);

    write(repo, 'src/money-b.ts',
      `export function renderPaymentTotal(value: number, unit = 'USD') {\n` +
      `  const amount = value / 100;\n` +
      `  return new Intl.NumberFormat('en-GB', { style: 'currency', currency: unit }).format(amount);\n` +
      `}\n`);
    run(repo, output);
    const index = readIndex(output);
    const changed = index.facts.find((f) => f.identity === 'src/money-b.ts::renderPaymentTotal');
    const unchanged = index.facts.find((f) => f.identity === 'src/money-a.ts::formatCurrencyCents');

    assert('changed file refreshes function clone fact',
      changed?.exactBodyHash && unchanged?.exactBodyHash &&
        changed.exactBodyHash !== unchanged.exactBodyHash,
      JSON.stringify(index.facts));
    assert('changed run reuses unchanged function clone files',
      index.meta.incremental?.changedFiles >= 1 &&
        index.meta.incremental?.reusedFiles >= 1,
      JSON.stringify(index.meta.incremental));
    assert('changed file does not count as dropped',
      index.meta.incremental?.droppedFiles === 0,
      JSON.stringify(index.meta.incremental));

    const coldAfterChangeOutput = path.join(repo, '.audit-cold-after-change');
    run(repo, coldAfterChangeOutput, ['--no-incremental']);
    const coldAfterChange = readIndex(coldAfterChangeOutput);
    assert('changed incremental artifact equals cold artifact after same change',
      JSON.stringify(stableIndex(index)) === JSON.stringify(stableIndex(coldAfterChange)));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output);
    run(repo, output);

    write(repo, 'src/new-exact-c.ts',
      `export const parseThree = (raw: string) => {\n` +
      `  const value = raw.trim();\n` +
      `  return value.toUpperCase();\n` +
      `};\n`);
    run(repo, output);
    const index = readIndex(output);
    const matchingGroup = (index.exactBodyGroups ?? []).find((group) =>
      (group.identities ?? []).includes('src/exact-a.ts::parseOne') &&
      (group.identities ?? []).includes('src/new-exact-c.ts::parseThree'));

    assert('global clone groups rebuild from mixed fresh and reused facts',
      !!matchingGroup,
      JSON.stringify(index.exactBodyGroups));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output);
    run(repo, output);

    rmSync(path.join(repo, 'src/exact-b.ts'), { force: true });
    run(repo, output);
    const index = readIndex(output);

    assert('deleted file function clone facts disappear',
      !index.facts.some((f) => f.ownerFile === 'src/exact-b.ts'),
      JSON.stringify(index.facts));
    assert('deleted file contributes function clone dropped count',
      index.meta.incremental?.droppedFiles >= 1,
      JSON.stringify(index.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output);
    run(repo, output);

    rmSync(path.join(repo, 'src/money-a.ts'), { force: true });
    write(repo, 'src/moved-money-a.ts',
      `export function formatCurrencyCents(cents: number, currency = 'USD') {\n` +
      `  const dollars = cents / 100;\n` +
      `  return new Intl.NumberFormat('en-US', { style: 'currency', currency }).format(dollars);\n` +
      `}\n`);
    run(repo, output);
    const index = readIndex(output);

    assert('moved file with same content is treated as changed under relPath identity',
      index.meta.incremental?.changedFiles >= 1 &&
        index.meta.incremental?.droppedFiles >= 1 &&
        index.facts.some((f) => f.identity === 'src/moved-money-a.ts::formatCurrencyCents'),
      JSON.stringify(index.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output);
    run(repo, output);
    run(repo, output, ['--clear-incremental-cache']);
    const index = readIndex(output);
    assert('--clear-incremental-cache clears function clone cache before run',
      index.meta.incremental?.enabled === true &&
        index.meta.incremental?.reusedFiles === 0 &&
        index.meta.incremental?.changedFiles >= 4,
      JSON.stringify(index.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    run(repo, output, ['--no-incremental']);
    const index = readIndex(output);
    assert('--no-incremental reports disabled function clone cache',
      index.meta.incremental?.enabled === false &&
        index.meta.incremental?.reason === 'disabled-by-flag',
      JSON.stringify(index.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

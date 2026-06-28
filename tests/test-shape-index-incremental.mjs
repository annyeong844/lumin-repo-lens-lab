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
const CLI = path.join(ROOT, 'build-shape-index.mjs');
const AUDIT = path.join(ROOT, 'audit-repo.mjs');

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
  return mkdtempSync(path.join(tmpdir(), 'lumin-shape-inc-'));
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

function runAudit(root, output, args = []) {
  return execFileSync(NODE, [AUDIT, '--root', root, '--output', output, ...args], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function readIndex(output) {
  return JSON.parse(readFileSync(path.join(output, 'shape-index.json'), 'utf8'));
}

function stableIndex(index) {
  const facts = (index.facts ?? []).map(({ observedAt: _observedAt, ...fact }) => fact);
  const { meta, ...rest } = index;
  const {
    generated: _generated,
    observedAt: _metaObservedAt,
    incremental: _incremental,
    ...stableMeta
  } = meta ?? {};
  return {
    meta: stableMeta,
    ...rest,
    facts,
  };
}

function setupRepo(repo) {
  write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
  write(repo, 'src/a.ts', 'export interface UserA { id: string; name?: string }\n');
  write(repo, 'src/b.ts', 'export type UserB = { name?: string; id: string };\n');
  write(repo, 'src/c.ts', 'export type Other = { id: number; name?: string };\n');
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

    assert('shape-index incremental equals cold public facts',
      JSON.stringify(stableIndex(firstIncremental)) === JSON.stringify(stableIndex(cold)));
    assert('warm shape-index equals cold public facts',
      JSON.stringify(stableIndex(warm)) === JSON.stringify(stableIndex(cold)));
    assert('warm run reports strict incremental enabled',
      warm.meta.incremental?.enabled === true &&
        warm.meta.incremental?.identityMode === 'strict-content-hash',
      JSON.stringify(warm.meta.incremental));
    assert('warm run reused unchanged shape facts',
      warm.meta.incremental?.reusedFiles >= 3,
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

    write(repo, 'src/b.ts', 'export type UserB = { name?: string; id: number };\n');
    run(repo, output);
    const index = readIndex(output);
    const userA = index.facts.find((f) => f.identity === 'src/a.ts::UserA');
    const userB = index.facts.find((f) => f.identity === 'src/b.ts::UserB');

    assert('changed file updates shape hash',
      userA?.hash && userB?.hash && userA.hash !== userB.hash,
      JSON.stringify(index.facts));
    assert('changed run reuses unchanged files and refreshes changed file',
      index.meta.incremental?.changedFiles >= 1 &&
        index.meta.incremental?.reusedFiles >= 1,
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

    rmSync(path.join(repo, 'src/b.ts'), { force: true });
    run(repo, output);
    const index = readIndex(output);

    assert('deleted file shape facts disappear',
      !index.facts.some((f) => f.ownerFile === 'src/b.ts'),
      JSON.stringify(index.facts));
    assert('deleted file contributes dropped count',
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
    run(repo, output, ['--no-incremental']);
    const index = readIndex(output);
    assert('--no-incremental reports disabled shape-index cache',
      index.meta.incremental?.enabled === false &&
        index.meta.incremental?.reason === 'disabled-by-flag',
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
    runAudit(repo, output, ['--profile', 'full', '--no-incremental']);
    const coldIndex = readIndex(output);
    assert('audit-repo forwards --no-incremental to build-shape-index',
      coldIndex.meta.incremental?.enabled === false &&
        coldIndex.meta.incremental?.reason === 'disabled-by-flag',
      JSON.stringify(coldIndex.meta.incremental));

    const cacheRoot = path.join(repo, '.shape-cache');
    runAudit(repo, output, ['--profile', 'full', '--cache-root', cacheRoot]);
    runAudit(repo, output, ['--profile', 'full', '--cache-root', cacheRoot]);
    const warmIndex = readIndex(output);
    assert('audit-repo forwards --cache-root to build-shape-index',
      warmIndex.meta.incremental?.enabled === true &&
        path.resolve(warmIndex.meta.incremental.cacheRoot) === path.resolve(cacheRoot) &&
        warmIndex.meta.incremental.reusedFiles >= 3,
      JSON.stringify(warmIndex.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

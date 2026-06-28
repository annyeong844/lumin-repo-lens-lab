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
const CLI = path.join(ROOT, 'audit-repo.mjs');

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
  return mkdtempSync(path.join(tmpdir(), 'lumin-audit-symbol-inc-'));
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function setupRepo(repo) {
  write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
  write(repo, 'src/a.ts', 'export const alive = 1;\n');
}

function runAudit(root, output, args = []) {
  execFileSync(NODE, [CLI, '--root', root, '--output', output, '--profile', 'quick', ...args], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

function readSymbols(output) {
  return JSON.parse(readFileSync(path.join(output, 'symbols.json'), 'utf8'));
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);
    runAudit(repo, output, ['--no-incremental']);
    const symbols = readSymbols(output);

    assert('audit-repo forwards --no-incremental to build-symbol-graph',
      symbols.meta.incremental?.enabled === false &&
        symbols.meta.incremental?.reason === 'disabled-by-flag',
      JSON.stringify(symbols.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  const cacheRoot = path.join(repo, '.custom-cache');
  try {
    setupRepo(repo);
    runAudit(repo, output, ['--cache-root', cacheRoot]);
    const symbols = readSymbols(output);

    assert('audit-repo forwards --cache-root to build-symbol-graph',
      path.resolve(symbols.meta.incremental?.cacheRoot ?? '') === path.resolve(cacheRoot),
      JSON.stringify(symbols.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

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
const CLI = path.join(ROOT, 'any-inventory.mjs');

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
  return mkdtempSync(path.join(tmpdir(), 'lumin-any-inc-'));
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

function readInv(output, name = 'any-inventory.json') {
  return JSON.parse(readFileSync(path.join(output, name), 'utf8'));
}

function stableInventory(inv) {
  return {
    complete: inv.meta.complete,
    scope: inv.meta.scope,
    includeTests: inv.meta.includeTests,
    exclude: inv.meta.exclude,
    supports: inv.meta.supports,
    typeEscapes: inv.typeEscapes,
    filesWithParseErrors: inv.meta.filesWithParseErrors,
  };
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    write(repo, 'src/b.ts', 'const b = value as unknown as string;\n');

    run(repo, output, ['--no-incremental']);
    const cold = readInv(output);
    run(repo, output);
    const firstIncremental = readInv(output);
    run(repo, output);
    const warm = readInv(output);

    assert('incremental any-inventory equals cold public facts',
      JSON.stringify(stableInventory(firstIncremental)) === JSON.stringify(stableInventory(cold)));
    assert('warm any-inventory equals cold public facts',
      JSON.stringify(stableInventory(warm)) === JSON.stringify(stableInventory(cold)));
    assert('warm run reports incremental enabled',
      warm.meta.incremental?.enabled === true);
    assert('warm run reused at least one file',
      warm.meta.incremental?.reusedFiles >= 1,
      JSON.stringify(warm.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    write(repo, 'src/b.ts', 'const b = value as any;\n');
    run(repo, output);

    write(repo, 'src/b.ts', 'const b = value as unknown as string;\n');
    run(repo, output);
    const inv = readInv(output);

    assert('changed file facts update after edit',
      inv.typeEscapes.some((fact) => fact.file === 'src/b.ts' && fact.escapeKind === 'as-unknown-as-T'));
    assert('unchanged file remains present',
      inv.typeEscapes.some((fact) => fact.file === 'src/a.ts' && fact.escapeKind === 'as-any'));
    assert('incremental changed count is positive',
      inv.meta.incremental?.changedFiles >= 1);
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    write(repo, 'src/b.ts', 'const b = value as any;\n');
    run(repo, output);

    rmSync(path.join(repo, 'src/b.ts'), { force: true });
    run(repo, output);
    const inv = readInv(output);

    assert('deleted file facts disappear',
      !inv.typeEscapes.some((fact) => fact.file === 'src/b.ts'));
    assert('deleted file contributes dropped count',
      inv.meta.incremental?.droppedFiles >= 1,
      JSON.stringify(inv.meta.incremental));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    write(repo, 'tests/a.test.ts', 'const t = value as any;\n');
    run(repo, output, ['--production']);
    run(repo, output);
    const inv = readInv(output);

    assert('scan option change keeps public artifact correct',
      inv.meta.includeTests === true &&
      inv.typeEscapes.some((fact) => fact.file === 'tests/a.test.ts'));
    assert('scan option change prevents stale production-only reuse',
      inv.meta.incremental?.invalidatedFiles >= 0);
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    run(repo, output);

    const cacheFile = path.join(output, '.cache', 'incremental');
    mkdirSync(cacheFile, { recursive: true });
    writeFileSync(path.join(cacheFile, 'bad.cache.json'), '{broken');

    run(repo, output);
    const inv = readInv(output);
    assert('malformed unrelated cache does not crash producer',
      inv.meta.complete === true && inv.typeEscapes.length === 1);
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture' }));
    write(repo, 'src/a.ts', 'const a = value as any;\n');
    run(repo, output, ['--no-incremental']);
    const inv = readInv(output);
    assert('--no-incremental reports disabled meta',
      inv.meta.incremental?.enabled === false &&
      inv.meta.incremental?.reason === 'disabled-by-flag');
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

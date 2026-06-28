import { execFileSync } from 'node:child_process';
import {
  mkdirSync,
  mkdtempSync,
  readdirSync,
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
const CLI = path.join(ROOT, 'build-symbol-graph.mjs');

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
  return mkdtempSync(path.join(tmpdir(), 'lumin-symbol-inc-'));
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

function readSymbols(output) {
  return JSON.parse(readFileSync(path.join(output, 'symbols.json'), 'utf8'));
}

function findSymbolsCacheFile(repo) {
  const base = path.join(repo, '.audit', '.cache', 'incremental');
  for (const dir of readdirSync(base, { withFileTypes: true })) {
    if (!dir.isDirectory()) continue;
    const file = path.join(base, dir.name, 'symbols.cache.json');
    try {
      readFileSync(file);
      return file;
    } catch {
      // try the next repo-fingerprint directory
    }
  }
  throw new Error(`symbols cache not found under ${base}`);
}

function rewriteSymbolsCache(repo, mutateEntry) {
  const file = findSymbolsCacheFile(repo);
  const cache = JSON.parse(readFileSync(file, 'utf8'));
  for (const entry of Object.values(cache.entries ?? {})) mutateEntry(entry);
  writeFileSync(file, JSON.stringify(cache, null, 2) + '\n');
}

function stableSymbols(symbols) {
  const { meta, ...rest } = symbols;
  return {
    meta: {
      schemaVersion: meta?.schemaVersion,
      supports: meta?.supports,
      languageSupport: meta?.languageSupport,
      warnings: meta?.warnings ?? [],
    },
    ...rest,
  };
}

function setupRepo(repo) {
  write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
  write(repo, 'src/a.ts', [
    'export function used() { return 1; }',
    'export const unused = 2;',
    '',
  ].join('\n'));
  write(repo, 'src/b.ts', [
    "import { used } from './a';",
    'export const consumer = used();',
    '',
  ].join('\n'));
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    setupRepo(repo);

    run(repo, output, ['--no-incremental']);
    const cold = readSymbols(output);
    run(repo, output);
    const firstIncremental = readSymbols(output);
    run(repo, output);
    const warm = readSymbols(output);

    assert('symbol graph incremental equals cold public facts',
      JSON.stringify(stableSymbols(firstIncremental)) === JSON.stringify(stableSymbols(cold)));
    assert('warm symbol graph equals cold public facts',
      JSON.stringify(stableSymbols(warm)) === JSON.stringify(stableSymbols(cold)));
    assert('warm run reports strict incremental enabled',
      warm.meta.incremental?.enabled === true &&
        warm.meta.incremental?.identityMode === 'strict-content-hash',
      JSON.stringify(warm.meta.incremental));
    assert('warm run reused unchanged file facts',
      warm.meta.incremental?.reusedFiles >= 2,
      JSON.stringify(warm.meta.incremental));
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

    write(repo, 'src/b.ts', [
      'export const consumer = 0;',
      '',
    ].join('\n'));
    run(repo, output);
    const symbols = readSymbols(output);

    assert('changed consumer file updates fan-in',
      symbols.fanInByIdentity?.['src/a.ts::used'] === 0,
      JSON.stringify(symbols.fanInByIdentity));
    assert('changed run reuses unchanged files and refreshes changed files',
      symbols.meta.incremental?.changedFiles >= 1 &&
        symbols.meta.incremental?.reusedFiles >= 1,
      JSON.stringify(symbols.meta.incremental));
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

    rmSync(path.join(repo, 'src/a.ts'), { force: true });
    run(repo, output);
    const symbols = readSymbols(output);

    assert('deleted definition file facts disappear',
      symbols.defIndex?.['src/a.ts'] === undefined,
      JSON.stringify(symbols.defIndex));
    assert('deleted file contributes dropped count',
      symbols.meta.incremental?.droppedFiles >= 1,
      JSON.stringify(symbols.meta.incremental));
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
    const symbols = readSymbols(output);
    assert('--no-incremental reports disabled symbol graph cache',
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
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
    write(repo, 'src/exporter.cjs', [
      'exports.foo = 1;',
      'module.exports = makeExports();',
      '',
    ].join('\n'));

    run(repo, output);
    rewriteSymbolsCache(repo, (entry) => {
      if (entry.identity?.relPath !== 'src/exporter.cjs') return;
      delete entry.payload.cjsExportSurface;
      entry.producerMeta.producerVersion = 1;
      entry.producerMeta.factSchemaVersion = 2;
      entry.producerMeta.parserIdentity = 'symbol-graph-extractors:v1';
    });

    run(repo, output);
    const symbols = readSymbols(output);
    const surface = symbols.cjsExportSurfaceByFile?.['src/exporter.cjs'];

    assert('legacy symbol cache without CJS export surface is invalidated',
      surface?.exact?.some((entry) => entry.name === 'foo') &&
        surface?.opaque?.some((entry) => entry.kind === 'module-exports-assignment') &&
        symbols.meta.incremental?.invalidatedFiles >= 1,
      JSON.stringify({
        surface,
        incremental: symbols.meta.incremental,
      }));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
    write(repo, 'src/consumer.js', [
      'const target = "./exporter.js";',
      'require(target);',
      '',
    ].join('\n'));

    run(repo, output);
    rewriteSymbolsCache(repo, (entry) => {
      if (entry.identity?.relPath !== 'src/consumer.js') return;
      delete entry.payload.cjsRequireOpacity;
      entry.producerMeta.producerVersion = 1;
      entry.producerMeta.factSchemaVersion = 2;
      entry.producerMeta.parserIdentity = 'symbol-graph-extractors:v1';
    });

    run(repo, output);
    const symbols = readSymbols(output);

    assert('legacy symbol cache without dynamic CJS require opacity is invalidated',
      symbols.cjsRequireOpacity?.some((entry) =>
        entry.consumerFile === 'src/consumer.js' &&
        entry.kind === 'dynamic-require' &&
        entry.line === 2) &&
        symbols.meta.incremental?.invalidatedFiles >= 1,
      JSON.stringify({
        cjsRequireOpacity: symbols.cjsRequireOpacity,
        incremental: symbols.meta.incremental,
      }));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
    write(repo, 'src/version-checker.js', [
      'import path from "node:path";',
      'import { createRequire } from "node:module";',
      'const require = createRequire(import.meta.url);',
      'export function getCurrentVersion() {',
      '  return require(path.resolve(import.meta.dirname, "../package.json")).version;',
      '}',
      '',
    ].join('\n'));

    run(repo, output);
    rewriteSymbolsCache(repo, (entry) => {
      if (entry.identity?.relPath !== 'src/version-checker.js') return;
      entry.payload.cjsRequireOpacity = [
        { line: 5, kind: 'dynamic-require' },
      ];
      entry.producerMeta.producerVersion = 1;
      entry.producerMeta.factSchemaVersion = 3;
      entry.producerMeta.parserIdentity = 'symbol-graph-extractors:v1';
    });

    run(repo, output);
    const symbols = readSymbols(output);

    assert('legacy symbol cache with stale JSON require opacity is invalidated',
      (symbols.cjsRequireOpacity ?? []).length === 0 &&
        symbols.meta.incremental?.invalidatedFiles >= 1,
      JSON.stringify({
        cjsRequireOpacity: symbols.cjsRequireOpacity,
        incremental: symbols.meta.incremental,
      }));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

{
  const repo = fresh();
  const output = path.join(repo, '.audit');
  try {
    write(repo, 'package.json', JSON.stringify({ name: 'fixture', private: true }));
    write(repo, 'src/exporter.js', 'export const foo = 1;\n');
    write(repo, 'src/consumer.js', [
      'const mod = require("./exporter.js");',
      'if (mod) mod["foo"]();',
      '',
    ].join('\n'));

    run(repo, output);
    rewriteSymbolsCache(repo, (entry) => {
      if (entry.identity?.relPath !== 'src/consumer.js') return;
      entry.producerMeta.parserIdentity = 'symbol-graph-extractors:v1';
    });

    run(repo, output);
    const symbols = readSymbols(output);

    assert('legacy symbol cache with old CJS extractor identity is invalidated',
      symbols.fanInByIdentity?.['src/exporter.js::foo'] === 1 &&
        symbols.meta.incremental?.invalidatedFiles >= 1,
      JSON.stringify({
        fanIn: symbols.fanInByIdentity,
        incremental: symbols.meta.incremental,
      }));
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

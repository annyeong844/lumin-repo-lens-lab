// Regression guard: namespace re-exports must not keep every source export
// alive. `export * as ns from "./source"` exposes a namespace object, but only
// observed namespace members should count as named consumers.

import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}

function writeFixtureFile(root, rel, content) {
  const file = path.join(root, rel);
  mkdirSync(path.dirname(file), { recursive: true });
  writeFileSync(file, content);
}

function runSymbolGraph(files) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-namespace-reexport-'));
  const out = path.join(dir, '.audit');
  mkdirSync(out, { recursive: true });
  for (const [rel, content] of Object.entries(files)) {
    writeFixtureFile(dir, rel, content);
  }
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'build-symbol-graph.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
      '--no-incremental',
    ], { encoding: 'utf8' });
    return JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

const symbols = runSymbolGraph({
  'src/source.ts': [
    'export function nsUsedFunc() { return 1; }',
    'export function nsUnusedFunc() { return 2; }',
    'export const nsUsedConst = 3;',
    'export const nsUnusedConst = 4;',
  ].join('\n'),
  'src/barrel.ts': 'export * as ns from "./source";\n',
  'src/consumer.ts': [
    'import { ns } from "./barrel";',
    'ns.nsUsedFunc();',
    'console.log(ns.nsUsedConst);',
  ].join('\n'),
});

const fanIn = symbols.fanInByIdentity ?? {};
const fanInSpace = symbols.fanInByIdentitySpace ?? {};
const dead = new Set((symbols.deadProdList ?? []).map((item) => `${item.file}::${item.symbol}`));

assert('NR1. used namespace function gets exact fan-in through namespace re-export',
  fanIn['src/source.ts::nsUsedFunc'] === 1,
  JSON.stringify(fanIn, null, 2));
assert('NR2. used namespace const gets exact fan-in through namespace re-export',
  fanIn['src/source.ts::nsUsedConst'] === 1,
  JSON.stringify(fanIn, null, 2));
assert('NR3. unused namespace function remains a dead export candidate',
  dead.has('src/source.ts::nsUnusedFunc'),
  JSON.stringify(symbols.deadProdList, null, 2));
assert('NR4. unused namespace const remains a dead export candidate',
  dead.has('src/source.ts::nsUnusedConst'),
  JSON.stringify(symbols.deadProdList, null, 2));
assert('NR5. namespace re-export does not add broad fan-in to unused members',
  fanInSpace['src/source.ts::nsUnusedFunc']?.broad === 0 &&
    fanInSpace['src/source.ts::nsUnusedConst']?.broad === 0,
  JSON.stringify(fanInSpace, null, 2));

const chainedSymbols = runSymbolGraph({
  'src/source.ts': [
    'export function chainedUsedFunc() { return 1; }',
    'export function chainedUnusedFunc() { return 2; }',
    'export const chainedUsedConst = 3;',
    'export const chainedUnusedConst = 4;',
  ].join('\n'),
  'src/barrel.ts': 'export * as ns from "./source";\n',
  'src/outer.ts': 'export { ns } from "./barrel";\n',
  'src/consumer.ts': [
    'import { ns } from "./outer";',
    'ns.chainedUsedFunc();',
    'console.log(ns.chainedUsedConst);',
  ].join('\n'),
});

const chainedFanIn = chainedSymbols.fanInByIdentity ?? {};
const chainedFanInSpace = chainedSymbols.fanInByIdentitySpace ?? {};
const chainedDead = new Set((chainedSymbols.deadProdList ?? [])
  .map((item) => `${item.file}::${item.symbol}`));

assert('NR6. chained namespace re-export function gets exact fan-in',
  chainedFanIn['src/source.ts::chainedUsedFunc'] === 1,
  JSON.stringify(chainedFanIn, null, 2));
assert('NR7. chained namespace re-export const gets exact fan-in',
  chainedFanIn['src/source.ts::chainedUsedConst'] === 1,
  JSON.stringify(chainedFanIn, null, 2));
assert('NR8. chained namespace re-export unused function remains dead',
  chainedDead.has('src/source.ts::chainedUnusedFunc'),
  JSON.stringify(chainedSymbols.deadProdList, null, 2));
assert('NR9. chained namespace re-export unused const remains dead',
  chainedDead.has('src/source.ts::chainedUnusedConst'),
  JSON.stringify(chainedSymbols.deadProdList, null, 2));
assert('NR10. chained namespace re-export does not add broad fan-in to unused members',
  chainedFanInSpace['src/source.ts::chainedUnusedFunc']?.broad === 0 &&
    chainedFanInSpace['src/source.ts::chainedUnusedConst']?.broad === 0,
  JSON.stringify(chainedFanInSpace, null, 2));

const escapeSymbols = runSymbolGraph({
  'src/source.ts': [
    'export function escapeFunc() { return 1; }',
    'export const escapeConst = 2;',
  ].join('\n'),
  'src/barrel.ts': 'export * as ns from "./source";\n',
  'src/consumer.ts': [
    'import { ns } from "./barrel";',
    'function observe(value: unknown) { return value; }',
    'observe(ns);',
  ].join('\n'),
});

const escapeFanInSpace = escapeSymbols.fanInByIdentitySpace ?? {};
const namespaceDiagnostics = escapeSymbols.namespaceReExportDiagnostics ?? [];

assert('NR11. opaque namespace escape keeps target members broad-shadowed',
  escapeFanInSpace['src/source.ts::escapeFunc']?.broad === 1 &&
    escapeFanInSpace['src/source.ts::escapeConst']?.broad === 1,
  JSON.stringify(escapeFanInSpace, null, 2));
assert('NR12. opaque namespace escape is reported as a namespace diagnostic',
  namespaceDiagnostics.some((item) =>
    item.kind === 'opaque-namespace-escape' &&
    item.consumerFile === 'src/consumer.ts' &&
    item.exportedName === 'ns' &&
    item.targetFile === 'src/source.ts' &&
    item.reason === 'namespace-object-escaped'),
  JSON.stringify(namespaceDiagnostics, null, 2));

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

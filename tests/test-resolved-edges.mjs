// PCEF P2a: symbols.json must preserve resolved file-level internal edges.
//
// Symbol fan-in and module reachability are different lenses. A side-effect
// import or broad CJS escape should not keep every export live, but it still
// proves the target file is evaluated/reachable for later P2 BFS.

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

function runSymbolGraph(files) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-resolved-edges-'));
  const src = path.join(dir, 'src');
  const out = path.join(dir, '.audit');
  mkdirSync(src, { recursive: true });
  mkdirSync(out, { recursive: true });
  for (const [name, content] of Object.entries(files)) {
    const file = path.join(src, name);
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, content);
  }
  try {
    execFileSync(process.execPath, [
      path.join(ROOT, 'build-symbol-graph.mjs'),
      '--root', dir,
      '--output', out,
      '--production',
    ], { encoding: 'utf8' });
    return JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function edgeKey(edge) {
  return `${edge.from} -> ${edge.to} :: ${edge.kind} :: typeOnly=${edge.typeOnly}`;
}

function hasEdge(symbols, expected) {
  return (symbols.resolvedInternalEdges ?? []).some((edge) =>
    Object.entries(expected).every(([key, value]) => edge[key] === value));
}

const symbols = runSymbolGraph({
  'named.ts': 'export const named = 1;\n',
  'defaulted.ts': 'export default function defaulted() {}\n',
  'namespace.ts': 'export const member = 1;\nexport const other = 2;\n',
  'types.ts': 'export interface TypeOnly { value: string }\n',
  'side-effect.ts': 'globalThis.sideEffectRan = true;\nexport const hidden = 1;\n',
  'reexport-source.ts': 'export const reexported = 1;\n',
  'star-source.ts': 'export const star = 1;\n',
  'dynamic.ts': 'export const dyn = 1;\n',
  'cjs.js': 'export const cjsNamed = 1;\nexport const cjsEscaped = 2;\nexport const cjsSide = 3;\n',
  'style.css': '.root { color: red; }\n',
  'consumer.ts': [
    'import { named } from "./named";',
    'import defaulted from "./defaulted";',
    'import * as ns from "./namespace";',
    'import type { TypeOnly } from "./types";',
    'import styles from "./style.css?inline";',
    'import "./side-effect";',
    'export { reexported } from "./reexport-source";',
    'export * from "./star-source";',
    'const mod = await import("./dynamic");',
    'mod.dyn;',
    'const { cjsNamed } = require("./cjs.js");',
    'const cjsNs = require("./cjs.js");',
    'use(cjsNs);',
    'require("./cjs.js");',
    'named; defaulted; ns.member; styles; let t: TypeOnly;',
  ].join('\n'),
});

assert('E1. symbols.json advertises resolvedInternalEdges support',
  symbols.meta?.supports?.resolvedInternalEdges === true,
  JSON.stringify(symbols.meta?.supports));
assert('E2. resolvedInternalEdges is an array',
  Array.isArray(symbols.resolvedInternalEdges),
  JSON.stringify(symbols.resolvedInternalEdges));

const expected = [
  { from: 'src/consumer.ts', to: 'src/named.ts', kind: 'import-named', typeOnly: false },
  { from: 'src/consumer.ts', to: 'src/defaulted.ts', kind: 'import-default', typeOnly: false },
  { from: 'src/consumer.ts', to: 'src/namespace.ts', kind: 'import-namespace', typeOnly: false },
  { from: 'src/consumer.ts', to: 'src/types.ts', kind: 'import-named', typeOnly: true },
  { from: 'src/consumer.ts', to: 'src/side-effect.ts', kind: 'import-side-effect', typeOnly: false },
  { from: 'src/consumer.ts', to: 'src/reexport-source.ts', kind: 'reexport-named', typeOnly: false },
  { from: 'src/consumer.ts', to: 'src/star-source.ts', kind: 'reexport-broad', typeOnly: false },
  { from: 'src/consumer.ts', to: 'src/dynamic.ts', kind: 'dynamic-literal', typeOnly: false },
  { from: 'src/consumer.ts', to: 'src/cjs.js', kind: 'cjs-require-exact', typeOnly: false },
  { from: 'src/consumer.ts', to: 'src/cjs.js', kind: 'cjs-namespace-escape', typeOnly: false },
  { from: 'src/consumer.ts', to: 'src/cjs.js', kind: 'cjs-side-effect', typeOnly: false },
];

for (const edge of expected) {
  assert(`E3. resolved edge ${edge.kind} ${edge.to}`,
    hasEdge(symbols, edge),
    (symbols.resolvedInternalEdges ?? []).map(edgeKey).join('\n'));
}

assert('E4. side-effect-only import does not create named fan-in',
  symbols.fanInByIdentity['src/side-effect.ts::hidden'] === 0,
  JSON.stringify(symbols.fanInByIdentity));
assert('E5. side-effect-only CJS require does not create named fan-in',
  symbols.fanInByIdentity['src/cjs.js::cjsSide'] === 0,
  JSON.stringify(symbols.fanInByIdentity));
assert('E6. non-source asset import is not reported as resolver blindness',
  !(symbols.unresolvedInternalSpecifierRecords ?? []).some((record) =>
    record.specifier === './style.css?inline'),
  JSON.stringify(symbols.unresolvedInternalSpecifierRecords));
assert('E7. non-source asset import does not become a JS module reachability edge',
  !(symbols.resolvedInternalEdges ?? []).some((edge) =>
    edge.source === './style.css?inline' || edge.to === 'src/style.css'),
  JSON.stringify(symbols.resolvedInternalEdges));

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

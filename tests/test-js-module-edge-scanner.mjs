import { parseOxcOrThrow } from '../_lib/parse-oxc.mjs';
import {
  MODULE_EDGE_SCANNER_POLICY_VERSION,
  scanJsModuleEdgesFast,
} from '../_lib/js-module-edge-scanner.mjs';

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

function normalizeEdges(edges) {
  return [...edges]
    .map((edge) => ({
      source: edge.source,
      typeOnly: !!edge.typeOnly,
      reExport: !!edge.reExport,
      dynamic: !!edge.dynamic,
    }))
    .sort((a, b) =>
      a.source.localeCompare(b.source) ||
      Number(a.typeOnly) - Number(b.typeOnly) ||
      Number(a.reExport) - Number(b.reExport) ||
      Number(a.dynamic) - Number(b.dynamic));
}

function oxcTopologyEdges(filename, source) {
  const parsed = parseOxcOrThrow(filename, source);
  const edges = [];

  function pushImportExpression(node) {
    const s = node.source;
    if (s && (s.type === 'Literal' || s.type === 'StringLiteral') &&
        typeof s.value === 'string') {
      edges.push({
        source: s.value,
        typeOnly: false,
        reExport: false,
        dynamic: true,
      });
    }
  }

  function walk(node) {
    if (!node || typeof node !== 'object') return;
    if (node.type === 'ImportExpression') pushImportExpression(node);
    for (const key of Object.keys(node)) {
      if (key === 'type' || key === 'start' || key === 'end') continue;
      const value = node[key];
      if (Array.isArray(value)) {
        for (const child of value) walk(child);
      } else if (value && typeof value === 'object' && typeof value.type === 'string') {
        walk(value);
      }
    }
  }

  for (const node of parsed.program.body) {
    if (node.type === 'ImportDeclaration') {
      edges.push({
        source: node.source.value,
        typeOnly: node.importKind === 'type',
        reExport: false,
        dynamic: false,
      });
    } else if (
      (node.type === 'ExportNamedDeclaration' || node.type === 'ExportAllDeclaration') &&
      node.source
    ) {
      const specs = node.specifiers ?? [];
      const allSpecsTypeOnly = specs.length > 0 && specs.every((s) => s.exportKind === 'type');
      edges.push({
        source: node.source.value,
        typeOnly: node.exportKind === 'type' || allSpecsTypeOnly,
        reExport: true,
        dynamic: false,
      });
    }
  }
  walk(parsed.program);
  return normalizeEdges(edges);
}

function assertEquivalentAccepted(label, filename, source) {
  const scan = scanJsModuleEdgesFast(source, { filename });
  const expected = oxcTopologyEdges(filename, source);
  assert(`${label}: scanner accepts file`,
    scan.ok === true &&
      scan.policyVersion === MODULE_EDGE_SCANNER_POLICY_VERSION &&
      scan.mode === 'fast-module-edge',
    JSON.stringify(scan));
  assert(`${label}: scanner edges match Oxc topology edges`,
    JSON.stringify(normalizeEdges(scan.edges ?? [])) === JSON.stringify(expected),
    `scanner=${JSON.stringify(normalizeEdges(scan.edges ?? []))}\noxc=${JSON.stringify(expected)}`);
}

function assertFallback(label, source, reason) {
  const scan = scanJsModuleEdgesFast(source, { filename: 'fixture.ts' });
  assert(label,
    scan.ok === false &&
      scan.mode === 'fallback-required' &&
      scan.risk?.includes(reason),
    JSON.stringify(scan));
}

assertEquivalentAccepted('T1. static imports, re-exports, type edges, and literal dynamic imports',
  'fixture.ts',
  [
    "import def, { named } from './dep';",
    "import type { T } from './types';",
    "import './side-effect';",
    "export { named as renamed } from './dep';",
    "export { type T2 } from './more-types';",
    "export type { T3 } from './even-more-types';",
    "export * from './barrel';",
    "export type * from './type-barrel';",
    "export async function lazy() { return import('./lazy'); }",
  ].join('\n'));

assertEquivalentAccepted('T2. fake module syntax inside comments, strings, regex, and templates is ignored',
  'fixture.ts',
  [
    "// import fake from './comment';",
    "const s = 'export * from \"./string\"';",
    'const d = "import(\\"./double\\")";',
    'const r = /import\\s+fake\\s+from\\s+["\\\']\\.\\/regex["\\\']/;',
    'const t = `export * from "./template"`;',
    "import real from './real';",
  ].join('\n'));

assertEquivalentAccepted('T3. unrelated interpolated template literals are accepted',
  'fixture.ts',
  [
    "const name = 'world';",
    "const message = `hello ${name}`;",
    "import real from './real';",
    "export async function lazy() { return import('./lazy'); }",
  ].join('\n'));

assertEquivalentAccepted('T4. import and export attributes with string specifiers are accepted',
  'fixture.ts',
  [
    "import data from './data.json' with { type: 'json' };",
    "export * from './other.json' assert { type: 'json' };",
  ].join('\n'));

{
  const scan = scanJsModuleEdgesFast("import value from './dep';\n", { filename: 'fixture.ts' });
  assert('T5. accepted edges preserve line numbers',
    scan.ok === true && scan.edges?.[0]?.line === 1,
    JSON.stringify(scan));
}

assertFallback('T6. non-literal dynamic import falls back',
  "export function load(name) { return import(name); }",
  'non-literal-dynamic-import');

assertFallback('T7. template dynamic import falls back',
  'export function load(name) { return import(`./${name}.ts`); }',
  'template-dynamic-import');

assertFallback('T8. require call falls back',
  "const value = require('./cjs');",
  'require-call');

assertFallback('T9. import.meta.glob falls back',
  "const routes = import.meta.glob('./routes/*.ts');",
  'import-meta-glob');

assertFallback('T10. TypeScript import-equals falls back',
  "import foo = require('./foo');",
  'ts-import-equals');

assertFallback('T11. TypeScript export assignment falls back',
  'export = foo;',
  'ts-export-assignment');

assertFallback('T12. ambient module declaration falls back',
  "declare module './virtual' { export const x: number; }",
  'ts-ambient-module');

assertFallback('T13. JSX text currently falls back instead of risking fake edges',
  "export const View = () => <span>import fake from './jsx-text'</span>;",
  'unsupported-syntax');

{
  const source = Array.from({ length: 6000 }, (_, i) =>
    `const s${i} = "value-${i}";`).join('\n');
  const started = Date.now();
  const scan = scanJsModuleEdgesFast(source, { filename: 'many-strings.ts' });
  const elapsedMs = Date.now() - started;
  assert('T14. scanner handles many string literals without quadratic line scans',
    scan.ok === true &&
      scan.edges?.length === 0 &&
      elapsedMs < 1500,
    JSON.stringify({ elapsedMs, ok: scan.ok, edges: scan.edges?.length, risk: scan.risk }));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

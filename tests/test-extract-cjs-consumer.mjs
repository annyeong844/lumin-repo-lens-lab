// PCEF P0: CommonJS consumers must be represented in the per-file
// extractor before SAFE_FIX ranking is widened. These tests stay at the
// extractor layer so failures point at `_lib/extract-ts.mjs`, not the
// resolver or graph builder.

import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { extractDefinitionsAndUses } from '../_lib/extract-ts.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}

function extractInfo(source) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-cjs-extract-'));
  const file = path.join(dir, 'consumer.js');
  writeFileSync(file, source);
  try {
    return extractDefinitionsAndUses(file);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function extractSource(source) {
  return extractInfo(source).uses;
}

function hasUse(uses, kind, name) {
  return uses.some((u) => u.kind === kind && (name === undefined || u.name === name));
}

function usesFor(source) {
  return extractInfo(source).uses;
}

{
  const uses = extractSource('require("./exporter");\n');
  assert('CJS1. bare require statement emits side-effect-only use',
    hasUse(uses, 'cjs-side-effect-only', '*'),
    JSON.stringify(uses));
  assert('CJS1b. bare require statement does not emit exact consumer',
    !uses.some((u) => u.kind === 'cjs-require-exact' || u.kind === 'cjs-namespace-member'),
    JSON.stringify(uses));
}

{
  const uses = extractSource('const { foo, bar: renamed } = require("./exporter");\n');
  assert('CJS2. require destructuring emits exact foo consumer',
    hasUse(uses, 'cjs-require-exact', 'foo'),
    JSON.stringify(uses));
  assert('CJS2b. require destructuring emits exact aliased property consumer',
    hasUse(uses, 'cjs-require-exact', 'bar'),
    JSON.stringify(uses));
}

{
  const uses = extractSource('const mod = require("./exporter");\nmod.foo();\nmod.bar;\n');
  assert('CJS3. const namespace require emits exact member call consumer',
    hasUse(uses, 'cjs-namespace-member', 'foo'),
    JSON.stringify(uses));
  assert('CJS3b. const namespace require emits exact member read consumer',
    hasUse(uses, 'cjs-namespace-member', 'bar'),
    JSON.stringify(uses));
}

{
  const uses = extractSource('const mod = require("./exporter");\nconst { foo, bar: renamed } = mod;\n');
  assert('CJS3c. const namespace require alias destructuring emits exact foo consumer',
    hasUse(uses, 'cjs-namespace-member', 'foo'),
    JSON.stringify(uses));
  assert('CJS3d. const namespace require alias destructuring emits exact aliased member consumer',
    hasUse(uses, 'cjs-namespace-member', 'bar'),
    JSON.stringify(uses));
}

{
  const uses = extractSource('const mod = require("./exporter");\nconst { foo, ...rest } = mod;\n');
  assert('CJS3e. require alias rest destructuring stays broad',
    hasUse(uses, 'cjs-namespace-escape', '*') &&
      !hasUse(uses, 'cjs-namespace-member', 'foo'),
    JSON.stringify(uses));
}

{
  const uses = extractSource('require("./exporter").foo();\n');
  assert('CJS4. direct require member call emits exact member consumer',
    hasUse(uses, 'cjs-namespace-member', 'foo'),
    JSON.stringify(uses));
}

{
  const uses = extractSource('const mod = require("./exporter");\nuse(mod);\n');
  assert('CJS5. escaping require namespace emits broad escape',
    hasUse(uses, 'cjs-namespace-escape', '*'),
    JSON.stringify(uses));
}

{
  const uses = extractSource('let mod = require("./exporter");\nmod.foo();\n');
  assert('CJS6. non-const require namespace is broad escape, not exact',
    hasUse(uses, 'cjs-namespace-escape', '*') &&
      !hasUse(uses, 'cjs-namespace-member', 'foo'),
    JSON.stringify(uses));
}

{
  const uses = extractSource('module.exports = require("./exporter");\n');
  assert('CJS7. module.exports require emits broad re-export',
    hasUse(uses, 'cjs-reexport-broad', '*'),
    JSON.stringify(uses));
}

{
  const info = extractInfo('const target = "./exporter";\nrequire(target);\n');
  assert('CJS8. dynamic require records CJS opacity',
    info.cjsRequireOpacity?.some((entry) => entry.kind === 'dynamic-require' && entry.line === 2),
    JSON.stringify(info));
  assert('CJS8b. dynamic require does not pretend to be exact CJS consumer',
    !info.uses.some((u) => u.kind?.startsWith('cjs-')),
    JSON.stringify(info.uses));
}

{
  const info = extractInfo([
    'import path from "node:path";',
    'import { createRequire } from "node:module";',
    'const require = createRequire(import.meta.url);',
    'export function getCurrentVersion() {',
    '  return require(path.resolve(import.meta.dirname, "../../package.json")).version;',
    '}',
    '',
  ].join('\n'));
  assert('CJS9. static package.json metadata require does not create CJS opacity',
    !info.cjsRequireOpacity?.some((entry) => entry.kind === 'dynamic-require'),
    JSON.stringify(info));
  assert('CJS9b. static package.json metadata require does not pretend to be a CJS consumer',
    !info.uses.some((u) => u.kind?.startsWith('cjs-')),
    JSON.stringify(info.uses));
}

{
  const uses = usesFor('const mod = require("./exporter");\nmod["foo"]();\nrequire("./exporter")["bar"];\n');
  assert('CJS10. static computed CJS members are exact consumers',
    hasUse(uses, 'cjs-namespace-member', 'foo') &&
      hasUse(uses, 'cjs-namespace-member', 'bar') &&
      !hasUse(uses, 'cjs-namespace-escape', '*'),
    JSON.stringify(uses));
}

{
  const uses = usesFor([
    'const mod = require("./exporter");',
    'if (mod) mod.foo();',
    'mod && mod.bar();',
    'typeof mod !== "undefined" && mod.baz;',
    '',
  ].join('\n'));
  assert('CJS11. simple guard reads do not degrade exact CJS member consumers',
    hasUse(uses, 'cjs-namespace-member', 'foo') &&
      hasUse(uses, 'cjs-namespace-member', 'bar') &&
      hasUse(uses, 'cjs-namespace-member', 'baz') &&
      !hasUse(uses, 'cjs-namespace-escape', '*'),
    JSON.stringify(uses));
}

{
  const uses = usesFor('const mod = require("./exporter");\nif ("foo" in mod) mod.foo();\n');
  assert('CJS12. key introspection remains broad CJS evidence',
    hasUse(uses, 'cjs-namespace-escape', '*') &&
      !hasUse(uses, 'cjs-namespace-member', 'foo'),
    JSON.stringify(uses));
}

{
  const uses = usesFor('const mod = require("./exporter");\nfunction f(mod) { mod.foo(); }\n');
  assert('CJS13. shadowed function parameter does not exact-protect outer require',
    !hasUse(uses, 'cjs-namespace-member', 'foo') &&
      !hasUse(uses, 'cjs-namespace-escape', '*'),
    JSON.stringify(uses));
}

{
  const uses = usesFor([
    'const mod = require("./exporter");',
    'mod.foo = 1;',
    'mod.bar++;',
    'delete mod.baz;',
    '',
  ].join('\n'));
  assert('CJS14. namespace member writes degrade to broad escape, not exact consumers',
    hasUse(uses, 'cjs-namespace-escape', '*') &&
      !hasUse(uses, 'cjs-namespace-member', 'foo') &&
      !hasUse(uses, 'cjs-namespace-member', 'bar') &&
      !hasUse(uses, 'cjs-namespace-member', 'baz'),
    JSON.stringify(uses));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

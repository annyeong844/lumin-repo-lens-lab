// CommonJS export-surface extraction is a blind-zone guard, not full CJS
// semantics. The extractor should record mechanically obvious named exports
// and separately record opaque export forms that can hide named exports.

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

function extractSource(source) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-cjs-export-surface-'));
  const file = path.join(dir, 'exporter.cjs');
  writeFileSync(file, source);
  try {
    return extractDefinitionsAndUses(file);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function hasExact(surface, name, kind) {
  return (surface?.exact ?? []).some((entry) =>
    entry.name === name && (kind === undefined || entry.kind === kind));
}

function hasOpaque(surface, kind) {
  return (surface?.opaque ?? []).some((entry) => entry.kind === kind);
}

{
  const info = extractSource([
    'exports.foo = 1;',
    'module.exports.bar = 2;',
    'exports["quoted"] = 3;',
    'module.exports = { baz: 4, renamed: localValue };',
    'exports[dynamicName] = 5;',
    'module.exports = makeExports();',
    '',
  ].join('\n'));

  assert('CJSX1. exact exports.foo assignment is recorded',
    hasExact(info.cjsExportSurface, 'foo', 'exports-member'),
    JSON.stringify(info.cjsExportSurface));
  assert('CJSX1b. exact module.exports.bar assignment is recorded',
    hasExact(info.cjsExportSurface, 'bar', 'module-exports-member'),
    JSON.stringify(info.cjsExportSurface));
  assert('CJSX1c. exact quoted exports member assignment is recorded',
    hasExact(info.cjsExportSurface, 'quoted', 'exports-member'),
    JSON.stringify(info.cjsExportSurface));
  assert('CJSX1d. exact module.exports object properties are recorded',
    hasExact(info.cjsExportSurface, 'baz', 'module-exports-object') &&
      hasExact(info.cjsExportSurface, 'renamed', 'module-exports-object'),
    JSON.stringify(info.cjsExportSurface));
  assert('CJSX1e. computed export name is recorded as opaque',
    hasOpaque(info.cjsExportSurface, 'computed-export-name'),
    JSON.stringify(info.cjsExportSurface));
  assert('CJSX1f. non-object module.exports assignment is recorded as opaque',
    hasOpaque(info.cjsExportSurface, 'module-exports-assignment'),
    JSON.stringify(info.cjsExportSurface));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

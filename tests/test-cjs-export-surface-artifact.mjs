// CommonJS export surface should survive extraction into symbols.json so
// downstream classifiers can treat CJS files as supported or opaque by fact,
// not by guessing from file extension alone.

import { execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}

const REPO = process.cwd();

function runNode(args, cwd = REPO) {
  return execFileSync(process.execPath, args, {
    cwd,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  });
}

const root = mkdtempSync(path.join(os.tmpdir(), 'lrl-cjs-export-artifact-'));
const out = path.join(root, '.audit');
try {
  mkdirSync(path.join(root, 'src'), { recursive: true });
  writeFileSync(path.join(root, 'package.json'), JSON.stringify({ private: true }, null, 2));
  writeFileSync(path.join(root, 'src', 'exporter.cjs'), [
    'exports.foo = 1;',
    'module.exports.bar = 2;',
    'module.exports = { baz: 3 };',
    'exports[dynamicName] = 4;',
    '',
  ].join('\n'));

  runNode(['build-symbol-graph.mjs', '--root', root, '--output', out]);
  const symbols = JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
  const surface = symbols.cjsExportSurfaceByFile?.['src/exporter.cjs'];

  assert('CJSXA1. symbols.json advertises CJS export surface support',
    symbols.meta?.supports?.cjsExportSurface === true,
    JSON.stringify(symbols.meta?.supports, null, 2));
  assert('CJSXA2. symbols.json keeps exact CJS export names by file',
    surface?.exact?.some((entry) => entry.name === 'foo' && entry.kind === 'exports-member') &&
      surface?.exact?.some((entry) => entry.name === 'bar' && entry.kind === 'module-exports-member') &&
      surface?.exact?.some((entry) => entry.name === 'baz' && entry.kind === 'module-exports-object'),
    JSON.stringify(surface, null, 2));
  assert('CJSXA3. symbols.json keeps opaque CJS export forms by file',
    surface?.opaque?.some((entry) => entry.kind === 'computed-export-name'),
    JSON.stringify(surface, null, 2));
} finally {
  rmSync(root, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

// Integration guard for the CJS support slices. Exact CJS exports,
// alias destructuring consumers, and dynamic require opacity must survive
// together after branch integration.

import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
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

function runSymbolGraph(files) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'lrl-cjs-integration-'));
  const out = path.join(root, '.audit');
  try {
    mkdirSync(path.join(root, 'src'), { recursive: true });
    writeFileSync(path.join(root, 'package.json'), JSON.stringify({ private: true }, null, 2));
    for (const [name, content] of Object.entries(files)) {
      writeFileSync(path.join(root, 'src', name), content);
    }
    execFileSync(process.execPath, [
      path.join(REPO, 'build-symbol-graph.mjs'),
      '--root', root,
      '--output', out,
      '--production',
    ], { cwd: REPO, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
    return JSON.parse(readFileSync(path.join(out, 'symbols.json'), 'utf8'));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

{
  const symbols = runSymbolGraph({
    'exporter.cjs': [
      'exports.foo = 1;',
      'module.exports.bar = 2;',
      'exports[dynamicName] = 3;',
      '',
    ].join('\n'),
    'consumer.js': [
      'const mod = require("./typed-exporter.js");',
      'const { foo } = mod;',
      'const target = "./typed-exporter.js";',
      'require(target);',
      '',
    ].join('\n'),
    'typed-exporter.js': 'export const foo = 1;\nexport const bar = 2;\n',
  });

  const surface = symbols.cjsExportSurfaceByFile?.['src/exporter.cjs'];
  assert('CJSI1. exact and opaque CJS export surface facts coexist',
    symbols.meta?.supports?.cjsExportSurface === true &&
      surface?.exact?.some((entry) => entry.name === 'foo') &&
      surface?.exact?.some((entry) => entry.name === 'bar') &&
      surface?.opaque?.some((entry) => entry.kind === 'computed-export-name'),
    JSON.stringify(surface, null, 2));

  assert('CJSI2. namespace alias destructuring protects exact ESM export',
    symbols.fanInByIdentity?.['src/typed-exporter.js::foo'] === 1 &&
      symbols.fanInByIdentity?.['src/typed-exporter.js::bar'] === 0,
    JSON.stringify(symbols.fanInByIdentity, null, 2));

  assert('CJSI3. dynamic require is reported as CJS opacity evidence',
    symbols.cjsRequireOpacity?.some((entry) =>
      entry.consumerFile === 'src/consumer.js' &&
      entry.kind === 'dynamic-require' &&
      entry.line === 4),
    JSON.stringify(symbols.cjsRequireOpacity, null, 2));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

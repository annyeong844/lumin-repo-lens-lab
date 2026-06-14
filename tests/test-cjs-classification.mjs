// PCEF P0 integration guard: CJS exact consumers should affect
// fanInByIdentity, while side-effect-only require should not keep every
// export alive.

import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}

function runSymbolGraph(files) {
  const dir = mkdtempSync(path.join(os.tmpdir(), 'lrl-cjs-graph-'));
  const src = path.join(dir, 'src');
  const out = path.join(dir, '.audit');
  mkdirSync(src, { recursive: true });
  mkdirSync(out, { recursive: true });
  for (const [name, content] of Object.entries(files)) {
    writeFileSync(path.join(src, name), content);
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

{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\nexport const bar = 2;\n',
    'consumer.js': 'const { foo } = require("./exporter.js");\n',
  });
  assert('G1. cjs destructuring increases exact fan-in',
    symbols.fanInByIdentity['src/exporter.js::foo'] === 1,
    JSON.stringify(symbols.fanInByIdentity));
  assert('G1b. unrelated sibling export remains dead',
    symbols.fanInByIdentity['src/exporter.js::bar'] === 0 &&
      symbols.deadProdList.some((d) => d.file === 'src/exporter.js' && d.symbol === 'bar'),
    JSON.stringify({ fanIn: symbols.fanInByIdentity, dead: symbols.deadProdList }));
}

{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\nexport const bar = 2;\n',
    'consumer.js': 'require("./exporter.js");\n',
  });
  assert('G2. bare cjs side-effect require does not protect named exports',
    symbols.fanInByIdentity['src/exporter.js::foo'] === 0 &&
      symbols.fanInByIdentity['src/exporter.js::bar'] === 0,
    JSON.stringify(symbols.fanInByIdentity));
}

{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\nexport const bar = 2;\n',
    'consumer.js': 'const mod = require("./exporter.js");\nmod.foo();\n',
  });
  assert('G3. cjs namespace member increases exact fan-in',
    symbols.fanInByIdentity['src/exporter.js::foo'] === 1 &&
      symbols.fanInByIdentity['src/exporter.js::bar'] === 0,
    JSON.stringify(symbols.fanInByIdentity));
}

{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\nexport const bar = 2;\n',
    'consumer.js': 'const mod = require("./exporter.js");\nconst { foo } = mod;\n',
  });
  assert('G3b. cjs namespace alias destructuring increases exact fan-in',
    symbols.fanInByIdentity['src/exporter.js::foo'] === 1 &&
      symbols.fanInByIdentity['src/exporter.js::bar'] === 0,
    JSON.stringify(symbols.fanInByIdentity));
}

{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\nexport const bar = 2;\n',
    'consumer.js': 'const mod = require("./exporter.js");\nuse(mod);\n',
  });
  assert('G4. cjs namespace escape marks file namespace-shadowed',
    symbols.deadTotal === 2 && symbols.trulyDead === 0,
    JSON.stringify({ deadTotal: symbols.deadTotal, trulyDead: symbols.trulyDead }));
}

{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\n',
    'consumer.js': 'const target = "./exporter.js";\nrequire(target);\n',
  });
  assert('G5. dynamic cjs require is preserved as opacity evidence',
    symbols.cjsRequireOpacity?.some((entry) =>
      entry.consumerFile === 'src/consumer.js' &&
      entry.kind === 'dynamic-require' &&
      entry.line === 2),
    JSON.stringify(symbols.cjsRequireOpacity));
}

{
  const symbols = runSymbolGraph({
    'consumer.js': [
      'import path from "node:path";',
      'import { createRequire } from "node:module";',
      'const require = createRequire(import.meta.url);',
      'export function getCurrentVersion() {',
      '  return require(path.resolve(import.meta.dirname, "../package.json")).version;',
      '}',
      '',
    ].join('\n'),
  });
  assert('G6. static package.json require does not create dynamic CJS opacity',
    (symbols.cjsRequireOpacity ?? []).length === 0,
    JSON.stringify(symbols.cjsRequireOpacity));
}

{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\nexport const bar = 2;\nexport const baz = 3;\nexport const unused = 4;\n',
    'consumer.js': [
      'const mod = require("./exporter.js");',
      'if (mod) mod.foo();',
      'mod && mod["bar"];',
      'require("./exporter.js")["baz"];',
      '',
    ].join('\n'),
  });
  assert('G7. guarded and static computed CJS members increase exact fan-in',
    symbols.fanInByIdentity['src/exporter.js::foo'] === 1 &&
      symbols.fanInByIdentity['src/exporter.js::bar'] === 1 &&
      symbols.fanInByIdentity['src/exporter.js::baz'] === 1 &&
      symbols.fanInByIdentity['src/exporter.js::unused'] === 0,
    JSON.stringify(symbols.fanInByIdentity));
}

{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\nexport const bar = 2;\n',
    'consumer.js': 'const mod = require("./exporter.js");\nif ("foo" in mod) mod.foo();\n',
  });
  assert('G8. CJS key introspection stays broad and prevents truly-dead confidence',
    symbols.deadTotal === 2 &&
      symbols.trulyDead === 0 &&
      symbols.fanInByIdentity['src/exporter.js::foo'] === 0,
    JSON.stringify({
      deadTotal: symbols.deadTotal,
      trulyDead: symbols.trulyDead,
      fanIn: symbols.fanInByIdentity,
    }));
}

{
  const symbols = runSymbolGraph({
    'exporter.js': 'export const foo = 1;\nexport const bar = 2;\n',
    'consumer.js': [
      'const mod = require("./exporter.js");',
      'mod.foo = 10;',
      '',
    ].join('\n'),
  });
  assert('G9. CJS namespace member write is broad escape, not exact fan-in',
    symbols.deadTotal === 2 &&
      symbols.trulyDead === 0 &&
      symbols.fanInByIdentity['src/exporter.js::foo'] === 0 &&
      symbols.fanInByIdentity['src/exporter.js::bar'] === 0,
    JSON.stringify({
      deadTotal: symbols.deadTotal,
      trulyDead: symbols.trulyDead,
      fanIn: symbols.fanInByIdentity,
    }));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

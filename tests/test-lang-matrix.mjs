// Regression guard for language dispatch (v1.8.0).
//
// Before v1.8.0 every JS-family file was force-parsed with `lang: 'ts'`.
// That silently failed on .jsx (JSX syntax not accepted in TS mode) — pure
// React+JS codebases produced empty def/use lists so everything looked
// dead. `.cjs`/`.mts`/`.cts`/`.jsx` were additionally missing from the
// default collection list in build-symbol-graph and measure-topology, so
// those files weren't even walked.
//
// This suite creates a fixture that exercises each extension oxc-parser
// accepts — .ts, .tsx, .js, .jsx, .mjs, .cjs, .mts, .cts, .d.ts — and asserts
// build-symbol-graph:
//   - walks all of them
//   - produces defs from each
//   - reports correct dead/used for a cross-file import path

import { execSync } from 'node:child_process';
import { writeFileSync, mkdirSync, rmSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { langForFile, canContainJsx, nonJsLangForFile } from '../_lib/lang.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const FX = '/tmp/fx-lang-matrix';
const OUT = '/tmp/out-lang-matrix';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── Unit-level: langForFile / canContainJsx / nonJsLangForFile ──
assert('L1. langForFile(.tsx) = tsx', langForFile('a.tsx') === 'tsx');
assert('L2. langForFile(.jsx) = jsx', langForFile('a.jsx') === 'jsx');
assert('L3. langForFile(.ts) = ts',   langForFile('a.ts') === 'ts');
assert('L4. langForFile(.js) = js',   langForFile('a.js') === 'js');
assert('L5. langForFile(.mjs) = js',  langForFile('a.mjs') === 'js');
assert('L6. langForFile(.cjs) = js',  langForFile('a.cjs') === 'js');
assert('L7. langForFile(.mts) = ts',  langForFile('a.mts') === 'ts');
assert('L8. langForFile(.cts) = ts',  langForFile('a.cts') === 'ts');
assert('L9. langForFile(.d.ts) = dts', langForFile('a.d.ts') === 'dts');
assert('L10. langForFile(.d.mts) = dts', langForFile('a.d.mts') === 'dts');
assert('L11. langForFile(.d.cts) = dts', langForFile('a.d.cts') === 'dts');
assert('L12. langForFile(.py) = null', langForFile('a.py') === null);
assert('L13. langForFile(.go) = null', langForFile('a.go') === null);

assert('L14. canContainJsx(.tsx) = true',  canContainJsx('a.tsx') === true);
assert('L15. canContainJsx(.jsx) = true',  canContainJsx('a.jsx') === true);
assert('L16. canContainJsx(.ts) = false',  canContainJsx('a.ts') === false);
assert('L17. canContainJsx(.js) = false',  canContainJsx('a.js') === false);

assert('L18. nonJsLangForFile(.py) = python',  nonJsLangForFile('a.py') === 'python');
assert('L19. nonJsLangForFile(.go) = go',      nonJsLangForFile('a.go') === 'go');
assert('L20. nonJsLangForFile(.ts) = null',    nonJsLangForFile('a.ts') === null);

// ── Integration: full build-symbol-graph scan across extension matrix ──
rmSync(FX, { recursive: true, force: true });
rmSync(OUT, { recursive: true, force: true });
mkdirSync(path.join(FX, 'src'), { recursive: true });
writeFileSync(path.join(FX, 'package.json'), '{"name":"lang-matrix","type":"module"}');

// JSX: the original bug — App.jsx imports a component from Button.jsx
writeFileSync(path.join(FX, 'src/Button.jsx'),
  'export function Button({ label }) { return <button>{label}</button>; }\n');
writeFileSync(path.join(FX, 'src/App.jsx'),
  "import { Button } from './Button.jsx';\n" +
  'export function App() { return <div><Button label="hi" /></div>; }\n');

// JSX-in-JS: common in Next.js/React projects that use .js route modules.
writeFileSync(path.join(FX, 'src/JsxInJs.js'),
  'export function JsxInJs() { return <section>JSX in JS</section>; }\n');
writeFileSync(path.join(FX, 'src/JsxInJsConsumer.js'),
  "import { JsxInJs } from './JsxInJs.js';\n" +
  'export const rendered = JsxInJs();\n');

// CJS: CommonJS module, default export consumed by another file
writeFileSync(path.join(FX, 'src/legacyHelper.cjs'),
  'export const cjsHelper = (x) => x + 1;\n' +
  'export const cjsUnused = () => 0;\n');
writeFileSync(path.join(FX, 'src/legacyConsumer.mjs'),
  "import { cjsHelper } from './legacyHelper.cjs';\n" +
  'export const result = cjsHelper(1);\n');

// MTS/CTS
writeFileSync(path.join(FX, 'src/typed.mts'),
  'export const mtsValue: number = 1;\n');
writeFileSync(path.join(FX, 'src/legacy.cts'),
  'export const ctsValue = 2;\n');

// Standard TS
writeFileSync(path.join(FX, 'src/classic.ts'),
  "import { cjsHelper } from './legacyHelper.cjs';\n" +
  "import { mtsValue } from './typed.mjs';\n" + // .mjs → resolver should find .mts
  'export const classic = cjsHelper(mtsValue);\n');

// Declaration files: Nuxt-style declaration-only value exports.
writeFileSync(path.join(FX, 'src/declarations.d.ts'),
  'export const runtimeDependencies: string[];\n' +
  'export interface PublicDeclaration { enabled: boolean }\n');

try {
  execSync(`node build-symbol-graph.mjs --root ${FX} --output ${OUT}`,
    { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });
  assert('I1. build-symbol-graph exits 0 on mixed-extension fixture', true);
} catch (e) {
  assert('I1. build-symbol-graph exits 0 on mixed-extension fixture',
    false, e.stderr?.toString().slice(0, 400) ?? e.message);
}

const sym = JSON.parse(readFileSync(path.join(OUT, 'symbols.json'), 'utf8'));

assert('I2. all 10 files walked (was 0 for pure-JSX repo pre-1.8.0)',
  sym.files === 10,
  `got files=${sym.files}, expected 10`);

assert('I2b. JSX syntax inside .js files parses without blind-zone entries',
  (sym.filesWithParseErrors ?? []).length === 0,
  `parse errors: ${(sym.filesWithParseErrors ?? []).join(', ')}`);

assert('I3. totalDefs > 0 (JSX files parse cleanly, was 0 pre-1.8.0)',
  sym.totalDefs >= 9,
  `got totalDefs=${sym.totalDefs}`);

// Dead list shape check
const deadSymbols = new Set(sym.deadProdList.map((x) => x.symbol));

assert('I4. cjsHelper NOT dead (used across .cjs → .mjs + .ts)',
  !deadSymbols.has('cjsHelper'),
  `cjsHelper unexpectedly in dead list`);

assert('I5. cjsUnused IS dead (no cross-file consumer)',
  deadSymbols.has('cjsUnused'),
  `cjsUnused missing from dead list — .cjs file not scanned?`);

assert('I6. Button NOT dead (used by App.jsx via import)',
  !deadSymbols.has('Button'),
  `Button unexpectedly in dead list — .jsx not parsed?`);

assert('I7. mtsValue NOT dead (used by classic.ts via .mjs spec)',
  !deadSymbols.has('mtsValue'),
  `mtsValue unexpectedly in dead list — .mts not resolved from .mjs spec?`);

assert('I8. JsxInJs NOT dead (used across .js files containing JSX)',
  !deadSymbols.has('JsxInJs'),
  `JsxInJs unexpectedly in dead list — .js JSX fallback missing?`);

assert('I9. .d.ts declaration-only value export parsed as a definition',
  sym.defIndex?.['src/declarations.d.ts']?.runtimeDependencies,
  `defIndex entry missing: ${JSON.stringify(sym.defIndex?.['src/declarations.d.ts'])}`);

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

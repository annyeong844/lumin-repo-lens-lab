// Tests for collectFiles() language-filter correctness.
// Self-contained fixture under /tmp/fixture-collect:
//   main.py            ← root .py
//   main.go            ← root .go
//   some_test.go       ← root *_test.go  (Go test convention)
//   build-tool.mjs     ← root .mjs (should leak into py/go scans per user report)
//   root-entry.ts      ← root .ts  (FP-13 legitimate root entry for TS)
//   package.json
//   src/a.ts, src/b.ts
//   tests/a.test.ts, tests/thing_test.py
//   pkg/worker.go, pkg/worker_test.go
import { collectFiles } from '../_lib/collect-files.mjs';
import { writeFileSync, mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';

const ROOT = '/tmp/fixture-collect';
const ROOT_ABS = path.resolve(ROOT);
const VENDOR_ROOT = '/tmp/vendor/fixture-collect-root-segment';
const VENDOR_ROOT_ABS = path.resolve(VENDOR_ROOT);
rmSync(ROOT, { recursive: true, force: true });
rmSync(VENDOR_ROOT, { recursive: true, force: true });
mkdirSync(path.join(ROOT, 'src'), { recursive: true });
mkdirSync(path.join(ROOT, 'src', 'nested'), { recursive: true });
mkdirSync(path.join(ROOT, 'tests'), { recursive: true });
mkdirSync(path.join(ROOT, 'runtime-tests', 'workerd'), { recursive: true });
mkdirSync(path.join(ROOT, 'test-utils'), { recursive: true });
mkdirSync(path.join(ROOT, 'pkg'), { recursive: true });
mkdirSync(path.join(ROOT, 'output', 'nested'), { recursive: true });
writeFileSync(path.join(ROOT, 'package.json'), '{"name":"fx","type":"module"}');
writeFileSync(path.join(ROOT, 'main.py'), 'def hello(): return "py"\n');
writeFileSync(path.join(ROOT, 'main.go'), 'package main\nfunc main() {}\n');
writeFileSync(path.join(ROOT, 'some_test.go'), 'package main\nfunc TestFoo() {}\n');
writeFileSync(path.join(ROOT, 'build-tool.mjs'), 'export const ver = 1;\n');
writeFileSync(path.join(ROOT, 'root-entry.ts'), 'export const entry = true;\n');
writeFileSync(path.join(ROOT, 'src/a.ts'),
  'export const x = 1;\nexport async function lazy() { return import("./b"); }\n');
writeFileSync(path.join(ROOT, 'src/b.ts'),
  'export const y = 2;\nconst internal = 3;\nexport { internal as publicName };\n');
writeFileSync(path.join(ROOT, 'src/build-index.ts'),
  'export const buildIndex = true;\n');
writeFileSync(path.join(ROOT, 'src/socket-test-support.ts'),
  'export const socketTestSupport = true;\n');
writeFileSync(path.join(ROOT, 'src/skip-me.js'),
  'export const skipMe = true;\n');
writeFileSync(path.join(ROOT, 'src/nested/exact-file.js'),
  'export const exactFile = true;\n');
writeFileSync(path.join(ROOT, 'tests/a.test.ts'),
  "import { x } from '../src/a';\nconsole.log(x);\n");
writeFileSync(path.join(ROOT, 'runtime-tests/workerd/index.ts'),
  'export default { fetch() { return new Response("ok"); } };\n');
writeFileSync(path.join(ROOT, 'test-utils/helper.ts'),
  'export const testHelper = true;\n');
writeFileSync(path.join(ROOT, 'tests/thing_test.py'), 'def test_x(): pass\n');
writeFileSync(path.join(ROOT, 'pkg/worker.go'), 'package pkg\nfunc Worker() {}\n');
writeFileSync(path.join(ROOT, 'pkg/worker_test.go'),
  'package pkg\nfunc TestWorker() {}\n');
writeFileSync(path.join(ROOT, 'output/generated.ts'), 'export const generated = true;\n');
writeFileSync(path.join(ROOT, 'output/nested/generated2.ts'), 'export const generated2 = true;\n');
mkdirSync(path.join(VENDOR_ROOT, 'src', 'vendor'), { recursive: true });
writeFileSync(path.join(VENDOR_ROOT, 'package.json'), '{"name":"vendor-root","type":"module"}');
writeFileSync(path.join(VENDOR_ROOT, 'src/keep.ts'), 'export const keep = true;\n');
writeFileSync(path.join(VENDOR_ROOT, 'src/vendor/skip.ts'), 'export const skip = true;\n');

const rel = (files) => files.map((f) =>
  path.relative(ROOT_ABS, f).split(path.sep).join('/')
).sort();
const relVendor = (files) => files.map((f) =>
  path.relative(VENDOR_ROOT_ABS, f).split(path.sep).join('/')
).sort();

let failed = 0;
let passed = 0;
function assert(label, ok, detail) {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── Bug class A: root .mjs / .ts leak into non-JS scans ──────────
{
  const py = rel(collectFiles(ROOT, { languages: ['py'], includeTests: true }));
  assert(
    'A1. languages=[py] does not include root .mjs files',
    !py.some((f) => f.endsWith('.mjs')),
    `leaked: ${JSON.stringify(py.filter((f) => f.endsWith('.mjs')))}`,
  );
  assert(
    'A2. languages=[py] does not include root .ts files',
    !py.some((f) => f.endsWith('.ts')),
    `leaked: ${JSON.stringify(py.filter((f) => f.endsWith('.ts')))}`,
  );
  assert(
    'A3. languages=[py] returns only .py files',
    py.every((f) => f.endsWith('.py')),
    `non-py present: ${JSON.stringify(py.filter((f) => !f.endsWith('.py')))}`,
  );
}
{
  const go = rel(collectFiles(ROOT, { languages: ['go'], includeTests: true }));
  assert(
    'A4. languages=[go] does not include root .mjs files',
    !go.some((f) => f.endsWith('.mjs')),
    `leaked: ${JSON.stringify(go.filter((f) => f.endsWith('.mjs')))}`,
  );
  assert(
    'A5. languages=[go] returns only .go files',
    go.every((f) => f.endsWith('.go')),
    `non-go present: ${JSON.stringify(go.filter((f) => !f.endsWith('.go')))}`,
  );
}

// ── Bug class B: root-level .py / .go are silently missed ───────
{
  const py = rel(collectFiles(ROOT, { languages: ['py'], includeTests: true }));
  assert(
    'B1. languages=[py] includes root main.py',
    py.includes('main.py'),
    `got: ${JSON.stringify(py)}`,
  );
}
{
  const go = rel(collectFiles(ROOT, { languages: ['go'], includeTests: true }));
  assert(
    'B2. languages=[go] includes root main.go',
    go.includes('main.go'),
    `got: ${JSON.stringify(go)}`,
  );
  assert(
    'B3. languages=[go] includes root some_test.go (when includeTests)',
    go.includes('some_test.go'),
    `got: ${JSON.stringify(go)}`,
  );
}

// ── Bug class C: includeTests=false only filters JS test naming ──
{
  const pyProd = rel(collectFiles(ROOT, { languages: ['py'], includeTests: false }));
  assert(
    'C1. includeTests=false drops *_test.py (pytest convention)',
    !pyProd.some((f) => /(^|\/)[^/]*_test\.py$/.test(f)),
    `_test.py leaked: ${JSON.stringify(pyProd.filter((f) => /_test\.py$/.test(f)))}`,
  );
}
{
  const goProd = rel(collectFiles(ROOT, { languages: ['go'], includeTests: false }));
  assert(
    'C2. includeTests=false drops *_test.go (go test convention)',
    !goProd.some((f) => /(^|\/)[^/]*_test\.go$/.test(f)),
    `_test.go leaked: ${JSON.stringify(goProd.filter((f) => /_test\.go$/.test(f)))}`,
  );
}

// ── Regression: existing JS/TS behavior preserved ───────────────
{
  const tsAll = rel(collectFiles(ROOT, {
    languages: ['ts', 'tsx', 'js', 'mjs', 'cjs', 'jsx'],
    includeTests: true,
  }));
  assert(
    'R1. JS/TS scan picks up root-entry.ts (FP-13)',
    tsAll.includes('root-entry.ts'),
    `got: ${JSON.stringify(tsAll)}`,
  );
  assert(
    'R2. JS/TS scan picks up build-tool.mjs (root-entry)',
    tsAll.includes('build-tool.mjs'),
    `got: ${JSON.stringify(tsAll)}`,
  );
  assert(
    'R3. JS/TS scan with includeTests=true keeps .test.ts',
    tsAll.some((f) => f.endsWith('.test.ts')),
    `got: ${JSON.stringify(tsAll)}`,
  );
}
{
  const tsProd = rel(collectFiles(ROOT, {
    languages: ['ts', 'tsx', 'js', 'mjs', 'cjs', 'jsx'],
    includeTests: false,
  }));
  assert(
    'R4. JS/TS scan with includeTests=false drops .test.ts',
    !tsProd.some((f) => /\.test\.tsx?$/.test(f)),
    `leaked: ${JSON.stringify(tsProd.filter((f) => /\.test\.tsx?$/.test(f)))}`,
  );
  assert(
    'R4b. JS/TS scan with includeTests=false drops *-test-support.ts',
    !tsProd.includes('src/socket-test-support.ts'),
    `test-support file leaked: ${JSON.stringify(tsProd)}`,
  );
  assert(
    'R4c. JS/TS scan with includeTests=false drops runtime-tests/ directories',
    !tsProd.some((f) => f.startsWith('runtime-tests/')),
    `runtime-tests leaked: ${JSON.stringify(tsProd.filter((f) => f.startsWith('runtime-tests/')))}`,
  );
  assert(
    'R4d. JS/TS scan with includeTests=false drops test-utils/ directories',
    !tsProd.some((f) => f.startsWith('test-utils/')),
    `test-utils leaked: ${JSON.stringify(tsProd.filter((f) => f.startsWith('test-utils/')))}`,
  );
}

// ── Regression: .py / .go do not leak into JS/TS scan ───────────
{
  const tsAll = rel(collectFiles(ROOT, { languages: ['ts', 'tsx', 'js', 'mjs', 'cjs', 'jsx'], includeTests: true }));
  assert(
    'R5. JS/TS scan does not include .py files',
    !tsAll.some((f) => f.endsWith('.py')),
    `leaked: ${JSON.stringify(tsAll.filter((f) => f.endsWith('.py')))}`,
  );
  assert(
    'R6. JS/TS scan does not include .go files',
    !tsAll.some((f) => f.endsWith('.go')),
    `leaked: ${JSON.stringify(tsAll.filter((f) => f.endsWith('.go')))}`,
  );
}

// ── Regression: user excludes apply to root-level search dirs ───
{
  const tsExcluded = rel(collectFiles(ROOT, {
    languages: ['ts', 'tsx', 'js', 'mjs', 'cjs', 'jsx'],
    includeTests: true,
    exclude: ['output'],
  }));
  assert(
    'R7. --exclude output prunes root-level output/ search dir',
    !tsExcluded.some((f) => f.startsWith('output/')),
    `output leaked: ${JSON.stringify(tsExcluded.filter((f) => f.startsWith('output/')))}`,
  );
}

{
  const relRoot = path.relative(process.cwd(), ROOT) || '.';
  const tsExcluded = rel(collectFiles(relRoot, {
    languages: ['ts', 'tsx', 'js', 'mjs', 'cjs', 'jsx'],
    includeTests: true,
    exclude: ['output'],
  }));
  assert(
    'R8. --exclude output also works when root is relative',
    !tsExcluded.some((f) => f.startsWith('output/')),
    `output leaked: ${JSON.stringify(tsExcluded.filter((f) => f.startsWith('output/')))}`,
  );
}

{
  const tsExcluded = rel(collectFiles(ROOT, {
    languages: ['ts', 'tsx', 'js', 'mjs', 'cjs', 'jsx'],
    includeTests: true,
    exclude: ['build'],
  }));
  assert(
    'R9. --exclude build prunes directory segments, not build-* filenames',
    tsExcluded.includes('src/build-index.ts'),
    `build-index.ts was incorrectly excluded: ${JSON.stringify(tsExcluded)}`,
  );
}

{
  const tsExcluded = rel(collectFiles(ROOT, {
    languages: ['ts', 'tsx', 'js', 'mjs', 'cjs', 'jsx'],
    includeTests: true,
    exclude: ['skip-me.js'],
  }));
  assert(
    'R10. --exclude skip-me.js excludes matching file basename',
    !tsExcluded.includes('src/skip-me.js'),
    `skip-me.js leaked: ${JSON.stringify(tsExcluded)}`,
  );
}

{
  const tsExcluded = rel(collectFiles(ROOT, {
    languages: ['ts', 'tsx', 'js', 'mjs', 'cjs', 'jsx'],
    includeTests: true,
    exclude: ['src/nested/exact-file.js'],
  }));
  assert(
    'R11. --exclude src/nested/exact-file.js excludes exact file path suffix',
    !tsExcluded.includes('src/nested/exact-file.js') && tsExcluded.includes('src/skip-me.js'),
    `unexpected file-path exclude result: ${JSON.stringify(tsExcluded)}`,
  );
}

{
  const tsExcluded = rel(collectFiles(ROOT, {
    languages: ['ts', 'tsx', 'js', 'mjs', 'cjs', 'jsx'],
    includeTests: true,
    exclude: ['src/a.ts'],
  }));
  assert(
    'R12. file-path exclude does not prune sibling files',
    !tsExcluded.includes('src/a.ts') && tsExcluded.includes('src/b.ts'),
    `unexpected sibling pruning: ${JSON.stringify(tsExcluded)}`,
  );
}

{
  const tsExcluded = relVendor(collectFiles(VENDOR_ROOT, {
    languages: ['ts', 'tsx', 'js', 'mjs', 'cjs', 'jsx'],
    includeTests: true,
    exclude: ['vendor'],
  }));
  assert(
    'R13. --exclude vendor matches repo-relative paths, not absolute parent dirs',
    tsExcluded.includes('src/keep.ts') && !tsExcluded.includes('src/vendor/skip.ts'),
    `unexpected vendor-root result: ${JSON.stringify(tsExcluded)}`,
  );
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

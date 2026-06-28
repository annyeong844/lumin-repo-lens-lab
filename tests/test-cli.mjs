// Tests for parseCliArgs() includeTests negation.
// We probe by mutating process.argv and re-importing the module.
import { mkdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
// Use file:// URLs for dynamic import so absolute Windows paths (`C:\...`)
// don't trip ERR_UNSUPPORTED_ESM_URL_SCHEME — Node's ESM loader requires
// a valid URL scheme, not a raw path, on Windows.
// v1.10.1: the `_lib/resolver.mjs` facade was retired in favor of
// direct subpath imports. `parseCliArgs` lives in `_lib/cli.mjs`
// and `isTestLikePath` in `_lib/test-paths.mjs` — each dynamic
// import below targets its actual source module.
const cliUrl = pathToFileURL(path.resolve(__dirname, '../_lib/cli.mjs')).href;
const testPathsUrl = pathToFileURL(path.resolve(__dirname, '../_lib/test-paths.mjs')).href;
const FAKE_ROOT = '/tmp/fx-cli-probe';
mkdirSync(FAKE_ROOT, { recursive: true });

let failed = 0;
let passed = 0;
function assert(label, ok, detail) {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

async function run(argv) {
  const saved = process.argv.slice();
  process.argv = ['node', 'script.mjs', '--root', FAKE_ROOT, ...argv];
  const bust = `?t=${Date.now()}-${Math.random()}`;
  try {
    const mod = await import(`${cliUrl}${bust}`);
    return mod.parseCliArgs();
  } finally {
    process.argv = saved;
  }
}

const cli1 = await run([]);
assert('default: includeTests === true (boolean)',
  cli1.includeTests === true, `got ${JSON.stringify(cli1.includeTests)} (${typeof cli1.includeTests})`);
assert('default: output === <root>/.audit (shared with public orchestrator)',
  cli1.output === path.join(path.resolve(FAKE_ROOT), '.audit'), `got ${cli1.output}`);

const cli2 = await run(['--include-tests']);
assert('--include-tests: true (boolean)',
  cli2.includeTests === true, `got ${JSON.stringify(cli2.includeTests)} (${typeof cli2.includeTests})`);

const cli3 = await run(['--no-include-tests']);
assert('--no-include-tests: false (boolean) ← was silently true',
  cli3.includeTests === false, `got ${JSON.stringify(cli3.includeTests)} (${typeof cli3.includeTests})`);

const cli4 = await run(['--no-tests']);
assert('--no-tests (short alias): false',
  cli4.includeTests === false, `got ${JSON.stringify(cli4.includeTests)} (${typeof cli4.includeTests})`);

const cli5 = await run(['--production']);
assert('--production: false',
  cli5.includeTests === false, `got ${JSON.stringify(cli5.includeTests)} (${typeof cli5.includeTests})`);

const cli6 = await run(['--include-tests=false']);
assert('--include-tests=false: false (boolean) ← was string "false" / truthy',
  cli6.includeTests === false, `got ${JSON.stringify(cli6.includeTests)} (${typeof cli6.includeTests})`);

const cli7 = await run(['--include-tests=true']);
assert('--include-tests=true: true (boolean) ← was string "true"',
  cli7.includeTests === true, `got ${JSON.stringify(cli7.includeTests)} (${typeof cli7.includeTests})`);

// Also verify: --production wins even if --include-tests is passed.
const cli8 = await run(['--include-tests', '--production']);
assert('--include-tests + --production: false (production wins)',
  cli8.includeTests === false, `got ${JSON.stringify(cli8.includeTests)} (${typeof cli8.includeTests})`);

// v1.3.0 merge: --exclude-tests alias (pulled from the other patch)
const cli10 = await run(['--exclude-tests']);
assert('--exclude-tests (alias from merged patch): false',
  cli10.includeTests === false, `got ${JSON.stringify(cli10.includeTests)}`);

// v1.3.0 merge: isTestLikePath is exported and recognizes each convention.
const { isTestLikePath } = await import(`${testPathsUrl}?isTestLikePath=${Date.now()}`);
assert('isTestLikePath: foo.test.ts',
  isTestLikePath('src/foo.test.ts') === true,
  `got ${isTestLikePath('src/foo.test.ts')}`);
assert('isTestLikePath: bar.spec.js',
  isTestLikePath('src/bar.spec.js') === true,
  `got ${isTestLikePath('src/bar.spec.js')}`);
assert('isTestLikePath: test_foo.py (pytest)',
  isTestLikePath('src/test_foo.py') === true,
  `got ${isTestLikePath('src/test_foo.py')}`);
assert('isTestLikePath: bar_test.go',
  isTestLikePath('src/bar_test.go') === true,
  `got ${isTestLikePath('src/bar_test.go')}`);
assert('isTestLikePath: tests/helper.ts (path segment)',
  isTestLikePath('/abs/tests/helper.ts') === true,
  `got ${isTestLikePath('/abs/tests/helper.ts')}`);
assert('isTestLikePath: runtime-tests/workerd/index.ts (path segment)',
  isTestLikePath('/abs/runtime-tests/workerd/index.ts') === true,
  `got ${isTestLikePath('/abs/runtime-tests/workerd/index.ts')}`);
assert('isTestLikePath: test-utils/helper.ts (path segment)',
  isTestLikePath('/abs/test-utils/helper.ts') === true,
  `got ${isTestLikePath('/abs/test-utils/helper.ts')}`);
assert('isTestLikePath: src/foo-test-support.ts',
  isTestLikePath('src/foo-test-support.ts') === true,
  `got ${isTestLikePath('src/foo-test-support.ts')}`);
assert('isTestLikePath: src/contest.ts must NOT match (substring false positive)',
  isTestLikePath('src/contest.ts') === false,
  `got ${isTestLikePath('src/contest.ts')}`);

// Regression: other fields still work.
const cli9 = await run(['--verbose']);
assert('unrelated --verbose does not affect includeTests',
  cli9.includeTests === true && cli9.verbose === true,
  `got includeTests=${cli9.includeTests} verbose=${cli9.verbose}`);

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

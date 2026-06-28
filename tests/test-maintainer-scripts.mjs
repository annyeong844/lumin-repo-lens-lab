import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

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

{
  const source = readFileSync(path.join(ROOT, 'scripts/run-syntax-check.mjs'), 'utf8');
  assert('MS1. run-syntax-check reports child process spawn errors explicitly',
    source.includes('if (result.error)') &&
      source.includes('failed to start node --check') &&
      source.includes('result.error.message'),
    source);
}

{
  const source = readFileSync(path.join(ROOT, 'scripts/run-tests.mjs'), 'utf8');
  assert('MS2. run-tests reports child process spawn errors explicitly',
    source.includes('if (result.error)') &&
      source.includes('failed to start test suite') &&
      source.includes('result.error.message'),
    source);
}

{
  const publisher = readFileSync(path.join(ROOT, 'scripts/publish-public-plugin.mjs'), 'utf8');
  assert('MS3. publish-public-plugin uses try/catch optional JSON reads instead of existsSync/readFileSync',
    publisher.includes('function readOptionalJson') &&
      publisher.includes('if (error?.code === \'ENOENT\') return null;') &&
      !publisher.includes('existsSync(path.join(checkoutDir, \'skills/lumin-repo-lens-lab/package-lock.json\'))\n    ? readJson'),
    publisher.slice(publisher.indexOf('function validatePackageSurface'), publisher.indexOf('function hasActualGitChanges')));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

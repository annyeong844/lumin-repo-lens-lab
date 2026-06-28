// Regression guard for symlink aliasing.
//
// Scenario: a vendored module sits outside the normal source tree and
// is exposed via an in-tree symlink. `collectFiles` skips symlinks and
// walks the canonical file only. Before v1.8.0 the resolver returned
// the symlink path, so downstream lookups keyed by canonical path
// missed the entry and reported falsely-dead symbols.
//
// The fix: resolver canonicalizes every returned file path via
// `realpathSync`. This test reproduces the original misbehavior's
// setup and asserts the resolver returns the realpath.
//
// Platform gating — why we probe instead of always running:
// File symlink creation on Windows requires `SeCreateSymbolicLinkPrivilege`
// (admin shell or Developer Mode). A skill should be usable from a normal
// shell — no user is going to elevate just to run the skill's tests.
// Upstream precedent: neither dependency-cruiser nor oxc exercise real OS
// symlinks in their suites (dep-cruiser's `test/.../symlink.js` is just a
// regular file named "symlink.js"; oxc has zero `symlinkSync` calls). Both
// tools delegate symlink resolution to the OS at runtime and trust Linux
// CI for regression coverage. We do the same: probe once, skip cleanly on
// platforms where fixture creation isn't permitted, keep full coverage on
// Linux/macOS CI and on Windows with Developer Mode enabled.

import {
  writeFileSync, mkdirSync, rmSync, symlinkSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { detectRepoMode } from '../_lib/repo-mode.mjs';
import { buildAliasMap } from '../_lib/alias-map.mjs';
import { makeResolver } from '../_lib/resolver-core.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
// `/tmp/...` is Linux-only; use os.tmpdir() for Windows portability.
const ROOT = path.join(tmpdir(), 'fx-symlink-aliasing');

// ── Platform probe: can we create a file symlink here? ───────────
// A tiny probe under tmpdir before touching the fixture. If this throws
// EPERM/EACCES we know the whole suite can't run; skip cleanly.
const probeDir = path.join(tmpdir(), `fx-symlink-probe-${process.pid}-${Date.now()}`);
mkdirSync(probeDir, { recursive: true });
let symlinksAvailable = true;
try {
  writeFileSync(path.join(probeDir, 'target'), '');
  symlinkSync(path.join(probeDir, 'target'), path.join(probeDir, 'link'));
} catch (e) {
  if (e.code === 'EPERM' || e.code === 'EACCES') {
    symlinksAvailable = false;
  } else {
    rmSync(probeDir, { recursive: true, force: true });
    throw e;
  }
}
rmSync(probeDir, { recursive: true, force: true });

if (!symlinksAvailable) {
  console.log('  SKIP  symlink-aliasing suite — this platform does not allow');
  console.log('        creating file symlinks without elevated privilege');
  console.log('        (Windows: admin shell or Developer Mode). Linux CI');
  console.log('        covers the v1.8.0 realpath-resolution regression.');
  console.log('        At runtime the tool delegates to fs.realpathSync, so');
  console.log('        existing symlinks are still resolved correctly on this');
  console.log('        platform — only fixture creation is gated.');
  console.log('\n0 passed, 0 failed (6 skipped)');
  process.exit(0);
}

rmSync(ROOT, { recursive: true, force: true });
mkdirSync(path.join(ROOT, 'src'), { recursive: true });
mkdirSync(path.join(ROOT, 'vendored'), { recursive: true });
writeFileSync(path.join(ROOT, 'package.json'),
  '{"name":"fx-symlink","type":"module"}');

// Vendored module
writeFileSync(path.join(ROOT, 'vendored/lib.ts'),
  'export const vendoredValue = 42;\n');

// Symlink from src/lib.ts to the vendored module
symlinkSync('../vendored/lib.ts', path.join(ROOT, 'src/lib.ts'));

// Consumer imports the symlink
writeFileSync(path.join(ROOT, 'src/app.ts'),
  "import { vendoredValue } from './lib.js';\n" +
  'export const used = vendoredValue;\n'
);

// Second symlink case: a directory symlink. Consumer imports by
// extension-less spec, so resolver enters `/index.*` lookup on the
// realpath side.
mkdirSync(path.join(ROOT, 'shared/core'), { recursive: true });
writeFileSync(path.join(ROOT, 'shared/core/index.ts'),
  'export const sharedCore = 1;\n'
);
symlinkSync('../shared/core', path.join(ROOT, 'src/core-link'));
writeFileSync(path.join(ROOT, 'src/consumer.ts'),
  "import { sharedCore } from './core-link';\n" +
  'export const c = sharedCore;\n'
);

const mode = detectRepoMode(ROOT);
const resolve = makeResolver(ROOT, buildAliasMap(ROOT, mode));

let passed = 0, failed = 0;
function eq(label, actual, expected) {
  if (actual === expected) { passed++; console.log(`  PASS  ${label}`); }
  else {
    failed++;
    console.log(`  FAIL  ${label}`);
    console.log(`        got:      ${actual}`);
    console.log(`        expected: ${expected}`);
  }
}

// T1. File-symlink: resolver returns realpath, not the symlink path
const appTs = path.join(ROOT, 'src/app.ts');
eq('T1. file-symlink resolved to realpath (not src/lib.ts)',
  resolve(appTs, './lib.js'),
  path.join(ROOT, 'vendored/lib.ts'));

// T2. Explicit extension-less form hits the same realpath
eq('T2. extensionless symlink spec resolves to realpath',
  resolve(appTs, './lib'),
  path.join(ROOT, 'vendored/lib.ts'));

// T3. Directory-symlink with /index.ts lookup canonicalizes too
const consumerTs = path.join(ROOT, 'src/consumer.ts');
eq('T3. dir-symlink + /index.ts lookup returns realpath',
  resolve(consumerTs, './core-link'),
  path.join(ROOT, 'shared/core/index.ts'));

// T4. Null / EXTERNAL pass through unchanged (canonicalize must not
// break sentinel returns)
eq('T4. null passes through unchanged',
  resolve(appTs, ''),
  null);
eq('T5. EXTERNAL passes through unchanged',
  resolve(appTs, 'some-npm-package'),
  'EXTERNAL');

// T6. Non-symlinked relative import still works (no regression in the
// common case)
writeFileSync(path.join(ROOT, 'src/normal.ts'), 'export const n = 1;\n');
eq('T6. non-symlinked relative import unchanged',
  resolve(appTs, './normal'),
  path.join(ROOT, 'src/normal.ts'));

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

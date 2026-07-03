#!/usr/bin/env node
// scripts/run-tests.mjs — portable test runner (replaces POSIX for-loop).
//
// Walks default `tests/test-*.mjs` suites and runs each via a fresh node
// subprocess.
// Stops on first failure, mirroring `for f in ...; do ... || exit 1`.
// Works on bash, zsh, PowerShell, and cmd.exe without shell-syntax hazards.
//
// Legacy umbrella suites stay runnable through explicit npm scripts, but are not
// part of the default Node gate. Their protected contracts have focused Vitest
// and/or Rust cargo gates.

import { readdirSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(__dirname, '..');
const TESTS_DIR = path.join(REPO, 'tests');
const LEGACY_NODE_SUITES = new Set([
  'test-audit-repo.mjs',
]);

const suites = readdirSync(TESTS_DIR)
  .filter((f) => f.startsWith('test-') && f.endsWith('.mjs') && !LEGACY_NODE_SUITES.has(f))
  .sort();
const skippedLegacySuites = readdirSync(TESTS_DIR)
  .filter((f) => LEGACY_NODE_SUITES.has(f))
  .sort();

if (skippedLegacySuites.length > 0) {
  process.stdout.write(
    `[run-tests] skipping legacy umbrella suite(s): ${skippedLegacySuites.join(', ')}\n`
  );
}

let failed = 0;
for (const suite of suites) {
  const result = spawnSync(process.execPath, [path.join(TESTS_DIR, suite)], {
    stdio: 'inherit',
  });
  if (result.error) {
    process.stderr.write(`[run-tests] failed to start test suite ${suite}: ${result.error.message}\n`);
    failed += 1;
    process.exit(1);
  }
  if (result.status !== 0) {
    process.stderr.write(`[run-tests] FAIL: ${suite} (exit ${result.status})\n`);
    failed += 1;
    process.exit(1);
  }
}

process.stdout.write(`[run-tests] ${suites.length} suites passed\n`);
process.exit(failed === 0 ? 0 : 1);

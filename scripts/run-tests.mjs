#!/usr/bin/env node
// scripts/run-tests.mjs — portable test runner (replaces POSIX for-loop).
//
// Walks `tests/test-*.mjs` and runs each via a fresh node subprocess.
// Stops on first failure, mirroring `for f in ...; do ... || exit 1`.
// Works on bash, zsh, PowerShell, and cmd.exe without shell-syntax hazards.

import { readdirSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(__dirname, '..');
const TESTS_DIR = path.join(REPO, 'tests');

const suites = readdirSync(TESTS_DIR)
  .filter((f) => f.startsWith('test-') && f.endsWith('.mjs'))
  .sort();

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

#!/usr/bin/env node
// scripts/run-syntax-check.mjs — portable `node --check` walker.
//
// Replaces the POSIX for-loop in `npm run check` with a Node-native version
// that works on bash / zsh / PowerShell / cmd.exe identically.

import { readdirSync, statSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(__dirname, '..');

const files = [];
// _lib/*.mjs
const libDir = path.join(REPO, '_lib');
try {
  for (const f of readdirSync(libDir)) {
    if (f.endsWith('.mjs')) files.push(path.join(libDir, f));
  }
} catch { /* no _lib/ dir */ }
// Root *.mjs
for (const f of readdirSync(REPO)) {
  const full = path.join(REPO, f);
  try {
    if (f.endsWith('.mjs') && statSync(full).isFile()) files.push(full);
  } catch { /* skip */ }
}
files.sort();

let checked = 0;
for (const f of files) {
  const result = spawnSync(process.execPath, ['--check', f], { stdio: 'inherit' });
  if (result.error) {
    process.stderr.write(
      `[run-syntax-check] failed to start node --check for ${path.relative(REPO, f)}: ${result.error.message}\n`
    );
    process.exit(1);
  }
  if (result.status !== 0) {
    process.stderr.write(`[run-syntax-check] FAIL: ${path.relative(REPO, f)}\n`);
    process.exit(1);
  }
  checked += 1;
}

process.stdout.write(`[run-syntax-check] syntax OK — ${checked} files\n`);
process.exit(0);

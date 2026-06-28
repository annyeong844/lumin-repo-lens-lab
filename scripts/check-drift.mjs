#!/usr/bin/env node
// Version drift guard. Checks that all these places agree on one version:
//   - package.json        (authoritative source)
//   - emit-sarif.mjs      (TOOL_VERSION constant — surfaces in SARIF)
//   - CHANGELOG.md        (top entry version)
//   - package-lock.json   (added v1.8.3 after four-release drift)
//
// Does NOT check tests/README.md — generation drift for that file is
// covered by a sibling script, `npm run check:test-doc`, which runs
// scripts/update-test-doc.mjs in --check mode. Assertion counts
// themselves are authoritative only in `npm test` output.
//
// Runs in CI. Exit 0 if all sources agree, 1 with a table of mismatches.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

const mismatches = [];
function check(label, actual, expected, file) {
  if (actual === expected) return;
  mismatches.push({ label, actual, expected, file });
}

// ─── 1. package.json version ─────────────────────────────
const pkg = JSON.parse(readFileSync(path.join(ROOT, 'package.json'), 'utf8'));
const canonicalVersion = pkg.version;

// ─── 2. emit-sarif.mjs TOOL_VERSION ──────────────────────
{
  const src = readFileSync(path.join(ROOT, 'emit-sarif.mjs'), 'utf8');
  const m = src.match(/TOOL_VERSION\s*=\s*['"]([^'"]+)['"]/);
  check(
    'emit-sarif.mjs TOOL_VERSION',
    m?.[1],
    canonicalVersion,
    'emit-sarif.mjs',
  );
}

// ─── 3. CHANGELOG.md top entry ───────────────────────────
{
  const src = readFileSync(path.join(ROOT, 'CHANGELOG.md'), 'utf8');
  const m = src.match(
    /^##\s+((?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?)/m
  );
  check(
    'CHANGELOG.md latest entry',
    m?.[1],
    canonicalVersion,
    'CHANGELOG.md',
  );
}

// ─── 4. package-lock.json (v1.8.3) ───────────────────────
// Real release drift caught in dogfood: package-lock.json was at 1.4.0
// while package.json was 1.8.2. npm always updates the lockfile's top
// `version` field during `npm install`, but a `sed`-based version bump
// in the other files left the lockfile untouched for four releases in a
// row because we never ran install between bumps.
// Fix: the authoritative fix is `npm install --package-lock-only`, but
// drift guard catches the mismatch either way.
{
  try {
    const lock = JSON.parse(readFileSync(path.join(ROOT, 'package-lock.json'), 'utf8'));
    check(
      'package-lock.json version',
      lock.version,
      canonicalVersion,
      'package-lock.json',
    );
    check(
      'package-lock.json packages[""].version',
      lock.packages?.['']?.version,
      canonicalVersion,
      'package-lock.json',
    );
  } catch {
    // Lockfile missing / unreadable — not a hard drift, just skip.
    // (Local checkouts without node_modules are a supported state.)
  }
}

// ─── 5. tests/README.md ───────────────────────────────────
// Not checked in this script. Generated README drift is checked
// separately by `npm run check:test-doc`, which runs
// scripts/update-test-doc.mjs in --check mode. Assertion counts
// themselves remain authoritative only in `npm test` output; prose
// counts are not compared here.

// ─── Report ──────────────────────────────────────────────
if (mismatches.length === 0) {
  console.log(`[check-drift] all sources agree on version ${canonicalVersion}`);
  process.exit(0);
}

console.error('[check-drift] DRIFT DETECTED:');
console.error('');
const width = Math.max(...mismatches.map((m) => m.label.length));
for (const m of mismatches) {
  console.error(`  ${m.label.padEnd(width)}  got ${JSON.stringify(m.actual)}  expected ${JSON.stringify(m.expected)}  (${m.file})`);
}
console.error('');
console.error(`  Fix: update the listed files so they match package.json version ${canonicalVersion}`);
console.error(`       or update package.json if the other values are authoritative.`);
process.exit(1);

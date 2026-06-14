#!/usr/bin/env node
// CI guard: every `*.mjs` filename referenced in live documentation
// must actually exist on disk.
//
// Motivation: v1.9.7 reviewer caught that SKILL.md:57 referenced
// `compare-repos.mjs` as part of the Workflow, but the file did not
// exist. A tool whose core claim is "evidence before claims"
// shouldn't have its own live docs point at non-existent evidence
// producers. This check prevents future regressions of the same
// shape.
//
// Scope:
//   - SKILL.md
//   - templates/report-template.md
//   - tests/README.md (but that's generated, so it should always be
//     consistent with disk; included as a double-check)
//
// Excluded:
//   - CHANGELOG.md — historical record; old version entries may
//     reference scripts that existed then and don't now
//   - docs/maintainer/false-positive-patterns-ledger.md — historical
//     pattern ledger
//   - node_modules, tests/ fixtures, code comments (too noisy)

import { readFileSync, existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');

const DOCS_TO_SCAN = [
  'SKILL.md',
  'templates/report-template.md',
  'tests/README.md',
];

// Enumerate all .mjs files actually present at the repo root and in
// `_lib/` + `scripts/` + `tests/`. That's where docs reference them.
function enumerateMjs() {
  const set = new Set();
  for (const d of ['.', '_lib', 'scripts', 'tests']) {
    const full = path.join(ROOT, d);
    if (!existsSync(full)) continue;
    for (const f of readdirSync(full)) {
      if (f.endsWith('.mjs')) set.add(f);
    }
  }
  return set;
}

const presentMjs = enumerateMjs();

// Match bare references like `compare-repos.mjs` (with optional
// backticks, in inline code or filenames). Ignore matches that are
// fragments of longer tokens like `foo-compare-repos.mjs` by
// anchoring on word-boundary-equivalent characters.
const MJS_RE = /(?:^|[^\w./\\-])([a-zA-Z][\w.-]*\.mjs)/g;

const problems = [];
for (const relDoc of DOCS_TO_SCAN) {
  const full = path.join(ROOT, relDoc);
  if (!existsSync(full)) {
    problems.push({ doc: relDoc, issue: 'doc-missing' });
    continue;
  }
  const text = readFileSync(full, 'utf8');
  const seen = new Set();
  for (const match of text.matchAll(MJS_RE)) {
    const name = match[1];
    if (seen.has(name)) continue;
    seen.add(name);
    // Skip obviously-stale-looking patterns inside fenced code blocks
    // that clearly describe non-real files (e.g., "fake-test.mjs" in
    // an example). We don't try hard — just the real drift case.
    if (!presentMjs.has(name)) {
      problems.push({ doc: relDoc, name, issue: 'missing-on-disk' });
    }
  }
}

if (problems.length > 0) {
  console.error(`[check-doc-script-refs] ${problems.length} problem(s):`);
  for (const p of problems) {
    if (p.issue === 'doc-missing') {
      console.error(`  ${p.doc}: document does not exist`);
    } else {
      console.error(`  ${p.doc} references ${p.name} but file is not present on disk`);
    }
  }
  console.error('');
  console.error('Either (a) create the referenced file, (b) remove the reference,');
  console.error('or (c) if the reference is intentional (e.g., inside a fenced');
  console.error('example block), rename or comment it to avoid matching `*.mjs`.');
  process.exit(1);
}

console.log('[check-doc-script-refs] all documented .mjs references resolve on disk');
process.exit(0);

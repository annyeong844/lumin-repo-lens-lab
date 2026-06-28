// Regression guard for v1.9.8 "Evidence Honesty Patch" tools:
//   - compare-repos.mjs: artifact-level repo comparison
//   - scripts/check-doc-script-refs.mjs: CI guard against doc drift
//     where SKILL.md references non-existent .mjs files

import { execSync } from 'node:child_process';
import {
  writeFileSync, readFileSync, mkdirSync, rmSync, mkdtempSync, copyFileSync,
  cpSync,
} from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function run(cmd) {
  try {
    return { ok: true, out: execSync(cmd, { cwd: DIR, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }) };
  } catch (e) {
    return { ok: false, out: (e.stdout || '') + (e.stderr || '') };
  }
}

// ───────────────────────────────────────────────────────────
// A. compare-repos.mjs — artifact-level diff
// ───────────────────────────────────────────────────────────
{
  const LEFT = mkdtempSync(path.join(tmpdir(), 'fx-cmp-left-'));
  const RIGHT = mkdtempSync(path.join(tmpdir(), 'fx-cmp-right-'));
  const OUT = mkdtempSync(path.join(tmpdir(), 'fx-cmp-out-'));

  try {
    // Synthesize asymmetric artifacts. Left has 10 files, 3 safe fixes;
    // right has 15 files, 7 safe fixes. Expected deltas: files +5,
    // safeFixes +4.
    writeFileSync(path.join(LEFT, 'triage.json'), JSON.stringify({
      summary: { files: 10, loc: 1000, buildSystem: 'vite' },
    }));
    writeFileSync(path.join(LEFT, 'fix-plan.json'), JSON.stringify({
      meta: { resolverBlindness: { gate: 'ok' } },
      summary: { SAFE_FIX: 3, REVIEW_FIX: 5, DEGRADED: 1, MUTED: 2, total: 11 },
      safeFixes: [], reviewFixes: [], degraded: [], muted: [],
    }));
    writeFileSync(path.join(LEFT, 'symbols.json'), JSON.stringify({
      files: 10, totalDefs: 80, deadInProd: 4,
      uses: { resolvedInternal: 200, external: 50, unresolvedInternal: 0, unresolvedInternalRatio: 0 },
    }));

    writeFileSync(path.join(RIGHT, 'triage.json'), JSON.stringify({
      summary: { files: 15, loc: 1500, buildSystem: 'vite' },
    }));
    writeFileSync(path.join(RIGHT, 'fix-plan.json'), JSON.stringify({
      meta: { resolverBlindness: { gate: 'ok' } },
      summary: { SAFE_FIX: 7, REVIEW_FIX: 3, DEGRADED: 2, MUTED: 1, total: 13 },
      safeFixes: [], reviewFixes: [], degraded: [], muted: [],
    }));
    writeFileSync(path.join(RIGHT, 'symbols.json'), JSON.stringify({
      files: 15, totalDefs: 120, deadInProd: 6,
      uses: { resolvedInternal: 300, external: 75, unresolvedInternal: 0, unresolvedInternalRatio: 0 },
    }));

    const r = run(`node compare-repos.mjs --left ${LEFT} --right ${RIGHT} --output ${OUT} --left-label L --right-label R`);
    assert('C1. compare-repos exits 0 on valid inputs',
      r.ok, r.out);

    const cmp = JSON.parse(readFileSync(path.join(OUT, 'compare.json'), 'utf8'));

    assert('C2. deltas.files = +5 (15 - 10)',
      cmp.deltas.files === 5, `got ${cmp.deltas.files}`);

    assert('C3. deltas.safeFixes = +4 (7 - 3)',
      cmp.deltas.safeFixes === 4, `got ${cmp.deltas.safeFixes}`);

    assert('C4. deltas.degraded = +1 (2 - 1)',
      cmp.deltas.degraded === 1, `got ${cmp.deltas.degraded}`);

    assert('C5. both sides listed fix-plan.json + symbols.json + triage.json',
      cmp.left.artifactsFound.length === 3 && cmp.right.artifactsFound.length === 3,
      `left=${JSON.stringify(cmp.left.artifactsFound)}, right=${JSON.stringify(cmp.right.artifactsFound)}`);

    assert('C6. missingArtifacts correctly flags what was not present (no runtime-evidence etc.)',
      cmp.missingArtifacts.left.includes('runtime-evidence.json') &&
      cmp.missingArtifacts.left.includes('staleness.json'),
      `missingLeft: ${JSON.stringify(cmp.missingArtifacts.left)}`);

    // C7: asymmetric case — one side missing an artifact should yield
    // null deltas for that dimension, not a wrong number.
    const RIGHT2 = mkdtempSync(path.join(tmpdir(), 'fx-cmp-right2-'));
    const OUT2 = mkdtempSync(path.join(tmpdir(), 'fx-cmp-out2-'));
    writeFileSync(path.join(RIGHT2, 'triage.json'), JSON.stringify({
      summary: { files: 15, loc: 1500 },
    }));
    // No fix-plan on right — left has it. Compare should null-out safeFixes delta.
    const r2 = run(`node compare-repos.mjs --left ${LEFT} --right ${RIGHT2} --output ${OUT2}`);
    const cmp2 = JSON.parse(readFileSync(path.join(OUT2, 'compare.json'), 'utf8'));
    assert('C7. asymmetric missing artifact → delta is null (not an invented number)',
      cmp2.deltas.safeFixes === null,
      `got ${cmp2.deltas.safeFixes}`);
    rmSync(RIGHT2, { recursive: true, force: true });
    rmSync(OUT2, { recursive: true, force: true });
  } finally {
    rmSync(LEFT, { recursive: true, force: true });
    rmSync(RIGHT, { recursive: true, force: true });
    rmSync(OUT, { recursive: true, force: true });
  }
}

// ───────────────────────────────────────────────────────────
// B. check-doc-script-refs.mjs — CI guard
// ───────────────────────────────────────────────────────────
{
  const FIXTURE = mkdtempSync(path.join(tmpdir(), 'fx-docrefs-'));
  try {
    // Build a minimal fixture with the same layout the guard expects.
    mkdirSync(path.join(FIXTURE, 'scripts'), { recursive: true });
    mkdirSync(path.join(FIXTURE, '_lib'), { recursive: true });
    mkdirSync(path.join(FIXTURE, 'tests'), { recursive: true });
    mkdirSync(path.join(FIXTURE, 'templates'), { recursive: true });

    // Copy the real check-doc-script-refs.mjs over so we're exercising
    // the real implementation, not a re-implementation.
    copyFileSync(
      path.join(DIR, 'scripts/check-doc-script-refs.mjs'),
      path.join(FIXTURE, 'scripts/check-doc-script-refs.mjs'),
    );

    // Case 1: SKILL.md references a file that exists on disk.
    writeFileSync(path.join(FIXTURE, 'real-tool.mjs'), '// stub\n');
    writeFileSync(path.join(FIXTURE, 'SKILL.md'),
      '# Skill\n\nRun `real-tool.mjs` for this.\n');
    writeFileSync(path.join(FIXTURE, 'templates/report-template.md'), '# Report\n');
    writeFileSync(path.join(FIXTURE, 'tests/README.md'), '# Tests\n');

    const r1 = run(`node ${FIXTURE}/scripts/check-doc-script-refs.mjs`);
    assert('D1. guard exits 0 when every referenced .mjs exists',
      r1.ok && r1.out.includes('resolve on disk'),
      r1.out);

    // Case 2: SKILL.md references a file that does NOT exist.
    writeFileSync(path.join(FIXTURE, 'SKILL.md'),
      '# Skill\n\nRun `ghost-tool.mjs` for this.\n');
    const r2 = run(`node ${FIXTURE}/scripts/check-doc-script-refs.mjs`);
    assert('D2. guard exits non-zero when a referenced .mjs is missing',
      !r2.ok && r2.out.includes('ghost-tool.mjs'),
      r2.out);

    // Case 3: the guard's error message should suggest concrete fixes
    assert('D3. guard error message suggests remediation (create/remove/rename)',
      r2.out.includes('create') && r2.out.includes('remove'),
      `message did not include remediation: ${r2.out.slice(0, 300)}`);

    // Case 4: references in _lib/ and scripts/ also count as "present"
    writeFileSync(path.join(FIXTURE, '_lib/helper.mjs'), '// stub\n');
    writeFileSync(path.join(FIXTURE, 'SKILL.md'),
      '# Skill\n\nInternal: `helper.mjs`.\n');
    const r3 = run(`node ${FIXTURE}/scripts/check-doc-script-refs.mjs`);
    assert('D4. files under _lib/ count as present (helper.mjs resolves)',
      r3.ok,
      r3.out);
  } finally {
    rmSync(FIXTURE, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

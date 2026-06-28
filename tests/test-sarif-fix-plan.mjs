// Regression guard for v1.9.5 claim "When fix-plan.json is present,
// SARIF severity comes from the tier → level map."
//
// Strategy: synthesize a fix-plan.json with one entry per tier, run
// emit-sarif.mjs against that directory, assert the emitted SARIF
// carries the expected severity distribution and per-result properties.
//
// This is the integration counterpart to test-rank-fixes.mjs (which
// verifies the predicate + merge layer independently). Without this
// test, the tier→level claim was unverified end-to-end.

import { execSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, rmSync, mkdtempSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const FIXTURE = mkdtempSync(path.join(tmpdir(), 'fx-sarif-fixplan-'));
const OUT = path.join(FIXTURE, 'artifacts');
mkdirSync(OUT, { recursive: true });

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

try {
  // Make a fake repo (emit-sarif needs a valid root)
  writeFileSync(path.join(FIXTURE, 'package.json'), '{"name":"fixture","type":"module"}');

  // Build a fix-plan.json with one entry per tier
  const plan = {
    meta: {
      generated: new Date().toISOString(),
      root: FIXTURE,
      tool: 'rank-fixes.mjs',
      inputs: { 'dead-classify.json': true, 'runtime-evidence.json': false,
                'staleness.json': false, 'symbols.json': false },
      resolverBlindness: null,
    },
    summary: { SAFE_FIX: 1, REVIEW_FIX: 1, DEGRADED: 1, MUTED: 1, total: 4 },
    safeFixes: [{
      finding: { id: 'x', file: 'src/safe.ts', line: 10, symbol: 'SafeSym',
                 kind: 'FunctionDeclaration', bucket: 'C', action: 'remove' },
      evidence: {
        runtime: { status: 'dead-confirmed', grounding: 'grounded', confidence: 'high', hitsInSymbol: 0 },
        staleness: { tier: 'fossil', grounding: 'grounded', lineLastTouchedDaysAgo: 900 },
        policy: { excluded: false },
      },
      tier: 'SAFE_FIX',
      reason: 'AST-dead + runtime-dead-confirmed + staleness-fossil + bucket-C',
    }],
    reviewFixes: [{
      finding: { id: 'x', file: 'src/review.ts', line: 20, symbol: 'ReviewSym',
                 kind: 'FunctionDeclaration', bucket: 'A',
                 action: 'demote', fileInternalUses: 2 },
      evidence: { policy: { excluded: false } },
      tier: 'REVIEW_FIX',
      reason: 'bucket-A; missing: no-runtime, no-staleness',
    }],
    degraded: [{
      finding: { id: 'x', file: 'src/deg.ts', line: 30, symbol: 'DegSym',
                 kind: 'FunctionDeclaration', bucket: 'C', action: 'remove' },
      evidence: {
        runtime: { status: 'executed', grounding: 'grounded', confidence: 'high', hitsInSymbol: 7 },
        policy: { excluded: false },
      },
      tier: 'DEGRADED',
      reason: 'runtime-executed (7 hits)',
    }],
    muted: [{
      finding: { id: 'x', file: 'eslint.config.mjs', line: 1, symbol: 'default',
                 kind: 'default', bucket: 'excluded', action: 'Policy-excluded: config_FP22' },
      evidence: { policy: { excluded: true, reason: 'config_FP22' } },
      tier: 'MUTED',
      reason: 'policy-excluded: config_FP22',
    }],
  };
  writeFileSync(path.join(OUT, 'fix-plan.json'), JSON.stringify(plan, null, 2));

  // Run emit-sarif
  execSync(`node emit-sarif.mjs --root ${FIXTURE} --output ${OUT}`,
    { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

  const sarif = JSON.parse(readFileSync(path.join(OUT, 'lumin-repo-lens-lab.sarif'), 'utf8'));
  const ga001 = sarif.runs[0].results.filter((r) => r.ruleId === 'GA001');

  // S1: emit-sarif used fix-plan.json (verifiable via artifactsUsed property
  // or via the tier property appearing on results)
  assert('S1. emit-sarif took the fix-plan branch (results carry .properties.tier)',
    ga001.every((r) => r.properties?.tier !== undefined),
    `first result props: ${JSON.stringify(ga001[0]?.properties)}`);

  // S2: total emitted count is SAFE + REVIEW + DEGRADED = 3 (MUTED excluded)
  assert('S2. MUTED is NOT emitted to SARIF (3 results for 4 fix-plan entries)',
    ga001.length === 3,
    `emitted=${ga001.length}, expected 3`);

  // S3: no MUTED tier in emitted results
  const mutedLeaked = ga001.find((r) => r.properties?.tier === 'MUTED');
  assert('S3. no result carries tier=MUTED',
    !mutedLeaked,
    `leaked: ${JSON.stringify(mutedLeaked)}`);

  // S4: SAFE_FIX → warning
  const safe = ga001.find((r) => r.properties?.tier === 'SAFE_FIX');
  assert('S4. SAFE_FIX emitted as SARIF warning',
    safe?.level === 'warning',
    `got level=${safe?.level}`);

  // S5: REVIEW_FIX → note
  const review = ga001.find((r) => r.properties?.tier === 'REVIEW_FIX');
  assert('S5. REVIEW_FIX emitted as SARIF note',
    review?.level === 'note',
    `got level=${review?.level}`);

  // S6: DEGRADED → note (crucial: runtime-executed must never be warning)
  const deg = ga001.find((r) => r.properties?.tier === 'DEGRADED');
  assert('S6. DEGRADED (runtime-executed) emitted as SARIF note (never warning)',
    deg?.level === 'note',
    `got level=${deg?.level}`);

  // S7: the bucket label is carried through in properties.proposalBucket
  assert('S7. properties.proposalBucket preserves the classifier bucket',
    safe?.properties?.proposalBucket === 'C' &&
    review?.properties?.proposalBucket === 'A' &&
    deg?.properties?.proposalBucket === 'C',
    `buckets: safe=${safe?.properties?.proposalBucket}, ` +
    `review=${review?.properties?.proposalBucket}, deg=${deg?.properties?.proposalBucket}`);

  // S8: the reason is carried through (for downstream filtering)
  assert('S8. properties.reason carries the ranking reason',
    typeof safe?.properties?.reason === 'string' && safe.properties.reason.includes('runtime-dead-confirmed'),
    `reason: ${safe?.properties?.reason}`);

  // S9: level distribution at top of SARIF
  const byLevel = { warning: 0, note: 0, error: 0 };
  for (const r of ga001) byLevel[r.level] = (byLevel[r.level] ?? 0) + 1;
  assert('S9. overall distribution: 1 warning, 2 notes, 0 errors',
    byLevel.warning === 1 && byLevel.note === 2 && byLevel.error === 0,
    JSON.stringify(byLevel));

  // S10: hitsInSymbol carries through for DEGRADED (runtime-executed)
  assert('S10. runtime hitsInSymbol surfaces in properties for audit',
    deg?.properties?.hitsInSymbol === 7,
    `got hitsInSymbol=${deg?.properties?.hitsInSymbol}`);
} finally {
  rmSync(FIXTURE, { recursive: true, force: true });
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

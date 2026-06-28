// tests/test-p6-measurement.mjs
//
// P6-0 measurement harness contract. Pins the pure denominator,
// candidate-count, schema round-trip, dirty-worktree, and readiness-gate
// semantics before the CLI writes p6-measurement.json.

import { execFileSync } from 'node:child_process';
import { writeFileSync, readFileSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import {
  buildCandidateCounts,
  buildSchemaRoundTrip,
  computeReadiness,
  mergeMeasurementArtifacts,
  normalizeAdjudicationEntries,
} from '../_lib/p6-measurement.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, '..');
const NODE = process.execPath;
const CLI = path.join(DIR, 'p6-measurement.mjs');

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function hasReason(readiness, code) {
  return (readiness.reasons ?? []).some((r) => r.code === code);
}

// ─── Candidate-count availability ─────────────────────────

{
  const counts = buildCandidateCounts({ fixPlan: null, deadClassify: null, canonDrift: null });
  assert('P6-1a. missing fix-plan => candidateCounts.available=false',
    counts.available === false, JSON.stringify(counts));
  assert('P6-1b. missing fix-plan cleanup counts are null, not zero',
    counts.reviewVisibleCleanup === null && counts.safeFix === null && counts.reviewFix === null,
    JSON.stringify(counts));
  assert('P6-1c. missing dead-classify rawTierC is null',
    counts.rawTierC === null, JSON.stringify(counts));
  assert('P6-1d. missing canon-drift total is null',
    counts.canonDrift.available === false && counts.canonDrift.total === null,
    JSON.stringify(counts.canonDrift));
}

{
  const counts = buildCandidateCounts({
    fixPlan: {
      safeFixes: [{}, {}],
      reviewFixes: [{}],
      degraded: [{}, {}, {}],
      muted: [{}],
    },
    deadClassify: { summary: { category_C: 7 } },
    canonDrift: {
      summary: { driftCount: 5 },
      perSource: { naming: { status: 'drift', driftCount: 5 } },
    },
  });
  assert('P6-2a. review-visible cleanup = safeFixes + reviewFixes',
    counts.reviewVisibleCleanup === 3 && counts.safeFix === 2 && counts.reviewFix === 1,
    JSON.stringify(counts));
  assert('P6-2b. degraded/muted/rawTierC/canonDrift reported separately',
    counts.degraded === 3 && counts.muted === 1 && counts.rawTierC === 7 &&
    counts.canonDrift.total === 5,
    JSON.stringify(counts));
}

// ─── Adjudication denominator ─────────────────────────────

{
  const entries = normalizeAdjudicationEntries([
    { tier: 'SAFE_FIX', verdict: 'true_dead', corpusName: 'a' },
    { tier: 'SAFE_FIX', verdict: 'false_positive', corpusName: 'a' },
    { tier: 'SAFE_FIX', verdict: 'inconclusive', corpusName: 'a' },
    { tier: 'SAFE_FIX', verdict: 'not_applicable', corpusName: 'a' },
    { tier: 'REVIEW_FIX', verdict: 'true_dead', corpusName: 'a' },
  ]);
  const readiness = computeReadiness({
    corpus: [
      { name: 'a', commit: 'abc', worktreeDirty: false, locBucket: '25k' },
      { name: 'b', commit: 'def', worktreeDirty: false, locBucket: '25k' },
    ],
    candidateCounts: buildCandidateCounts({
      fixPlan: { safeFixes: [{}], reviewFixes: [{}], degraded: [], muted: [] },
      deadClassify: { summary: { category_C: 1 } },
      canonDrift: { summary: { driftCount: 0 }, perSource: {} },
    }),
    adjudicationEntries: entries,
    schemaRoundTrip: { attempted: true, knownSchemaDriftBugs: [], sources: {} },
  });
  assert('P6-3a. inconclusive/not_applicable excluded from SAFE_FIX denominator',
    readiness.safeFix.fpRate === 0.5 &&
    readiness.safeFix.inconclusive === 1 &&
    readiness.safeFix.notApplicable === 1,
    JSON.stringify(readiness.safeFix));
  assert('P6-3b. review-visible denominator includes SAFE_FIX + REVIEW_FIX',
    Math.abs(readiness.reviewVisibleCleanup.fpRate - (1 / 3)) < 0.000001,
    JSON.stringify(readiness.reviewVisibleCleanup));
}

// ─── Readiness blockers ───────────────────────────────────

{
  const readiness = computeReadiness({
    corpus: [{ name: 'a', commit: 'abc', worktreeDirty: false, locBucket: '25k' }],
    candidateCounts: buildCandidateCounts({ fixPlan: null, deadClassify: null, canonDrift: null }),
    adjudicationEntries: [],
    schemaRoundTrip: { attempted: false, knownSchemaDriftBugs: [], sources: {} },
  });
  assert('P6-4a. unavailable candidate counts force Red',
    readiness.gate === 'Red' && hasReason(readiness, 'candidate-counts-unavailable'),
    JSON.stringify(readiness));
  assert('P6-4b. missing adjudication forces fp-rate-unknown',
    hasReason(readiness, 'fp-rate-unknown'), JSON.stringify(readiness));
  assert('P6-4c. schemaRoundTrip.attempted=false blocks Green',
    hasReason(readiness, 'schema-roundtrip-not-attempted'), JSON.stringify(readiness));
}

{
  const candidateCounts = buildCandidateCounts({
    fixPlan: { safeFixes: [{}], reviewFixes: [{}], degraded: [], muted: [] },
    deadClassify: { summary: { category_C: 2 } },
    canonDrift: { summary: { driftCount: 0 }, perSource: {} },
  });
  const readiness = computeReadiness({
    corpus: [
      { name: 'a', commit: 'abc', worktreeDirty: true, locBucket: '25k' },
      { name: 'b', commit: 'def', worktreeDirty: null, locBucket: '25k' },
    ],
    candidateCounts,
    adjudicationEntries: [
      { corpusName: 'a', tier: 'SAFE_FIX', verdict: 'true_dead' },
      { corpusName: 'b', tier: 'REVIEW_FIX', verdict: 'true_dead' },
    ],
    schemaRoundTrip: { attempted: true, knownSchemaDriftBugs: [], sources: {} },
  });
  assert('P6-5a. dirty worktree without snapshot/contentHash blocks Green',
    readiness.gate === 'Red' && hasReason(readiness, 'dirty-worktree-without-snapshot'),
    JSON.stringify(readiness));
  assert('P6-5b. unknown dirty state blocks Green',
    hasReason(readiness, 'dirty-worktree-unknown'), JSON.stringify(readiness));
}

{
  const candidateCounts = buildCandidateCounts({
    fixPlan: { safeFixes: [{}], reviewFixes: [{}], degraded: [], muted: [] },
    deadClassify: { summary: { category_C: 2 } },
    canonDrift: { summary: { driftCount: 0 }, perSource: {} },
  });
  const readiness = computeReadiness({
    corpus: [
      { name: 'a', commit: 'abc', worktreeDirty: false, locBucket: '25k' },
      { name: 'b', snapshotId: 'snap-b', worktreeDirty: false, locBucket: '50k' },
    ],
    candidateCounts,
    adjudicationEntries: [
      { corpusName: 'a', tier: 'SAFE_FIX', verdict: 'true_dead' },
      { corpusName: 'a', tier: 'REVIEW_FIX', verdict: 'true_dead' },
      { corpusName: 'b', tier: 'SAFE_FIX', verdict: 'true_dead' },
      { corpusName: 'b', tier: 'REVIEW_FIX', verdict: 'true_dead' },
    ],
    schemaRoundTrip: { attempted: true, knownSchemaDriftBugs: [], sources: {} },
    minAdjudicatedPerCorpus: 2,
  });
  assert('P6-6. clean corpus + low FP + attempted roundtrip can reach Green',
    readiness.gate === 'Green',
    JSON.stringify(readiness));
}

{
  const candidateCounts = {
    available: true,
    missingArtifacts: [],
    reviewVisibleCleanup: 4,
    safeFix: 0,
    reviewFix: 4,
    degraded: 0,
    muted: 0,
    rawTierC: 4,
    byCorpus: {
      a: { reviewVisibleCleanup: 2, safeFix: 0, reviewFix: 2 },
      b: { reviewVisibleCleanup: 2, safeFix: 0, reviewFix: 2 },
    },
    canonDrift: { available: true, missingArtifacts: [], total: 0, perSource: {} },
  };
  const readiness = computeReadiness({
    corpus: [
      { name: 'a', commit: 'abc', worktreeDirty: false, locBucket: '25k' },
      { name: 'b', commit: 'def', worktreeDirty: false, locBucket: '50k' },
    ],
    candidateCounts,
    adjudicationEntries: [
      { corpusName: 'a', tier: 'REVIEW_FIX', verdict: 'true_dead' },
      { corpusName: 'a', tier: 'REVIEW_FIX', verdict: 'true_dead' },
      { corpusName: 'b', tier: 'REVIEW_FIX', verdict: 'true_dead' },
      { corpusName: 'b', tier: 'REVIEW_FIX', verdict: 'true_dead' },
    ],
    schemaRoundTrip: { attempted: true, knownSchemaDriftBugs: [], sources: {} },
    minAdjudicatedPerCorpus: 2,
  });
  assert('P6-6b. measured-zero SAFE_FIX population blocks Green without Red',
    readiness.gate === 'Yellow' &&
    hasReason(readiness, 'safe-fix-population-empty') &&
    !hasReason(readiness, 'fp-rate-unknown') &&
    readiness.reviewVisibleCleanup.fpRate === 0,
    JSON.stringify(readiness));
}

// ─── Schema round-trip record-based v1 ────────────────────

{
  const rt = buildSchemaRoundTrip({
    manifest: null,
    canonDrift: {
      perSource: {
        naming: { status: 'parse-error', driftCount: 0 },
        topology: { status: 'clean', driftCount: 0 },
      },
    },
  });
  assert('P6-7a. existing canon-drift checked source marks roundtrip attempted',
    rt.attempted === true, JSON.stringify(rt));
  assert('P6-7b. parse-error source records known schema drift bug',
    rt.knownSchemaDriftBugs.some((b) => b.source === 'naming' && b.status === 'parse-error'),
    JSON.stringify(rt));
}

{
  const rt = buildSchemaRoundTrip({
    manifest: {
      checkCanon: {
        perSource: {
          naming: { status: 'skipped-missing-canon', driftCount: 0 },
        },
      },
    },
    canonDrift: {
      perSource: {
        naming: { status: 'drift', driftCount: 1 },
      },
    },
  });
  assert('P6-7c. direct canon-drift artifact overrides stale manifest.checkCanon',
    rt.attempted === true &&
    rt.sources.naming.status === 'drift' &&
    rt.sources.naming.driftCount === 1,
    JSON.stringify(rt));
}

// ─── Multi-corpus merge (P6-0b measurement-only) ──────────

{
  const merged = mergeMeasurementArtifacts([
    {
      corpus: [{ name: 'a', commit: 'abc', worktreeDirty: false, locBucket: '25k' }],
      candidateCounts: {
        available: true,
        missingArtifacts: [],
        reviewVisibleCleanup: 1,
        safeFix: 0,
        reviewFix: 1,
        degraded: 0,
        muted: 0,
        rawTierC: 1,
        canonDrift: {
          available: true,
          missingArtifacts: [],
          total: 0,
          perSource: { naming: { status: 'clean', driftCount: 0 } },
        },
      },
      adjudication: {
        entries: [{ corpusName: 'a', tier: 'REVIEW_FIX', verdict: 'true_dead' }],
      },
      runtime: { wallMs: 10, childProcessCount: 1, steps: [{ step: 'audit', status: 'ok', ms: 10 }] },
      schemaRoundTrip: {
        attempted: true,
        sources: { naming: { status: 'clean', driftCount: 0 } },
        knownSchemaDriftBugs: [],
      },
    },
    {
      corpus: [{ name: 'b', commit: 'def', worktreeDirty: false, locBucket: '50k' }],
      candidateCounts: {
        available: true,
        missingArtifacts: [],
        reviewVisibleCleanup: 1,
        safeFix: 1,
        reviewFix: 0,
        degraded: 0,
        muted: 0,
        rawTierC: 1,
        canonDrift: {
          available: true,
          missingArtifacts: [],
          total: 0,
          perSource: { topology: { status: 'clean', driftCount: 0 } },
        },
      },
      adjudication: {
        entries: [{ corpusName: 'b', tier: 'SAFE_FIX', verdict: 'true_dead' }],
      },
      runtime: { wallMs: 20, childProcessCount: 2, steps: [{ step: 'audit', status: 'ok', ms: 20 }] },
      schemaRoundTrip: {
        attempted: true,
        sources: { topology: { status: 'clean', driftCount: 0 } },
        knownSchemaDriftBugs: [],
      },
    },
  ]);
  const readiness = computeReadiness({
    corpus: merged.corpus,
    candidateCounts: merged.candidateCounts,
    adjudicationEntries: merged.adjudicationEntries,
    schemaRoundTrip: merged.schemaRoundTrip,
    minAdjudicatedPerCorpus: 1,
  });
  assert('P6-8a. merge sums cleanup counts and keeps per-corpus totals',
    merged.candidateCounts.reviewVisibleCleanup === 2 &&
    merged.candidateCounts.byCorpus.a.reviewVisibleCleanup === 1 &&
    merged.candidateCounts.byCorpus.b.reviewVisibleCleanup === 1,
    JSON.stringify(merged.candidateCounts));
  assert('P6-8b. merge prefixes schema/canon sources by corpus',
    merged.schemaRoundTrip.sources['a:naming'].status === 'clean' &&
    merged.candidateCounts.canonDrift.perSource['b:topology'].status === 'clean',
    JSON.stringify(merged.schemaRoundTrip));
  assert('P6-8c. merged two-corpus low-FP baseline can reach Green',
    readiness.gate === 'Green',
    JSON.stringify(readiness));
}

{
  const baseCandidateCounts = (tier) => ({
    available: true,
    missingArtifacts: [],
    reviewVisibleCleanup: 1,
    safeFix: tier === 'SAFE_FIX' ? 1 : 0,
    reviewFix: tier === 'REVIEW_FIX' ? 1 : 0,
    degraded: 0,
    muted: 0,
    rawTierC: 1,
    canonDrift: { available: true, missingArtifacts: [], total: 0, perSource: {} },
  });
  const merged = mergeMeasurementArtifacts([
    {
      corpus: [{ name: 'reviewed', commit: 'abc', worktreeDirty: false, locBucket: '25k' }],
      candidateCounts: baseCandidateCounts('REVIEW_FIX'),
      adjudication: { entries: [{ corpusName: 'reviewed', tier: 'REVIEW_FIX', verdict: 'true_dead' }] },
      schemaRoundTrip: { attempted: true, sources: {}, knownSchemaDriftBugs: [] },
    },
    {
      corpus: [{ name: 'unreviewed', commit: 'def', worktreeDirty: false, locBucket: '25k' }],
      candidateCounts: baseCandidateCounts('REVIEW_FIX'),
      adjudication: { entries: [] },
      schemaRoundTrip: { attempted: true, sources: {}, knownSchemaDriftBugs: [] },
    },
  ]);
  const readiness = computeReadiness({
    corpus: merged.corpus,
    candidateCounts: merged.candidateCounts,
    adjudicationEntries: merged.adjudicationEntries,
    schemaRoundTrip: merged.schemaRoundTrip,
  });
  assert('P6-8d. merged corpus with review-visible candidates but no adjudication is Red',
    readiness.gate === 'Red' && hasReason(readiness, 'fp-rate-unknown'),
    JSON.stringify(readiness));
}

// ─── CLI smoke ────────────────────────────────────────────

{
  const root = mkdtempSync(path.join(tmpdir(), 'p6m-root-'));
  const out = mkdtempSync(path.join(tmpdir(), 'p6m-out-'));
  const adj = path.join(root, 'adjudication.json');
  try {
    writeFileSync(path.join(out, 'fix-plan.json'), JSON.stringify({
      safeFixes: [{ finding: { file: 'src/a.ts', symbol: 'A', line: 1, bucket: 'C' } }],
      reviewFixes: [],
      degraded: [],
      muted: [],
    }));
    writeFileSync(path.join(out, 'dead-classify.json'), JSON.stringify({
      summary: { category_C: 1 },
    }));
    writeFileSync(path.join(out, 'canon-drift.json'), JSON.stringify({
      summary: { driftCount: 0 },
      perSource: { naming: { status: 'clean', driftCount: 0 } },
    }));
    writeFileSync(path.join(out, 'manifest.json'), JSON.stringify({
      commandsRun: [{ step: 'rank-fixes.mjs', status: 'ok', ms: 12 }],
    }));
    writeFileSync(adj, JSON.stringify({
      entries: [{ corpusName: 'cli', tier: 'SAFE_FIX', verdict: 'true_dead' }],
    }));

    execFileSync(NODE, [CLI,
      '--root', root,
      '--output', out,
      '--corpus-name', 'cli',
      '--repo', 'local-cli',
      '--commit', 'abc123',
      '--worktree-dirty', 'false',
      '--loc-bucket', '25k',
      '--adjudication', adj,
    ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const artifact = JSON.parse(readFileSync(path.join(out, 'p6-measurement.json'), 'utf8'));
    assert('P6-9a. CLI writes p6-measurement.json',
      artifact.schemaVersion === 'p6-measurement.v1',
      JSON.stringify(artifact));
    assert('P6-9b. CLI preserves candidate counts and adjudication entries',
      artifact.candidateCounts.safeFix === 1 && artifact.adjudication.entries.length === 1,
      JSON.stringify(artifact));
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

{
  const root = mkdtempSync(path.join(tmpdir(), 'p6m-merge-root-'));
  const out = mkdtempSync(path.join(tmpdir(), 'p6m-merge-out-'));
  const a = path.join(root, 'a.json');
  const b = path.join(root, 'b.json');
  try {
    const artifact = (name, tier) => ({
      schemaVersion: 'p6-measurement.v1',
      corpus: [{ name, commit: `${name}-commit`, worktreeDirty: false, locBucket: '25k' }],
      candidateCounts: {
        available: true,
        missingArtifacts: [],
        reviewVisibleCleanup: 1,
        safeFix: tier === 'SAFE_FIX' ? 1 : 0,
        reviewFix: tier === 'REVIEW_FIX' ? 1 : 0,
        degraded: 0,
        muted: 0,
        rawTierC: 1,
        canonDrift: { available: true, missingArtifacts: [], total: 0, perSource: {} },
      },
      adjudication: { entries: [{ corpusName: name, tier, verdict: 'true_dead' }] },
      runtime: { wallMs: 1, childProcessCount: 1, steps: [] },
      schemaRoundTrip: { attempted: true, sources: {}, knownSchemaDriftBugs: [] },
    });
    writeFileSync(a, JSON.stringify(artifact('merge-a', 'REVIEW_FIX')));
    writeFileSync(b, JSON.stringify(artifact('merge-b', 'SAFE_FIX')));

    execFileSync(NODE, [CLI,
      '--root', root,
      '--output', out,
      '--merge', a,
      '--merge', b,
    ], { cwd: DIR, stdio: ['ignore', 'pipe', 'pipe'] });

    const merged = JSON.parse(readFileSync(path.join(out, 'p6-measurement.json'), 'utf8'));
    assert('P6-10a. CLI --merge writes a merged p6-measurement.json',
      merged.meta.mode === 'merge' && merged.corpus.length === 2,
      JSON.stringify(merged));
    assert('P6-10b. CLI --merge recomputes aggregate readiness',
      merged.readiness.gate === 'Green' &&
      merged.candidateCounts.reviewVisibleCleanup === 2,
      JSON.stringify(merged.readiness));
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

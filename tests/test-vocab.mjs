// Pins `_lib/vocab.mjs` constant values so a silent rename in the
// vocab module doesn't desync consumers.
//
// Downstream consumers (SARIF readers, external tooling that reads
// `dead-classify.json` or `fix-plan.json`) depend on the exact string
// values of evidence labels and taint kinds. Renaming `TAINT.UNRESOLVED_SPEC_MATCH`
// from `'unresolved-specifier-could-match'` to anything else is a
// breaking change — this test surfaces that as an explicit failure
// rather than a silently-dropped tier.
//
// If a value intentionally changes, the diff here is the checkpoint:
// the test change, the constant change, and the downstream consumer
// updates should all land in the same commit.

import {
  EVIDENCE,
  EVIDENCE_VALUES,
  TAINT,
  BLOCKING_TAINTS,
  SOFT_TAINTS,
  provenanceFields,
  getProvenanceFieldNames,
} from '../_lib/vocab.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail) {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else {
    failed++;
    console.log(`  FAIL  ${label}`);
    if (detail) console.log(`        ${detail}`);
  }
}
function eq(label, actual, expected) {
  const ok = actual === expected;
  assert(label, ok, ok ? '' : `expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
}

// ─── Evidence labels ──────────────────────────────────────────
eq('E1. EVIDENCE.AST_REF_COUNT literal',      EVIDENCE.AST_REF_COUNT,    'ast-ident-ref-count');
eq('E2. EVIDENCE.TEXT_ZERO_REF_COUNT literal', EVIDENCE.TEXT_ZERO_REF_COUNT, 'text-zero-ident-ref-count');
eq('E3. EVIDENCE.REGEX_FALLBACK literal',     EVIDENCE.REGEX_FALLBACK,   'regex-text-fallback-parse-error');
eq('E4. EVIDENCE.PARSE_ERROR literal',        EVIDENCE.PARSE_ERROR,      'parse-error');

assert('E5. EVIDENCE_VALUES Set has every EVIDENCE value',
  Object.values(EVIDENCE).every((v) => EVIDENCE_VALUES.has(v)) &&
  EVIDENCE_VALUES.size === Object.values(EVIDENCE).length);

// ─── Taint kinds ──────────────────────────────────────────────
eq('T1. TAINT.UNRESOLVED_SPEC_MATCH literal',       TAINT.UNRESOLVED_SPEC_MATCH,       'unresolved-specifier-could-match');
eq('T2. TAINT.UNRESOLVED_SPEC_MATCH_UNKNOWN literal', TAINT.UNRESOLVED_SPEC_MATCH_UNKNOWN, 'unresolved-specifier-could-match-unknown');
eq('T3. TAINT.RESOLVER_BLIND_ZONE_RELEVANT literal', TAINT.RESOLVER_BLIND_ZONE_RELEVANT, 'resolver-blind-zone-relevant');
eq('T4. TAINT.GENERATED_ARTIFACT_MISSING_RELEVANT literal', TAINT.GENERATED_ARTIFACT_MISSING_RELEVANT, 'generated-artifact-missing-relevant');
eq('T5. TAINT.DEFINING_FILE_PARSE_ERROR literal',   TAINT.DEFINING_FILE_PARSE_ERROR,   'defining-file-parse-error');
eq('T6. TAINT.PARSE_ERRORS_ELSEWHERE literal',      TAINT.PARSE_ERRORS_ELSEWHERE,      'parse-errors-present');

// ─── Severity groups ──────────────────────────────────────────
assert('S1. BLOCKING_TAINTS contains UNRESOLVED_SPEC_MATCH',
  BLOCKING_TAINTS.has(TAINT.UNRESOLVED_SPEC_MATCH));
assert('S2. BLOCKING_TAINTS contains DEFINING_FILE_PARSE_ERROR',
  BLOCKING_TAINTS.has(TAINT.DEFINING_FILE_PARSE_ERROR));
assert('S3. SOFT_TAINTS contains PARSE_ERRORS_ELSEWHERE',
  SOFT_TAINTS.has(TAINT.PARSE_ERRORS_ELSEWHERE));
assert('S3b. SOFT_TAINTS contains RESOLVER_BLIND_ZONE_RELEVANT',
  SOFT_TAINTS.has(TAINT.RESOLVER_BLIND_ZONE_RELEVANT));
assert('S4. BLOCKING and SOFT do not overlap',
  ![...BLOCKING_TAINTS].some((k) => SOFT_TAINTS.has(k)));
assert('S5. every TAINT value is in either BLOCKING or SOFT',
  Object.values(TAINT).every((v) => BLOCKING_TAINTS.has(v) || SOFT_TAINTS.has(v)));

// ─── Frozen — defensive against accidental mutation ──────────
assert('F1. EVIDENCE is frozen',       Object.isFrozen(EVIDENCE));
assert('F2. TAINT is frozen',          Object.isFrozen(TAINT));
assert('F3. BLOCKING_TAINTS is frozen', Object.isFrozen(BLOCKING_TAINTS));
assert('F4. SOFT_TAINTS is frozen',    Object.isFrozen(SOFT_TAINTS));

// ─── provenanceFields forwarder ───────────────────────────────
{
  const c = {
    // Normal classified-candidate shape from classify-dead-exports.
    symbol: 'x', file: 'src/x.ts', line: 1, kind: 'VariableDeclaration',
    fileInternalUses: 0,
    fileInternalUsesEvidence: 'ast-ident-ref-count',
    fileInternalRefs: { typeRefs: 0, valueRefs: 0 },
    supportedBy: [{ kind: 'ast-ident-ref-count', count: 0 }],
    taintedBy: [],
    resolverConfidence: 'high',
    parseStatus: 'ok',
    declarationExportDependency: true,
    declarationExportRefs: { count: 1, lines: [2] },
  };
  const out = provenanceFields(c);
  assert('P1. forwarder carries fileInternalUsesEvidence',
    out.fileInternalUsesEvidence === c.fileInternalUsesEvidence);
  assert('P2. forwarder carries fileInternalRefs',
    out.fileInternalRefs === c.fileInternalRefs);
  assert('P3. forwarder carries supportedBy',
    out.supportedBy === c.supportedBy);
  assert('P4. forwarder carries taintedBy',
    out.taintedBy === c.taintedBy);
  assert('P5. forwarder carries resolverConfidence',
    out.resolverConfidence === c.resolverConfidence);
  assert('P6. forwarder carries parseStatus',
    out.parseStatus === c.parseStatus);
  assert('P7. forwarder carries declarationExportDependency',
    out.declarationExportDependency === c.declarationExportDependency);
  assert('P8. forwarder carries declarationExportRefs',
    out.declarationExportRefs === c.declarationExportRefs);
  assert('P9. forwarder does NOT leak unrelated fields (symbol/file/line)',
    out.symbol === undefined && out.file === undefined && out.line === undefined);
}

{
  // `undefined` values are OMITTED, not copied as undefined keys —
  // consumers can still use `if (x.foo)` without tripping on the key.
  const c = { fileInternalUsesEvidence: undefined, parseStatus: 'ok' };
  const out = provenanceFields(c);
  assert('P10. undefined values are omitted from the forwarded object',
    !('fileInternalUsesEvidence' in out) && out.parseStatus === 'ok');
}

// ─── Field-name inventory ────────────────────────────────────
{
  const names = getProvenanceFieldNames();
  assert('P11. getProvenanceFieldNames returns an array',
    Array.isArray(names) && names.length > 0);
  // Returned array must be a COPY — mutating it doesn't affect the
  // forwarder's internal list.
  names.push('hackedField');
  const again = getProvenanceFieldNames();
  assert('P12. getProvenanceFieldNames returns a fresh copy each call',
    !again.includes('hackedField'));
}

// ─── DELTA_LABELS — P2-1 6-label enumeration (D-2 fix, 2026-04-21) ────
//
// Post-write delta entries carry one of six labels. Before the vocab
// export, each of `summarize()` and `requiredAcknowledgements()` used
// hardcoded string comparisons — a rename in one site + miss in the
// other would silently drop summary counters. Pinning the enumeration
// here makes that class of drift fail at test time.
{
  const { DELTA_LABELS, DELTA_LABEL_VALUES } = await import('../_lib/vocab.mjs');

  assert('L1. DELTA_LABELS.PLANNED === "planned"',
    DELTA_LABELS.PLANNED === 'planned');
  assert('L2. DELTA_LABELS.PLANNED_NOT_OBSERVED === "planned-not-observed"',
    DELTA_LABELS.PLANNED_NOT_OBSERVED === 'planned-not-observed');
  assert('L3. DELTA_LABELS.SILENT_NEW === "silent-new"',
    DELTA_LABELS.SILENT_NEW === 'silent-new');
  assert('L4. DELTA_LABELS.PRE_EXISTING === "pre-existing"',
    DELTA_LABELS.PRE_EXISTING === 'pre-existing');
  assert('L5. DELTA_LABELS.REMOVED === "removed"',
    DELTA_LABELS.REMOVED === 'removed');
  assert('L6. DELTA_LABELS.OBSERVED_UNBASELINED === "observed-unbaselined"',
    DELTA_LABELS.OBSERVED_UNBASELINED === 'observed-unbaselined');

  assert('L7. exactly 6 DELTA_LABELS entries',
    Object.keys(DELTA_LABELS).length === 6);
  assert('L8. DELTA_LABEL_VALUES mirrors DELTA_LABELS values',
    DELTA_LABEL_VALUES.size === 6 &&
    Object.values(DELTA_LABELS).every((v) => DELTA_LABEL_VALUES.has(v)));
  assert('L9. DELTA_LABELS is frozen',
    Object.isFrozen(DELTA_LABELS));

  const EXPECTED = new Set([
    'planned', 'planned-not-observed', 'silent-new',
    'pre-existing', 'removed', 'observed-unbaselined',
  ]);
  const actual = new Set(Object.values(DELTA_LABELS));
  assert('L10. DELTA_LABELS matches canonical 6-label union from p2-1.md v3 §4.1',
    actual.size === EXPECTED.size &&
    [...EXPECTED].every((v) => actual.has(v)));

  const { requiredAcknowledgements } = await import('../_lib/post-write-delta.mjs');
  const fakeDelta = {
    entries: Object.values(DELTA_LABELS).map((label) => ({ label, diagnostics: [] })),
  };
  const req = requiredAcknowledgements(fakeDelta);
  assert('L11. requiredAcknowledgements filters to exactly DELTA_LABELS.SILENT_NEW',
    req.length === 1 && req[0].label === DELTA_LABELS.SILENT_NEW);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

// Tests for _lib/pre-write-lookup-name.mjs — P1-1 step 5.3.
//
// Pinning rules from docs/history/phases/p1/p1-1.md §4.3 + §5.3:
//   - Capability-first: supports.identityFanIn / supports.anyContamination
//     determine whether absence means "clean/grounded" or "[확인 불가]".
//   - Canonical-first: recognized canonical owner claims ARE the FIRST
//     source of "already exists"; cross-check AST afterward.
//   - Identity-keyed fan-in ONLY. symbols.topSymbolFanIn[name] is NEVER
//     read — pinned structurally.
//   - Label-specific contamination rendering (4 states + clean + capability-absent).
//   - EXISTS_MULTIPLE preserves all identities; never silently picks one.
//   - `CANONICAL_EXISTS_AST_DISAGREE` is a STATE, never emits literal
//     `CANONICAL DRIFT:` — that's P1-3.
//   - Near-name hint uses cheap filters (length delta ≤ 2, shared prefix ≥ 4)
//     before Levenshtein.

import { lookupName } from '../_lib/pre-write-lookup-name.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── Helpers: build minimal symbols.json fixtures ─────────────

function buildSymbols({
  identitiesByFile = {},
  fanInByIdentity = {},
  fanInByIdentitySpace = {},
  topSymbolFanIn = [],
  supports = {
    anyContamination: true,
    identityFanIn: true,
    identityFanInSpace: true,
    reExportRecords: 'file-level',
  },
  unresolvedInternalSpecifiers = [],
  filesWithParseErrors = [],
  defAnyContamination = {},  // 'ownerFile::name' → anyContamination obj
  defKindsByIdentity = {},
} = {}) {
  // Build defIndex with optional anyContamination annotations embedded.
  const defIndex = {};
  for (const [file, names] of Object.entries(identitiesByFile)) {
    defIndex[file] = {};
    for (const name of names) {
      const key = `${file}::${name}`;
      const defInfo = {
        kind: defKindsByIdentity[key] ?? 'TSTypeAliasDeclaration',
        line: 1,
      };
      if (defAnyContamination[key]) {
        defInfo.anyContamination = defAnyContamination[key];
      }
      defIndex[file][name] = defInfo;
    }
  }
  return {
    meta: { schemaVersion: 3, supports },
    defIndex,
    fanInByIdentity,
    fanInByIdentitySpace,
    topSymbolFanIn,
    unresolvedInternalSpecifiers,
    filesWithParseErrors,
  };
}

// ═══ Structural: EXISTS single identity, canonical absent ═══

{
  const sym = buildSymbols({
    identitiesByFile: { 'src/utils/date.ts': ['formatDate'] },
    fanInByIdentity: { 'src/utils/date.ts::formatDate': 8 },
    fanInByIdentitySpace: {
      'src/utils/date.ts::formatDate': { value: 7, type: 1, broad: 0 },
    },
  });
  const r = lookupName('formatDate', { symbols: sym, canonicalClaims: [] });
  assert('T1. EXISTS single identity', r.result === 'EXISTS', `result=${r.result}`);
  assert('T1b. identities has exactly one entry',
    r.identities.length === 1);
  assert('T1c. identity keyed ownerFile::exportedName',
    r.identities[0].identity === 'src/utils/date.ts::formatDate');
  assert('T1d. fan-in grounded at 8',
    r.identities[0].fanIn === 8 && r.identities[0].fanInConfidence === 'grounded');
  assert('T1d2. fan-in space breakdown is grounded when producer emits it',
    r.identities[0].fanInSpace?.value === 7 &&
      r.identities[0].fanInSpace?.type === 1 &&
      r.identities[0].fanInSpace?.broad === 0 &&
      r.identities[0].fanInSpaceConfidence === 'grounded',
    JSON.stringify(r.identities[0].fanInSpace));
  assert('T1d3. fan-in space citation names symbols.json.fanInByIdentitySpace',
    r.identities[0].citations.some((c) =>
      c.includes("symbols.json.fanInByIdentitySpace['src/utils/date.ts::formatDate']")),
    JSON.stringify(r.identities[0].citations));
  assert('T1e. canonical not consulted (empty claims)',
    r.canonicalAstStatus === 'not-consulted' && r.canonicalClaim === null);
  assert('T1f. anyContamination clean (supports=true, no annotation)',
    r.identities[0].anyContamination.state === 'clean');
}

// ═══ Fan-in space absent capability remains explicit and non-fatal ═══

{
  const sym = buildSymbols({
    identitiesByFile: { 'src/a.ts': ['formatDate'] },
    fanInByIdentity: { 'src/a.ts::formatDate': 8 },
    fanInByIdentitySpace: { 'src/a.ts::formatDate': { value: 8, type: 0, broad: 0 } },
    supports: {
      anyContamination: true,
      identityFanIn: true,
      identityFanInSpace: false,
      reExportRecords: 'file-level',
    },
  });
  const r = lookupName('formatDate', { symbols: sym, canonicalClaims: [] });
  assert('T1g. supports.identityFanInSpace=false leaves total fan-in grounded',
    r.identities[0].fanIn === 8 && r.identities[0].fanInConfidence === 'grounded');
  assert('T1h. supports.identityFanInSpace=false marks fan-in space unavailable',
    r.identities[0].fanInSpace === null &&
      r.identities[0].fanInSpaceConfidence === 'unavailable',
    JSON.stringify(r.identities[0]));
}

// ═══ EXISTS_MULTIPLE — two identities sharing name ═══

{
  const sym = buildSymbols({
    identitiesByFile: {
      'apps/admin/types.ts': ['User'],
      'apps/blog/types.ts': ['User'],
    },
    fanInByIdentity: {
      'apps/admin/types.ts::User': 5,
      'apps/blog/types.ts::User': 2,
    },
  });
  const r = lookupName('User', { symbols: sym, canonicalClaims: [] });
  assert('T2. EXISTS_MULTIPLE when two files share exportedName',
    r.result === 'EXISTS_MULTIPLE', `result=${r.result}`);
  assert('T2b. identities has both entries',
    r.identities.length === 2);
  const owners = r.identities.map((i) => i.ownerFile).sort();
  assert('T2c. both owner files present',
    owners.includes('apps/admin/types.ts') && owners.includes('apps/blog/types.ts'));
  assert('T2d. per-identity fan-in preserved (5 and 2)',
    r.identities.find((i) => i.ownerFile === 'apps/admin/types.ts').fanIn === 5 &&
    r.identities.find((i) => i.ownerFile === 'apps/blog/types.ts').fanIn === 2);
}

// ═══ Pinning: topSymbolFanIn[name] is NEVER read ═══

{
  const sym = buildSymbols({
    identitiesByFile: { 'src/a.ts': ['formatDate'] },
    // intentionally: fanInByIdentity is empty; topSymbolFanIn has a value
    fanInByIdentity: {},
    topSymbolFanIn: [
      { defFile: 'src/a.ts', symbol: 'formatDate', count: 999, kind: 'const' },
    ],
  });
  const r = lookupName('formatDate', { symbols: sym, canonicalClaims: [] });
  assert('T3. identity-fan-in map EMPTY → fanIn=null regardless of topSymbolFanIn',
    r.identities[0].fanIn === null &&
    r.identities[0].fanInConfidence === 'unavailable',
    `fanIn=${r.identities[0].fanIn}, topSymbolFanIn=${JSON.stringify(sym.topSymbolFanIn)}`);
  const citation = r.identities[0].citations?.join(' ') || '';
  assert('T3b. [확인 불가] citation when fan-in identity-unavailable',
    citation.includes('확인 불가') ||
    (r.citations.join(' ').includes('확인 불가')),
    `citations=${JSON.stringify(r.citations)}`);
}

// ═══ supports.identityFanIn === false ═══

{
  const sym = buildSymbols({
    identitiesByFile: { 'src/a.ts': ['formatDate'] },
    fanInByIdentity: { 'src/a.ts::formatDate': 8 },
    supports: { anyContamination: true, identityFanIn: false, reExportRecords: 'file-level' },
  });
  const r = lookupName('formatDate', { symbols: sym, canonicalClaims: [] });
  assert('T4. supports.identityFanIn=false → fanIn unavailable even if map present',
    r.identities[0].fanIn === null &&
    r.identities[0].fanInConfidence === 'unavailable');
}

// ═══ NOT_OBSERVED with near-name hints ═══

{
  const sym = buildSymbols({
    identitiesByFile: {
      'src/utils/date.ts': ['formatDate', 'formatDateTime', 'formatTimeAgo'],
    },
    fanInByIdentity: {
      'src/utils/date.ts::formatDate': 3,
      'src/utils/date.ts::formatDateTime': 1,
      'src/utils/date.ts::formatTimeAgo': 2,
    },
  });
  const r = lookupName('formatTimestamp', { symbols: sym, canonicalClaims: [] });
  assert('T5. NOT_OBSERVED when no exact match',
    r.result === 'NOT_OBSERVED');
  assert('T5b. identities empty',
    r.identities.length === 0);
  assert('T5c. nearNames array populated with similar names',
    r.nearNames.length > 0 &&
    r.nearNames.some((n) => n.name === 'formatDate' || n.name === 'formatDateTime'),
    `nearNames=${JSON.stringify(r.nearNames)}`);
  assert('T5d. nearNames capped at 5',
    r.nearNames.length <= 5);
}

// ═══ NOT_OBSERVED with NO near-names (distant names) ═══

{
  const sym = buildSymbols({
    identitiesByFile: {
      'src/x.ts': ['completelyUnrelatedThing', 'another_entirely_different'],
    },
    fanInByIdentity: {},
  });
  const r = lookupName('xyzzy', { symbols: sym, canonicalClaims: [] });
  assert('T6. NOT_OBSERVED with distant names yields empty nearNames',
    r.result === 'NOT_OBSERVED' && r.nearNames.length === 0);
}

// ═══ NOT_OBSERVED with intent-token semantic hints ═══
//
// These are deliberately search hints, not reuse claims. They catch the
// vibe-coder case where the planned name is morphologically distant from
// the existing helper but the intent words line up.

{
  const sym = buildSymbols({
    identitiesByFile: {
      '_lib/artifacts.mjs': ['loadIfExists', 'readJsonFile'],
      '_lib/check-canon-artifact.mjs': ['loadHelperRegistryCanon'],
    },
    fanInByIdentity: {},
  });
  const r = lookupName('loadArtifactJson', {
    symbols: sym,
    canonicalClaims: [],
    intentDeclaration: {
      name: 'loadArtifactJson',
      kind: 'function',
      why: 'load a JSON artifact file with existence check',
    },
  });
  assert('T6b. semanticHints catch existing artifact JSON helpers',
    r.result === 'NOT_OBSERVED' &&
    r.semanticHints.some((h) => h.name === 'loadIfExists') &&
    r.semanticHints.some((h) => h.name === 'readJsonFile'),
    JSON.stringify(r.semanticHints));
  assert('T6c. irrelevant morphology-only load helper is not promoted by intent tokens',
    !r.semanticHints.some((h) => h.name === 'loadHelperRegistryCanon'),
    JSON.stringify(r.semanticHints));
}

{
  const sym = buildSymbols({
    identitiesByFile: {
      '_lib/paths.mjs': ['relPath', 'fileExists', 'pathExists'],
    },
    fanInByIdentity: {},
  });
  const r = lookupName('getRelativePath', { symbols: sym, canonicalClaims: [] });
  assert('T6d. semanticHints normalize relPath ↔ relative path',
    r.result === 'NOT_OBSERVED' &&
    r.nearNames.length === 0 &&
    r.semanticHints.some((h) =>
      h.name === 'relPath' &&
      h.matchedTokens.includes('relative') &&
      h.matchedTokens.includes('path')
    ),
    JSON.stringify({ nearNames: r.nearNames, semanticHints: r.semanticHints }));
}

// ═══ Canonical-first: CANONICAL_EXISTS_AND_EXISTS ═══

{
  const sym = buildSymbols({
    identitiesByFile: { 'src/protocol/ids.ts': ['SessionId'] },
    fanInByIdentity: { 'src/protocol/ids.ts::SessionId': 8 },
  });
  const canonicalClaims = [
    { name: 'SessionId', ownerFile: 'src/protocol/ids.ts', line: 42, file: 'canonical/type-ownership.md', section: 'Single owner (strong)' },
  ];
  const r = lookupName('SessionId', { symbols: sym, canonicalClaims });
  assert('T7. CANONICAL_EXISTS_AND_EXISTS when canonical aligned with AST',
    r.result === 'CANONICAL_EXISTS_AND_EXISTS');
  assert('T7b. canonicalClaim present with line number',
    r.canonicalClaim !== null && r.canonicalClaim.line === 42);
  assert('T7c. canonicalAstStatus aligned',
    r.canonicalAstStatus === 'aligned');
}

// ═══ CANONICAL_EXISTS_AST_ABSENT ═══

{
  const sym = buildSymbols({
    identitiesByFile: {},
    fanInByIdentity: {},
  });
  const canonicalClaims = [
    { name: 'TokenKind', ownerFile: 'src/auth/token.ts', line: 7, file: 'canonical/type-ownership.md', section: 'Single owner (strong)' },
  ];
  const r = lookupName('TokenKind', { symbols: sym, canonicalClaims });
  assert('T8. CANONICAL_EXISTS_AST_ABSENT when canonical says X, AST empty',
    r.result === 'CANONICAL_EXISTS_AST_ABSENT');
  assert('T8b. canonicalAstStatus ast-absent',
    r.canonicalAstStatus === 'ast-absent');
  assert('T8c. identities empty',
    r.identities.length === 0);
}

// ═══ CANONICAL_EXISTS_AST_DISAGREE (no CANONICAL DRIFT literal) ═══

{
  const sym = buildSymbols({
    identitiesByFile: { 'src/other/path.ts': ['User'] },
    fanInByIdentity: { 'src/other/path.ts::User': 3 },
  });
  const canonicalClaims = [
    { name: 'User', ownerFile: 'src/models/User.ts', line: 3, file: 'canonical/type-ownership.md', section: 'Single owner (strong)' },
  ];
  const r = lookupName('User', { symbols: sym, canonicalClaims });
  assert('T9. CANONICAL_EXISTS_AST_DISAGREE when AST owner differs',
    r.result === 'CANONICAL_EXISTS_AST_DISAGREE',
    `result=${r.result}`);
  assert('T9b. canonicalAstStatus owner-disagrees',
    r.canonicalAstStatus === 'owner-disagrees');
  assert('T9c. identities preserved (the one AST observes)',
    r.identities.length === 1 && r.identities[0].ownerFile === 'src/other/path.ts');
  // P1-1 contract: do NOT emit literal "CANONICAL DRIFT:" — that's P1-3.
  const allText = JSON.stringify(r);
  assert('T9d. result does NOT contain literal "CANONICAL DRIFT:" (reserved for P1-3)',
    !allText.includes('CANONICAL DRIFT:'),
    `result contained the phrase`);
}

// ═══ Canonical absent/unrecognized → not-consulted ═══

{
  const sym = buildSymbols({
    identitiesByFile: { 'src/a.ts': ['formatDate'] },
    fanInByIdentity: { 'src/a.ts::formatDate': 2 },
  });
  const r = lookupName('formatDate', { symbols: sym, canonicalClaims: [] });
  assert('T10. canonical empty list → canonicalAstStatus "not-consulted"',
    r.canonicalAstStatus === 'not-consulted' && r.result === 'EXISTS');
}

// ═══ anyContamination state matrix ═══

// Clean
{
  const sym = buildSymbols({
    identitiesByFile: { 'src/t.ts': ['CleanType'] },
    fanInByIdentity: { 'src/t.ts::CleanType': 1 },
    // no defAnyContamination → annotation absent
  });
  const r = lookupName('CleanType', { symbols: sym, canonicalClaims: [] });
  assert('T11. supports=true + no annotation → state:clean',
    r.identities[0].anyContamination.state === 'clean');
}

// unknown-surface only
{
  const sym = buildSymbols({
    identitiesByFile: { 'src/t.ts': ['UnkType'] },
    fanInByIdentity: { 'src/t.ts::UnkType': 1 },
    defAnyContamination: {
      'src/t.ts::UnkType': {
        label: 'unknown-surface',
        labels: ['unknown-surface'],
        measurements: { totalFields: 2, anyFields: 0, unknownFields: 2, anyFieldRatio: 0 },
      },
    },
  });
  const r = lookupName('UnkType', { symbols: sym, canonicalClaims: [] });
  assert('T12. unknown-surface only → state:unknown-surface-only',
    r.identities[0].anyContamination.state === 'unknown-surface-only');
  // NEVER contaminated wording
  const txt = JSON.stringify(r.identities[0].anyContamination);
  assert('T12b. unknown-surface never uses "contaminated" in its state',
    r.identities[0].anyContamination.state !== 'any-contaminated');
}

// has-any only (mild)
{
  const sym = buildSymbols({
    identitiesByFile: { 'src/t.ts': ['MildType'] },
    fanInByIdentity: { 'src/t.ts::MildType': 1 },
    defAnyContamination: {
      'src/t.ts::MildType': {
        label: 'has-any',
        labels: ['has-any'],
        measurements: { totalFields: 5, anyFields: 1, unknownFields: 0, anyFieldRatio: 0.2 },
      },
    },
  });
  const r = lookupName('MildType', { symbols: sym, canonicalClaims: [] });
  assert('T13. has-any only → state:has-any-only',
    r.identities[0].anyContamination.state === 'has-any-only',
    `state=${r.identities[0].anyContamination.state}`);
}

// any-contaminated
{
  const sym = buildSymbols({
    identitiesByFile: { 'src/t.ts': ['DirtyType'] },
    fanInByIdentity: { 'src/t.ts::DirtyType': 1 },
    defAnyContamination: {
      'src/t.ts::DirtyType': {
        label: 'any-contaminated',
        labels: ['has-any', 'any-contaminated'],
        measurements: { totalFields: 3, anyFields: 2, unknownFields: 0, anyFieldRatio: 0.67 },
      },
    },
  });
  const r = lookupName('DirtyType', { symbols: sym, canonicalClaims: [] });
  assert('T14. any-contaminated → state:any-contaminated',
    r.identities[0].anyContamination.state === 'any-contaminated');
  assert('T14b. raw measurements surfaced',
    r.identities[0].anyContamination.measurements?.anyFieldRatio === 0.67);
  assert('T14c. any-contaminated uses warn-on-reuse recommendation, not degraded measurement',
    r.identities[0].anyContamination.recommendation?.action === 'warn-on-reuse' &&
      r.identities[0].citations.some((c) => c.includes("[grounded, anyContamination.label = 'any-contaminated'")) &&
      !r.identities[0].citations.some((c) => c.includes('[degraded, any-contaminated')),
    JSON.stringify(r.identities[0], null, 2));
}

// severely-any-contaminated
{
  const sym = buildSymbols({
    identitiesByFile: { 'src/t.ts': ['VeryDirty'] },
    fanInByIdentity: { 'src/t.ts::VeryDirty': 1 },
    defAnyContamination: {
      'src/t.ts::VeryDirty': {
        label: 'severely-any-contaminated',
        labels: ['has-any', 'any-contaminated', 'severely-any-contaminated'],
        measurements: { totalFields: 7, anyFields: 6, unknownFields: 0, anyFieldRatio: 0.85, indexSignatureAny: false },
      },
    },
  });
  const r = lookupName('VeryDirty', { symbols: sym, canonicalClaims: [] });
  assert('T15. severely-any-contaminated → state:severely-any-contaminated',
    r.identities[0].anyContamination.state === 'severely-any-contaminated');
  assert('T15b. raw measurements preserved (anyFieldRatio 0.85)',
    r.identities[0].anyContamination.measurements?.anyFieldRatio === 0.85);
  assert('T15c. severe contamination uses warn-on-reuse recommendation, not degraded measurement',
    r.identities[0].anyContamination.recommendation?.action === 'warn-on-reuse' &&
      r.identities[0].citations.some((c) => c.includes("[grounded, anyContamination.label = 'severely-any-contaminated'")) &&
      !r.identities[0].citations.some((c) => c.includes('[degraded, any-contaminated')),
    JSON.stringify(r.identities[0], null, 2));
}

// capability-absent: supports=false + annotation absent → NOT clean
{
  const sym = buildSymbols({
    identitiesByFile: { 'src/t.ts': ['UnknownContamType'] },
    fanInByIdentity: { 'src/t.ts::UnknownContamType': 1 },
    supports: { anyContamination: false, identityFanIn: true, reExportRecords: 'file-level' },
  });
  const r = lookupName('UnknownContamType', { symbols: sym, canonicalClaims: [] });
  assert('T16. supports.anyContamination=false → state:capability-absent (NOT clean)',
    r.identities[0].anyContamination.state === 'capability-absent',
    `state=${r.identities[0].anyContamination.state}`);
}

// ═══ Resolver confidence demotion ═══

{
  const sym = buildSymbols({
    identitiesByFile: { 'apps/other/components/authControl.tsx': ['AuthControl'] },
    fanInByIdentity: { 'apps/other/components/authControl.tsx::AuthControl': 0 },
    unresolvedInternalSpecifiers: ['@/components/authControl'],
  });
  const r = lookupName('AuthControl', { symbols: sym, canonicalClaims: [] });
  assert('T17. path-shape matches unresolved specifier → resolverConfidence demoted',
    r.identities[0].resolverConfidence === 'medium' || r.identities[0].resolverConfidence === 'low',
    `resolverConfidence=${r.identities[0].resolverConfidence}`);
}

{
  const sym = buildSymbols({
    identitiesByFile: { 'src/broken.ts': ['BrokenSym'] },
    fanInByIdentity: { 'src/broken.ts::BrokenSym': 0 },
    filesWithParseErrors: ['src/broken.ts'],
  });
  const r = lookupName('BrokenSym', { symbols: sym, canonicalClaims: [] });
  assert('T18. file in parseErrors → resolverConfidence demoted',
    r.identities[0].resolverConfidence !== 'high',
    `resolverConfidence=${r.identities[0].resolverConfidence}`);
}

// ═══ Citations shape: every result has citations array ═══

{
  const sym = buildSymbols({
    identitiesByFile: { 'src/a.ts': ['formatDate'] },
    fanInByIdentity: { 'src/a.ts::formatDate': 8 },
  });
  const r = lookupName('formatDate', { symbols: sym, canonicalClaims: [] });
  assert('T19. result has citations array',
    Array.isArray(r.citations));
  assert('T19b. at least one citation carries [grounded ...]',
    r.citations.some((c) => /\[grounded/.test(c)));
}

// ═══ Near-name cheap filter behavior ═══
//
// Length delta > 2 → candidate skipped before Levenshtein. Pinning test:
// exact distance would qualify, but length delta rule kills it.

{
  const sym = buildSymbols({
    identitiesByFile: {
      'src/a.ts': [
        'fmt',                      // too short — length delta vs 'format' is 3
        'format',                   // exact distance 0 vs 'format' (but we query different)
        'formatLongerName',         // length delta vs 'format' = 10
      ],
    },
    fanInByIdentity: {},
  });
  const r = lookupName('formatX', { symbols: sym, canonicalClaims: [] });
  // 'format' has length delta 1 vs 'formatX' (7), distance 1 → qualifies
  // 'fmt' has length delta 4 vs 'formatX' — should be filtered out
  // 'formatLongerName' length delta vs 'formatX' = 9 — filtered out
  assert('T20. cheap filter A: length delta > 2 candidate NOT in nearNames',
    !r.nearNames.some((n) => n.name === 'fmt' || n.name === 'formatLongerName'),
    `nearNames=${JSON.stringify(r.nearNames)}`);
  assert('T20b. qualifying candidate (length delta ≤ 2) IS in nearNames',
    r.nearNames.some((n) => n.name === 'format'));
}

// ═══ Canonical owner-disagrees but AST has multiple + X ═══
//
// Edge case from p1-1.md §4.3: canonical says X, AST has X AND Y.
// Result: CANONICAL_EXISTS_AND_EXISTS (canonical aligned with X) with
// identities.length >= 2. The extra Y is surfaced — canon didn't
// declare it, Claude sees it anyway.

{
  const sym = buildSymbols({
    identitiesByFile: {
      'src/models/User.ts': ['User'],
      'apps/legacy/user.ts': ['User'],
    },
    fanInByIdentity: {
      'src/models/User.ts::User': 5,
      'apps/legacy/user.ts::User': 1,
    },
  });
  const canonicalClaims = [
    { name: 'User', ownerFile: 'src/models/User.ts', line: 3, file: 'canonical/type-ownership.md', section: 'Single owner (strong)' },
  ];
  const r = lookupName('User', { symbols: sym, canonicalClaims });
  assert('T21. canonical aligns with one of multiple identities → CANONICAL_EXISTS_AND_EXISTS',
    r.result === 'CANONICAL_EXISTS_AND_EXISTS');
  assert('T21b. all identities preserved (the canonical one + extras)',
    r.identities.length === 2);
  assert('T21c. canonicalAstStatus is aligned (canonical owner IS in AST)',
    r.canonicalAstStatus === 'aligned');
}

// ═══ Cue-tier token policy: weak common token only is suppressed ═══

{
  const sym = buildSymbols({
    identitiesByFile: {
      'src/store.ts': ['createStore'],
      'src/storage.ts': ['createJSONStorage'],
    },
    fanInByIdentity: {},
  });
  const r = lookupName('createLogger', {
    symbols: sym,
    canonicalClaims: [],
    intentDeclaration: {
      name: 'createLogger',
      kind: 'function',
      why: 'create a logger helper',
    },
  });
  assert('T22. createLogger does not promote create-only token matches',
    r.semanticHints.length === 0,
    JSON.stringify(r.semanticHints));
  assert('T22a. createLogger does not promote create-only near-name matches',
    r.nearNames.length === 0,
    JSON.stringify(r.nearNames));
  assert('T22b. create-only candidates are preserved as suppressedSemanticHints',
    r.suppressedSemanticHints.length === 2 &&
    r.suppressedSemanticHints.every((h) =>
      h.reason === 'domain-token-overlap' &&
      h.matchedTokens.includes('create')),
    JSON.stringify(r.suppressedSemanticHints));
}

{
  const sym = buildSymbols({
    identitiesByFile: {
      'src/users/profile.ts': ['findUserProfile'],
    },
    fanInByIdentity: {},
  });
  const r = lookupName('getUserProfile', {
    symbols: sym,
    canonicalClaims: [],
    intentDeclaration: {
      name: 'getUserProfile',
      kind: 'function',
      why: 'get user profile data',
    },
  });
  assert('T23. weak token plus rare supporting tokens can remain an agent review hint',
    r.semanticHints.some((h) =>
      h.name === 'findUserProfile' &&
      h.matchedTokens.includes('user') &&
      h.matchedTokens.includes('profile')),
    JSON.stringify(r.semanticHints));
}

// ═══ WT-23: suppressed diagnostics explain why plausible candidates fell out ═══

{
  const sym = buildSymbols({
    identitiesByFile: {
      'src/services/user.ts': ['fetchUser'],
      'src/services/post.ts': ['fetchPost'],
      'src/utils/format.ts': ['formatTimestamp'],
    },
    fanInByIdentity: {},
  });
  const r = lookupName('searchUser', {
    symbols: sym,
    canonicalClaims: [],
    intentDeclaration: {
      name: 'searchUser',
      kind: 'function',
      why: 'search user data',
      ownerFile: 'src/services/user-search.ts',
    },
  });
  assert('T24. lookupName always exposes intentTokens for diagnostics',
    Array.isArray(r.intentTokens) &&
      r.intentTokens.includes('search') &&
      r.intentTokens.includes('user'),
    JSON.stringify(r.intentTokens));
  assert('T24a. fetchUser remains below formal semantic/near-name thresholds',
    !r.semanticHints.some((h) => h.name === 'fetchUser') &&
      !r.nearNames.some((h) => h.name === 'fetchUser'),
    JSON.stringify({ semanticHints: r.semanticHints, nearNames: r.nearNames }));
  assert('T24b. fetchUser semantic suppression records single non-weak token',
    r.suppressedSemanticHints.some((h) =>
      h.name === 'fetchUser' &&
      h.reason === 'single-non-weak-token-only' &&
      h.score === 1 &&
      h.matchedTokens.includes('user') &&
      h.locality?.sameDir === true &&
      h.locality?.sameFile === false),
    JSON.stringify(r.suppressedSemanticHints));
  assert('T24c. fetchUser near-name suppression records the distance gate',
    Array.isArray(r.suppressedNearNames) &&
      r.suppressedNearNames.some((h) =>
        h.name === 'fetchUser' &&
        h.reason === 'near-distance-exceeded' &&
        h.distance > 2 &&
        h.locality?.sameDir === true &&
        h.locality?.sameFile === false),
    JSON.stringify(r.suppressedNearNames));
  assert('T24d. suppressed diagnostic counts preserve raw counts separately from capped entries',
    Number.isInteger(r.suppressedSemanticHintCount) &&
      Number.isInteger(r.suppressedNearNameCount) &&
      r.suppressedSemanticHintCount >= r.suppressedSemanticHints.length &&
      r.suppressedNearNameCount >= r.suppressedNearNames.length,
    JSON.stringify({
      suppressedSemanticHintCount: r.suppressedSemanticHintCount,
      suppressedNearNameCount: r.suppressedNearNameCount,
    }));
}

{
  const sym = buildSymbols({
    identitiesByFile: {
      'src/services/user.ts': ['fetchUser'],
      'src/services/post.ts': ['fetchPost'],
    },
    defKindsByIdentity: {
      'src/services/user.ts::fetchUser': 'FunctionDeclaration',
      'src/services/post.ts::fetchPost': 'FunctionDeclaration',
    },
    fanInByIdentity: {},
  });
  const r = lookupName('searchUser', {
    symbols: sym,
    canonicalClaims: [],
    intentDeclaration: {
      name: 'searchUser',
      kind: 'function',
      why: 'search user data',
      ownerFile: 'src/services/user-search.ts',
    },
  });
  const policy = r.serviceOperationSiblingPolicy;
  const promoted = policy?.promoted?.find((h) => h.name === 'fetchUser');
  assert('T25. service-operation sibling policy is emitted with stable identity',
    policy?.policyId === 'prewrite-service-operation-sibling-cue' &&
      policy?.policyVersion === 'prewrite-service-operation-sibling-cue-v1',
    JSON.stringify(policy));
  assert('T25a. read-query sibling is promoted only inside policy evidence',
    promoted &&
      promoted.operationFamily === 'read-query' &&
      promoted.ownerFile === 'src/services/user.ts' &&
      promoted.sharedDomainTokens?.includes('user') &&
      promoted.locality?.sameDir === true &&
      promoted.locality?.sameFile === false,
    JSON.stringify(policy));
  assert('T25b. policy promotion preserves supporting suppressed reasons and signature limits',
    promoted?.supportingReasons?.includes('single-non-weak-token-only') &&
      promoted?.supportingReasons?.includes('near-distance-exceeded') &&
      promoted?.signatureSupport?.status === 'unavailable' &&
      promoted?.signatureSupport?.reason === 'no-signature-facts',
    JSON.stringify(promoted));
  assert('T25c. policy evidence does not relax formal near/semantic thresholds',
    !r.semanticHints.some((h) => h.name === 'fetchUser') &&
      !r.nearNames.some((h) => h.name === 'fetchUser'),
    JSON.stringify({ semanticHints: r.semanticHints, nearNames: r.nearNames }));
}

{
  const sym = buildSymbols({
    identitiesByFile: {
      'src/services/user.ts': ['fetchUser'],
    },
    defKindsByIdentity: {
      'src/services/user.ts::fetchUser': 'FunctionDeclaration',
    },
    fanInByIdentity: {},
  });
  const createResult = lookupName('createUser', {
    symbols: sym,
    canonicalClaims: [],
    intentDeclaration: {
      name: 'createUser',
      kind: 'function',
      why: 'create user data',
      ownerFile: 'src/services/user-create.ts',
    },
  });
  const postResult = lookupName('searchPost', {
    symbols: sym,
    canonicalClaims: [],
    intentDeclaration: {
      name: 'searchPost',
      kind: 'function',
      why: 'search user data while writing the post search flow',
      ownerFile: 'src/services/post-search.ts',
    },
  });
  assert('T26. operation-family mismatch is muted, not promoted',
    !createResult.serviceOperationSiblingPolicy?.promoted?.some((h) => h.name === 'fetchUser') &&
      createResult.serviceOperationSiblingPolicy?.muted?.some((h) =>
        h.name === 'fetchUser' &&
        h.reason === 'service-sibling-operation-family-mismatch'),
    JSON.stringify(createResult.serviceOperationSiblingPolicy));
  assert('T26a. domain mismatch is muted, not promoted',
    !postResult.serviceOperationSiblingPolicy?.promoted?.some((h) => h.name === 'fetchUser') &&
      postResult.serviceOperationSiblingPolicy?.muted?.some((h) =>
        h.name === 'fetchUser' &&
        h.reason === 'service-sibling-domain-mismatch'),
    JSON.stringify(postResult.serviceOperationSiblingPolicy));
}

{
  const r = lookupName('queryLibraryDoc', {
    symbols: buildSymbols({
      identitiesByFile: {
        'apps/server/src/repository.ts': [
          'ListLibraryDocsOptions',
          'listLibraryDocs',
        ],
      },
      defKindsByIdentity: {
        'apps/server/src/repository.ts::ListLibraryDocsOptions': 'TSInterfaceDeclaration',
        'apps/server/src/repository.ts::listLibraryDocs': 'FunctionDeclaration',
      },
    }),
    canonicalClaims: [],
    intentDeclaration: {
      name: 'queryLibraryDoc',
      kind: 'function',
      why: 'query library docs from the repository',
      ownerFile: 'apps/server/src/repository.ts',
    },
  });
  assert('T26b. type-like service candidates are muted, not promoted',
    r.serviceOperationSiblingPolicy?.promoted?.some((h) => h.name === 'listLibraryDocs') &&
      !r.serviceOperationSiblingPolicy?.promoted?.some((h) => h.name === 'ListLibraryDocsOptions') &&
      r.serviceOperationSiblingPolicy?.muted?.some((h) =>
        h.name === 'ListLibraryDocsOptions' &&
        h.reason === 'service-sibling-non-callable-definition' &&
        h.definitionKind === 'TSInterfaceDeclaration'),
    JSON.stringify(r.serviceOperationSiblingPolicy));
}

{
  const sym = buildSymbols({
    identitiesByFile: {
      'src/services/user.ts': ['fetchUser'],
      'src/services/post.ts': ['fetchPost'],
    },
    fanInByIdentity: {},
  });
  const r = lookupName('xyzzy', {
    symbols: sym,
    canonicalClaims: [],
    intentDeclaration: {
      name: 'xyzzy',
      kind: 'function',
      why: 'unrelated marker',
      ownerFile: 'src/services/xyzzy.ts',
    },
  });
  assert('T27. unrelated intents do not collect suppressed candidate noise',
    Array.isArray(r.suppressedNearNames) &&
      r.suppressedNearNames.length === 0 &&
      Array.isArray(r.suppressedSemanticHints) &&
      r.suppressedSemanticHints.length === 0,
    JSON.stringify({
      suppressedNearNames: r.suppressedNearNames,
      suppressedSemanticHints: r.suppressedSemanticHints,
    }));
  assert('T27a. unrelated intents do not collect service-operation policy noise',
    r.serviceOperationSiblingPolicy?.evaluatedCandidateCount === 0 &&
      r.serviceOperationSiblingPolicy?.promotedCandidateCount === 0 &&
      r.serviceOperationSiblingPolicy?.mutedCandidateCount === 0,
    JSON.stringify(r.serviceOperationSiblingPolicy));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

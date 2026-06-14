// Tests for _lib/pre-write-drift.mjs — P1-3 step 5.1.
//
// Pinning rules from docs/history/phases/p1/p1-3.md §4.1 + §5.1:
//   - Read-only projection: does NOT re-parse canonical or re-run lookup.
//     Structural check (T-final) greps module source for forbidden imports.
//   - No drift entry for aligned / not-consulted states.
//   - One drift entry per disagreement (owner-disagrees OR ast-absent).
//   - Canonical declares X, AST has X+Y (canonical aligned with X) → NO drift.
//     The extra Y is canon-draft staleness, not drift — separate concern.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { computeDrift } from '../_lib/pre-write-drift.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

// ── Fixture helpers ─────────────────────────────────────────

function canonicalClaim({ name, ownerFile, line = 42, file = 'canonical/type-ownership.md', section = 'Single owner (strong)' } = {}) {
  return { name, ownerFile, line, file, section };
}

function nameLookup({ intentName, canonicalAstStatus, identities = [], canonicalClaim = null }) {
  return {
    kind: 'name',
    intentName,
    result: 'EXISTS',
    identities,
    canonicalClaim,
    canonicalAstStatus,
    nearNames: [],
    citations: [],
  };
}

function identity(ownerFile, exportedName) {
  return {
    identity: `${ownerFile}::${exportedName}`,
    ownerFile,
    exportedName,
    fanIn: 1,
    fanInConfidence: 'grounded',
    anyContamination: { state: 'clean' },
    resolverConfidence: 'high',
    citations: [],
  };
}

// ═══ T1. Empty canonical → empty drift ═══

{
  const drift = computeDrift({ canonicalClaims: [], lookups: [] });
  assert('T1. empty inputs → empty drift', drift.length === 0);
}

// ═══ T2. All aligned → empty drift ═══

{
  const claims = [canonicalClaim({ name: 'SessionId', ownerFile: 'src/protocol/ids.ts' })];
  const lookups = [nameLookup({
    intentName: 'SessionId',
    canonicalAstStatus: 'aligned',
    identities: [identity('src/protocol/ids.ts', 'SessionId')],
    canonicalClaim: claims[0],
  })];
  const drift = computeDrift({ canonicalClaims: claims, lookups });
  assert('T2. aligned → empty drift', drift.length === 0);
}

// ═══ T3. owner-disagrees → one drift entry ═══

{
  const claims = [canonicalClaim({ name: 'User', ownerFile: 'src/models/User.ts', line: 42 })];
  const lookups = [nameLookup({
    intentName: 'User',
    canonicalAstStatus: 'owner-disagrees',
    identities: [identity('apps/legacy/user.ts', 'User')],
    canonicalClaim: claims[0],
  })];
  const drift = computeDrift({ canonicalClaims: claims, lookups });
  assert('T3. owner-disagrees → 1 drift entry', drift.length === 1);
  assert('T3b. drift.kind === owner-disagrees', drift[0].kind === 'owner-disagrees');
  assert('T3c. drift.canonicalOwner matches claim', drift[0].canonicalOwner === 'src/models/User.ts');
  assert('T3d. drift.canonicalLine preserved', drift[0].canonicalLine === 42);
  assert('T3e. drift.astOwners contains the disagreeing owner',
    drift[0].astOwners.length === 1 && drift[0].astOwners[0] === 'apps/legacy/user.ts');
  assert('T3f. drift.intentName === "User"', drift[0].intentName === 'User');
  assert('T3g. drift.canonicalFile preserved',
    drift[0].canonicalFile === 'canonical/type-ownership.md');
}

// ═══ T4. ast-absent → one drift entry ═══

{
  const claims = [canonicalClaim({ name: 'GoneType', ownerFile: 'src/types/gone.ts', line: 7 })];
  const lookups = [nameLookup({
    intentName: 'GoneType',
    canonicalAstStatus: 'ast-absent',
    identities: [],
    canonicalClaim: claims[0],
  })];
  const drift = computeDrift({ canonicalClaims: claims, lookups });
  assert('T4. ast-absent → 1 drift entry', drift.length === 1);
  assert('T4b. drift.kind === ast-absent', drift[0].kind === 'ast-absent');
  assert('T4c. drift.astOwners is an empty array',
    Array.isArray(drift[0].astOwners) && drift[0].astOwners.length === 0);
  assert('T4d. drift.canonicalOwner preserved',
    drift[0].canonicalOwner === 'src/types/gone.ts');
}

// ═══ T5. not-consulted → empty drift ═══

{
  const lookups = [nameLookup({
    intentName: 'anything',
    canonicalAstStatus: 'not-consulted',
    identities: [identity('src/x.ts', 'anything')],
    canonicalClaim: null,
  })];
  const drift = computeDrift({ canonicalClaims: [], lookups });
  assert('T5. not-consulted → empty drift', drift.length === 0);
}

// ═══ T6. Canonical aligned with X, AST has X+Y → empty drift ═══
//
// Per §5.1 pinning from p1-3.md: when canonical declares X and AST has
// X AND Y, drift is EMPTY because canonical IS aligned with X. The extra
// Y is canon-draft staleness (canon didn't list Y), NOT owner-disagreement.

{
  const claims = [canonicalClaim({ name: 'User', ownerFile: 'src/models/User.ts' })];
  const lookups = [nameLookup({
    intentName: 'User',
    canonicalAstStatus: 'aligned',  // canonical IS aligned with one of the identities
    identities: [
      identity('src/models/User.ts', 'User'),    // canonical owner
      identity('apps/legacy/user.ts', 'User'),   // extra — canon-draft staleness
    ],
    canonicalClaim: claims[0],
  })];
  const drift = computeDrift({ canonicalClaims: claims, lookups });
  assert('T6. canonical aligned + extra AST identity → empty drift',
    drift.length === 0,
    `drift=${JSON.stringify(drift)}`);
}

// ═══ T7. Multiple intent names, mix of states ═══

{
  const claims = [
    canonicalClaim({ name: 'Aligned', ownerFile: 'src/aligned.ts' }),
    canonicalClaim({ name: 'Disagree', ownerFile: 'src/disagree.ts' }),
    canonicalClaim({ name: 'Absent', ownerFile: 'src/absent.ts' }),
  ];
  const lookups = [
    nameLookup({
      intentName: 'Aligned',
      canonicalAstStatus: 'aligned',
      identities: [identity('src/aligned.ts', 'Aligned')],
      canonicalClaim: claims[0],
    }),
    nameLookup({
      intentName: 'Disagree',
      canonicalAstStatus: 'owner-disagrees',
      identities: [identity('src/other.ts', 'Disagree')],
      canonicalClaim: claims[1],
    }),
    nameLookup({
      intentName: 'Absent',
      canonicalAstStatus: 'ast-absent',
      identities: [],
      canonicalClaim: claims[2],
    }),
  ];
  const drift = computeDrift({ canonicalClaims: claims, lookups });
  assert('T7. mixed states → exactly 2 drift entries (disagree + absent)',
    drift.length === 2,
    `drift=${JSON.stringify(drift)}`);
  const kinds = drift.map((d) => d.kind).sort();
  assert('T7b. drift kinds are ast-absent + owner-disagrees',
    kinds[0] === 'ast-absent' && kinds[1] === 'owner-disagrees');
  assert('T7c. "Aligned" is NOT in the drift list',
    !drift.some((d) => d.intentName === 'Aligned'));
}

// ═══ T8. No duplicate drift per intent name ═══

{
  const claims = [canonicalClaim({ name: 'X', ownerFile: 'src/x.ts' })];
  const lookups = [
    nameLookup({
      intentName: 'X',
      canonicalAstStatus: 'owner-disagrees',
      identities: [identity('apps/x1.ts', 'X'), identity('apps/x2.ts', 'X')],
      canonicalClaim: claims[0],
    }),
  ];
  const drift = computeDrift({ canonicalClaims: claims, lookups });
  assert('T8. one name with multiple disagreeing owners → 1 drift entry',
    drift.length === 1);
  assert('T8b. drift.astOwners lists both disagreeing owners',
    drift[0].astOwners.length === 2);
}

// ═══ T9. Lookups without matching canonical claim → ignored ═══

{
  const lookups = [
    nameLookup({
      intentName: 'NoCanon',
      canonicalAstStatus: 'not-consulted',
      identities: [identity('src/x.ts', 'NoCanon')],
      canonicalClaim: null,
    }),
  ];
  const drift = computeDrift({ canonicalClaims: [], lookups });
  assert('T9. lookup without canonical claim → no drift',
    drift.length === 0);
}

// ═══ T10. Non-name lookups (file / dep / shape) are ignored ═══

{
  const claims = [canonicalClaim({ name: 'X', ownerFile: 'src/x.ts' })];
  const lookups = [
    { kind: 'file', intentFile: 'src/y.ts', result: 'NEW_FILE' },
    { kind: 'dependency', depName: 'dayjs', result: 'NEW_PACKAGE' },
    { kind: 'shape', shape: { fields: ['a'] }, result: 'UNAVAILABLE' },
  ];
  const drift = computeDrift({ canonicalClaims: claims, lookups });
  assert('T10. non-name lookups do not produce drift',
    drift.length === 0);
}

// ═══ T-final. Structural pinning — read-only module ═══
//
// docs/history/phases/p1/p1-3.md §9: drift module is read-only. It MUST NOT import
// parseCanonicalFile or lookupName. Grep the source to enforce.

{
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const src = readFileSync(path.join(__dirname, '..', '_lib', 'pre-write-drift.mjs'), 'utf8');
  assert('T-final-a. module does NOT import parseCanonicalFile',
    !/from\s+['"][^'"]*pre-write-canonical-parser/.test(src),
    'module imports the canonical parser — re-parsing forbidden');
  assert('T-final-b. module does NOT import lookupName',
    !/from\s+['"][^'"]*pre-write-lookup-name/.test(src),
    'module imports name lookup — re-running forbidden');
  assert('T-final-c. module does NOT read filesystem',
    !/\breadFileSync\b|\bexistsSync\b|\breadSync\b/.test(src),
    'module reads filesystem — should be a pure projection');
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

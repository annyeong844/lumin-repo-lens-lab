// Tests for _lib/pre-write-lookup-shape.mjs — P1-2 step 5.3.
//
// Pinning rules from docs/history/phases/p1/p1-2.md §4.3 + §5.3:
//   - Missing shape-index or legacy fields-only intent returns UNAVAILABLE.
//   - Exact hash lookup consults validated shape-index.json facts[].
//   - No heuristic field-overlap fallback path exists: no defIndex scan,
//     no symbols.uses iteration, no grep-like matching.
//   - Citation substring contains both "shape-hash" AND "P4" so the
//     pin survives minor wording edits.

import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { lookupShape } from '../_lib/pre-write-lookup-shape.mjs';
import { extractShapeHashFactsFromSource } from '../_lib/shape-hash.mjs';
import { functionSignatureFromTypeLiteral } from '../_lib/function-signature-hash.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else    { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

const HASH_A = `sha256:${'a'.repeat(64)}`;
const HASH_B = `sha256:${'b'.repeat(64)}`;
const TYPE_LITERAL = '{ year: number }';
const TYPE_LITERAL_HASH = extractShapeHashFactsFromSource(
  `export type __IntentShape = ${TYPE_LITERAL};\n`,
  '__intent_shape.ts'
).facts[0].hash;
const UNION_LITERAL = '"open" | "closed"';
const UNION_LITERAL_HASH = extractShapeHashFactsFromSource(
  `export type __IntentShape = ${UNION_LITERAL};\n`,
  '__intent_shape.ts'
).facts[0].hash;
const FUNCTION_TYPE_LITERAL = '(raw: string) => string';
const FUNCTION_SIGNATURE = functionSignatureFromTypeLiteral(FUNCTION_TYPE_LITERAL);
const FUNCTION_SIGNATURE_HASH = FUNCTION_SIGNATURE.hash;

function shapeIndex({ complete = true } = {}) {
  return {
    schemaVersion: 'shape-index.v1',
    meta: { complete },
    groupsByHash: {
      [HASH_A]: ['src/a.ts::CalendarA', 'src/b.ts::CalendarB'],
      [TYPE_LITERAL_HASH]: ['src/c.ts::CalendarC'],
      [UNION_LITERAL_HASH]: ['src/status.ts::Status'],
    },
    facts: [
      {
        kind: 'shape-hash',
        hash: HASH_A,
        identities: ['src/a.ts::CalendarA'],
        identity: 'src/a.ts::CalendarA',
        ownerFile: 'src/a.ts',
        exportedName: 'CalendarA',
        fields: [{ name: 'year', type: 'number' }],
        confidence: 'high',
      },
      {
        kind: 'shape-hash',
        hash: HASH_A,
        identities: ['src/b.ts::CalendarB'],
        identity: 'src/b.ts::CalendarB',
        ownerFile: 'src/b.ts',
        exportedName: 'CalendarB',
        fields: [{ name: 'year', type: 'number' }],
        confidence: 'high',
      },
      {
        kind: 'shape-hash',
        hash: TYPE_LITERAL_HASH,
        identities: ['src/c.ts::CalendarC'],
        identity: 'src/c.ts::CalendarC',
        ownerFile: 'src/c.ts',
        exportedName: 'CalendarC',
        fields: [{ name: 'year', type: 'number' }],
        confidence: 'high',
      },
      {
        kind: 'shape-hash',
        hash: UNION_LITERAL_HASH,
        identities: ['src/status.ts::Status'],
        identity: 'src/status.ts::Status',
        ownerFile: 'src/status.ts',
        exportedName: 'Status',
        shapeKind: 'literal-union',
        fields: [],
        literals: [
          { kind: 'string', value: 'closed' },
          { kind: 'string', value: 'open' },
        ],
        confidence: 'high',
      },
    ],
  };
}

function functionCloneIndex({ complete = true } = {}) {
  return {
    schemaVersion: 'function-clones.v3',
    meta: { complete },
    facts: [
      {
        kind: 'function-body-fingerprint',
        identity: 'src/user-a.ts::normalizeUserName',
        ownerFile: 'src/user-a.ts',
        exportedName: 'normalizeUserName',
        localName: 'normalizeUserName',
        visibility: 'file-local',
        exported: false,
        normalizedSignatureHash: FUNCTION_SIGNATURE_HASH,
        signature: FUNCTION_SIGNATURE.signature,
        confidence: 'high',
      },
    ],
  };
}

// ═══ Missing shape-index with exact hash → UNAVAILABLE ═══

{
  const r = lookupShape({ fields: [], hash: HASH_A }, {});
  assert('T1. exact shape with missing shape-index → result:UNAVAILABLE',
    r.result === 'UNAVAILABLE');
  assert('T1b. kind:"shape"', r.kind === 'shape');
  assert('T1c. shape preserved on result',
    JSON.stringify(r.shape) === JSON.stringify({ fields: [], hash: HASH_A }));
  assert('T1d. missing-index citation tells caller how to enable P4 lookup',
    /build-shape-index\.mjs/.test((r.citations ?? []).join(' ')),
    JSON.stringify(r.citations));
}

// ═══ Legacy fields-only shape → UNAVAILABLE even when index exists ═══

{
  const r = lookupShape({ fields: ['year'] }, { shapeIndex: shapeIndex() });
  assert('T2. fields-only shape cannot claim structural equality',
    r.result === 'UNAVAILABLE');
  assert('T2b. citation says field names alone are not structural evidence',
    /field names alone/.test((r.citations ?? []).join(' ')),
    JSON.stringify(r.citations));
}

// ═══ Citation carries shape-hash + P4 substring ═══

{
  const r = lookupShape({ fields: ['x'] }, {});
  const citation = (r.citations ?? []).join(' ');
  assert('T3. citation mentions "shape-hash"',
    /shape-hash/i.test(citation), `citations=${JSON.stringify(r.citations)}`);
  assert('T3b. citation mentions "P4"',
    /P4/.test(citation));
  assert('T3c. citation marker is [확인 불가]',
    /\[확인 불가/.test(citation));
}

// ═══ Exact hash hit → SHAPE_MATCH ═══

{
  const r = lookupShape({ fields: [], hash: HASH_A }, { shapeIndex: shapeIndex() });
  assert('T4. exact hash hit → SHAPE_MATCH',
    r.result === 'SHAPE_MATCH',
    JSON.stringify(r));
  assert('T4b. all grouped identities returned sorted',
    JSON.stringify(r.matches.map((m) => m.identity)) ===
    JSON.stringify(['src/a.ts::CalendarA', 'src/b.ts::CalendarB']),
    JSON.stringify(r.matches));
  assert('T4c. grounded citation references shape-index facts[]',
    /shape-index\.json facts\[\]/.test((r.citations ?? []).join(' ')),
    JSON.stringify(r.citations));
}

// ═══ Intent typeLiteral is normalized to an exact hash ═══

{
  const r = lookupShape({ fields: [], typeLiteral: TYPE_LITERAL }, { shapeIndex: shapeIndex() });
  assert('T4d. typeLiteral hit → SHAPE_MATCH',
    r.result === 'SHAPE_MATCH',
    JSON.stringify(r));
  assert('T4e. typeLiteral-derived shapeHash is carried on the lookup',
    r.shapeHash === TYPE_LITERAL_HASH && r.shapeHashSource === 'typeLiteral',
    JSON.stringify(r));
  assert('T4f. typeLiteral citation documents normalizer use',
    /typeLiteral normalized/.test((r.citations ?? []).join(' ')),
    JSON.stringify(r.citations));
}

{
  const r = lookupShape({ fields: [], typeLiteral: UNION_LITERAL }, { shapeIndex: shapeIndex() });
  assert('T4f2. union literal typeLiteral hit → SHAPE_MATCH',
    r.result === 'SHAPE_MATCH' &&
    r.matches[0]?.shapeKind === 'literal-union' &&
    r.matches[0]?.literals?.length === 2,
    JSON.stringify(r));
}

{
  const r = lookupShape({
    fields: [],
    hash: HASH_A,
    typeLiteral: TYPE_LITERAL,
  }, { shapeIndex: shapeIndex() });
  assert('T4g. mismatched explicit hash + typeLiteral → UNAVAILABLE',
    r.result === 'UNAVAILABLE',
    JSON.stringify(r));
  assert('T4h. mismatch citation names both hashes',
    /does not match/.test((r.citations ?? []).join(' ')),
    JSON.stringify(r.citations));
}

{
  const r = lookupShape({
    fields: [],
    typeLiteral: '{ [K in keyof T]: T[K] }',
  }, { shapeIndex: shapeIndex() });
  assert('T4i. unsupported typeLiteral → UNAVAILABLE',
    r.result === 'UNAVAILABLE',
    JSON.stringify(r));
}

// ═══ Exact hash miss → NOT_OBSERVED only when index complete ═══

{
  const r = lookupShape({ fields: [], hash: HASH_B }, { shapeIndex: shapeIndex() });
  assert('T5. exact hash miss with complete index → NOT_OBSERVED',
    r.result === 'NOT_OBSERVED',
    JSON.stringify(r));
}

{
  const r = lookupShape({ fields: [], hash: HASH_B }, { shapeIndex: shapeIndex({ complete: false }) });
  assert('T5b. exact hash miss with incomplete index → UNAVAILABLE',
    r.result === 'UNAVAILABLE',
    JSON.stringify(r));
}

// ═══ Function signature lookup preserves file-local helper visibility ═══

{
  const r = lookupShape(
    { fields: [], typeLiteral: FUNCTION_TYPE_LITERAL },
    { functionClones: functionCloneIndex() }
  );
  assert('T5c. function signature can match file-local helper facts',
    r.result === 'SIGNATURE_MATCH' &&
      r.matches?.[0]?.identity === 'src/user-a.ts::normalizeUserName',
    JSON.stringify(r, null, 2));
  assert('T5d. function signature match preserves file-local visibility',
    r.matches?.[0]?.visibility === 'file-local' &&
      r.matches?.[0]?.exported === false &&
      r.matches?.[0]?.localName === 'normalizeUserName',
    JSON.stringify(r.matches, null, 2));
}

// ═══ Malformed index / invalid hash → UNAVAILABLE ═══

{
  const r = lookupShape({ fields: [], hash: HASH_A }, { shapeIndex: { schemaVersion: 'wrong' } });
  assert('T6. malformed shape-index → UNAVAILABLE',
    r.result === 'UNAVAILABLE');
}

{
  const r = lookupShape({ fields: [], hash: 'abc' }, { shapeIndex: shapeIndex() });
  assert('T6b. invalid shape hash → UNAVAILABLE',
    r.result === 'UNAVAILABLE');
}

{
  const r = lookupShape({ fields: [], hash: HASH_A }, {
    shapeIndex: {
      schemaVersion: 'shape-index.v1',
      meta: { complete: true },
      groupsByHash: { [HASH_A]: ['src/fake.ts::Ghost'] },
      facts: [],
    },
  });
  assert('T6c. groupsByHash ghost identity without fact → UNAVAILABLE',
    r.result === 'UNAVAILABLE',
    JSON.stringify(r));
  assert('T6d. ghost mismatch is reported as malformed/inconsistent artifact',
    /group-mismatch|groupsByHash/.test((r.citations ?? []).join(' ')),
    JSON.stringify(r.citations));
}

// ═══ No heuristic path — structural check ═══
//
// Read the module source and assert it does NOT contain suspicious
// keywords that would indicate a heuristic fallback. This catches any
// future drift toward "fresh grep over interface fields" even when the
// tests above pass.

{
  const __dirname = path.dirname(fileURLToPath(import.meta.url));
  const src = readFileSync(path.join(__dirname, '..', '_lib', 'pre-write-lookup-shape.mjs'), 'utf8');

  assert('T7. module does NOT reference defIndex (no structural iteration)',
    !/\bdefIndex\b/.test(src), `found defIndex in source`);
  assert('T7b. module does NOT iterate over symbols.uses',
    !/\bsymbols\.uses\b/.test(src));
  assert('T7c. module does NOT match on "interface" string literals',
    !/['"]interface['"]/.test(src));
}

// ═══ Purity — same shape in, same result out ═══

{
  const a = lookupShape({ fields: [], hash: HASH_A }, { shapeIndex: shapeIndex() });
  const b = lookupShape({ fields: [], hash: HASH_A }, { shapeIndex: shapeIndex() });
  assert('T8. deterministic return shape',
    JSON.stringify(a) === JSON.stringify(b));
}

// ═══ Even when symbols has rich fixture data, result is STILL UNAVAILABLE ═══

{
  const symbols = {
    defIndex: {
      'src/a.ts': {
        CalendarShape: { kind: 'TSInterfaceDeclaration', line: 1 },
      },
    },
  };
  const r = lookupShape({ fields: ['year', 'month', 'day', 'hour'] }, { symbols });
  assert('T9. rich defIndex does not seduce the lookup into matching',
    r.result === 'UNAVAILABLE');
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

# Pre-Write Cue Tiers Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add evidence-item cue tiers to pre-write so AI coding agents can distinguish grounded facts, review tasks, muted noise, and unavailable evidence.

**Architecture:** Keep existing lookup modules as the source of raw observations. Add a small cue-tier layer that converts lookup results into `cueCards[]`, `suppressedCues[]`, and `unavailableEvidence[]`, then render those fields in JSON and Markdown without changing CLI exit behavior. Token suppression stays deterministic and intentionally small.

**Tech Stack:** Node.js ESM, existing pre-write CLI, existing test harness with direct `node tests/*.mjs` scripts.

---

## File Structure

- Create `_lib/pre-write-token-policy.mjs`
  - Owns deterministic pre-write tokenization, weak-token policy constants, and policy metadata.
  - No artifact I/O.

- Create `_lib/pre-write-cue-tiers.mjs`
  - Pure adapter from existing lookup results to cue-tier artifacts.
  - Produces `cueCards`, `suppressedCues`, `unavailableEvidence`, and `cuePolicy`.
  - Does not run producers or mutate lookup results.

- Modify `_lib/pre-write-lookup-name.mjs`
  - Reuse `_lib/pre-write-token-policy.mjs`.
  - Preserve existing `semanticHints`.
  - Add `suppressedSemanticHints` for weak-token-only and insufficient-support token candidates.
  - Defer high-fanout token suppression until thresholds are designed and tested.

- Modify `pre-write.mjs`
  - After all lookups are produced, call `classifyPreWriteCues({ lookups, intent })`.
  - Add returned cue fields to the advisory object.

- Modify `_lib/pre-write-render.mjs`
  - Preserve existing JSON fields.
  - Add JSON defaults for `cueCards`, `suppressedCues`, `unavailableEvidence`, and `cuePolicy`.
  - Add Markdown sections for `Grounded facts`, `Agent review cues`, and optionally `Unavailable evidence`.
  - Do not change exit behavior.

- Modify `README.md`
  - Replace semantic duplicate wording with grounded reuse cues and agent review tasks.

- Generated mirror through `npm run build:skill`
  - Mirrors `_lib/` and command surface into `skills/lumin-repo-lens-lab/`.

---

### Task 1: RED Tests For Cue Tier Artifact Contract

**Files:**
- Create: `tests/test-pre-write-cue-tiers.mjs`
- Modify: `scripts/update-test-doc.mjs` and generated `tests/README.md` in the final docs/mirror task.

- [ ] **Step 1: Add the direct cue-tier test file**

Create `tests/test-pre-write-cue-tiers.mjs` with this content:

```js
import {
  classifyPreWriteCues,
  CUE_TIERS,
  UNAVAILABLE_STATUS,
} from '../_lib/pre-write-cue-tiers.mjs';
import { tokenizePreWrite } from '../_lib/pre-write-token-policy.mjs';

let passed = 0, failed = 0;
function assert(label, ok, detail = '') {
  if (ok) { passed++; console.log(`  PASS  ${label}`); }
  else { failed++; console.log(`  FAIL  ${label}\n        ${detail}`); }
}

function findCard(result, identity) {
  return result.cueCards.find((card) => card.candidate?.identity === identity);
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'formatDate',
      result: 'EXISTS',
      identities: [{
        identity: 'src/date.ts::formatDate',
        ownerFile: 'src/date.ts',
        exportedName: 'formatDate',
        fanIn: 3,
        fanInConfidence: 'grounded',
        citations: [`[grounded, symbols.json.fanInByIdentity['src/date.ts::formatDate'] = 3]`],
      }],
      nearNames: [],
      semanticHints: [],
      suppressedSemanticHints: [],
    }],
    intent: { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/date.ts::formatDate');
  assert('T1. exact symbol identity creates a cue card',
    !!card, JSON.stringify(result, null, 2));
  assert('T1b. exact symbol cue is SAFE_CUE claim-only',
    card.cues.some((cue) =>
      cue.cueTier === CUE_TIERS.SAFE &&
      cue.safeMeaning === 'claim-only' &&
      cue.evidenceLane === 'exact-symbol' &&
      cue.notSafeFor.includes('semantic-equivalence')),
    JSON.stringify(card, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'shape',
      result: 'SIGNATURE_MATCH',
      shapeHash: 'sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
      shapeHashSource: 'functionSignature',
      signature: '<S,U>((S)=>U):(S)=>U',
      matches: [{
        identity: 'src/shallow.ts::useShallow',
        ownerFile: 'src/shallow.ts',
        exportedName: 'useShallow',
        confidence: 'medium',
      }],
      citations: ['[grounded, function-clones.json facts[] matched 1 identities]'],
    }],
    intent: { names: ['composeProjection'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/shallow.ts::useShallow');
  assert('T2. function signature match creates cue card',
    !!card, JSON.stringify(result, null, 2));
  assert('T2b. function signature cue is SAFE_CUE',
    card.cues.some((cue) =>
      cue.cueTier === CUE_TIERS.SAFE &&
      cue.evidenceLane === 'function-signature' &&
      cue.claim === 'same normalized function signature'),
    JSON.stringify(card, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'useShallowFromState',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [{ name: 'useShallow', ownerFile: 'src/shallow.ts', distance: 2 }],
      semanticHints: [],
      suppressedSemanticHints: [],
      citations: ['[degraded, fuzzy-name match; search hint only]'],
    }, {
      kind: 'shape',
      result: 'SIGNATURE_MATCH',
      shapeHash: 'sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb',
      shapeHashSource: 'functionSignature',
      signature: '<S,U>((S)=>U):(S)=>U',
      matches: [{
        identity: 'src/shallow.ts::useShallow',
        ownerFile: 'src/shallow.ts',
        exportedName: 'useShallow',
      }],
      citations: ['[grounded, function-clones.json facts[] matched 1 identities]'],
    }],
    intent: { names: ['useShallowFromState'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/shallow.ts::useShallow');
  const tiers = new Set(card?.cues?.map((cue) => cue.cueTier));
  assert('T3. one candidate can carry SAFE_CUE and AGENT_REVIEW_CUE together',
    tiers.has(CUE_TIERS.SAFE) && tiers.has(CUE_TIERS.AGENT_REVIEW),
    JSON.stringify(card, null, 2));
  assert('T3b. mixed candidate renderTier is AGENT_REVIEW_CUE',
    card?.renderTier === CUE_TIERS.AGENT_REVIEW,
    JSON.stringify(card, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'createLogger',
      result: 'NOT_OBSERVED',
      identities: [],
      nearNames: [],
      semanticHints: [],
      suppressedSemanticHints: [{
        name: 'createStore',
        ownerFile: 'src/store.ts',
        matchedTokens: ['create'],
        reason: 'weak-common-token-only',
        candidateCount: 2,
      }, {
        name: 'createJSONStorage',
        ownerFile: 'src/storage.ts',
        matchedTokens: ['create'],
        reason: 'weak-common-token-only',
        candidateCount: 2,
      }],
    }],
    intent: { names: ['createLogger'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T4. weak common token candidates become suppressed cues',
    result.suppressedCues.length === 2 &&
    result.suppressedCues.every((cue) =>
      cue.cueTier === CUE_TIERS.MUTED &&
      cue.reason === 'weak-common-token-only' &&
      cue.tokenPolicyVersion === 'prewrite-token-policy-v1'),
    JSON.stringify(result.suppressedCues, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'shape',
      shape: { fields: [], typeLiteral: '<S>(selector: (state: S) => S) => S' },
      result: 'UNAVAILABLE',
      reason: 'missing-artifact',
      artifact: 'function-clones.json',
      citations: ['[확인 불가, function-clones.json absent]'],
    }],
    intent: { names: [], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T5. unavailable evidence is separate from suppressed cues',
    result.unavailableEvidence.length === 1 &&
    result.unavailableEvidence[0].status === UNAVAILABLE_STATUS &&
    result.suppressedCues.length === 0,
    JSON.stringify(result, null, 2));
  assert('T5b. unavailable evidence preserves lookup reason and artifact',
    result.unavailableEvidence[0].reason === 'missing-artifact' &&
    result.unavailableEvidence[0].artifact === 'function-clones.json',
    JSON.stringify(result.unavailableEvidence, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'name',
      intentName: 'generatedHelper',
      result: 'EXISTS',
      identities: [{
        identity: 'dist/generated.ts::generatedHelper',
        ownerFile: 'dist/generated.ts',
        exportedName: 'generatedHelper',
        policyExcluded: true,
        policyReason: 'generated-output',
        citations: ['[grounded, exact symbol exists]'],
      }],
      nearNames: [],
      semanticHints: [],
      suppressedSemanticHints: [],
    }],
    intent: { names: ['generatedHelper'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
  });
  assert('T6. policy-excluded exact evidence stays out of cueCards',
    result.cueCards.length === 0,
    JSON.stringify(result, null, 2));
  assert('T6b. policy-excluded exact evidence remains in suppressed cues',
    result.suppressedCues.some((cue) =>
      cue.reason === 'policy-excluded' &&
      cue.policyReason === 'generated-output' &&
      cue.originalCueTier === CUE_TIERS.SAFE &&
      cue.claim === 'exact exported symbol exists' &&
      cue.evidence?.[0]?.artifact === 'symbols.json'),
    JSON.stringify(result.suppressedCues, null, 2));
}

{
  const result = classifyPreWriteCues({
    lookups: [{
      kind: 'file',
      intentFile: 'src/logger.ts',
      result: 'FILE_EXISTS',
    }],
    intent: { names: [], shapes: [], files: ['src/logger.ts'], dependencies: [], plannedTypeEscapes: [] },
  });
  const card = findCard(result, 'src/logger.ts::__file__');
  assert('T7. exact file exists creates SAFE_CUE',
    card?.cues?.some((cue) =>
      cue.cueTier === CUE_TIERS.SAFE &&
      cue.evidenceLane === 'exact-file' &&
      cue.claim === 'exact file exists'),
    JSON.stringify(result, null, 2));
}

{
  assert('T8. token policy preserves class/process/status stems',
    tokenizePreWrite('className').includes('class') &&
    tokenizePreWrite('processConfig').includes('process') &&
    tokenizePreWrite('statusCheck').includes('status') &&
    tokenizePreWrite('analysisReport').includes('analysis'),
    JSON.stringify({
      className: tokenizePreWrite('className'),
      processConfig: tokenizePreWrite('processConfig'),
      statusCheck: tokenizePreWrite('statusCheck'),
      analysisReport: tokenizePreWrite('analysisReport'),
    }, null, 2));
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
```

- [ ] **Step 2: Run the direct cue-tier test and verify RED**

Run:

```bash
node tests/test-pre-write-cue-tiers.mjs
```

Expected: FAIL with a module-not-found error for `_lib/pre-write-cue-tiers.mjs`.

- [ ] **Step 3: Commit the RED test**

Run:

```bash
git add tests/test-pre-write-cue-tiers.mjs
git commit -m "test: add pre-write cue tier contract"
```

Expected: commit succeeds with only the new test file staged.

---

### Task 2: Token Policy And Cue Tier Module

**Files:**
- Create: `_lib/pre-write-token-policy.mjs`
- Create: `_lib/pre-write-cue-tiers.mjs`
- Test: `tests/test-pre-write-cue-tiers.mjs`

- [ ] **Step 1: Create the deterministic token policy module**

Create `_lib/pre-write-token-policy.mjs`:

```js
export const TOKENIZER_VERSION = 'camel-snake-kebab-digit-v1';
export const TOKEN_POLICY_VERSION = 'prewrite-token-policy-v1';

export const WEAK_COMMON_TOKENS = Object.freeze([
  'add',
  'build',
  'check',
  'create',
  'delete',
  'get',
  'load',
  'make',
  'parse',
  'read',
  'return',
  'save',
  'set',
  'update',
  'write',
]);

const WEAK_COMMON_TOKEN_SET = new Set(WEAK_COMMON_TOKENS);

export function normalizePreWriteToken(token) {
  const t = String(token ?? '').toLowerCase();
  if (t === 'rel') return 'relative';
  if (t === 'ctx') return 'context';
  if (t === 'cfg') return 'config';
  if (t === 'config') return 'configuration';
  if (t === 'exists' || t === 'existing' || t === 'existence') return 'exist';
  if (t.length > 4 && t.endsWith('ies') && !['series', 'species'].includes(t)) return `${t.slice(0, -3)}y`;
  // Avoid broad trailing-s stemming. It corrupts class/process/status/analysis.
  return t;
}

export function tokenizePreWrite(value) {
  return String(value ?? '')
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/([A-Za-z])([0-9])/g, '$1 $2')
    .replace(/([0-9])([A-Za-z])/g, '$1 $2')
    .replace(/[^A-Za-z0-9]+/g, ' ')
    .trim()
    .split(/\s+/)
    .filter(Boolean)
    .map(normalizePreWriteToken);
}

export function uniquePreWriteTokens(...parts) {
  return [...new Set(parts.flatMap(tokenizePreWrite))];
}

export function isWeakCommonToken(token) {
  return WEAK_COMMON_TOKEN_SET.has(String(token ?? '').toLowerCase());
}

export function tokenPolicyMetadata() {
  return {
    tokenizerVersion: TOKENIZER_VERSION,
    tokenPolicyVersion: TOKEN_POLICY_VERSION,
    weakCommonTokens: [...WEAK_COMMON_TOKENS],
  };
}
```

- [ ] **Step 2: Create the cue-tier module**

Create `_lib/pre-write-cue-tiers.mjs`:

Candidate-level `renderTier` is only for default rendering. If any cue on a
candidate is `AGENT_REVIEW_CUE`, the candidate renders as `AGENT_REVIEW_CUE`;
the individual `SAFE_CUE` items remain preserved in `cues[]` with their own
claim-only evidence.

```js
import {
  TOKENIZER_VERSION,
  TOKEN_POLICY_VERSION,
  tokenPolicyMetadata,
} from './pre-write-token-policy.mjs';

export const CUE_TIERS = Object.freeze({
  SAFE: 'SAFE_CUE',
  AGENT_REVIEW: 'AGENT_REVIEW_CUE',
  MUTED: 'MUTED_CUE',
});

export const UNAVAILABLE_STATUS = 'UNAVAILABLE';

const POLICY_EXCLUDED_RE = /(^|\/)(dist|build|coverage|vendor|generated|node_modules)\//;
const TIER_PRIORITY = Object.freeze({
  [CUE_TIERS.SAFE]: 0,
  [CUE_TIERS.AGENT_REVIEW]: 1,
  [CUE_TIERS.MUTED]: 2,
});

function tierRank(tier) {
  return TIER_PRIORITY[tier] ?? 99;
}

function candidateKey(candidate) {
  return candidate?.identity ?? `${candidate?.ownerFile ?? 'unknown'}::${candidate?.exportedName ?? 'unknown'}`;
}

function sortCards(cards) {
  return cards.sort((a, b) =>
    tierRank(a.renderTier) - tierRank(b.renderTier) ||
    String(a.candidate?.ownerFile ?? '').localeCompare(String(b.candidate?.ownerFile ?? '')) ||
    String(a.candidate?.exportedName ?? '').localeCompare(String(b.candidate?.exportedName ?? '')) ||
    String(a.candidate?.identity ?? '').localeCompare(String(b.candidate?.identity ?? ''))
  );
}

function sortSuppressed(cues) {
  return cues.sort((a, b) =>
    String(a.reason ?? '').localeCompare(String(b.reason ?? '')) ||
    String(a.ownerFile ?? '').localeCompare(String(b.ownerFile ?? '')) ||
    String(a.exportedName ?? a.name ?? '').localeCompare(String(b.exportedName ?? b.name ?? ''))
  );
}

function isPolicyExcludedCandidate(candidate) {
  return candidate?.policyExcluded === true ||
    POLICY_EXCLUDED_RE.test(String(candidate?.ownerFile ?? '').replace(/\\/g, '/'));
}

function policyReasonFor(candidate) {
  if (candidate?.policyReason) return candidate.policyReason;
  const file = String(candidate?.ownerFile ?? '').replace(/\\/g, '/');
  const match = file.match(POLICY_EXCLUDED_RE);
  return match ? `path:${match[2]}` : 'policy-excluded';
}

function ensureCard(map, candidate) {
  const key = candidateKey(candidate);
  if (!map.has(key)) {
    map.set(key, {
      candidate: {
        identity: candidate.identity,
        ownerFile: candidate.ownerFile,
        exportedName: candidate.exportedName,
      },
      renderTier: CUE_TIERS.SAFE,
      cues: [],
    });
  }
  return map.get(key);
}

function addCue(cardMap, suppressedCues, candidate, cue) {
  if (isPolicyExcludedCandidate(candidate)) {
    suppressedCues.push({
      ...cue,
      cueTier: CUE_TIERS.MUTED,
      originalCueTier: cue.cueTier,
      reason: 'policy-excluded',
      policyReason: policyReasonFor(candidate),
      ownerFile: candidate.ownerFile,
      exportedName: candidate.exportedName,
      identity: candidate.identity,
    });
    return;
  }
  const card = ensureCard(cardMap, candidate);
  card.cues.push(cue);
  // Any review cue makes the candidate render as a review task, while each
  // grounded cue stays preserved independently in cues[].
  if (cue.cueTier === CUE_TIERS.AGENT_REVIEW) {
    card.renderTier = CUE_TIERS.AGENT_REVIEW;
  }
}

function safeCue({ lane, claim, evidence }) {
  return {
    cueTier: CUE_TIERS.SAFE,
    safeMeaning: 'claim-only',
    notSafeFor: ['semantic-equivalence', 'auto-reuse', 'auto-fix'],
    evidenceLane: lane,
    claim,
    confidence: 'grounded',
    evidence,
  };
}

function reviewCue({ lane, claim, evidence }) {
  return {
    cueTier: CUE_TIERS.AGENT_REVIEW,
    evidenceLane: lane,
    claim,
    confidence: 'heuristic-review',
    evidence,
  };
}

function candidateFromIdentity(identity, fallback = {}) {
  const [ownerFile, exportedName] = String(identity ?? '').split('::');
  return {
    identity,
    ownerFile: fallback.ownerFile ?? ownerFile,
    exportedName: fallback.exportedName ?? exportedName,
    policyExcluded: fallback.policyExcluded,
    policyReason: fallback.policyReason,
  };
}

function addNameLookup({ lookup, cardMap, suppressedCues }) {
  for (const identity of lookup.identities ?? []) {
    addCue(cardMap, suppressedCues, identity, safeCue({
      lane: 'exact-symbol',
      claim: 'exact exported symbol exists',
      evidence: [{
        artifact: 'symbols.json',
        matchedField: 'defIndex',
        candidateIdentity: identity.identity,
        algorithmVersion: 'exact-symbol.v1',
      }],
    }));
  }

  for (const near of lookup.nearNames ?? []) {
    const identity = `${near.ownerFile}::${near.name}`;
    addCue(cardMap, suppressedCues, candidateFromIdentity(identity, near), reviewCue({
      lane: 'near-name',
      claim: 'near exported name',
      evidence: [{
        artifact: 'symbols.json',
        matchedField: 'defIndex',
        algorithmVersion: 'near-name.v1',
        distance: near.distance,
      }],
    }));
  }

  for (const hint of lookup.semanticHints ?? []) {
    const identity = `${hint.ownerFile}::${hint.name}`;
    addCue(cardMap, suppressedCues, candidateFromIdentity(identity, hint), reviewCue({
      lane: 'intent-token',
      claim: 'supported intent-token overlap',
      evidence: [{
        artifact: 'symbols.json',
        matchedField: 'defIndex',
        algorithmVersion: TOKEN_POLICY_VERSION,
        tokens: hint.matchedTokens ?? [],
      }],
    }));
  }

  for (const hint of lookup.suppressedSemanticHints ?? []) {
    suppressedCues.push({
      cueTier: CUE_TIERS.MUTED,
      evidenceLane: 'intent-token',
      reason: hint.reason ?? 'weak-common-token-only',
      tokens: hint.matchedTokens ?? [],
      candidateCount: hint.candidateCount ?? 1,
      tokenizerVersion: TOKENIZER_VERSION,
      tokenPolicyVersion: TOKEN_POLICY_VERSION,
      ownerFile: hint.ownerFile,
      exportedName: hint.name,
      identity: `${hint.ownerFile}::${hint.name}`,
    });
  }
}

function addFileLookup({ lookup, cardMap, suppressedCues }) {
  if (lookup.result !== 'FILE_EXISTS') return;
  const candidate = {
    identity: `${lookup.intentFile}::__file__`,
    ownerFile: lookup.intentFile,
    exportedName: '__file__',
  };
  addCue(cardMap, suppressedCues, candidate, safeCue({
    lane: 'exact-file',
    claim: 'exact file exists',
    evidence: [{
      artifact: 'topology.json',
      matchedField: 'nodes',
      file: lookup.intentFile,
      algorithmVersion: 'exact-file.v1',
    }],
  }));
}

function addShapeLookup({ lookup, cardMap, suppressedCues, unavailableEvidence }) {
  if (lookup.result === 'UNAVAILABLE') {
    unavailableEvidence.push({
      evidenceLane: lookup.shapeHashSource === 'functionSignature' ? 'function-signature' : 'shape-hash',
      status: UNAVAILABLE_STATUS,
      reason: lookup.reason ?? lookup.unavailableReason ?? 'lookup-unavailable',
      artifact: lookup.artifact ?? (
        lookup.shapeHashSource === 'functionSignature' ? 'function-clones.json' : 'shape-index.json'
      ),
      citations: lookup.citations ?? [],
    });
    return;
  }
  if (lookup.result !== 'SHAPE_MATCH' && lookup.result !== 'SIGNATURE_MATCH') return;
  const lane = lookup.result === 'SIGNATURE_MATCH' ? 'function-signature' : 'shape-hash';
  const claim = lookup.result === 'SIGNATURE_MATCH'
    ? 'same normalized function signature'
    : 'same normalized type shape';
  const artifact = lookup.result === 'SIGNATURE_MATCH' ? 'function-clones.json' : 'shape-index.json';
  for (const match of lookup.matches ?? []) {
    addCue(cardMap, suppressedCues, match, safeCue({
      lane,
      claim,
      evidence: [{
        artifact,
        matchedField: lookup.result === 'SIGNATURE_MATCH' ? 'normalizedSignatureHash' : 'hash',
        algorithmVersion: lookup.result === 'SIGNATURE_MATCH'
          ? 'function-signature.normalized.v1'
          : 'shape-hash.normalized.v1',
        hash: lookup.shapeHash,
      }],
    }));
  }
}

export function classifyPreWriteCues({ lookups = [], intent = {} } = {}) {
  const cardMap = new Map();
  const suppressedCues = [];
  const unavailableEvidence = [];

  for (const lookup of lookups) {
    if (lookup.kind === 'name') addNameLookup({ lookup, cardMap, suppressedCues });
    else if (lookup.kind === 'file') addFileLookup({ lookup, cardMap, suppressedCues });
    else if (lookup.kind === 'shape') addShapeLookup({ lookup, cardMap, suppressedCues, unavailableEvidence });
  }

  return {
    cuePolicy: tokenPolicyMetadata(),
    cueCards: sortCards([...cardMap.values()]),
    suppressedCues: sortSuppressed(suppressedCues),
    unavailableEvidence: unavailableEvidence.sort((a, b) =>
      String(a.evidenceLane ?? '').localeCompare(String(b.evidenceLane ?? '')) ||
      String(a.reason ?? '').localeCompare(String(b.reason ?? ''))
    ),
    intentNameCount: Array.isArray(intent.names) ? intent.names.length : 0,
  };
}
```

- [ ] **Step 3: Run syntax checks**

Run:

```bash
node --check _lib/pre-write-token-policy.mjs
node --check _lib/pre-write-cue-tiers.mjs
```

Expected: both commands exit 0.

- [ ] **Step 4: Run the cue-tier test and verify GREEN**

Run:

```bash
node tests/test-pre-write-cue-tiers.mjs
```

Expected: all tests pass and summary prints `failed` count as 0.

- [ ] **Step 5: Commit the cue-tier module**

Run:

```bash
git add _lib/pre-write-token-policy.mjs _lib/pre-write-cue-tiers.mjs tests/test-pre-write-cue-tiers.mjs
git commit -m "Add pre-write cue tier classifier"
```

Expected: commit succeeds.

---

### Task 3: Name Lookup Weak-Token Suppression

**Files:**
- Modify: `_lib/pre-write-lookup-name.mjs`
- Modify: `tests/test-pre-write-lookup-name.mjs`
- Test: `tests/test-pre-write-lookup-name.mjs`

- [ ] **Step 1: Add RED tests for `createLogger` suppression and supported tokens**

Append these cases before the final summary in `tests/test-pre-write-lookup-name.mjs`:

```js
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
  assert('T22b. create-only candidates are preserved as suppressedSemanticHints',
    r.suppressedSemanticHints.length === 2 &&
    r.suppressedSemanticHints.every((h) =>
      h.reason === 'weak-common-token-only' &&
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
```

- [ ] **Step 2: Run the lookup-name test and verify RED**

Run:

```bash
node tests/test-pre-write-lookup-name.mjs
```

Expected: FAIL because `suppressedSemanticHints` is not present yet or `createLogger` suppression is not implemented.

- [ ] **Step 3: Refactor token helpers to use the token policy module**

In `_lib/pre-write-lookup-name.mjs`, replace the local `normalizeSemanticToken` implementation and token splitting helper import area with:

```js
import { specifierCouldMatchFile } from './finding-provenance.mjs';
import {
  isWeakCommonToken,
  uniquePreWriteTokens,
} from './pre-write-token-policy.mjs';
```

Delete the existing `SEMANTIC_WEAK_VERBS` constant. Then replace
`splitSemanticTokens` and `uniqueTokens` with compatibility wrappers:

```js
function splitSemanticTokens(value) {
  return uniquePreWriteTokens(value)
    .filter((token) =>
      token.length >= 2 &&
      !SEMANTIC_STOP_TOKENS.has(token)
    );
}

function uniqueTokens(...parts) {
  return [...new Set(parts.flatMap(splitSemanticTokens))];
}
```

Keep `SEMANTIC_STOP_TOKENS` in this file for the current lookup policy.

- [ ] **Step 4: Replace `computeSemanticHints` with promoted and suppressed output**

Change `computeSemanticHints` to this shape:

High-fanout token suppression is intentionally deferred in this slice. This
step only suppresses weak-token-only and insufficient-non-weak-support matches.

```js
function computeSemanticHintCandidates(intentName, intentDeclaration, defIndex) {
  const queryTokens = uniqueTokens(intentName, intentDeclaration?.kind, intentDeclaration?.why);
  if (queryTokens.length === 0) return { semanticHints: [], suppressedSemanticHints: [] };
  const querySet = new Set(queryTokens);
  const semanticHints = [];
  const suppressedSemanticHints = [];

  for (const [file, namesObj] of Object.entries(defIndex ?? {})) {
    for (const name of Object.keys(namesObj ?? {})) {
      if (name === intentName) continue;
      const fileStem = file.split(/[\\/]/).pop()?.replace(/\.[^.]+$/, '') ?? '';
      const ownerDir = file.split(/[\\/]/).slice(0, -1).join(' ');
      const candidateNameTokens = uniqueTokens(name);
      const candidateSupportTokens = uniqueTokens(fileStem, ownerDir);
      const candidateTokens = [...new Set([...candidateNameTokens, ...candidateSupportTokens])];
      const matchedTokens = candidateTokens.filter((token) => querySet.has(token));
      if (matchedTokens.length === 0) continue;

      const score = matchedTokens.length;
      if (score < SEMANTIC_HINT_MIN_SCORE) {
        if (matchedTokens.length === 1 && matchedTokens.every(isWeakCommonToken)) {
          suppressedSemanticHints.push({
            name,
            ownerFile: file,
            matchedTokens,
            score,
            reason: 'weak-common-token-only',
          });
        }
        continue;
      }

      const matchedNameTokens = candidateNameTokens.filter((token) => querySet.has(token));
      const strongNameMatches = matchedNameTokens.filter((token) => !isWeakCommonToken(token));
      const strongSupportMatches = candidateSupportTokens
        .filter((token) =>
          querySet.has(token) &&
          !isWeakCommonToken(token) &&
          !strongNameMatches.includes(token)
        );

      if (strongNameMatches.length < 2 && !(strongNameMatches.length === 1 && strongSupportMatches.length >= 1)) {
        suppressedSemanticHints.push({
          name,
          ownerFile: file,
          matchedTokens,
          matchedNameTokens,
          matchedSupportTokens: strongSupportMatches,
          score,
          reason: matchedTokens.every(isWeakCommonToken)
            ? 'weak-common-token-only'
            : 'insufficient-non-weak-support',
        });
        continue;
      }

      semanticHints.push({
        name,
        ownerFile: file,
        matchedTokens,
        matchedNameTokens,
        matchedSupportTokens: strongSupportMatches,
        score,
      });
    }
  }

  const sortHints = (arr) => arr.sort((a, b) =>
    b.score - a.score ||
    a.ownerFile.localeCompare(b.ownerFile) ||
    a.name.localeCompare(b.name)
  );
  const candidateCount = suppressedSemanticHints.length;
  return {
    semanticHints: sortHints(semanticHints).slice(0, SEMANTIC_HINT_MAX_RESULTS),
    suppressedSemanticHints: sortHints(suppressedSemanticHints)
      .slice(0, SEMANTIC_HINT_MAX_RESULTS)
      .map((hint) => ({ ...hint, candidateCount })),
  };
}
```

- [ ] **Step 5: Update `lookupName` to return suppressed semantic hints**

In `lookupName`, replace:

```js
const semanticHints = identities.length === 0
  ? computeSemanticHints(intentName, intentDeclaration, defIndex)
  : [];
```

with:

```js
const semanticCandidateResult = identities.length === 0
  ? computeSemanticHintCandidates(intentName, intentDeclaration, defIndex)
  : { semanticHints: [], suppressedSemanticHints: [] };
const semanticHints = semanticCandidateResult.semanticHints;
const suppressedSemanticHints = semanticCandidateResult.suppressedSemanticHints;
```

Add `suppressedSemanticHints` to the returned object next to `semanticHints`.

- [ ] **Step 6: Run lookup-name test and verify GREEN**

Run:

```bash
node tests/test-pre-write-lookup-name.mjs
```

Expected: all tests pass.

- [ ] **Step 7: Run cue-tier test again**

Run:

```bash
node tests/test-pre-write-cue-tiers.mjs
```

Expected: all tests pass.

- [ ] **Step 8: Commit weak-token suppression**

Run:

```bash
git add _lib/pre-write-lookup-name.mjs _lib/pre-write-token-policy.mjs tests/test-pre-write-lookup-name.mjs tests/test-pre-write-cue-tiers.mjs
git commit -m "Suppress weak pre-write token cues"
```

Expected: commit succeeds.

---

### Task 4: Advisory JSON Integration

**Files:**
- Modify: `pre-write.mjs`
- Modify: `_lib/pre-write-render.mjs`
- Modify: `tests/test-pre-write-render.mjs`
- Modify: `tests/test-pre-write-cli.mjs`

- [ ] **Step 1: Add RED renderJson test for cue fields**

In `tests/test-pre-write-render.mjs`, append before the final summary:

```js
// ═══ Cue-tier JSON fields ═══

{
  const advisory = {
    invocationId: 'cue-json-test',
    intentHash: 'cue-hash',
    intent: { names: ['createLogger'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [],
    cueCards: [{
      candidate: {
        identity: 'src/logger.ts::createLogger',
        ownerFile: 'src/logger.ts',
        exportedName: 'createLogger',
      },
      renderTier: 'SAFE_CUE',
      cues: [{
        cueTier: 'SAFE_CUE',
        safeMeaning: 'claim-only',
        notSafeFor: ['semantic-equivalence', 'auto-reuse', 'auto-fix'],
        evidenceLane: 'exact-symbol',
        claim: 'exact exported symbol exists',
        confidence: 'grounded',
        evidence: [{ artifact: 'symbols.json', matchedField: 'defIndex' }],
      }],
    }],
    suppressedCues: [{
      cueTier: 'MUTED_CUE',
      evidenceLane: 'intent-token',
      reason: 'weak-common-token-only',
      tokens: ['create'],
      candidateCount: 2,
      tokenizerVersion: 'camel-snake-kebab-digit-v1',
      tokenPolicyVersion: 'prewrite-token-policy-v1',
    }],
    unavailableEvidence: [{
      evidenceLane: 'function-signature',
      status: 'UNAVAILABLE',
      reason: 'missing-artifact',
      artifact: 'function-clones.json',
    }],
    cuePolicy: {
      tokenizerVersion: 'camel-snake-kebab-digit-v1',
      tokenPolicyVersion: 'prewrite-token-policy-v1',
      weakCommonTokens: ['create'],
    },
    boundaryChecks: [],
    drift: [],
    capabilities: null,
    failures: [],
  };
  const json = renderJson(advisory);
  assert('N1. renderJson preserves cueCards',
    json.cueCards?.[0]?.cues?.[0]?.cueTier === 'SAFE_CUE',
    JSON.stringify(json.cueCards));
  assert('N2. renderJson preserves suppressedCues',
    json.suppressedCues?.[0]?.reason === 'weak-common-token-only',
    JSON.stringify(json.suppressedCues));
  assert('N3. renderJson preserves unavailableEvidence',
    json.unavailableEvidence?.[0]?.status === 'UNAVAILABLE',
    JSON.stringify(json.unavailableEvidence));
  assert('N4. renderJson preserves cuePolicy',
    json.cuePolicy?.tokenPolicyVersion === 'prewrite-token-policy-v1',
    JSON.stringify(json.cuePolicy));
}
```

- [ ] **Step 2: Run render test and verify RED**

Run:

```bash
node tests/test-pre-write-render.mjs
```

Expected: FAIL because `renderJson` does not yet preserve cue tier fields.

- [ ] **Step 3: Integrate cue classification in `pre-write.mjs`**

Add import:

```js
import { classifyPreWriteCues } from './_lib/pre-write-cue-tiers.mjs';
```

After shape lookups and before advisory assembly, add:

```js
const cueTierResult = classifyPreWriteCues({ lookups, intent });
```

Add these fields to `advisory`:

```js
  cueCards: cueTierResult.cueCards,
  suppressedCues: cueTierResult.suppressedCues,
  unavailableEvidence: cueTierResult.unavailableEvidence,
  cuePolicy: cueTierResult.cuePolicy,
```

- [ ] **Step 4: Preserve cue fields in renderJson**

In `_lib/pre-write-render.mjs`, update `renderJson` to include:

```js
    cueCards: advisory.cueCards ?? [],
    suppressedCues: advisory.suppressedCues ?? [],
    unavailableEvidence: advisory.unavailableEvidence ?? [],
    cuePolicy: advisory.cuePolicy ?? null,
```

- [ ] **Step 5: Add CLI regression for `createLogger` suppression**

Append to `tests/test-pre-write-cli.mjs` before the final summary:

```js
// ═══ Cue tiers: createLogger weak token suppression reaches JSON artifact ═══
{
  const fx = mkdtempSync(path.join(tmpdir(), 'pw-cue-create-'));
  const out = mkdtempSync(path.join(tmpdir(), 'pw-cue-create-out-'));
  try {
    write(fx, 'package.json', JSON.stringify({ name: 'pw-cue-create', type: 'module' }));
    write(fx, 'src/store.ts', 'export const createStore = () => ({});\n');
    write(fx, 'src/storage.ts', 'export const createJSONStorage = () => ({});\n');
    const intent = {
      names: [{ name: 'createLogger', kind: 'function', why: 'create a logger helper' }],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const intentPath = path.join(out, 'intent.json');
    writeFileSync(intentPath, JSON.stringify(intent));

    const stdout = runPreWrite(fx, out, intentPath);
    const parsed = JSON.parse(readFileSync(path.join(out, 'pre-write-advisory.latest.json'), 'utf8'));
    assert('P14.CUE1. create-only hints do not render in default Markdown',
      !stdout.includes('createStore') && !stdout.includes('createJSONStorage'),
      stdout);
    assert('P14.CUE2. create-only hints are recorded as suppressedCues',
      parsed.suppressedCues?.length >= 2 &&
      parsed.suppressedCues.every((cue) =>
        cue.reason === 'weak-common-token-only' &&
        cue.tokenPolicyVersion === 'prewrite-token-policy-v1'),
      JSON.stringify(parsed.suppressedCues));
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}
```

- [ ] **Step 6: Run JSON integration tests**

Run:

```bash
node tests/test-pre-write-render.mjs
node tests/test-pre-write-cli.mjs
```

Expected: both tests pass.

- [ ] **Step 7: Commit JSON integration**

Run:

```bash
git add pre-write.mjs _lib/pre-write-render.mjs tests/test-pre-write-render.mjs tests/test-pre-write-cli.mjs
git commit -m "Surface pre-write cue tiers in advisory JSON"
```

Expected: commit succeeds.

---

### Task 5: Markdown Cue Sections

**Files:**
- Modify: `_lib/pre-write-render.mjs`
- Modify: `tests/test-pre-write-render.mjs`
- Test: `tests/test-pre-write-render.mjs`

- [ ] **Step 1: Add RED Markdown tests for cue sections and forbidden wording**

Append to `tests/test-pre-write-render.mjs` before the final summary:

```js
// ═══ Cue-tier Markdown sections and wording guard ═══

{
  const advisory = {
    invocationId: 'cue-md-test',
    intentHash: 'cue-md-hash',
    intent: { names: ['useShallowFromState'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [],
    cueCards: [{
      candidate: {
        identity: 'src/shallow.ts::useShallow',
        ownerFile: 'src/shallow.ts',
        exportedName: 'useShallow',
      },
      renderTier: 'AGENT_REVIEW_CUE',
      cues: [{
        cueTier: 'SAFE_CUE',
        safeMeaning: 'claim-only',
        notSafeFor: ['semantic-equivalence', 'auto-reuse', 'auto-fix'],
        evidenceLane: 'function-signature',
        claim: 'same normalized function signature',
        confidence: 'grounded',
        evidence: [{ artifact: 'function-clones.json', matchedField: 'normalizedSignatureHash' }],
      }, {
        cueTier: 'AGENT_REVIEW_CUE',
        evidenceLane: 'near-name',
        claim: 'near exported name',
        confidence: 'heuristic-review',
        evidence: [{ artifact: 'symbols.json', matchedField: 'defIndex', distance: 2 }],
      }],
    }],
    suppressedCues: [{
      cueTier: 'MUTED_CUE',
      evidenceLane: 'intent-token',
      reason: 'weak-common-token-only',
      tokens: ['use'],
      candidateCount: 1,
      tokenizerVersion: 'camel-snake-kebab-digit-v1',
      tokenPolicyVersion: 'prewrite-token-policy-v1',
    }],
    unavailableEvidence: [{
      evidenceLane: 'shape-hash',
      status: 'UNAVAILABLE',
      reason: 'missing-artifact',
      artifact: 'shape-index.json',
    }],
    cuePolicy: { tokenizerVersion: 'camel-snake-kebab-digit-v1', tokenPolicyVersion: 'prewrite-token-policy-v1' },
    boundaryChecks: [],
    drift: [],
    capabilities: null,
    failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('O1. Grounded facts section renders SAFE_CUE claim',
    md.includes('### Grounded facts') &&
    md.includes('same normalized function signature') &&
    md.includes('src/shallow.ts::useShallow'),
    md);
  assert('O2. Agent review cues section renders review cue separately',
    md.includes('### Agent review cues') &&
    md.includes('near exported name'),
    md);
  assert('O3. Muted cue details are not rendered by default',
    !md.includes('weak-common-token-only') && !md.includes('Muted noise'),
    md);
  assert('O4. Unavailable evidence section renders lane status',
    md.includes('### Unavailable evidence') &&
    md.includes('shape-index.json'),
    md);
  assert('O5. renderer avoids semantic/reuse-forcing wording',
    !/does the same thing|semantically equivalent|reuse this/i.test(md),
    md);
}

{
  const advisory = {
    invocationId: 'cue-md-dedupe-test',
    intentHash: 'cue-md-dedupe-hash',
    intent: { names: ['formatDate'], shapes: [], files: [], dependencies: [], plannedTypeEscapes: [] },
    lookups: [{
      kind: 'name',
      intentName: 'formatDate',
      result: 'EXISTS',
      identities: [{
        identity: 'src/date.ts::formatDate',
        ownerFile: 'src/date.ts',
        exportedName: 'formatDate',
      }],
      nearNames: [],
      semanticHints: [],
      suppressedSemanticHints: [],
    }],
    cueCards: [{
      candidate: {
        identity: 'src/date.ts::formatDate',
        ownerFile: 'src/date.ts',
        exportedName: 'formatDate',
      },
      renderTier: 'SAFE_CUE',
      cues: [{
        cueTier: 'SAFE_CUE',
        safeMeaning: 'claim-only',
        notSafeFor: ['semantic-equivalence', 'auto-reuse', 'auto-fix'],
        evidenceLane: 'exact-symbol',
        claim: 'exact exported symbol exists',
        confidence: 'grounded',
        evidence: [{ artifact: 'symbols.json', matchedField: 'defIndex' }],
      }],
    }],
    suppressedCues: [],
    unavailableEvidence: [],
    cuePolicy: { tokenizerVersion: 'camel-snake-kebab-digit-v1', tokenPolicyVersion: 'prewrite-token-policy-v1' },
    boundaryChecks: [],
    drift: [],
    capabilities: null,
    failures: [],
  };
  const md = renderMarkdown(advisory);
  assert('O6. exact cue-covered candidate appears once in default Markdown',
    (md.match(/src\/date\.ts::formatDate/g) ?? []).length === 1,
    md);
}
```

- [ ] **Step 2: Run render test and verify RED**

Run:

```bash
node tests/test-pre-write-render.mjs
```

Expected: FAIL because cue-tier Markdown sections do not exist yet.

- [ ] **Step 3: Add cue section render helpers**

In `_lib/pre-write-render.mjs`, add these helpers before `renderCapabilityNotes`:

```js
function evidenceSummary(evidence) {
  const items = Array.isArray(evidence) ? evidence : [];
  if (items.length === 0) return 'evidence recorded';
  return items.map((item) => {
    const parts = [];
    if (item.artifact) parts.push(item.artifact);
    if (item.matchedField) parts.push(item.matchedField);
    if (item.algorithmVersion) parts.push(item.algorithmVersion);
    return parts.join(' / ');
  }).filter(Boolean).join('; ');
}

function renderCueSections(advisory) {
  const cueCards = advisory.cueCards ?? [];
  const unavailable = advisory.unavailableEvidence ?? [];
  const grounded = [];
  const review = [];

  for (const card of cueCards) {
    for (const cue of card.cues ?? []) {
      const row = `- \`${card.candidate?.identity ?? 'unknown'}\` — ${cue.claim}.`;
      const evidence = `  [${cue.confidence ?? 'grounded'}, ${evidenceSummary(cue.evidence)}; cueTier=${cue.cueTier}]`;
      if (cue.cueTier === 'SAFE_CUE') {
        grounded.push(row, evidence, '  Note: grounded fact only; not a semantic-equivalence or auto-reuse claim.');
      } else if (cue.cueTier === 'AGENT_REVIEW_CUE') {
        review.push(row, evidence, '  action: inspect the cited file or symbol before creating parallel code.');
      }
    }
  }

  const out = [];
  if (grounded.length > 0) {
    out.push('### Grounded facts', '', ...grounded, '');
  }
  if (review.length > 0) {
    out.push('### Agent review cues', '', ...review, '');
  }
  if (unavailable.length > 0) {
    out.push('### Unavailable evidence', '');
    for (const u of unavailable) {
      out.push(`- ${u.evidenceLane ?? 'unknown'} — ${u.status ?? 'UNAVAILABLE'} (${u.reason ?? 'unknown'}).`);
      if (u.artifact) out.push(`  artifact: \`${u.artifact}\``);
    }
    out.push('');
  }
  return out;
}

function cueCoveredIdentities(advisory) {
  const covered = new Set();
  for (const card of advisory.cueCards ?? []) {
    const identity = card.candidate?.identity;
    if (!identity) continue;
    for (const cue of card.cues ?? []) {
      if (['exact-symbol', 'near-name', 'intent-token', 'function-signature', 'shape-hash', 'exact-file']
        .includes(cue.evidenceLane)) {
        covered.add(identity);
      }
    }
  }
  return covered;
}

function unavailableEvidenceLanes(advisory) {
  return new Set((advisory.unavailableEvidence ?? [])
    .map((u) => u.evidenceLane)
    .filter(Boolean));
}

function lookupCandidateIdentities(lookup) {
  const out = [];
  for (const identity of lookup.identities ?? []) {
    if (identity.identity) out.push(identity.identity);
  }
  for (const near of lookup.nearNames ?? []) {
    if (near.ownerFile && near.name) out.push(`${near.ownerFile}::${near.name}`);
  }
  for (const hint of lookup.semanticHints ?? []) {
    if (hint.ownerFile && hint.name) out.push(`${hint.ownerFile}::${hint.name}`);
  }
  for (const match of lookup.matches ?? []) {
    if (match.identity) out.push(match.identity);
  }
  if (lookup.kind === 'file' && lookup.intentFile) out.push(`${lookup.intentFile}::__file__`);
  return out;
}

function shouldSkipLegacyLookup(lookup, coveredIdentities, coveredUnavailableLanes) {
  if (lookup.kind === 'shape' && lookup.result === 'UNAVAILABLE') {
    const lane = lookup.shapeHashSource === 'functionSignature' ? 'function-signature' : 'shape-hash';
    return coveredUnavailableLanes.has(lane);
  }
  const identities = lookupCandidateIdentities(lookup);
  return identities.length > 0 && identities.every((identity) => coveredIdentities.has(identity));
}
```

- [ ] **Step 4: Call cue section renderer from `renderMarkdown`**

In `renderMarkdown`, after `renderCapabilityNotes(lookups)` and before section routing output, add cue rendering and dedupe legacy lookup sections that are already covered by cue cards:

```js
  out.push(...renderCueSections(advisory));
  const coveredCueIdentities = cueCoveredIdentities(advisory);
  const coveredUnavailableLanes = unavailableEvidenceLanes(advisory);
```

Then, wherever the renderer loops through `lookups` for legacy sections, skip cue-covered rows:

```js
  for (const lookup of lookups) {
    if (shouldSkipLegacyLookup(lookup, coveredCueIdentities, coveredUnavailableLanes)) continue;
    // existing section routing remains unchanged
  }
```

This keeps the new cue-tier surface from duplicating the old exact/near/shape/file rows. Legacy sections still render evidence lanes that do not yet have cue cards.

- [ ] **Step 5: Run render test and verify GREEN**

Run:

```bash
node tests/test-pre-write-render.mjs
```

Expected: all tests pass.

- [ ] **Step 6: Run pre-write CLI regression**

Run:

```bash
node tests/test-pre-write-cli.mjs
```

Expected: all tests pass.

- [ ] **Step 7: Commit Markdown cue sections**

Run:

```bash
git add _lib/pre-write-render.mjs tests/test-pre-write-render.mjs tests/test-pre-write-cli.mjs
git commit -m "Render pre-write cue tier sections"
```

Expected: commit succeeds.

---

### Task 6: Docs, Skill Mirror, And Final Validation

**Files:**
- Modify: `README.md`
- Generated: `skills/lumin-repo-lens-lab/**`
- Test: mirror and targeted pre-write tests

- [ ] **Step 1: Update README wording**

In `README.md`, update the pre-write Q/A around semantic duplicates so it says:

```md
**Q. Does pre-write understand semantic duplicates?**

No. Pre-write does not claim semantic equivalence from names alone. It surfaces
grounded facts such as exact symbol/file matches, exact shape hashes, and exact
function signature hashes, then separates weaker agent-review cues from muted
token noise. Exact normalized body-hash cueing is deferred until a body-hash
lane exists in the lookup artifacts. When two helpers only share a common verb
such as `create`, the default chat surface stays quiet and the muted cue remains
in JSON diagnostics.
```

- [ ] **Step 2: Run targeted syntax and behavior tests**

Run:

```bash
node --check _lib/pre-write-token-policy.mjs
node --check _lib/pre-write-cue-tiers.mjs
node tests/test-pre-write-cue-tiers.mjs
node tests/test-pre-write-lookup-name.mjs
node tests/test-pre-write-render.mjs
node tests/test-pre-write-cli.mjs
```

Expected: all commands exit 0.

- [ ] **Step 3: Build the shipping skill mirror**

Run:

```bash
npm run build:skill
```

Expected: command exits 0 and mirrors changed engine files into `skills/lumin-repo-lens-lab/`.

- [ ] **Step 4: Run mirror and package checks**

Run:

```bash
npm run check:drift
node tests/test-skill-package.mjs
node tests/test-skill-surface.mjs
```

Expected: all commands exit 0.

- [ ] **Step 5: Regenerate and check test documentation**

Add a `scripts/update-test-doc.mjs` description for `test-pre-write-cue-tiers.mjs`, then run:

```bash
npm run update-test-doc
npm run check:test-doc
```

Expected: `tests/README.md` is regenerated deterministically and the check exits 0.

- [ ] **Step 6: Run whitespace check**

Run:

```bash
git diff --check
```

Expected: command exits 0.

- [ ] **Step 7: Commit docs and mirror**

Run:

```bash
git add README.md skills/lumin-repo-lens-lab pre-write.mjs \
  scripts/update-test-doc.mjs \
  _lib/pre-write-token-policy.mjs \
  _lib/pre-write-cue-tiers.mjs \
  _lib/pre-write-lookup-name.mjs \
  _lib/pre-write-render.mjs \
  tests/README.md \
  tests/test-pre-write-cue-tiers.mjs \
  tests/test-pre-write-lookup-name.mjs \
  tests/test-pre-write-render.mjs \
  tests/test-pre-write-cli.mjs
git commit -m "Update pre-write cue docs and skill mirror"
```

Expected: commit succeeds.

- [ ] **Step 8: Record final changed files**

Run:

```bash
git status --short
git log --oneline -5
```

Expected: working tree is clean and recent commits include the cue-tier implementation commits.

---

## Final Verification Before PR

Run the focused validation suite:

```bash
node tests/test-pre-write-cue-tiers.mjs
node tests/test-pre-write-lookup-name.mjs
node tests/test-pre-write-render.mjs
node tests/test-pre-write-cli.mjs
npm run update-test-doc
npm run check:test-doc
npm run check:drift
node tests/test-skill-package.mjs
node tests/test-skill-surface.mjs
git diff --check
```

Expected: every command exits 0.

Run full CI only if implementation touches shared extraction, symbol graph,
ranking, or packaging beyond the files listed in this plan:

```bash
npm run ci
```

Expected: exit 0. If only this plan's narrow files are touched, the focused
validation suite is the required local gate.

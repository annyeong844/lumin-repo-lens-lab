# TS Near Clone Bounded Retrieval Implementation Plan

> **Status:** Superseded and must not be executed. Production JS/TS
> function-clone grouping and near retrieval moved to Rust audit-core;
> `_lib/function-clone-artifact.mjs` is now a fact-extraction boundary only.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the Rust v9 bounded near-function clone retrieval contract into the TS/JS function-clone producer without adding timeouts, repository caps, or forced candidate quotas.

**Architecture:** Keep `_lib/function-clone-artifact.mjs` as the TS/JS owner. Split near retrieval into small local helpers inside that file first, then only extract if the file becomes unmanageable. Preserve full significant call tokens for scoring while using retained high-IDF tokens only for candidate generation and deterministic dedupe.

**Tech Stack:** Node ESM, existing `threshold-policies.mjs`, existing function-clone fixtures, product JSON artifact contract. Local execution in this session must not run Node per user instruction; Node/product runs are written as external verification steps.

---

## Preconditions

- Work from `C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab`.
- Do not touch the forbidden main repo.
- Current `main` has unrelated dirty JS/TS test files. Implementation must use an isolated worktree from clean `origin/main`.
- Do not run Node in the current session.
- Tests must prove product behavior, not helper existence.

## File Structure

- Modify: `_lib/function-clone-artifact.mjs`
  - Add IDF helpers.
  - Split significant tokens from retained generation tokens.
  - Replace exhaustive bucket scan with retained-token bounded retrieval.
  - Add streaming top-N projection.
  - Add candidate generation policy/summary/skipped bucket artifact fields.
- Modify: `skills/lumin-repo-lens-lab/_engine/lib/threshold-policies.mjs`
  - Add bounded retrieval knobs and score formula metadata to `function-clone-near-policy`.
- Modify: `tests/_helpers/build-function-clone-index-review-near-cases.js`
  - Add behavior assertions for low-IDF skip, low+high survival, uncapped count vs projection, raw estimate labels, and detailed scoring evidence.
- Modify if needed: `tests/_helpers/build-function-clone-index.js`
  - Only add fixture support if existing helpers cannot set enough source files or inspect artifacts.
- Do not modify exact/structure/signature tests unless a product contract now depends on the new artifact fields.

## Task 1: Isolated Worktree

**Files:**
- No repository file changes expected.

- [ ] **Step 1: Detect whether current checkout is already isolated**

Run:

```powershell
git rev-parse --git-dir
git rev-parse --git-common-dir
git rev-parse --show-superproject-working-tree
git branch --show-current
```

Expected:

- If `.git` equals the common dir and no superproject is reported, this is the normal dirty checkout.
- Continue with a separate worktree.

- [ ] **Step 2: Create a clean worktree from `origin/main`**

Use a sibling worktree outside the dirty checkout:

```powershell
git fetch origin main
git worktree add ..\lumin-repo-lens-lab-ts-near-bounded -b codex/ts-near-bounded-retrieval origin/main
Set-Location ..\lumin-repo-lens-lab-ts-near-bounded
git status --short --branch
```

Expected:

```text
## codex/ts-near-bounded-retrieval...origin/main
```

or equivalent clean branch status.

- [ ] **Step 3: Copy no dirty files from main checkout**

Do not copy the current dirty `tests/` tree. Work only from the clean worktree.

## Task 2: Policy Metadata

**Files:**
- Modify: `skills/lumin-repo-lens-lab/_engine/lib/threshold-policies.mjs`

- [ ] **Step 1: Add bounded retrieval metadata to the existing policy**

In the `function-clone-near-policy` entry, keep existing score weights and
thresholds unless the implementation also changes the scoring formula. Add
these threshold fields:

```js
minSingleTokenIdf: 3.0,
callIdfSaturation: 6.0,
skippedLowDiscriminationBucketSampleLimit: 16,
```

Add policy metadata next to `thresholds` and `calibration`:

```js
retrieval: {
  contractVersion: 'function-clone-near-retrieval.v1',
  mode: 'bounded-retrieval',
  candidateCountScope: 'scored-candidates-from-retained-retrieval-evidence',
  pairDedupe: 'ordered-shared-retained-token',
  projection: 'streaming-top-n',
  idfFormula: 'ln((functionCount + 1) / (documentFrequency + 1))',
  idfScope: 'repository-local-function-call-token-document-frequency',
},
scoreFormula: {
  version: 'function-clone-near-score-idf-sum-v1',
  callTokenComponent: 'shared-idf-sum-saturated',
  previousCallTokenComponent: 'jaccard',
  thresholdCompatibility: 'threshold-number-retained-but-call-component-changed',
},
```

Keep `policyVersion: 'function-clone-near-policy-v1'` unless a downstream
review requires bumping it. The score formula version carries the semantic
change.

- [ ] **Step 2: Verify policy summary still serializes**

Static check only in this session:

```powershell
git diff -- skills/lumin-repo-lens-lab/_engine/lib/threshold-policies.mjs
```

Expected:

- The object remains valid ESM syntax by inspection.
- No Node execution in this session.

## Task 3: Near Retrieval Model Helpers

**Files:**
- Modify: `_lib/function-clone-artifact.mjs`

- [ ] **Step 1: Add constants sourced from policy thresholds**

Near existing `FUNCTION_CLONE_NEAR_THRESHOLDS`, derive:

```js
const NEAR_MIN_SINGLE_TOKEN_IDF =
  FUNCTION_CLONE_NEAR_THRESHOLDS.minSingleTokenIdf ?? 3.0;
const NEAR_CALL_IDF_SATURATION =
  FUNCTION_CLONE_NEAR_THRESHOLDS.callIdfSaturation ?? 6.0;
const NEAR_SKIPPED_BUCKET_SAMPLE_LIMIT =
  FUNCTION_CLONE_NEAR_THRESHOLDS.skippedLowDiscriminationBucketSampleLimit ?? 16;
```

- [ ] **Step 2: Add IDF helpers**

Add helpers near `jaccard`/`rangeSimilarity`:

```js
function roundScore(n) {
  return Math.round(n * 1000) / 1000;
}

function choose2(n) {
  const count = Number(n ?? 0);
  return count > 1 ? (count * (count - 1)) / 2 : 0;
}

function callTokenIdfs(facts) {
  const documentFrequency = new Map();
  for (const fact of facts ?? []) {
    for (const token of significantCallTokens(fact)) {
      documentFrequency.set(token, (documentFrequency.get(token) ?? 0) + 1);
    }
  }
  const totalFunctions = Number(facts?.length ?? 0);
  const out = new Map();
  for (const [token, count] of documentFrequency) {
    out.set(token, Math.log((totalFunctions + 1) / (count + 1)));
  }
  return out;
}

function tokenIdf(token, tokenIdfs) {
  return tokenIdfs.get(token) ?? 0;
}

function retainedCallTokens(fact, tokenIdfs) {
  return significantCallTokens(fact)
    .filter((token) => tokenIdf(token, tokenIdfs) >= NEAR_MIN_SINGLE_TOKEN_IDF)
    .sort();
}

function sharedSorted(left, right) {
  const rightSet = new Set(right ?? []);
  return [...new Set(left ?? [])].filter((entry) => rightSet.has(entry)).sort();
}
```

If `roundScore` already exists, do not duplicate it.

- [ ] **Step 3: Add score helpers**

Add:

```js
function sharedTokenIdfSum(sharedTokens, tokenIdfs) {
  return (sharedTokens ?? []).reduce(
    (sum, token) => sum + tokenIdf(token, tokenIdfs),
    0,
  );
}

function saturatedCallTokenIdfScore(sharedIdfSum) {
  return Math.min(1, sharedIdfSum / NEAR_CALL_IDF_SATURATION);
}

function sharedSignificantCallTokenDetails(sharedTokens, retainedSharedTokens, tokenIdfs) {
  const retainedSet = new Set(retainedSharedTokens ?? []);
  return (sharedTokens ?? []).map((token) => ({
    token,
    idf: roundScore(tokenIdf(token, tokenIdfs)),
    retained: retainedSet.has(token),
  }));
}
```

## Task 4: Retained Buckets And Skipped Evidence

**Files:**
- Modify: `_lib/function-clone-artifact.mjs`

- [ ] **Step 1: Build eligible facts once**

In `buildNearFunctionCandidates`, replace repeated `significantCallTokens(fact)`
calls with enriched near facts:

```js
const baseEligible = facts
  .filter((fact) => !grouped.has(fact.identity))
  .filter((fact) => significantCallTokens(fact).length > 0)
  .filter((fact) => fact.generator !== true)
  .sort((a, b) => a.identity.localeCompare(b.identity));

const tokenIdfs = callTokenIdfs(baseEligible);
const eligible = baseEligible.map((fact) => {
  const significantTokens = significantCallTokens(fact);
  const retainedTokens = retainedCallTokens(fact, tokenIdfs);
  return {
    ...fact,
    significantCallTokens: significantTokens,
    retainedCallTokens: retainedTokens,
    retainedCallTokenSet: new Set(retainedTokens),
    nameTokens: nameTokens(fact.exportedName),
  };
});
```

Do not overwrite `callTokens`; keep artifact facts backward-compatible.

- [ ] **Step 2: Build all-token buckets for skipped evidence**

Add:

```js
function bucketByTokens(facts, tokenField) {
  const buckets = new Map();
  for (const fact of facts) {
    for (const token of fact[tokenField] ?? []) {
      if (!buckets.has(token)) buckets.set(token, []);
      buckets.get(token).push(fact);
    }
  }
  return buckets;
}
```

Use `significantCallTokens` for skipped low-discrimination evidence:

```js
const allBuckets = bucketByTokens(eligible, 'significantCallTokens');
```

- [ ] **Step 3: Summarize skipped low-discrimination buckets**

Add:

```js
function skippedLowDiscriminationBuckets(allBuckets, tokenIdfs) {
  const buckets = [];
  for (const [token, facts] of allBuckets) {
    const idf = tokenIdf(token, tokenIdfs);
    const rawPairEstimate = choose2(facts.length);
    if (idf >= NEAR_MIN_SINGLE_TOKEN_IDF || rawPairEstimate === 0) continue;
    buckets.push({
      token,
      idf: roundScore(idf),
      postingCount: facts.length,
      rawPairEstimate,
      reason: 'below-min-single-token-idf',
    });
  }
  buckets.sort((a, b) =>
    b.rawPairEstimate - a.rawPairEstimate ||
    a.token.localeCompare(b.token));
  return {
    count: buckets.length,
    rawPairEstimate: buckets.reduce((sum, bucket) => sum + bucket.rawPairEstimate, 0),
    examples: buckets.slice(0, NEAR_SKIPPED_BUCKET_SAMPLE_LIMIT),
  };
}
```

Expected behavior:

- Low-IDF tokens remain in `significantCallTokens`.
- Only their bucket generation is skipped.

## Task 5: Compatibility Postings

**Files:**
- Modify: `_lib/function-clone-artifact.mjs`

- [ ] **Step 1: Add band helpers**

Use simple deterministic bands. Match Rust's logarithmic-ish intent without
adding a new dependency:

```js
function sizeBand(value, base = 2) {
  const n = Math.max(0, Number(value ?? 0));
  if (n <= 1) return 0;
  return Math.floor(Math.log(n) / Math.log(base)) + 1;
}

function compatibilityKey(fact) {
  return [
    fact.async === true ? 'async' : 'sync',
    `params:${Number(fact.paramCount ?? 0)}`,
    `body:${sizeBand(fact.bodyLoc)}`,
    `statements:${sizeBand(fact.statementCount)}`,
  ].join('|');
}
```

- [ ] **Step 2: Build retained postings by token and compatibility key**

Add:

```js
function retainedPostingsByToken(eligible) {
  const postings = new Map();
  for (const fact of eligible) {
    const key = compatibilityKey(fact);
    for (const token of fact.retainedCallTokens) {
      if (!postings.has(token)) postings.set(token, new Map());
      const byKey = postings.get(token);
      if (!byKey.has(key)) byKey.set(key, []);
      byKey.get(key).push(fact);
    }
  }
  for (const byKey of postings.values()) {
    for (const facts of byKey.values()) {
      facts.sort((a, b) => a.identity.localeCompare(b.identity));
    }
  }
  return postings;
}
```

- [ ] **Step 3: Add retained raw pair estimate**

Add:

```js
function retainedRawPairEstimate(postings) {
  let total = 0;
  for (const byKey of postings.values()) {
    let postingCount = 0;
    for (const facts of byKey.values()) postingCount += facts.length;
    total += choose2(postingCount);
  }
  return total;
}
```

This is a raw work estimate and may double-count pairs that share multiple
retained tokens.

## Task 6: Deterministic Pair Generation And Scoring

**Files:**
- Modify: `_lib/function-clone-artifact.mjs`

- [ ] **Step 1: Add earliest-token dedupe**

Add:

```js
function earliestSharedRetainedToken(left, right) {
  for (const token of left.retainedCallTokens) {
    if (right.retainedCallTokenSet.has(token)) return token;
  }
  return null;
}
```

- [ ] **Step 2: Add streaming top-N projection**

Use a small sorted projection array instead of retaining every candidate:

```js
function compareNearCandidates(a, b) {
  return (
    (b.generatedOnly ? 0 : 1) - (a.generatedOnly ? 0 : 1) ||
    b.score - a.score ||
    a.identities.join('|').localeCompare(b.identities.join('|'))
  );
}

function pushProjectedCandidate(projected, candidate) {
  projected.push(candidate);
  projected.sort(compareNearCandidates);
  if (projected.length > FUNCTION_CLONE_NEAR_THRESHOLDS.maxNearCandidates) {
    projected.pop();
  }
}
```

This keeps memory bounded by projection size. It is acceptable because
`maxNearCandidates` is 50.

- [ ] **Step 3: Replace exhaustive candidate loop**

Iterate retained postings:

```js
const projected = [];
let generatedUniquePairCount = 0;
let scoredPairCount = 0;
let nearFunctionCandidateCount = 0;

for (const token of [...postings.keys()].sort()) {
  const byKey = postings.get(token);
  for (const key of [...byKey.keys()].sort()) {
    const bucket = byKey.get(key);
    for (let i = 0; i < bucket.length; i++) {
      for (let j = i + 1; j < bucket.length; j++) {
        const a = bucket[i];
        const b = bucket[j];
        if (earliestSharedRetainedToken(a, b) !== token) continue;
        generatedUniquePairCount++;

        const sharedCallTokens = sharedSorted(
          a.significantCallTokens,
          b.significantCallTokens,
        );
        const sharedRetainedCallTokens = sharedSorted(
          a.retainedCallTokens,
          b.retainedCallTokens,
        );
        if (sharedCallTokens.length === 0) continue;
        if (
          sharedCallTokens.length === 1 &&
          tokenIdf(sharedCallTokens[0], tokenIdfs) < NEAR_MIN_SINGLE_TOKEN_IDF
        ) continue;

        scoredPairCount++;
        const sharedCallTokenIdfSum = sharedTokenIdfSum(sharedCallTokens, tokenIdfs);
        const callTokenIdfScore = saturatedCallTokenIdfScore(sharedCallTokenIdfSum);
        // Continue with name/body/statement scoring and candidate construction.
        // After threshold pass:
        // if (candidate.generatedOnly !== true) nearFunctionCandidateCount++;
        // pushProjectedCandidate(projected, candidate);
      }
    }
  }
}
```

`nearFunctionCandidateCount` is the uncapped review-visible count. Increment it
only after a candidate passes the near threshold and only when
`candidate.generatedOnly !== true`, matching the existing product summary
semantics.

For this first slice, same compatibility key only is allowed. Do not implement
adjacent partition pairs until a fixture proves the additional complexity is
needed.

- [ ] **Step 4: Score with IDF component**

Replace:

```js
const callTokenJaccard = jaccard(aCalls, bCalls);
```

with:

```js
const sharedCallTokenIdfSum = sharedTokenIdfSum(sharedCallTokens, tokenIdfs);
const callTokenIdfScore = saturatedCallTokenIdfScore(sharedCallTokenIdfSum);
```

Then compute:

```js
const score = roundScore(
  (callTokenIdfScore * FUNCTION_CLONE_NEAR_THRESHOLDS.weights.callTokenJaccard) +
  (nameTokenJaccard * FUNCTION_CLONE_NEAR_THRESHOLDS.weights.nameTokenJaccard) +
  (bodyLocSimilarity * FUNCTION_CLONE_NEAR_THRESHOLDS.weights.bodyLocSimilarity) +
  (statementCountSimilarity * FUNCTION_CLONE_NEAR_THRESHOLDS.weights.statementCountSimilarity)
);
```

Keep the existing `weights.callTokenJaccard` key for compatibility unless the
policy metadata is changed in the same commit. The meaning is documented by
`scoreFormulaVersion`.

- [ ] **Step 5: Emit detailed candidate evidence**

Candidate object must include:

```js
generationToken: token,
sharedCallTokens,
sharedSignificantCallTokens: sharedSignificantCallTokenDetails(
  sharedCallTokens,
  sharedRetainedCallTokens,
  tokenIdfs,
),
sharedCallTokenIdfSum: roundScore(sharedCallTokenIdfSum),
callTokenIdfScore: roundScore(callTokenIdfScore),
```

Keep old fields that downstream consumers may already read:

```js
sharedNameTokens,
nameTokenJaccard,
bodyLocRange,
statementCountRange,
reason,
reasons,
```

## Task 7: Artifact Policy And Summary

**Files:**
- Modify: `_lib/function-clone-artifact.mjs`

- [ ] **Step 1: Return projection plus diagnostics from `buildNearFunctionCandidates`**

Change return shape from array to object:

```js
return {
  candidates: projected,
  candidateGenerationPolicy,
  candidateGenerationSummary,
  skippedLowDiscriminationBucketCount,
  skippedLowDiscriminationRawPairEstimate,
  skippedLowDiscriminationPairEstimateKind:
    'raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-skipped-tokens',
  skippedLowDiscriminationBuckets: skipped.examples,
};
```

Update caller:

```js
const nearProjection = buildNearFunctionCandidates(stampedFacts, exactBodyGroups, structureGroups);
const nearFunctionCandidates = nearProjection.candidates;
```

- [ ] **Step 2: Add candidate generation policy**

Construct:

```js
const candidateGenerationPolicy = {
  mode: 'bounded-retrieval',
  retrievalContractVersion: 'function-clone-near-retrieval.v1',
  idfFormula: 'ln((functionCount + 1) / (documentFrequency + 1))',
  idfScope: 'repository-local-function-call-token-document-frequency',
  bucketMinIdf: NEAR_MIN_SINGLE_TOKEN_IDF,
  callIdfSaturation: NEAR_CALL_IDF_SATURATION,
  scoreFormulaVersion: 'function-clone-near-score-idf-sum-v1',
  candidateCountScope: 'scored-candidates-from-retained-retrieval-evidence',
  pairDedupe: 'ordered-shared-retained-token',
  projection: 'streaming-top-n',
};
```

- [ ] **Step 3: Add candidate generation summary**

Construct:

```js
const candidateGenerationSummary = {
  eligibleFunctionCount,
  retainedCallTokenBucketCount,
  retainedRawPairEstimate: retainedRawPairs,
  retainedRawPairEstimateKind:
    'raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-retained-tokens',
  generatedUniquePairCount,
  scoredPairCount,
  nearFunctionCandidateCount,
  compatibilitySkippedRawPairEstimateByReason: {},
  compatibilitySkippedPairEstimateKind:
    'raw-partition-estimate-does-not-enumerate-rejected-pairs',
  nearFunctionCandidateCountScope: 'bounded-retrieval-retained-evidence',
};
```

For the first same-key-only slice, compute compatibility skipped estimate as:

```js
retainedRawPairEstimate - generatedUniquePairCount
```

Attribute it to `compatibilityPartitionMismatch` until adjacent partition
support is implemented. If this feels too broad during review, rename the field
to `compatibilitySkippedRawPairEstimate` and do not claim per-reason detail.

- [ ] **Step 4: Attach fields to artifact**

At top-level artifact return, add:

```js
candidateGenerationPolicy: nearProjection.candidateGenerationPolicy,
candidateGenerationSummary: nearProjection.candidateGenerationSummary,
skippedLowDiscriminationBucketCount: nearProjection.skippedLowDiscriminationBucketCount,
skippedLowDiscriminationRawPairEstimate: nearProjection.skippedLowDiscriminationRawPairEstimate,
skippedLowDiscriminationPairEstimateKind: nearProjection.skippedLowDiscriminationPairEstimateKind,
skippedLowDiscriminationBuckets: nearProjection.skippedLowDiscriminationBuckets,
```

In `meta`, set:

```js
nearFunctionCandidateCount: nearProjection.candidateGenerationSummary.nearFunctionCandidateCount,
```

Add `nearFunctionCandidateCount` to summary if not already present in the
summary object.

## Task 8: Product Behavior Tests

**Files:**
- Modify: `tests/_helpers/build-function-clone-index-review-near-cases.js`

- [ ] **Step 1: Add low-IDF single-token fixture case**

Append a case that writes several helpers sharing only a generic token that
falls below IDF threshold. Product assertions:

```js
assert(
  "near low-IDF single-token bucket is skipped",
  index.meta.nearFunctionCandidateCount === 0 &&
    index.skippedLowDiscriminationBucketCount > 0 &&
    index.skippedLowDiscriminationRawPairEstimate > 0 &&
    index.skippedLowDiscriminationBuckets.some((bucket) => bucket.token === "render"),
  JSON.stringify(index, null, 2),
);
```

- [ ] **Step 2: Add low+high token survival case**

Create two helpers that share one common low-IDF token and one rare token.
Assert:

```js
assert(
  "near low-IDF token remains scoring evidence when high-IDF token generates pair",
  Boolean(candidate) &&
    candidate.generationToken === "parseSchema" &&
    candidate.sharedCallTokens.includes("render") &&
    candidate.sharedCallTokens.includes("parseSchema") &&
    candidate.sharedSignificantCallTokens.some((entry) =>
      entry.token === "render" && entry.retained === false) &&
    candidate.sharedSignificantCallTokens.some((entry) =>
      entry.token === "parseSchema" && entry.retained === true),
  JSON.stringify(candidate, null, 2),
);
```

- [ ] **Step 3: Add uncapped count vs projection assertion**

Use enough pairs to exceed a local projection limit only if the implementation
allows dependency injection. If it does not, assert the general invariant:

```js
assert(
  "near count is never below projected candidate length",
  index.meta.nearFunctionCandidateCount >= index.nearFunctionCandidates.length &&
    index.nearFunctionCandidates.length <= 50,
  JSON.stringify({
    count: index.meta.nearFunctionCandidateCount,
    projected: index.nearFunctionCandidates.length,
  }),
);
```

Do not add test-only production knobs just to force `maxNearCandidates = 1`.

- [ ] **Step 4: Add raw estimate labeling assertions**

Assert:

```js
assert(
  "near retrieval raw estimate kinds are explicit",
  index.candidateGenerationSummary?.retainedRawPairEstimateKind ===
    "raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-retained-tokens" &&
    index.skippedLowDiscriminationPairEstimateKind ===
      "raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-skipped-tokens",
  JSON.stringify(index.candidateGenerationSummary, null, 2),
);
```

- [ ] **Step 5: Add policy metadata assertions**

Assert:

```js
assert(
  "near bounded retrieval policy exposes IDF and score formula provenance",
  index.candidateGenerationPolicy?.idfFormula ===
    "ln((functionCount + 1) / (documentFrequency + 1))" &&
    index.candidateGenerationPolicy?.idfScope ===
      "repository-local-function-call-token-document-frequency" &&
    index.candidateGenerationPolicy?.scoreFormulaVersion ===
      "function-clone-near-score-idf-sum-v1",
  JSON.stringify(index.candidateGenerationPolicy, null, 2),
);
```

## Task 9: Verification And Commit

**Files:**
- All modified files from earlier tasks.

- [ ] **Step 1: Static checks allowed in this session**

Run:

```powershell
git diff --check
git diff --stat
git status --short --branch
```

Expected:

- No whitespace errors.
- Only the planned files are modified.

- [ ] **Step 2: Node verification deferred**

Do not run Node in this session. Record in final response that product Node
tests were not run due user instruction.

External verification command for reviewer/CI:

```powershell
node tests/test-build-function-clone-index.mjs
```

Expected external behavior:

- Function-clone near behavior tests pass.
- `function-clones.json` includes bounded retrieval policy, summary, skipped
  bucket evidence, and candidate-level IDF evidence.

- [ ] **Step 3: Commit only planned files**

Run:

```powershell
git add -- _lib/function-clone-artifact.mjs `
  skills/lumin-repo-lens-lab/_engine/lib/threshold-policies.mjs `
  tests/_helpers/build-function-clone-index-review-near-cases.js
git commit -m "Backport bounded near clone retrieval to TS"
```

If `tests/_helpers/build-function-clone-index.js` was modified, include it
explicitly in the `git add`.

- [ ] **Step 4: Push branch**

Run:

```powershell
git push -u origin codex/ts-near-bounded-retrieval
```

Do not merge to `main` until external Node verification has run.

## Self-Review Notes

- Spec coverage:
  - IDF formula and scope: Task 3, Task 7, Task 8.
  - significant vs retained token separation: Task 3, Task 4, Task 6.
  - low-IDF bucket skip without pair exclusion: Task 4, Task 6, Task 8.
  - deterministic dedupe: Task 6.
  - raw estimate labels: Task 7, Task 8.
  - uncapped count vs projection: Task 6, Task 7, Task 8.
- No wall-clock timeouts, repo-size caps, or forced 50-candidate quotas are introduced.
- Node execution is explicitly deferred because the user asked not to run Node locally.

# TS Near Clone Bounded Retrieval Design

> **Status:** Superseded. Since the audit-core function-clone migration,
> `_lib/function-clone-artifact.mjs` owns JS/TS fact extraction only and Rust
> `lumin-audit-core/src/function_clones/near.rs` owns production near retrieval.
> Do not implement this design or restore a JS artifact-construction owner.

## Objective

Port the Rust v9 near-function clone retrieval contract back to the TS/JS
function-clone producer.

The goal is not to make TS emit more near candidates. The goal is to stop
treating near clone detection as "compare every possible pair, then slice 50"
and instead define near candidates as bounded, high-discrimination review
evidence with artifact-visible omissions.

## Owner

Primary owner:

- `_lib/function-clone-artifact.mjs`

Supporting owner:

- `skills/lumin-repo-lens-lab/_engine/lib/threshold-policies.mjs`

The CLI wrapper `build-function-clone-index.mjs` should remain a thin producer
entry point. Exact body groups, structure groups, and signature groups are out
of scope for this slice.

## Current Problem

The TS/JS near lane currently mirrors the old Rust shape:

1. collect significant call tokens
2. build `token -> [function facts]` buckets
3. enumerate every pair inside each bucket
4. store global pair keys in an unbounded `Set`
5. push every passing candidate into `candidates[]`
6. sort all candidates and return `slice(0, maxNearCandidates)`

This was acceptable at smaller TS/JS operating scale, but the shape is the same
structural debt that made the Rust lane fail on large Rust workspaces before
v9. The product semantics are also wrong: `maxNearCandidates` is a projection
limit, not an analysis quota, and low-discrimination buckets should not create
grounded absence claims when skipped.

## Chosen Approach

Implement the Rust v9 bounded retrieval contract in TS/JS near candidate
generation.

Keep the language-specific token extractor and existing exact/structure
normalizers. Change only the near retrieval path.

The new TS/JS near lane should:

- compute repository-local IDF for significant call tokens
- use high-IDF retained tokens as pair-generation sources
- preserve the full significant token set for scoring and explanations
- derive pair dedupe from deterministic retained-token order
- keep projected candidates with streaming top-N behavior
- expose skipped low-discrimination bucket evidence in `function-clones.json`

## Retrieval Contract

### Significant Tokens

`significantCallTokens(fact)` remains the full scoring/evidence token set after
the existing generic-token filter.

Do not mutate facts or permanently remove low-IDF tokens from this set. Rust v8
proved that doing so corrupts score evidence by making shared token IDF sums
disappear.

### IDF Calculation

Use the same repository-local IDF formula as Rust v9:

```text
idf(token) = ln((functionCount + 1) / (documentFrequency(token) + 1))
```

Definitions:

- `functionCount`: number of parsed function clone facts in the current
  artifact before exact/structure grouping removes near-eligible facts
- `documentFrequency(token)`: number of function clone facts whose full
  `significantCallTokens` set contains the token
- logarithm base: natural log, matching Rust `f64::ln()`
- scope: repository-local function call-token document frequency

This must be represented in policy metadata so `bucketMinIdf = 3.0` and
`callIdfSaturation = 6.0` remain interpretable.

### Retained Tokens

Add a retained generation token set:

```text
retainedCallTokens(fact) =
  significantCallTokens(fact).filter(idf(token) >= minSingleTokenIdf)
```

Retained tokens are only for bucket generation and deterministic pair dedupe.
They are not the full scoring evidence.

### Low-Discrimination Buckets

If a token IDF is below `minSingleTokenIdf`, do not use that token bucket as a
pair-generation source.

This is not pair exclusion. A pair that shares both a low-IDF token and a
high-IDF token still surfaces from the high-IDF token.

### Compatibility Partitioning

Retained postings must be partitioned before pair enumeration. Do not open a
large retained bucket and then reject most of its pairs afterward.

The first TS slice should partition by deterministic cheap evidence already on
the function facts:

- async qualifier
- parameter-count band
- body LOC band
- statement-count band

The retained posting index shape should be:

```text
token -> compatibilityKey -> function facts
```

Pair loops run only within compatible partition pairs. If adjacent bands are
allowed, partition-pair enumeration must use canonical ordering:

```text
partitionKeyA <= partitionKeyB
```

For the same partition, enumerate `i < j`. For distinct compatible partitions,
enumerate the Cartesian product once. Do not enumerate both `A -> B` and
`B -> A`.

The implementation may keep the first slice conservative, but it must push
cheap guards before pair enumeration where possible.

Retained-side compatibility estimates are computed from posting sizes, not by
enumerating rejected function pairs. Use a fixed guard order to attribute raw
skip estimates so reason buckets do not overlap:

1. qualifier mismatch
2. parameter-count band mismatch
3. body LOC band mismatch
4. statement-count band mismatch

### Pair Dedupe

A pair is generated only from the earliest retained shared call token under
deterministic token order:

```text
generator_token == min(shared_retained_call_tokens(left, right))
```

The candidate score and explanation must be computed from the full shared
significant token set, not just the generator token and not just retained
tokens.

Candidate-level evidence should expose the distinction:

```json
{
  "generationToken": "parseSchema",
  "sharedSignificantCallTokens": [
    { "token": "render", "idf": 1.2, "retained": false },
    { "token": "parseSchema", "idf": 4.8, "retained": true }
  ]
}
```

`sharedCallTokens` may remain as the simple string array for existing
consumers, but the detailed field is the debugging contract.

### Scoring

Replace call-token Jaccard as the primary call component with Rust v9's shared
IDF sum component:

```text
sharedCallTokenIdfSum = sum(idf(token) for token in shared significant tokens)
callTokenIdfScore = min(1.0, sharedCallTokenIdfSum / callIdfSaturation)
```

The weighted score keeps the existing feature shape:

```text
score =
  callTokenIdfScore * callTokenWeight +
  nameTokenJaccard * nameTokenWeight +
  bodyLocSimilarity * bodyLocWeight +
  statementCountSimilarity * statementCountWeight
```

Single-token candidates still need the minimum single-token IDF gate. A single
shared token below that threshold cannot surface a candidate.

Candidate output should include:

- `sharedCallTokenIdfSum`
- `callTokenIdfScore`
- `generationToken`
- `sharedSignificantCallTokens[]` with token IDF and retained flag

This makes the Rust v8 regression impossible to hide: low-IDF tokens may be
absent as generation sources, but they remain visible as scoring evidence.

## Count Semantics

The count fields must have fixed meanings:

- `eligibleFunctionCount`: near retrieval target function fact count after
  exact/structure grouped identities are removed and generated-only facts are
  excluded according to the existing policy
- `retainedCallTokenBucketCount`: retained token buckets with
  `idf >= minSingleTokenIdf` and at least two postings
- `retainedRawPairEstimate`: raw sum of `choose2(postingCount)` over retained
  buckets; this is a work estimate, not a unique pair count
- `generatedUniquePairCount`: unique pairs that reached generation through a
  compatible partition and the earliest shared retained-token rule
- `scoredPairCount`: generated pairs that reached score calculation
- `nearFunctionCandidateCount`: uncapped count of generated candidates that pass
  the bounded retrieval near threshold
- `nearFunctionCandidates.length`: projected candidate array length, always
  `<= maxNearCandidates`

Never derive `nearFunctionCandidateCount` from the projected array length.
Streaming top-N projection must increment the uncapped count separately.

## Artifact Surface

Expose bounded retrieval metadata in `function-clones.json`.

The artifact should include:

```json
{
  "candidateGenerationPolicy": {
    "mode": "bounded-retrieval",
    "retrievalContractVersion": "function-clone-near-retrieval.v1",
    "idfFormula": "ln((functionCount + 1) / (documentFrequency + 1))",
    "idfScope": "repository-local-function-call-token-document-frequency",
    "bucketMinIdf": 3.0,
    "callIdfSaturation": 6.0,
    "scoreFormulaVersion": "function-clone-near-score-idf-sum-v1",
    "candidateCountScope": "scored-candidates-from-retained-retrieval-evidence",
    "pairDedupe": "ordered-shared-retained-token",
    "projection": "streaming-top-n"
  },
  "candidateGenerationSummary": {
    "eligibleFunctionCount": 0,
    "retainedCallTokenBucketCount": 0,
    "retainedRawPairEstimate": 0,
    "retainedRawPairEstimateKind": "raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-retained-tokens",
    "generatedUniquePairCount": 0,
    "scoredPairCount": 0,
    "compatibilitySkippedRawPairEstimateByReason": {},
    "compatibilitySkippedPairEstimateKind": "raw-partition-estimate-does-not-enumerate-rejected-pairs",
    "nearFunctionCandidateCountScope": "bounded-retrieval-retained-evidence"
  },
  "skippedLowDiscriminationBucketCount": 0,
  "skippedLowDiscriminationRawPairEstimate": 0,
  "skippedLowDiscriminationPairEstimateKind": "raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-skipped-tokens",
  "skippedLowDiscriminationBuckets": []
}
```

The skipped bucket examples are capped for artifact size. The count and raw pair
estimate are uncapped.

Raw pair estimates are work estimates. They may double-count pairs that share
multiple skipped tokens and must not be described as unique omitted pair counts.

The same rule applies to retained buckets. `retainedRawPairEstimate` is a raw
work estimate and may double-count pairs that share multiple retained tokens.

Skipped bucket examples should use a count name that does not imply unique
omitted pairs:

```json
{
  "token": "render",
  "idf": 1.24,
  "postingCount": 183,
  "rawPairEstimate": 16653,
  "reason": "below-min-single-token-idf"
}
```

## Policy Versioning

Keep `function-clone-near-policy-v1` if the public score threshold remains
unchanged, but do not imply that the old score semantics are unchanged.

Add the bounded retrieval knobs to policy metadata:

- `minSingleTokenIdf = 3.0`
- `callIdfSaturation = 6.0`
- `idfFormula = "ln((functionCount + 1) / (documentFrequency + 1))"`
- `idfScope = "repository-local-function-call-token-document-frequency"`
- `retrievalContractVersion = "function-clone-near-retrieval.v1"`
- `candidateGenerationMode = "bounded-retrieval"`
- `candidateCountScope = "scored-candidates-from-retained-retrieval-evidence"`
- `scoreFormulaVersion = "function-clone-near-score-idf-sum-v1"`

Because the call-token component changes from Jaccard to saturated shared IDF
sum, add calibration metadata:

```json
{
  "scoreFormulaVersion": "function-clone-near-score-idf-sum-v1",
  "scoreCalibration": {
    "callTokenComponent": "shared-idf-sum-saturated",
    "previousCallTokenComponent": "jaccard",
    "callIdfSaturation": 6.0,
    "thresholdCompatibility": "threshold-number-retained-but-call-component-changed"
  }
}
```

## Tests

Tests must prove product behavior, not helper existence.

Required product cases:

1. A low-IDF single-token bucket produces no near candidate and appears in
   skipped-bucket evidence:
   - `nearFunctionCandidateCount === 0`
   - `skippedLowDiscriminationBucketCount > 0`
   - `skippedLowDiscriminationRawPairEstimate > 0`
   - `skippedLowDiscriminationBuckets[]` includes the token
2. A pair sharing a low-IDF token and a high-IDF token still appears, proving
   low-IDF bucket exclusion is not pair exclusion:
   - the candidate appears
   - `generationToken` is the high-IDF token
   - `sharedSignificantCallTokens[]` includes both the low-IDF and high-IDF
     tokens
3. A multi-retained-token pair is generated once, and generator token order does
   not change the final score or ranking:
   - `nearFunctionCandidateCount === 1`
   - `nearFunctionCandidates.length === 1`
   - score is stable when source token order changes
4. The score explanation includes full shared significant tokens, including
   low-IDF tokens that were not generation sources:
   - `sharedCallTokens` includes the low-IDF token
   - `sharedCallTokenIdfSum` includes that token's IDF
   - `callTokenIdfScore` is computed from the full shared significant set
5. Large retained buckets are partitioned before pair enumeration, and the
   artifact reports raw compatibility-skip estimates instead of silently doing a
   full `O(k^2)` loop:
   - `retainedRawPairEstimate` is large
   - `generatedUniquePairCount` and `scoredPairCount` are much smaller
   - `compatibilitySkippedRawPairEstimateByReason` has values
   - `compatibilitySkippedPairEstimateKind` is present
6. `nearFunctionCandidateCount` is uncapped while
   `nearFunctionCandidates.length <= maxNearCandidates`:
   - force `maxNearCandidates = 1` in a product fixture
   - assert `nearFunctionCandidateCount > 1`
   - assert `nearFunctionCandidates.length === 1`
7. Skipped low-discrimination estimates are labeled as raw estimates that may
   double-count.
8. Retained raw pair estimates are labeled as raw estimates that may
   double-count pairs shared by multiple retained tokens.

Because the user asked not to run Node in the current workflow, implementation
verification may be split:

- local static review and `git diff --check` here
- external Node/product artifact execution when CI or reviewer capacity returns

Do not fake product coverage with tests that only prove helper exports exist.

## Non-Goals

- Do not add wall-clock timeouts.
- Do not cap repositories by file count, LOC, or function count.
- Do not force exactly 50 near candidates.
- Do not rewrite TS exact, structure, or signature clone grouping.
- Do not alter Rust near retrieval in this slice.
- Do not add approximate ANN, LSH, or WAND-style retrieval in this first TS
  backport. Those are separate lanes.

## Success Criteria

The TS/JS backport is done when:

- `function-clones.json` exposes the bounded retrieval policy and summary
- skipped low-discrimination buckets are visible
- low-IDF generation exclusion does not remove full scoring evidence
- IDF formula, scope, and score formula version are artifact-visible
- retained and skipped raw estimates are labeled as raw work estimates
- projected near candidates remain deterministic
- no new wall-clock timeout or repository-size cap is introduced
- downstream consumers can distinguish "no evidence", "not projected", and
  "low-discrimination bucket skipped"

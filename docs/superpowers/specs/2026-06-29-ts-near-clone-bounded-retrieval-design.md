# TS Near Clone Bounded Retrieval Design

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

Pair loops run only within compatible postings or explicitly adjacent bands.
The implementation may keep the first slice conservative, but it must push
cheap guards before pair enumeration where possible.

### Pair Dedupe

A pair is generated only from the earliest retained shared call token under
deterministic token order:

```text
generator_token == min(shared_retained_call_tokens(left, right))
```

The candidate score and explanation must be computed from the full shared
significant token set, not just the generator token and not just retained
tokens.

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

## Artifact Surface

Expose bounded retrieval metadata in `function-clones.json`.

The artifact should include:

```json
{
  "candidateGenerationPolicy": {
    "mode": "bounded-retrieval",
    "retrievalContractVersion": "function-clone-near-retrieval.v1",
    "bucketMinIdf": 3.0,
    "callIdfSaturation": 6.0,
    "candidateCountScope": "scored-candidates-from-retained-retrieval-evidence",
    "pairDedupe": "ordered-shared-retained-token",
    "projection": "streaming-top-n"
  },
  "candidateGenerationSummary": {
    "eligibleFunctionCount": 0,
    "retainedCallTokenBucketCount": 0,
    "retainedRawPairEstimate": 0,
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

## Policy Versioning

Keep `function-clone-near-policy-v1` if the public score threshold remains
unchanged.

Add the bounded retrieval knobs to policy metadata:

- `minSingleTokenIdf = 3.0`
- `callIdfSaturation = 6.0`
- `retrievalContractVersion = "function-clone-near-retrieval.v1"`
- `candidateGenerationMode = "bounded-retrieval"`
- `candidateCountScope = "scored-candidates-from-retained-retrieval-evidence"`

If the TS score formula changes from Jaccard to IDF-sum scoring, add explicit
calibration metadata rather than hiding the change behind the old threshold
numbers.

## Tests

Tests must prove product behavior, not helper existence.

Required product cases:

1. A low-IDF single-token bucket produces no near candidate and appears in
   skipped-bucket evidence.
2. A pair sharing a low-IDF token and a high-IDF token still appears, proving
   low-IDF bucket exclusion is not pair exclusion.
3. A multi-retained-token pair is generated once, and generator token order does
   not change the final score or ranking.
4. The score explanation includes full shared significant tokens, including
   low-IDF tokens that were not generation sources.
5. Large retained buckets are partitioned before pair enumeration, and the
   artifact reports raw compatibility-skip estimates instead of silently doing a
   full `O(k^2)` loop.
6. `nearFunctionCandidateCount` is uncapped while
   `nearFunctionCandidates.length <= maxNearCandidates`.
7. Skipped low-discrimination estimates are labeled as raw estimates that may
   double-count.

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
- projected near candidates remain deterministic
- no new wall-clock timeout or repository-size cap is introduced
- downstream consumers can distinguish "no evidence", "not projected", and
  "low-discrimination bucket skipped"

# Rust Near Clone Retrieval Design

## Objective

Convert Rust near-function clone detection from an exhaustive pair scanner into
a bounded retrieval pipeline.

The goal is not to hide work behind a cap. The goal is to define near-function
cues as high-discrimination review evidence from the start, while preserving
artifact honesty:

- no wall-time timeout
- no repository-size cap
- no forced quota to fill 50 candidates
- no grounded absence claim for skipped low-discrimination buckets
- deterministic output and counts

## Problem

The current Rust implementation inherited the TS/JS near-candidate shape:

1. group exact/structure matches
2. collect eligible functions
3. build call-token buckets
4. compare function pairs from shared buckets
5. score passing candidates
6. project at most `maxNearCandidates`

Recent Rust work fixed the largest memory issue by streaming the projected
top-N candidate array instead of storing every candidate. That prevents the
artifact projection from keeping `O(pair count)` candidate objects.

The deeper problem remains: candidate generation still starts from pairwise
comparison. Large low-discrimination buckets can force millions of pair
evaluations before the policy rejects them. On a large production Rust
workspace such as codex-rs, this makes near-candidate analysis too expensive
for routine dogfooding.

This is a signal-definition problem, not only a performance problem. Near
candidates are review cues, not semantic proof. A useful near lane should not
try to inspect every possible pair; it should retrieve plausible high-value
pairs and make any omitted low-discrimination evidence visible.

## Chosen Approach

Use a bounded retrieval architecture:

```text
facts -> exact/structure/signature grouping -> retrieval indexes
      -> candidate generation -> scoring -> streaming projection
      -> diagnostics and omitted-bucket evidence
```

This replaces "compare everything then trim" with "retrieve plausible pairs,
score them, and project the best review cues."

## Pipeline

### 1. Fact Phase

Each function body fact already records the necessary local evidence:

- identity
- file and line
- callable kind and qualifiers
- body LOC
- statement count
- call tokens
- name tokens
- generated-file classification

This phase remains per-function and deterministic.

### 2. Strong Group Phase

Exact body groups, structure groups, and signature groups run before near
candidate generation. Their grouped identities are removed from near
eligibility.

This keeps near focused on review cues that were not already explained by
stronger grouping lanes.

### 3. Retrieval Index Phase

Build small, explicit indexes from eligible facts:

- call-token retrieval postings:
  token -> compatibility key -> function indexes
- name-token index: token -> function indexes
- body-size band index
- statement-count band index

The first implementation should only make call-token buckets bounded, because
that is where the measured blow-up occurs. Name and size indexes can remain
supporting filters until the call-token retrieval model is stable.

The call-token retrieval posting key must include deterministic compatibility
dimensions that can be checked before pair enumeration:

- qualifier signature: async/unsafe/const
- parameter-count band
- body LOC band
- statement-count band

Pair loops operate only inside compatible postings or explicitly allowed
adjacent bands. This keeps "cheap guards" out of the post-pair filter path. The
retrieval model should avoid opening a full retained token bucket and then
rejecting most of its `O(k^2)` pairs.

### 4. Low-Discrimination Bucket Exclusion

Use the existing `minSingleTokenIdf = 3.0` policy earlier in the pipeline.

If a call token's repository-local IDF is below `minSingleTokenIdf`, the token's
bucket is not used as a pair-generation source.

This does not drop a pair if the same two functions also share a higher-IDF
token. The pair will still be generated from the high-discrimination bucket.
Only "this pair is visible solely because of a low-discrimination token" is
excluded before scoring.

This is not a new threshold. It moves an existing score-time rule to candidate
generation, where it belongs.

### 5. Candidate Generation

For every retained call-token compatibility posting:

1. iterate deterministic function pairs inside compatible postings only
2. skip a pair unless this token is the earliest retained shared call token for
   that pair
3. score the candidate with existing near-candidate scoring

The pair dedupe must not use an unbounded `Set` of all pair keys. Dedupe should
be derived from deterministic token order and each pair's shared retained
tokens.

Pair dedupe invariant:

```text
generator_token == min(shared_retained_call_tokens(left, right))
```

A pair is generated only from the earliest retained shared call token under
deterministic token order. When generated, the score is computed from the full
retained shared-token set, not only from the generator token. Deterministic
token order is a dedupe mechanism, not a ranking input.

### 6. Streaming Projection

Keep the current streaming projection behavior:

- maintain only the projected top-N candidate array
- keep `nearFunctionCandidateCount` as the uncapped review-visible total
- preserve the same ordering as full score-and-sort projection
- never treat `maxNearCandidates` as an analysis cap or quota

If there are 13 review-worthy near candidates, the artifact should show 13, not
force 50.

### 7. Artifact Honesty

The artifact must expose omitted-bucket evidence so users can distinguish:

- "no near evidence exists"
- "near evidence exists but was not projected"
- "low-discrimination buckets were intentionally not used for pair generation"

Add a machine-readable diagnostics surface under function clone groups:

```json
{
  "candidateGenerationPolicy": {
    "mode": "bounded-retrieval",
    "retrievalContractVersion": "function-clone-near-retrieval.v1",
    "bucketMinIdf": 3.0,
    "candidateCountScope": "scored-candidates-from-retained-retrieval-evidence",
    "pairDedupe": "ordered-shared-retained-token",
    "projection": "streaming-top-n"
  },
  "candidateGenerationSummary": {
    "eligibleFunctionCount": 12345,
    "retainedCallTokenBucketCount": 420,
    "retainedRawPairEstimate": 850000,
    "generatedUniquePairCount": 24000,
    "scoredPairCount": 3100,
    "compatibilitySkippedRawPairEstimateByReason": {
      "qualifierMismatch": 12000,
      "parameterCountDelta": 3000,
      "bodyLocBandMismatch": 5000,
      "statementCountBandMismatch": 900
    },
    "compatibilitySkippedPairEstimateKind": "raw-partition-estimate-does-not-enumerate-rejected-pairs",
    "nearFunctionCandidateCountScope": "bounded-retrieval-retained-evidence"
  },
  "skippedLowDiscriminationBuckets": [
    {
      "token": "assert",
      "idf": 2.6,
      "functionCount": 1234,
      "rawPairEstimate": 760761,
      "reason": "below-min-single-token-idf"
    }
  ],
  "skippedLowDiscriminationBucketCount": 16,
  "skippedLowDiscriminationRawPairEstimate": 3900000,
  "skippedLowDiscriminationPairEstimateKind": "raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-skipped-tokens"
}
```

The example array should be capped for artifact size, but the count and pair
estimate must remain uncapped.

Diagnostics must distinguish raw bucket-pair estimates from unique pair counts.
Skipped low-discrimination estimates are raw sums of `C(functionCount, 2)` per
bucket and may double-count pairs that share multiple skipped tokens. They are
work estimates, not unique omitted-pair counts.

Retained-side rejection diagnostics must follow the same rule. Any field that
describes pairs avoided by compatibility partitioning is an estimate computed
from posting sizes, not an exact count gathered by enumerating rejected pairs.

## Policy And Versioning

The near policy remains `function-clone-near-policy-v1` unless the scoring
thresholds change.

The Rust calibration version should bump because candidate generation semantics
change:

```text
rust-function-clone-near-calibration.v6
```

The policy should expose:

- `retrievalContractVersion = "function-clone-near-retrieval.v1"`
- `candidateGenerationMode = "bounded-retrieval"`
- `candidateCountScope = "scored-candidates-from-retained-retrieval-evidence"`
- `bucketMinIdf = minSingleTokenIdf`
- `skippedLowDiscriminationBucketSampleLimit`
- `pairDedupe = "ordered-shared-retained-token"`
- `projection = "streaming-top-n"`

`nearFunctionCandidateCount` is not the count of all possible near clones in the
complete pair universe. It is the uncapped count of candidates that passed from
retained retrieval evidence into review-worthy near scoring.

## TS/JS Relationship

TS/JS currently has the same inherited structure: it builds pair keys and
candidate arrays, then sorts and slices. That was acceptable at smaller
operating scale but is still the same structural debt.

Rust should implement and validate bounded retrieval first because codex-rs
exposes the problem immediately. Once stable, the same retrieval contract should
be ported back to TS/JS. The language-specific token extractors can differ, but
the retrieval contract should be shared:

- low-discrimination buckets do not generate pairs
- high-discrimination shared evidence can still surface a pair
- projected arrays are not quotas
- omitted low-discrimination work is visible

## Tests

Tests must prove product behavior, not scaffolding.

Required Rust tests:

1. A low-IDF single-token bucket produces no near candidate and appears in
   skipped-bucket evidence.
2. A pair sharing both a low-IDF token and a high-IDF token still appears,
   proving low-IDF bucket exclusion is not pair exclusion.
3. Multi-token pairs are counted once, with no unbounded pair-key set.
4. `nearFunctionCandidateCount` remains uncapped while
   `nearFunctionCandidates.length <= nearFunctionCandidateProjectionLimit`.
5. Skipped-bucket counts and pair estimates remain visible even when examples
   are capped.
6. A large retained high-IDF token bucket is partitioned before pair
   enumeration, so incompatible LOC/statement bands do not force full-bucket
   `O(k^2)` pair loops.
7. Generator-token order does not change ranking or score for a pair sharing
   multiple retained tokens.
8. Skipped pair estimates are exposed as raw estimates, including the estimate
   kind that says they may double-count pairs shared by multiple skipped tokens.

Large-repo dogfood should measure:

- pair evaluation count before and after
- skipped low-discrimination bucket count
- top projected candidate overlap against the previous exhaustive scorer on a
  representative member such as codex-core
- whether full codex-rs completes without OOM

## Non-Goals

- Do not add wall-clock timeouts.
- Do not cap repositories by file count, LOC, or function count.
- Do not force exactly 50 near candidates.
- Do not turn skipped low-discrimination buckets into absence claims.
- Do not rewrite exact, structure, or signature grouping in this slice.
- Do not implement the TS/JS backport in the first Rust implementation slice.
- Do not implement WAND-style upper-bound pruning, LSH, or ANN retrieval in the
  first slice. Those may be future lanes after the bounded-retrieval contract is
  stable and measurable.

## Implementation Shape

Keep the work localized to the near lane:

- `function_clones/near.rs`: orchestration
- `function_clones/near/model.rs`: candidate-generation summary structs
- `function_clones/near/scoring.rs`: IDF and score helpers
- `function_clones/near/candidate.rs`: projection of scored pairs
- `protocol/function_clones/*`: artifact policy and diagnostics shape
- `canonical/rust-source-health.md`: canonical contract update

If new helper families are needed, canonical ownership must be documented
before code.

## Success Criteria

The slice is successful when:

- focused function-body fingerprint tests pass
- `cargo check` and `cargo clippy -D warnings` pass for `lumin-rust-source-health`
- representative large-member simulation preserves top projected candidates
  while reducing pair evaluations materially
- full codex-rs no longer fails from candidate/candidate-key accumulation
- the artifact explains what was skipped and why

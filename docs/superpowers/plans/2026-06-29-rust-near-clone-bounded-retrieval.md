# Rust Near Clone Bounded Retrieval Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert Rust near-function clone candidate generation from retained-token full-bucket pair loops to bounded retrieval with compatibility postings and artifact-visible skipped evidence.

**Architecture:** Keep the work inside `rust-source-health`'s existing near lane. Build retained call-token postings keyed by compatibility dimensions before pair enumeration, generate each pair only from its earliest retained shared token, score from the full retained shared-token set, and serialize candidate-generation diagnostics that distinguish raw estimates from unique counts.

**Tech Stack:** Rust 1.95 offline basepack, `serde` protocol structs, existing `function_clones/near/*` modules, product-behavior integration tests under `tests/integration/function_body_fingerprints/near.rs`.

---

## File Map

- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol.rs`
  - Bump near calibration to v6.
  - Add retrieval contract constants and sample-limit metadata.
- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol/function_clones.rs`
  - Add serialized candidate-generation diagnostics fields to `AstFunctionCloneGroups`.
- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol/function_clones/groups.rs`
  - Add diagnostic structs for policy, summary, skipped buckets, and raw estimate kinds.
- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol/function_clones/policy.rs`
  - Expose retrieval contract fields in `AstNearFunctionCandidatePolicy`.
- Modify: `experiments/rust-sidecar/rust-source-health/src/function_clones.rs`
  - Pass diagnostics from near projection into artifact.
- Modify: `experiments/rust-sidecar/rust-source-health/src/function_clones/near/model.rs`
  - Add `CompatibilityKey`, diagnostic summary structs, and projection diagnostics.
- Modify: `experiments/rust-sidecar/rust-source-health/src/function_clones/near.rs`
  - Replace raw `token -> Vec<usize>` pair generation with retained compatibility postings.
  - Keep earliest-retained-token dedupe without an unbounded pair-key set.
- Modify: `experiments/rust-sidecar/rust-source-health/src/function_clones/near/candidate.rs`
  - Reuse existing full-score logic; no score semantics change.
- Modify: `experiments/rust-sidecar/rust-source-health/tests/integration/function_body_fingerprints/near.rs`
  - Add product-behavior tests for low-IDF skipped evidence, high+low shared pair preservation, partitioning, generator-token invariance, and raw estimate naming.
- Modify: `canonical/rust-source-health.md`
  - Document the Rust-only bounded retrieval guard and artifact evidence contract before relying on it in code.

---

## Task 1: Protocol Contract And Canonical Owner

**Files:**
- Modify: `canonical/rust-source-health.md`
- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol.rs`
- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol/function_clones.rs`
- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol/function_clones/groups.rs`
- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol/function_clones/policy.rs`

- [ ] **Step 1: Add canonical note**

Add a short section to `canonical/rust-source-health.md` under the Rust source-health/function clone area:

```markdown
### Near-function bounded retrieval

Rust near-function clone candidates use bounded retrieval for large repositories.
Low-discrimination call-token buckets do not generate pairs, but pairs that also
share retained higher-discrimination tokens remain eligible. Compatibility
guards such as qualifiers, parameter count, body LOC, and statement count must
be applied before pair enumeration where possible. The artifact must expose raw
skipped-bucket estimates and the retrieval contract version; these estimates are
work estimates and may double-count pairs shared by multiple skipped tokens.

Do not add wall-clock timeouts or repository-size caps to this lane.
```

- [ ] **Step 2: Add protocol constants**

In `protocol.rs`, change calibration:

```rust
pub const RUST_FUNCTION_CLONE_NEAR_CALIBRATION_VERSION: &str =
    "rust-function-clone-near-calibration.v6";
```

Add:

```rust
pub const RUST_FUNCTION_CLONE_NEAR_RETRIEVAL_CONTRACT_VERSION: &str =
    "function-clone-near-retrieval.v1";
pub const RUST_FUNCTION_CLONE_NEAR_CANDIDATE_GENERATION_MODE: &str =
    "bounded-retrieval";
pub const RUST_FUNCTION_CLONE_NEAR_CANDIDATE_COUNT_SCOPE: &str =
    "scored-candidates-from-retained-retrieval-evidence";
pub const RUST_FUNCTION_CLONE_NEAR_PAIR_DEDUPE: &str =
    "ordered-shared-retained-token";
pub const RUST_FUNCTION_CLONE_NEAR_PROJECTION: &str = "streaming-top-n";
pub const RUST_FUNCTION_CLONE_NEAR_SKIPPED_BUCKET_SAMPLE_LIMIT: usize =
    RUST_FUNCTION_CLONE_NEAR_MAX_CANDIDATES;
pub const RUST_FUNCTION_CLONE_NEAR_SKIPPED_PAIR_ESTIMATE_KIND: &str =
    "raw-bucket-pairs-may-double-count-pairs-shared-by-multiple-skipped-tokens";
pub const RUST_FUNCTION_CLONE_NEAR_COMPATIBILITY_SKIPPED_PAIR_ESTIMATE_KIND: &str =
    "raw-partition-estimate-does-not-enumerate-rejected-pairs";
```

- [ ] **Step 3: Add serialized diagnostic structs**

In `protocol/function_clones/groups.rs`, add structs:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstNearFunctionCandidateGenerationPolicy {
    pub mode: &'static str,
    pub retrieval_contract_version: &'static str,
    pub bucket_min_idf: f64,
    pub candidate_count_scope: &'static str,
    pub pair_dedupe: &'static str,
    pub projection: &'static str,
    pub skipped_low_discrimination_bucket_sample_limit: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstNearFunctionCandidateGenerationSummary {
    pub eligible_function_count: usize,
    pub retained_call_token_bucket_count: usize,
    pub retained_raw_pair_estimate: usize,
    pub generated_unique_pair_count: usize,
    pub scored_pair_count: usize,
    pub compatibility_skipped_raw_pair_estimate_by_reason:
        AstNearFunctionCompatibilitySkippedPairEstimates,
    pub compatibility_skipped_pair_estimate_kind: &'static str,
    pub near_function_candidate_count_scope: &'static str,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstNearFunctionCompatibilitySkippedPairEstimates {
    pub qualifier_mismatch: usize,
    pub parameter_count_delta: usize,
    pub body_loc_band_mismatch: usize,
    pub statement_count_band_mismatch: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AstSkippedLowDiscriminationBucket {
    pub token: String,
    pub idf: f64,
    pub function_count: usize,
    pub raw_pair_estimate: usize,
    pub reason: &'static str,
}
```

- [ ] **Step 4: Add fields to `AstFunctionCloneGroups`**

Add:

```rust
pub candidate_generation_policy: AstNearFunctionCandidateGenerationPolicy,
pub candidate_generation_summary: AstNearFunctionCandidateGenerationSummary,
pub skipped_low_discrimination_buckets: Vec<AstSkippedLowDiscriminationBucket>,
pub skipped_low_discrimination_bucket_count: usize,
pub skipped_low_discrimination_raw_pair_estimate: usize,
pub skipped_low_discrimination_pair_estimate_kind: &'static str,
```

Default values must use the constants above.

- [ ] **Step 5: Expose retrieval fields in near policy**

Add fields to `AstNearFunctionCandidatePolicy`:

```rust
pub retrieval_contract_version: &'static str,
pub candidate_generation_mode: &'static str,
pub candidate_count_scope: &'static str,
pub skipped_low_discrimination_bucket_sample_limit: usize,
pub pair_dedupe: &'static str,
pub projection: &'static str,
```

Populate them from constants.

---

## Task 2: Bounded Retrieval Model

**Files:**
- Modify: `experiments/rust-sidecar/rust-source-health/src/function_clones/near/model.rs`
- Modify: `experiments/rust-sidecar/rust-source-health/src/function_clones/near.rs`

- [ ] **Step 1: Add model types**

Add to `near/model.rs`:

```rust
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct CompatibilityKey {
    pub(super) qualifier_signature: QualifierSignature,
    pub(super) param_count: usize,
    pub(super) body_loc_band: usize,
    pub(super) statement_count_band: usize,
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct QualifierSignature {
    pub(super) is_async: bool,
    pub(super) is_unsafe: bool,
    pub(super) is_const: bool,
}

#[derive(Default)]
pub(super) struct CandidateGenerationDiagnostics {
    pub(super) eligible_function_count: usize,
    pub(super) retained_call_token_bucket_count: usize,
    pub(super) retained_raw_pair_estimate: usize,
    pub(super) generated_unique_pair_count: usize,
    pub(super) scored_pair_count: usize,
    pub(super) skipped_low_discrimination_buckets:
        Vec<crate::protocol::AstSkippedLowDiscriminationBucket>,
    pub(super) skipped_low_discrimination_bucket_count: usize,
    pub(super) skipped_low_discrimination_raw_pair_estimate: usize,
}
```

Extend `NearFunctionCandidateProjection` with:

```rust
pub(in crate::function_clones) diagnostics: CandidateGenerationDiagnostics,
```

- [ ] **Step 2: Add compatibility helper methods**

In `near.rs`, add helpers:

```rust
fn compatibility_key(fact: &NearFact<'_>) -> CompatibilityKey { ... }
fn range_band(value: usize) -> usize { ... }
fn raw_pair_estimate(count: usize) -> usize { count.saturating_mul(count.saturating_sub(1)) / 2 }
```

Use coarse logarithmic-ish bands, but assign generation only to partitions that still pass final `range_similarity`; the final score path remains authoritative.

---

## Task 3: Candidate Generation Rewrite

**Files:**
- Modify: `experiments/rust-sidecar/rust-source-health/src/function_clones/near.rs`

- [ ] **Step 1: Build retained and skipped buckets**

Replace `by_call_token: BTreeMap<&str, Vec<usize>>` with:

```rust
let mut retained = BTreeMap::<String, BTreeMap<CompatibilityKey, Vec<usize>>>::new();
let mut skipped = Vec::<AstSkippedLowDiscriminationBucket>::new();
```

Use `token_idfs` and `RUST_FUNCTION_CLONE_NEAR_MIN_SINGLE_TOKEN_IDF` to skip low-IDF token buckets before generation. Keep count/raw estimates visible.

- [ ] **Step 2: Generate from compatible postings**

For each retained token and each compatible posting, iterate pairs only inside that posting. Keep the existing earliest-token dedupe logic, but make it operate on retained shared tokens only.

- [ ] **Step 3: Preserve full scoring**

Call `candidate::near_candidate_from_pair(left, right, &token_idfs)` unchanged. This guarantees the candidate score still uses the full retained shared-token set through existing scoring code.

- [ ] **Step 4: Populate diagnostics**

Fill:

- `eligible_function_count`
- `retained_call_token_bucket_count`
- `retained_raw_pair_estimate`
- `generated_unique_pair_count`
- `scored_pair_count`
- skipped bucket sample/count/raw estimate

Do not enumerate rejected pairs to compute diagnostics. Any avoided-work numbers are raw estimates from bucket/posting sizes.

---

## Task 4: Wire Diagnostics Into Artifact

**Files:**
- Modify: `experiments/rust-sidecar/rust-source-health/src/function_clones.rs`
- Modify: `experiments/rust-sidecar/rust-source-health/src/protocol/function_clones.rs`

- [ ] **Step 1: Copy diagnostics from projection**

Set `AstFunctionCloneGroups` diagnostic fields from `near_function_candidates.diagnostics`.

- [ ] **Step 2: Keep summary count semantics**

Keep:

```rust
near_function_candidate_count: near_function_candidates.review_visible_count,
```

This is the retained retrieval evidence count, not the all-pairs universe.

---

## Task 5: Product Behavior Tests

**Files:**
- Modify: `experiments/rust-sidecar/rust-source-health/tests/integration/function_body_fingerprints/near.rs`

- [ ] **Step 1: Update calibration assertion**

Change expected calibration from:

```rust
"rust-function-clone-near-calibration.v5"
```

to:

```rust
"rust-function-clone-near-calibration.v6"
```

- [ ] **Step 2: Assert policy diagnostics exist**

In `function_body_clone_groups_include_ts_style_near_candidates`, assert:

```rust
assert_eq!(
    groups["policy"]["nearCandidatePolicy"]["retrievalContractVersion"],
    "function-clone-near-retrieval.v1"
);
assert_eq!(
    groups["candidateGenerationPolicy"]["mode"],
    "bounded-retrieval"
);
assert_eq!(
    groups["candidateGenerationPolicy"]["candidateCountScope"],
    "scored-candidates-from-retained-retrieval-evidence"
);
```

- [ ] **Step 3: Add low-IDF skipped evidence assertion**

Extend the low-IDF test to assert `skippedLowDiscriminationBucketCount > 0`, `skippedLowDiscriminationRawPairEstimate > 0`, and `skippedLowDiscriminationPairEstimateKind` names raw estimates.

- [ ] **Step 4: Add high+low shared pair test**

Use two functions sharing `assert` plus a rare token like `unwrap_switch`. Assert the near candidate still appears and contains `unwrap_switch`.

- [ ] **Step 5: Add generator order invariance test**

Use two functions sharing two retained rare tokens. Assert candidate score and shared token set include both tokens, regardless of token sort order in source.

- [ ] **Step 6: Add partition diagnostic test**

Use many same-token functions with incompatible qualifiers or param counts. Assert generated/scored counts are lower than retained raw pair estimate and compatibility estimate kind says no rejected-pair enumeration occurred.

---

## Task 6: Verification

**Files:**
- No code edits.

- [ ] **Step 1: Format**

Run:

```powershell
wsl bash -lc "cd /mnt/c/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab/experiments && RUSTUP_HOME=/mnt/c/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab/tools/offline-rust-basepack/.work/extracted/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline/rustup CARGO_HOME=/mnt/c/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab/tools/offline-rust-basepack/.work/extracted/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline/cargo PATH=/mnt/c/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab/tools/offline-rust-basepack/.work/extracted/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/mnt/c/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab/tools/offline-rust-basepack/.work/extracted/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline/target/lumin-repo-lens-lab cargo fmt --all --check"
```

- [ ] **Step 2: Focused behavior tests**

Run:

```powershell
wsl bash -lc "cd /mnt/c/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab/experiments && RUSTUP_HOME=/mnt/c/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab/tools/offline-rust-basepack/.work/extracted/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline/rustup CARGO_HOME=/mnt/c/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab/tools/offline-rust-basepack/.work/extracted/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline/cargo PATH=/mnt/c/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab/tools/offline-rust-basepack/.work/extracted/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin CARGO_NET_OFFLINE=true CARGO_TARGET_DIR=/mnt/c/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab/tools/offline-rust-basepack/.work/extracted/lumin-rust-1.95.0-x86_64-unknown-linux-gnu-offline/target/lumin-repo-lens-lab cargo test --offline --locked -p lumin-rust-source-health function_body_fingerprints -- --nocapture"
```

- [ ] **Step 3: Check and clippy**

Run `cargo check --offline --locked -p lumin-rust-source-health` and `cargo clippy --offline --locked -p lumin-rust-source-health --all-targets -- -D warnings` with the same offline environment prefix.

- [ ] **Step 4: Git hygiene**

Run:

```powershell
git diff --check -- experiments/rust-sidecar/rust-source-health canonical/rust-source-health.md
git status --short --branch
```

---

## Task 7: Commit

**Files:**
- Stage only Rust source-health and canonical files touched by this plan.

- [ ] **Step 1: Commit**

Run:

```powershell
git add canonical/rust-source-health.md experiments/rust-sidecar/rust-source-health
git diff --cached --check
git commit -m "Bound Rust near clone retrieval"
```

- [ ] **Step 2: Push**

Run:

```powershell
git push
```

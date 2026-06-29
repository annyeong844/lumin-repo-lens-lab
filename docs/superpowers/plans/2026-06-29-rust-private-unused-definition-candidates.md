# Rust Private Unused Definition Candidates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Promote genuinely private Rust zero-reference definitions into raw unused-definition `findings[]` without creating safe edits.

**Architecture:** Keep ownership inside `rust-source-health/src/dead_exports.rs`. Public, trait, cfg, FFI, derive, opaque, test-only, generated, and Rust-entrypoint gates remain blockers; only supported module-owned private functions, consts, and statics with zero observed production and test references become `remove-candidate` evidence with `safeAction: null`.

**Tech Stack:** Rust 1.95 offline Cargo, `rust-source-health` typed protocol, existing integration artifact harness.

---

### Task 1: Private Candidate Classification

**Files:**
- Modify: `experiments/rust-sidecar/rust-source-health/src/dead_exports.rs`
- Test: `experiments/rust-sidecar/rust-source-health/tests/integration/unused_definitions.rs`

- [x] **Step 1: Add product-behavior coverage**

Extend `unused_definitions.rs` with one fixture where `fn truly_dead_private_helper() {}` is not referenced and one fixture where a private helper is referenced only from `#[cfg(test)]`. The first must appear in `findings[]` with `tier/action = "remove-candidate"` and `safeAction = null`; the second must not appear as a finding and must expose `RUST-FP-G`.

- [x] **Step 2: Implement private module-owned candidate promotion**

In `classify_unused_definitions`, after existing public gate handling, route non-public module definitions with zero production refs through private classification. Block cfg, FFI, derive, opaque, and test-only evidence before emitting a candidate. Emit candidates into `findings[]`; keep `excludedCandidates[]` for blockers.

- [x] **Step 3: Preserve action safety**

Private candidates must keep `safeAction: null` and searched scope `crate-local-name-and-qualified-path-refs`. Do not add `SAFE_FIX`, edit spans, wall-time caps, or new Rust-only thresholds.

- [x] **Step 4: Verify**

Run offline Rust checks for `lumin-rust-source-health`: `cargo fmt --all`, `cargo check --offline --locked -p lumin-rust-source-health`, targeted `unused_definition` tests, and `cargo clippy --offline --locked -p lumin-rust-source-health --all-targets -- -D warnings`.

- [x] **Step 5: Dogfood sanity**

Run source-health on ripgrep and codex-core. Confirm public/trait/opaque blockers still do not leak into `findings[]`, and record whether private candidates appear in real repositories.

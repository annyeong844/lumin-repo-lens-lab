# Rust Near Retrieval Product Evidence Review

Reviewed range:

```text
1cee71a^..4911e3b
```

Head commit:

```text
4911e3b Expose Rust near retrieval evidence in product artifacts
```

## Purpose

This packet reviews the Rust near-function clone retrieval work from design
through product artifact exposure.

The important product question is not just whether `rust-source-health` computes
bounded near candidates, but whether downstream users can see why near evidence
is missing, bounded, or projected. The latest commit exposes that transparency
through the compact syntax artifact and the unified `lumin-rust-analyzer`
product artifact.

## What Changed

- Rust near clone retrieval was changed from exhaustive pair generation to
  bounded retrieval with retained high-IDF generation buckets.
- Scoring evidence was kept separate from generation evidence:
  `significant_call_tokens` remains the full scoring/evidence set, while
  `retained_call_tokens` is only a generation/dedupe key set.
- `rust-source-health` compact output now keeps:
  - `candidateGenerationPolicy`
  - `candidateGenerationSummary`
  - skipped low-discrimination bucket counts, raw estimates, estimate kind, and
    capped examples
- `lumin-rust-analyzer` product output now carries near retrieval transparency
  into:
  - top-level `summary`
  - `phases.syntax.summary`

## Dogfood Baseline

The v9 dogfood baseline recorded from the 2026-06-29 review packet is now
preserved in `canonical/rust-source-health.md`:

- full `codex-rs` completed without the prior near-candidate OOM
- shared call-token IDF sums matched the pre-retrieval v7 baseline for common
  top candidate pairs:
  - `ripgrep`: 50/50
  - `bytes`: 50/50
  - `clap`: 49/49
  - `serde`: 46/46

Those dogfood runs were not rerun while creating this packet. This packet
verifies the product contract and code paths around the already-recorded
baseline.

## Verification

Fresh commands run before the latest commit:

```text
cargo fmt --all
cargo check --offline --locked -p lumin-rust-source-health -p lumin-rust-analyzer
cargo test --offline --locked -p lumin-rust-source-health --test integration -- --nocapture
cargo test --offline --locked -p lumin-rust-analyzer --test integration artifact_contract -- --nocapture
cargo clippy --offline --locked -p lumin-rust-source-health -p lumin-rust-analyzer --all-targets -- -D warnings
git diff --check
```

Results:

- `rust-source-health` integration tests: 68 passed
- `lumin-rust-analyzer` artifact contract tests: 6 passed
- `cargo check`: passed
- `cargo clippy -D warnings`: passed
- `git diff --check`: passed

Node was not run.

## Included Files

- `commit-list.txt`: reviewed commits
- `latest-commit-stat.txt`: latest commit summary
- `working-diff-check.txt`: output from `git diff --check`
- `cached-diff-check.txt`: output from `git diff --cached --check`
- `artifact-field-map.md`: product artifact fields added or preserved
- `verification-summary.json`: machine-readable verification summary

The external review zip also contains `tracked.patch` and `git-status.txt`.
The committed baseline omits the generated patch because its verbatim diff
contents trip repository whitespace checks even though the reviewed commit
range itself passed `git diff --check`.

## Known Workspace State

The product repo still has unrelated dirty JS/TS test files. They were present
before this packet work and were not staged, reverted, or modified for the Rust
near retrieval product evidence work.

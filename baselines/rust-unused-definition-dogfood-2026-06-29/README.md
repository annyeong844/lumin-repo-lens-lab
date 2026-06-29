# Rust Unused Definition Dogfood

Date: 2026-06-29

This packet records the Rust unused-definition dogfood rerun after merging the
`rust-dead-export-design` branch into `main`.

The purpose is to verify that public zero-reference Rust definitions do not
become removal findings. They must stay blocked by artifact-visible Rust FP
gates until Rust has proof-carrying edit safety.

Large JSON artifacts are kept outside the product repository:

```text
C:\Users\endof\Downloads\lumin-perf-lab\review\rust-unused-definition-dogfood
```

## Important Correction

The externally reported `668` count is reproducible on `codex-core`, but it is
not the public-surface gate count.

On `codex-core`:

```text
RUST-FP-A public surface gate = 435
RUST-FP-B trait impl gate     = 668
```

So the important product claim is:

- all `RUST-FP-A` public zero-reference definitions are excluded candidates
- none are `remove-candidate`
- none have `safeAction`
- the `668` count is a separate trait-impl contract gate, also excluded from
  removal

## Verification Commands

Node was not run.

Rust verification used the offline Rust 1.95 basepack:

```text
cargo fmt --all
cargo check --offline --locked -p lumin-rust-source-health
cargo test --offline --locked -p lumin-rust-source-health unused_definition -- --nocapture
cargo clippy --offline --locked -p lumin-rust-source-health --all-targets -- -D warnings
```

Results:

```text
cargo fmt: passed
cargo check: passed
unused_definition integration tests: 9 passed
cargo clippy -D warnings: passed
```

## Dogfood Targets

### codex-rs

Root:

```text
C:\Users\endof\Downloads\repo\suyeonevo\codex-main\codex-rs
```

Artifact:

```text
C:\Users\endof\Downloads\lumin-perf-lab\review\rust-unused-definition-dogfood\codex-rs-rust-health.json
```

Run result:

```text
profile=compact files=2406 skipped=0 signals=30337
elapsed=4:26.11
max_rss_kb=313112
artifact_bytes=25373800
```

Unused-definition summary:

```text
definition_count=49891
candidate_count=0
review_count=0
degraded_count=33
findings_len=0
excluded_candidates_len=9233
blocked_public_surface_count=1901
blocked_trait_impl_count=2906
blocked_opaque_count=1819
blocked_derive_surface_count=2508
blocked_cfg_count=33
blocked_ffi_count=0
test_only_support_count=66
```

Gate/action summary:

```text
RUST-FP-A=1901
RUST-FP-B=2906
RUST-FP-C=1819
RUST-FP-E=2508
RUST-FP-F=33
RUST-FP-G=66

action.demote-to-restricted=1901
action.review=7299
action.degraded=33
```

Safety check:

```text
RUST-FP-A remove-candidate or safeAction count=0
```

### codex-core

Root:

```text
C:\Users\endof\Downloads\repo\suyeonevo\codex-main\codex-rs\core
```

Artifact:

```text
C:\Users\endof\Downloads\lumin-perf-lab\review\rust-unused-definition-dogfood\codex-core-rust-health.json
```

Run result:

```text
profile=compact files=496 skipped=0 signals=9372
elapsed=0:30.88
max_rss_kb=87780
artifact_bytes=5971616
```

Unused-definition summary:

```text
definition_count=9567
candidate_count=0
review_count=0
degraded_count=10
findings_len=0
excluded_candidates_len=1416
blocked_public_surface_count=435
blocked_trait_impl_count=668
blocked_opaque_count=202
blocked_derive_surface_count=75
blocked_cfg_count=10
blocked_ffi_count=0
test_only_support_count=26
```

Gate/action summary:

```text
RUST-FP-A=435
RUST-FP-B=668
RUST-FP-C=202
RUST-FP-E=75
RUST-FP-F=10
RUST-FP-G=26

action.demote-to-restricted=435
action.review=971
action.degraded=10
```

Safety check:

```text
RUST-FP-A remove-candidate or safeAction count=0
```

### ripgrep

Root:

```text
C:\Users\endof\Downloads\repo\Util\ripgrep-master
```

The checkout has no `.git` directory, so the artifact used
`--source-commit dogfood-local`.

Artifact:

```text
C:\Users\endof\Downloads\lumin-perf-lab\review\rust-unused-definition-dogfood\ripgrep-rust-health.json
```

Run result:

```text
profile=compact files=100 skipped=0 signals=1234
elapsed=0:04.20
max_rss_kb=31852
artifact_bytes=2180544
```

Unused-definition summary:

```text
definition_count=3379
candidate_count=0
review_count=0
degraded_count=5
findings_len=0
excluded_candidates_len=1413
blocked_public_surface_count=222
blocked_trait_impl_count=985
blocked_opaque_count=147
blocked_derive_surface_count=54
blocked_cfg_count=5
blocked_ffi_count=0
test_only_support_count=0
```

Gate/action summary:

```text
RUST-FP-A=222
RUST-FP-B=985
RUST-FP-C=147
RUST-FP-E=54
RUST-FP-F=5

action.demote-to-restricted=222
action.review=1186
action.degraded=5
```

Safety check:

```text
RUST-FP-A remove-candidate or safeAction count=0
```

## Product Contract Checked

The dogfood artifacts confirm:

- Rust unused-definition analysis currently emits no removal findings.
- Public zero-reference Rust definitions are represented as excluded candidates
  with `RUST-FP-A`.
- `RUST-FP-A` candidates use `action="demote-to-restricted"`,
  `tier="review"`, and `safeAction=null`.
- Trait-impl methods are separately blocked by `RUST-FP-B`.
- Large Rust repositories complete without elapsed-time caps or timeouts.

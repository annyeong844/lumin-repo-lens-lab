# M7 Rust Workspace Structure Verification - 2026-06-17

This note records the current verification shape for the Rust M7 workspace
after the workspace consolidation, typed oracle split, integration-test target
consolidation, and dependency-boundary pass.

## Commands

All commands are run from:

```text
C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments
```

Use the workspace path, not individual crate manifests:

```text
cargo lumin-fmt
cargo lumin-clippy
cargo lumin-test
```

The GitHub Rust job follows the same path: one `experiments/` workspace, one
`ci-test` profile, and no per-crate default `debug` test runs. The aliases live
in `experiments/.cargo/config.toml` and expand to the explicit `fmt`, `clippy`,
and `test` commands above. CI change detection treats `experiments/Cargo.toml`,
`experiments/Cargo.lock`, `experiments/.cargo/*`, `experiments/rust-common/*`,
`experiments/rust-main/*`, and `experiments/rust-sidecar/*` as Rust surfaces.

## Current Evidence

Latest local verification observed:

- `cargo lumin-fmt`: pass.
- `cargo lumin-clippy`: pass.
- `cargo lumin-test`: pass.
- `cargo tree -p lumin-rust-cargo-oracle --edges normal --invert windows-sys`:
  `warning: nothing to print.`
- `rg "serde_json::Value|json!\(|value_at\(|require_equal|require_present"
  experiments/rust-main/lumin-rust-analyzer/src`: no product-layer matches.
- `rg "serde_json::Value|\bValue\b|json!\("
  experiments/rust-main/rust-cargo-oracle/src/protocol
  experiments/rust-main/rust-cargo-oracle/src/artifact
  experiments/rust-main/rust-cargo-oracle/src/oracle_plan.rs
  experiments/rust-main/rust-cargo-oracle/src/scope`: no protocol or scope
  `Value` matches; remaining matches are raw Cargo/rustc JSON decoding and
  input-hash materialization boundaries.
- Production panic scan over analyzer, oracle, source-health, topology-scanner,
  and common sources excluding tests: no `unwrap()`, `expect()`, `panic!`,
  `todo!`, `unimplemented!`, `dbg!`, or `unsafe {` matches. The word `unsafe`
  still appears in data-field names and path-policy messages; those are not
  unsafe blocks.
- `rg "root\.descendants\(|collect_file_syntax|collect_syntax_node"
  experiments/rust-sidecar/rust-source-health/src/analyzer*`: one
  `root.descendants()` loop in `analyzer/syntax.rs`, with per-node dispatch in
  `analyzer/syntax/visit.rs`.
- `rg "allow\(dead_code\)|dead_code|src_path"
  experiments/rust-main/rust-cargo-oracle/src
  experiments/rust-sidecar/rust-source-health/src`: no dead-code allowance or
  stale `src_path` metadata field matches.
- `cargo clippy --locked -p lumin-rust-cargo-oracle --all-targets --profile
  ci-test`: pass after removing the unused metadata parameter from cargo config
  discovery.
- `cargo test --locked -p lumin-rust-cargo-oracle --profile ci-test`: pass
  after the signature cleanup and after limiting `cargo metadata --no-deps` to
  the `metadata-only` oracle mode. `cargo-check` and `targeted-cargo-check`
  still keep full dependency metadata for semantic diagnostic classification
  and input identity.
- `cargo test --locked -p lumin-rust-cargo-oracle --profile ci-test`: pass
  after narrowing `analysisInputSetHash` source-byte hashing to selected local
  package dependency closures. The integration case
  `analysis_input_hash_ignores_unselected_workspace_member_source` proves that
  an unselected workspace member source change does not invalidate selected
  semantic evidence, while workspace root config and selected package source
  remain part of the hash. The integration case
  `analysis_input_hash_tracks_selected_package_local_dependency_source` proves
  that a selected package's local path dependency source still invalidates the
  selected semantic evidence hash, while an independent unselected member does
  not. The targeted-oracle integration case
  `targeted_analysis_input_hash_tracks_selected_package_local_dependency_source`
  proves the same closure behavior for `targeted-cargo-check`, which is the
  unified analyzer path that narrows oracle work from Rust AST target paths.
  `targeted_analysis_input_hash_uses_normalized_target_path_set` proves target
  path order, duplicate entries, and slash direction do not change the selected
  semantic evidence hash when the selected package set is the same.
- `cargo clippy --locked -p lumin-rust-source-health --all-targets --profile
  ci-test`: pass after verifying the single traversal source-health analyzer
  structure.
- `cargo test --locked -p lumin-rust-source-health --profile ci-test`: pass
  after the same analyzer traversal verification.
- `cargo clippy --locked -p lumin-rust-analyzer --all-targets --profile
  ci-test`: pass after lowering bridge coverage evidence once per artifact
  build, consolidating action-tier, oracle-confidence, and action-reason serde
  naming rules, and exposing unlinked semantic diagnostics in the product
  summary.
- `cargo test --locked -p lumin-rust-analyzer --profile ci-test`: pass after
  the same coverage evidence propagation, serde naming changes, and unlinked
  diagnostic summary change. This includes the unit edge
  `semantic_diagnostic_without_primary_span_is_not_linked_to_a_file` and 17 CLI
  integration tests.
- `rg "CoverageEvidence::from_coverage_entries"
  experiments/rust-main/lumin-rust-analyzer/src`: one product-layer lowering
  site in `product_artifact/build.rs`. Policy-summary raw coverage scans remain
  for degraded examples and unavailable-entry counts.
- `rg -F '#[serde(rename = '
  experiments/rust-main/lumin-rust-analyzer/src/policy/action
  experiments/rust-main/lumin-rust-analyzer/src/policy/types.rs`: no remaining
  field-by-field action-tier, oracle-confidence, or action-reason rename
  repeats. Remaining explicit renames are semantic exceptions:
  `oracle-covered`/`oracle-partial`/`oracle-unavailable`/`oracle-missing` and
  `total`.
- `summary.semanticUnlinkedDiagnostics` is present in the unified product
  artifact. The current unified fixture reports `1`, making the existing
  top-level diagnostic without a per-file ref visible instead of silently
  disappearing from the file projection.
- `policy/action/projection/gates.rs` and `policy/action/projection/sections.rs`
  were removed after the decomposition audit. The gate projection structs now
  live beside `ActionPolicyProjection` in `model.rs`, and the constant-backed
  constructors live beside the lowering code in `build.rs`.
- `policy/evidence/coverage/bridge.rs` was removed after the same audit. The
  coverage bridge entry projection now lives with `CoverageEvidence` in
  `coverage.rs`, while support and taint evidence keep their separate
  vocabulary modules.
- Public API doctests were deliberately not added for `run_oracle` and
  `analyze_root`. `run_oracle` is exercised by the oracle integration harness,
  while `analyze_root` is exercised through the wrapper CLI artifact path that
  calls it. Compile-only doctests would prove import surface trivia rather than
  product behavior.

The final command matters for the dependency-boundary review. `windows-sys`
still appears in the full workspace normal graph through `ra_ap_syntax`, which
is the Rust AST parser lane. It no longer appears in the `rust-cargo-oracle`
production graph through `tempfile`.

## Integration Targets

`cargo metadata --format-version 1 --no-deps` shows one integration test target
per Rust product crate:

| Crate | Integration target |
| --- | --- |
| `lumin-rust-analyzer` | `tests/integration.rs` |
| `rust-cargo-oracle` | `tests/integration.rs` |
| `rust-source-health` | `tests/integration.rs` |
| `topology-scanner` | `tests/integration.rs` |

This keeps the behavior tests while avoiding one compiled test binary per small
scenario file.

The targeted workspace fixtures have one active owner:
`tests/support/fixtures/targeted_workspace/*`. Legacy wrapper files
`tests/support/fixtures/broad_targeted_workspace.rs` and
`tests/support/fixtures/two_package_targeted_workspace.rs` were removed because
they were no longer connected through `tests/support/fixtures.rs`.

Single-package scenario setup also has one active owner:
`tests/support/scenarios/single_package.rs`. It covers metadata-only,
cargo-check, targeted-cargo-check, and targeted-with-integration setup while the
assertions remain in the behavior-specific test modules.

The unified artifact oracle-bridge contract assertions also have one owner:
`tests/support/artifact_contract/oracle_bridge/mod.rs`. The former
`coverage.rs` and `projection.rs` files were removed because they only split two
small assertions from the same bridge contract.

The unified artifact summary bridge assertions live in
`tests/support/artifact_contract/summary/mod.rs`. The former summary
`bridge.rs` file was removed because it only checked two bridge fields from the
same summary contract.

## Target Size Snapshot

Current local `experiments/target` directory sizes after the latest verification:

| Directory | Size |
| --- | ---: |
| `ci-test` | 978.5 MiB |
| `debug` | 409.0 MiB |
| `release` | 175.0 MiB |
| `dev-small` | 86.9 MiB |

The large file classes are `.exe`, `.rlib`, and `.rmeta`, not PDB files. The
remaining workspace-level `windows-sys` weight comes from `ra_ap_syntax`, so
removing it would undermine the Rust AST dogfooding path.

The `codex-main/codex-rs` workspace keeps the same broad pattern: one Rust
workspace target directory, size-aware dev/test profiles, and a distinct
release/profiling split. This lab keeps `ci-test` stricter than that reference
workspace for local disk pressure (`debug = "none"`, `strip = "symbols"`), and
adds an explicit stripped thin-LTO release profile for product binaries.

Release binary sizes from `cargo build --locked --workspace --release`:

| Binary | Size |
| --- | ---: |
| `lumin-rust-analyzer.exe` | 2.53 MiB |
| `lumin-rust-source-health.exe` | 1.28 MiB |
| `lumin-rust-cargo-oracle.exe` | 1.22 MiB |
| `lumin-topology-scanner.exe` | 0.32 MiB |

## Large Repo Metadata-Only Snapshot

Benchmark root:
`C:\Users\endof\Downloads\repo\suyeonevo\codex-main\codex-rs`.

Evidence artifacts are under:
`C:\Users\endof\Downloads\lumin-perf-lab\review\rust-perf-codex-main`.

The large-repo bottleneck was not output JSON size. The unified analyzer output
stayed about 6 MiB. The slow path was `metadata-only` semantic setup loading the
full Cargo dependency graph before producing unavailable semantic coverage.

`metadata-only` now requests workspace-only Cargo metadata with `--no-deps`.
Modes that actually run the compiler oracle (`cargo-check` and
`targeted-cargo-check`) keep full dependency metadata.

| Run | Before | After |
| --- | ---: | ---: |
| Analyzer wall time | 30241 ms | 8161 ms |
| Analyzer syntax phase | 6658 ms | 4565 ms |
| Analyzer semantic phase | 23293 ms | 2930 ms |
| Analyzer output size | 6.07 MiB | 6.02 MiB |
| Oracle-only metadata-only wall time | 15373 ms | 5262 ms |

The analyzer still scanned 2406 Rust files and reported 58030 opaque AST
surfaces. This change preserved the syntax dogfooding surface and reduced the
unrequested semantic setup cost.

## Large Repo Targeted Oracle Cache Snapshot

`targeted-cargo-check` uses a Cargo target directory because it runs rustc. The
default remains `--cargo-target-dir-mode isolated-temp`, which removes its owned
temporary target directory when the oracle run exits. For dogfooding larger Rust
workspaces, callers may opt into `--cargo-target-dir-mode reusable-temp`; this
uses a Lumin-owned temp target cache instead of the analyzed repo's `target/`
directory.

Latest codex-main dogfood:

- Command root:
  `C:\Users\endof\Downloads\repo\suyeonevo\codex-main\codex-rs`.
- Artifact:
  `C:\Users\endof\Downloads\lumin-perf-lab\review\codex-main-targeted-cap3-reusable-temp-2.json`.
- Mode: `targeted-cargo-check` with `--targeted-package-cap 3` and
  `--cargo-target-dir-mode reusable-temp`.
- Artifact size: 1,619,380 bytes.
- `oraclePlan.status`: `ran`.
- Cargo event stream parse status: `complete`.
- Selected packages: `codex-ansi-escape`, `codex-async-utils`,
  `codex-aws-auth`.
- Timings in artifact: analyzer 29,665 ms, syntax 16,882 ms, semantic
  12,780 ms.
- Reusable target cache:
  `C:\Users\endof\AppData\Local\Temp\lumin-rust-cargo-oracle-reusable-target-3d9027da48037038`.
- Reusable target cache size after this run: 596.67 MiB.
- The analyzed repo's existing `codex-rs\target` directory was not updated by
  this run; its observed last-write time stayed at 2026-06-19 01:19:22.

This is intentionally target-cache reuse, not semantic-result cache reuse.
Artifacts still expose the run mode and target directory through
`meta.input.cargoTargetDirMode` and `meta.input.cargoTargetDir`, while
`analysisInputSetComplete` remains false.

## Workspace Notes

- `experiments/Cargo.toml` owns Rust workspace members, shared dependency
  versions, clippy lints, size-oriented test profiles, and release/profiling
  binary profiles.
- `experiments/.cargo/config.toml` disables MSVC debug linker output for local
  Windows builds and owns the canonical Rust workspace verification aliases.
- `experiments/Cargo.lock` is the workspace lockfile. The old per-crate
  lockfiles are intentionally gone.
- `tempfile` is a dev-dependency for fixtures. The production oracle command
  runner uses a small RAII wrapper over `std::fs::OpenOptions`.

The main/reference repository was not used for this verification.

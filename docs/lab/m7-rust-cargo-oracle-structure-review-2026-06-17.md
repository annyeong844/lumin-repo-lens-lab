# M7 Rust Cargo Oracle Structure Review - 2026-06-17

## Scope

This note reviews the Rust-side M7 cargo oracle structure after the first
implementation and follow-up refactors. The change is intentionally structural:
move `rust-cargo-oracle` from a single large root toward a typed,
module-owned shape inside the shared Rust workspace.

The reviewed implementation lives under:

- `experiments/rust-main/rust-cargo-oracle/`
- `experiments/rust-sidecar/rust-source-health/`
- `experiments/rust-main/lumin-rust-analyzer/`
- `experiments/rust-common/`
- `experiments/Cargo.toml`
- `canonical/oracle-registry.json`
- `canonical/evidence-ladder.md`
- `docs/lab/m7-semantic-oracle-design-2026-06-17.md`
- `tests/fixtures/m7-cargo-json-diagnostic-capture-v4/`

## Result

Decision: `m7-rust-cargo-oracle-structure-ready-for-review`.

The important review finding from the previous packet was correct: the first
`rust-cargo-oracle` implementation had a healthy classifier model, but the
crate root carried too many unrelated responsibilities. That is now corrected.
`lib.rs` is a public facade. `driver.rs` owns one oracle run, and the domain
modules own artifact assembly, schema, classification, cargo process execution,
metadata ownership, scope resolution, input hashing, and toolchain probing.

The current stopping point is deliberate. More slicing would be decomposition
for its own sake. The remaining splits follow owner boundaries; tiny helper
fragments without a domain owner have already been folded back into their
parent modules.

## Structural Outcome

Current module split:

| Module | Responsibility |
| --- | --- |
| `experiments/Cargo.toml` | Workspace owner for Rust members, dependencies, clippy lints, and size-oriented profiles. |
| `rust-common` | Shared CLI usage errors, repo/path helpers, atomic JSON writes, and SHA-256 helpers. |
| `lib.rs` | Public facade and re-export surface for `run_oracle`, options, and protocol types. |
| `driver.rs` | Orchestrates one cargo oracle run and writes the artifact when output is configured. |
| `protocol.rs` / `protocol/*` | Typed `semantic-health.v1` schema, claim kinds, rules, enums, spans, coverage, findings, and summaries. |
| `artifact.rs` / `artifact/*` | Builds findings, diagnostics, safe actions, coverage, summary, command provenance, and artifact meta. |
| `classify.rs` / `classify/*` | Normalizes cargo diagnostics and assigns confidence/claim/rule. |
| `command.rs` / `command/*` | Builds and runs cargo/rust commands with timeout and output capture handling. |
| `cargo_json.rs` / `cargo_json/*` | Parses cargo JSON stream records and preserves stream parse evidence. |
| `metadata.rs` / `metadata/*` | Owns cargo metadata structs and package selection. |
| `ownership.rs` / `ownership/*` | Resolves user-code, dependency, generated, and unknown span ownership. |
| `scope.rs` / `scope/*` | Builds package/target/feature/cfg/target-triple coverage scope. |
| `config.rs` / `config/*` | Discovers and reads cargo config inputs. |
| `input_hash.rs` | Builds the analysis input set hash from files, env, config, and args. |
| `rustc_span.rs` / `rustc_span/*` | Converts raw rustc span JSON into project-owned primary spans and safe-edit evidence. |
| `rustc_diagnostic.rs` / `rustc_diagnostic/*` | Owns rustc diagnostic shape and E-code normalization. |
| `toolchain.rs` | Captures cargo/rustc toolchain metadata. |
| `util.rs` | Owns oracle-local timestamp formatting only. |
| `path_util.rs` | Owns oracle-local path normalization and containment helpers. |

Largest current oracle production files:

| File | Lines |
| --- | ---: |
| `classify/rules.rs` | 138 |
| `rustc_span/raw.rs` | 131 |
| `artifact.rs` | 126 |
| `ownership/resolver.rs` | 125 |
| `input_hash.rs` | 118 |
| `main.rs` | 113 |
| `rustc_diagnostic.rs` | 113 |
| `cargo_json/record/kind.rs` | 112 |
| `artifact/coverage/absence.rs` | 111 |
| `toolchain.rs` | 110 |
| `config.rs` | 108 |
| `artifact/findings.rs` | 108 |
| `classify/ledger.rs` | 106 |
| `artifact/summaries.rs` | 105 |
| `driver.rs` | 101 |

This is a better stopping point than further slicing. The remaining files are
cohesive domain modules, not miscellaneous buckets.

## Codex-Main Benchmark

`C:\Users\endof\Downloads\repo\suyeonevo\codex-main\codex-rs` is the benchmark
for this structural pass. Its Rust workspace uses the same broad pattern:

- a workspace manifest that centralizes members, internal dependencies, lints,
  and profiles;
- many small internal crates where the boundary is a real product or utility
  owner;
- local modules inside each crate that group by domain owner rather than by
  one-function helper fragments.

The current Lumin Rust workspace follows that pattern:

- `experiments/Cargo.toml` centralizes the Rust crates and shared dependency
  versions.
- `lumin-rust-common` is justified because it removes repeated
  `usage_error`/repo-root/path/hash/atomic-write logic across crates.
- `dev-small`, `test`, and `ci-test` profiles intentionally reduce debug
  output because artifact size and `target/` growth are product concerns here.

The benchmark does not justify flattening `lumin-rust-common` back into each
crate, and it does not justify making every protocol vocabulary file bigger.
It does justify stopping helper sprawl. If a new file has no stable product or
domain owner, do not create it.

## Test Binary Footprint

Integration tests are now consolidated by crate:

| Crate | Integration target |
| --- | --- |
| `lumin-rust-analyzer` | `tests/integration.rs` |
| `rust-cargo-oracle` | `tests/integration.rs` |
| `rust-source-health` | `tests/integration.rs` |
| `topology-scanner` | `tests/integration.rs` |

This keeps behavior coverage while reducing test-binary fanout. The point is
not to shrink tests by replacing behavior with scaffolding; the point is to stop
each small scenario file from becoming a separate compiled integration binary.

## Follow-Up Hardening

The post-structure review found a few operational risks around provenance and
process execution. The current packet closes the merge-critical ones:

- `command.rs` drains stdout and stderr while cargo is still running, so large
  cargo JSON output cannot fill the pipe and become a false timeout.
- `cargo metadata` no longer runs with `--no-deps`, preserving local path
  dependency package roots for ownership resolution.
- `cargo metadata` now receives the same explicit feature selection as
  `cargo check`, and unit tests lock the command argument construction without a
  fake process.
- cargo config discovery now follows Cargo precedence for the supported subset:
  current/deeper config before parent config, and extensionless `.cargo/config`
  before `.cargo/config.toml` in the same directory.
- malformed one-character quoted cargo config values are ignored instead of
  panicking the parser.
- `classify.rs` keeps the full cargo event while summarizing diagnostics and
  uses event `package_id` to prevent non-selected package diagnostics from
  becoming user-code findings.
- when cargo metadata is unavailable, root `src/` diagnostics can still classify
  as user-code while dependency-shaped paths do not get promoted blindly.
- `classify.rs` preserves already-emitted cargo JSON messages even when the
  command times out; timeout only blocks clean coverage, not emitted evidence.
- `scope.rs` reads target evidence from top-level cargo event `target` first,
  matching the observed `compiler-message` JSON shape.
- finding and coverage command provenance now includes the actual configured
  cargo binary, not the literal string `cargo`.
- the CLI finds the default registry root from `--root` when `--repo-root` is
  not supplied.
- `rust-source-health` stdin mode now rejects duplicate file paths and mismatched
  `sha256`/`text` pairs instead of copying untrusted hashes into artifacts.
- `rust-source-health` now maps usage/config failures to exit code 2 in both
  stdin sidecar mode and Rust wrapper CLI mode, preserving the legacy wrapper
  contract without emitting a JSON artifact.
- cargo target scope now ignores non-selected package cargo events, so dependency
  compiler-message events cannot replace the selected package target in the
  declared clean scope.
- artifacts explicitly mark `analysisInputSetComplete: false` and list missing
  influence kinds, keeping the hash as provenance instead of a reusable cache
  key.
- regression tests now cover cargo config and `RUSTFLAGS` changes affecting the
  analysis input hash.
- unit tests now cover cargo metadata feature argument construction directly.
- absence-clean coverage now requires an explicit `build-finished` event with
  `success: true`; malformed completion cannot prove clean.
- rustc E-code normalization now follows the registry contract
  `^E[0-9]+$`, not only the currently common four-digit code shape.
- multi-target fallback scope uses `target: "<multiple>"` instead of choosing an
  arbitrary first lib/bin target.

## Review Questions

1. Does the current `artifact/*` split still match product-owner boundaries, or
   has any file become a one-off helper bucket?
2. Are `ClaimKind` and `ClassificationRule` now sufficiently tied to
   `canonical/oracle-registry.json` for the M7 first slice?
3. Is `cacheReuse.policy = no-reuse-unless-complete-influence-set-is-captured`
   still honest enough while `analysisInputSetHash` remains a conservative but
   incomplete provenance identity?
4. Should `cfgSetComplete: false` prevent any production-facing "clean"
   rendering, or is the current scoped wording enough for experimental output?
5. Should the workspace keep `ci-test` debug output at `none`, or is there a
   narrower profile split that preserves enough local debugging without growing
   `target/` back toward multi-GB default builds?

## What This Closes

The refactor directly addresses these review points:

- `lib.rs` is no longer a 1500+ line single module.
- Artifact output is typed through `protocol.rs` instead of ad hoc top-level
  `serde_json::Value` construction.
- `lumin-rust-analyzer/src` no longer uses `serde_json::Value` for product
  artifact projection; the unified product layer is typed through
  `product_artifact/*`, `product_files/*`, `product_summary.rs`, and
  `policy/*`.
- Remaining `serde_json::Value` in `rust-cargo-oracle/src` is confined to raw
  Cargo/rustc JSON decoding boundaries before data is lowered into
  project-owned protocol types.
- Summary counts are based on typed findings and diagnostics, not JSON pointer
  re-parsing.
- Shared usage-error, repo-root, path, hash, and atomic-write helpers have one
  owner in `lumin-rust-common` instead of drifting across binaries.
- CLI usage exits are keyed by the shared `UsageError` type through
  `is_usage_error` downcasting, not by matching error-message prefixes.
- `scope/*` no longer reads process environment directly while resolving target
  triples or cfg flags. `CompilationEnvironment` captures the relevant Cargo
  and Rust variables once, then passes that typed snapshot through the scope,
  cargo config, and input-hash paths.
- Cargo config handling uses `toml::Value` for the supported `[build] target`
  and `[build] rustflags` subset. There is no hand-rolled TOML parser in the
  oracle scope path.
- Cargo config discovery now takes only the crate root and captured compilation
  environment. It no longer accepts unused metadata just because scope
  construction has metadata nearby.
- `meta.output` is now present when output is configured, so the CLI can report
  the written file instead of dumping a full artifact by accident.
- Diagnostic claim/rule vocabulary is represented by enums, and the registry
  contract test checks emitted classifier rules against the registry.
- `PrimarySpan`, `PrimarySpanExpansion`, and `PrimarySpanLocation` are
  project-owned typed protocol structs with one camelCase JSON contract.
- Safe-action eligibility, blockers, and edits now come from one
  `safe_action_analysis` pass before being lowered into `SafeActionDecision`.
  The earlier risk of separate blocker and edit passes drifting apart is gone.
- `ActionPolicy` no longer stores a JSON `Value` beside duplicate scalar
  fields. The policy keeps typed counts and examples, then lowers once into the
  serializable projection.
- Action tier, oracle-confidence, and action-policy reason projection strings
  now live on serde `rename_all` rules where the vocabulary has a consistent
  case convention. Field-by-field `SAFE_FIX`/`REVIEW_FIX`, lowercase
  confidence, and kebab-case reason repeats were removed without changing the
  serialized artifact contract.
- The unified analyzer lowers oracle coverage entries into `CoverageEvidence`
  once per artifact build and reuses that typed view for the top-level oracle
  bridge, per-file bridge, and per-finding bridge projections. The remaining
  raw coverage scans are policy-summary scans for degraded examples and
  unavailable-entry counts, not duplicate bridge lookup logic.
- The action-policy projection split was checked for over-decomposition.
  `policy/action/projection/gates.rs` and `sections.rs` had no independent
  owner beyond two projection fields and their constant-backed constructors, so
  they were folded back into `model.rs` and `build.rs`. The remaining
  projection files own real sections: examples, reason counts, model shape, and
  lowering.
- The coverage bridge projection split was also checked. `policy/evidence`
  still owns coverage/support/taint evidence as separate vocabulary families,
  but `policy/evidence/coverage/bridge.rs` was only a private shape for the two
  coverage entries produced by `CoverageEvidence`, so it was folded into
  `coverage.rs`.
- Semantic diagnostics that cannot be attached to a file projection are no
  longer silent. The full diagnostics remain in top-level
  `semanticDiagnostics[]`, and `summary.semanticUnlinkedDiagnostics` reports how
  many diagnostics lacked a file ref in `files[*].semantic.diagnostics[]`.
- Rule-backed rustc lint diagnostics no longer claim verified authority IDs.
- Path normalization has a single owner inside the cargo oracle crate.
- Production Rust paths are panic-free for this packet after excluding
  `#[cfg(test)]`/test support modules: no `unwrap()`, `expect()`, `panic!`,
  `todo!`, `dbg!`, or `unsafe` blocks remain in the analyzer, oracle,
  source-health, topology-scanner, or common product sources.
- Workspace-level profiles move Rust build output toward the codex-main pattern
  while being stricter about debug-size pressure for this lab.
- Integration tests keep real behavior/edge/hard-stop coverage while reducing
  unnecessary compiled test target fanout.
- Test support fixture ownership was checked after the integration target
  consolidation. The active targeted-workspace fixture owner is
  `tests/support/fixtures/targeted_workspace/*`; unused legacy wrapper files
  under `tests/support/fixtures/` were removed instead of keeping parallel
  fixture entrypoints.
- Single-package scenario setup now has one owner in
  `tests/support/scenarios/single_package.rs`. The metadata-only, cargo-check,
  targeted-cargo-check, and targeted-with-integration variants still drive
  separate product behaviors, but they no longer repeat the same temp repo and
  single-crate fixture setup across four tiny files.
- The unified artifact oracle-bridge contract test now has one owner in
  `tests/support/artifact_contract/oracle_bridge/mod.rs`. The former
  `coverage.rs` and `projection.rs` files only split two tiny bridge assertions,
  so they were folded back into the bridge contract owner.
- The unified artifact summary bridge assertions were also folded into
  `tests/support/artifact_contract/summary/mod.rs`. The former `bridge.rs`
  file only checked two summary bridge fields and did not have an independent
  contract owner.
- The oracle command runner no longer carries `tempfile` in the production
  dependency graph; tests still use it as a fixture utility.
- `rust-source-health` file analysis now has one syntax tree traversal owner:
  `collect_file_syntax` loops over `root.descendants()` once and dispatches
  each node through `collect_syntax_node` for facts, signals, AST facts, method
  calls, macro calls, cfg gates, and opaque surfaces.
- The earlier `metadata.rs` dead-code allowance is gone; cargo metadata
  deserialization now carries only fields used by selection, ownership,
  scope, and input-hash paths.

## External Feedback Disposition

This table tracks the external review feedback against the current tree. It is
not a success declaration for the whole Rust migration; it is the disposition
for this structure packet.

| Feedback item | Disposition | Current evidence |
| --- | --- | --- |
| `lumin-rust-analyzer` drops typed inputs into `serde_json::Value` and re-parses string paths. | Closed. | `lumin-rust-analyzer/src` has no `serde_json::Value`, `json!`, `value_at`, `require_equal`, or `require_present` matches. Product lowering is typed through `product_artifact/*`, `product_files/*`, `product_summary.rs`, and `policy/*`. |
| No Rust workspace; path dependencies force duplicate helpers. | Closed. | `experiments/Cargo.toml` owns the workspace and shared dependencies. `rust-common` owns usage errors, repo-root/path helpers, SHA-256, and atomic JSON writes. |
| `PrimarySpan` mixes snake_case and camelCase in one JSON object. | Closed. | `rust-cargo-oracle/src/protocol/span.rs` owns `PrimarySpan`, `PrimarySpanExpansion`, and `PrimarySpanLocation` with `#[serde(rename_all = "camelCase")]`. |
| Protocol still has broad untyped `Value` holes for oracle plan, scope, spans, and raw diagnostics. | Closed with boundary exceptions. | `protocol/`, `artifact/`, `oracle_plan.rs`, and `scope/` have no `serde_json::Value` or `json!` matches. Remaining `Value` use is confined to raw Cargo/rustc JSON decoding and input-hash materialization before typed lowering. |
| Exit-code classification depends on error-message prefix matching. | Closed. | `lumin-rust-common::UsageError` is downcast by `is_usage_error`; analyzer, oracle, and source-health delegate to that shared type instead of local string matching. |
| The feedback specifically suggested `thiserror` for all library errors. | Intentional non-change. | The concrete bug was usage/runtime exit classification. A shared typed `UsageError` fixes that without adding a new dependency or forcing every runtime `anyhow` context into an enum before the public API stabilizes. |
| `push_semantic` silently drops diagnostics when file projection linking fails. | Closed. | Top-level `semanticDiagnostics[]` remains complete, and `summary.semanticUnlinkedDiagnostics` exposes diagnostics that could not be attached to `files[*].semantic.diagnostics[]`. |
| Safe-action blockers and safe edits are calculated by separate passes that can drift. | Closed. | `artifact/safe_action.rs` lowers exactly one `safe_action_analysis` result into either blockers or a `SafeAction`. |
| `ActionPolicy` stores duplicated JSON and scalar fields. | Closed. | `ActionPolicy` keeps typed counts/examples and lowers once into a serializable projection; it no longer stores a parallel JSON `Value`. |
| Coverage bridge entries are looked up twice in the policy layer. | Closed. | `CoverageEvidence` is built once per artifact build in `product_artifact/build.rs` and shared by bridge/file/finding projections. |
| `rust-source-health` walks the syntax tree multiple times. | Closed. | `collect_file_syntax` owns a single `root.descendants()` loop and dispatches each node through `collect_syntax_node`. |
| Production command output uses fragile temp-file cleanup. | Closed. | The oracle command runner uses a small RAII wrapper over `std::fs::OpenOptions`; `tempfile` remains a test-only fixture dependency. |
| Over-decomposition risk after the structural pass. | Closed for the current packet. | Thin action projection and coverage bridge modules were folded back into their parent owners; docs now explicitly reject more file splitting for line-count optics. |
| Public API doctests for `analyze_root` / `run_oracle`. | Intentional non-change. | `run_oracle` is exercised through the oracle integration harness, and `analyze_root` is exercised through the source-health wrapper CLI path that calls it and verifies the emitted artifact. Adding compile-only doctests would mostly prove import trivia, not product behavior. |

## Remaining Review Notes

- M7 fixture files are included in the staged change set as empirical cargo
  diagnostic evidence. Its `summary.json` was generated before the Rust M7
  implementation became the product artifact owner; do not treat that historical
  capture summary's span key spelling as the current `semantic-health.v1`
  contract. The current product span schema is owned by
  `rust-cargo-oracle/src/protocol/span.rs` and serializes project-owned spans as
  camelCase.
- `analysisInputSetHash` remains intentionally non-reusable. The artifact states
  `no-reuse-unless-complete-influence-set-is-captured`; consumers must not treat
  it as a safe cache key.
- The broader working tree still contains generated-review changes outside this
  Rust packet. Review this as the Rust migration packet, not as a small isolated
  Rust-only diff.
- `artifact/*`, `protocol/*`, `classify/*`, and `command/*` are intentionally
  split by owner. Do not split them again for line-count optics. Split only when
  a new durable product concept needs a real owner.
- `tempfile` remains useful in tests, but production command output capture is
  now a small RAII wrapper over `std::fs::OpenOptions`, so the product path does
  not pull `tempfile -> windows-sys`.

# Rust `.mjs` Replacement Matrix

Status: migration control spec
Owner: Rust migration track
Date: 2026-07-01

## Purpose

This document prevents the wrong migration: rewriting every `.mjs` file because
the repo is moving more analysis into Rust.

The migration target is narrower:

1. Move Rust-language product analysis to Rust owners.
2. Keep JS/TS-language behavior on the checked JS/TS owners until Rust has an
   equivalent parser, resolver, and artifact contract for that language lane.
3. Delete or deprecate `.mjs` only after the product route no longer depends on
   that owner.

Rust migration means "replace the owner for a product behavior." It does not
mean "translate every file extension."

## Inventory

Checked command:

```powershell
rg --files -g '*.mjs' -g '!node_modules' -g '!experiments/target'
```

Current `.mjs` surface:

| Surface | Count | Migration meaning |
|---|---:|---|
| Root engine scripts | 33 | Product and maintainer entrypoints. Replace only from the outside in. |
| Root `_lib/` modules | 120 | Main JS/TS engine owners and shared helpers. Replace by behavior owner, not by filename. |
| `skills/lumin-repo-lens-lab/_engine/` | 151 | Generated distribution mirror. Not an independent source owner. |
| `tests/` | 352 | Verification surface. Do not mass-port just to reduce `.mjs` count. |
| `test-harness/` | 5 | Maintainer harness. Not a product engine owner. |
| `scripts/` | 12 | Packaging/build scripts. Keep unless the packaging contract changes. |
| Other | 11 | Config or local support surfaces; classify case by case. |

Source-owner scope for this matrix is root scripts plus root `_lib/`.
Generated skill-package copies inherit the source decision.

## Status Labels

| Status | Meaning |
|---|---|
| Rust-owned for Rust lane | Rust is or should become the product owner for Rust input. JS/TS code may remain the owner for JS/TS input. |
| Partial Rust owner | Rust owns part of the lane, but deletion would hide behavior or evidence. |
| JS/TS owner retained | Behavior is JS/TS-specific or orchestration-level. Do not port yet. |
| Utility candidate | Port only when a Rust caller needs the helper. No standalone migration. |
| Distribution/test surface | Do not count as product owner. |

## Replacement Matrix

| Behavior | Current JS/TS owner | Rust destination | Status | Rule |
|---|---|---|---|---|
| Rust syntax health, signal summaries, generated/test path policy | `_lib/extract-*`, syntax fact consumers, audit wrappers | `experiments/rust-sidecar/rust-source-health` | Rust-owned for Rust lane | Rust source-health owns Rust syntax facts. JS/TS extractors still own JS/TS syntax facts. |
| Rust function clone groups, signature groups, near candidates | `build-function-clone-index.mjs`, `_lib/function-clone-artifact.mjs`, `_lib/function-signature-hash.mjs` | `rust-source-health/src/function_clones*`, `src/analyzer/syntax/items/function_bodies*`, `src/analyzer/syntax/items/functions/signature.rs` | Rust-owned for Rust lane | Rust artifact may replace JS clone output for Rust files only. JS/TS clone artifact remains the TS/JS language owner. |
| Rust shape/signature lookup for pre-write | `build-shape-index.mjs`, `_lib/shape-hash.mjs`, `_lib/shape-index-*` | `rust-source-health` AST facts plus `lumin-rust-analyzer/src/prewrite/lookup/shape.rs` | Rust-owned for Rust lane | Rust shapes/signatures replace JS shape lanes only for Rust pre-write intents. |
| Rust unused definition / dead export evidence | `classify-dead-exports.mjs`, `_lib/classify-facts*.mjs`, `_lib/export-action-safety.mjs`, parts of `rank-fixes.mjs` | `rust-source-health/src/dead_exports.rs`, `src/protocol/unused_definitions.rs` | Partial Rust owner | Rust may own raw evidence and RUST-FP gates. Positive remove candidates require dogfood-proven reachability before any JS owner is retired. |
| Cargo/rustc semantic oracle and safe/review tiers | TS/JS uses `tsc`/OXC-derived evidence; no Cargo equivalent | `experiments/rust-main/rust-cargo-oracle`, `lumin-rust-analyzer` policy bridge | Rust-owned for Rust lane | Cargo artifacts are Rust-only necessity. They must remain artifact-visible and must not introduce timeouts or repo-size caps. |
| Rust pre-write dependency/file/shape/signature/local/service cues | `pre-write.mjs`, `_lib/pre-write-*` | `lumin-rust-analyzer/src/prewrite*` | Rust-owned for Rust lane | Rust pre-write can replace JS pre-write only for Rust intent lanes whose artifact contracts are typed and dogfooded. |
| Rust topology sidecar | `measure-topology.mjs`, `_lib/rust-topology-*` | `experiments/rust-sidecar/topology-scanner` | Partial Rust owner | Topology scanner is a sidecar, not a full replacement for JS topology/reporting. |
| Symbol graph, module reachability, call graph, resolver diagnostics | `build-symbol-graph.mjs`, `build-module-reachability.mjs`, `build-call-graph.mjs`, `resolve-method-calls.mjs`, `_lib/*resolver*`, `_lib/module-reachability.mjs`, `_lib/call-graph-bounded.mjs` | None yet | JS/TS owner retained | These depend on JS/TS module, package, SFC/MDX, and resolver semantics. Do not replace until Rust has that language contract. |
| OXC parsing, tsconfig/package exports, SFC/MDX consumers, public JS package surface | `_lib/parse-oxc.mjs`, `_lib/tsconfig-paths.mjs`, `_lib/package-exports.mjs`, `_lib/sfc-consumers.mjs`, `_lib/mdx-consumers.mjs`, `_lib/public-surface.mjs` | None yet | JS/TS owner retained | These are not Rust-code analysis. Keep them until a Rust JS/TS parser/resolver lane exists. |
| Audit orchestration and lifecycle commands | `audit-repo.mjs`, `pre-write.mjs`, `post-write.mjs`, `generate-canon-draft.mjs`, `check-canon.mjs`, `emit-sarif.mjs`, `merge-runtime-evidence.mjs` | Unified Rust analyzer may feed evidence, but not replace orchestration yet | JS/TS owner retained | Orchestrators are outer product surfaces. Replace after leaf analyzers and artifact contracts are stable. |
| Shared file/artifact utilities | `_lib/atomic-write.mjs`, `_lib/artifacts.mjs`, `_lib/paths.mjs`, `_lib/collect-files.mjs`, `_lib/incremental*.mjs` | `experiments/rust-common`, Rust wrapper/cache modules | Utility candidate | Port only where Rust code consumes the helper. Do not create a parallel utility crate for unused helpers. |
| Skill package build and maintainer scripts | `scripts/*.mjs`, `eslint.config.mjs`, `vitest.config.mjs` | None | Distribution/test surface | Keep unless the package/build contract itself moves. |
| Tests and harnesses | `tests/*.mjs`, `test-harness/*.mjs` | Rust tests for Rust behavior only | Distribution/test surface | Do not mass-rewrite JS tests. JS/TS behavior keeps JS tests; Rust behavior gets Rust product tests. |

## First Migration Candidates

The next Rust migration work should follow this order.

1. **Rust source-health product route**

   Route Rust audit evidence through `lumin-rust-source-health` compact artifacts
   and unified analyzer summaries. This is already the strongest Rust owner:
   syntax facts, clone groups, signature groups, compact cache, and unused
   definition raw evidence all have Rust owners.

2. **Rust function clone and signature deprecation for Rust files**

   Mark JS function-clone artifacts as JS/TS-language owners, not Rust owners.
   Rust files should use Rust clone groups. The JS clone engine stays because it
   still owns JS/TS function clone behavior.

3. **Rust dead-export evidence, not unsafe removal**

   Keep Rust unused-definition output as evidence first. Do not promote private
   remove candidates into a strong action until reachability is dogfooded on
   large repositories and macro/nested/reference cases are covered. Public
   surfaces, trait/impl surfaces, cfg, derive, opaque macro surfaces, and FFI
   remain RUST-FP gates.

4. **Rust pre-write shape/signature/local cues**

   Rust pre-write lanes should consume typed Rust source-health facts directly.
   Do not route them through serialized JS-style shapes when typed Rust owners
   exist.

5. **Rust cargo oracle integration**

   Cargo/rustc semantic evidence has no TS/JS equivalent. Keep it Rust-owned,
   artifact-visible, and free of elapsed-time caps.

## Deletion Gate For Any `.mjs`

No `.mjs` source owner should be deleted until all of these are true:

1. A Rust owner is documented in canonical docs.
2. The Rust artifact field map is explicit: old field, new field, changed
   semantics, and any omitted scope.
3. Dogfood has covered at least `ripgrep`, `codex-rs` or another large Rust
   workspace, and one small focused fixture for the edge being migrated.
4. The generated skill package still contains every referenced runtime file.
5. JS/TS behavior is either still owned by JS/TS or explicitly out of scope.
6. Tests prove product behavior, not module existence or fixture trivia.
7. The JS owner is deprecated before deletion unless it was already unreachable.

## Non-Goals

- Do not rewrite all `.mjs` files.
- Do not merge the JS test-harness split just to make the repository look more
  Rust-shaped.
- Do not delete `skills/lumin-repo-lens-lab/_engine/*.mjs` by hand; it is a
  generated distribution surface.
- Do not replace JS/TS parser, resolver, SFC, MDX, package export, or public
  surface behavior until Rust has a checked language owner for those semantics.
- Do not add Rust-only thresholds, caps, or policy constants unless canonical
  docs explain why TS/JS does not need them and why Rust does.

## Current Decision

The next work is Rust engine hardening and owner handoff, not a mass `.mjs`
translation.

The repo should move from leaf Rust analyzers outward:

1. Rust source-health and cargo-oracle produce stable typed evidence.
2. `lumin-rust-analyzer` consumes that evidence without stringly JS-shaped
   detours.
3. Product routes prefer Rust evidence for Rust files.
4. Only then do root `.mjs` owners get deprecated or deleted for the Rust lane.

This keeps the migration honest: Rust replaces behavior it actually owns, and
JS/TS keeps behavior Rust cannot yet prove.

## Current Handoff Audit: 2026-07-01

Checked source state:

- `build-function-clone-index.mjs` imports `JS_FAMILY_LANGS` from
  `_lib/lang.mjs` and builds snapshots with `languages: JS_FAMILY_LANGS`.
  `_lib/lang.mjs` defines `JS_FAMILY_LANGS` as
  `ts, tsx, mts, cts, js, jsx, mjs, cjs`; `.rs` is not included.
- `build-shape-index.mjs` uses the same `JS_FAMILY_LANGS` snapshot scope.
  The root JS shape producer is therefore the JS/TS language owner, not a Rust
  file owner.
- `lumin-rust-analyzer pre-write` is the Rust execution surface for Rust
  pre-write source intents. It consumes typed `rust-source-health` in memory
  and owns Rust name, file, exact shape-hash, function-signature, Cargo
  dependency, inline-pattern, and planned-type-escape declaration lanes.
- `lumin-rust-analyzer` unified product mode consumes typed Rust
  `rust-source-health` evidence through `SyntaxPhase`. Unless the caller
  explicitly requests `--source-health-profile full`, the default is the
  compact `analyze_root_compact` library response, not a JS-shaped serialized
  source-health artifact. Compact syntax now carries an uncapped
  compiler-oracle opacity count for targeted Cargo path selection. Full
  `HealthResponse` remains a Rust diagnostic/compatibility mode for raw AST
  inspection and precise semantic-finding opaque-overlap analysis.
- `pre-write.mjs` / `audit-repo.mjs --pre-write` remain JS lifecycle
  orchestrators. They may continue to package JS/TS advisory output, but
  `symbols.json`, `shape-index.json`, and `function-clones.json` must not be
  treated as Rust absence evidence.
- `audit-repo.mjs --pre-write --rust-pre-write` is the explicit public route
  for Rust source intents. `audit-repo.mjs --pre-write --pre-write-engine auto`
  may also enter the Rust route, but only when the intent transport explicitly
  declares `language: "rust"`. It does not infer Rust from filenames,
  dependencies, or repository shape. The route invokes
  `lumin-rust-analyzer pre-write` and records
  `preWrite.producer = "lumin-rust-analyzer"` in `manifest.json`.
  Generated packages must supply `LUMIN_RUST_ANALYZER_BIN` or run from a
  checkout that includes `experiments/Cargo.toml`; missing Rust
  analyzer support is a hard-stop, not permission to fall back to JS.
- `audit-repo.mjs --rust-analyzer` is the explicit public audit route for
  producing the unified Rust analyzer artifact from a normal audit run. It
  runs only when requested and only after `triage.json` has counted Rust files,
  writes `rust-analyzer-health.latest.json`, and records
  `manifest.rustAnalysis`. The default audit route still counts Rust files and
  records a Rust blind zone, but does not spend Cargo/Rust analyzer work unless
  this flag is present. A missing or failing analyzer remains artifact-visible;
  it is not a JS fallback.

Result:

- No code deletion is justified for `build-function-clone-index.mjs` or
  `build-shape-index.mjs`; they already scan JS-family inputs only.
- The owner handoff gap has narrowed past command selection. The explicit Rust
  route exists, `--pre-write-engine auto` routes from a checked language
  declaration without guessing, and the orchestrator wraps Rust-native
  pre-write output in the standard lifecycle advisory shape so post-write can
  consume the invocation/file-delta contract without treating JS artifacts as
  Rust absence evidence.
- Rust source-health product routing has also moved past raw JS-shaped
  artifact handoff for the unified analyzer: the checked default
  product path uses compact typed Rust evidence, including targeted Cargo
  oracle path selection, and the full syntax artifact is kept as an explicit
  Rust diagnostic compatibility path, not a JS/TS owner.

Checked on 2026-07-01 without running Node:

- `cargo fmt --package lumin-rust-source-health --package lumin-rust-analyzer --check`
- `cargo check --locked -p lumin-rust-source-health -p lumin-rust-analyzer`
- `cargo clippy --locked -p lumin-rust-source-health -p lumin-rust-analyzer --all-targets -- -D warnings`
- `cargo test --locked -p lumin-rust-source-health --test integration -- --nocapture`
  (`97 passed`)
- `cargo test --locked -p lumin-rust-analyzer --test integration -- --nocapture`
  (`112 passed`, including
  `unified_cli_uses_compact_source_health_by_default_for_metadata_only` and
  `unified_cli_preserves_full_source_health_diagnostic_mode`)

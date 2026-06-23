# canonical/rust-source-health.md

> **Role:** canonical naming, shape, helper, and module contract for the Rust source health track.
> **Owner:** this file.
> **Status:** M6 spine addition.
> **Last updated:** 2026-06-23

---

## 1. Why this exists

Rust source health must not grow a second private language inside the repo.

The failure mode is predictable: one worker writes `makeSignal`, another writes
`build_signal`, a third hand-builds a JSON shape, and a month later nobody knows
which one is truth. That is how clone disease starts.

This file is the canonical map. If a new Rust source health task needs a shape,
helper, name, enum, validator, runtime setting, or file boundary, check here
first. If the right name is missing, amend this file before implementation.

## 2. Authority

This file wins for the Rust source health track when it conflicts with an
implementation plan, review packet, or worker-local convenience helper.

It does not change the JS/TS topology scanner, M2-M5 Rust topology sidecar,
pre-write gate, SARIF output, markdown audit output, or stable package defaults.
Rust source health is now a syntax phase inside the unified Rust analyzer
surface. Its compatibility CLI may still emit `rust-health.json`, but the
product artifact is the unified Rust analyzer artifact. The compatibility CLI
defaults to a compact artifact that keeps summary, skipped-file evidence,
signals, parse status, file facts, and per-file AST counts while omitting raw
AST fact arrays. Full raw AST facts are diagnostic evidence and require
`--artifact-profile full`.

## 3. Naming Lowering

| Surface | Convention | Example |
|---|---|---|
| JSON field | `camelCase` | `signalsByKind`, `byteStart`, `unsafeBlocks` |
| JSON string enum / reason / kind | `kebab-case` | `unwrap-call`, `invalid-utf8`, `syntax-only` |
| CLI flag | `kebab-case` | `--source-commit`, `--worker-stack-bytes` |
| Rust module / function / field | `snake_case` | `review_signal`, `worker_stack_bytes` |
| Rust type / enum / struct | `PascalCase` | `FileHealth`, `RuntimeConfig` |
| Rust constant | `SCREAMING_SNAKE_CASE` | `PARSER_VERSION` |
| File path | `kebab-case` unless local convention already exists | `rust-source-health` |

Lowering examples:

| Concept | JSON | Rust |
|---|---|---|
| unwrap method signal | `unwrap-call` | `review_signal(SignalKind::UnwrapCall, ...)` |
| parse error | `parse.errors[]` | `syntax_parse_error(...)` |
| worker stack bytes | `workerStackBytes` | `worker_stack_bytes` |
| source hash | `sha256` | `sha256` |

## 4. Owned Protocol Boundary

All JSON-visible shapes are project-owned Rust structs. No
`ra_ap_syntax` type may cross into the protocol, public module surface, JSON
artifact, or validator.

Allowed:

- `pub(crate)` use of `ra_ap_syntax` inside parser/analyzer modules.
- A `pub(crate)` internal facade if it prevents import noise inside the Rust
  sidecar.

Forbidden:

- `pub use ra_ap_syntax::*` from any public module.
- Any protocol field typed as a `ra_ap_syntax` node, token, range, syntax kind,
  or parser error.
- Any JSON field named after a third-party crate type.

The protocol owns its names. The parser is an implementation detail.

## 5. Canonical Rust Modules

| File | Owns | Must not own |
|---|---|---|
| `src/protocol.rs` / `src/protocol/*.rs` | Request/response structs, schema constants, project-owned enums/strings | parser traversal, signal construction logic |
| `src/locations.rs` | `LineIndex`, byte-to-line/column conversion | signal kinds, summary counts |
| `src/signals.rs` | `review_signal(...)`, `syntax_parse_error(...)`, signal visibility policy application | parser traversal, summary counts |
| `src/summary.rs` | `summarize(...)` for `BTreeMap<String, FileHealth>` | signal construction, path policy |
| `src/parallel.rs` | local Rayon `ThreadPool`, `RuntimeConfig`, stack/thread policy | AST storage, file analysis |
| `src/analyzer.rs` / `src/analyzer/*.rs` | syntax traversal, file-level analysis, AST fact extraction, AST opaque surface detection | protocol schema changes, final artifact metadata |
| `src/lib.rs` / `src/driver/*.rs` | library phase entrypoint, stdin compatibility dispatch, request validation, pool install, exit behavior | parser traversal |
| `src/main.rs` | thin compatibility CLI entrypoint that delegates to `src/lib.rs` | parser traversal, request validation |
| `src/wrapper.rs` / `src/wrapper/*.rs` | Rust-main file discovery, path policy, hashing, UTF-8 decode, skipped-file evidence, final metadata, artifact write | parser traversal, signal construction |

No extra Rust module may create `Signal`, `ParseError`, `Summary`, `Location`,
or runtime pool settings unless this table is amended first.

## 6. JavaScript Boundary

Rust source health does not own a JavaScript wrapper surface anymore. It is a
Rust library phase with a thin compatibility CLI. The product execution surface
is the unified Rust analyzer. New `rust-source-health` `.mjs` wrappers are
forbidden unless this canonical file is amended with a migration reason.

## 7. Canonical Constructors And Helpers

### Rust

| Purpose | Canonical name | Owner |
|---|---|---|
| review signal construction | `review_signal(kind, line_index, range)` | `src/signals.rs` |
| signal muting | `mute_signal(signal, reason)` | `src/signals.rs` |
| signal visibility policy | `apply_signal_policy(signals, classifications)` | `src/signals.rs` |
| parse error construction | `syntax_parse_error(message, line_index, range)` | `src/signals.rs` |
| location conversion | `LineIndex::location(byte_start, byte_end)` | `src/locations.rs` |
| AST fact range conversion | `ast_location(line_index, range)` | `src/analyzer.rs` |
| file syntax collection | `collect_file_syntax(...)` | `src/analyzer/syntax.rs` |
| single-pass syntax node dispatch | `collect_syntax_node(...)` | `src/analyzer/syntax/visit.rs` |
| Rust record shape hash extraction | `collect_struct_shape_hash(...)` | `src/analyzer/syntax/items/shapes.rs` |
| artifact summary | `summarize(files)` | `src/summary.rs` |
| local Rayon pool | `build_pool(runtime_config)` | `src/parallel.rs` |
| unsafe block syntax check | `is_unsafe_block_expr(node)` | `src/analyzer.rs` |
| method call signal scan | `collect_method_call_signal(...)` | `src/analyzer.rs` |
| macro call signal scan | `collect_macro_call_signal(...)` | `src/analyzer.rs` |

`review_signal`, `syntax_parse_error`, and `ast_location` are the only
production helpers that convert `TextRange` into `Location`.

## 7.1 AST Fact Shape

Rust source health emits a project-owned `files[*].ast` object. This is the
Rust analogue of the JS/TS extractor shape: cheap syntax observations first,
then semantic oracles only where the syntax surface is opaque.

Canonical JSON fields:

- `ast.definitions[]`: named Rust item definitions with `kind`, `name`,
  `visibility`, and `location`.
- `ast.shapeHashes[]`: exact Rust AST shape-hash facts for supported
  non-generic named-field structs. The normalized form is Rust-owned and
  includes record field names, field visibility, and compacted Rust type text.
  Tuple structs, unit structs, generic structs, type aliases, and field-name-only
  intents remain unsupported until a checker-grade or explicitly documented
  producer exists.
- `ast.functionSignatures[]`: exact Rust callable signature facts for parsed
  top-level functions and `impl` methods. The normalized form is Rust-owned and
  includes callable kind, receiver kind/text, compacted generic params,
  parameter type text, and return type text. It does not include the function
  name or body, and it does not claim semantic equivalence.
- `ast.impls[]`: Rust `impl` block observations with `target`, optional
  `trait`, method owner evidence, and `location`. This is the Rust analogue of
  the JS/TS `classMethodIndex`: impl methods are visible as owner evidence
  without pretending AST-only analysis has type or trait-solving certainty.
- `ast.useTrees[]`: `use` tree observations with raw tree text, optional path,
  optional terminal `name`, optional `alias`, glob status, visibility, and
  `location`. Simple public re-export aliases such as
  `pub use crate::model::Thing as Alias` expose `name = "Thing"` and
  `alias = "Alias"` so Rust pre-write can mirror the TS/JS exported alias
  exact-name cue without parsing raw syntax text.
- `ast.pathRefs[]`: qualified expression- and type-position path references
  with raw path text, terminal name, and `location`. Local variable refs and
  constructor-like single-segment paths are not emitted as raw path facts.
- `ast.methodCallCounts`: per-file method-name counts for all observed method
  call sites.
- `ast.methodCalls[]`: review-relevant method call observations with method
  name, receiver text, and `location`. This is not an every-call-site dump.
- `ast.macroCalls[]`: macro call observations with path/name and `location`.
- `ast.cfgGates[]`: `cfg` / `cfg_attr` attributes with normalized expression
  text and `location`.
- `ast.opaqueSurfaces[]`: syntax surfaces where AST-only analysis must not
  pretend semantic certainty. Current kinds are `macro-expansion` and
  `cfg-gate`. Each surface carries `visibility` and optional `muteReason`,
  using the same practical review/muted discipline as Rust syntax signals.

AST opaque surfaces are evidence, not findings. They are the escalation map for
the unified analyzer: Cargo/rustc oracle evidence may clear or qualify them, but
the syntax phase must preserve them raw.

Opaque surface muting is not deletion. Common, low-review-value syntax opacity
is still auditable as `muted`: test/generated paths, direct test-only AST
contexts, assertion macros, collection literal macros, data literal macros,
formatting macros, IO formatting macros, logging macros, built-in derive
macros, and known data/schema derive macros such as `Serialize`, `Deserialize`,
`JsonSchema`, `TS`, `ExperimentalApi`, and qualified `prost::Message`. Risky or
unknown macro expansion remains `review`, including `panic!`, `todo!`,
`unimplemented!`, custom bang macros, unknown derive macros, and attribute/proc
macros. Test attribute macros such as `tokio::test` are test context and mute
the attribute plus syntax inside the function as `test-attribute`. Inert
compiler/lint/tool attributes such as `allow`, `warn`, `expect`, and
`rustfmt::skip` are not opaque expansion surfaces. Known derive helper
attributes such as `serde`, `schemars`, `ts`, `prost`, `clap`, `arg`,
`command`, and `thiserror` helpers like `error`, `from`, and `source` are not
standalone opaque expansion surfaces; the owning derive macro remains the
review or muted surface. Non-test `cfg` gates remain `review` because AST-only
analysis cannot know which branch is live.

### 7.2 Rust Pre-Write Consumer

`lumin-rust-analyzer pre-write` may consume
`HealthResponse::files[*].ast` in memory to answer declared Rust name intents,
and `HealthResponse::files` / `skippedFiles` to answer declared Rust file
intents. The analyzer owns intent, lookup, cue, and advisory policy.
`rust-source-health` remains the owner of raw AST extraction and path
classification.

The normal unified artifact must not embed a repository-wide definition or
impl-method index. The pre-write consumer builds a borrowed view and serializes
only matched advisory evidence. Impl methods remain separate owner evidence
and must not be promoted into definition-lane SAFE cues.
Public, crate, and restricted non-glob `useTrees` with a terminal name or alias
may enter exact-name lookup as claim-only SAFE cues. This is the Rust analogue
of TS/JS exported alias handling: it proves the Rust name is already surfaced by
source syntax, not semantic equivalence, auto-reuse, or auto-fix safety.

Rust file intent lookup is the Rust analogue of the JS/TS pre-write file lane:

- a requested path present in `HealthResponse::files` is `FILE_EXISTS`;
- a safe repo-relative `.rs` path absent from `HealthResponse::files` and
  `skippedFiles`, under the source-health path policy, is `NEW_FILE`;
- a path whose existing filesystem component is a symlink is
  `FILE_STATUS_UNKNOWN`, even if it is absent from `HealthResponse::files` and
  `skippedFiles`, because M6 source health does not follow symlinked files or
  directories;
- skipped files, non-Rust paths, excluded `target` / `vendor` paths, and unsafe
  path text are `FILE_STATUS_UNKNOWN`.

For safe repo-relative `.rs` intents, the file lane may emit the JS/TS
`domainCluster` watch cue from same-directory `HealthResponse::files` siblings.
The Rust cue follows the JS/TS token policy for domain prefixes, domain-token
matches, minimum match counts, and capped examples. Rust source health does not
currently expose file LOC in this lane, so `domainCluster.totalLoc` and example
`loc` values remain `null` rather than inventing a line-count claim.

The file lane does not evaluate boundary rules because Rust pre-write intent
does not carry planned `from -> to` edges. It must emit `NOT_EVALUATED`, matching
the JS/TS P1-2 behavior.

Rust shape intent lookup follows the JS/TS P4 discipline: it must not infer
structural equality from loose field names and must not add fuzzy shape
matching. Non-empty shape intents emit `coverage.shapes = "ran"` because Rust
source health now owns narrow exact-hash producers. A `shape.hash` matching
`HealthResponse::files[*].ast.shapeHashes[].hash` returns `SHAPE_MATCH`. A
`shape.hash` matching
`HealthResponse::files[*].ast.functionSignatures[].hash` returns
`SIGNATURE_MATCH`, mirroring the JS/TS `_lib/pre-write-lookup-shape.mjs`
`functionSignature` branch. Fields-only intents remain `UNAVAILABLE` because
field names alone are not structural equality evidence. `typeLiteral` without
an exact hash remains `UNAVAILABLE`; Rust must not parse TS/JS type literals in
this lane. An unmatched exact hash is also `UNAVAILABLE` for now, not
`NOT_OBSERVED`, because the Rust producer does not yet make complete absence
claims for every Rust shape or callable form. A positive exact-hash
`SHAPE_MATCH` may emit the JS/TS `shape-hash` `SAFE_CUE` as claim-only evidence.
A positive function-signature `SIGNATURE_MATCH` may emit the JS/TS
`function-signature` cue: top-level public/crate/restricted Rust functions may
be claim-only `SAFE_CUE`; private functions and all `impl` methods remain
`AGENT_REVIEW_CUE`. In all cases `notSafeFor` must preserve that the cue is not
semantic equivalence, auto-reuse, or auto-fix proof. No absence cue, fuzzy cue,
field-only cue, or `typeLiteral` cue may be emitted from this lane until a
checker-grade or explicitly documented Rust producer owns that evidence.

Rust dependency intent lookup is the Rust analogue of the JS/TS
`pre-write-lookup-dep.mjs` lane:

- `Cargo.toml` replaces `package.json` as the declaration source.
- `[dependencies]`, `[dev-dependencies]`, and `[build-dependencies]` replace
  `dependencies`, `devDependencies`, and `peerDependencies`.
- `HealthResponse::files[*].ast.useTrees`, `pathRefs`, and `macroCalls`
  replace `symbols.json.dependencyImportConsumers` as the static import graph.
- declared dependency plus one or more observed Rust path consumers is
  `DEPENDENCY_AVAILABLE`;
- declared dependency plus zero observed consumers is
  `DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS` only when the Rust syntax scan is
  complete enough to make a grounded zero-consumer statement;
- declared dependency with parse errors or skipped files is
  `DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE`, never zero observed;
- undeclared dependency is `NEW_PACKAGE`.

Cargo dependency declarations are package/member scoped. In a workspace,
`[workspace.dependencies]` is not a declaration for every member by itself; it
only counts for a member when that member inherits it with `workspace = true`
or declares the dependency directly. A declaration in one member must not make a
consumer in another member `DEPENDENCY_AVAILABLE`. When observed Rust path
consumers live in a member whose `Cargo.toml` does not declare or inherit the
dependency, pre-write reports `NEW_PACKAGE` for that consuming manifest scope
and cites the member manifest that lacks the declaration.

The Rust-only normalization from Cargo package key to code path crate root is
required because Cargo packages may use hyphens while Rust paths use
underscores, for example `tracing-subscriber` is imported as
`tracing_subscriber`. This is a language/package-model translation, not a new
policy. As in JS/TS, declared-with-zero-consumers must not be described as
unused or cleanup because static imports do not cover build scripts, examples,
cfg-gated code, generated code, runtime plugins, or external cargo commands.
When dependency intents are requested, a missing or malformed root
`Cargo.toml` is a hard-stop because Rust cannot produce a grounded declaration
lookup without its manifest source.

The JS/TS renderer promotes dependency hub warnings only when
`existingImports.countConfidence === "grounded"` and
`observedImportCount >= 10`. Rust pre-write has no markdown renderer, so the
same checked rule is represented as an `AGENT_REVIEW_CUE` on the dependency
candidate. `sample-only` and `unavailable` counts must never produce a
dependency hub cue.

Rust pre-write policy constants that mirror the JS/TS pre-write lanes must be
artifact-visible under `meta.lookupPolicy`. This includes the checked JS/TS
source files and the caps/thresholds for near-name hints, semantic hints,
service-operation sibling review, local-operation sibling review, file
domain-cluster cues, and dependency hub cues. These fields are provenance for
existing advisory policy. They are not repository-size caps, time limits, or
permission to skip analysis.

`meta.lookupPolicy.jsTsPrecedent` must include the JS/TS intent, cue tier, and
lookup owners that Rust pre-write has translated or intentionally exposes as
unsupported evidence: `_lib/pre-write-intent.mjs`,
`_lib/pre-write-cue-tiers.mjs`, `_lib/pre-write-lookup-name.mjs`,
`_lib/pre-write-lookup-file.mjs`, `_lib/pre-write-lookup-shape.mjs`,
`_lib/pre-write-lookup-dep.mjs`, and
`_lib/pre-write-lookup-inline-patterns.mjs`.

Rust pre-write lookup helpers have canonical owners:

- `lumin-rust-analyzer/src/prewrite/index.rs` owns the borrowed candidate index
  built from `HealthResponse::files[*].ast`. It may expose exact, near,
  semantic, service, local, shape, and dependency lookup candidates, but it must
  not serialize a repository-wide index into the product artifact.
- `lumin-rust-analyzer/src/prewrite/tokens.rs` owns shared pre-write token
  splitting, normalization, and weak-token classification. Lane-local token
  helpers may compose it, but must not redefine the shared tokenizer.
- `lumin-rust-analyzer/src/prewrite/lookup/name.rs` owns exact Rust name lookup
  against candidate-index entries.
- `lumin-rust-analyzer/src/prewrite/lookup/near.rs` owns JS/TS-derived near-name
  lookup, capped Levenshtein scoring, suppressed-near ordering, and the
  associated `meta.lookupPolicy.nearName` provenance.
- `lumin-rust-analyzer/src/prewrite/lookup/file.rs` owns Rust file intent lookup
  results, including `FILE_EXISTS`, `NEW_FILE`, skipped-file,
  symlink-unknown, and source-health path-policy handling.
- `lumin-rust-analyzer/src/prewrite/lookup/file/domain_cluster.rs` owns the
  JS/TS-derived file domain-cluster cue translation. Its `candidates`, `path`,
  and `tokens` submodules are implementation details of that lane.
- `lumin-rust-analyzer/src/prewrite/lookup/shape.rs` owns exact
  `SHAPE_MATCH`, exact `SIGNATURE_MATCH`, fields-only/type-literal
  unavailable handling, and the shape/signature lookup coverage states. Its
  `candidate`, `matches`, `model`, and `evidence` submodules are
  implementation details of that lane.
- `lumin-rust-analyzer/src/prewrite/lookup/service.rs` owns service-operation
  sibling review policy and the associated JS/TS-derived lookup policy
  provenance.
- `lumin-rust-analyzer/src/prewrite/lookup/local.rs` owns same-file local
  operation sibling review policy and the associated JS/TS-derived lookup
  policy provenance.
- `lumin-rust-analyzer/src/prewrite/lookup/dependency.rs` owns Cargo dependency
  intent lookup orchestration and the JS/TS dependency lane translation.
- `lumin-rust-analyzer/src/prewrite/lookup/dependency/graph.rs` owns the Rust
  AST static import graph, dependency import observations, and graph
  completeness evidence.
- `lumin-rust-analyzer/src/prewrite/lookup/dependency/manifest.rs` owns Cargo
  manifest aggregation, workspace dependency inheritance checks, declaration
  lookup, and binding observed consumers to the package/member manifest that
  owns the file.
- `lumin-rust-analyzer/src/prewrite/lookup/dependency/declarations.rs` owns
  Cargo dependency table scanning, target-specific dependency tables, renamed
  package handling, and manifest-key-to-code-root mapping.
- `lumin-rust-analyzer/src/prewrite/lookup/dependency/workspace.rs` owns Cargo
  workspace member expansion and `workspace.exclude` path handling. Its `glob`
  submodules own Cargo member glob expansion only; exclude entries remain
  literal path prefixes. Declared workspace members that Cargo cannot resolve
  to member `Cargo.toml` files are hard-stop manifest mismatches, not empty
  scopes. `workspace.exclude` applies to glob-expanded members before manifest
  lookup, so excluded glob matches do not need `Cargo.toml`; an explicitly
  listed member remains explicit Cargo input. If every matched glob member is
  excluded, the declaration scope is empty rather than a hard-stop.
- `lumin-rust-analyzer/src/prewrite/lookup/dependency/scope.rs` and
  `lumin-rust-analyzer/src/prewrite/lookup/dependency/targets.rs` own package
  scope matching, including explicit Cargo target paths outside a member
  directory.
- `lumin-rust-analyzer/src/prewrite/lookup/dependency/projection.rs` owns
  dependency lookup artifact projection, citations, count confidence, and
  examples.
- `lumin-rust-analyzer/src/prewrite/lookup/inline_pattern.rs` owns Rust inline
  extraction unsupported evidence until a Rust-owned inline-pattern producer or
  bridge exists.

Rust pre-write semantic hint helpers have canonical owners:

- `lumin-rust-analyzer/src/prewrite/lookup/semantic.rs` owns the semantic hint
  lane orchestration and the JS/TS-derived semantic hint policy constants
  exposed under `meta.lookupPolicy.semanticHint`.
- `lumin-rust-analyzer/src/prewrite/lookup/semantic/tokens.rs` owns semantic
  hint query token construction, candidate support-token extraction, and token
  match evidence for this lane. It may use the shared
  `prewrite/tokens.rs` tokenizer, but no other module should define a second
  semantic-token matcher for pre-write cues.
- `lumin-rust-analyzer/src/prewrite/lookup/semantic/order.rs` owns
  deterministic ordering for promoted and suppressed semantic hints.

`lumin-rust-analyzer/src/prewrite/lookup/unavailable.rs` owns the shared
`unavailableEvidence[]` artifact shape for Rust pre-write lookup lanes. Shape
lookup and inline extraction may create lane-specific unavailable evidence
through that owner, but they must not define second copies of the evidence
shape in lane-local model files.

Rust workspace common helpers have canonical owners:

- `experiments/rust-common/src/error.rs` owns shared `UsageError`,
  `usage_error(...)`, and `is_usage_error(...)` downcast classification. Rust
  CLIs must not classify usage/runtime exits by matching error-message text.
- `experiments/rust-common/src/path.rs` owns shared repository-root discovery,
  existing-directory canonicalization, POSIX path text normalization, and exact
  path-segment matching.
- `experiments/rust-common/src/json.rs` owns compact and pretty atomic JSON
  artifact writes with a trailing newline.
- `experiments/rust-common/src/hash.rs` owns the shared `sha256:`-prefixed byte,
  text, and file hashing helpers.

Cargo/rustc semantic checks are a Rust-only necessity: JS/TS lanes do not
produce Cargo `target/` build products, but Rust oracle runs do. Rust must not
write into the analyzed repository's `target/` directory by default. The
`rust-cargo-oracle` semantic artifact must make this visible under
`meta.input.cargoTargetDirPolicy`:

- `repoTargetDirUsed = false` for owned temp target modes.
- `ownedTempTargetDir = true` for `isolated-temp` and `reusable-temp`.
- `incrementalDisabled = true` and `debugSymbolsDisabled = true` when the
  oracle applies its compact Cargo profile environment.
- `staleCleanupOwnedTempTargetDirs = true` when the oracle removes only its own
  temp target directories from the OS temp directory.
- `staleIsolatedTargetDirMaxAgeSeconds` and
  `staleReusableTargetDirMaxAgeSeconds` are retention metadata for owned temp
  target cleanup. They are not analysis time limits.

These fields are transparency evidence only. They are not timeouts, analysis
caps, or permission to skip large repositories.

Targeted Cargo checks select every package with review-visible Rust syntax
evidence, then execute one multi-package `cargo check` invocation for the
selected package set. This keeps Cargo command provenance honest, lets Cargo own
the workspace scheduling, and avoids rerunning the same workspace graph once per
selected package. This is not an analysis cap: `targetPathCount`,
`candidatePackageCount`, and `selectedPackageCount` still describe the full
selected scope. Cargo may still emit the same underlying user-code diagnostic
more than once, for example when several selected packages depend on the same
broken workspace member. This is a Rust-only package-scope artifact, not a
second finding. `rust-cargo-oracle` must deduplicate identical diagnostics
before projecting `diagnostics[]`, `findings[]`, safe actions, and product
summary counts. The identity includes the diagnostic level/code/message,
rendered first line, primary spans including ownership class, and suggestion
candidate spans so distinct rustc suggestions remain distinct.

Rust oracle plan example arrays are compact artifact projections, not analysis
limits. When `oraclePlan` caps target-path, omitted-package, selected-package,
or unmatched-path examples, the artifact must preserve the full counts and
publish the example caps under `oraclePlan.sampleLimits`.

Unified analyzer semantic finding span examples are also compact artifact
projections. When `semanticFindings[*].macroExpansionSpanExamples` is capped,
the product artifact must preserve `macroExpansionSpanCount` and publish the cap
under `policy.semantic.productProjection.sampleLimits.findingSpans`.

Rust planned type escape intent support follows the JS/TS pre-write Step 2
contract:

- `intent.plannedTypeEscapes[]` is a user-declared plan, not an analysis lookup.
  Rust pre-write validates and preserves the declaration and reports
  `coverage.plannedTypeEscapes = "ran"` even when the list is empty.
- Rust must not invent a TS `any` equivalent or emit unavailable evidence for
  this lane. Post-write type-escape extraction is TS/JS-specific; Rust safety
  and opacity evidence belongs to the Rust syntax/oracle lanes.
- The normalized declaration order and optional `codeShape` /
  `alternativeConsidered` fields must remain stable so a downstream post-write
  phase can compare declared intent with observed language-specific evidence.

Rust inline extraction intent support is the Rust analogue of the JS/TS
`pre-write-lookup-inline-patterns.mjs` lane:

- `intent.refactorSources[]` is accepted as optional explicit extraction
  source evidence and follows the JS/TS input contract: `file` must be a safe
  POSIX repository-relative path, `lines[]` must contain positive integers when
  present, and `why` must be non-empty when present.
- Rust does not currently produce `inline-patterns.json`. When
  `refactorSources` is non-empty, the pre-write artifact must make that omitted
  scope visible as `coverage.inlinePatterns = "unsupported"`,
  `inlinePatternLookups[]` with `result = "UNAVAILABLE"`, and
  `unavailableEvidence[]` on the `inline-extraction` lane.
- Missing Rust inline-pattern support must not create `cueCards[]` or
  `suppressedCues[]`. Review cues for repeated inline statement patterns require
  a future Rust-owned inline-pattern producer or an explicit artifact bridge.

## 8. Do Not Invent These Again

These names are banned unless this file is amended with a reason:

- `makeSignal`
- `buildSignal`
- `signalForRange`
- `newSignal`
- `makeParseError`
- `buildParseError`
- `toLocation`
- `rangeToLocation`
- `makeLocation`
- `countSummary`
- `buildSummary`
- `summarizeSignals`
- `makeRustHealthArtifact`
- `buildRustHealthJson`
- `isTargetPath`
- `isVendorPath`
- `createThreadPool`
- `globalThreadPool`

Some of those names are tempting. That is the problem.

## 9. Direct Shape Construction Ban

Rust source health code must not hand-build these shapes outside their owners:

- `Signal`
- `ParseError`
- `Location`
- `Summary`
- final `rust-health.json` metadata outside `src/wrapper.rs` / `src/main.rs`
- Rayon runtime pool configuration

Allowed exception: tests may build literal fixtures when the point is validator
behavior. Tests must not copy production constructors under a different name.

## 10. Rayon Runtime Contract

Rust source health uses a local Rayon pool.

- Use `ThreadPoolBuilder::build()`, not `build_global()`.
- Runtime request fields are `threadCount` and `workerStackBytes`.
- Rust fields are `thread_count` and `worker_stack_bytes`.
- Default worker stack is `DEFAULT_WORKER_STACK_BYTES = 16 * 1024 * 1024`.
- AST nodes and `ra_ap_syntax` syntax trees do not cross worker boundaries as
  shared long-lived state.
- Analyze independent files in parallel, then reassemble results into a
  deterministic `BTreeMap`.

If analysis later needs a graph-wide shared AST/cache, that is a new canonical
amendment, not an inline helper.

## 11. Summary Invariants

Final artifacts must satisfy these counts:

- `summary.files === Object.keys(files).length`
- `summary.skippedFiles === skippedFiles.length`
- `summary.parseErrorFiles === count(files where parse.ok === false)`
- `summary.parseErrors === sum(files[*].parse.errors.length)`
- `summary.signals === sum(files[*].signals.length)`
- `summary.signalsByKind[kind] === count(signals where signal.kind === kind)`
- `summary.reviewSignals === count(signals where signal.visibility === "review")`
- `summary.mutedSignals === count(signals where signal.visibility === "muted")`
- `summary.signalsByVisibility[visibility] === count(signals where signal.visibility === visibility)`
- `summary.reviewSignalsByKind[kind] === count(signals where signal.kind === kind and signal.visibility === "review")`
- `summary.mutedSignalsByReason[reason] === count(signals where signal.visibility === "muted" and signal.muteReason === reason)`
- `summary.unsafeBlocks === sum(files[*].facts.unsafeBlocks)`
- `summary.unsafeFunctions === sum(files[*].facts.unsafeFunctions)`
- `summary.definitions === sum(files[*].ast.definitions.length)`
- `summary.shapeHashes === sum(files[*].ast.shapeHashes.length)`
- `summary.functionSignatures === sum(files[*].ast.functionSignatures.length)`
- `summary.implBlocks === sum(files[*].ast.impls.length)`
- `summary.implMethods === sum(files[*].ast.impls[*].methods.length)`
- `summary.useTrees === sum(files[*].ast.useTrees.length)`
- `summary.pathRefs === sum(files[*].ast.pathRefs.length)`
- `summary.methodCallSites === sum(files[*].ast.methodCallCounts values)`
- `summary.methodCalls === sum(files[*].ast.methodCalls.length)`
- `summary.macroCalls === sum(files[*].ast.macroCalls.length)`
- `summary.cfgGates === sum(files[*].ast.cfgGates.length)`
- `summary.opaqueSurfaces === sum(files[*].ast.opaqueSurfaces.length)`
- `summary.reviewOpaqueSurfaces === count(ast.opaqueSurfaces where visibility === "review")`
- `summary.mutedOpaqueSurfaces === count(ast.opaqueSurfaces where visibility === "muted")`
- `summary.mutedOpaqueSurfacesByReason[reason] === count(ast.opaqueSurfaces where visibility === "muted" and muteReason === reason)`

Rust-main wrapper mode recomputes summary after adding skipped-file evidence.
stdin compatibility mode emits no skipped-file evidence.

## 12. Path And Artifact Ordering

- Root-relative paths use POSIX slash.
- Absolute paths and normalized `..` paths are rejected before sidecar input.
- Symlinked files/directories are not followed in M6.
- `target` and `vendor` are path segments, not substring matches.
- Rust source health owns test-like path classification for Rust artifacts.
  This policy absorbs the legacy JS path-screening convention; the JS helper
  is not the source of truth for new Rust work. Test-like path segments are
  exact path components: `tests/`, `test/`, `integration/`, `e2e/`,
  `fixtures/`, `fixture/`, `mocks/`, `mock/`, `test-support/`,
  `test-utils/`, `runtime-tests/`, `playground(s)/`, `examples/`,
  `benches/`, any `__*__/` convention directory, and `*-fixture(s)`.
  Rust module files `tests.rs`, `test.rs`, `*.test.rs`, `*.spec.rs`,
  `*_test.rs`, and `*_tests.rs` are also test-like. Substrings are not enough:
  `contest.rs` remains source.
- Rust source health also mutes signals in explicit Rust test-only AST context
  without dropping raw evidence. Signals inside a direct `#[cfg(test)]` module,
  impl, or function carry `muteReason: "cfg-test"`. Signals inside a direct
  `#[test]` function carry `muteReason: "test-attribute"`. This is a review
  visibility policy only; the signal claim remains `syntax-only`.
- Rust source health applies the same visibility vocabulary to AST opaque
  surfaces. The raw `ast.opaqueSurfaces[]` evidence stays present, while the
  unified product artifact exposes review/muted opaque summaries, muted reason
  counts, and capped review examples instead of embedding full raw AST lanes.
- The standalone Rust source health CLI follows the same size discipline:
  `--artifact-profile compact` is the default and emits `astSummary` per file
  plus capped `reviewOpaqueSurfaceExamples`; `--artifact-profile full` preserves
  the raw compatibility shape with full `files[*].ast` arrays. The compact
  `astSummary` must publish `reviewOpaqueSurfaceSampleLimit` beside the capped
  example array so the artifact shows that truncation is a projection choice,
  not an analysis cap.
- Output `files` keys are sorted by path.
- `signals` are sorted by `location.byteStart`, then `kind`.
- `parse.errors` are sorted by `location.byteStart`, then `message`.
- `skippedFiles` are sorted by path.

## 13. Review Gate For New Helpers

Before adding any Rust source health or Rust pre-write helper:

1. Search this file for the intended concept.
2. Search `experiments/rust-sidecar/` and
   `experiments/rust-main/lumin-rust-analyzer/src/prewrite/` for the intended
   behavior.
3. If an owner exists, import it.
4. If no owner exists, amend this file with the new canonical name and owner.
5. Only then implement.

No "small local helper for now." That phrase is where clones breed.

## 14. Mechanical Checks

Before a Rust source health implementation review packet is accepted, run scans
equivalent to:

```bash
rg -n "makeSignal|buildSignal|signalForRange|makeParseError|buildParseError|toLocation|rangeToLocation|buildSummary|countSummary|summarizeSignals|createThreadPool|globalThreadPool" experiments/rust-sidecar tests
rg -n "pub use ra_ap_syntax|ra_ap_syntax::.*(Request|Response|FileHealth|Signal|ParseError|Location|Summary)" experiments/rust-sidecar
rg -n "Signal \\{|ParseError \\{|Summary \\{" experiments/rust-sidecar/rust-source-health/src
```

Expected result: no matches except this canonical file, tests that explicitly
exercise validator failure, or documented owner modules listed above.

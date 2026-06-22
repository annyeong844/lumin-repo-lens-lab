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
- `ast.impls[]`: Rust `impl` block observations with `target`, optional
  `trait`, method owner evidence, and `location`. This is the Rust analogue of
  the JS/TS `classMethodIndex`: impl methods are visible as owner evidence
  without pretending AST-only analysis has type or trait-solving certainty.
- `ast.useTrees[]`: `use` tree observations with raw tree text, optional path,
  glob status, visibility, and `location`.
- `ast.pathRefs[]`: qualified expression-position path references with raw path
  text, terminal name, and `location`. Local variable refs and constructor-like
  single-segment paths are not emitted as raw path facts.
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

Rust file intent lookup is the Rust analogue of the JS/TS pre-write file lane:

- a requested path present in `HealthResponse::files` is `FILE_EXISTS`;
- a safe repo-relative `.rs` path absent from `HealthResponse::files` and
  `skippedFiles`, under the source-health path policy, is `NEW_FILE`;
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

Rust shape intent lookup follows the JS/TS P4 discipline for unsupported
evidence: it must not infer structural equality from loose field names and must
not add fuzzy shape matching. Until a Rust-owned shape-index equivalent exists,
non-empty shape intents emit `coverage.shapes = "unsupported"`,
`shapeLookups[]` rows with `result = "UNAVAILABLE"`, and
`unavailableEvidence[]` rows on the `shape-hash` lane. Fields-only intents cite
the JS/TS rule that field names alone are not structural equality evidence;
exact hashes or `typeLiteral` entries cite the missing Rust shape lookup lane.
No SAFE or review cue may be emitted from this unsupported shape lane.

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
  Rust module files `tests.rs`, `test.rs`, `*.test.rs`, `*.spec.rs`, and
  `*_test.rs` are also test-like. Substrings are not enough: `contest.rs`
  remains source.
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
  the raw compatibility shape with full `files[*].ast` arrays.
- Output `files` keys are sorted by path.
- `signals` are sorted by `location.byteStart`, then `kind`.
- `parse.errors` are sorted by `location.byteStart`, then `message`.
- `skippedFiles` are sorted by path.

## 13. Review Gate For New Helpers

Before adding any Rust source health helper:

1. Search this file for the intended concept.
2. Search `experiments/rust-sidecar/` for the intended behavior.
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

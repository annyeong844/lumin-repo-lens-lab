# Rust Structural Review Checklist (v2.1)

Use this companion whenever Rust findings are in scope. It supplements
`REVIEW_CHECKLIST.md`; it does not let JS/TS artifacts make Rust claims.

The operating model is explicit:

1. Lumin and Rust tooling extract reproducible evidence.
2. rustc, clippy, cargo, Miri, and ecosystem tools enforce compiler-level rules.
3. The AI review model reads those outputs and the cited source, then makes the
   design judgment. Layer 3 is not a handoff to an unavailable human reviewer.

Use `BLOCKER` for merge-blocking defects, `FIX` for required changes, and
`WATCH` for justified follow-up. Tool gates are triggers, not verdicts.

---

## Rust evidence contract

### 1. Product-run gate

Rust product claims require both:

- `manifest.json.rustAnalysis.status = "complete"`
- `manifest.json.rustAnalysis.available = true`

The effective scope is `manifest.json.rustAnalysis.scanScope`. Cite it for
absence claims. When the gate is not complete, Rust claims stay
`[unknown]`/`[degraded]` and name the scan range or blind zone.

### 2. Unified analyzer artifact

`rust-analyzer-health.latest.json` grounds the product-facing Rust lane:

- file and parse coverage: `summary.files`, `summary.syntaxParseErrorFiles`,
  `summary.syntaxParseErrors`, `phases.syntax.skippedFileCount`, and
  `phases.syntax.skippedFileExamples[]`;
- review signals: `summary.syntaxReviewSignals`,
  `summary.syntaxReviewSignalExamples[]`, and per-file
  `files.<path>.syntax.signalSummary.byKind`;
- unsafe syntax facts: `files.<path>.syntax.facts.unsafeBlocks` and
  `files.<path>.syntax.facts.unsafeFunctions` when non-zero;
- macro/cfg opacity:
  `summary.syntaxReviewOpaqueSurfaces` and
  `summary.syntaxReviewOpaqueSurfaceExamples[]`;
- clone counts: `summary.syntaxFunctionCloneExactBodyGroups`,
  `summary.syntaxFunctionCloneStructureGroups`,
  `summary.syntaxFunctionCloneSignatureGroups`, and
  `summary.syntaxFunctionCloneNearCandidates`;
- compiler-oracle evidence: `summary.oracleBridgeStatus`,
  `summary.actionTierSummary`, `semanticFindings[]`, and
  `semanticDiagnostics[]`.

The emitted macro/cfg opacity count is
`rust-analyzer-health.latest.json.summary.syntaxReviewOpaqueSurfaces`.
No checked artifact emits a JSON field named
`compilerOracleOpaqueSurfaces`; reviews must not cite that name.

The unified artifact exposes clone counts and examples, not the complete raw
clone member groups. `artifactRefs.syntax.rawArtifact` documents the
compatibility CLI route for the full syntax lane.

### 3. Standalone Rust source-health artifact

When a checked `rust-health.json` was deliberately produced by
`lumin-rust-source-health`, it may additionally ground:

- `summary.signalsByKind`, `summary.unsafeBlocks`, and full per-file syntax facts;
- full-profile `functionCloneGroups.exactBodyGroups[]`, `structureGroups[]`,
  `signatureGroups[]`, and `nearFunctionCandidates[]`; compact artifacts expose
  the corresponding counts plus capped `*Examples[]` arrays instead;
- `unusedDefinitionAnalysis.summary`, `findings[]`, `excludedCandidates[]`, and
  `degradedScopes[]`.

Do not pretend these raw lanes are present in the unified product artifact.
`unusedDefinitionAnalysis` is currently a standalone source-health projection;
if that artifact was not produced, Rust unused-definition detail is `[unknown]`.
Compact source-health artifacts retain uncapped unused-definition summary
counts but may cap excluded-candidate examples. Read
`unusedDefinitionAnalysis.excludedCandidateProjection` before treating the
example array as complete.

### 4. Rust topology

The JS/TS `topology.json` is not Rust evidence by default. Rust module topology
is grounded only when the run registers Rust topology evidence such as
`topology.json.meta.rustTopologyScanner` and the cited nodes/edges cover the
Rust scope being reviewed. Otherwise Rust mod cycles and module fan-in are
`[unknown]`; source inspection may support a model judgment but not a fabricated
artifact citation.

### 5. Ghost-citation guard

- Never cite an artifact name without a field value.
- Never claim absence without `manifest.json.rustAnalysis.scanScope`.
- Never use `symbols.json`, `shape-index.json`, or `function-clones.json` as
  Rust shape, clone, or dead-definition evidence.
- A skipped file, capped example list, muted signal, or absent optional cargo
  oracle is not clean evidence.

---

## Responsibility layers

### Layer 1: lumin-grounded

- Syntax signal existence and distribution.
- Parse failures and skipped-file evidence.
- Per-file unsafe syntax counts.
- Macro/cfg opacity count and capped examples.
- Unified clone counts and standalone raw clone groups when explicitly emitted.
- Standalone unused-definition analysis when explicitly emitted.
- Invocation-specific Rust pre-write and matching post-write evidence.

### Layer 2: rustc, clippy, cargo, and Miri

- `clippy::unwrap_used`, `expect_used`, and `panic` for production policy.
- `clippy::await_holding_lock` for lock guards crossing `.await`.
- `clippy::undocumented_unsafe_blocks` for required `// SAFETY:` comments.
- rustc `dead_code`, including an audit of broad `#[allow(dead_code)]`.
- rustfmt and the repository's clippy baseline.
- `cargo semver-checks` for public API compatibility.
- `cargo audit`, `cargo deny`, and `cargo udeps` for dependency hygiene.
- doctests and `cargo test --doc` for public examples.
- Miri for exercised unsafe test paths.

If the model finds one of these defects in source but the expected tool did not,
record the enforcement gap: tool not run, lint missing, feature/cfg combination
not exercised, or `#[allow]` hiding the signal.

### Layer 3: AI review-model judgment

The review model reads source and decides:

- whether a clone is legitimate ownership or clone-to-compile;
- whether `Arc<Mutex<T>>` models real shared state or hides a better ownership
  transfer/channel design;
- whether error layers preserve meaningful causes;
- whether a typestate/newtype/trait enforces a real invariant or is ceremony;
- whether structural clone candidates are semantically equivalent;
- whether tests exercise product contracts rather than private scaffolding;
- whether security, FFI, cancellation, and observability boundaries are sound.

These are model judgments with file:line evidence. Do not label them
`[grounded]` unless an artifact/tool field directly proves the stated fact.

---

## A. Size and simplicity

- Does file/module/function size fit the responsibility?
- Does a large function contain separable validation, normalization, IO, state
  mutation, and projection responsibilities?
- Are helper families cohesive, or is there a helper zoo?
- Are modules split by ownership, or merely split into forwarding files?
- Are concrete dependencies preventing a useful boundary?
- If Rust topology evidence exists, are there mod cycles or inverted flows?
- Do clone, lock, and panic signals point to a deeper ownership problem?

Use per-file `syntax.facts.maxFunctionLines` as a positive cue only. It does not
identify the function or prove multiple responsibilities; the model must read
the file before recommending a split.

## B. Duplication and shape ownership

- Read raw clone members before recommending a merge. Count-only unified
  summaries do not prove which functions should converge.
- Are equivalent structs/enums independently owned and drifting?
- Does standalone `unusedDefinitionAnalysis` show migration residue, and do its
  RUST-FP gates/blocked scopes preserve uncertainty?
- Are repeated trait impls better expressed by an existing derive or blanket
  impl without obscuring behavior?
- Do multiple entrypoints independently implement the same pipeline instead of
  delegating to one owner?

## C. Cohesion and boundaries

- Does each module own a coherent responsibility and dependency direction?
- Are parsing, validation, normalization, and error conversion concentrated at
  trust boundaries?
- Is mutation routed through a visible owner rather than scattered interior
  mutability?
- Do `pub`, `pub(crate)`, and `pub(super)` enforce the intended boundary?
- Are `utils.rs`/`helpers.rs` becoming unowned dependency magnets?
- Do glob re-exports or third-party parser types leak into public protocols?
- Do workspace crate boundaries improve ownership/compilation, or only scatter
  files?

## D. Types and contracts

- Are trait/generic/dynamic-dispatch choices justified by real consumers?
- Are JSON-to-Rust lowering and string vocabularies canonical and consistent?
- Should state-dependent `Option` fields be an enum variant instead?
- Do newtypes enforce validation or identity rather than rename primitives?
- Does boundary parsing make invalid states unrepresentable where worthwhile?
- Are `TryFrom` and typed parsing used instead of unchecked casts/unwraps?
- Does an ordering contract justify typestate, after H's ceremony check?

## E. Failure handling

- Do production library/service paths return `Result` instead of panicking?
- Are errors swallowed through `.ok()`, `let _ =`, empty `if let Ok`, or
  misleading defaults?
- Does fallback preserve artifact-visible evidence instead of hiding a broken
  owner?
- Are cause chains retained through `#[from]`, `source()`, and `.context()`?
- Is the error type appropriate to the layer (`thiserror` library,
  `anyhow`/`eyre` application boundary)?
- Does RAII cover resource cleanup and unwind paths?
- Are spawned tasks joined or intentionally supervised?
- Can futures be silently dropped before required effects complete?

## F. Abstraction and tests

- Is the abstraction proportional to its consumers and invariants?
- Do tests cover a real happy path, realistic edge cases, and required hard
  stops?
- Do assertions target public behavior/artifact contracts rather than private
  helper shape?
- Are fixtures/builders/mocks becoming a second product implementation?
- Are mocks placed at slow/external boundaries rather than inside core logic?
- Do doctests exercise public APIs where applicable?
- Do unsafe tests state and exercise the claimed invariant, with Miri where
  practical?

## G. Rust-specific design

### Ownership and borrowing

- Prefer borrowed inputs for analysis helpers and owned values at process,
  thread-job, and final-artifact boundaries.
- Treat non-trivial clones of source text, ASTs, vectors, maps, and graph state
  as guilty until the ownership reason is visible.
- Check whether `Cow<'_, str>` or a narrower lifetime removes repeated
  allocation without complicating the API.
- Reject self-referential/`Pin`/unsafe designs when a simpler owned boundary
  exists.

### Traits and conversions

- Prefer `From`/`TryFrom` over scattered hand-written conversions.
- Check `Display` versus `Debug`, derives, object safety, monomorphization cost,
  sealed traits, and orphan-rule newtypes against actual API needs.

### Concurrency

- Confirm `Send`/`Sync`, `Rc`/`Arc`, and `RefCell`/`Mutex` choices match the
  actual thread boundary.
- Challenge `Arc<Mutex<T>>` in analysis pipelines; job ownership transfer is
  usually cleaner.
- Use explicit local Rayon pools with thread/stack policy, never an implicit
  process-global pool.
- Review atomic ordering and cancellation behavior where present.

### Unsafe and FFI

- Keep unsafe blocks minimal and wrapped by a safe invariant-bearing API.
- Require meaningful `// SAFETY:` explanations, not ceremonial comments.
- Prevent panics from crossing FFI callbacks (`catch_unwind` where required).
- Prefer safe alternatives to raw pointers, `transmute`, and `MaybeUninit`.

### Hot-path allocation

- Inspect repeated `clone`, `String`, `Vec`, `Box`, `format!`, and intermediate
  `collect()` only on measured or structurally obvious hot paths.

## H. Ceremony and excess contracts

Evaluate H beside D and G:

- Does a single-implementation trait have a real second implementation or
  boundary purpose?
- Does a newtype enforce an invariant?
- Are error variants actually distinguished by callers?
- Does a tiny object need a builder or typestate stack?
- Does a single-call generic buy anything?
- Is wiring code larger than the behavior it protects?

Ask: **what product behavior or invariant breaks if this contract is deleted?**
Compile errors in forwarding declarations alone indicate a ceremony candidate.

Preserve real protocol, trust, process/thread, public API, and unsafe-isolation
boundaries even when they have one consumer.

Team-specific owner/naming/shape rules move through:
`canon-draft` → explicit AI review-model promotion with a checked diff →
`check-canon`.

## I. Security, dependencies, and operations

Lumin does not ground this section by itself. The review model combines source
inspection with dedicated tools:

- validate stdin, files, environment, subprocess, and FFI inputs at the trust
  boundary;
- trace untrusted values to paths, commands, FFI, unsafe, logs, and artifacts;
- check secret leakage and diagnostic redaction;
- justify new crates, features, transitive dependencies, licenses, MSRV, and
  lockfile changes;
- use `cargo audit`, `cargo deny`, `cargo udeps`, and feature-combination builds;
- require observable failure paths and meaningful logs/tracing;
- keep non-obvious public and unsafe contracts documented where they live.

---

## Cadence

- Every Rust edit transaction: invocation-specific Rust `pre-write` → edit →
  matching `post-write`, before broad generators mutate the scan range.
- Once per branch or large refactor: full audit with `--rust-analyzer`.
- Compiler lane on every commit: rustfmt, configured clippy, and relevant cargo
  tests/checks.
- Security/compatibility tools run at their repository-defined cadence.
- If `manifest.json.rustAnalysis.status` is not complete, Rust product claims
  remain `[unknown]`/`[degraded]`.

## Output contract

For every reported Rust finding include:

1. severity (`BLOCKER`, `FIX`, or `WATCH`);
2. evidence label with exact artifact value or file:line source range;
3. symptom;
4. cause;
5. first repair boundary;
6. missing evidence or macro/cfg/scan-scope limitation.

The AI review model makes the final judgment. It must not turn absent evidence
into a clean verdict, and it must not defer a source-readable decision merely
because no human reviewer is present.

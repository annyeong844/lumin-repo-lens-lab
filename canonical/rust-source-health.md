# canonical/rust-source-health.md

> **Role:** canonical naming, shape, helper, and module contract for the Rust source health track.
> **Owner:** this file.
> **Status:** M6 spine addition.
> **Last updated:** 2026-06-16

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
Rust source health emits a separate `rust-health.json` artifact.

## 3. Naming Lowering

| Surface | Convention | Example |
|---|---|---|
| JSON field | `camelCase` | `signalsByKind`, `byteStart`, `unsafeBlocks` |
| JSON string enum / reason / kind | `kebab-case` | `unwrap-call`, `invalid-utf8`, `syntax-only` |
| CLI flag | `kebab-case` | `--rust-source-health-bin`, `--worker-stack-bytes` |
| Rust module / function / field | `snake_case` | `review_signal`, `worker_stack_bytes` |
| Rust type / enum / struct | `PascalCase` | `FileHealth`, `RuntimeConfig` |
| Rust constant | `SCREAMING_SNAKE_CASE` | `PARSER_VERSION` |
| JS function / variable | `camelCase` | `runRustSourceHealth`, `hasPathSegment` |
| JS constant | `SCREAMING_SNAKE_CASE` | `RUST_SOURCE_HEALTH_SCHEMA_VERSION` |
| File path | `kebab-case` unless local convention already exists | `rust-source-health-runner.mjs` |

Lowering examples:

| Concept | JSON | Rust | JS |
|---|---|---|---|
| unwrap method signal | `unwrap-call` | `review_signal("unwrap-call", ...)` | validated as `signal.kind` |
| parse error | `parse.errors[]` | `syntax_parse_error(...)` | `validateRustHealthFinalArtifact(...)` |
| worker stack bytes | `workerStackBytes` | `worker_stack_bytes` | `workerStackBytes` |
| source hash | `sha256` | `sha256` | `sha256` |

## 4. Owned Protocol Boundary

All JSON-visible shapes are project-owned structs or plain JS objects. No
`ra_ap_syntax` type may cross into the protocol, public module surface, JSON
artifact, or JS validator.

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
| `src/protocol.rs` | Request/response structs, schema constants, project-owned enums/strings | parser traversal, signal construction logic |
| `src/locations.rs` | `LineIndex`, byte-to-line/column conversion | signal kinds, summary counts |
| `src/signals.rs` | `review_signal(...)`, `syntax_parse_error(...)` | parser traversal, summary counts |
| `src/summary.rs` | `summarize(...)` for `BTreeMap<String, FileHealth>` | signal construction, path policy |
| `src/parallel.rs` | local Rayon `ThreadPool`, `RuntimeConfig`, stack/thread policy | AST storage, file analysis |
| `src/analyzer.rs` | syntax traversal and file-level analysis | protocol schema changes, final artifact metadata |
| `src/main.rs` | stdin/stdout contract, request validation, pool install, exit behavior | filesystem walking, output file writes |

No extra Rust module may create `Signal`, `ParseError`, `Summary`, `Location`,
or runtime pool settings unless this table is amended first.

## 6. Canonical JS Modules

| File | Owns | Must not own |
|---|---|---|
| `_lib/rust-source-health-schema.mjs` | schema constants, summary recompute, sidecar/final validators | filesystem walking, child process execution |
| `_lib/rust-source-health-runner.mjs` | file collection, path policy, hashing, UTF-8 decode, sidecar execution, final artifact assembly | schema rules not delegated to the schema module |
| `scripts/run-rust-source-health.mjs` | lab CLI parsing and user-facing command wiring | duplicate schema validation, duplicate path policy |

If a second JS file wants to compute summary, validate locations, decide path
policy, or assemble final metadata, it imports these modules. It does not clone
the logic.

## 7. Canonical Constructors And Helpers

### Rust

| Purpose | Canonical name | Owner |
|---|---|---|
| review signal construction | `review_signal(kind, line_index, range)` | `src/signals.rs` |
| parse error construction | `syntax_parse_error(message, line_index, range)` | `src/signals.rs` |
| location conversion | `LineIndex::location(byte_start, byte_end)` | `src/locations.rs` |
| artifact summary | `summarize(files)` | `src/summary.rs` |
| local Rayon pool | `build_pool(runtime_config)` | `src/parallel.rs` |
| unsafe block syntax check | `is_unsafe_block_expr(node)` | `src/analyzer.rs` |
| method call signal scan | `collect_method_call_signals(...)` | `src/analyzer.rs` |
| macro call signal scan | `collect_macro_call_signals(...)` | `src/analyzer.rs` |

`review_signal` and `syntax_parse_error` are the only production helpers that
convert `TextRange` into `Location`.

### JavaScript

| Purpose | Canonical name | Owner |
|---|---|---|
| sidecar artifact validation | `validateRustHealthSidecarArtifact(artifact)` | `_lib/rust-source-health-schema.mjs` |
| final artifact validation | `validateRustHealthFinalArtifact(artifact)` | `_lib/rust-source-health-schema.mjs` |
| summary recompute | `summarizeRustHealthArtifact(artifact)` | `_lib/rust-source-health-schema.mjs` |
| invariant diagnostics | `rustHealthInvariantProblems(artifact)` | `_lib/rust-source-health-schema.mjs` |
| stable object compare | `stableObject(value)` | `_lib/rust-source-health-schema.mjs` |
| path segment predicate | `hasPathSegment(path, segment)` | `_lib/rust-source-health-runner.mjs` |
| path exclusion policy | `isExcludedByPathPolicy(path)` | `_lib/rust-source-health-runner.mjs` |
| input collection | `collectRustSourceHealthInput(options)` | `_lib/rust-source-health-runner.mjs` |
| sidecar execution | `runRustSourceHealthSidecar(options)` | `_lib/rust-source-health-runner.mjs` |
| final assembly | `buildFinalRustHealthArtifact(options)` | `_lib/rust-source-health-runner.mjs` |
| artifact write | `writeRustHealthArtifact(outputPath, artifact)` | `_lib/rust-source-health-runner.mjs` |
| full run | `runRustSourceHealth(options)` | `_lib/rust-source-health-runner.mjs` |

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
- final `rust-health.json` metadata
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
- `summary.unsafeBlocks === sum(files[*].facts.unsafeBlocks)`
- `summary.unsafeFunctions === sum(files[*].facts.unsafeFunctions)`

The wrapper recomputes summary after adding wrapper-owned skipped-file evidence.
It does not trust sidecar summary after mutation.

## 12. Path And Artifact Ordering

- Root-relative paths use POSIX slash.
- Absolute paths and normalized `..` paths are rejected before sidecar input.
- Symlinked files/directories are not followed in M6.
- `target` and `vendor` are path segments, not substring matches.
- Output `files` keys are sorted by path.
- `signals` are sorted by `location.byteStart`, then `kind`.
- `parse.errors` are sorted by `location.byteStart`, then `message`.
- `skippedFiles` are sorted by path.

## 13. Review Gate For New Helpers

Before adding any Rust source health helper:

1. Search this file for the intended concept.
2. Search `_lib/` and `experiments/rust-sidecar/` for the intended behavior.
3. If an owner exists, import it.
4. If no owner exists, amend this file with the new canonical name and owner.
5. Only then implement.

No "small local helper for now." That phrase is where clones breed.

## 14. Mechanical Checks

Before a Rust source health implementation review packet is accepted, run scans
equivalent to:

```bash
rg -n "makeSignal|buildSignal|signalForRange|makeParseError|buildParseError|toLocation|rangeToLocation|buildSummary|countSummary|summarizeSignals|createThreadPool|globalThreadPool" experiments/rust-sidecar _lib scripts tests
rg -n "pub use ra_ap_syntax|ra_ap_syntax::.*(Request|Response|FileHealth|Signal|ParseError|Location|Summary)" experiments/rust-sidecar
rg -n "Signal \\{|ParseError \\{|Summary \\{" experiments/rust-sidecar/rust-source-health/src
```

Expected result: no matches except this canonical file, tests that explicitly
exercise validator failure, or documented owner modules listed above.

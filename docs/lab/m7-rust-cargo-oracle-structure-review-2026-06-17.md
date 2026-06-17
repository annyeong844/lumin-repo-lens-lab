# M7 Rust Cargo Oracle Structure Review - 2026-06-17

## Scope

This note reviews the Rust-side M7 cargo oracle structure after the first
implementation and follow-up refactor. The change is intentionally structural:
move `rust-cargo-oracle` from a single large `lib.rs` toward the typed,
module-owned shape already used by `rust-source-health`.

The reviewed implementation lives under:

- `experiments/rust-main/rust-cargo-oracle/`
- `experiments/rust-sidecar/rust-source-health/`
- `canonical/oracle-registry.json`
- `canonical/evidence-ladder.md`
- `docs/lab/m7-semantic-oracle-design-2026-06-17.md`
- `tests/fixtures/m7-cargo-json-diagnostic-capture-v4/`

## Result

Decision: `m7-rust-cargo-oracle-structure-ready-for-review`.

The important review finding from the previous packet was correct: the first
`rust-cargo-oracle` implementation had a healthy classifier model, but the
crate root carried too many unrelated responsibilities. That is now corrected.
`lib.rs` is an orchestration entrypoint, not the owner of artifact schema,
diagnostic classification, cargo process execution, metadata ownership, scope
resolution, input hashing, toolchain probing, and IO utilities.

## Structural Outcome

Current module split:

| Module | Responsibility |
| --- | --- |
| `lib.rs` | Orchestrates one cargo oracle run and writes the artifact. |
| `protocol.rs` | Typed `semantic-health.v1` schema, claim kinds, rules, and enums. |
| `artifact.rs` | Builds findings, diagnostics, coverage, summary, and artifact meta. |
| `classify.rs` | Normalizes cargo diagnostics and assigns confidence/claim/rule. |
| `command.rs` | Runs cargo/rust commands with timeout handling. |
| `metadata.rs` | Owns cargo metadata structs and package selection. |
| `ownership.rs` | Resolves user-code, dependency, generated, and unknown span ownership. |
| `scope.rs` | Builds package/target/feature/cfg/target-triple coverage scope. |
| `config.rs` | Discovers and reads cargo config inputs. |
| `input_hash.rs` | Builds the analysis input set hash from files, env, config, and args. |
| `toolchain.rs` | Captures cargo/rustc toolchain metadata. |
| `util.rs` | Owns atomic JSON writes and SHA-256 helpers. |
| `path_util.rs` | Owns shared path normalization and containment helpers. |

Line count after the split:

| File | Lines |
| --- | ---: |
| `artifact.rs` | 425 |
| `classify.rs` | 361 |
| `command.rs` | 157 |
| `config.rs` | 164 |
| `input_hash.rs` | 140 |
| `lib.rs` | 128 |
| `main.rs` | 124 |
| `metadata.rs` | 131 |
| `ownership.rs` | 236 |
| `path_util.rs` | 32 |
| `protocol.rs` | 413 |
| `scope.rs` | 351 |
| `toolchain.rs` | 50 |
| `util.rs` | 45 |

This is a better stopping point than further slicing. The remaining large files
are cohesive domain modules, not miscellaneous buckets.

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

1. Does `artifact.rs` own the right boundary, or should coverage building move
   into a future `coverage.rs` once more coverage kinds exist?
2. Are `ClaimKind` and `ClassificationRule` now sufficiently tied to
   `canonical/oracle-registry.json` for the M7 first slice?
3. Is `cacheReuse.policy = no-reuse-unless-complete-influence-set-is-captured`
   still honest enough while `analysisInputSetHash` remains a conservative but
   incomplete provenance identity?
4. Should `cfgSetComplete: false` prevent any production-facing "clean"
   rendering, or is the current scoped wording enough for experimental output?

## What This Closes

The refactor directly addresses these review points:

- `lib.rs` is no longer a 1500+ line single module.
- Artifact output is typed through `protocol.rs` instead of ad hoc top-level
  `serde_json::Value` construction.
- Summary counts are based on typed findings and diagnostics, not JSON pointer
  re-parsing.
- `meta.output` is now present when output is configured, so the CLI can report
  the written file instead of dumping a full artifact by accident.
- Diagnostic claim/rule vocabulary is represented by enums, and the registry
  contract test checks emitted classifier rules against the registry.
- Rule-backed rustc lint diagnostics no longer claim verified authority IDs.
- Path normalization has a single owner inside the cargo oracle crate.

## Remaining Review Notes

- M7 fixture files are included in the staged change set. Keep
  `tests/fixtures/m7-cargo-json-diagnostic-capture-v4/` staged because the tests
  read it directly.
- `analysisInputSetHash` remains intentionally non-reusable. The artifact states
  `no-reuse-unless-complete-influence-set-is-captured`; consumers must not treat
  it as a safe cache key.
- The broader working tree still contains generated-review changes and deleted
  legacy `.mjs` files. Review this as the Rust migration packet, not as a small
  isolated Rust-only diff.
- `artifact.rs` is intentionally the largest module after the split. It is
  cohesive today; only split it again when coverage/finding families grow enough
  to justify a dedicated module.

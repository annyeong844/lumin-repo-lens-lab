# Rust Pre-Write Name Cues Design

## Decision

Add an on-demand `pre-write` command to the existing
`lumin-rust-analyzer` package. The command consumes the typed Rust source
health response in memory and emits a small pre-write advisory. It does not
serialize a repository-wide symbol index into the normal unified analyzer
artifact and does not create another crate.

This is the Rust migration of the checked TS/JS path:

| TS/JS owner | Rust destination |
| --- | --- |
| `_lib/extract-ts.mjs::collectClassMethodSurface` | existing `rust-source-health` `AstFacts::impls` |
| `_lib/symbol-graph-artifact.mjs::buildClassMethodIndex` | `lumin-rust-analyzer::prewrite::index` in-memory typed index |
| `_lib/pre-write-intent.mjs::validateIntent` | `lumin-rust-analyzer::prewrite::intent` |
| `_lib/pre-write-lookup-name.mjs::lookupName` | `lumin-rust-analyzer::prewrite::lookup` |
| `_lib/pre-write-token-policy.mjs` | `lumin-rust-analyzer::prewrite::tokens` |
| `_lib/pre-write-cue-tiers.mjs::addNameLookup` | `lumin-rust-analyzer::prewrite::cues` |
| `skills/.../producers/pre-write.mjs` name lane | `lumin-rust-analyzer pre-write` command |

The JS/TS implementation remains the owner for JS/TS. The Rust destination
replaces the same mechanism only for Rust source input.

## Product Boundary

The existing invocation remains compatible:

```text
lumin-rust-analyzer --root <path> --source-commit <sha> ...
```

The new invocation is explicit:

```text
lumin-rust-analyzer pre-write \
  --root <path> \
  --source-commit <sha> \
  --intent <path> \
  [--output <path>] \
  [--threads <n>] \
  [--worker-stack-bytes <bytes>]
```

`pre-write` runs the Rust syntax phase only. It must not run Cargo metadata or
Cargo check. The existing cargo/rustc oracle can verify diagnostics and clean
build scope, but it cannot prove that an unreferenced name is absent from a
repository. Running it for this query would add cost without adding authority.

No wall-clock timeout, repository cap, file cap, package cap, or new threshold
is introduced.

## Modules

Keep the work inside the existing analyzer package:

```text
src/prewrite.rs                 command orchestration and public module boundary
src/prewrite/intent.rs          typed intent parsing and normalization
src/prewrite/index.rs           borrowed in-memory candidate enumeration
src/prewrite/tokens.rs          direct TS/JS token-policy migration
src/prewrite/lookup.rs          exact, near-name, and intent-token lookup
src/prewrite/cues.rs            SAFE/REVIEW/MUTED cue projection
src/prewrite/artifact.rs        typed advisory artifact and coverage metadata
```

`main.rs` remains the process entrypoint. `cli.rs` selects either the legacy
analyze command or the explicit pre-write command. Source-health continues to
own AST extraction and path classification; pre-write policy belongs to the
unified analyzer, not the leaf syntax crate.

Do not create a persistent `ImplMethodIndex` protocol field. The index is a
borrowed view over `HealthResponse::files` and disappears after advisory
serialization.

## Intent Contract

The command accepts the existing five-key pre-write transport:

```json
{
  "names": [],
  "shapes": [],
  "files": [],
  "dependencies": [],
  "plannedTypeEscapes": []
}
```

For this migration slice:

- `names` accepts non-empty strings or objects with `name`, optional `kind`,
  optional `why`, and optional `ownerFile` / `file` / `targetFile` locality.
- Missing top-level arrays default to empty arrays with the same explicit
  `missing-intent-key-defaulted` warning used by TS/JS.
- A present top-level key with the wrong type is a usage error and exits 2.
- Non-empty `shapes`, `files`, `dependencies`, and `plannedTypeEscapes` remain
  valid transport. Their lane coverage is emitted as `unsupported`; they are
  never silently discarded and never interpreted as successfully checked.
- The observed compatibility field `taskId` is accepted as an optional string
  and preserved as typed data. Other unknown top-level fields are rejected.
  TS/JS preserves arbitrary extras because its transport is dynamic; doing the
  same with `serde_json::Value` would reopen the typed-boundary defect already
  removed from the Rust integration layer. A new extra field must first gain a
  project-owned Rust type.

This is an intentionally visible migration boundary, not a permanent reduced
Rust intent model. Later slices may replace each `unsupported` lane only when
its TS/JS owner is migrated end to end.

## Candidate Index

The in-memory index enumerates two evidence lanes from `HealthResponse`:

1. `ast.definitions[]` becomes the Rust definition lane.
2. `ast.impls[].methods[]` becomes the Rust impl-method lane.

An impl method must not also become a definition-lane SAFE candidate. The
index excludes a function definition when its location equals a method
location in the same file. Raw source-health facts remain unchanged; this is a
consumer-specific projection equivalent to TS/JS keeping class methods out of
`defIndex` while retaining them in `classMethodIndex`.

Candidate identities are deterministic:

```text
definition:  <file>::<name>
inherent impl method: <file>::<impl-target>#<name>
trait impl method:    <file>::<impl-target> as <trait-path>#<name>
```

Trait implementation identity includes the observed trait path but does not
claim trait resolution. Multiple candidates with the same identity remain
separate location-bearing rows and sort by file, owner, name, then location.

## Lookup Policy

Port the checked TS/JS name policy without tuning:

- Exact definition names produce `EXISTS` or `EXISTS_MULTIPLE` identities.
- Exact impl methods are considered search candidates, not definition
  identities; distance zero still routes through the impl-method REVIEW lane.
- Near-name filtering uses the TS/JS values:
  - maximum ordinary Levenshtein distance: `2`
  - maximum ordinary length delta: `2`
  - shared-prefix route minimum: `4`
  - result count: `5`
- The shared-prefix route retains the TS/JS relaxed length rule and capped
  distance calculation. This preserves the verified
  `handleBulkDelete` -> `handleDelete` behavior.
- Intent-token hints use the TS/JS minimum score `2` and result count `5`.
- Token metadata remains `camel-snake-kebab-digit-v1` and
  `prewrite-token-policy-v1`.
- Weak tokens and aliases are copied exactly from
  `_lib/pre-write-token-policy.mjs`; Rust does not add vocabulary.

When no exact definition is observed, lookup searches both definition and
impl-method candidates for near-name and intent-token hints. Sorting follows
the TS/JS order: strongest match first, impl-method lane before definition lane
at equal distance, then name and owner file.

## Cue Policy

Use the TS/JS vocabulary and meanings:

- Exact definition evidence emits `SAFE_CUE` with `safeMeaning: "claim-only"`.
  It explicitly remains unsafe for semantic equivalence, automatic reuse, and
  automatic fixes.
- Exact or approximate impl-method evidence emits `AGENT_REVIEW_CUE` with
  `evidenceLane: "impl-method-name"`. AST proves the owner syntax exists but
  not method dispatch, trait selection, or semantic equivalence.
- Definition near-name and intent-token hints also emit
  `AGENT_REVIEW_CUE`; heuristics never become SAFE.
- Candidates in files already classified by source-health as test-like or
  generated are emitted as `MUTED_CUE`. No duplicate path list is added to the
  analyzer.

SAFE is not weakened merely because the repository also contains opaque
syntax. A directly observed exact definition remains a grounded existence
claim. Opaque evidence only limits absence and heuristic claims.

## Coverage And Failure Semantics

The advisory records parser, source-health policy, signal-policy, source
commit, and per-lane coverage.

- Malformed JSON, malformed intent fields, invalid root paths, and final
  artifact contract failures hard-stop.
- Source parse errors remain artifact-visible and do not abort repository
  analysis.
- If a name is not observed and review-visible macro or cfg opaque surfaces
  exist, the result remains `NOT_OBSERVED` but carries `taintedBy` evidence.
  It must not claim the name does not exist.
- Muted opaque surfaces stay auditable but do not raise the rendered cue tier.
- Unsupported intent lanes report `unsupported`; they do not hard-stop valid
  name lookup and do not report zero findings as completed coverage.
- Output ordering is deterministic.

## Artifact Size And Runtime

The normal unified analyzer artifact is byte-for-byte shape-compatible; no
symbol index or impl-method array is added to it. The pre-write artifact
contains only normalized intent, lookup results, cue cards, suppressed cues,
coverage, and provenance. Repository-wide candidates remain in memory and only
matched or suppressed examples enter the output.

The syntax scan keeps the existing local Rayon pool and stack policy. There is
no new dependency, crate, target directory, or Cargo invocation.

## Product Verification

Rust-only tests exercise the real CLI and real Rust source files:

1. A top-level exact function emits a claim-only SAFE cue.
2. `handleBulkDelete` finds `EventDispatcher::handleDelete` through the copied
   shared-prefix policy, emits REVIEW, and never appears as SAFE definition
   evidence.
3. An exact impl method remains REVIEW with distance zero.
4. A matching method under a test/generated-classified path becomes MUTED by
   the existing source-health classification.
5. A missing name with a review-visible custom macro or non-test cfg gate is
   `NOT_OBSERVED` with opaque `taintedBy` evidence, never an absence claim.
6. Missing intent arrays normalize with warnings; present malformed arrays
   hard-stop with exit code 2.
7. Non-empty unsupported lanes are preserved and marked unsupported.
8. `taskId` round-trips as typed data and an unknown extra field hard-stops
   instead of entering a `Value` map.
9. Repeated runs produce identical advisory content apart from no timestamp;
   the pre-write artifact contains no generated timestamp.

Tests do not invoke Node, use fake analyzer outputs, or assert file existence as
the behavior under test.

## Acceptance Criteria

- `lumin-rust-analyzer pre-write` completes the Rust name-cue vertical slice
  from real AST extraction through typed advisory output.
- Existing analyzer invocations and unified artifact shape remain compatible.
- No new timeout, cap, threshold, path policy, or SAFE/REVIEW/MUTED rule exists
  without an exact TS/JS or source-health owner.
- Full Rust workspace format, lint, and test CI passes.
- A PR records the TS/JS owner-to-Rust destination mapping and is merged only
  after Rust CI and review are clean.

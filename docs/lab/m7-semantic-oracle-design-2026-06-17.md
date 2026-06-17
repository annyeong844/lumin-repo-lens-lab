# M7 Semantic Oracle Design

Date: 2026-06-17

## Decision

M7 adds a cross-language semantic oracle contract. It does not replace syntax
health, topology, shape, deadness, or existing TS/JS parser surfaces.

The product rule is:

```text
cheap syntax finds candidates;
expensive semantic oracles verify only the claims they are authorized to verify;
coverage gaps are visible;
clean requires a ran oracle;
semantic silence refutes nothing unless the oracle registry says it does.
```

This is a product identity decision, not a Rust-only feature. Lumin is an
evidence engine, not a lint runner. Tool output is evidence. The claim layer
must read confidence, coverage, oracle authority, and analysis input identity
before saying anything.

## Why This Exists

Depwire-style tree-sitter analysis showed the failure mode clearly:
deterministic parsing is not deterministic meaning. Comment stripping, regex
filters, and AST patterns can reduce noise, but they cannot prove semantic
deadness, type safety, borrow correctness, runtime reachability, or clone cost.

The TS/JS path already learned the same lesson. OXC and AST passes are useful
for fast structure. The TypeScript compiler API is used where TypeScript itself
owns the meaning, such as `tsconfig` parsing and path resolution. Unsupported
shape cases emit diagnostics instead of fabricated facts. Missing capability is
not treated as clean.

M7 generalizes that proven pattern.

## Non-Goals

- Do not turn Lumin into ESLint, Clippy, Ruff, or a generic lint aggregator.
- Do not promote syntax-only M6 findings to semantic findings.
- Do not add Clippy to the first Rust semantic oracle.
- Do not claim Python runtime reachability from static type output.
- Do not hide unavailable semantic coverage behind clean summaries.
- Do not let semantic silence refute syntax candidates without registry
  authority.
- Do not change stable public command behavior in this design packet.

## Canonical Sources

M7 is governed by:

- `canonical/evidence-ladder.md`
- `canonical/oracle-registry.json`
- `canonical/invariants.md` §10

The registry is the source of truth for oracle slots. The markdown explains the
rules and review language.

The first Rust implementation lives under
`experiments/rust-main/rust-cargo-oracle`. The earlier M7 cargo-oracle
JavaScript prototype is not an implementation surface; cargo diagnostic
classification and artifact emission are owned by the Rust binary.

## Evidence Model

M7 separates source, confidence, and coverage. It also binds semantic evidence
to a full analysis input set.

```json
{
  "source": {
    "oracleId": "rust.cargo-check",
    "version": "rustc 1.xx.x",
    "command": "cargo check --message-format=json",
    "registryContentHash": "sha256:..."
  },
  "confidence": {
    "tier": "verified",
    "authorityIds": [
      "rust.rustc.error-diagnostic"
    ],
    "claimKind": "verified.rust.rustc-error-diagnostic"
  },
  "coverage": {
    "status": "ran",
    "clean": false,
    "cleanKind": "verified-rustc-error-absence",
    "cleanScope": "rust.cargo-check verified rustc error diagnostics for package+target+featureSet+targetTriple+cfgSet+profile",
    "scope": {
      "kind": "crate-target-configuration",
      "package": "lumin-rust-source-health",
      "target": "lib",
      "featureSet": ["default"],
      "targetTriple": "x86_64-pc-windows-msvc",
      "cfgSet": ["debug_assertions"],
      "cfgSetComplete": false,
      "profile": "dev"
    },
    "analysisInputSetHash": "sha256:..."
  }
}
```

`unanalyzed` is not a confidence tier. It is represented as coverage:

```json
{
  "coverage": {
    "status": "not-run",
    "reason": "semantic oracle was not requested"
  }
}
```

Unavailable is distinct from unsupported:

```json
{
  "coverage": {
    "status": "unavailable",
    "reason": "Rust toolchain missing"
  }
}
```

```json
{
  "coverage": {
    "status": "unsupported",
    "reason": "oracle does not support this language surface"
  }
}
```

## Registry Requirements

`canonical/oracle-registry.json` must remain machine-readable enough for
validators and renderers to branch without tool-name shortcuts.

Each oracle entry must include these common fields:

- `authorityIds`: stable ids for what the oracle may verify;
- `claimKinds`: stable ids for claims that may be emitted;
- `candidateKinds`: stable ids for candidates emitted by candidate layers;
- `refutesCandidateKinds`: explicit refutation relations, empty when silence
  refutes nothing;
- `analysisInputSet`: the inputs that must be included in
  `analysisInputSetHash`;
- `sourceKind`: provenance style, such as `syntax`, `syntax-heuristic`,
  `semantic-oracle`, or `rule-engine`.

The following fields are required when applicable:

- `coverageDimensions`: stable dimensions that must be part of the coverage
  scope, such as Rust feature set, target triple, and cfg set.
- `diagnosticClassification`: when one command stream contains mixed confidence
  classes, such as cargo JSON `error` diagnostics and rustc lint diagnostics.
- `completionSignal`: the event required before silence can become clean.

Prose fields may stay for humans, but implementation must use the stable ids.

Artifact metadata must include:

```json
{
  "oracleRegistryVersion": "oracle-registry.v1",
  "registryContentHash": "sha256:..."
}
```

Registry source-of-truth must be checked, not trusted by convention:

- markdown oracle ids must be registry ids;
- prose confidence tier names must exist in `confidenceTiers`;
- implementation-emitted oracle ids, claim kinds, candidate kinds, and
  refutation relations must be registry-listed.

## Partial Coverage

Semantic diagnostics and absence coverage are different ledgers.

Rust example:

1. `cargo check` starts for a crate target.
2. rustc emits a user-code primary span type error.
3. a dependency build script then fails before full target coverage completes.

The emitted user-code diagnostic can become a `verified` finding if it is inside
`rust.cargo-check` authority. The crate-target absence claim is still
`unavailable`, not `ran.clean`.

Cargo JSON streams also mix confidence classes:

- `compiler-message` with `level: "error"` and a user-code primary span may be
  `verified` when the diagnostic is inside rustc authority.
- rustc lint diagnostics from the same stream are `rule-backed`, not verified,
  even when `-D warnings` or `#![deny(warnings)]` raises them to
  `level: "error"`.
- classification is namespace-first: `codeNamespace: "rustc-lint"` wins before
  `level`.
- verified rustc error classification also requires a rustc error code matching
  `E[0-9]+`; M7 uses the generic claim kind
  `verified.rust.rustc-error-diagnostic` until code-family mapping is reviewed.
- codeless rustc errors may be verified only when they carry a user-code primary
  span and are not rustc lints. Summary or abort messages without such a span
  remain candidates or diagnostics.
- unmatched cargo diagnostics are visible diagnostics or candidates; they are
  never silently dropped and never promoted to verified.
- an empty diagnostic stream is not clean; clean coverage requires
  `build-finished` with `success: true` for the same package, target, feature
  set, target triple, cfg set, profile, and analysis input set.

User-code primary span means the primary span resolves to a workspace member
source file, excluding `target/` and `OUT_DIR` generated files. Macro expansion
chains resolve to the nearest user-written span. If no user-written span can be
found, the diagnostic is not user-code-primary for verified classification.

Implementation must store:

- `findings[]` for emitted diagnostics and review signals;
- `coverage[]` for oracle execution state and clean/absence claims.

One verified finding does not prove complete verified coverage.

## Language Slots

### TS/JS

Existing pattern:

- candidate layer: OXC/parser-backed structural facts;
- verified semantic layer: TypeScript compiler API for `tsconfig`, `extends`,
  `baseUrl`, `paths`, and related compiler-owned config semantics;
- unsupported semantic cases: diagnostics, not fake facts;
- missing capabilities: `[확인 불가]`, not clean.

M7 does not rewrite TS/JS. It names the pattern so Rust and Python do not invent
different evidence semantics.

TS/JS semantic input sets include `tsconfig`, extends chains, `baseUrl`,
`paths`, package boundary files, TypeScript compiler version, and compiler API
mode.

### Rust

First Rust M7 oracle:

```text
cargo metadata
cargo check --message-format=json
```

M7 Rust verified authority:

- workspace/package/target discovery from `cargo metadata`;
- rustc JSON diagnostics for type, borrow, name-resolution, cfg-expanded crate
  diagnostics, and user-code primary spans;
- coverage ledger by package, target, feature set, target triple, and cfg set.

M7 Rust does not include Clippy. Clippy is valuable later, but it is
rule-backed evidence. `clippy::correctness` may deserve high display priority,
but it must not dilute `verified`.

Rust M7 also reserves `rust.ra-hir` as a deferred middle semantic tier for
offline or unbuildable repositories. It is not part of first implementation, but
the registry keeps the slot so future name/type-resolution evidence does not get
invented as ad hoc syntax claims.

When `rust.ra-hir` is promoted later, conflicts with `rust.cargo-check` resolve
toward rustc/cargo-check as the ground-truth oracle.

Rust semantic input sets include:

- local package Rust source bytes;
- `Cargo.toml`;
- `Cargo.lock`;
- `rust-toolchain*`;
- selected package;
- selected target;
- selected feature flags;
- target triple;
- cfg set;
- profile;
- cargo and rustc versions;
- relevant cargo config;
- environment values that affect compilation;
- build script and proc-macro influence.

Build script and proc-macro influence may be impossible to statically enumerate.
When that influence set is incomplete, semantic cache reuse for the affected
scope is disabled. The implementation must rerun the oracle or mark coverage
unavailable; it must not reuse verified semantic results from an incomplete
`analysisInputSetHash`.

Rust unavailable cases are expected and first-class:

- missing Cargo manifest;
- missing toolchain;
- target triple or toolchain not installed;
- offline dependency fetch;
- proc-macro or build script failure;
- dependency failure without user-code primary span.

Rust unsupported cases mean the oracle does not support the requested language
surface or analysis target. Requested cfg, feature, package, or target surfaces
that were not selected are `not-run`.

Those are coverage gaps unless the oracle emits a user-code diagnostic inside
its declared authority.

Do not render "crate clean." Render "clean for this package, target, feature
set, target triple, cfg set, profile, oracle, authority, and analysis input
set."

The first cargo-check slice may emit `cfgSetComplete: false`. That means the
runner captured explicit `--cfg` values from supported environment/config
sources, but did not prove the full Cargo/rustc cfg universe. Render that as a
best-effort cfg scope, not exhaustive cfg cleanliness.

### Python

Initial Python pattern, when implemented:

```text
vulture-like candidate collection
ty semantic oracle
```

`ty` is preferred as the first Python semantic slot because it is Rust-native
and actively developed. That is a provenance and maintenance argument, not a
license to overclaim.

Python verified authority must remain narrow:

- static type diagnostics inside `ty`'s configured analysis model;
- no claim about dynamic import execution, monkeypatch behavior, runtime
  reachability, or untyped runtime contracts.

Python semantic input sets include source bytes, `pyproject.toml`, ty
configuration, Python version, import roots, dependency and stub search paths,
and ty version.

`ty` must be version-pinned before clean claims depend on it. Diagnostic code or
format drift is a registry and policy change.

`ty` silence is narrow. No ty diagnostics means ty-clean under ty's configured
model, not Python type-clean in general. The registry also reserves a deferred
`python.pyright` slot so the Python semantic lane can be swapped or compared
without rewriting confidence semantics.

## Candidate Confirmation Flow

M7 should not run expensive semantic oracles for every syntax scan by default.

Recommended flow:

1. Syntax layer emits candidates and affected scopes.
2. Candidate scopes are grouped by semantic oracle scope:
   - Rust: crate package/target.
   - Python: package/module scope.
   - TS/JS: tsconfig scope where applicable.
3. Semantic oracle runs only when requested or when a product mode explicitly
   requires verified coverage.
4. Oracle JSON diagnostics are matched back to source spans and scopes.
5. Matched diagnostics become `verified` only inside oracle authority.
6. Unmatched candidates remain candidates.
7. Compiler silence is not automatic refutation unless the registry lists the
   candidate kind in `refutesCandidateKinds` for that oracle and scope relation.

This preserves speed while giving high-trust paths a way to pay for semantic
certainty.

## Refutation Policy

M7 starts conservative:

```text
semantic silence refutes no syntax candidate unless oracle-registry.json
explicitly lists the candidate kind and scope relation.
```

For example, `cargo check` silence must not mean:

- `.clone()` is cheap;
- panic is unreachable;
- syntax deadness is false;
- unsafe code is sound.

Refutation requires:

- registry-listed `refutesCandidateKinds`;
- matching coverage scope;
- `coverage.status === "ran"`;
- matching `analysisInputSetHash`;
- renderer citation of the refuting oracle and authority id.

When the first refutation relation is added, use this object shape:

```json
{
  "candidateOracleId": "rust.ra-ap-syntax",
  "candidateKind": "rust.parse-error.syntax",
  "scopeRelation": "same-file",
  "requiresCoverageStatus": "ran",
  "authorityId": "rust.rustc.cfg-expanded-diagnostic"
}
```

## Artifact Shape

This design does not finalize one artifact filename. The implementation plan
should choose one of these after review:

- `semantic-health.json` for cross-language semantic coverage;
- `rust-semantic-health.json` for a Rust-only first slice;
- a new section in an existing artifact only if existing contracts can remain
  backward-compatible.

Required fields regardless of filename:

```json
{
  "schemaVersion": "semantic-health.v1",
  "policyVersion": "evidence-ladder.v1",
  "meta": {
    "generated": "2026-06-17T00:00:00.000Z",
    "oracleRegistryVersion": "oracle-registry.v1",
    "registryContentHash": "sha256:..."
  },
  "coverage": [],
  "findings": [],
  "diagnostics": []
}
```

Each finding must carry:

- `source.oracleId`;
- `source.version`;
- `source.command` or API mode;
- `source.commandArgs` when the source is a CLI command;
- `source.registryContentHash`;
- `confidence.tier`;
- `confidence.authorityIds`;
- `confidence.ruleIds` when the finding is rule-backed;
- `confidence.claimKind`;
- `coverageRef`;
- `span`;
- `message`;
- `code` when the oracle provides one;
- `analysisInputSetHash`.

Each coverage entry must carry:

- oracle id;
- scope;
- status;
- `clean` only when status is `ran`;
- `cleanKind` when clean is present;
- `cleanScope` when clean is present;
- reason for non-ran statuses;
- `analysisInputSetHash`;
- command metadata (`command` and `commandArgs`);
- process result metadata (`exitCode` and `elapsedMs`) where applicable.

## Stale Evidence

M7 must treat stale verified evidence as a coverage gap.

Rules:

- If the analysis input set hash for a scope changes, prior findings for that
  scope are no longer verified.
- Cached semantic results may be reused only when oracle id, oracle version,
  policy version, registry content hash, command mode, scope, and
  `analysisInputSetHash` all match.
- Rust cargo-check `analysisInputSetHash` includes local package Rust source
  bytes. Manifest, lockfile, and toolchain hashes alone are not enough to reuse
  or trust prior semantic evidence.
- Reused coverage must say it is reused and cite the cache key.

## Rendering Contract

Renderers and ranking code must branch on confidence and coverage, not tool
names.

Allowed:

- `verified` + matching authority → grounded finding.
- `rule-backed` → rule-backed review evidence.
- `candidate` → review prompt only.
- `not-run` / `unavailable` / `unsupported` → visible coverage gap.

Forbidden:

- candidate rendered as confirmed finding;
- unavailable rendered as unsupported;
- not-run rendered as clean;
- stale verified rendered as current verified;
- tool-name-specific shortcuts that bypass the registry;
- semantic silence used as refutation without registry authority.
- Rust clean rendered without package, target, feature set, target triple, cfg
  set, profile, oracle, authority, and analysis input set.

## M6 Amendment

M6 Rust syntax health remains useful. Its authority is narrowed:

```text
M6 emits syntax-only candidates and counters.
M6 does not emit semantic findings.
M6 does not prove clone cost, panic reachability, borrow behavior, or deadness.
M6 clean means syntax-health clean only, never semantic clean.
```

Any M6 output that looks claim-like must carry `candidate` confidence or a
syntax-only caveat.

## Implementation Gate

Before implementation starts, reviewers should approve:

- the evidence ladder canon;
- the oracle registry shape;
- the Rust M7 initial scope: `cargo metadata` + `cargo check` only;
- the deferral of Clippy to a later rule-backed lane;
- the stale evidence policy;
- the partial coverage model;
- the registry refutation model;
- whether the first artifact is Rust-only or cross-language.

## Acceptance Checklist

- Candidate cannot be rendered as a verified finding.
- Not-run coverage cannot express clean.
- Unavailable and unsupported are distinct in artifacts and UI text.
- Cargo check dependency/toolchain failure without user-code primary span becomes
  coverage unavailable, not a finding.
- Cargo check user-code primary diagnostics may be verified even when target
  absence coverage is unavailable.
- Cargo check rustc lint diagnostics are rule-backed, not verified.
- Denied rustc lints remain rule-backed even when emitted as `level: "error"`.
- Codeless rustc errors are verified only with user-code primary spans.
- Cargo note, help, and failure-note messages are non-findings.
- Non-user-code primary diagnostics are not user-facing findings; non-user-code
  primary errors make absence-clean coverage unavailable.
- Cargo diagnostics that match no verified or rule-backed classification rule
  are surfaced as candidates only when they are real user-code error or warning
  diagnostics.
- Empty cargo diagnostic streams are not clean without complete stream parsing
  and `build-finished` success.
- User-code primary span is defined and excludes dependency files, `target/`,
  `OUT_DIR`, and unresolved macro expansion spans.
- Build-script/proc-macro scopes do not reuse semantic cache entries unless the
  complete influence set is captured.
- Rust semantic clean is scoped to package, target, feature set, target triple,
  cfg set, profile, oracle, authority, and analysis input set.
- Rustc JSON user-code primary diagnostic becomes verified only inside rustc
  authority.
- Stale `analysisInputSetHash` drops verified evidence to a coverage gap.
- `clean: true` is always scoped to oracle, authority, scope, and analysis input
  set.
- Registry content hash is recorded with findings and coverage.
- Semantic silence only refutes registry-listed candidate kinds.
- `canonical/oracle-registry.json` remains the single source of truth for oracle
  ids, authority, claim kinds, refutation, and confidence.

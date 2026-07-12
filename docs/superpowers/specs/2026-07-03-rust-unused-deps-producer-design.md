# Rust Unused Deps Producer Design

## Goal

Move `unused-deps.json` production from JS/MJS into `lumin-audit-core` while
preserving the existing review-only dependency hygiene contract.

This is a producer migration slice, not a package-removal feature. The artifact
must keep saying "review this dependency declaration" and must never imply
safe removal, lockfile edits, `SAFE_FIX`, SARIF findings, or package-manager
commands.

## Current State

JS currently owns the producer:

- `build-unused-deps.mjs` reads repo/package evidence and writes
  `unused-deps.json`.
- `_lib/unused-deps-artifact.mjs` owns package identity normalization,
  package-script tool evidence, package-scope matching, workspace-internal
  muting, and dependency classification.
- `audit-repo.mjs` executes `build-unused-deps.mjs` after
  `build-symbol-graph.mjs`.

Rust currently owns only shallow consumption:

- `artifact_summaries.rs` summarizes an already-produced `unused-deps.json`
  into `manifest.json.unusedDependencies`.
- `manifest_evidence.rs` reads that artifact as optional evidence.
- `orchestration_plan.rs` still marks `build-unused-deps.mjs` as
  `producerOwner: "js-mjs"`.

The migration should make Rust own the producer semantics, then leave the JS
entrypoint as a thin compatibility wrapper until the package surface no longer
needs it.

## Non-Goals

- Do not remove `unused-deps.json`.
- Do not change schema version or policy version.
- Do not add dependency allowlists or new package-manager profiles.
- Do not execute package scripts.
- Do not read lockfile graph semantics.
- Do not infer framework plugin dependency use beyond evidence already modeled
  by the existing producer contract.
- Do not run or expand the legacy Node umbrella as a required gate.

## Rust Owner Boundary

Add a new Rust owner module:

```text
experiments/rust-main/lumin-audit-core/src/unused_deps.rs
```

It owns:

- `unused-deps.v1` artifact construction
- `unused-deps-review-policy-v1` policy fields
- package specifier normalization
- dependency declaration collection from JS-supplied package records
- package-script tool evidence extraction
- package-scope consumer matching
- dependency classification into `used`, `muted`, `review-unused`,
  `confidence-limited`, or `unavailable`
- deterministic summary and package ordering

It must not own:

- JS/TS symbol graph production
- source parsing
- package manager execution
- manifest summary projection already owned by `artifact_summaries.rs`
- audit orchestration sequencing beyond exposing a CLI/wrapper target

`canonical/audit-core.md` must be updated before implementation to name
`unused_deps.rs` as the producer owner and to state that the JS entrypoint is
only compatibility wrapping.

## CLI Shape

Add an audit-core command:

```text
lumin-audit-core unused-deps-artifact --input <path|-> [--result-output <path>]
```

The input is a typed request assembled by the JS wrapper:

```json
{
  "schemaVersion": "lumin-unused-deps-producer-request.v1",
  "root": "/repo",
  "includeTests": true,
  "exclude": [],
  "packageRecords": [
    {
      "root": "/repo",
      "relRoot": ".",
      "packageJson": {}
    }
  ],
  "symbols": {}
}
```

The output is the existing `unused-deps.json` artifact shape. In
`--result-output` mode, Rust writes the full artifact to the result file so the
Node child-process stdout buffer cannot become a repository-size limit.

The initial JS wrapper may still write `unused-deps.json` after receiving the
Rust result. A later slice can let audit-core write the artifact directly if
the broader orchestrator owns that write path.

## Shared Rust Producer CLI Contract

Producer commands must accept `--input <path|->` and may write either to stdout
or to `--result-output <path>`. JS wrappers must use `--result-output` for
normal repository runs.

Rust must write artifact JSON only to the selected result channel. Diagnostics
must go to stderr. Invalid JSON, schema mismatch, invalid normalized paths, and
failed result-file writes must exit non-zero and must not produce a partial
success artifact.

Wrappers must treat a non-zero exit, missing result file, or malformed result
JSON as producer failure rather than falling back to JS classification.

## Request Schema Details

`symbols.dependencyImportConsumers` entries are the only import-consumer input
for this producer. Each consumer is interpreted as:

```json
{
  "file": "src/app.ts",
  "fromSpec": "@scope/pkg/subpath",
  "depRoot": "@scope/pkg",
  "kind": "import",
  "source": "symbols.json.dependencyImportConsumers",
  "typeOnly": false
}
```

- `file` is required. It may be repo-relative, `./`-prefixed, or absolute under
  the normalized request `root`; absolute paths under `root` are converted to
  repo-relative paths before package-scope matching.
- backslashes normalize to `/`.
- `depRoot` wins when present; otherwise Rust derives the dependency root from
  `fromSpec`.
- `fromSpec`, `kind`, `source`, and `typeOnly` are evidence fields. Missing
  `kind` defaults to `import`; missing `source` defaults to
  `symbols.json.dependencyImportConsumers`.
- consumers outside the owning package scope do not satisfy that package's
  dependency declarations.

The package declarations come only from JS-supplied `packageRecords[].packageJson`
fields. Rust does not read package files directly in this slice.

## Product Semantics To Preserve

### Package Identity

- `react/jsx-runtime` normalizes to `react`.
- `@scope/pkg/subpath` normalizes to `@scope/pkg`.
- relative, absolute, `node:`, URL, and import-map-like specifiers do not become
  package dependencies.

### Package Scope

Consumers only count for the package scope that owns their file.

- A root package excludes files under child workspace package roots.
- A child package owns files under its own `relRoot`.
- Sibling package consumers must not satisfy another package's declaration.

### Classification

Classification order must match the JS producer:

1. observed external import consumer -> `used` /
   `external-import-consumer`
2. direct package-script tool evidence -> `muted` /
   `package-script-tool`
3. `peerDependencies` -> `muted` / `peer-contract`
4. `optionalDependencies` -> `muted` / `optional-runtime`
5. `@types/*` -> `muted` / `ambient-types`
6. dependency name matches another workspace package -> `muted` /
   `workspace-internal`
7. otherwise -> `review-unused` / `no-observed-consumer`

Each declaration is classified independently. If the same package appears in
multiple dependency fields, observed import evidence makes each declaration
`used`; otherwise the field-specific mute rules still apply per declaration.

`confidence-limited` is part of the artifact vocabulary but is not newly emitted
by this slice unless already emitted by the checked JS producer. Any emission
condition must be covered by a Rust test before implementation.

If `symbols.meta.supports.dependencyImportConsumers` is not true or
`symbols.dependencyImportConsumers` is absent, the artifact must be
`status: "unavailable"` with `reason: "input-artifact-missing"`, not a
zero-unused claim.

### Package Script Tool Evidence

Script command tokenization preserves checked JS behavior; it is not a
shell-perfect parser. Unsupported shell constructs must preserve current JS
behavior rather than becoming a new Rust-only interpretation in this slice.

The test matrix should cover direct tools, `cross-env vite`, `FOO=bar vite`,
quoted commands, `node ./node_modules/.bin/vite`, `npx`, `bunx`, `npm exec`,
`pnpm exec`, `pnpm dlx`, and `npm run` wrapper non-evidence.

## Wrapper Strategy

`build-unused-deps.mjs` becomes a thin wrapper:

1. parse CLI flags through the existing JS helper;
2. keep `detectRepoMode(...)` for the first Rust slice unless a separate Rust
   repo-mode owner is introduced;
3. read `symbols.json`;
4. build the Rust request payload;
5. call `lumin-audit-core unused-deps-artifact --result-output <temp>`;
6. write the returned artifact to `unused-deps.json`;
7. keep the current console summary wording.

This keeps package/workspace discovery unchanged while moving the actual
dependency hygiene classification into Rust. The wrapper must not keep semantic
reason strings, classification order, or package-script classifier tables after
this slice.

`orchestration_plan.rs` should then mark the step as `ProducerOwner::Rust`.
The step name may remain `build-unused-deps.mjs` while the public wrapper
exists, but `producerOwner` must reflect that the artifact semantics are
Rust-owned.

## Tests

Cargo tests are authoritative for the migrated producer.

Required Rust tests:

- golden fixture parity with canonical JSON equivalence, including ordering;
- static import consumer classifies as `used`;
- package script tool classifies as `muted`;
- peer, optional, ambient type, and workspace-internal declarations stay muted;
- declaration with no consumer or explanation becomes `review-unused`;
- duplicate declarations across dependency fields classify independently;
- missing dependency-import-consumer support emits `unavailable`;
- root package consumers do not leak into child package declarations;
- child package consumers do not satisfy sibling package declarations;
- command token parsing preserves the checked JS behavior for direct tools,
  `npx`, `bunx`, `npm exec`, `pnpm exec`, and `npm run` wrapper non-evidence;
- output package/dependency/evidence ordering is deterministic.

Wrapper tests should be minimal and Node-only if needed:

- `node tests/test-unused-deps-producer.mjs` can remain as compatibility
  coverage while it calls the Rust-backed wrapper.
- Vitest is reference coverage only during this migration and is not required
  as the authoritative gate for this slice.

Verification commands for the implementation PR:

```text
npm run test:audit-runtime-gate
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test unused_deps
node tests/test-unused-deps-producer.mjs
```

Do not run the full legacy audit umbrella as part of this slice.

## Acceptance

This slice is complete when:

- `unused_deps.rs` owns artifact construction;
- `canonical/audit-core.md` records that owner boundary;
- `build-unused-deps.mjs` is a thin Rust wrapper, not a second classifier;
- semantic reason strings and classification order exist only in Rust;
- `orchestration_plan.rs` reports the step as Rust-owned;
- the artifact shape remains `unused-deps.v1` /
  `unused-deps-review-policy-v1`;
- golden fixture parity proves deterministic ordering and existing artifact
  shape;
- existing review-only safety invariants still hold;
- Cargo tests cover the migrated producer behavior;
- the checked Node wrapper test passes without restoring the 16-minute legacy
  umbrella as a default gate.

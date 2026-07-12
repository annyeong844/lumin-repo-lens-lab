# Audit Artifact Registry Rust Core Design

## Goal

Move the audit artifact registry and the first manifest evidence summaries from
JS-owned ad hoc objects into a typed Rust core, while preserving every existing
JS/TS producer lane until parity is proven.

This is the first slice toward moving `audit-repo.mjs` orchestration into Rust.
It is deliberately not a full orchestrator rewrite.

## Context

Current JS owners:

- `audit-repo.mjs` owns orchestration, child process execution, lifecycle modes,
  command telemetry, producer performance, summary rendering, and manifest
  finalization.
- `_lib/audit-manifest.mjs` owns artifact enumeration and manifest evidence
  projection.
- `_lib/artifacts.mjs` owns JSON reads and read metrics.

Current Rust owners:

- `experiments/rust-common` owns low-level shared helpers: path, JSON writes,
  hash, and usage-error classification.
- `experiments/rust-main/lumin-rust-analyzer` owns Rust analysis and Rust
  pre-write.
- No Rust crate currently owns audit manifest or artifact-registry product
  semantics.

The first Rust migration point must avoid touching JS/TS analysis behavior. The
safe seam is the artifact registry: it reads already-produced artifacts and
summarizes typed metadata. It does not execute producers or parse source code.

## Non-Goals

- Do not rewrite `audit-repo.mjs` in this slice.
- Do not move `symbols.json`, `shape-index.json`, `function-clones.json`,
  `dead-classify.json`, resolver, SFC, or any JS/TS producer into Rust.
- Do not make `--rust-analyzer` default in this slice.
- Do not add elapsed-time caps, repository-size caps, or timeout logic.
- Do not change JS/TS analysis claims or route JS/TS files through Rust.
- Do not add a new Node test migration batch as part of the Rust core design.

## Recommended Approach

Create a new workspace crate:

`experiments/rust-main/lumin-audit-core`

Do not put this in `rust-common`. `rust-common` is a low-level helper crate; an
audit manifest registry is product-domain logic. Mixing those would recreate the
same owner drift this migration is trying to remove.

Do not put this in `lumin-rust-analyzer`. The analyzer owns Rust code evidence,
not whole-audit manifest semantics.

## First Slice

### 1. Typed Artifact Registry

Rust owns the registry currently represented by:

- `_lib/audit-manifest.mjs` `ARTIFACT_CANDIDATES`
- `_lib/audit-manifest.mjs` `DYNAMIC_ARTIFACT_PATTERNS`
- `_lib/audit-manifest.mjs` `collectProducedArtifacts(outDir)`

Rust output must preserve existing behavior:

- Include known static artifacts when present.
- Include dynamic artifacts matching the existing pre-write, post-write,
  any-inventory, and canon-drift filename patterns.
- Return sorted artifact names.
- Keep stale Rust analyzer artifacts out of `manifest.artifactsProduced` when
  the Rust analyzer was not requested or did not run in the current invocation.
  That filtering currently happens in `audit-repo.mjs`
  `collectManifestProducedArtifacts(rustAnalysis)` and must remain part of the
  product contract.

### 2. Typed Rust Analyzer Artifact Summary

Rust owns the summary currently represented by:

- `_lib/audit-manifest.mjs` `rustScanScopeFromArtifact`
- `_lib/audit-manifest.mjs` `buildRustAnalysisSummary`

The Rust core must classify `rust-analyzer-health.latest.json` as:

- absent: `null` summary, preserving current JS caller behavior
- `root-mismatch`
- `invalid-shape`
- `complete` / `available: true`

The `complete` projection must preserve:

- artifact name
- status
- availability
- schema version
- policy version
- producer
- mode
- source-health profile
- semantic mode
- scan scope copied from `meta.input` or `phases.syntax.meta.input`
- file count
- syntax review signal count
- review opaque surface count
- clone exact/structure/signature/near counts
- action tier summary
- oracle bridge status

The root check must use canonical path comparison and must not rely on string
equality for Windows path spelling.

### 3. Thin JS Wrapper

JS remains the public entrypoint for this slice.

`_lib/audit-manifest.mjs` should call the Rust core for the new typed pieces and
keep the rest of `buildManifestEvidence` in JS until subsequent explicitly
scoped slices migrate each summary. If the Rust core is unavailable during
development, the behavior must be explicit:

- product package path: report unavailable evidence or keep using the existing
  JS implementation only behind a documented transition flag
- tests/spec path: fail loudly rather than silently drifting

The wrapper must not create a second Rust summary implementation that keeps
growing after the Rust core exists.

## CLI Shape

The Rust core must expose a small CLI for compatibility with the current JS
entrypoint:

```text
lumin-audit-core artifact-registry --output <dir> [--rust-analysis-ran]
lumin-audit-core rust-analysis-summary --root <repo> --artifact <path>
```

The CLI emits JSON to stdout. It does not write files in the first slice.

Preferred library shape:

```rust
pub fn collect_produced_artifacts(
    out_dir: &Path,
    rust_analysis_usable: bool,
) -> Result<Vec<String>>;

pub fn summarize_rust_analysis_artifact(
    root: &Path,
    artifact: &serde_json::Value,
) -> RustAnalysisSummary;
```

The implementation may deserialize typed subsets instead of full artifacts.
Unknown extra fields must be ignored.

## Evidence And Tests

Tests must prove product behavior, not merely that functions exist.

Rust behavior tests:

- static and dynamic artifact files are enumerated in deterministic order
- stale `rust-analyzer-health.latest.json` is excluded when
  `rust_analysis_usable = false`
- Rust analyzer artifact with mismatched root returns `root-mismatch`
- malformed Rust analyzer artifact returns `invalid-shape`
- valid Rust analyzer artifact returns `complete` and preserves scan scope

JS wrapper parity checks:

- Existing audit-manifest tests continue to pass.
- The stale Rust artifact case still removes
  `rust-analyzer-health.latest.json` from `manifest.artifactsProduced`.
- Review pack and summary output continue to mention Rust analyzer evidence only
  when `manifest.rustAnalysis.status === "complete"` and `available === true`.

No Node command is required while drafting this spec. Implementation review may
run Node only when explicitly approved.

## Canonical Updates

Before implementation, add canonical ownership for the new crate:

- `canonical/rust-source-health.md` should not own this crate; it is Rust source
  health-specific.
- Add a new canonical section or small canonical file for audit-core ownership
  if no existing audit orchestration canon owns it.
- The owner entry must state that `lumin-audit-core` owns typed audit artifact
  registry and manifest evidence summaries, not source analysis.

## Migration Path

Subsequent explicitly scoped slices can move more manifest evidence into the
same Rust core:

1. artifact registry and Rust analyzer summary
2. generated artifact summary
3. dependency hygiene and block clone shallow summaries
4. resolver diagnostics and blind-zone summary
5. producer performance summary
6. audit-repo orchestration command graph

Each slice must preserve JS/TS producer behavior until the specific lane has a
Rust parity proof.

## Acceptance Criteria

- A new Rust core crate is the owner for artifact registry and Rust analyzer
  summary contracts.
- JS wrappers no longer hand-own those contracts after the Rust core is wired.
- Existing manifest shape stays compatible.
- No JS/TS producer lane changes behavior.
- No elapsed-time or repository-size cap is introduced.
- The goal remains active after this slice; this is progress toward the full
  orchestrator migration, not completion of the whole migration.

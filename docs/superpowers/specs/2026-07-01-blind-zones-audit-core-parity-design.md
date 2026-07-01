# Blind Zones Audit-Core Parity Design

## Goal

Prepare `manifest.json.blindZones` for a later Rust audit-core owner without
changing the current JS-owned behavior.

The migration target is a typed Rust projection in
`experiments/rust-main/lumin-audit-core`, but this slice is a parity design only.
`_lib/blind-zones.mjs` remains the product owner until the Rust port can prove
the same outputs on the protected JS behavior cases and real audit artifacts.

## Current Owner

`_lib/blind-zones.mjs` owns:

- `detectBlindZones(...)`
- `formatBlindZonesSummary(...)`
- language support interpretation from `symbols.meta.languageSupport`
- resolver confidence-gap policy through `resolver-blind-zone-policy`
- parser, CommonJS, SFC, HTML-entry, and unsupported-language blind-zone
  projection

`_lib/audit-manifest.mjs` calls `detectBlindZones` after Rust-owned manifest
evidence has been summarized. That call is intentionally still JS-owned.

## Non-Goals

- Do not move JS/TS producer behavior into Rust.
- Do not change threshold values or policy hashes.
- Do not run JS/TS files through Rust analysis.
- Do not replace `audit-repo.mjs` orchestration.
- Do not remove the JS implementation before parity evidence exists.
- Do not add elapsed-time caps, repository-size caps, or timeout behavior.

## Protected Semantics

The Rust port must preserve these behaviors exactly.

### Language And Shape Gaps

- Rust files produce a `rust` scan-gap only when current-run Rust analysis is
  not complete and available.
- SFC files (`vue`, `svelte`, `astro`) produce one grouped `sfc-scan-gap`, not
  per-extension noise.
- Python files produce `python-method-resolution` when Python extraction is
  enabled, and `python-scan-gap` when `symbols.meta.languageSupport.python`
  reports unavailable.
- Go files follow the same enabled/unavailable split.
- Unknown file share uses `resolver-blind-zone-policy.thresholds.shapeUnknownFileShare`.

### Resolver Confidence Gaps

Resolver blind zones trigger on any of:

- unresolved internal ratio at or above policy threshold
- absolute unresolved internal count at or above policy threshold
- concentrated unresolved prefix meeting the policy count/share thresholds

Resolver details prefer `resolver-diagnostics.json` summary fields when present,
then fall back to grouped `symbols.json` summaries, then fall back to raw
specifier records. The policy summary must keep the same policy id, version,
hashes, thresholds, and calibration fields.

### Precision Gaps

The Rust port must preserve:

- parser warnings from `symbols.meta.warnings`
- opaque CommonJS export surfaces from `symbols.cjsExportSurfaceByFile`
- dynamic CommonJS require calls from `symbols.cjsRequireOpacity`
- unresolved HTML module entrypoints from `entry-surface.json`

### Missing Inputs

Missing artifacts do not invent blind zones. Each branch is skipped when its
input artifact is missing or malformed for that branch. This is a product
contract, not a convenience fallback.

## Rust Port Shape

When implemented, use a new audit-core module:

`experiments/rust-main/lumin-audit-core/src/blind_zones.rs`

It should expose typed structs first:

```rust
pub struct BlindZoneSummary { ... }
pub struct BlindZoneDetails { ... }
pub fn summarize_blind_zones(input: BlindZoneInput<'_>) -> Vec<BlindZoneSummary>;
```

The CLI surface should be separate from `manifest-evidence-summary` at first:

```text
lumin-audit-core blind-zones-summary --input <fixture.json>
```

The first Rust CLI should consume a single fixture payload containing the same
objects the JS helper receives: `triage`, `symbols`, `deadClassify`,
`entrySurface`, `resolverDiagnostics`, and `rustAnalysis`. Keeping the first CLI
fixture-based avoids changing orchestrator wiring before parity is proven.

Only after parity is proven should `manifest-evidence-summary` grow an optional
blind-zone projection and `_lib/audit-manifest.mjs` become a thin wrapper for
that field.

## Parity Evidence Required Before Owner Change

Because Node execution is currently not part of this work loop, parity evidence
can be prepared as fixture JSON now and executed later in the JS lane when Node
is allowed.

Required fixture classes:

- clean TS/JS repo: zero blind zones
- Rust files with no complete Rust analysis: `rust` scan-gap
- Rust files with complete Rust analysis: no `rust` scan-gap
- grouped SFC scan gap
- Python enabled precision-gap
- Python unavailable scan-gap
- Go enabled precision-gap
- Go unavailable scan-gap
- resolver ratio trigger
- resolver absolute-count trigger
- resolver prefix-concentration trigger
- resolver details preference order
- parser parse-error precision-gap
- opaque CommonJS export precision-gap
- dynamic CommonJS require precision-gap
- unresolved HTML entry confidence-gap
- missing artifacts do not invent zones

Real artifact parity should also compare at least:

- one clean TS-only audit output
- one mixed TS/Rust audit output with successful Rust analysis
- one resolver-limited output containing grouped unresolved reasons
- one output with generated/SFC evidence

The comparison must be exact JSON equality after stable serialization. If field
ordering differs internally, the harness should compare parsed values, not raw
text.

## Migration Sequence

1. Keep current JS owner and this spec.
2. Add Rust typed `blind_zones.rs` plus fixture-based CLI.
3. Add Rust tests for the protected semantics above using real-shaped fixture
   payloads, not scaffolding existence tests.
4. When Node verification is allowed, run JS helper and Rust CLI against the
   same fixtures and real artifacts.
5. Only after parity passes, switch `_lib/audit-manifest.mjs` to call Rust for
   `blindZones`.
6. Keep `formatBlindZonesSummary` JS-owned unless console rendering also moves
   to Rust; it is presentation, not typed manifest evidence.

## Acceptance

This design is satisfied when:

- the owner boundary is documented in canonical audit-core notes;
- the future Rust module has a named owner and non-goals;
- the protected JS behaviors are explicitly listed;
- no current JS behavior changes in this slice.

# Blind Zones Audit-Core Owner Boundary

## Goal

Keep `manifest.json.blindZones` Rust-owned while preserving the JS producer
semantics that still feed the field.

The current migration state is no longer "future Rust port". The typed Rust
projection exists in `experiments/rust-main/lumin-audit-core/src/blind_zones.rs`
and is wired into `manifest_evidence.rs`. `_lib/blind-zones.mjs` remains the JS
parity oracle and console-summary owner until its remaining consumers are either
retired or moved behind an explicit Rust-owned rendering contract.

## Current Owner

Rust audit-core owns:

- final `manifest.json.blindZones` projection through
  `manifest_evidence.rs` calling `blind_zones.rs`
- typed `BlindZoneSummary` shape and severity vocabulary
- output-dir wiring through `lumin-audit-core manifest-evidence-summary`
- fixture/case parity CLI through `lumin-audit-core blind-zones-summary`
- current-run Rust-analysis gating for the `rs` scan-gap

`_lib/blind-zones.mjs` still owns:

- `detectBlindZones(...)`
- `formatBlindZonesSummary(...)`
- the checked JS parity/reference behavior for the same artifact inputs
- console one-line summary formatting
- Vitest-facing JS helper import surface

JS/TS producer artifacts still own their source semantics:

| Source artifact | Still JS-owned meaning | Rust blind-zone use |
|---|---|---|
| `triage.json` | language/file counts and shape counters | input evidence for language, SFC, Rust, and unclassified scan gaps |
| `symbols.json` | JS/TS graph, language support, parser warnings, CJS opacity, unresolved internal summaries | input evidence for precision and resolver confidence gaps |
| `resolver-diagnostics.json` | resolver failure grouping and diagnostic summaries | preferred resolver blind-zone details when present |
| `entry-surface.json` | HTML module entrypoint discovery | input evidence for `html-entry-surface` |
| `dead-classify.json` | deadness producer evidence | carried in the input shape; currently not interpreted by Rust blind zones |
| `rust-analyzer-health.latest.json` plus current run state | Rust analyzer availability and current-run freshness | clears or preserves the `rs` scan-gap |

## Non-Goals

- Do not move JS/TS producer behavior into Rust.
- Do not change threshold values or policy hashes.
- Do not run JS/TS files through Rust analysis.
- Do not replace `audit-repo.mjs` orchestration.
- Do not remove the JS helper while JS tests and console summaries still import
  it.
- Do not add elapsed-time caps, repository-size caps, or timeout behavior.

## Protected Semantics

The Rust projection must preserve these behaviors exactly.

### Language And Shape Gaps

- Rust files produce an `rs` scan-gap only when current-run Rust analysis is
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

## Implemented Rust Shape

The Rust module exists here:

`experiments/rust-main/lumin-audit-core/src/blind_zones.rs`

It exposes typed structs first:

```rust
pub struct BlindZoneSummary { ... }
pub fn summarize_blind_zones(input: BlindZoneInput<'_>) -> Vec<BlindZoneSummary>;
```

The standalone parity CLI exists:

```text
lumin-audit-core blind-zones-summary --input <fixture.json>
```

The Rust CLI consumes a single fixture payload containing the same
objects the JS helper receives: `triage`, `symbols`, `deadClassify`,
`entrySurface`, `resolverDiagnostics`, and `rustAnalysis`. It also accepts the
shared parity corpus as a batch fixture:

```text
lumin-audit-core blind-zones-summary --cases <cases.json>
```

The batch output is parity-runner evidence shaped as
`[{ "name": "...", "blindZones": [...] }]`. It is not a `manifest.json`
surface and must not be wired into `_lib/audit-manifest.mjs` until the JS
producer output has been compared against the same cases.

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

## Current Migration Sequence

Completed:

1. Rust typed `blind_zones.rs` plus fixture/case CLI.
2. Rust tests for protected semantics using real-shaped payloads.
3. Shared fixture corpus in `tests/fixtures/audit-core-blind-zones/cases.json`.
4. `manifest_evidence.rs` wires `summarize_blind_zones` into final
   `manifest.json.blindZones`.

Still pending:

1. JS helper vs Rust CLI exact parity runner over the shared cases.
2. Real artifact parity over at least the four outputs listed above.
3. A decision on whether to keep `formatBlindZonesSummary` JS-owned or move
   console rendering into audit-core under a separate rendering owner.
4. Removal or narrowing of `_lib/blind-zones.mjs` only after every JS consumer
   has a replacement owner.

Next safe implementation slice:

1. Add a JS parity runner that imports `_lib/blind-zones.mjs`, invokes
   `lumin-audit-core blind-zones-summary --cases`, and compares parsed JSON
   outputs for `tests/fixtures/audit-core-blind-zones/cases.json`.
2. Wire that runner into the existing focused blind-zone test surface, not the
   full audit path.
3. Keep `formatBlindZonesSummary` untouched.
4. Do not change threshold policies, source artifact parsing, or final manifest
   shape in the same slice.

## Acceptance

This owner boundary is satisfied when:

- `canonical/audit-core.md` names Rust as the final `manifest.json.blindZones`
  projection owner and `_lib/blind-zones.mjs` as JS parity/summary owner;
- the protected semantics above match the Rust tests and shared fixture corpus;
- `_lib/blind-zones.mjs` remains only for JS parity/reference and console
  summary work until those consumers move;
- no JS/TS producer behavior is reinterpreted in audit-core.

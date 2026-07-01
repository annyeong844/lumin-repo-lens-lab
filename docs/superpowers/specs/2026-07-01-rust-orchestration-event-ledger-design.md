# Rust Orchestration Event Ledger Design

## Goal

Move audit orchestration result evidence from JS-owned mutable arrays into a
typed Rust audit-core event ledger, without moving child process execution yet.

This is the next slice toward a Rust-owned `audit-repo` orchestrator. The public
entrypoint and producer execution remain JS/MJS for now, but the shape and
summary meaning of `commandsRun`, `skipped`, and `producer-performance.json`
should become Rust-owned.

## Current Owner

`audit-repo.mjs` currently owns three responsibilities in one place:

- producer execution through `runStep(...)` and `runRustAnalyzerStep(...)`;
- raw event accumulation through mutable `commandsRun` and `skipped` arrays;
- `producer-performance.json` construction through
  `buildProducerPerformanceArtifact(...)`.

`lumin-audit-core` already owns adjacent typed projections:

- `orchestration_plan.rs` owns the planned audit command graph;
- `producer_performance.rs` owns `manifest.json.performance` summary
  projection from an already-produced `producer-performance.json`;
- `orchestration_result.rs` owns `manifest.json.orchestration` summary
  projection from an already-produced `producer-performance.json`;
- `manifest_meta.rs` owns `manifest.json.meta`.

The missing layer is the raw execution ledger that sits between the Rust-owned
plan and the Rust-owned result summaries.

## Non-Goals

- Do not rewrite `audit-repo.mjs` in this slice.
- Do not make Rust spawn JS/MJS producer processes yet.
- Do not change any JS/TS producer behavior.
- Do not migrate `blindZones`; it remains JS-owned until parity is proven.
- Do not move human markdown rendering (`auditSummary`, `reviewPack`,
  `topologyMermaid`) into Rust.
- Do not add elapsed-time caps, repository-size caps, timeouts, or hardcoded
  producer limits.
- Do not infer child peak RSS. Existing memory evidence is orchestrator process
  snapshots only, and the artifact must keep saying that.

## Recommended Approach

Add a Rust audit-core module:

```text
experiments/rust-main/lumin-audit-core/src/orchestration_events.rs
```

This module owns a typed event ledger and the construction of
`producer-performance.json` from that ledger. JS remains the executor and sends
completed event records to Rust.

The first implementation should add a CLI:

```text
lumin-audit-core producer-performance-artifact --input <ledger.json>
```

The CLI reads a single ledger payload and emits the full
`producer-performance.json` shape to stdout. The JS wrapper can then replace
`buildProducerPerformanceArtifact(...)` with a thin call into audit-core while
leaving child process execution unchanged.

## Ledger Shape

The input shape should be explicitly Rust-owned and separate from
`producer-performance.json`:

```json
{
  "schemaVersion": "lumin-audit-orchestration-ledger.v1",
  "generated": "2026-07-01T00:00:00.000Z",
  "root": "/repo",
  "output": "/repo/.audit",
  "profile": "quick",
  "scanRange": {
    "includeTests": true,
    "production": false,
    "excludes": [],
    "autoExcludes": []
  },
  "cache": {
    "noIncremental": false,
    "cacheRoot": "/repo/.audit/.cache",
    "clearIncrementalCache": false
  },
  "generatedArtifacts": {
    "mode": "default"
  },
  "artifactReads": {
    "totalReadCount": 0,
    "totalReadBytes": 0,
    "totalJsonParseMs": 0
  },
  "artifacts": {
    "producedCount": 0,
    "totalBytes": 0,
    "largest": [],
    "byName": {}
  },
  "events": [
    {
      "kind": "producer",
      "name": "triage-repo.mjs",
      "status": "ok",
      "wallMs": 12,
      "memory": {
        "before": { "rssBytes": 1 },
        "after": { "rssBytes": 2 },
        "delta": { "rssBytes": 1 }
      }
    },
    {
      "kind": "skipped",
      "name": "emit-sarif.mjs",
      "reason": "not in --sarif mode"
    }
  ]
}
```

The ledger is not a public product artifact at first. It is an audit-core input
contract used by the JS wrapper. It may become a diagnostic artifact later only
if that helps review or migration.

## Output Contract

The Rust output must preserve the existing `producer-performance.json` product
shape:

- `schemaVersion: "producer-performance.v1"`;
- `generated`, `root`, `output`, `profile`;
- `scanRange`, `cache`, and `generatedArtifacts`;
- `summary` counts;
- `memory.measurement = "orchestrator-process-snapshots"`;
- `memory.childPeakRssAvailable = false`;
- `artifacts`, `artifactReads`, `producers`, and `skipped`.

Status behavior must stay compatible:

- `ok` increments `okCount`;
- statuses starting with `failed` increment `failedCount`;
- skipped ledger events become `skipped[]` entries with `status: "skipped"`;
- unknown producer statuses are preserved as producer statuses and counted only
  through the status-specific result summary layer.

The Rust module must not silently zero malformed required ledger sections. A
missing or invalid `events`, `artifacts`, or `artifactReads` field is a hard
contract failure for the CLI. This differs from summary projection modules,
which may report unavailable summaries from already-produced artifacts.

## JS Wrapper Migration

The JS executor should keep doing only these things in this slice:

- spawn producer processes;
- decide whether a step is skipped by an already-known Rust plan or runtime
  precondition;
- collect the raw observations it alone can observe: status, wall time, stderr
  snippet, and orchestrator memory snapshots;
- enumerate artifact sizes and artifact read metrics until those measurements
  also move to Rust.

Then it should pass a ledger payload to audit-core and receive
`producer-performance.json` from Rust.

After that wiring, JS should no longer hand-build:

- `summary.producerCount`;
- `summary.okCount`;
- `summary.failedCount`;
- `summary.skippedCount`;
- `summary.totalWallMs`;
- `summary.maxObservedOrchestratorRssBytes`;
- `producers[]`;
- `skipped[]`;
- the top-level `producer-performance.json` object.

The next wrapper-thinning slice moves the remaining base runtime ledger
projection into Rust as well. `audit-repo.mjs` should pass typed
`commandsRun[]`, `skipped[]`, the already-observed ordinary `artifactReads`
summary, and the produced artifact name list to audit-core. Rust then:

- converts `commandsRun[]` and `skipped[]` into typed ledger events;
- reads base producer phase timing sidecars from `.producer-phases/`;
- merges those phase sidecar reads into the supplied artifact-read metrics;
- measures produced artifact sizes from the JS-supplied artifact names; and
- builds the same `producer-performance.json` artifact.

This is still not lifecycle execution ownership. Lifecycle raw blocks and
ordinary JSON artifact reads remain JS-owned until a separate parity plan moves
them.

The wrapper-thinning CLI is:

```text
lumin-audit-core producer-performance-runtime-artifact --input <runtime.json|->
```

The runtime input schema is
`lumin-audit-producer-performance-runtime.v1`. The older
`lumin-audit-orchestration-ledger.v1` input remains a typed lower-level
compatibility contract, but `audit-repo.mjs` should call the runtime input
wrapper so event construction, phase sidecar reads, artifact-size measurement,
and `producer-performance.json` projection stay Rust-owned for base audit runs.

## Why Not Rust Executor Yet

Moving execution first would mix several contracts in one risky change:

- process spawning and stdio behavior;
- optional versus required failure propagation;
- `pre-write` / `post-write` / `canon-draft` / `check-canon` lifecycle exit
  policy;
- artifact read telemetry;
- final manifest writing;
- JS package entrypoint compatibility.

The event ledger isolates the product semantics first. Once Rust owns the
ledger and producer-performance construction, a later Rust executor can replace
the JS executor without changing manifest evidence shapes.

## Error Handling

The Rust CLI should fail with a usage/contract error when:

- the ledger is not valid JSON;
- `schemaVersion` is missing or not `lumin-audit-orchestration-ledger.v1`;
- `generated`, `root`, `output`, or `profile` is absent or empty;
- `profile` is not `quick`, `full`, or `ci`;
- `events` is missing or not an array;
- a producer event is missing `name` or `status`;
- a skipped event is missing `name` or `reason`;
- `artifactReads` or `artifacts` has an invalid required shape.

These are hard failures because the wrapper is asking Rust to own the product
contract. Silent fallback would recreate the JS/Rust drift this migration is
removing.

## Tests

Tests should prove product behavior:

- a ledger with producer and skipped events emits the existing
  `producer-performance.json` shape and summary counts;
- failed-required and failed-optional statuses are preserved in `producers[]`
  and counted as failures;
- memory snapshots preserve orchestrator-only wording and max RSS summary;
- artifact size and artifact-read summaries are copied without lossy
  recomputation;
- malformed ledger input hard-stops instead of producing a clean-looking
  artifact;
- the CLI emits JSON to stdout and rejects bad profiles.

No Node tests are required for the Rust module. JS wrapper parity can be checked
later when Node is allowed, by comparing the previous JS-built
`producer-performance.json` with the Rust-built output for the same captured
ledger.

## Canonical Updates

Before implementation, update `canonical/audit-core.md`:

- add the event ledger to `lumin-audit-core` scope;
- add `orchestration_events.rs` as owner of the typed ledger and
  `producer-performance.json` construction from completed JS executor
  observations;
- keep child process execution, live telemetry collection, artifact size
  enumeration, and final manifest writing outside Rust ownership for this
  slice.

The generated skill package canonical copy must be updated with the same owner
entry.

## Migration Sequence

1. Add this spec.
2. Add canonical owner entries.
3. Add `orchestration_events.rs` with typed ledger structs and builder.
4. Add `producer-performance-artifact --input <path|->` CLI.
5. Add Rust behavior tests and CLI hard-stop tests.
6. Add a JS wrapper function in both maintainer and skill-package copies of
   `_lib/audit-manifest.mjs`.
7. Replace JS `buildProducerPerformanceArtifact(...)` with the Rust wrapper,
   while keeping execution and event capture in JS.
8. Re-run Rust checks. Do not run Node unless explicitly approved.

## Acceptance

This design is satisfied when:

- `producer-performance.json` shape construction is Rust-owned;
- JS no longer hand-computes producer-performance summary fields;
- `commandsRun` and `skipped` remain JS-observed runtime evidence but are passed
  through a Rust-owned ledger contract;
- `manifest.performance` and `manifest.orchestration` continue to use the
  existing Rust summary projections;
- no JS/TS producer behavior changes;
- no elapsed-time cap, repository-size cap, timeout, or quota is introduced.

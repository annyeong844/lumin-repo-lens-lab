# Artifact Read Metrics Audit-Core Migration

## Decision

Move ordinary audit artifact-read metric aggregation from `_lib/artifacts.mjs`
into `lumin-audit-core`.

This is an orchestration telemetry slice only. It does not move JS/TS producer
artifact meaning, `manifest.json.blindZones`, or JSON artifact parsing into
Rust.

## Checked JS Behavior

The current JS helper `createArtifactReadMetrics({ rootDir, largestLimit })`
owns the `artifact-read-metrics.v1` summary used by
`producer-performance.json`.

Each successful or failed JSON read reports one raw observation:

- `filePath`
- `bytes`
- `readMs`
- `jsonParseMs`
- `ok`

The helper then:

1. normalizes `filePath` to a metric name relative to `rootDir`;
2. clamps negative or malformed numeric fields to zero after rounding;
3. increments total read count, total bytes, total read ms, total JSON parse
   ms, and parse-failure count;
4. aggregates the same counters in `byName`;
5. emits deterministic `largestReads` sorted by bytes descending then name;
6. emits deterministic `slowestJsonParses` sorted by parse ms descending then
   name;
7. caps those projection arrays to `largestLimit`, currently ten.

`readJsonFile()` still owns the actual file read, UTF-8 BOM stripping, JSON
parse, parse-failure logging, strict/non-strict parse behavior, and returning
`null` for missing or non-strict malformed files.

## Owner Boundary

The migrated Rust module owns:

- the typed raw read observation input shape;
- metric-name normalization from JS-supplied `rootDir` and `filePath`;
- `artifact-read-metrics.v1` summary projection;
- deterministic `byName`, `largestReads`, and `slowestJsonParses` ordering;
- the projection limit for largest/slowest read lists;
- merging phase-sidecar reads into the same summary shape.

It must not own:

- artifact file discovery;
- JSON reading or parsing for ordinary producer artifacts;
- parse-failure log text from `_lib/artifacts.mjs`;
- JS/TS producer semantics;
- blind-zone interpretation;
- final `manifest.json` writing.

## Data Flow

JS keeps a thin raw event collector:

```text
readJsonFile()
  -> observeRead(raw event)
  -> buildProducerPerformanceArtifact()
  -> audit-core artifact-read summary
  -> audit-core producer-performance artifact
```

The wrapper may keep the existing `observeRead(record)` call site shape, but
`summary()` must call audit-core instead of recomputing the summary in JS.

The new Rust command should accept a request like:

```json
{
  "schemaVersion": "lumin-audit-artifact-read-events.v1",
  "rootDir": "C:/repo/.audit",
  "largestLimit": 10,
  "reads": [
    {
      "filePath": "C:/repo/.audit/symbols.json",
      "bytes": 120,
      "readMs": 1,
      "jsonParseMs": 2,
      "ok": true
    }
  ]
}
```

and return the existing `artifact-read-metrics.v1` summary shape.

`ProducerPerformanceRuntimeInput.artifactReads` remains the summary shape, not
the raw event list. The JS wrapper builds that summary immediately before
calling `producer-performance-runtime-artifact`, preserving the current final
producer-performance API.

## Error Handling

Missing files remain invisible to the metric stream because current
`readJsonFile()` returns before `observeRead()` when a file does not exist.

Malformed ordinary JSON files remain visible as `ok: false` read events because
that is the checked JS behavior today.

Malformed artifact-read summary requests are internal wrapper errors and should
hard-stop. They indicate an audit-core wrapper contract bug, not a degraded
producer artifact.

Phase sidecar reads stay best-effort. A malformed phase timing file records an
artifact-read failure but does not produce a phase claim.

## Canonical Updates

After implementation:

- `canonical/audit-core.md` should list `artifact_read_metrics.rs` as the owner
  of ordinary artifact-read metric summary projection.
- `orchestration_events.rs` should stop owning local artifact-read summary math
  and reuse the shared Rust module for phase-sidecar read merges.
- the "ordinary artifact-read measurement" JS-owned exception should be removed
  or narrowed to "ordinary artifact JSON read/parse events".

## Verification

Product behavior tests should prove:

- a successful ordinary read and a failed ordinary parse produce the same
  summary counters and `byName` shape as the checked JS contract;
- path normalization preserves relative POSIX names under `rootDir` and falls
  back to basename outside `rootDir`;
- largest and slowest projections are deterministic and capped;
- phase sidecar reads still merge into an existing ordinary-read summary;
- malformed request schema hard-stops;
- the JS wrapper no longer computes the summary math itself.


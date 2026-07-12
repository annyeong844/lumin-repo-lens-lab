# Rust Child-Execution Orchestrator Design

## Goal

Move base audit child-process execution from `audit-repo.mjs` into
`lumin-audit-core` without moving JS/TS producer semantics.

This is the next step after the Rust orchestration plan and typed
producer-performance event projection. Rust should own the executor that runs
the planned base pipeline, observes runtime outcomes, and emits typed execution
events that feed the later
`producer-performance.json` ledger. JS/MJS producers remain producers; Rust
only owns the process runner and typed observation contract.

## Current State

Already Rust-owned:

- audit profile command graph in `orchestration_plan.rs`;
- typed `commandsRun` / `skipped` runtime-log shapes in `manifest_root.rs`;
- typed event ledger and `producer-performance.json` projection in
  `orchestration_events.rs`;
- artifact-size measurement for JS-supplied produced artifact names in
  `artifact_measurement.rs`;
- manifest projection and final summary patches for Rust-owned fields.

Still JS-owned:

- child process execution in `audit-repo.mjs`;
- runtime status observation, stderr snippet extraction, wall-clock timing, and
  orchestrator memory snapshots;
- pre/post-write, canon-draft, and check-canon child lifecycle blocks;
- ordinary artifact-read timing and lifecycle phase timing reads;
- human companion renderers;
- final `manifest.json` write.

The first executor slice should move only the base audit profile execution.
Lifecycle modes stay JS-owned until their raw block contracts are separately
ported.

## Non-Goals

- Do not port JS/TS producer internals.
- Do not reinterpret `blindZones`.
- Do not move `auditSummary`, `reviewPack`, or `topologyMermaid` rendering.
- Do not move final `manifest.json` writing in this slice.
- Do not add elapsed-time caps, repository-size caps, timeouts, or forced
  quotas.
- Do not invent new skip policy. Use the existing Rust orchestration plan and
  current JS precondition meanings.
- Do not infer child peak RSS. The existing memory evidence remains
  orchestrator process snapshots before and after child execution.

## Recommended Architecture

Add a Rust audit-core module:

```text
experiments/rust-main/lumin-audit-core/src/orchestration_executor.rs
```

The module should own:

- execution of `OrchestrationPlan.steps[]` for the base pipeline;
- existing precondition checks for planned base steps;
- command argv construction matching current `audit-repo.mjs`;
- wall-clock duration measurement;
- orchestrator memory snapshots before and after each child;
- stderr snippet capture capped to the existing 500-byte/character product
  surface;
- conversion of observed outcomes into typed `LedgerEvent` values;
- `RustAnalysisRun` observation for the optional `lumin-rust-analyzer` step.

The module must not own producer analysis behavior. A step with
`producerOwner: "js-mjs"` is still executed as a JS/MJS child through the same
script entrypoint and arguments. Rust is the runner, not the producer.

## CLI Surface

Add a CLI:

```text
lumin-audit-core execute-base-plan --input <request.json|->
```

The request should include the current JS runner values that are not analysis
semantics:

```json
{
  "schemaVersion": "lumin-audit-executor-request.v1",
  "plan": { "schemaVersion": "lumin-audit-orchestration-plan.v1" },
  "root": "/repo",
  "output": "/repo/.audit",
  "scriptsDir": "/repo-or-skill-engine",
  "nodeExecutable": "/path/to/node",
  "verbose": false,
  "scanRange": {
    "includeTests": true,
    "production": false,
    "excludes": [],
    "autoExcludes": []
  },
  "incremental": {
    "noIncremental": false,
    "cacheRoot": "/repo/.audit/.cache",
    "clearIncrementalCache": false
  },
  "generatedArtifacts": {
    "mode": "default"
  },
  "rustAnalyzer": {
    "requested": false,
    "invocation": null
  }
}
```

The output should be a typed execution result:

```json
{
  "schemaVersion": "lumin-audit-executor-result.v1",
  "events": [],
  "commandsRun": [],
  "skipped": [],
  "rustAnalysisRun": {
    "requested": false,
    "ran": false,
    "status": "not-requested"
  },
  "exitPolicy": {
    "basePipelineFailedRequired": false,
    "recommendedExitCode": 0
  }
}
```

The duplicated `commandsRun`, `skipped`, and `events` surfaces are intentional
for the first compatibility slice: `manifest_root.rs` already consumes the
runtime-log fields, while later `producer-performance.json` projection consumes
the same observations through the Rust runtime input. They must be built from
the same typed observed events so they cannot drift.

## Step Execution Contract

For JS/MJS base steps, Rust must preserve the current argv contract:

```text
node <scriptsDir>/<script>
  --root <root>
  --output <output>
  [--production]
  [--exclude <path>]...
  [--no-incremental / --cache-root <path>] for incremental producers
  [--generated-artifacts <mode>] for build-symbol-graph.mjs
```

The current incremental producer set is:

- `measure-topology.mjs`;
- `measure-staleness.mjs`;
- `build-block-clone-index.mjs`;
- `build-symbol-graph.mjs`;
- `build-shape-index.mjs`;
- `build-function-clone-index.mjs`.

The current generated-artifact mode forwarding applies only to
`build-symbol-graph.mjs`.

Required step behavior:

- status `ok` when the child exits zero;
- status `failed-required` when a required child exits non-zero;
- status `failed-optional` when an optional child exits non-zero;
- required child failure stops the base pipeline and returns
  `recommendedExitCode = 1`;
- optional child failure records the failure and continues;
- stderr snippet uses the existing 500-character product surface;
- verbose mode inherits stdout/stderr like the JS runner;
- non-verbose mode captures stdout/stderr like the JS runner.

## Preconditions And Skips

Rust must preserve existing preconditions:

| Step | Precondition | Skip reason |
|---|---|---|
| `build-resolver-diagnostics.mjs` | `symbols.json` exists | `symbols.json missing (symbol graph step failed or was skipped)` |
| `build-entry-surface.mjs` | `symbols.json` exists | `symbols.json missing (symbol graph step failed or was skipped)` |
| `build-module-reachability.mjs` | `symbols.json` and `entry-surface.json` exist | `symbols.json or entry-surface.json missing` |
| `export-action-safety.mjs` | `dead-classify.json` exists | `dead-classify.json missing (classify step failed or was skipped)` |
| `rank-fixes.mjs` | `dead-classify.json` exists | `dead-classify.json missing (classify step failed or was skipped)` |
| `merge-runtime-evidence.mjs` | coverage exists in `coverage/` or `.nyc_output/` | `no coverage-final.json in coverage/ or .nyc_output/` |
| `measure-staleness.mjs` | root is a git worktree | `not a git working tree` |

Planned skips such as `emit-sarif.mjs` when SARIF is not requested must be
copied from the Rust plan instead of recomputed with a second string policy.

## Rust Analyzer Step

`lumin-rust-analyzer` remains a Rust producer, but its execution can move with
the base executor because it is already represented as a planned step.

The executor must preserve current behavior:

- if `--rust-analyzer` is not requested, report `not-requested`;
- if triage reports zero Rust files, record a skipped runtime event and return
  `status: "skipped"`;
- if analyzer invocation cannot be resolved, record skipped/unavailable with
  the existing reason;
- on success, record command status `ok`, artifact
  `rust-analyzer-health.latest.json`, `rustFiles`, source commit, and
  analyzer invocation provenance;
- on failure, record `failed-optional`, stderr snippet, and return
  `status: "failed-optional"`.

The executor may receive analyzer invocation data from the JS wrapper for the
first slice, or Rust may resolve it if the resolution logic is explicitly
ported. Either way, the result artifact must show which source was used
(`env:LUMIN_RUST_ANALYZER_BIN` or `cargo:experiments`).

The Rust-file count must be read when the executor reaches the
`lumin-rust-analyzer` step. The JS wrapper must not pre-read `triage.json`
while building the executor request, because the base pipeline may not have
created the current `triage.json` yet.

## Measurement Contract

Rust executor owns only what it can observe while running children:

- wall-clock duration for each child;
- orchestrator process memory snapshots before and after each child;
- child exit status;
- stderr snippet;
- skip decisions based on the Rust plan and filesystem preconditions.

It does not yet own:

- ordinary artifact-read metrics from `_lib/artifacts.mjs`;
- lifecycle phase timing file reads from `_lib/producer-phase-timing.mjs`;
- final produced-artifact enumeration;
- final `manifest.json` write.

Those can move in later slices after the executor result is stable.

## JS Wrapper Migration

After the Rust CLI exists, `audit-repo.mjs` should become thinner:

1. parse CLI flags and build the Rust orchestration plan;
2. call `execute-base-plan` for the base pipeline;
3. use the returned `commandsRun`, `skipped`, `rustAnalysisRun`, and typed
   execution events;
4. continue to run lifecycle helpers in JS;
5. continue to build JS-owned `blindZones` and human renderers;
6. continue final manifest write until all remaining fields have Rust owners.

The JS wrapper must not retain a second base-pipeline `runStep` implementation
after this migration. Keeping both would recreate the drift this work is
removing.

## Error Handling

Hard-stop when:

- the request schema is invalid;
- the plan schema is not `lumin-audit-orchestration-plan.v1`;
- `scriptsDir`, `root`, `output`, or `nodeExecutable` is missing or empty;
- a planned step has an unsupported mode or owner for this executor slice;
- a required producer fails;
- `git rev-parse` needed for staleness precondition fails in an unexpected
  non-boolean way.

Do not hard-stop when:

- an optional producer fails;
- a precondition is unmet;
- the Rust analyzer is unavailable;
- an artifact expected by a later optional step is missing and the plan already
  defines a skip reason.

Every non-hard-stop omission must be visible in `skipped[]` or
`commandsRun[]`.

## Tests

Tests should prove product behavior, not scaffolding:

- a plan with one successful JS/MJS fixture child emits one `commandsRun`
  entry and one producer ledger event;
- an optional failing child records `failed-optional` and continues;
- a required failing child records `failed-required` and returns recommended
  exit code 1;
- missing `symbols.json` skips resolver diagnostics with the existing reason;
- planned SARIF skip is copied from the Rust plan;
- non-verbose mode captures stderr snippets and caps them at the existing
  product surface;
- malformed request shape hard-stops;
- no timeout, repository-size cap, or elapsed-time cap appears in the executor.

Node-backed integration can stay out of CI until explicitly allowed. The Rust
executor module should still be tested with temporary fixture commands that
exercise the same process semantics. If a fixture is not a real child process,
the test is too weak.

## Canonical Updates

Before implementation, update `canonical/audit-core.md`:

- add `orchestration_executor.rs` as the owner of base audit child execution
  and typed runtime observations;
- keep lifecycle child execution JS-owned;
- keep artifact-read timing, phase timing reads, human renderers,
  `blindZones`, and final manifest writing outside this slice;
- change `orchestration_plan.rs` step `executionOwner` only for the supported
  base executor slice after the executor exists.

## Migration Sequence

1. Add this spec.
2. Add canonical owner entry for the future executor slice.
3. Add typed executor request/result protocol.
4. Port precondition checks into Rust using the existing plan strings.
5. Add child process runner with argv arrays only.
6. Add memory/wall/stderr observation.
7. Emit typed execution events, `commandsRun`, `skipped`, and
   `rustAnalysisRun` from one typed event source.
8. Wire `audit-repo.mjs` to call Rust for the base pipeline.
9. Delete JS base-pipeline `runStep` once Rust is active.
10. Run Rust verification. Do not run Node unless explicitly approved.

## Acceptance

This slice is done when:

- Rust owns base audit child execution;
- JS no longer has a base-pipeline `runStep` implementation;
- lifecycle helpers remain JS-owned and unchanged;
- manifest summaries and `producer-performance.json` are produced from the
  same typed Rust execution observations plus the still JS-owned artifact-read
  and phase-timing measurements;
- no JS/TS producer behavior changes;
- no timeout, repository-size cap, elapsed-time cap, or quota is introduced;
- all omissions and skipped work are artifact-visible.

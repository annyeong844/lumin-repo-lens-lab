# Rust Post-Write Lifecycle Design

Date: 2026-07-01
Owner: `lumin-audit-core`

## Checked JS Contract

The current `audit-repo.mjs --post-write` path:

- refuses to run without `--pre-write-advisory` and records
  `manifest.postWrite = { requested: true, ran: false, reason:
  "--pre-write-advisory missing" }`;
- spawns the existing `post-write.mjs` producer with `--root`, `--output`,
  `--pre-write-advisory`, optional `--delta-out`, `--no-fresh-audit`,
  forwarded scan args, and forwarded incremental args;
- records child failure as `ran: false` with a
  `post-write.mjs exited non-zero:` reason while leaving advisory exit behavior
  to the orchestrator;
- on success records `deltaPath` pointing at
  `<deltaOutDir>/post-write-delta.latest.json`;
- reads the delta artifact best-effort and surfaces only manifest summary
  fields. If the artifact is unreadable, summary fields stay absent rather
  than defaulting to clean evidence.

## Rust Owner Boundary

`post_write_lifecycle.rs` owns this lifecycle wrapper and raw manifest block
projection. It may spawn the existing `post-write.mjs` entrypoint and read the
child-produced delta artifact.

It must not own:

- post-write delta computation;
- type-escape or file-delta semantics;
- markdown rendering;
- final `manifest.json` writing;
- pre-write advisory construction.

## JS Wrapper

`audit-repo.mjs` keeps the public CLI and final manifest assembly. Its
post-write branch becomes a request builder around
`executePostWriteLifecycle`.

`--strict-post-write` and `--strict-post-write-confidence` remain thin
orchestrator exit policies over the returned block. They do not change the
delta producer or manifest block owner.

## Exit Contract

The Rust result returns `{ block, exitCode, stdout?, stderr? }`.

- Missing `--pre-write-advisory` returns `exitCode=2` with
  `ran=false`, preserving the existing hard-stop branch.
- Child non-zero/spawn failure returns `ran=false`, a reason, and
  `exitCode=0`.
- Successful child execution returns `ran=true`, `deltaPath`, projected
  summary fields when readable, and `exitCode=0`.
- Child stdout/stderr are captured and returned so the JS wrapper can replay
  the existing `post-write.mjs` user-visible output without making Rust own
  final manifest writing.

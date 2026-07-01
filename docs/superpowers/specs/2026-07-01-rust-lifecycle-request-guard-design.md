# Rust Lifecycle Request Guard Design

Date: 2026-07-01

## Goal

Move the request-level audit lifecycle guard from `audit-repo.mjs` into
`lumin-audit-core` without moving JS/TS producer behavior.

This slice owns only the checked hard-stop projections that happen before
intent reading or child execution:

- `--pre-write && --post-write`
- `--pre-write` without `--intent`

## Checked JS Behavior

The migrated JS code produced these user-visible outcomes:

- Mutual exclusion writes
  `[audit-repo] --pre-write and --post-write are mutually exclusive\n`,
  sets both raw lifecycle blocks to `{ requested: true, ran: false, reason }`,
  and exits 2.
- Missing pre-write intent writes
  `[audit-repo] --pre-write requested but skipped: --intent <file|-> missing\n`,
  sets `preWrite.requested = true`, `preWrite.ran = false`,
  `preWrite.reason = "--intent missing"`, keeps the requested engine, and adds
  explicit owner fields only for `--pre-write-engine rust` or `js`.

## Rust Owner

`experiments/rust-main/lumin-audit-core/src/lifecycle_request.rs` owns:

- request schema validation
- raw skipped-block projection
- checked stderr text
- checked exit code 2 for blocked requests

The JS wrapper owns:

- CLI argument parsing
- reading intent files/stdin later in the route path
- invoking the Rust CLI wrapper
- final manifest assembly

## Non-Goals

This does not move:

- JS/TS `pre-write.mjs` producer semantics
- pre-write engine routing
- child process execution
- post-write delta semantics
- final `manifest.json` writing

Those already have separate owners or require separate parity plans.

## Compatibility Wrapper

`_lib/audit-manifest.mjs` exposes `evaluateLifecycleRequestGuard(...)`, a thin
JSON wrapper over:

```text
lumin-audit-core lifecycle-request-guard --input -
```

`audit-repo.mjs` calls this wrapper before attempting pre-write routing or
post-write execution. If the Rust result is `blocked`, JS copies the returned
raw lifecycle blocks into the manifest and sets the returned exit code.

## Verification

Product behavior is covered by
`experiments/rust-main/lumin-audit-core/tests/lifecycle_request.rs`:

- mutual exclusion projects both blocks and checked stderr
- missing pre-write intent preserves explicit Rust owner fields
- clear requests do not invent lifecycle blocks
- CLI emits typed JSON and rejects unsupported schema versions

No Node-based test is required for this slice; the JS side is a compatibility
wrapper around the typed Rust contract.

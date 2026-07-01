# Rust Pre-Write Lifecycle Migration

## Decision

Move the Rust pre-write lifecycle wrapper from `audit-repo.mjs` into
`lumin-audit-core/src/pre_write_lifecycle.rs`.

This is a Rust-owned orchestration slice only. It does not port
`pre-write.mjs`, JS/TS pre-write producer semantics, or
`lumin-rust-analyzer` internals.

## Checked JS Behavior

The previous Rust pre-write path in `audit-repo.mjs` did four things after
engine routing selected `rust`:

1. run `lumin-rust-analyzer pre-write` with `--root`, `--source-commit`,
   `--intent -`, `--output`, `--production` when tests are excluded, and every
   effective `--exclude`;
2. copy the native Rust artifact to `rust-pre-write-artifact.latest.json`;
3. build and write `pre-write-advisory.<invocationId>.json` plus
   `pre-write-advisory.latest.json`;
4. project `manifest.preWrite` with advisory paths, native artifact paths,
   source commit, engine selection, and analyzer invocation provenance.

The migrated Rust command preserves those semantics for the Rust engine through
`execute-rust-pre-write --input <path|->`.

## Product-Mode IO

The product wrapper calls audit-core with `--result-output <file>`. In that
mode audit-core inherits the analyzer child's stdout/stderr, so users still see
the analyzer stream directly, and writes the typed lifecycle result out of band
for the JS wrapper to parse.

Plain JSON mode remains available for tests and diagnostics.

## Owner Boundary

`pre_write_lifecycle.rs` owns:

- the Rust pre-write request/result schema;
- Rust analyzer child argv/stdin projection;
- child failure to `manifest.preWrite` block projection;
- native artifact latest copy;
- Rust advisory construction and writes;
- JS-supplied file inventory and failure pass-through into the Rust advisory;
- product-mode result-file behavior.

It must not own:

- JS/TS `pre-write.mjs` producer semantics;
- scan-scope walking or source inventory interpretation;
- Rust analyzer syntax/oracle behavior;
- post-write delta semantics;
- final `manifest.json` writing.

## Why JS/TS Pre-Write Stays JS-Owned

The JS/TS pre-write engine consumes JS/TS producer artifacts and contracts that
are not owned by audit-core yet. Moving that path now would force Rust to
reinterpret JS/TS producer meaning. That is exactly the stringly-typed
migration failure this track is avoiding.

The JS wrapper still routes engine selection and still owns the JS/TS engine
path. The Rust engine path is now a thin audit-core wrapper call.

## Verification

Behavior is covered by `lumin-audit-core` product tests:

- analyzer success writes native latest, advisory latest/specific, and a
  successful `manifest.preWrite` block;
- JS-supplied file inventory and failures are preserved in the advisory;
- analyzer child failure returns a non-ran block without writing advisory files;
- CLI `--result-output` streams child stdout/stderr and writes clean JSON;
- malformed request schema hard-stops.

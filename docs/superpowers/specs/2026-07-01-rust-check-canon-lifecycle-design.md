# Rust Check-Canon Lifecycle Design

Date: 2026-07-01
Owner: `lumin-audit-core`

## Checked JS Contract

The existing JS owner is `_lib/audit-check-canon.mjs`. It does not own
canon-drift semantics. It only:

- expands `--sources` over `type-ownership`, `helper-registry`, `topology`,
  and `naming`;
- rejects unknown sources before spawning a child;
- invokes `check-canon.mjs` with `--root`, `--output`, `--source`, and the
  forwarded scan args;
- uses one `--source all` child when all sources are requested and the primary
  `symbols.json` plus `topology.json` artifacts already exist;
- falls back to per-source children when those artifacts are absent or when a
  subset is requested;
- reads the child-produced `canon-drift.json` and projects
  `manifest.checkCanon.perSource`;
- treats drift and per-source parser/missing-canon failures as legitimate
  source outcomes, not spawn failures;
- applies advisory exit behavior by default and strict drift/all-failed exit
  escalation when `--strict-check-canon` is set.

## Rust Owner Boundary

`check_canon_lifecycle.rs` owns the typed lifecycle wrapper above. It may spawn
the existing `check-canon.mjs` entrypoint and read the resulting
`canon-drift.json`.

It must not own:

- `check-canon.mjs` drift detection;
- `_lib/check-canon-*` parser contracts;
- markdown rendering for canon drift reports;
- final `manifest.json` writing.

## JS Wrapper

`audit-repo.mjs` keeps the public CLI and manifest assembly. Its check-canon
path becomes a thin request builder around `executeCheckCanonLifecycle`.

`_lib/audit-manifest.mjs` is the compatibility bridge that invokes
`lumin-audit-core execute-check-canon --input -`.

## Exit Contract

The Rust result returns `{ block, exitCode }`.

- Unknown source values return `block.ran=false`, a reason, and `exitCode=1`.
- Advisory mode returns `exitCode=0`.
- Strict mode returns `2` if no source was checked, `1` if any drift exists,
  otherwise `0`.

The public JS orchestrator preserves the existing rule: check-canon only
changes the final process exit code when the previous exit code is still `0`.

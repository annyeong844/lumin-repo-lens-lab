# Vitest Maintainer Scripts Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-13.
> **Pilot candidate:** `tests/test-maintainer-scripts.mjs`.

---

## Purpose

This review decides whether the maintainer-scripts guard suite is a reasonable
next Vitest pilot candidate. It does not add the Vitest suite. The goal is to
confirm that a future runner migration can improve execution mechanics without
weakening the script-safety checks or pretending that source-text guards are
behavioral integration tests.

## Reviewed Evidence

- Preserved Node command: `node tests/test-maintainer-scripts.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:maintainer-scripts`.
- Scripts guarded by the current suite:
  - `scripts/run-syntax-check.mjs`,
  - `scripts/run-tests.mjs`,
  - `scripts/publish-public-plugin.mjs`.
- Existing reviewed all-pilot command: `npm run test:vitest`.
- Documentation guards: `npm run check:test-doc` and
  `npm run check:doc-script-refs`.

## Result

The suite is acceptable as the next low-risk Vitest pilot candidate, but the
pilot should preserve its current source-text guard shape.

The current suite protects maintainer-script failure handling. Two checks make
sure child-process spawn failures are not silently treated as successful test
or syntax-check runs. One check makes sure the public-package publisher uses a
try/catch optional JSON reader instead of an `existsSync` then `readFileSync`
time-of-check/time-of-use pattern for optional package lock metadata.

Those are implementation-safety guards. Rewriting them into temporary script
fixtures would be larger than the current suite and would risk testing a fake
script harness instead of the maintainer scripts that actually ship. A Vitest
pilot should therefore keep the assertions source-facing, but express them as
named `it(...)` blocks with clearer failure localization.

## Protected Invariants

The future Vitest pilot must preserve these maintainer-script contracts:

- `scripts/run-syntax-check.mjs` explicitly checks `spawnSync(...).error` and
  reports a failed `node --check` process start with `result.error.message`;
- `scripts/run-tests.mjs` explicitly checks `spawnSync(...).error` and reports
  a failed test-suite process start with `result.error.message`;
- `scripts/publish-public-plugin.mjs` uses a `readOptionalJson(...)`
  try/catch helper for optional JSON files;
- `readOptionalJson(...)` returns `null` only for `ENOENT` and rethrows other
  JSON or filesystem errors;
- the public-package publisher does not reintroduce an `existsSync(...)`
  followed by optional package-lock `readJson(...)` pattern.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-maintainer-scripts.mjs` remains runnable.
- The pilot may use source-text assertions because the existing suite is a
  guard against specific script implementation regressions.
- The pilot should not introduce temporary script/package fixtures in this
  slice. A behavior-level fixture would need a separate design for safely
  injecting spawn failures and optional package files without testing a fake
  script wrapper.
- `npm run test:vitest` must stay scoped to reviewed `tests/*.test.mjs` files.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/maintainer-scripts.test.mjs`,
2. `npm run test:vitest:maintainer-scripts`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the three current Node checks represented as
named Vitest `it(...)` blocks. It should also run both:

- `node tests/test-maintainer-scripts.mjs`
- `npm run test:vitest:maintainer-scripts`

Do not migrate resolver, deadness, pre-write, ranking, performance, or
public-package install-verification suites as part of the maintainer-scripts
pilot.

# Vitest Temp Repo Fixture Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-12.
> **Pilot:** `tests/temp-repo-fixture-helper.test.mjs`.

---

## Purpose

This review closes the WM-08 gate before any additional test-runner migration.
The pilot tested whether Vitest improves the execution mechanics of one small
suite without changing analyzer evidence, public package behavior, or existing
Node test entrypoints.

## Reviewed Evidence

- Pilot command: `npm run test:vitest`.
- Node parity command: `node tests/test-temp-repo-fixture-helper.mjs`.
- Existing full Node lane: `npm test`.
- Documentation guards: `npm run check:test-doc` and
  `npm run check:doc-script-refs`.
- CI-adjacent guards: `npm run check`, `npm run check:drift`,
  `npm run check:skill-triggering`, `npm run check:behavior`,
  `npm run lint`, and `npm run check:public-plugin`.

## Result

The pilot is acceptable as a separate runner lane.

Vitest improved this suite's execution mechanics:

- individual cases are named as runner-level tests instead of custom
  `check(...)` calls;
- assertion failures would point at a single `it(...)` block;
- `npm run test:vitest` gives a focused command for one pilot suite;
- the existing Node suite remains runnable and still covers the same six edge
  cases;
- `npm test` remains the authoritative Node-script suite and does not discover
  the Vitest file.

The pilot does not prove that the full suite should migrate. It proves that
small setup-only helper contracts can be mirrored in Vitest while preserving the
old Node path during review.

## Protected Invariants

The Vitest pilot keeps the same temp repo helper contract:

- default root, output, and package creation;
- nested text writes with parent directory creation;
- JSON write/read with a trailing newline;
- output-root artifact writes and reads;
- unsafe path rejection before path resolution;
- unsupported root/output selector rejection;
- cleanup constrained to the helper-created fixture root.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Additional migrations must be one-suite-at-a-time and must name the protected
  invariant before changing the test shape.

## Recommendation

Proceed only to another low-risk, setup-heavy suite if the next PR names:

1. the protected invariant,
2. the exact edge case that must still fail if the migration loses meaning,
3. the Node command that remains runnable,
4. the focused Vitest command that proves the migrated shape.

Do not migrate resolver, deadness, pre-write, performance, ranking, or public
package suites until their suite-specific risks are reviewed in the wiki.

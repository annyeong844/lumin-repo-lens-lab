# Vitest Python Conventions Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-python-conventions.mjs`

---

## Purpose

This review decides whether `tests/test-python-conventions.mjs` may move to a
focused Lane D Vitest mirror. It does not add the Vitest suite.

The suite protects Python convention handling in `build-symbol-graph.mjs`: it
guards package self-reference import resolution, `__all__` public-surface
filtering, framework decorator registration, and dunder runtime-dispatch
exclusion. It is acceptable as a single-suite batch because the fixture surface
is Python-specific and does not need to absorb JavaScript/TypeScript resolver
families, dead-export action proof, generated artifacts, symlink aliasing, or
performance/incremental cache behavior.

## Reviewed Evidence

| Suite                               | Preserved Node Command                   | Proposed Focused Vitest Command          | Surface Under Review         |
| ----------------------------------- | ---------------------------------------- | ---------------------------------------- | ---------------------------- |
| `tests/test-python-conventions.mjs` | `node tests/test-python-conventions.mjs` | `npm run test:vitest:python-conventions` | Python convention extraction |

Current Node evidence checked for this review:

```text
node tests/test-python-conventions.mjs # 13 passed, 0 failed
```

Goal lane: Lane D, resolver/surface. This review covers only Python convention
support in the symbol graph pipeline.

## Result

This suite is acceptable as one focused Vitest mirror.

The future implementation PR may add one mirror file and one focused script,
provided it keeps the Node entrypoint runnable and keeps Python fixture meaning
local to the suite. The mirror must preserve the current skip behavior when
`python3` is unavailable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- package self-reference imports such as `from fxpyselfref.agents.loader import
load_agent` resolve to the root package file, not to a duplicated
  `<root>/<root>/...` probe;
- a self-reference imported symbol stays out of `deadProdList`;
- an unconsumed consumer function in the same fixture can still be reported as
  dead, proving the self-reference handling is not blanket liveness;
- modules declaring `__all__` expose only listed names as public candidates;
- `__all__` itself stays out of `deadProdList`;
- names listed in `__all__` can still be dead candidates when unconsumed;
- top-level names not listed in `__all__`, including `_helper` and
  `internal_util`, remain module-private and do not enter `deadProdList`;
- framework decorators such as `@app.command()`, `@app.command(name=...)`, and
  `@app.callback()` prevent registered functions from being reported dead;
- undecorated functions in the same framework fixture remain eligible dead
  candidates;
- dunder runtime hooks such as `__getattr__` and `__dir__` do not enter the
  dead list;
- ordinary functions beside dunder hooks remain eligible dead candidates;
- when `python3` is unavailable, the suite reports a clean skip rather than a
  failure.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- duplicating the root package name during self-reference import resolution
  must fail;
- treating every definition in a self-reference fixture as live must fail;
- ignoring `__all__` and surfacing unlisted helpers as dead must fail;
- treating `__all__` as a dead symbol must fail;
- muting names listed in `__all__` merely because the module has an `__all__`
  declaration must fail;
- treating framework-decorated functions as ordinary unused functions must
  fail;
- muting undecorated functions only because nearby decorated functions exist
  must fail;
- reporting dunder runtime hooks as dead candidates must fail;
- hard-failing on machines without `python3` must fail.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The Node suite remains runnable and authoritative until a later cleanup spec
  retires it.
- The fixture boundary is temporary Python package trees, `build-symbol-graph`
  invocation, and `symbols.json.deadProdList` reads.
- Shared setup may create Python package files, run the symbol graph producer,
  read `symbols.json`, and preserve the `python3` availability gate.
- Shared helpers must not decide Python public-surface semantics, decorator
  liveness policy, dunder dispatch policy, JS/TS resolver behavior, action
  safety, or deadness ranking.
- The mirror must not change Python extraction behavior, symbol graph output
  contracts, rank/fix classification, public package behavior, or resolver
  behavior.
- The mirror must not absorb symlink aliasing, generated/framework resource
  packs, other resolver unsupported-family suites, deadness/ranking suites, or
  performance/incremental suites.

## Implementation Notes

- Prefer one Vitest file: `tests/python-conventions.test.mjs`.
- Add one focused script: `test:vitest:python-conventions`.
- Keep the four fixture sections separate:
  1. self-reference import resolution;
  2. `__all__` public-surface filtering;
  3. framework-registered decorators;
  4. dunder runtime dispatch.
- Use temporary directories that are portable on Windows and POSIX.
- Preserve the `python3` skip path as an explicit `describe.skip` or early test
  guard with a clear reason.

## Validation Commands

The implementation PR must run:

```text
node tests/test-python-conventions.mjs
npm run test:vitest:python-conventions
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-python-conventions.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/test-migration-candidate-board.md
git diff --check
```

Before merge, the implementation should also keep the broader runner lane
green:

```text
npm run check
npm run lint
npm run test:vitest
npm test
```

## Non-Goals

- Do not change Python extraction semantics.
- Do not add JS/TS resolver behavior.
- Do not migrate symlink aliasing in this batch.
- Do not change dead-export ranking or action-safety proof.
- Do not add generated/resource capability packs.
- Do not treat Python convention evidence as automatic deletion proof.

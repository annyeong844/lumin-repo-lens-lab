# Vitest P6 Member Precision Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-23.
> **Pilot candidate:** `tests/test-p6-member-precision.mjs`.

---

## Purpose

This review decides whether `tests/test-p6-member-precision.mjs` may move to a
focused Vitest mirror. The suite protects P6 member-level deadness precision:
namespace imports and dynamic import bindings should protect only the member
that is actually read, while unrelated siblings must remain concrete dead
candidates.

The key risk is blunt. A broad mirror could accidentally accept blanket
namespace liveness again. That would hide false negatives where a namespace or
dynamic import touches one export but makes every sibling look alive.

## Reviewed Evidence

| Suite                                | Preserved Node Command                    | Proposed Focused Vitest Command           | Surface Under Review                                      |
| ------------------------------------ | ----------------------------------------- | ----------------------------------------- | --------------------------------------------------------- |
| `tests/test-p6-member-precision.mjs` | `node tests/test-p6-member-precision.mjs` | `npm run test:vitest:p6-member-precision` | symbol graph member fan-in and namespace shadow precision |

Goal lane: deadness/ranking calibration. This is a suite-specific review for
member precision evidence, not permission to migrate P6 measurement,
P6 safe-fix calibration, rank-fixes, corpus precision, cue-tier policy, or
audit-repo umbrella behavior.

Fresh preserved-command evidence on 2026-05-23:

```text
node tests/test-p6-member-precision.mjs
12 passed, 0 failed
```

## Result

This suite now has a narrow Vitest mirror at
`tests/p6-member-precision.test.mjs`, and the preserved Node command remains
runnable. The mirror keeps the production `build-symbol-graph.mjs`
child-process path and reads the real `symbols.json` output for each fixture.

The mirror must not extract helper logic that decides namespace fan-in,
dynamic import binding ownership, lexical shadowing, dead candidate
classification, or conservative whole-file shadow behavior.

## Protected Invariants

The future Vitest mirror must preserve these 12 contracts:

### Direct Namespace Import

- P6M-1a: a namespace direct member access protects only the called export.
- P6M-1b: an unrelated namespace sibling remains a concrete dead candidate
  with `namespaceShadowed === false`.

### Dynamic Import Variable

- P6M-2a: an `await import()` binding direct member access protects the called
  export.
- P6M-2b: an `await import()` binding direct member access does not
  blanket-protect siblings.

### Dynamic Import `.then()`

- P6M-3a: an `import().then()` callback member access protects the called
  export.
- P6M-3b: an `import().then()` direct member access does not blanket-protect
  siblings.

### Conservative Namespace Degradation

- P6M-4: a degraded namespace alias keeps the conservative whole-file shadow
  instead of promoting sibling deletes.

### Shadowed Dynamic Bindings

- P6M-5a: a shadowed dynamic binding keeps outer member attribution lexical.
- P6M-5b: a shadowed inner dynamic binding attributes to its own module.
- P6M-5c: a shadowed dynamic binding does not hide unrelated dead exports.

### Shadowed Namespace Parameters

- P6M-6a: a namespace-shaped function parameter shadow does not steal module
  attribution.
- P6M-6b: a namespace shadowed-only export remains a concrete dead candidate.

## Edge-Case Failures To Preserve

The mirror must fail if:

- namespace member access revives every export in the imported file;
- dynamic import member access revives unrelated siblings;
- `import().then()` callback bindings lose their target module identity;
- lexical shadowing credits a member access to the wrong module;
- local parameters named like namespace imports steal static module
  attribution;
- conservative degraded aliases start producing automated dead candidates;
- `namespaceShadowed` is used as a vague excuse to hide concrete dead siblings;
- the test validates only process exit without reading `fanInByIdentity` and
  `deadProdList`.

## Fixture Boundary

Allowed shared helper behavior:

- create and remove temporary fixture repos;
- write small package and source files;
- run `build-symbol-graph.mjs` as a child process;
- read `symbols.json`;
- find entries in `deadProdList` by symbol name;
- assert `fanInByIdentity`, `deadProdList`, `deadTotal`, `trulyDead`, and
  `namespaceShadowed` values.

Forbidden helper behavior:

- deciding member fan-in;
- deciding whether a namespace access is precise or degraded;
- deciding dynamic import binding ownership;
- deciding lexical shadowing;
- deciding dead candidate classification;
- replacing the production symbol-graph producer with a helper-only model;
- sharing precision logic with P6 measurement, P6 safe-fix calibration,
  corpus, rank-fixes, export-action-safety, cue-tier policy, or audit-repo
  umbrella suites.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The preserved Node command remains runnable and authoritative until a later
  cleanup spec retires it.
- The mirror must not change `build-symbol-graph.mjs`, symbol extraction,
  dynamic import analysis, dead-export classification, ranking, or P6
  measurement artifacts.
- The mirror must not absorb `tests/test-p6-measurement.mjs`,
  `tests/test-p6-safe-fix-calibration.mjs`, `tests/test-rank-fixes.mjs`,
  `tests/test-corpus.mjs`, cue-tier policy, or `tests/test-audit-repo.mjs`.
- The mirror must not turn member precision evidence into `SAFE_FIX` action
  proof.

## Recommendation

The narrow implementation PR added:

1. `tests/p6-member-precision.test.mjs`;
2. `npm run test:vitest:p6-member-precision`;
3. candidate-board and goal updates moving this suite from `REVIEWED` to
   `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the 12 current Node
assertions as named Vitest cases. It remains covered by the preserved Node
command, the focused Vitest command, and the doc guards.

## Validation Commands

The implementation PR must run:

```text
node tests/test-p6-member-precision.mjs
npm run test:vitest:p6-member-precision
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-p6-member-precision.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/vitest-mirror-closure-audit.md
git diff --check
```

Keep `docs/lumin-wiki/test-migration-candidate-board.md` as a targeted wide
table edit; do not run Prettier write on that file unless a separate table
normalization PR owns the churn.

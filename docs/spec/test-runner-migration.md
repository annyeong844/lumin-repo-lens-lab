# Test Runner Migration

> **Role:** maintainer-facing spec for introducing a professional JavaScript
> test runner without losing the current Node-based verification surface.
> **Status:** SPEC.
> **Last updated:** 2026-05-12

---

## 1. Problem

Most current suites are executable `node tests/test-*.mjs` scripts. That keeps
the repo dependency-light and easy to run, but many suites carry their own
mini-runner shape:

- local `passed` / `failed` counters,
- local `assert` / `check` helpers,
- manual cleanup in `finally`,
- ad hoc failure output,
- limited focused-test selection,
- no shared watch, timeout, or reporter behavior.

The problem is not that Node is wrong. The problem is that every test file has
to be its own tiny test runner. That makes test reform harder and encourages
helper extraction for execution mechanics instead of preserving the actual
edge-case invariant.

## 2. Goals

- Introduce a professional runner pilot without changing analyzer behavior.
- Keep the existing Node test entrypoints during the pilot.
- Improve assertion readability, cleanup hooks, failure output, and focused
  execution for one small suite first.
- Use the wiki milestone board to prevent broad test churn.
- Keep public package/runtime verification on Node unless a future spec proves
  otherwise.
- Make rollback simple: remove the pilot runner config and pilot suite without
  changing engine code.

## 3. Non-Goals

- Do not convert the full test suite in one PR.
- Do not remove `node tests/test-*.mjs` entrypoints during the pilot.
- Do not make Bun a required runtime for tests, CI, public package validation,
  or local maintainer verification.
- Do not introduce coverage gates in the pilot.
- Do not change `scripts/run-tests.mjs` behavior until a pilot has real data.
- Do not move analyzer semantics into runner helpers.

## 4. Decision

Use **Vitest** for the first professional-runner pilot.

Why Vitest first:

- It keeps Node as the runtime assumption.
- It is a test runner change, not a runtime/package-manager migration.
- It gives standard `describe` / `it` / `expect`, lifecycle hooks, focused
  tests, watch mode, reporters, and timeout control.
- It can coexist with current Node-script tests.
- It can be added as a dev-only dependency.

Keep **Bun** parked as a future evaluation.

Why Bun is not first:

- It changes more than the runner; it changes runtime assumptions.
- Lumin's plugin/package verification currently depends on Node.
- Requiring Bun for validation would raise contributor and CI requirements.
- Bun may still be useful later as an optional speed experiment, but that needs
  a separate compatibility and packaging spec.

## 5. Pilot Scope

The recommended first pilot is the temp repo fixture helper contract because it
is small, test-only, and already has a clear edge-case boundary:

```text
tests/test-temp-repo-fixture-helper.mjs
```

The pilot should either:

1. add a parallel Vitest version of the suite while keeping the Node suite, or
2. migrate the suite only after preserving an equivalent Node-accessible command
   in package scripts.

The first option is safer for the pilot because it lets reviewers compare the
same contract in both shapes before removing anything.

The pilot must continue to cover:

- default root/output/package creation,
- nested text writes,
- JSON write/read with trailing newline,
- output-root artifact reads,
- unsafe path rejection,
- unsupported root/output selector rejection,
- guarded cleanup.

## 6. Coexistence Model

During the pilot:

- `npm test` continues to run the existing Node test suite.
- A new script may run only the pilot Vitest suite, for example:

```json
{
  "scripts": {
    "test:vitest": "vitest run"
  }
}
```

- CI should not require Vitest until the pilot is reviewed.
- `tests/README.md` should list the pilot command only after the project
  decides whether generated docs should track professional-runner suites.
- The pilot must not change public package build or install verification.
- The all-pilot Vitest command must be scoped to first-party pilot files. It
  must not recursively collect behavior corpora, generated fixture repositories,
  or audit output directories as test suites.

If the pilot graduates, a later spec should define whether `scripts/run-tests.mjs`
delegates to Vitest for migrated suites or whether Node-script and Vitest suites
remain separate lanes for a while.

## 7. Test Shape Rules

Vitest should improve readability without weakening contracts.

Prefer:

```js
import { describe, expect, it } from 'vitest';

describe('temp repo fixture', () => {
  it('rejects parent traversal before resolving paths', () => {
    expect(() => fx.write('../outside.ts', 'x\n')).toThrow(/fixture path/i);
  });
});
```

Avoid:

- one broad happy-path test that hides individual safety contracts,
- snapshots for small structural contracts,
- runner-only assertions that prove Vitest runs but not that the original edge
  case is protected,
- replacing explicit cleanup checks with lifecycle hooks unless cleanup failure
  is still asserted somewhere.

## 8. Acceptance Criteria For WM-08

Before marking the Vitest pilot complete:

- Vitest is dev-only.
- The pilot suite has a focused command.
- The existing Node suite remains runnable.
- The pilot covers the same edge cases as the original suite.
- No analyzer behavior changes.
- No public package runtime dependency changes.
- `npm test` behavior is unchanged unless explicitly reviewed.
- The PR records whether Vitest improves failure output, focused execution, and
  cleanup readability enough to justify more migrations.

## 9. Rollback

The pilot must be reversible by removing:

- the Vitest dev dependency,
- the Vitest config if one is added,
- the pilot test file,
- the pilot npm script.

Rollback must not require changing engine code, public package code, or existing
Node-script tests.

## 10. Future Bun Evaluation

Bun can be evaluated later only as an optional experiment. A Bun spec should
answer:

- Which Node APIs used by the tests are fully compatible?
- Does Windows behavior match current Node behavior?
- Does Bun change package-lock or install expectations?
- Can public package verification remain Node-based?
- Is the speedup large enough to justify the extra runtime requirement?

Until those questions are answered, Bun must not become the default local or CI
test runtime for Lumin.

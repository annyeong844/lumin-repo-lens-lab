# Vitest Refactor Plan Verifier Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-13.
> **Pilot candidate:** `tests/test-refactor-plan-verifier.mjs`.

---

## Purpose

This review decides whether the refactor-plan verifier is a reasonable next
Vitest pilot candidate. It does not add the Vitest suite. The goal is to
confirm that a future runner migration can improve execution mechanics without
changing the humane plan contract, the CLI verifier, or the existing Node test
entrypoint.

## Reviewed Evidence

- Preserved Node command: `node tests/test-refactor-plan-verifier.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:refactor-plan-verifier`.
- Current library under test: `test-harness/lib/verify-refactor-plan.mjs`.
- Existing reviewed all-pilot command: `npm run test:vitest`.
- Documentation guards: `npm run check:test-doc` and
  `npm run check:doc-script-refs`.

## Result

The suite is acceptable as the next low-risk Vitest pilot candidate.

The suite is a maintainer verifier, not analyzer ranking logic. It checks saved
refactor-plan output for required sections, evidence anchors, pre-write
handoff, coding-agent handoff, tone guard, and CLI behavior. That makes it a
good fit for the same migration pattern used by the behavior corpus and
citation verifier pilots: keep the Node entrypoint, mirror each contract as a
runner-level `it(...)` block, and keep verifier semantics in the test harness
library.

The migration should remain narrow. It should not change plan parsing,
section-name policy, tone policy, pre-write handoff requirements, or
chat-facing wording rules.

## Protected Invariants

The future Vitest pilot must preserve these refactor-plan verifier contracts:

- a valid SHORT plan with code changes, pre-write handoff, coding-agent prompt,
  evidence anchor, and verification section passes;
- a code-changing SHORT plan without a pre-write handoff fails with
  `missing-prewrite-handoff`;
- a code-changing SHORT plan without a coding-agent prompt fails with
  `missing-coding-agent-prompt`;
- raw JSON blocks in default chat-facing plans fail with `raw-json-in-chat`;
- discouraging wording fails with `discouraging-tone`;
- plans without an artifact or claim-label evidence anchor fail with
  `missing-evidence-anchor`;
- a FULL handoff plan with the required sections passes;
- CLI verification exits zero for a valid plan;
- CLI verification exits non-zero and names `verification-section` when the
  verification section is missing.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-refactor-plan-verifier.mjs` remains runnable.
- `test-harness/lib/verify-refactor-plan.mjs` remains the verifier
  implementation; the Vitest suite should call it rather than reimplementing
  section or tone checks.
- Temporary fixture data is limited to Markdown files used by CLI cases.
- The shared temp repo helper should not be introduced here. This suite does
  not need a repo-shaped fixture, and adding one would make the fixture boundary
  broader than the verifier behavior under test.
- `npm run test:vitest` must stay scoped to reviewed `tests/*.test.mjs` files.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/refactor-plan-verifier.test.mjs`,
2. `npm run test:vitest:refactor-plan-verifier`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep every current Node case represented as a
named Vitest `it(...)` block. It should also run both:

- `node tests/test-refactor-plan-verifier.mjs`
- `npm run test:vitest:refactor-plan-verifier`

Do not migrate resolver, deadness, pre-write, ranking, performance, or
public-package suites as part of the refactor-plan verifier pilot.

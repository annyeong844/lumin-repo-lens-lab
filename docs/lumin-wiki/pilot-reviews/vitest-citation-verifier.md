# Vitest Citation Verifier Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-12.
> **Pilot candidate:** `tests/test-citation-verifier.mjs`.

---

## Purpose

This review decides whether the citation verifier is a reasonable next Vitest
pilot candidate. It does not add the Vitest suite. The goal is to confirm that a
future runner migration would improve execution mechanics without changing the
grounded-citation contract or weakening the existing Node verifier path.

## Reviewed Evidence

- Preserved Node command: `node tests/test-citation-verifier.mjs`.
- Proposed focused Vitest command: `npm run test:vitest:citation-verifier`.
- Current library under test: `test-harness/lib/verify-citations.mjs`.
- Existing reviewed all-pilot command: `npm run test:vitest`.
- Documentation guards: `npm run check:test-doc` and
  `npm run check:doc-script-refs`.

## Result

The suite is acceptable as the next low-risk Vitest pilot candidate.

The suite is a maintainer verifier, not analyzer ranking logic. It checks
whether saved model output uses mechanically falsifiable `[grounded, ...]`
citations. That makes it a good fit for the same migration pattern used by the
behavior corpus verifier: keep the Node entrypoint, mirror the cases as
runner-level `it(...)` blocks, and keep verifier semantics in the test harness
library.

The migration should remain narrow. It should not change citation parsing,
artifact lookup, Markdown wording policy, or user-facing answer rules.

## Protected Invariants

The future Vitest pilot must preserve these citation-verifier contracts:

- scalar, bracket-path, `.length`, object-literal, and root `package.json`
  citations pass when they match the artifact value;
- value mismatches fail with `value-mismatch`;
- grounded citations without a `path = value` assignment fail as
  unfalsifiable;
- missing artifact paths fail;
- placeholder expected values such as `N` fail;
- trailing unverified clauses are warnings, not proof;
- CLI verification exits zero for valid citation files;
- CLI verification exits non-zero for mismatched citation files;
- CLI verification can read Markdown from stdin.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-citation-verifier.mjs` remains runnable.
- `test-harness/lib/verify-citations.mjs` remains the verifier implementation;
  the Vitest suite should call it rather than reimplementing citation parsing.
- Temporary fixture data is limited to artifact JSON files, Markdown files, and
  a root `package.json` used for root-file fallback.
- The shared temp repo helper is optional for this pilot. If used, it must only
  provide setup and cleanup; citation semantics stay local to the verifier.
- `npm run test:vitest` must stay scoped to reviewed `tests/*.test.mjs` files.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/citation-verifier.test.mjs`,
2. `npm run test:vitest:citation-verifier`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep every current Node case represented as a
named Vitest `it(...)` block. It should also run both:

- `node tests/test-citation-verifier.mjs`
- `npm run test:vitest:citation-verifier`

Do not migrate resolver, deadness, pre-write, ranking, performance, or
public-package suites as part of the citation-verifier pilot.

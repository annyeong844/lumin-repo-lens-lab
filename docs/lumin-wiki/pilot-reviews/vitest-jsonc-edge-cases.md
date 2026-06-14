# Vitest JSONC Edge Cases Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-jsonc-edge-cases.mjs`.

---

## Purpose

This review decides whether `tests/test-jsonc-edge-cases.mjs` is a reasonable
next Vitest pilot candidate. It does not add the Vitest suite. The goal is to
preserve the JSONC parser regression cases that previously caused scoped
`tsconfig.json` path discovery to silently drop real app aliases.

This suite is a good next test-reform slice because it is small, concrete, and
edge-case driven. It exercises parser inputs that broke a real resolver fix,
but it does not require changing resolver expansion, deadness, ranking,
performance, generated artifact, or cue-tier behavior.

## Reviewed Evidence

- Preserved Node command: `node tests/test-jsonc-edge-cases.mjs`.
- Proposed focused Vitest command: `npm run test:vitest:jsonc-edge-cases`.
- Parser/discovery module under review:
  `_lib/tsconfig-paths.mjs`.
- Related resolver regression suite:
  `node tests/test-tsconfig-paths-scoped.mjs`.
- Test migration board:
  `docs/lumin-wiki/test-migration-candidate-board.md`.

## Result

The suite is acceptable as the next narrow Vitest pilot candidate.

The future mirror should keep the suite focused on JSONC parsing and scoped
path-discovery inputs. It should not broaden into resolver behavior or
dead-export classification. The old Node entrypoint must remain runnable, and
the Vitest suite should express each current edge case as a named runner-level
assertion.

## Protected Invariants

The future Vitest pilot must preserve these JSONC parser contracts:

- `$schema` URLs containing `//` inside string literals do not get stripped as
  comments.
- Real JSONC line comments, block comments, trailing comments, and trailing
  commas parse successfully.
- String literals containing `/* ... */`-looking text stay string literals
  rather than block-comment ranges.
- UTF-8 BOM-prefixed `tsconfig.json` files remain part of the visible parser
  contract.
- App-local `paths` entries survive even when an `extends` target points to a
  missing workspace or hoisted package.
- A duyet-shaped fixture with many sibling app `tsconfig.json` files discovers
  every local `@/*` scope entry, not an arbitrary subset.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- A regex-like comment stripper must not remove `https://...` schema strings.
- A regex-like block-comment stripper must not delete `/* ... */` text inside a
  string literal.
- Strict JSON parsing must not reject valid JSONC comments or trailing commas.
- Missing `extends` targets must not erase local `compilerOptions.paths`.
- A multi-app fixture must fail if only some apps produce scoped path entries.
- The mirror must not hide a future BOM regression by weakening or deleting the
  existing assertion.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-jsonc-edge-cases.mjs` remains runnable.
- The fixture boundary is temporary `tsconfig.json` directory setup only.
- JSONC parser and path-discovery meaning stays local to this suite.
- The pilot must not change resolver scoring, graph edges, deadness, ranking,
  pre-write cues, performance cache identity, or generated artifact policies.
- The pilot must not absorb `test-tsconfig-paths-scoped.mjs`; scoped resolver
  behavior remains a separate suite.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/jsonc-edge-cases.test.mjs`,
2. `npm run test:vitest:jsonc-edge-cases`,
3. a candidate-board update moving this suite from `REVIEWED` to `DONE`.

The implementation PR should keep every current Node case represented as a
named Vitest `it(...)` block. It may use the setup-only temp repo helper for
directory creation and cleanup, but parser semantics, scoped path assertions,
and duyet-shaped edge cases must stay local to this suite.

Run both commands when changing this suite:

- `node tests/test-jsonc-edge-cases.mjs`
- `npm run test:vitest:jsonc-edge-cases`

Do not migrate `test-tsconfig-paths-scoped.mjs`, resolver expansion suites,
deadness/ranking suites, performance/incremental suites, or cue-tier suites as
part of the JSONC edge-case pilot.

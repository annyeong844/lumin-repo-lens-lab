# Vitest Namespace Re-Export Deadness Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-21.
> **Pilot candidate:** `tests/test-namespace-reexport-deadness.mjs`.

---

## Purpose

This review decides whether `tests/test-namespace-reexport-deadness.mjs` may
move to a focused Vitest mirror. The suite protects namespace re-export member
precision in `symbols.json`: `export * as ns from "./source"` must not keep
every source export alive, chained namespace re-exports must preserve exact
member fan-in, and opaque namespace escapes must remain explicit diagnostics.

The key risk is a broad mirror that only proves "symbol graph runs" while
dropping the false-liveness regression guard. A correct mirror must fail if
namespace imports become blanket fan-in, if unused namespace siblings disappear
from dead candidates, or if namespace-object escapes stop surfacing as review
evidence.

## Reviewed Evidence

| Suite                                        | Preserved Node Command                            | Proposed Focused Vitest Command                   | Surface Under Review                                                       |
| -------------------------------------------- | ------------------------------------------------- | ------------------------------------------------- | -------------------------------------------------------------------------- |
| `tests/test-namespace-reexport-deadness.mjs` | `node tests/test-namespace-reexport-deadness.mjs` | `npm run test:vitest:namespace-reexport-deadness` | namespace re-export fan-in, dead candidates, and opaque escape diagnostics |

Goal lane: deadness/ranking graph lens. This is a suite-specific review for
namespace re-export precision, not permission to migrate corpus, ranking,
action-safety, P6 calibration, or general symbol-graph behavior.

Fresh preserved-command evidence on 2026-05-21:

```text
node tests/test-namespace-reexport-deadness.mjs
12 passed, 0 failed
```

## Result

This suite has a focused Vitest mirror in
`tests/namespace-reexport-deadness.test.mjs`, and the mirror stays local to
temporary fixture creation, real `build-symbol-graph.mjs` execution, and
`symbols.json` assertions.

The future mirror may share setup-only helpers for writing small TypeScript
fixtures, running `build-symbol-graph.mjs`, reading `symbols.json`, and cleaning
temporary directories. It must not extract helper logic that decides namespace
member fan-in, broad shadowing, deadness, or opaque escape diagnostics.

## Protected Invariants

The future Vitest mirror must preserve these 12 contracts:

- NR1: a directly used namespace re-exported function receives exact fan-in of
  one through the namespace object.
- NR2: a directly used namespace re-exported const receives exact fan-in of one
  through the namespace object.
- NR3: an unused namespace re-exported function remains a concrete dead export
  candidate.
- NR4: an unused namespace re-exported const remains a concrete dead export
  candidate.
- NR5: namespace re-export usage does not add broad fan-in to unused sibling
  members.
- NR6: a chained namespace re-exported function receives exact fan-in of one.
- NR7: a chained namespace re-exported const receives exact fan-in of one.
- NR8: an unused function behind a chained namespace re-export remains dead.
- NR9: an unused const behind a chained namespace re-export remains dead.
- NR10: chained namespace re-export usage does not add broad fan-in to unused
  sibling members.
- NR11: an opaque namespace object escape keeps target members broad-shadowed
  rather than falsely dead.
- NR12: the opaque namespace escape is recorded in
  `symbols.namespaceReExportDiagnostics[]` with
  `kind: "opaque-namespace-escape"` and
  `reason: "namespace-object-escaped"`.

## Edge-Case Failures To Preserve

The mirror must fail if:

- namespace re-export access becomes blanket-alive for every exported member;
- exact namespace member fan-in is lost for direct namespace imports;
- exact namespace member fan-in is lost through a chained re-export;
- unused namespace siblings are hidden from `deadProdList`;
- `fanInByIdentitySpace[identity].broad` increments for unused siblings in
  precise member-access cases;
- opaque namespace escapes are treated as concrete member usage;
- opaque namespace escapes are treated as clean absence;
- `namespaceReExportDiagnostics[]` stops recording the consumer file,
  exported namespace name, target file, or reason.

## Fixture Boundary

Allowed shared helpers:

- create and clean temporary repositories;
- write small TypeScript files for source, barrel, outer barrel, and consumer
  fixtures;
- run the real `build-symbol-graph.mjs` command with `--production` and
  `--no-incremental`;
- read `symbols.json`;
- assert `fanInByIdentity`, `fanInByIdentitySpace`, `deadProdList`, and
  `namespaceReExportDiagnostics`.

Forbidden helper behavior:

- deciding whether a namespace member is used;
- collapsing direct and chained namespace re-export fixtures into one broad
  helper that hides the original regression shape;
- deciding whether broad fan-in should be present;
- deciding whether a dead candidate should be concrete or shadowed;
- deciding whether a namespace escape is opaque;
- swallowing `build-symbol-graph.mjs` failures or missing `symbols.json`;
- sharing semantic helper logic with corpus, rank-fixes, export-action-safety,
  P6 calibration, module reachability, or resolver blind-zone suites.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The preserved Node command remains runnable and authoritative until a later
  cleanup spec retires it.
- The mirror must not change namespace re-export extraction, member fan-in,
  dead-export classification, ranking, action safety, resolver behavior, P6
  calibration, or module reachability.
- The mirror must not absorb `tests/test-corpus.mjs`,
  `tests/test-export-action-safety.mjs`,
  `tests/test-finding-local-provenance.mjs`, `tests/test-rank-fixes.mjs`, P6
  suites, or `tests/test-audit-repo.mjs`.
- The mirror must not promote opaque namespace escape evidence to `SAFE_FIX`
  proof.

## Recommendation

The narrow implementation PR adds:

1. `tests/namespace-reexport-deadness.test.mjs`;
2. `npm run test:vitest:namespace-reexport-deadness`;
3. candidate-board updates moving this suite from `REVIEWED` to `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the 12 current Node
assertions as named Vitest cases. It remains covered by the preserved Node
command, the focused Vitest command, and the doc guards.

## Validation Commands

The implementation PR must run:

```text
node tests/test-namespace-reexport-deadness.mjs
npm run test:vitest:namespace-reexport-deadness
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-namespace-reexport-deadness.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/test-migration-candidate-board.md
git diff --check
```

# Vitest Export Action Safety Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-21.
> **Pilot candidate:** `tests/test-export-action-safety.mjs`.

---

## Purpose

This review decides whether `tests/test-export-action-safety.mjs` may move to a
focused Vitest mirror. The suite protects `export-action-safety.json`: a dead
export may become a concrete safe edit only when the producer can prove the
selected action and record why stronger actions remain blocked.

The key risk is that a broad mirror could preserve only "SAFE_FIX exists" while
dropping the action proof. A correct mirror must fail if side-effect
initializers, local value/type references, partial multi-declarators,
re-exports, or last-export module marker patches lose their specific safety
contracts.

## Reviewed Evidence

| Suite                                 | Preserved Node Command                     | Proposed Focused Vitest Command            | Surface Under Review                                    |
| ------------------------------------- | ------------------------------------------ | ------------------------------------------ | ------------------------------------------------------- |
| `tests/test-export-action-safety.mjs` | `node tests/test-export-action-safety.mjs` | `npm run test:vitest:export-action-safety` | concrete `safeAction` proof, blockers, and edit patches |

Goal lane: deadness/ranking action proof. This is a suite-specific review for
export action safety, not permission to migrate corpus, rank-fixes,
finding-local-provenance, P6 calibration, or general dead-export ranking.

Fresh preserved-command evidence on 2026-05-21:

```text
node tests/test-export-action-safety.mjs
14 passed, 0 failed
```

## Result

This suite has a focused Vitest mirror in
`tests/export-action-safety.test.mjs`, and the mirror stays local to temporary
root/output creation, source fixture writing, synthetic `dead-classify.json`
and `symbols.json` inputs, real `export-action-safety.mjs` execution,
`export-action-safety.json` assertions, and temporary directory cleanup.

It must not extract helper logic that decides which safe action is valid, which
blocker applies, whether a stronger action is allowed, whether a type
declaration may be deleted, or whether a module marker patch is required.

## Protected Invariants

The future Vitest mirror must preserve these 14 contracts:

- A1: a side-effect initializer selects `demote_export_declaration`.
- A1b: the selected demote action has no selected-action blockers.
- A1c: the side-effect initializer blocks stronger delete action only.
- A2: local value references preserve the binding through demotion.
- A2b: local value references block stronger delete action only.
- A3: local type references preserve the type binding through demotion.
- A3b: local type references block type deletion only.
- A4: an unreferenced interface can delete the type declaration.
- A4b: a B-bucket local type declaration dependency gets a demote action.
- A4c: the B-bucket declaration dependency blocks stronger delete action only.
- A5: a partial multi-declarator has no safe action in v1.
- A5b: the partial multi-declarator records an action blocker.
- A6: a re-export-from-source remains review-only in v1.
- A7: the last-export safe action includes a module marker patch.

## Edge-Case Failures To Preserve

The mirror must fail if:

- deadness alone becomes automated action proof;
- side-effect initializers become delete actions instead of demotion;
- selected actions carry blockers that only belong to stronger actions;
- local value references or local type references are deleted instead of
  preserved by demotion;
- unreferenced type declarations stop allowing type-only deletion;
- B-bucket declaration dependencies stop producing demotion evidence;
- partial multi-declarators get an unsafe partial edit;
- re-export-from-source cases become v1 safe actions;
- a last export is demoted or deleted without inserting `export {};` when the
  file must remain a module.

## Fixture Boundary

Allowed shared helpers:

- create and clean temporary root/output directories;
- write small TypeScript source fixtures;
- synthesize `dead-classify.json` proposal buckets;
- synthesize the minimal `symbols.json` needed by the producer;
- run the real `export-action-safety.mjs` command;
- read `export-action-safety.json`;
- assert `safeAction`, `actionBlockers`, `strongerActionBlockers`, and edit
  patch fields.

Forbidden helper behavior:

- deciding whether an export is safe to demote or delete;
- deciding whether local references are value or type blockers;
- deciding whether partial multi-declarators can be edited;
- deciding whether a re-export-from-source is review-only;
- deciding whether a module marker is required;
- swallowing command failures or missing artifacts;
- sharing semantic helper logic with corpus, rank-fixes,
  finding-local-provenance, namespace re-export, P6 calibration, module
  reachability, or resolver blind-zone suites.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The preserved Node command remains runnable and authoritative until a later
  cleanup spec retires it.
- The mirror must not change `export-action-safety.mjs`, dead-export
  classification, ranking, SAFE_FIX policy, public/framework/generated blockers,
  module reachability, or P6 calibration.
- The mirror must not absorb `tests/test-corpus.mjs`,
  `tests/test-finding-local-provenance.mjs`, `tests/test-rank-fixes.mjs`, P6
  suites, or `tests/test-audit-repo.mjs`.
- The mirror must not promote review-only re-export or partial multi-declarator
  evidence to automated action proof.

## Recommendation

The narrow implementation PR adds:

1. `tests/export-action-safety.test.mjs`;
2. `npm run test:vitest:export-action-safety`;
3. candidate-board updates moving this suite from `REVIEWED` to `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the 14 current Node
assertions as named Vitest cases. It remains covered by the preserved Node
command, the focused Vitest command, and the doc guards.

## Validation Commands

The implementation PR must run:

```text
node tests/test-export-action-safety.mjs
npm run test:vitest:export-action-safety
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-export-action-safety.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/test-migration-candidate-board.md
git diff --check
```

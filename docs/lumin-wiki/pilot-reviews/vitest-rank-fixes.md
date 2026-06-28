# Vitest Rank Fixes Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-21.
> **Pilot candidate:** `tests/test-rank-fixes.mjs`.

---

## Purpose

This review decides whether `tests/test-rank-fixes.mjs` may move to a focused
Vitest mirror. The suite protects the last action-ranking layer between
dead-export evidence and user-facing fix plans: `SAFE_FIX`, `REVIEW_FIX`,
`DEGRADED`, and `MUTED` must remain separated by local proof, blocking taints,
public-package contracts, generated blind zones, and framework/callback
boundaries.

The key risk is that a broad mirror could preserve only "fix-plan exists" while
dropping why a candidate is safe, review-only, degraded, or muted. That would
turn review evidence into automated action proof, or hide the exact blocker
that prevented promotion.

## Reviewed Evidence

| Suite                       | Preserved Node Command           | Proposed Focused Vitest Command  | Surface Under Review                                    |
| --------------------------- | -------------------------------- | -------------------------------- | ------------------------------------------------------- |
| `tests/test-rank-fixes.mjs` | `node tests/test-rank-fixes.mjs` | `npm run test:vitest:rank-fixes` | `tierForFinding()` ranking and `fix-plan.json` assembly |

Goal lane: deadness/ranking action-proof lane. This is a suite-specific review
for ranking and fix-plan evidence, not permission to migrate corpus, P6
calibration, audit-repo umbrella behavior, cue-tier policy, or broad
dead-classification behavior.

Fresh preserved-command evidence on 2026-05-21:

```text
node tests/test-rank-fixes.mjs
45 passed, 0 failed
```

## Result

This suite now has a narrow Vitest mirror at `tests/rank-fixes.test.mjs`, and
the preserved Node command remains runnable. The mirror shares setup-only
helpers for small finding objects, safe-action payloads,
synthetic evidence objects, temp output directories, artifact writers, and JSON
readback. It calls the real `tierForFinding()` and runs the real
`rank-fixes.mjs` producer for integration cases.

The mirror must not extract helper logic that decides tier, reason,
confidence, blocker structure, public contract risk, generated-artifact
relevance, call-graph support, or safe-fix grouping.

## Protected Invariants

The Vitest mirror preserves these 45 contracts:

### Pure Ranking Predicate

- R1: C bucket plus runtime-dead and fossil evidence promotes to `SAFE_FIX`.
- R2: runtime-executed evidence degrades even otherwise strong candidates.
- R3: policy-excluded candidates become `MUTED` regardless of other evidence.
- R4: C bucket with `safeAction` can promote under the static graph without
  runtime or staleness evidence.
- R4b: C bucket without `safeAction` proof stays `REVIEW_FIX`.
- R4c: classify-incomplete bucket becomes `DEGRADED`.
- R5: legacy repo-global resolver blindness still degrades when no local taint
  path exists.
- R6: recent staleness is context and does not block static `SAFE_FIX`.
- R7: A bucket export demotion can be `SAFE_FIX` when safe-action proof exists.
- R8: remove-export-specifier action can be `SAFE_FIX` with strong evidence.
- R8b: `strongerActionBlockers` do not block a weaker selected safe action.
- R8c: selected `actionBlockers` force `REVIEW_FIX`.
- R8d: `requiresModuleMarker` remains part of the safe action.
- R8e: entry-unreachable support yields medium confidence, not high.
- R8e2: entry-unreachable plus call-graph support yields high confidence.
- R8f: soft taint still blocks the entry-unreachable safe-fix boost.
- R8f2: resolver soft taint is not mislabeled as parse errors.
- R8f3: generated artifact provider taint returns structured blocking
  diagnostics.
- R8f4: generated consumer blind-zone taint returns structured blocking
  diagnostics.
- R8g: public deep-import risk blocks `SAFE_FIX`.
- R9: B bucket predicate-partner candidates stay `REVIEW_FIX`.
- R10: SARIF levels map `SAFE_FIX` to warning, review/degraded to note, and
  muted to null.
- R11: exported declaration dependency without safe action stays `REVIEW_FIX`.
- R11b: declaration dependency with demote safe action can be `SAFE_FIX`.
- R11c: exported declaration dependency still blocks delete action.

### Fix-Plan Integration

- I1: `fix-plan.json` summary has four tiers plus total, including muted
  excluded candidates.
- I1b: `excludedCandidates` materialize as `MUTED` findings instead of being
  silently dropped.
- I2: fossil C candidate ranks `SAFE_FIX`.
- I3: recent staleness ranks `SAFE_FIX` when static evidence is clean.
- I4: recent staleness candidate is not review-only under clean static
  evidence.
- I5: `fix-plan.meta.inputs` flags every optional input artifact.
- I6: resolver-blindness gate reports ok on a healthy fixture.
- I6b: local declaration export dependency plus demote action ranks `SAFE_FIX`.
- I6c: two evidence lenses give unreachable file high confidence.
- I6d: reachable file does not receive entry-unreachable support.
- I6e: call graph no-observed-callers adds independent support.
- I7: runtime-hit findings never reach `SAFE_FIX`.
- I7b: public deep-import risk remains `REVIEW_FIX`.
- I7b2: package `files` exclusion can allow `SAFE_FIX` when other proof is
  clean.
- I7b3: npm always-included main files keep `REVIEW_FIX`.
- I7c: package files without package name do not create blanket public
  deep-import risk.
- I7d: framework callback-like exports do not receive call-graph support.
- I7d2: call-graph support is withheld when bounded member-call stats are
  absent.
- I7e: generated blind-zone review entries preserve structured blocking
  diagnostics in `fix-plan.json`.
- I8: `SAFE_FIX` grouping is presentation-only and does not remove raw
  `safeFixes`.

## Edge-Case Failures To Preserve

The mirror must fail if:

- runtime hits are allowed to become `SAFE_FIX`;
- policy exclusions disappear instead of becoming muted evidence;
- missing safe-action proof is treated as enough for automated action;
- stronger-action blockers are confused with blockers for the selected action;
- module-marker insertion demotes otherwise safe edits;
- entry-unreachable evidence alone claims high confidence;
- call-graph support is granted without bounded member-call stats;
- framework callback-like symbols gain call-graph support solely from no direct
  callers;
- resolver soft taint is reported as parse-error taint;
- generated artifact blockers lose structured `blockedBy` diagnostics;
- public packages without explicit exports are treated as safe unless
  `files`/always-included package rules prove the candidate is not published;
- `package.json#files` exclusions incorrectly clear npm always-included entry
  files;
- nameless package fixtures create public deep-import risk;
- `excludedCandidates` are dropped from the plan;
- safe-fix grouping replaces or hides the raw `safeFixes` list.

## Fixture Boundary

Allowed shared helpers:

- construct small finding objects;
- construct safe-action payloads;
- construct runtime, staleness, resolver, policy, public-contract, call-graph,
  and generated-artifact evidence objects;
- create and remove temp output directories;
- write synthesized `dead-classify.json`, `runtime-evidence.json`,
  `staleness.json`, `symbols.json`, `export-action-safety.json`,
  `call-graph.json`, and `package.json` artifacts;
- run `rank-fixes.mjs` as a child process for integration cases;
- read `fix-plan.json` and assert summary, tier arrays, reasons, confidence,
  `blockedBy`, and grouping fields.

Forbidden helper behavior:

- deciding `SAFE_FIX`, `REVIEW_FIX`, `DEGRADED`, or `MUTED`;
- deciding whether a taint is blocking or soft;
- deciding public deep-import risk;
- deciding whether package `files`, `main`, or nameless package facts clear or
  create public risk;
- deciding generated-artifact relevance or `blockedBy` shape;
- deciding framework callback identity;
- deciding call-graph no-observed-caller support;
- deciding whether grouped safe fixes replace raw safe fixes;
- sharing semantic helper logic with corpus, P6 calibration,
  export-action-safety, finding-local provenance, cue-tier policy, or
  audit-repo umbrella suites.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The preserved Node command remains runnable and authoritative until a later
  cleanup spec retires it.
- The mirror must not change `_lib/ranking.mjs`, `rank-fixes.mjs`,
  `export-action-safety.mjs`, dead-export classification, public-surface
  detection, generated-artifact diagnostics, call-graph production, SARIF
  export, or fix application.
- The mirror must not absorb `tests/test-corpus.mjs`,
  `tests/test-finding-local-provenance.mjs`, `tests/test-export-action-safety.mjs`,
  P6 suites, cue-tier policy, or `tests/test-audit-repo.mjs`.
- The mirror must not convert review-only blockers into automated deletion
  language.

## Recommendation

The narrow implementation PR added:

1. `tests/rank-fixes.test.mjs`;
2. `npm run test:vitest:rank-fixes`;
3. candidate-board updates moving this suite from `REVIEWED` to `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the 45 current Node
assertions as named Vitest cases. It remains covered by the preserved Node
command, the focused Vitest command, and the doc guards.

## Validation Commands

The implementation PR must run:

```text
node tests/test-rank-fixes.mjs
npm run test:vitest:rank-fixes
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-rank-fixes.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md
git diff --check
```

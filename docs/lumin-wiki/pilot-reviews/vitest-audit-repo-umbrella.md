# Vitest Audit Repo Umbrella Review

> **Status:** REVIEWED - SPLIT BEFORE MIRROR.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-audit-repo.mjs`

---

## Purpose

This review decides whether `tests/test-audit-repo.mjs` may move directly to a
single focused Vitest mirror. It does not add a Vitest suite.

The answer is no. The suite is still valuable, but it is an umbrella regression
guard for the v1.9.9 product UX pass. It mixes artifact-brief wording,
blind-zone classification, manifest evidence, producer-performance metadata,
scan-range aliases, lifecycle artifact collection, maintainer self-audit
excludes, and full-profile staleness behavior. A single `audit-repo.test.mjs`
mirror would preserve the current file size problem instead of clarifying the
contracts.

## Reviewed Evidence

| Suite                       | Preserved Node Command           | Direct Focused Vitest Command | Surface Under Review             |
| --------------------------- | -------------------------------- | ----------------------------- | -------------------------------- |
| `tests/test-audit-repo.mjs` | `node tests/test-audit-repo.mjs` | _deferred_                    | audit-repo umbrella product pass |

Current Node evidence checked for this review:

```text
node tests/test-audit-repo.mjs # 97 passed, 0 failed
```

Goal lanes: mostly Lane H, with Lane D blind-zone/resolver confidence, Lane F
performance telemetry, and full-profile staleness behavior mixed in. This
review covers only the decision about the umbrella suite. It does not approve a
direct runner migration.

## Result

Do not add `tests/audit-repo.test.mjs` or
`npm run test:vitest:audit-repo` as one mirror.

The suite should stay Node-authoritative until its sections have narrower
review pages. Future implementation may split the current assertions into
smaller mirrors only after each mirror names its own protected invariant,
edge-case failure, preserved Node command, focused Vitest command, and fixture
boundary.

## Suggested Split

The current suite can be split into these future review tracks:

| Future Review Track                   | Current Sections    | Main Contract                                                                     |
| ------------------------------------- | ------------------- | --------------------------------------------------------------------------------- |
| audit artifact brief and review pack  | A0, O4, O7, O10c2   | Markdown summaries remain artifact maps and review packs, not recommendation text |
| blind-zone and confidence diagnostics | B1-B10e, O5, O6, O8 | unsupported or unresolved evidence limits claims without becoming fake certainty  |
| manifest and producer-performance     | O0-O3, O1-O1f4      | manifest and producer-performance metadata expose run, artifact, and phase facts  |
| scan range and self-audit exclusions  | O9, O11, O13, F2    | production aliases, user excludes, and maintainer auto-excludes stay explicit     |
| lifecycle artifact collection         | O12                 | opt-in lifecycle modes list their generated artifacts after running               |
| full-profile staleness and artifacts  | O10a-O10e, H        | subdirectory git roots produce full-profile staleness and optional artifacts      |

The future split may refine these names, but it must not collapse all of the
sections back into one broad mirror.

## Protected Invariants

Any future split mirror must preserve the section it owns without weakening
these umbrella-level contracts:

- unknown long options and unsupported generated-artifact modes exit 2 before
  falling back to defaults;
- audit summaries stay artifact briefs, not ranking or recommendation engines;
- review packs surface lane-specific evidence without telling the agent to paste
  whole lanes as final answers;
- blind-zone detection marks unsupported languages, parse errors, CJS opacity,
  dynamic CJS require calls, unresolved resolver ratios, absolute unresolved
  counts, concentrated unresolved roots, generated consumer scopes, affected
  package scopes, and blocked absence hints;
- clean supported repos still produce zero blind zones;
- `manifest.json` exposes the required evidence sections, producer-performance
  summary, artifact map, memory snapshot, and phase counters;
- quick profile runs the expected producer chain and does not run full-profile
  staleness or runtime evidence;
- console output points readers to produced artifacts and confidence limits
  without stale denominators;
- `--exclude`, generated-artifact mode, production aliases, and maintainer
  auto-excludes are forwarded into both producer inputs and manifest evidence;
- opt-in lifecycle artifacts appear in `artifactsProduced` after those modes
  run;
- full-profile audits rooted inside a git subdirectory still produce staleness,
  optional checklist support artifacts, a review pack, shape-drift cues,
  function-clone cues, and any-contamination lanes.

## Edge-Case Failures To Preserve

The split reviews must keep these failures visible:

- accidentally treating unsupported or unresolved inputs as high-confidence
  absence must fail;
- converting artifact summaries back into recommendation prompts must fail;
- hiding resolver blocked absence hints, generated consumer scopes, or affected
  package scopes must fail;
- dropping producer-performance memory, artifact-size, phase, or source-use
  counters must fail;
- scanning excluded test files under any production alias must fail;
- silently ignoring user excludes or maintainer auto-excludes must fail;
- omitting lifecycle artifacts after `--pre-write` or `--check-canon` runs must
  fail;
- skipping full-profile staleness for a git subdirectory root must fail;
- changing full-profile review-pack wording into external-API or subagent
  ownership instructions must fail.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The Node suite remains runnable and authoritative until the split is complete.
- A future mirror may share temporary-repo creation, command execution, JSON
  reads, Markdown reads, and cleanup helpers.
- Shared helpers must not decide blind-zone severity, manifest completeness,
  producer-performance meaning, scan-range policy, staleness behavior, or
  review-pack wording.
- Do not change `audit-repo.mjs`, `blind-zones.mjs`, `audit-summary.mjs`,
  `audit-review-pack.mjs`, producer-performance emission, staleness collection,
  resolver diagnostics, generated-artifact policy, deadness ranking, or
  action-safety proof in a runner-migration PR.
- Do not absorb already mirrored focused suites such as
  `test-audit-repo-canon-draft.mjs`, `test-audit-repo-check-canon.mjs`,
  `test-audit-repo-pre-write.mjs`, `test-audit-repo-post-write.mjs`, or
  `test-audit-repo-symbol-incremental.mjs`.
- Do not widen `npm run test:vitest` discovery beyond reviewed first-party
  `tests/*.test.mjs` files.

## Recommendation

Park the direct `test-audit-repo.mjs` mirror.

The next review should follow
[`vitest-audit-repo-split-tracks.md`](vitest-audit-repo-split-tracks.md), choose
one split track, and create a narrow review page for that track. Only after that
review exists should an implementation PR add a focused Vitest command. Until
then, keep `node tests/test-audit-repo.mjs` in the authoritative Node suite.

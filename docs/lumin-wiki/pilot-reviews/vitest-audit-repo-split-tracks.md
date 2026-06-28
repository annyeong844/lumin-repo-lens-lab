# Vitest Audit Repo Split-Track Dogfooding Review

> **Status:** REVIEWED - TRACKS ONLY.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-audit-repo.mjs`

---

## Purpose

This review applies the
[`Parked Suite Dogfooding`](../concepts/parked-suite-dogfooding.md) rules to the
parked `tests/test-audit-repo.mjs` umbrella suite. It does not add a Vitest
suite and it does not approve a direct `audit-repo.test.mjs` mirror.

The previous
[`Vitest Audit Repo Umbrella Review`](vitest-audit-repo-umbrella.md) decided
that the whole file must stay parked. This page defines the split tracks that a
future worker may review one at a time before any focused mirror or helper
extraction happens.

## Reviewed Evidence

| Suite                       | Preserved Node Command           | Proposed Focused Vitest Command | Surface Under Review           |
| --------------------------- | -------------------------------- | ------------------------------- | ------------------------------ |
| `tests/test-audit-repo.mjs` | `node tests/test-audit-repo.mjs` | _deferred_                      | audit-repo split-track dogfood |

Fresh Node evidence checked for this review:

```text
node tests/test-audit-repo.mjs
97 passed, 0 failed
```

The suite still acts as a v1.9.9 product UX regression guard. The split below
names review lanes only. Each lane still needs its own focused review page
before a Vitest mirror may be implemented.

## Split Tracks

| Track                                 | Current Assertions  | Protected Contract                                                                                                                                                   | Edge Failure That Must Stay Visible                                                                                               |
| ------------------------------------- | ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| artifact brief and review pack        | A0, O4, O7, O10c2   | Audit Markdown remains an artifact map and reviewer pack, not a recommendation engine.                                                                               | Ranking, numbered recommendations, final-answer prompts, stale denominators, or whole-lane paste instructions reappear.           |
| blind-zone and confidence diagnostics | B1-B10e, O5, O6, O8 | Unsupported languages, parse errors, CJS opacity, resolver gaps, and generated scopes limit claims without becoming fake certainty.                                  | Unsupported or unresolved evidence is treated as high-confidence absence or hidden from summary/review-pack text.                 |
| manifest and producer-performance     | O0-O3, O1-O1f4      | Manifest and `producer-performance.json` expose run metadata, artifact sizes, memory, phase counters, and source-use timings.                                        | Producer timings, memory snapshots, artifact read/parse counters, or phase counters disappear while the run still looks complete. |
| scan range and self-audit exclusions  | O9, O11, O13, F2    | Production aliases, user excludes, generated-artifact mode, and maintainer auto-excludes are forwarded into producers and manifest evidence.                         | Excluded test files or maintainer lab/corpus/generated mirrors enter production scan ranges silently.                             |
| lifecycle artifact collection         | O12                 | Opt-in lifecycle modes list pre-write, any-inventory, and check-canon artifacts only after those modes run.                                                          | Lifecycle artifacts are omitted after opt-in runs or appear during default runs that did not request them.                        |
| full-profile staleness and artifacts  | O10a-O10e, H        | Full-profile audits rooted inside a git subdirectory produce staleness, optional checklist support, review pack, shape-drift, clone, and any-contamination evidence. | Full profile skips staleness or converts unranked support artifacts into recommendation or ownership instructions.                |

## Already Split Elsewhere

Do not pull these already-reviewed surfaces back into the parked umbrella:

- command lifecycle wrappers are covered by
  [`vitest-audit-repo-command-lifecycle.md`](vitest-audit-repo-command-lifecycle.md);
- incremental forwarding through the orchestrator is covered by
  [`vitest-audit-repo-incremental-forwarding.md`](vitest-audit-repo-incremental-forwarding.md);
- direct canon, check-canon, pre-write, post-write, and incremental producer
  algorithms are covered by their own suites or remain parked in their own risk
  lanes.

## Helper Boundary

A future split-track implementation may share only setup and observation code:

- temporary repository creation and cleanup;
- source file writes;
- `audit-repo.mjs` command execution;
- JSON and Markdown artifact reads;
- path and line-ending normalization;
- stdout/stderr capture.

Shared helpers must not decide:

- blind-zone severity or resolver confidence;
- whether a manifest field is complete enough;
- producer-performance meaning;
- scan-range policy;
- lifecycle mode truth;
- staleness eligibility;
- review-pack wording;
- ranking, recommendation, or action-safety proof.

If a helper extraction is proposed before a focused mirror, the helper review
must include an edge-case contract test where the old umbrella suite would have
failed.

## Reviewed Tracks

The first focused review is the **artifact brief and review pack** track. It has
the clearest boundary between pure renderer inputs and audit-run output checks,
and it directly protects the user-facing failure mode where the audit summary
becomes a recommendation prompt.

That focused review now lives in
[`vitest-audit-repo-artifact-brief.md`](vitest-audit-repo-artifact-brief.md).

The next focused review is the **manifest and producer-performance** track. It
now lives in
[`vitest-audit-repo-manifest-performance.md`](vitest-audit-repo-manifest-performance.md).

The next focused review is the **blind-zone and confidence diagnostics** track.
It now lives in
[`vitest-audit-repo-blind-zone-confidence.md`](vitest-audit-repo-blind-zone-confidence.md).

The next focused review is the **scan range and self-audit exclusions** track.
It now lives in
[`vitest-audit-repo-scan-range.md`](vitest-audit-repo-scan-range.md).

The next focused review is the **lifecycle artifact collection** track. It now
lives in
[`vitest-audit-repo-lifecycle-artifacts.md`](vitest-audit-repo-lifecycle-artifacts.md).

The next focused review is the **full-profile staleness and artifacts** track.
It now lives in
[`vitest-audit-repo-full-profile-staleness.md`](vitest-audit-repo-full-profile-staleness.md).

The umbrella `tests/test-audit-repo.mjs` suite remains parked and
Node-authoritative for all tracks that have not been separately reviewed and
mirrored.

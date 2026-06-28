# Vitest Audit Repo Artifact Brief Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-audit-repo.mjs` artifact brief / review-pack
> split track.

---

## Purpose

This review narrows the parked `tests/test-audit-repo.mjs` umbrella suite to
one split track from
[`vitest-audit-repo-split-tracks.md`](vitest-audit-repo-split-tracks.md):
artifact brief and review-pack behavior.

It does not add a Vitest suite. It approves a future focused mirror only for
the A0/O4/O7/O10c2 assertions that prove audit Markdown and console previews
stay artifact maps, not recommendation prompts.

## Reviewed Evidence

| Source Suite                | Preserved Node Command           | Proposed Focused Vitest Command                 | Surface Under Review               |
| --------------------------- | -------------------------------- | ----------------------------------------------- | ---------------------------------- |
| `tests/test-audit-repo.mjs` | `node tests/test-audit-repo.mjs` | `npm run test:vitest:audit-repo-artifact-brief` | audit summary and review-pack text |
| future focused mirror       | _deferred_                       | `tests/audit-repo-artifact-brief.test.mjs`      | split-track mirror file            |

Fresh Node evidence checked during the split-track review:

```text
node tests/test-audit-repo.mjs
97 passed, 0 failed
```

This review uses that passing Node evidence as the baseline. A future
implementation PR must rerun the preserved Node command before adding the
focused mirror.

## Owned Assertions

The future mirror may own only these assertion groups:

| Current Assertions | Required Coverage                                                                                                                                                                                          | Preferred Fixture Mode                                 |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| A0pre, A0pre2      | Unknown long options and unsupported generated-artifact modes exit 2 before defaults are applied.                                                                                                          | real `audit-repo.mjs` command execution                |
| A0a-A0f4           | `renderAuditSummary` and `renderAuditReviewPack` output artifact briefs, unranked measured cues, expansion hints, living-audit tracking, resolver blocker warnings, and framework/resource surface counts. | direct renderer input objects                          |
| O4-O4h             | Actual audit runs enumerate produced artifacts, write summary Markdown, map evidence without ranking it, expose living-audit tracking, and write topology Mermaid.                                         | real `audit-repo.mjs` temp repository run              |
| O7-O7d             | Console output points to blind-zone review, artifact counts, summary paths, and artifact-brief preview without stale denominators or recommendation wording.                                               | real `audit-repo.mjs` temp repository run              |
| O10c2              | Full-profile audit writes the Claude Code review-pack artifact.                                                                                                                                            | real full-profile `audit-repo.mjs` temp repository run |

The mirror must not absorb B-series blind-zone semantics, O1 producer
performance metadata, O9/O11 scan-range forwarding, O12 lifecycle artifacts, or
O10 staleness behavior beyond the O10c2 review-pack existence check.

## Protected Invariants

The focused mirror must preserve these contracts:

- `audit-summary.latest.md` starts as an audit artifact brief, not a ranked
  recommendation engine;
- summary text tells readers not to paste it as the final user answer;
- measured cues stay unranked and point to raw artifacts;
- review packs surface lane-specific evidence without telling the agent to copy
  whole lanes as final answers;
- resolver blocked absence hints and framework/resource surface summaries remain
  review evidence, not action proof;
- console preview names produced artifacts and summary paths without stale
  denominators;
- generated `topology.mermaid.md` remains a human visual companion, not a
  hidden graph dependency for ranking;
- full-profile review-pack generation stays observable through the produced
  artifact list.

## Edge-Case Failures To Preserve

The future mirror must fail if:

- the summary emits numbered recommendations or coding-agent prompts;
- artifact maps stop naming `discipline.json`, `call-graph.json`, or
  `symbols.json` where the source evidence exists;
- post-write baseline-missing output claims clean zero deltas;
- living-audit tracking disappears from manifest or summary preview;
- resolver blocked absence hints disappear from Lane 3 review text;
- framework/resource surface counts disappear from summary or review pack;
- console output reports stale artifact denominators;
- review-pack wording becomes external-API advice, subagent ownership
  instruction, or final-answer copy guidance.

## Helper Boundary

Allowed shared helper behavior:

- create a temporary repo with a tiny `package.json` and source files;
- invoke `audit-repo.mjs` with explicit args;
- capture stdout and stderr separately;
- read JSON and Markdown artifacts from the output directory;
- normalize path separators and line endings for assertions;
- clean up temporary directories.

Forbidden helper behavior:

- decide whether text is a recommendation or an artifact brief;
- classify blind-zone severity;
- decide producer-performance completeness;
- rank measured cues;
- infer action-safety or `SAFE_FIX` proof;
- hide whether an assertion came from a direct renderer fixture or a real audit
  run.

## Recommendation

Proceed to one focused implementation PR that adds:

1. `tests/audit-repo-artifact-brief.test.mjs`;
2. `npm run test:vitest:audit-repo-artifact-brief`;
3. candidate-board updates moving only this split track to mirrored coverage.

The implementation PR must keep `node tests/test-audit-repo.mjs` runnable and
must not remove or weaken the umbrella Node suite. It should first watch the
focused Vitest command fail because it is missing, then add named Vitest cases
for the owned assertion groups above.

## Implementation Note

The focused mirror now lives at `tests/audit-repo-artifact-brief.test.mjs` and
is runnable with `npm run test:vitest:audit-repo-artifact-brief`. The umbrella
`node tests/test-audit-repo.mjs` command remains the authoritative suite for
all other parked split tracks.

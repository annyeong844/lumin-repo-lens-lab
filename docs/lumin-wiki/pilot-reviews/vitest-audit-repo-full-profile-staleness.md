# Vitest Audit Repo Full-Profile Staleness Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-23.
> **Pilot candidate:** `tests/test-audit-repo.mjs` full-profile staleness and
> artifacts split track.

---

## Purpose

This review narrows the parked `tests/test-audit-repo.mjs` umbrella suite to
one split track from
[`vitest-audit-repo-split-tracks.md`](vitest-audit-repo-split-tracks.md):
full-profile staleness and artifacts.

It is implemented by `tests/audit-repo-full-profile-staleness.test.mjs`, a
focused mirror only for the O10a-O10e/H assertions that prove full-profile
audits rooted inside a git subdirectory still produce staleness, optional
support artifacts, review-pack evidence, shape-drift cues, function-clone cues,
and any-contamination lanes.

## Reviewed Evidence

| Source Suite                | Preserved Node Command           | Focused Vitest Command                                  | Surface Under Review                 |
| --------------------------- | -------------------------------- | ------------------------------------------------------- | ------------------------------------ |
| `tests/test-audit-repo.mjs` | `node tests/test-audit-repo.mjs` | `npm run test:vitest:audit-repo-full-profile-staleness` | full-profile staleness and artifacts |
| focused mirror              | _implemented_                    | `tests/audit-repo-full-profile-staleness.test.mjs`      | split-track mirror file              |

Fresh implementation evidence:

```text
npm run test:vitest:audit-repo-full-profile-staleness
1 file passed, 1 test passed
```

```text
node tests/test-audit-repo.mjs
97 passed, 0 failed
```

The preserved Node suite remains runnable and authoritative for unrelated
`test-audit-repo.mjs` behavior.

## Owned Assertions

The future mirror may own only these assertion groups:

| Current Assertions | Required Coverage                                                                                                                       | Fixture Mode                                 |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| O10a               | Full-profile audits run `measure-staleness.mjs` when `--root` points at a subdirectory inside a larger git worktree.                    | real full-profile `audit-repo.mjs` temp repo |
| O10b               | The subdirectory-root full-profile run produces and lists `staleness.json`.                                                             | real full-profile `audit-repo.mjs` temp repo |
| O10c               | Full-profile support producers and artifacts are visible: call graph, barrel discipline, shape index, and function clone index.         | real full-profile `audit-repo.mjs` temp repo |
| O10c2              | The full-profile run writes a Claude Code review pack with reviewer-lane wording and without whole-lane paste instructions.             | real full-profile `audit-repo.mjs` temp repo |
| O10d               | Audit summary maps exact shape-drift cues to `shape-index.json` without ranking them.                                                   | real full-profile `audit-repo.mjs` temp repo |
| O10d2              | Summary and review pack surface function-clone cue counts and cite `function-clones.json` without converting them into recommendations. | real full-profile `audit-repo.mjs` temp repo |
| O10e               | Summary and review pack expose exported any-contamination and point readers at `symbols.json.typeOwnersByIdentity` owner maps.          | real full-profile `audit-repo.mjs` temp repo |

The mirror must not absorb quick-profile manifest/performance counters,
scan-range forwarding, lifecycle artifact collection, blind-zone confidence
policy, or ranking/action-safety proof.

## Protected Invariants

The focused mirror must preserve these contracts:

- full profile runs staleness even when the audit root is a subdirectory of a
  larger git repository;
- `staleness.json` is produced and listed in `manifest.json.artifactsProduced`;
- optional full-profile support producers stay observable through both
  `commandsRun` and artifact names;
- `audit-review-pack.latest.md` remains a reviewer-lane pack for Claude Code,
  not an external-API instruction or whole-lane paste prompt;
- exact shape-drift cue counts stay unranked and cite `shape-index.json`;
- function-clone cue counts stay unranked and cite `function-clones.json`;
- exported any-contamination stays framed as owner-map evidence from
  `symbols.json.typeOwnersByIdentity`;
- full-profile support artifacts do not become `SAFE_FIX`, ranking, or
  ownership proof in this mirror.

## Edge-Case Failures To Preserve

The future mirror must fail if:

- a git subdirectory root causes full profile to skip staleness;
- `staleness.json`, `call-graph.json`, `barrels.json`, `shape-index.json`, or
  `function-clones.json` disappears from the full-profile artifact list;
- review-pack wording tells an agent to paste a whole lane into a reviewer;
- shape-drift or function-clone evidence becomes a ranked recommendation;
- any-contamination owner-map evidence disappears from either summary or review
  pack text;
- the mirror starts asserting scan-range, lifecycle, producer-performance, or
  blind-zone confidence behavior already owned by other split tracks.

## Helper Boundary

Allowed shared helper behavior:

- create a temporary git repository with a nested audit root;
- write small TS fixtures that trigger staleness, shape-drift, clone, and
  any-contamination support evidence;
- initialize git metadata and commit the fixture;
- invoke `audit-repo.mjs --profile full --production`;
- read `manifest.json`, `checklist-facts.json`, `audit-summary.latest.md`, and
  `audit-review-pack.latest.md`;
- normalize path separators and line endings for assertions;
- clean up temporary directories.

Forbidden helper behavior:

- decide staleness eligibility;
- classify shape-drift or function-clone evidence;
- rank measured cues;
- infer action-safety or `SAFE_FIX` proof;
- rewrite review-pack wording;
- hide that the mirror depends on a real full-profile `audit-repo.mjs` run.

## Implementation Note

The focused mirror now owns this split track and the candidate board marks it
`DONE`. Keep `node tests/test-audit-repo.mjs` runnable and do not remove or
weaken the umbrella Node suite. This mirror is heavier than the other
audit-repo splits only because this contract genuinely requires a real
full-profile git fixture; do not use it as a dumping ground for unrelated
audit-repo behavior.

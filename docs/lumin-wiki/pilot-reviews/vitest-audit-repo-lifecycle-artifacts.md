# Vitest Audit Repo Lifecycle Artifacts Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-23.
> **Pilot candidate:** `tests/test-audit-repo.mjs` lifecycle artifact
> collection split track.

---

## Purpose

This review narrows the parked `tests/test-audit-repo.mjs` umbrella suite to
one split track from
[`vitest-audit-repo-split-tracks.md`](vitest-audit-repo-split-tracks.md):
lifecycle artifact collection.

It is implemented by the focused Vitest mirror
`tests/audit-repo-lifecycle-artifacts.test.mjs`. The mirror covers only the O12
assertions that prove opt-in lifecycle modes list their generated artifacts in
`manifest.json.artifactsProduced` after those modes run.

## Reviewed Evidence

| Source Suite                | Preserved Node Command           | Focused Vitest Command                               | Surface Under Review          |
| --------------------------- | -------------------------------- | ---------------------------------------------------- | ----------------------------- |
| `tests/test-audit-repo.mjs` | `node tests/test-audit-repo.mjs` | `npm run test:vitest:audit-repo-lifecycle-artifacts` | lifecycle artifact collection |
| focused mirror              | _implemented_                    | `tests/audit-repo-lifecycle-artifacts.test.mjs`      | split-track mirror file       |

Fresh evidence checked during implementation:

```text
npm run test:vitest:audit-repo-lifecycle-artifacts
1 file passed, 1 test passed

node tests/test-audit-repo.mjs
97 passed, 0 failed
```

The umbrella Node suite remains authoritative for the wider audit-repo product
pass and for all split tracks not listed below.

## Owned Assertions

The mirror owns only these assertion groups:

| Current Assertions | Required Coverage                                                                          | Fixture Mode                        |
| ------------------ | ------------------------------------------------------------------------------------------ | ----------------------------------- |
| O12a               | `--pre-write` runs produce and list `pre-write-advisory.latest.json`.                      | real `audit-repo.mjs` temp repo run |
| O12b               | `--pre-write` lifecycle runs include a timestamped `any-inventory.pre.<id>.json` snapshot. | real `audit-repo.mjs` temp repo run |
| O12c               | `--check-canon --sources all` runs produce and list `canon-drift.json`.                    | real `audit-repo.mjs` temp repo run |

The mirror must not absorb direct pre-write command lifecycle behavior,
check-canon algorithm contracts, artifact-summary wording, scan-range policy,
producer-performance metadata, or full-profile staleness behavior.

## Protected Invariants

The focused mirror must preserve these contracts:

- lifecycle artifacts appear in `manifest.json.artifactsProduced` after their
  opt-in modes run;
- pre-write advisory output remains listed by its stable latest artifact name;
- any-inventory pre snapshots remain timestamped or invocation-specific rather
  than collapsed into a fake stable name;
- check-canon output remains listed as `canon-drift.json`;
- the mirror proves artifact collection only, not lifecycle command semantics
  or canon/pre-write correctness.

## Edge-Case Failures To Preserve

The mirror must fail if:

- `--pre-write` runs stop listing `pre-write-advisory.latest.json`;
- timestamped `any-inventory.pre.*.json` snapshots disappear from the manifest;
- `--check-canon --sources all` runs stop listing `canon-drift.json`;
- the mirror starts asserting renderer wording, scan range, staleness, or
  producer-performance behavior;
- lifecycle artifacts appear in this mirror without the corresponding opt-in
  mode having run.

## Helper Boundary

Allowed shared helper behavior:

- create a temporary repository with a tiny package, source file, and intent
  file;
- invoke `audit-repo.mjs` with explicit `--pre-write`, `--intent`,
  `--check-canon`, and `--sources all` args;
- read `manifest.json`;
- clean up temporary directories.

Forbidden helper behavior:

- decide whether pre-write or check-canon output is semantically correct;
- infer canon drift meaning;
- inspect or rank advisory findings;
- classify scan-range, blind-zone, staleness, or producer-performance evidence;
- hide that this mirror uses a real `audit-repo.mjs` run.

## Recommendation

Keep this mirror as a manifest artifact-collection guard. Direct command
lifecycle wrappers remain covered by `vitest-audit-repo-command-lifecycle.md`,
and future audit-repo umbrella work should choose the remaining full-profile
staleness track separately.

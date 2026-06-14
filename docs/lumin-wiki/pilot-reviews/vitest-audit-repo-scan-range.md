# Vitest Audit Repo Scan Range Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-23.
> **Pilot candidate:** `tests/test-audit-repo.mjs` scan range and self-audit
> exclusions split track.

---

## Purpose

This review narrows the parked `tests/test-audit-repo.mjs` umbrella suite to
one split track from
[`vitest-audit-repo-split-tracks.md`](vitest-audit-repo-split-tracks.md): scan
range and self-audit exclusions.

It is implemented by the focused Vitest mirror
`tests/audit-repo-scan-range.test.mjs`. The mirror covers only the O9, O11, and
O13 assertions that prove user excludes, production scan aliases, generated
artifact mode, and maintainer self-audit auto-excludes reach both producer
inputs and manifest evidence.

## Reviewed Evidence

| Source Suite                | Preserved Node Command           | Focused Vitest Command                      | Surface Under Review                 |
| --------------------------- | -------------------------------- | ------------------------------------------- | ------------------------------------ |
| `tests/test-audit-repo.mjs` | `node tests/test-audit-repo.mjs` | `npm run test:vitest:audit-repo-scan-range` | scan range and self-audit exclusions |
| focused mirror              | _implemented_                    | `tests/audit-repo-scan-range.test.mjs`      | split-track mirror file              |

Fresh evidence checked during implementation:

```text
npm run test:vitest:audit-repo-scan-range
1 file passed, 3 tests passed

node tests/test-audit-repo.mjs
97 passed, 0 failed
```

The umbrella Node suite remains authoritative for the wider audit-repo product
pass and for all split tracks not listed below.

## Owned Assertions

The mirror owns only these assertion groups:

| Current Assertions | Required Coverage                                                                                                                                   | Fixture Mode                         |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| O9                 | `--exclude` reaches `build-symbol-graph`, manifest `scanRange.excludes`, manifest generated-artifact mode, and generated-consumer blind-zone facts. | real `audit-repo.mjs` temp repo run  |
| O11                | `--production`, `--no-tests`, `--exclude-tests`, and `--include-tests=false` exclude test files in manifest, triage, and symbol graph evidence.     | real `audit-repo.mjs` temp repo runs |
| O13                | Maintainer self-audit mode records automatic excludes and keeps lab/corpus/generated mirror definitions out of `symbols.json`.                      | real `audit-repo.mjs` temp repo run  |

The mirror must not absorb artifact-summary wording, blind-zone severity,
producer-performance metadata, lifecycle artifact collection, or full-profile
staleness behavior.

## Protected Invariants

The focused mirror must preserve these contracts:

- user `--exclude` entries are forwarded into producer scan scope, not only
  recorded in CLI metadata;
- manifest `scanRange.excludes` records user-supplied excludes;
- generated artifact mode stays visible in manifest evidence and producer
  blind-zone details;
- all production scan aliases set `includeTests: false` and `production: true`;
- triage and symbol graph outputs do not keep `.test.` files under production
  scan aliases;
- maintainer self-audit checkouts auto-exclude lab, corpus, generated mirror,
  and test-harness directories;
- auto-excluded maintainer mirror files do not appear in `symbols.defIndex`;
- excluded files do not become silent absence proof elsewhere in this mirror.

## Edge-Case Failures To Preserve

The mirror must fail if:

- user-excluded `output/` files leak into symbol definitions;
- manifest scan range loses the user exclude or generated artifact mode;
- generated consumer blind zones stop recording `mode: "prepared"` and unknown
  stale provenance;
- any production alias leaves `.test.ts` files in triage or symbol graph output;
- maintainer self-audit excludes disappear from `manifest.scanRange.autoExcludes`;
- maintainer lab/corpus/generated mirror files leak into definitions;
- this split starts asserting summary wording, lifecycle artifacts, staleness, or
  blind-zone confidence policy.

## Helper Boundary

Allowed shared helper behavior:

- create temporary repositories with tiny package/source fixtures;
- invoke `audit-repo.mjs` with explicit `--root`, `--output`, profile, exclude,
  generated-artifact, and production-scope args;
- read `manifest.json`, `triage.json`, and `symbols.json`;
- normalize path separators only for assertion readability;
- clean up temporary directories.

Forbidden helper behavior:

- decide scan-range policy;
- infer generated-artifact stale status;
- classify blind-zone severity;
- rank findings or infer action-safety proof;
- hide whether an assertion came from a real `audit-repo.mjs` run;
- absorb lifecycle, producer-performance, or full-profile staleness behavior.

## Recommendation

Keep this mirror narrow. Future audit-repo split work should choose one of the
remaining tracks from `vitest-audit-repo-split-tracks.md` instead of adding a
direct broad `audit-repo.test.mjs` mirror.

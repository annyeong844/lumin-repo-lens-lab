# Vitest Audit Repo Manifest Performance Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-19.
> **Pilot candidate:** `tests/test-audit-repo.mjs` manifest /
> producer-performance split track.

---

## Purpose

This review narrows the parked `tests/test-audit-repo.mjs` umbrella suite to
one split track from
[`vitest-audit-repo-split-tracks.md`](vitest-audit-repo-split-tracks.md):
manifest and `producer-performance.json` evidence.

It is implemented by the focused Vitest mirror
`tests/audit-repo-manifest-performance.test.mjs`. The mirror covers only the
O0-O3 and O1-O1f4 assertions that prove audit runs expose comparable metadata,
artifact-size totals, artifact read/parse counters, memory snapshots, producer
phase counters, and source-use resolver timings.

## Reviewed Evidence

| Source Suite                | Preserved Node Command           | Proposed Focused Vitest Command                       | Surface Under Review                       |
| --------------------------- | -------------------------------- | ----------------------------------------------------- | ------------------------------------------ |
| `tests/test-audit-repo.mjs` | `node tests/test-audit-repo.mjs` | `npm run test:vitest:audit-repo-manifest-performance` | manifest and producer-performance evidence |
| focused mirror              | _implemented_                    | `tests/audit-repo-manifest-performance.test.mjs`      | split-track mirror file                    |

Fresh Node evidence checked during this split-track review:

```text
node tests/test-audit-repo.mjs
97 passed, 0 failed

npm run test:vitest:audit-repo-manifest-performance
2 passed, 0 failed
```

This review uses the passing Node evidence as the baseline. The focused mirror
stays narrower than the umbrella suite and keeps `node tests/test-audit-repo.mjs`
authoritative for the remaining parked split tracks.

## Owned Assertions

The focused mirror owns only these assertion groups:

| Current Assertions | Required Coverage                                                                                                                                                                       | Preferred Fixture Mode                    |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
| O0a-O0b            | Default `.audit` output emits a privacy note, while explicit output outside the root emits a location note instead.                                                                     | real `audit-repo.mjs` temp repository run |
| O1-O1b             | `manifest.json` exposes profile, commandsRun, scanRange, confidence, resolver diagnostics, blind zones, generated-artifact metadata, and the producer-performance summary mirror.       | real `audit-repo.mjs` temp repository run |
| O1c-O1e            | `producer-performance.json` records schema, root/output/profile, scan range, producer timings, artifact sizes, artifact read/parse counters, and honest orchestrator memory snapshots.  | real `audit-repo.mjs` temp repository run |
| O1f-O1f4           | Heavy quick producers expose phase support, topology scanner/resolver counters, symbol graph extraction counters, assembly subphase timings, and source-use resolver operation timings. | real `audit-repo.mjs` temp repository run |
| O2-O3              | Quick profile runs the expected quick producers and excludes full-profile/runtime-only producers.                                                                                       | real `audit-repo.mjs` temp repository run |

The mirror does not absorb A0 artifact-summary wording, B-series blind-zone
semantics, O4/O7 artifact brief console previews, O9/O11 scan-range forwarding,
O12 lifecycle artifacts, or O10 full-profile staleness behavior.

## Protected Invariants

The focused mirror preserves these contracts:

- output location notes distinguish default `.audit/` privacy warnings from
  explicit outside-root output locations;
- `manifest.json.performance` points at `producer-performance.json`;
- manifest producer count mirrors `commandsRun.length`;
- `producer-performance.json` stays schema-versioned and records root, output,
  profile, and scan-range facts;
- every producer record keeps name, status, wall time, and memory snapshots;
- artifact-size totals and largest-artifact lists remain visible in
  `producer-performance.json` and mirrored in manifest performance summary;
- orchestrator artifact read counts, bytes, read time, and JSON parse time stay
  visible and mirrored into manifest performance fields;
- memory telemetry stays honest about being orchestrator-process snapshots, not
  child-process peak RSS;
- phase support remains present for heavy quick producers;
- topology counters keep scanner, parser, and resolver memoization facts
  visible;
- symbol graph counters keep extraction, graph assembly, source-use branch, and
  resolver-stage timings visible;
- quick profile runs triage, symbol graph, resolver diagnostics,
  dead-classification, and ranking producers;
- quick profile does not silently run full-profile call graph, barrel,
  shape-index, function-clone, runtime evidence, or staleness producers.

## Edge-Case Failures To Preserve

The mirror must fail if:

- output privacy and explicit-output notes collapse into the same message;
- `manifest.performance.artifact` stops naming `producer-performance.json`;
- producer counts drift between manifest and producer-performance output;
- artifact-size or largest-artifact lists disappear while the run still looks
  complete;
- artifact read/parse counters disappear or stop mirroring into manifest;
- memory fields imply unavailable child peak RSS is measured;
- heavy producer phase arrays disappear;
- topology scanner/parser/resolver counters disappear;
- symbol graph extraction, assembly, source-use, or resolver-stage counters
  disappear;
- quick profile starts running full-profile-only producers;
- the focused mirror hides which assertion came from manifest, producer
  performance, or command-profile behavior.

## Helper Boundary

Allowed shared helper behavior:

- create a temporary repository with a small package and TypeScript source
  files;
- invoke `audit-repo.mjs` with explicit `--root`, `--output`, `--profile`, and
  scan-range flags;
- capture stdout and stderr separately;
- read `manifest.json` and `producer-performance.json`;
- normalize path separators and line endings for assertions;
- clean up temporary directories.

Forbidden helper behavior:

- decide whether producer-performance metadata is complete enough;
- infer performance regressions from a single run;
- classify producer phases by importance;
- hide artifact read/parse counter names behind broad truthiness;
- merge scan-range, lifecycle, staleness, blind-zone, or renderer semantics into
  this mirror;
- rank producers or convert timing evidence into action recommendations.

## Recommendation

Keep this mirror narrow. Future work must not absorb blind-zone semantics,
scan-range forwarding, lifecycle artifacts, full-profile staleness, renderer
wording, or recommendation/action-safety policy into this manifest/performance
mirror.

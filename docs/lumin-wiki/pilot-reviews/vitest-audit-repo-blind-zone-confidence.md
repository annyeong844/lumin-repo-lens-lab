# Vitest Audit Repo Blind-Zone Confidence Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-23.
> **Pilot candidate:** `tests/test-audit-repo.mjs` blind-zone and confidence
> diagnostics split track.

---

## Purpose

This review narrows the parked `tests/test-audit-repo.mjs` umbrella suite to
one split track from
[`vitest-audit-repo-split-tracks.md`](vitest-audit-repo-split-tracks.md):
blind-zone and confidence diagnostics.

It is implemented by the focused Vitest mirror
`tests/audit-repo-blind-zones.test.mjs`. The mirror covers only the B-series
and O5/O6/O8 assertions that prove unsupported languages, parse errors, CJS
opacity, resolver gaps, generated consumer scopes, and candidate
blocked-absence hints limit claims instead of becoming fake certainty.

## Reviewed Evidence

| Source Suite                | Preserved Node Command           | Proposed Focused Vitest Command              | Surface Under Review                  |
| --------------------------- | -------------------------------- | -------------------------------------------- | ------------------------------------- |
| `tests/test-audit-repo.mjs` | `node tests/test-audit-repo.mjs` | `npm run test:vitest:audit-repo-blind-zones` | blind-zone and confidence diagnostics |
| focused mirror              | _implemented_                    | `tests/audit-repo-blind-zones.test.mjs`      | split-track mirror file               |

Fresh Node evidence checked during the split-track review:

```text
node tests/test-audit-repo.mjs
97 passed, 0 failed

npm run test:vitest:audit-repo-blind-zones
1 file passed, 8 tests passed
```

This review uses that passing Node evidence as the baseline. A future
implementation PR must rerun the preserved Node command before adding the
focused mirror.

## Owned Assertions

The future mirror may own only these assertion groups:

| Current Assertions | Required Coverage                                                                                                                                                 | Preferred Fixture Mode                       |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| B1-B2c             | Rust, Python, and Go unavailable surfaces create scan or precision gaps rather than repo-wide absence claims.                                                     | direct `detectBlindZones()` fixtures         |
| B3-B4b             | Resolver confidence gaps preserve threshold policy, high absolute unresolved counts, grouped unresolved reasons, and concentrated unresolved prefixes.            | direct `detectBlindZones()` fixtures         |
| B5-B5c             | Parse errors, opaque CJS export surfaces, and dynamic CJS require calls create precision gaps.                                                                    | direct `detectBlindZones()` fixtures         |
| B6-B8              | Clean supported repositories produce zero blind zones, and `formatBlindZonesSummary()` reports or suppresses output deterministically.                            | direct helper fixtures                       |
| B9-B10e            | Summary text surfaces resolver reason summaries, unresolved roots, generated consumer scopes, affected package scopes, and candidate-level blocked absence hints. | direct summary/review renderer input objects |
| O5-O6b             | Real clean TS audits produce zero blind zones while manifest confidence and extractor availability fields stay populated.                                         | real `audit-repo.mjs` temp repository run    |
| O8                 | Real Python-containing audits surface a Python precision or scan gap instead of silently claiming full precision.                                                 | real `audit-repo.mjs` temp repository run    |

The mirror must not absorb A0 artifact-summary wording, O1 producer
performance metadata, O9/O11 scan-range forwarding, O12 lifecycle artifacts, or
O10 full-profile staleness behavior.

## Protected Invariants

The focused mirror must preserve these contracts:

- unsupported source languages create scan gaps or precision gaps, not
  repo-wide absence proof;
- unavailable extractors create explicit gap evidence rather than pretending
  the language was fully analyzed;
- resolver gaps preserve policy version, threshold values, absolute unresolved
  counts, grouped reasons, and concentrated unresolved roots;
- parse errors, opaque CJS exports, and dynamic CJS require calls remain
  precision gaps;
- clean supported TS fixtures still produce zero blind zones;
- blind-zone summaries include severity counts when zones exist and return no
  summary when they do not;
- audit summaries surface resolver reason counts, unresolved roots, generated
  consumer scopes, affected package scopes, and blocked absence hint samples;
- `manifest.confidence` keeps parse-error, unresolved-ratio, and external
  import fields populated;
- per-language extractor availability remains visible in `symbols.json`;
- Python-containing audit runs surface a language precision or scan gap.

## Edge-Case Failures To Preserve

The future mirror must fail if:

- unsupported Rust, Python, or Go files are treated as clean absence;
- a missing Python or Go extractor produces a clean result instead of a scan
  gap;
- resolver unresolved ratios below the ratio threshold hide high absolute
  unresolved counts or concentrated unresolved roots;
- grouped unresolved reason details disappear from the blind-zone record;
- parse errors, CJS opacity, or dynamic CJS requires stop degrading precision;
- clean TS-only fixtures start producing spurious blind zones;
- generated consumer blind zones, affected package scopes, or blocked absence
  hints disappear from audit summary text;
- manifest confidence fields or extractor-availability metadata disappear while
  the audit still looks complete;
- the focused mirror hides whether evidence came from direct helper input or a
  real `audit-repo.mjs` run.

## Helper Boundary

Allowed shared helper behavior:

- create temporary repositories for clean TS and Python-containing audit runs;
- invoke `audit-repo.mjs` with explicit `--root`, `--output`, and profile args;
- read `manifest.json`, `symbols.json`, and summary Markdown;
- build direct `detectBlindZones()` and summary renderer input objects;
- normalize path separators and line endings for assertions;
- clean up temporary directories.

Forbidden helper behavior:

- decide blind-zone severity;
- decide resolver threshold policy;
- collapse unsupported-language, parser, CJS, generated-consumer, and resolver
  gaps into one generic assertion;
- infer action-safety or `SAFE_FIX` proof from blind-zone output;
- hide whether a fixture is direct helper input or a real audit run;
- absorb scan-range, lifecycle, producer-performance, or full-profile
  staleness behavior.

## Recommendation

Keep this mirror narrow. Future work must not absorb artifact-summary wording,
manifest/producer-performance metadata, scan-range forwarding, lifecycle
artifacts, full-profile staleness, deadness ranking, or action-safety proof into
this blind-zone/confidence mirror.

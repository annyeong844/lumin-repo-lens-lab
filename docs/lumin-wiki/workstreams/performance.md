# Performance Workstream

Performance work should be measurement-led. Lumin has already seen large wins
from resolver cache instrumentation, but further work should avoid speculative
architecture changes without counters.

## Current Themes

- `producer-performance.json` is the main evidence artifact.
- Resolver-stage cache wins are measured through stage timings and hit/miss
  counters.
- Scoped baseUrl, scoped tsconfig, and wildcard alias caches closed the largest
  uncached resolver stages on cal.diy.
- Scanner and producer fusion ideas remain design work unless corpus counters
  justify implementation.

## Test Inventory

| Suite Or Evidence                                           | Risk Type                     | Protected Invariant                                                                                                                | Edge Case Or Negative Guard                                                                                                                                     |
| ----------------------------------------------------------- | ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `tests/test-audit-repo.mjs`                                 | producer-performance artifact | Audit runs emit `producer-performance.json` with producer timings, artifact sizes, JSON read/parse counters, and memory snapshots. | Performance metadata must remain evidence, not recommendation prose.                                                                                            |
| `tests/test-classify-performance-metadata.mjs`              | producer metadata             | Classification performance metadata and text-zero shortcuts stay explicit.                                                         | Fast paths must still expose when a shortcut was used.                                                                                                          |
| `tests/test-js-module-edge-scanner.mjs`                     | scanner equivalence           | Tokenizer-state scanner fixtures preserve module-edge equivalence and fallback discipline.                                         | Fake imports in comments, strings, regexes, JSX text, or templates must not become edges.                                                                       |
| `tests/test-symbol-graph-incremental.mjs`                   | strict incremental cache      | Warm symbol graph runs reuse unchanged file facts while changed/deleted files refresh correctly.                                   | Legacy caches missing newer facts must invalidate rather than silently reuse stale symbol evidence.                                                             |
| `tests/test-shape-index-incremental.mjs`                    | strict incremental cache      | Warm shape-index runs match cold facts and handle changed/deleted files.                                                           | Deleted or changed files must not leave stale shape facts.                                                                                                      |
| `tests/test-function-clone-incremental.mjs`                 | strict incremental cache      | Warm function-clone runs match cold facts and handle changed/deleted files.                                                        | Function clone caches must not preserve stale fingerprints after edits or deletes.                                                                              |
| `tests/test-any-inventory-incremental.mjs`                  | strict incremental cache      | Any-inventory cold/warm equivalence holds across changed, deleted, and scan-option changes.                                        | Production/test scope changes must prevent stale public artifacts.                                                                                              |
| `tests/test-incremental-cache-store.mjs`                    | cache store contract          | Incremental cache schema, current-hash reuse, and malformed-cache fallback are stable.                                             | Malformed cache files must degrade safely instead of crashing or reusing invalid facts.                                                                         |
| `tests/test-incremental-snapshot.mjs`                       | repo snapshot identity        | Content hashes, unreadable file visibility, and snapshot identity stay deterministic.                                              | Unreadable or changed files must be visible to cache identity, not hidden.                                                                                      |
| `tests/test-incremental.mjs`                                | file hash/stat fast path      | File-hash cache and stat-first-cut fast path preserve correctness.                                                                 | Fast stat checks must not skip content changes that matter to artifacts.                                                                                        |
| `tests/test-audit-repo-symbol-incremental.mjs`              | orchestrator wiring           | Audit orchestrator forwards strict incremental flags to `build-symbol-graph`.                                                      | CLI cache flags must not be ignored by the producer path.                                                                                                       |
| `tests/test-function-clone-audit-forwarding.mjs`            | orchestrator wiring           | Audit orchestrator forwards incremental flags to the function-clone producer.                                                      | Function-clone cache behavior must be controlled by the same public audit flags.                                                                                |
| `tests/test-post-write-incremental.mjs`                     | lifecycle cache boundary      | Post-write after-snapshot incremental forwarding preserves immutable pre-write baseline semantics.                                 | Post-write must not mutate or reinterpret the pre-write baseline cache.                                                                                         |
| `tests/test-build-function-clone-index.mjs`                 | producer artifact shape       | Function clone producer emits clone cue artifacts and exact grouping.                                                              | Small exact clones with identical hashes must group without widening structure-group thresholds.                                                                |
| `tests/test-build-shape-index.mjs`                          | producer artifact shape       | Shape-index producer emits grouping, diagnostics, and scan-scope metadata.                                                         | Unsupported or out-of-scope shapes must stay diagnostic, not fake comparable facts.                                                                             |
| `tests/test-tsconfig-paths-scoped.mjs`                      | resolver cache counters       | Scoped tsconfig/baseUrl resolution emits cache/probe counters while preserving scoped correctness.                                 | Cache hits must not change per-importer resolution identity.                                                                                                    |
| `tests/test-wildcard.mjs`                                   | resolver cache overlap        | Exports wildcard subpath resolution remains deterministic after wildcard alias caching.                                            | Cached wildcard results must preserve unresolved/internal sentinels and generated virtual surfaces.                                                             |
| `docs/spec/lumin-fused-safer-graph.md`                      | performance architecture spec | Parser avoidance, scanner fallback, edge hashes, and fact fusion stay measurement-gated.                                           | Scanner/fusion work should not land as broad behavior change without equivalence fixtures and counters.                                                         |
| `docs/spec/block-clone-detection.md`                        | block clone architecture spec | Suffix-array/LCP block clone work stays separate from `function-clones.json` and review-only.                                      | Token/block repeated regions must not leak into top-level function clone, SAFE, fix-plan, or pre-write cue lanes.                                               |
| `docs/lab/wt09-block-clone-fixture-inventory-2026-05-24.md` | block clone fixture inventory | P1 block clone work started from named edge-case fixtures before runtime behavior.                                                 | A clone detector that only catches happy-path top-level functions is not enough; sentinel, overlap, subset, generated-file, destructuring, and leakage guards must be covered. |
| `docs/lab/wt09-beta59-block-clone-manifest-verification-2026-05-24.md` | beta.59 block clone manifest verification | Public install P2 verification used non-empty runtime artifact data. | Raw groups and instances stayed compressed into manifest counts, and Markdown remained unchanged. |
| `docs/lab/wt09-beta59-block-clone-noise-review-2026-05-24.md` | beta.59 block clone noise review | The first self-dogfood corpus had engine signal but was dominated by test mirror/scaffold groups. | Default Markdown should wait for a named noise/mute policy; the run saturated `maxGroups: 100`. |
| `docs/spec/block-clone-detection.md#noise-and-mute-policy`  | block clone noise policy      | `block-clone-noise-policy-v1` classifies raw groups before P3 rendering.                                                           | Raw groups stay auditable; only shallow review/muted counts and reason totals reach navigation surfaces.                                                         |
| `tests/test-build-block-clone-index.mjs`                    | block clone producer artifact | `block-clones.json` is review-only repeated normalized token-region evidence.                                                      | BC1-BC11 keep nested renamed blocks, destructuring, object-pattern keys, overlap/subset handling, generated/parse limitations, and no action-lane leakage pinned. |
| `tests/build-block-clone-index.test.mjs`                    | block clone Vitest mirror     | The reviewed block-clone producer contract has a focused Vitest mirror.                                                            | The mirror protects the same P1 boundaries without replacing the Node entrypoint.                                                                                |
| `tests/test-audit-manifest-export-surface.mjs`              | block clone manifest mirror   | `manifest.blockClones` exposes shallow status/count/policy metadata only.                                                          | Raw `groups[]`, `instances[]`, and source spans must stay in `block-clones.json`, not the manifest.                                                              |
| `tests/audit-manifest-export-surface.test.mjs`              | block clone manifest mirror   | The Vitest mirror pins the same shallow manifest contract.                                                                          | Review/navigation metadata must not become source-fragment evidence.                                                                                             |
| `docs/lab/wt18-*.md`                                        | lab evidence                  | cal.diy and public-package timing notes preserve measured before/after evidence.                                                   | Single-run wall time is lab evidence, not a universal performance claim.                                                                                        |

## Reform Direction

Performance tests should avoid bare wall-time claims when possible. Prefer:

- parser call counts
- scanner accepted/fallback counts
- resolver attempts/hits/misses
- artifact read bytes and parse time
- median multi-run lab evidence for wall time

Further performance work should start with a focused probe-target directory
cache spec if scopedBaseUrl unique-miss cost remains a blocker. Clone-detection
work should keep extending the separate block-clone artifact rather than
mutating the existing function-clone artifact. The current manifest mirror is
intentionally shallow and has beta.59 public-install verification. Any Markdown
or recommendation surface should wait for another corpus check using
`block-clone-noise-policy-v1`, especially because the first self-dogfood run
saturated `maxGroups: 100`.

## Reform Targets

- Keep performance tests centered on counters and cache identity, not bare
  single-run wall time.
- Separate correctness overlap from performance overlap: resolver cache tests
  also belong to resolver correctness, but performance inventory tracks the
  counter/cache invariant.
- Compare incremental suites for shared cold/warm fixture helpers before moving
  files.
- Keep scanner tests in shadow/equivalence style before expanding accepted
  syntax.
- Do not start scheduler, Rust/rayon, or full `js-facts` fusion work without a
  narrower measurement spec.

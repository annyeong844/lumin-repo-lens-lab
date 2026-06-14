# Function Clone Incremental Cache Design

Date: 2026-05-04
Status: design approved for implementation planning

## Goal

Make `build-function-clone-index.mjs` reuse unchanged per-file function clone
facts under the strict incremental cache contract.

This is the next full-profile cost slice after `any-inventory`,
`post-write`, `build-symbol-graph`, and `build-shape-index`.

## Non-Goals

- Do not make quick profile equivalent to full profile.
- Do not add a daemon, file watcher, or background service.
- Do not create a broad generic producer framework before a second clone-like
  producer proves the abstraction is needed.
- Do not change clone semantics, thresholds, or review wording.
- Do not cache final clone groups as source of truth.

## YAGNI And DRY Boundary

The implementation should follow the proven `shape-index` incremental pattern,
but it should not extract a large shared abstraction yet.

Allowed reuse:

- Use the existing incremental modules:
  `_lib/incremental-snapshot.mjs` and `_lib/incremental-cache-store.mjs`.
- Mirror the small producer-local structure used by `build-shape-index.mjs`:
  snapshot, per-file cache lookup, per-file extraction, aggregate assembly,
  cache write, and `meta.incremental`.

Avoid for this slice:

- A new generic "incremental producer runner".
- Shared type-heavy producer config objects.
- Changes to shape-index code just to reduce visual duplication.

If a third AST-per-file producer needs the same pattern later, extract the
shared runner then with concrete call sites in hand.

## Design

### Artifact Library Split

`_lib/function-clone-artifact.mjs` should expose per-file extraction and global
assembly separately.

- `extractFunctionCloneFilePayload({ src, relFile, scope })`
  returns only facts, diagnostics, parse errors, and read errors for one file.
- `functionCloneReadErrorPayload(relFile, message)` returns the same diagnostic
  shape for unreadable files.
- `assembleFunctionCloneArtifact({ facts, diagnostics, filesWithParseErrors,
  filesWithReadErrors, observedAt, ...meta })` sorts facts, stamps current-run
  metadata, rebuilds exact groups, structure groups, and near-function
  candidates every run.
- Existing `buildFunctionCloneArtifact(...)` remains as the cold-run wrapper for
  tests and direct callers.

Cached entries must contain only per-file payloads. Exact groups, structure
groups, and near-function candidates are global results and must always be
rebuilt from the current aggregate.

### Run-Scoped Metadata

Per-file cached payloads must not contain run-scoped metadata such as
`observedAt`, output paths, invocation ids, or incremental counters. The cache
stores only reusable source-derived facts and diagnostics tied to strict file
identity.

`assembleFunctionCloneArtifact(...)` stamps the current run's `observedAt` onto
fresh and reused facts before writing the public artifact. Cache reuse must not
leak stale run timestamps into public facts or derived review cues.

### Deterministic Candidate Assembly

Global assembly must be deterministic after mixing fresh and reused payloads.
Cache insertion order, filesystem enumeration order, and fresh-vs-reused
processing order must not affect emitted groups, candidate selection, or
candidate ordering.

Sorting and pruning should use stable tie-breakers:

- facts: owner file, range/line, exported name
- exact groups: signature hash and sorted member identities
- structure groups: signature hash and sorted member identities
- near candidates: score, sorted member identities, then deterministic pair id

Near-candidate pruning must use the same deterministic tie-breakers so cold and
warm runs emit the same top candidates.

### Producer Incremental Flow

`build-function-clone-index.mjs` should accept:

- `--no-incremental`
- `--cache-root`
- `--clear-incremental-cache`

The producer should use strict content-hash identity only. A warm cache hit
requires the same producer id/version, fact schema version, parser identity,
scan fingerprint, config fingerprint, normalized relative path, language,
test/prod classification, package scope, and content hash through the existing
cache store.

The function clone context fingerprint must cover clone-semantics inputs:
normalization version, exact and structure signature algorithm version,
near-candidate scoring/pruning version, clone thresholds, parser mode, scan
options, package scope, and test/prod classification. If any of these change
and affected files cannot be narrowed safely, the producer cache must be
invalidated.

The producer should report `meta.incremental` with:

- `enabled`
- `identityMode`
- `cacheVersion`
- `cacheRoot`
- `changedFiles`
- `reusedFiles`
- `droppedFiles`
- `invalidatedFiles`
- `reason`

Reused facts must be stamped with the current artifact `observedAt` during
assembly, matching the shape-index fix. Cache reuse must not leak stale run
timestamps into public artifacts.

Incremental counters use these meanings:

- `reusedFiles`: current snapshot files with accepted strict cache hits.
- `changedFiles`: current snapshot files freshly extracted because no valid
  entry existed, content changed, or the file is new.
- `invalidatedFiles`: current snapshot files with a prior entry rejected because
  producer/schema/parser/context identity changed.
- `droppedFiles`: prior cached paths not present in the current scan snapshot.

A current file is either reused or freshly extracted. `invalidatedFiles` is a
reason subset of freshly extracted `changedFiles`, matching the existing
shape-index counter semantics. Dropped files are counted separately because
they are not current snapshot files.

### Audit Pipeline

`audit-repo.mjs` should forward incremental flags to
`build-function-clone-index.mjs` by adding the producer to the existing
incremental producer set.

`--clear-incremental-cache` is orchestration-scoped. When the public audit CLI
receives it, the shared cache root should be cleared once before supported
incremental producers run. The orchestrator should not forward a whole-store
clear flag to each producer, because later producers could delete cache files
written earlier in the same audit.

No newly supported producer beyond `build-function-clone-index.mjs` should be
added in this slice.

### Skill Mirror

After source changes pass focused tests, run the existing skill build step so
the shipping mirror under `skills/lumin-repo-lens-lab/_engine` receives the same
producer changes.

## Error Handling

- Read errors stay visible in the artifact and invalidate prior clean facts.
- Read-error payloads without a current content hash must not become reusable
  clean cache entries.
- Parse errors stay visible in diagnostics and completeness metadata.
- Malformed or incompatible cache entries are ignored by the shared cache store
  and force recomputation.
- Disabled incremental mode produces a fresh artifact with
  `meta.incremental.enabled === false`.

## Testing

Add `tests/test-function-clone-incremental.mjs`.

Required coverage:

- Cold run writes `function-clones.json`.
- Warm run equals cold public clone facts.
- Warm run reports strict incremental enabled.
- Warm run reuses unchanged files.
- Reused facts are stamped with the current artifact `observedAt`.
- Changed file refreshes its clone facts and keeps unchanged facts reusable.
- Deleted file facts disappear and increase dropped count.
- Changed files do not count as dropped. Dropped means a prior cached path is
  absent from the current scan snapshot.
- Exact groups, structure groups, and near candidates are rebuilt correctly from
  mixed fresh and reused facts.
- `--no-incremental` reports disabled cache.
- `--clear-incremental-cache` clears the function clone cache.
- Deterministic near-candidate tie-breakers produce identical cold/warm output.
- Moved files with identical content are not reused unless strict relative path
  identity matches.
- `audit-repo.mjs` forwards `--no-incremental` and `--cache-root` to the
  function clone producer, and handles cache clearing once at the orchestrator
  level.

The cold/warm comparison should cover all review-relevant public outputs:
per-file facts, exact groups, structure groups, near-function candidates,
diagnostics, parse/read error metadata, completeness metadata, and counts.
Expected run metadata and `meta.incremental` may differ.

## Performance Boundary

This slice avoids reparsing and re-normalizing unchanged files. It does not
promise that clone grouping or near-candidate rebuild becomes sublinear.

Warm cost is still expected to include file walk, strict identity refresh,
changed-file extraction, aggregate assembly, exact group rebuild, structure
group rebuild, and near-candidate rebuild.

If warm runs remain expensive after this slice, measure extraction time
separately from global assembly/grouping time before adding more caching.

Focused regression:

- `node tests/test-function-clone-incremental.mjs`
- `node tests/test-build-function-clone-index.mjs`
- `node tests/test-shape-index-incremental.mjs`
- `node tests/test-audit-repo.mjs`
- `npm run check:test-doc`

Final verification:

- `npm run ci`

## Acceptance Criteria

- Public cold and warm `function-clones.json` artifacts are equivalent for all
  review-relevant facts, groups, candidates, diagnostics, completeness metadata,
  and counts, except for expected run metadata and `meta.incremental`.
- Cache hits never reuse stale `observedAt` values.
- Function clone groups remain review cues only; no semantic equivalence claim
  changes.
- The implementation does not introduce a broad generic incremental runner.
- Full CI passes locally before PR.

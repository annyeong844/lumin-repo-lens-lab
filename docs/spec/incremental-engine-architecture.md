# Incremental Engine Architecture

> **Role:** maintainer-facing architecture spec for making Lumin Repo Lens
> usable on large repositories without weakening scan correctness.
> **Status:** design draft, implementation deferred.
> **Last updated:** 2026-05-04
> **Implementation plan:** P0/P1 execution plan lives at
> [`docs/superpowers/plans/2026-05-04-incremental-engine-p0-p1.md`](../superpowers/plans/2026-05-04-incremental-engine-p0-p1.md).

---

## 1. Problem

Full profile carries the tool's strongest evidence: shape index, function-clone
cues, public-surface policy, call graph, topology, barrel discipline, and
post-write deltas. On large repositories, however, a full run can take minutes.
That is acceptable for a branch baseline or due-diligence pass, but not for an
agent loop after every small edit.

The current engine already has producer-local caches for some paths, but the
architecture is not yet a shared incremental substrate. If each feature adds its
own cache, the engine will accumulate incompatible invalidation rules and
eventually lose trust.

The target is stricter:

```text
Keep the scan range and evidence contract intact.
Reuse only facts whose inputs are proven unchanged.
Make every cache hit explainable and invalidatable.
```

Incremental mode is not a license to skip files. It is a way to avoid reparsing
unchanged files under the same scan contract.

## 2. Goals

- Make full-profile evidence practical in large TS/JS repositories.
- Make post-write viable inside an agent loop by paying changed-file cost
  instead of full after-snapshot parse cost.
- Provide one shared snapshot/cache contract for producers instead of
  feature-specific ad hoc caches.
- Preserve exact scan-scope behavior: includes, excludes, test/prod mode,
  generated/compiled policies, language filters, and root identity still apply.
- Make stale cache use impossible when parser version, producer version, scan
  options, package/tsconfig context, or source content changes.
- Keep artifacts honest: every producer that reuses cached facts must report
  changed, reused, dropped, and invalidated counts.
- Keep cache state private and disposable. Cache files may contain
  source-derived structure, symbol names, hashes, normalized bodies, import
  graphs, and parse diagnostics.

## 3. Non-goals

- Do not make `quick` equivalent to `full`.
- Do not hide full-profile cost by silently omitting expensive producers.
- Do not ignore compiled, generated, dist, or framework files because they are
  expensive. File inclusion remains scan-scope policy, not cache policy.
- Do not require git history or a clean working tree.
- Do not implement file watching or a daemon in v1.
- Do not apply source edits as part of this architecture. Post-write uses the
  cache only to compute evidence faster.

## 4. Core Principle

```text
Incremental correctness is a cache-key problem, not a ranking problem.
```

Ranking and review logic must not know whether a fact was freshly extracted or
reused from cache. It should only know whether the producer's artifact is
complete and within scan range.

The cache layer answers:

```text
Can this exact per-file fact packet be reused for this exact producer under
this exact scan context?
```

If the answer is not clearly yes, the producer recomputes.

## 5. Architecture

The shared incremental engine has four layers:

1. **Repo snapshot**: one normalized file list and file identity table for a
   scan invocation.
2. **Context fingerprint**: scan options and config inputs that affect producer
   output.
3. **Producer fact cache**: per-file producer payloads keyed by file identity,
   producer id, producer version, and context fingerprint.
4. **Artifact assembly**: producers combine fresh and reused per-file facts into
   the same public artifact shape they already write today.

```text
collect files
  -> repo snapshot
  -> producer asks for reusable per-file facts
  -> changed files are reprocessed
  -> unchanged files reuse cached facts
  -> public artifact is assembled normally
```

The public JSON artifact is still the source of truth for downstream steps. The
cache is only an implementation detail unless the user asks for debug detail.

## 6. Repo Snapshot

The snapshot records the exact files included in one scan:

```json
{
  "schemaVersion": 1,
  "root": "C:/repo",
  "scanOptions": {
    "includeTests": true,
    "exclude": [],
    "languages": ["ts", "tsx", "js", "jsx", "mjs", "cjs", "mts", "cts"]
  },
  "files": {
    "src/a.ts": {
      "absPath": "C:/repo/src/a.ts",
      "language": "ts",
      "isTestLike": false,
      "readable": true,
      "mtimeMs": 1770000000000,
      "size": 1234,
      "hash": "sha256:..."
    },
    "src/secret.ts": {
      "absPath": "C:/repo/src/secret.ts",
      "language": "ts",
      "isTestLike": false,
      "readable": false,
      "mtimeMs": null,
      "size": null,
      "hash": null,
      "readError": {
        "kind": "permission-denied"
      }
    }
  },
  "droppedSincePrevious": [
    {
      "path": "src/old.ts",
      "reason": "deleted"
    }
  ]
}
```

Snapshot collection must not change scan scope. If a file would be included by
`collectFiles()` today, it remains included.

Dropped-file reasons must distinguish at least:

- `deleted`
- `out_of_scope`
- `language_changed`
- `unreadable`
- `scan_option_changed`

These cases are not equivalent. A deleted file can remove facts; an out-of-scope
file means the current scan intentionally stopped observing it.

Unreadable files that remain in scan scope must remain visible in the current
snapshot with read-error state. They invalidate prior readable facts, but they
must not silently disappear from scan-range reporting. Confidence reporting
should be able to distinguish deleted, out-of-scope, and currently unreadable
files.

### 6.1 Per-file identity

Reusable producer facts must not be keyed by content hash alone. The normalized
relative path is part of the identity unless a producer explicitly proves
path-independence.

Minimum per-file identity:

```json
{
  "relPath": "src/a.ts",
  "language": "ts",
  "isTestLike": false,
  "packageScope": "packages/core",
  "contentHash": "sha256:...",
  "contextFingerprint": "sha256:..."
}
```

The same bytes can mean different things under `src/`, `test/`, `pages/`,
`app/routes/`, or another workspace package. Path, language, test/prod
classification, and package scope are therefore cache-key material.

### 6.2 Strict cache hits

A strict incremental cache hit requires equality of:

- producer id,
- producer version,
- fact schema version,
- parser identity,
- scan fingerprint,
- relevant config fingerprint,
- normalized relative path,
- language classification,
- test/prod classification,
- package scope,
- content hash.

Stat fingerprints may be used as a prefilter to avoid unnecessary hashing or
parsing only when they prove that a file is definitely changed. In strict mode,
they may not confirm an unchanged file, skip hashing, or produce a cache hit
without a content hash computed from the current file bytes.

If the engine ever introduces a non-strict identity mode that trusts
`mtimeMs + size`, it must report that mode in `meta.incremental.identityMode`
and must not use it for strict cold/warm artifact-equivalence claims. That mode
is a local speed experiment, not a public correctness claim.

## 7. Context Fingerprint

Producer output can depend on more than file bytes. The shared cache key must
include:

- producer id and producer version,
- parser package/version or parser feature version,
- scan options: include tests, excludes, language filters, profile inputs,
- relevant config files: `package.json`, `tsconfig*.json`, framework config,
  and alias/package resolution inputs when the producer depends on them,
- engine schema versions for emitted fact shapes,
- root identity and path normalization version.

Example:

```json
{
  "producer": "any-inventory",
  "producerVersion": 2,
  "parser": { "name": "oxc-parser", "version": "..." },
  "scanFingerprint": "sha256:...",
  "configFingerprint": "sha256:...",
  "factSchemaVersion": 1
}
```

When a context fingerprint changes, the producer must recompute affected facts.
If the engine cannot determine which files are affected by a config change, it
must invalidate the producer cache for the scan.

Resolver-aware producers must fingerprint the complete package and module
resolution context, including:

- lockfiles: `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `bun.lockb`,
- workspace manifests: `pnpm-workspace.yaml`, `turbo.json`, `nx.json`,
  `lerna.json`,
- `tsconfig` / `jsconfig` extends chains,
- known framework/build configs when they affect aliases, entrypoints, or
  generated-source conventions: `vite.config.*`, `next.config.*`,
  `nuxt.config.*`, `astro.config.*`, `svelte.config.*`.

The exact file set is producer-dependent. The invariant is that resolver output
must not be reused across resolver-affecting config changes.

Root and path identity must include case-sensitivity and symlink-resolution
policy. A cache produced under one path policy must not be reused under an
incompatible policy. `repoFingerprint` should be derived from normalized root
realpath, workspace/package markers, and git worktree identity when available;
it must not rely on display paths alone.

## 8. Producer Contract

Every incremental producer must expose the same high-level behavior:

```ts
runProducer({
  root,
  output,
  snapshot,
  cacheStore,
  contextFingerprint
}) -> artifact
```

Producer rules:

- Per-file extraction functions must be deterministic for the same source text
  and context fingerprint.
- Reused per-file facts must be copied into the assembled artifact exactly as
  fresh facts would be.
- Dropped files must remove their cached facts.
- Parse errors may be cached when file bytes were read and the content hash
  matches. Read errors without a content hash must not become clean cache hits;
  they may be reported as repeated run-level errors, but they do not prove the
  file's facts unchanged.
- Producer artifacts must expose `meta.incremental`:

```json
{
  "incremental": {
    "enabled": true,
    "cacheVersion": 1,
    "changedFiles": 3,
    "reusedFiles": 997,
    "droppedFiles": 1,
    "invalidatedFiles": 0,
    "reason": null
  }
}
```

If incremental mode is disabled or unavailable, `meta.incremental.enabled` must
be `false` with a reason.

## 9. Deterministic Artifact Assembly

Incremental assembly must produce the same public artifact shape and ordering as
a cold run. Merged fresh and reused facts must be sorted by deterministic keys:

- normalized relative path,
- symbol id or definition id,
- range start/end,
- export name,
- deterministic group id.

Cache insertion order, filesystem enumeration order, and changed-file processing
order must not affect public artifact order.

Shape and clone group ids must be derived from deterministic group content, not
from discovery order.

## 10. Cache Store

The incremental cache should live under a stable audit cache root, not inside a
per-run artifact directory. Public artifacts remain run-specific outputs; cache
state is shared only across compatible invocations for the same repo identity
and context fingerprint.

Cache entries should prefer normalized relative paths plus `repoFingerprint`.
Absolute paths may appear in debug-only metadata or run artifacts, but they
should not be required for portable cache identity unless explicitly documented.

Recommended layout:

```text
<audit-cache-root>/
  incremental/
    <repoFingerprint>/
      repo-snapshot.cache.json
      any-inventory.cache.json
      symbols.cache.json
      shape-index.cache.json
      function-clones.cache.json
```

Default cache root may be `<root>/.audit/.cache` when `--output` is the default
audit directory. If callers use per-run outputs such as `.audit/runs/<run-id>/`,
they should keep cache state under a stable sibling such as `.audit/.cache`.

CLI design should include:

- `--no-incremental`
- `--clear-incremental-cache`
- `--cache-root <path>`

Existing producer-local caches such as `topology.cache.json` and
`symbols.cache.json` can be migrated into this layout over time. During
migration, producers may continue reading legacy caches, but new producers
should use the shared cache store.

Legacy caches may be read only through compatibility adapters that enforce the
shared cache contract. A legacy entry must not become a strict hit unless it can
be validated against the current producer id, schema version, parser identity,
scan fingerprint, config fingerprint, normalized path identity, package scope,
and content hash.

The cache must not be committed by default.

## 11. Cache Safety And Privacy

Incremental caches are untrusted, source-derived audit artifacts. They may
contain repository structure, symbol names, normalized function bodies, import
graphs, type escape facts, parse diagnostics, and hashes. Treat them as private
audit artifacts. They should not be committed, uploaded, or shared unless the
corresponding audit artifacts are safe to share.

Implementation requirements:

- schema-validate cache entries before use,
- ignore unknown, incompatible, malformed, or oversized entries,
- write cache files atomically via temp-file plus rename,
- never execute cache contents,
- tolerate concurrent scans by degrading to recomputation when cache state is
  partial or incompatible.

Concurrent scans may race, but a race must not produce malformed public
artifacts. At worst, one invocation should ignore a partial cache and recompute.

## 12. Invalidation Rules

Hard invalidation triggers:

- producer version change,
- fact schema version change,
- parser version or parser mode change,
- scan options change,
- root changes,
- relevant config fingerprint change when affected files cannot be narrowed,
- source hash change,
- unreadable file after previous readable file,
- file dropped from scan range.

Soft invalidation triggers:

- config changed but affected files can be narrowed,
- package boundary changed for one workspace package,
- tsconfig path alias changed for one package scope,
- framework sentinel changed for one submodule.

Soft invalidation may recompute only affected submodules or files. If narrowing
is uncertain, promote the soft invalidation to hard invalidation for that
producer.

## 13. Producer Rollout Order

### P0: Shared Snapshot And Cache Store

- Add snapshot builder around current `collectFiles()` behavior.
- Add cache store read/write helpers.
- Add common `meta.incremental` shape.
- Keep all producers behavior-identical when incremental is disabled.

### P1: Any Inventory Adapter

`any-inventory` is the first producer because post-write depends on it and its
facts are per-file.

Expected behavior:

- unchanged files reuse prior `typeEscapes` and parse status,
- changed/new files re-run `extractTypeEscapes`,
- dropped files disappear from the artifact,
- post-write after-snapshot uses incremental mode by default,
- `--no-incremental` forces a cold after-snapshot.

This is the first vertical slice, but it is not a post-write-only cache.

### P2: Symbol Graph Adapter

Symbols are the central graph substrate. This phase must be conservative:

- per-file definitions/imports/re-exports can be cached,
- resolver inputs and tsconfig/package fingerprints must invalidate affected
  package scopes,
- unresolved internal reporting must remain run-level and scope-aware.

### P3: Shape Index And Function Clone Adapters

Shape and clone evidence are full-profile value drivers.

- Shape index can reuse per-file extracted shape facts and regroup globally.
- Function clone can reuse per-file normalized function bodies and rebuild
  cross-file groups globally.
- Grouping remains global because unchanged files can still match changed files.

### P4: Topology, Call Graph, And Reachability

These producers can cache per-file edges, but final graph metrics must be
rebuilt globally from the current edge set:

- SCCs,
- fan-in/fan-out,
- top hub files,
- call fan-in maps,
- module reachability.

Do not cache final graph conclusions without validating the edge set.

### P5: Calibration And Default Policy

After P1 through P4:

- measure self repo, ESLint, cal.com/cal.diy, Next, Nuxt, Hono, SvelteKit,
  Astro, and NestJS fixtures where available,
- report cold vs warm runtime,
- report cache hit rate,
- report any artifact count drift between cold and warm runs,
- only then decide which profiles enable incremental by default.

## 14. Post-Write Contract

Post-write should become the first user-visible beneficiary:

```text
pre-write snapshot -> code edit -> post-write incremental after-snapshot
```

Correctness requirements:

- post-write must compare against the invocation-specific
  `preWrite.anyInventoryPath`,
- the cache may accelerate after-snapshot construction, but it must never
  replace or mutate the invocation-specific pre-write baseline,
- baseline artifacts are immutable inputs to post-write comparison,
- after-snapshot may be incremental, but the resulting `any-inventory.json`
  must have the same public shape as a cold run,
- scan range parity must still be checked,
- cache failure must degrade to fresh scan or confidence-limited output, never
  to a false clean result,
- strict post-write modes must still escalate confidence limitations.

Performance target for large repos:

```text
Warm post-write cost ~= file walk + strict file identity refresh
                       + changed-file extraction + artifact assembly.
```

It is acceptable for full profile to remain a branch-level operation while
post-write becomes loop-safe.

## 15. Correctness Gates

Every incremental producer needs these tests:

- cold run and warm run produce identical public artifact for unchanged repo,
- one changed file updates only that file's facts and preserves all others,
- one dropped file removes its facts,
- scan option change invalidates the relevant cache,
- parser/producer version change invalidates the cache,
- malformed cache falls back without crashing,
- parse-error facts do not become false clean facts,
- read errors without content hashes do not become clean cache hits,
- generated/compiled files are neither silently dropped nor newly included by
  cache behavior,
- `--no-incremental` or equivalent produces a fresh artifact.

Full profile needs an end-to-end gate:

```text
cold full artifact set == warm full artifact set
for all public claims and counts, except for meta.incremental diagnostics.
```

## 16. Metrics

Report these separately:

- cold runtime,
- warm runtime with no source changes,
- warm runtime with one small edit,
- warm runtime with one package-level config edit,
- cache hit rate per producer,
- artifact count drift between cold and warm,
- cache invalidation reason distribution.

Do not market sublinear behavior alone. Absolute wall time matters for CI and
agent loops.

## 17. Open Questions

- Should the repo snapshot be a standalone producer artifact or an internal
  cache-only file?
- Should `--incremental` be default for `full` immediately after P1, or only for
  post-write until P2-P4 are calibrated?
- What cache retention policy is needed for long-lived branches?
- Should config fingerprint narrowing be package-root based first, then
  dependency-cone based later?
- Should strict content-hash equality be the only supported default forever, or
  should a clearly labeled non-strict identity mode exist for local-only speed
  experiments?

Initial recommendation:

- post-write uses incremental by default after P1,
- full profile exposes `--incremental` as opt-in until P2-P4 calibration proves
  cold/warm artifact equivalence across representative repositories,
- `--no-incremental` remains available wherever incremental can run,
- strict content-hash equality remains the default for public behavior.

## 18. Acceptance Criteria

This architecture is ready for implementation planning when:

- this spec is committed under `docs/spec/`,
- `docs/spec/README.md` links it as a maintainer-facing architecture spec,
- the first implementation plan names P0/P1 as the initial PR slice,
- P1 tests prove cold/warm `any-inventory` equivalence,
- post-write tests prove incremental after-snapshot still honors baseline,
  scan-range parity, and strict confidence gates.

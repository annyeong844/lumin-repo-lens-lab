# Rust Module Reachability Producer Design

## Goal

Move `module-reachability.json` artifact construction from JS into
`lumin-audit-core` while keeping `build-module-reachability.mjs` as the
compatibility wrapper that reads already-produced artifacts and writes the
final file.

This is a deliberately narrow slice. Rust must not take over JS/TS module
resolution, entry-surface discovery, source walking, package interpretation, or
language parsing. It only consumes the already-produced `symbols.json` and
`entry-surface.json` facts and projects the deterministic reachability artifact.

## Why This Slice Is Safe Now

`module-reachability` used to sit in the broader JS/TS resolver bucket because
it depends on JS/TS graph facts. The current implementation does not perform
resolution itself. It consumes:

- `symbols.json.defIndex`
- `symbols.json.reExportsByFile`
- `symbols.json.resolvedInternalEdges`
- `entry-surface.json.entryFiles`
- `entry-surface.json.globalCompleteness`
- `entry-surface.json.completenessBySubmodule`

That means the Rust migration can preserve the JS/TS producer boundary: JS still
owns the graph and entry facts; Rust owns only the deterministic graph walk and
artifact-local summary.

## Current Owner Split

Current JS ownership:

- `build-module-reachability.mjs` reads `symbols.json` and
  `entry-surface.json`, parses optional max-visit flags, calls
  `_lib/module-reachability.mjs`, writes `module-reachability.json`, and prints
  the console summary.
- `_lib/module-reachability.mjs` owns:
  - known-file collection from symbols and entry-surface facts
  - runtime-only adjacency
  - runtime+type adjacency
  - bounded BFS for runtime and type reachability
  - `boundedOutFiles` vs `unreachableFiles`
  - entry-unreachable runtime SCC detection
  - deterministic sorting and summary projection

Target ownership:

- Rust owns artifact construction and reachability projection semantics.
- JS keeps file reads, file writes, CLI compatibility, console summary, and
  producer sequencing.

## Rust CLI

Add:

```text
lumin-audit-core module-reachability-artifact --input <path|-> [--result-output <path>]
```

The command accepts:

```json
{
  "schemaVersion": "lumin-module-reachability-producer-request.v1",
  "root": "C:/repo",
  "symbols": {
    "defIndex": {},
    "reExportsByFile": {},
    "resolvedInternalEdges": []
  },
  "entrySurface": {
    "entryFiles": [],
    "globalCompleteness": "low",
    "completenessBySubmodule": {}
  },
  "maxFilesVisited": 200000,
  "maxEdgesVisited": 400000
}
```

The response is the existing artifact shape:

```json
{
  "meta": {
    "tool": "build-module-reachability.mjs",
    "schemaVersion": "module-reachability.v1",
    "mode": "full-bfs",
    "entrySurfaceFile": "entry-surface.json",
    "globalCompleteness": "low",
    "completenessBySubmodule": {},
    "maxFilesVisited": 200000,
    "maxEdgesVisited": 400000,
    "boundedOutReason": null,
    "supports": {
      "runtimeReachableFiles": true,
      "typeReachableFiles": true,
      "boundedOutFiles": true,
      "unreachableStronglyConnectedComponents": true
    }
  },
  "runtimeReachableFiles": [],
  "typeReachableFiles": [],
  "reachableFiles": [],
  "boundedOutFiles": [],
  "unreachableFiles": [],
  "unreachableStronglyConnectedComponents": [],
  "summary": {}
}
```

Direct stdout remains supported for debugging. JS wrappers must use
`--result-output` for normal repository runs to avoid stdout buffering failure
on large artifacts.

## Existing Limit Preservation

The existing JS producer already exposes visit limits:

- default `maxFilesVisited = 200000`
- default `maxEdgesVisited = 400000`
- optional CLI overrides `--max-files-visited` and `--max-edges-visited`

This migration must preserve those limits exactly. It must not introduce new
elapsed-time caps, repository-size caps, sampling, or hidden truncation. When an
existing limit is hit, the artifact must preserve the current bounded evidence:

- `meta.boundedOutReason` is `max-files-visited` or `max-edges-visited`
- files not reached after the bound are placed in `boundedOutFiles`
- those files are not claimed as `unreachableFiles`
- unreachable SCC evidence is omitted when traversal is bounded

## Bounded Traversal Parity

Bounded traversal must preserve the checked JS visit order and counter
semantics. Output sorting alone is not enough for parity because traversal order
can change the visited set before a bound is hit.

The Rust implementation must match the current JS helper:

- entry seeds come from `entrySurface.entryFiles` after backslash-to-slash
  normalization and `Set` deduplication; their insertion order is preserved, not
  lexicographically sorted before BFS
- adjacency targets are deduplicated and sorted before BFS
- duplicate edges do not increase `maxEdgesVisited` pressure after adjacency
  deduplication, matching the current `buildAdjacency` behavior
- during seed insertion, `maxFilesVisited` is checked before adding the next
  seed
- during adjacency traversal, `edgesVisited` increments before the edge-limit
  check; the limit is hit when `edgesVisited > maxEdgesVisited`
- for a new reachable target during adjacency traversal, `maxFilesVisited` is
  checked before adding that target
- if both limits could apply while processing an adjacency target, the edge
  limit has precedence because JS checks it before the file limit in that loop

## Shared Rust Producer CLI Contract

Producer commands must accept `--input <path|->` and may write either to stdout
or to `--result-output <path>`. JS wrappers must use `--result-output` for
normal repository runs.

Rust must write artifact JSON only to the selected result channel. Diagnostics
must go to stderr. Invalid JSON, schema mismatch, invalid request fields, and
failed result-file writes must exit non-zero and must not produce a partial
success artifact.

Wrappers must treat a non-zero exit, missing result file, or malformed result
JSON as producer failure rather than falling back to JS graph logic.

## Input Contract

Rust consumes only the graph facts it needs and ignores unknown fields.

Top-level `symbols` and `entrySurface` request fields must be JSON objects. The
existing JS helper treats missing nested graph fields as empty, so Rust must do
the same for `symbols.defIndex`, `symbols.reExportsByFile`,
`symbols.resolvedInternalEdges`, `entrySurface.entryFiles`,
`entrySurface.globalCompleteness`, and
`entrySurface.completenessBySubmodule`.

Known-file collection must match JS:

- all keys in `symbols.defIndex`
- all keys in `symbols.reExportsByFile`
- every `from` and `to` path in `symbols.resolvedInternalEdges`
- every path in `entrySurface.entryFiles`

Path normalization is intentionally small and parity-focused:

- graph paths are normalized by replacing `\` with `/`
- empty `from` or `to` edge endpoints are ignored
- no absolute-path rejection is added for consumed artifact facts in this
  slice, because the current JS helper does not reject them
- missing edge `typeOnly` is treated as false
- deterministic output sorting, not request sanitization, is the owner contract

## Re-export Facts

`symbols.reExportsByFile` is used only for known-file collection in this slice.
Reachability adjacency comes exclusively from `symbols.resolvedInternalEdges`,
matching the current JS helper. A re-export/barrel fixture should lock this down
so later migrations do not accidentally reinterpret re-export facts as graph
edges without a new parity decision.

## Reachability Contract

Rust must preserve the checked JS semantics:

- runtime graph excludes edges where `typeOnly === true`
- type graph includes type-only edges
- runtime seeds and type seeds both come from `entrySurface.entryFiles`
- `runtimeReachableFiles` is the runtime BFS visited set
- `typeReachableFiles` is the all-edge BFS visited set
- `reachableFiles` is the union of runtime and type reachable sets
- if traversal is unbounded, known files outside `reachableFiles` are
  `unreachableFiles`
- if traversal is bounded, known files outside `reachableFiles` are
  `boundedOutFiles`
- `summary` counts are derived from the emitted arrays/sets, not recomputed by
  the JS wrapper

The `summary` object must preserve the existing keys:

- `runtimeReachable`
- `typeReachable`
- `reachable`
- `boundedOut`
- `unreachable`
- `unreachableStronglyConnectedComponents`
- `unreachableStronglyConnectedFiles`
- `knownFiles`

## SCC Contract

Rust must preserve the existing SCC behavior:

- SCCs are computed only on the runtime graph
- SCCs are omitted when traversal is bounded
- only components with more than one file are emitted
- every file in an emitted component must be in `unreachableFiles`
- emitted component shape stays:

```json
{
  "kind": "entry-unreachable-scc",
  "graph": "runtime",
  "size": 2,
  "files": ["src/a.ts", "src/b.ts"],
  "note": "Files import each other, but none are reachable from the recorded entry surface."
}
```

This SCC evidence is review evidence only. It must not become a safe-delete or
dead-export proof in this slice.

Single-file self-cycles are not emitted as SCC evidence because the checked JS
helper emits only components with more than one file.

## Sorting And Determinism

The Rust artifact must keep JS ordering contracts:

- file arrays sort lexicographically
- adjacency targets are deduplicated and sorted before traversal
- SCC member files sort lexicographically
- SCC list sorts by descending size, then first file after member sorting
- summary values are deterministic primitive counts

## JS Wrapper Changes

`_lib/module-reachability.mjs` becomes a compatibility bridge that calls Rust.
It should keep only exported constants, if needed, and
`buildModuleReachabilityArtifact(request)`.

It must not retain parallel BFS, SCC, bounded traversal, or summary math after
the Rust owner lands.

The JS wrapper must not contain graph traversal, SCC detection, bounded
traversal decisions, or summary count recomputation after the Rust owner lands.

`build-module-reachability.mjs` should keep:

- existing required-artifact reads for `symbols.json` and `entry-surface.json`
- existing positive-integer parsing for max overrides
- artifact write and console summary

The wrapper request is built from already-read JSON; Rust returns the artifact.

## Orchestration Ownership

After the wrapper conversion, `build-module-reachability.mjs` should be marked
`ProducerOwner::Rust` in the audit-core orchestration plan. The step name,
phase name, preconditions, skip reason, and output artifact name remain
unchanged.

## Tests

Rust tests should cover:

- canonical parity against the checked JS output for clean and bounded fixtures
- runtime BFS reaches entry files and runtime transitive dependencies
- runtime BFS excludes type-only edges
- type reachability includes type-only edges
- `reachableFiles` is the runtime/type union
- clean traversal places isolated files in `unreachableFiles`
- bounded traversal places unvisited files in `boundedOutFiles`, not
  `unreachableFiles`
- bounded traversal at the file limit and edge limit
- entry-unreachable runtime SCCs are emitted as review evidence
- bounded traversal suppresses SCC evidence
- single-file self-cycles are not emitted as unreachable SCC evidence
- `reExportsByFile` contributes known files but not adjacency
- backslash paths normalize to slash paths
- absolute-looking artifact paths are preserved rather than rejected
- empty edge endpoints are ignored
- duplicate edges do not affect reachability or edge-limit behavior after
  adjacency deduplication
- entry files absent from symbol maps are still known and reachable seeds
- deterministic sorting
- CLI `--result-output`

Node compatibility tests should keep:

- `build-module-reachability.mjs` writes `module-reachability.json`
- `audit-repo` quick profile still runs the producer
- manifest `artifactsProduced` includes `module-reachability.json`
- audit summary/review pack still render the existing unreachable SCC language

Vitest mirrors may remain as reference coverage during migration, but the
authoritative new behavior should be cargo-owned.

## Non-Goals

This slice must not:

- migrate `entry-surface.json`
- migrate `symbols.json`
- reinterpret module resolution
- add new reachability policies
- add safe-delete or dead-export claims
- change bounded traversal limits
- add timeout or repository-size caps
- change summary/review Markdown rendering

# WT-18 Symbol Finalizer Cache Contract

> Date: 2026-07-10
> Status: implementation boundary
> Owner split: Rust owns `symbols.json`; JS owns the current incremental store

## Measured Problem

On the current 729-file repository corpus, an unchanged warm run reused all 729
per-file facts but still paid for the full finalizer request and artifact write:

| Measurement | Value |
|---|---:|
| compact finalizer request | 669,383 bytes |
| `symbols.json` | 1,685,080 bytes |
| `symbol-graph-artifact-command` | 1,285 ms |
| Rust helper startup on this Windows host | about 0.5 seconds |

Two consecutive unchanged warm artifacts were identical after removing only
`meta.generated`. Their normalized SHA-256 was
`43055d11bc00c7ff191a697bfa5d1a598a5f586945ef3c32d8be44c60c2cb2d6`.
The finalizer was therefore rebuilding the same semantic artifact.

## Decision

Add one strict, single-entry artifact cache for the Rust-produced symbol graph.
The cache reuses exact artifact bytes; it does not cache partial policy results
or create a JS projection path.

The identity includes:

- the full compact finalizer request except `generated`;
- the exact `incremental` block;
- symbol producer, fact schema, and parser identity versions;
- the expected audit-core runtime bridge contract version.
- filesystem signatures for every current audit-core binary candidate, without
  spawning the helper.

The cache manifest includes its schema version, request identity, artifact byte
length, and artifact SHA-256. Every hit verifies all four before reuse.
Artifact and manifest filenames include the request identity so concurrent
writers cannot pair one run's manifest with another run's bytes. Cleanup of an
older identity may cause a cache miss under a race, but cannot create a false
hit.

## Failure Contract

Cache absence, malformed metadata, missing bytes, byte-length mismatch, hash
mismatch, or identity mismatch is a cache miss. The producer records the miss
reason in numeric phase counters and runs the normal Rust finalizer. It never
treats a cache problem as clean evidence and never falls back to JS symbol
classification.

`--no-incremental` bypasses the artifact cache. Clearing the incremental cache
removes both per-file facts and the symbol artifact entry.

## Ownership

Rust remains the sole semantic owner of `symbols.json`. JS may hash the request,
verify the cache envelope, and atomically copy exact Rust-produced bytes because
snapshot and incremental storage are still JS-owned. JS must not parse or patch
the cached JSON. Moving this storage boundary into Rust belongs with the later
snapshot/file-inventory migration, not this finalizer performance slice.

## Acceptance

- cold and first warm runs still invoke `symbol-graph-artifact`;
- a repeated unchanged warm run reports a strict artifact-cache hit and does not
  invoke the Rust finalizer;
- cached and freshly finalized artifacts are byte-identical;
- a source, context, incremental-state, bridge-contract, or cache-byte change
  causes a visible miss and fresh Rust finalization;
- no elapsed-time cap, repository-size cap, or semantic fallback is introduced.

## Implementation Verification

The first warm run after the cold run correctly missed because its exact
`incremental` block differed, then stored a warm artifact. The next unchanged
warm run reported:

| Measurement | Value |
|---|---:|
| `symbolGraphFinalizerCacheHit` | 1 |
| `symbolGraphArtifactRequestBytes` | 0 |
| logical request before cache lookup | 674,145 bytes |
| `symbolGraphFinalizerCacheRestoredBytes` | 1,698,792 |
| `symbol-graph-finalizer-cache-identity` | 18 ms |
| `symbol-graph-finalizer-cache-lookup` | 19 ms |
| `symbol-graph-artifact-command` | 0 ms |
| `write-artifact` | 38 ms |

The restored artifact and the first warm Rust-produced artifact were
byte-identical with SHA-256
`97baf036aa946c5d352c71739b8e5c043cce0b3973a13c332976d3cba43b4688`.

After one byte was appended to the cached artifact, the next run reported
`symbolGraphFinalizerCacheMissCorrupt = 1`, ran the Rust finalizer in 1,248 ms,
and replaced the cache with a valid artifact. Corrupt cache state therefore
cannot become clean symbol evidence.

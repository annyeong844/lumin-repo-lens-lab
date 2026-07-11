# WSL pre-write discovery measurement (2026-07-11)

## Scope

Measured the packaged Linux `lumin-audit-core` against the 564-file maintainer
repository on a WSL `/mnt/c` checkout. Result files and temporary comparison
caches were written under `/tmp` unless the cache-location comparison required
the mounted checkout.

## Observations

| Mode | Elapsed | User CPU | System CPU |
| --- | ---: | ---: | ---: |
| no incremental cache | 11.21s | 1.00s | 1.47s |
| cold strict cache | 9.31s | 1.44s | 2.32s |
| warm strict cache, 564/564 facts reused | 8.67s | 0.18s | 0.65s |

The warm run removed OXC parsing but retained most wall time. Host Git clean
identity cost approximately 0.7s across root, status, and stage queries. The
remaining dominant work is repository filesystem discovery on DrvFS plus final
projection/result serialization.

Host Git visible tracked/untracked inventory matched the checked walker exactly
for this repository (564 files) and completed in 0.34s. It was not selected as
the product fix because Git-only discovery can omit ignored authored source in
other repositories. Enumerating every ignored path took 7.66s and emitted a
3.9 MiB path stream, so adding that query would erase the benefit.

## Decision

Keep the checked filesystem scan semantics. Parallelize independent,
already-sorted directory subtree walks on a local Rayon pool. Worker results
must be merged in input order at each directory and then sorted/deduplicated so
concurrency cannot change artifacts or error selection. Do not add a
repository-size cap, elapsed timeout, Git-only source scope, or stat/mtime
absence claim.

## Result

The checked recursive directory-job implementation was measured with the same
564-file request and a locally built static Linux audit-core:

| Mode | Before | After |
| --- | ---: | ---: |
| no incremental cache, repeated warm filesystem | 7.48-9.03s | 4.06-4.59s |
| cold strict cache | 9.31s | 6.58s |
| warm strict cache, all facts reused | 8.67s | 5.17-5.22s |

The old and new no-cache results had identical `files`, `symbols`, and
`topology` projections. The v45 strict cache also keys clean Git identities by
the actual source path and parses the exact selected Git blob bytes on cache
misses; dirty, untracked, or failed Git-blob reads use a content SHA over the
same worktree bytes sent to OXC.

The regenerated packaged skill was then dogfooded on the same WSL checkout with
`LUMIN_AUDIT_CORE_NO_AUTO_BUILD=1`, proving that no Cargo/source fallback was
used. Lifecycle-only pre-write completed in 6.56s and the paired post-write
completed in 5.01s with `No silent new any`.

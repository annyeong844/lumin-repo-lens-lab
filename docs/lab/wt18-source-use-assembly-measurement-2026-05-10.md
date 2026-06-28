# WT-18 Source-Use Assembly Measurement - 2026-05-10

This note records the first cal.diy full-audit run after
`build-symbol-graph.mjs` started emitting operation-level timings inside the
`assemble-source-uses` bucket.

The goal was to decide whether source-use assembly time was dominated by
resolver calls, consumer insertion, namespace/member propagation, named
re-export propagation, or unresolved diagnostics.

## Run

```powershell
node .\audit-repo.mjs `
  --root "C:\Users\endof\Downloads\cal.diy-main" `
  --profile full `
  --output "$env:TEMP\lumin-wt18-cal-diy-source-use-20260510181051" `
  --cache-root "$env:TEMP\lumin-wt18-cal-diy-source-use-cache-20260510181051" `
  --clear-incremental-cache
```

- Lumin checkout: `1268f98`
- Package version: `0.9.0-beta.37`
- Corpus: `C:\Users\endof\Downloads\cal.diy-main`
- Output:
  `C:\Users\endof\AppData\Local\Temp\lumin-wt18-cal-diy-source-use-20260510181051`
- Cache root:
  `C:\Users\endof\AppData\Local\Temp\lumin-wt18-cal-diy-source-use-cache-20260510181051`
- Profile: `full`
- Cache mode: cold-style run with a fresh cache root and
  `--clear-incremental-cache`

This is a single local run, not a median benchmark. Treat absolute wall time as
local machine evidence, not a regression or speedup claim.

## Producer Summary

`producer-performance.json.summary`:

| Metric | Value |
|---|---:|
| Producers ok | 16 |
| Producers skipped | 3 |
| Total wall time | 413,771 ms |
| Artifact count | 19 |
| Total artifact bytes | 45,074,114 |
| Orchestrator artifact reads | 34 |
| Orchestrator JSON parse time | 728 ms |
| Phase-supporting producers | 3 |

Top producer wall times:

| Producer | Wall time |
|---|---:|
| `build-symbol-graph.mjs` | 145,503 ms |
| `measure-topology.mjs` | 120,033 ms |
| `build-call-graph.mjs` | 63,895 ms |
| `build-function-clone-index.mjs` | 14,489 ms |
| `classify-dead-exports.mjs` | 14,378 ms |
| `checklist-facts.mjs` | 13,667 ms |
| `build-entry-surface.mjs` | 10,804 ms |
| `build-shape-index.mjs` | 10,102 ms |

## Symbol Graph Breakdown

`build-symbol-graph.mjs` wall time was 145,503 ms.

| Phase | Wall time | Share of symbol producer |
|---|---:|---:|
| `snapshot` | 4,924 ms | 3.4% |
| `cache-classification` | 14 ms | 0.0% |
| `extract-changed-files` | 12,186 ms | 8.4% |
| `assemble-symbol-graph` | 124,598 ms | 85.6% |
| `write-artifact` | 155 ms | 0.1% |

## Source-Use Operation Timings

`assemble-source-uses` wall time was 121,293 ms.

| Operation | Wall time | Share of source-use bucket |
|---|---:|---:|
| `assemble-source-use-resolve` | 120,468 ms | 99.3% |
| `assemble-source-use-resolved-internal` | 422 ms | 0.3% |
| `assemble-source-use-external` | 107 ms | 0.1% |
| `assemble-source-use-unresolved` | 96 ms | 0.1% |
| `assemble-source-use-namespace-reexport` | 33 ms | 0.0% |
| `assemble-source-use-generated-virtual` | 11 ms | 0.0% |
| `assemble-source-use-asset` | 0 ms | 0.0% |

The source-use operation sum is within timing overhead of the coarse source-use
phase. The result is decisive: source-use assembly is dominated by resolution,
not by consumer map insertion, namespace propagation, generated virtual handling,
or unresolved diagnostics.

## Source-Use Counters

| Counter | Value |
|---|---:|
| `sourceUseRecordsProcessed` | 67,967 |
| `sourceUseExternalBranchCount` | 23,427 |
| `sourceUseAssetBranchCount` | 218 |
| `sourceUseUnresolvedBranchCount` | 108 |
| `sourceUseGeneratedVirtualBranchCount` | 2,432 |
| `sourceUseNamespaceReExportBranchCount` | 17,433 |
| `sourceUseResolvedInternalBranchCount` | 24,349 |
| `sourceUseSkippedNamespaceAliasBranchCount` | 15,028 |
| `sourceUseNamespaceReExportMissBranchCount` | 17,431 |
| `sourceUseNamespaceReExportMemberBranchCount` | 2 |
| `sourceUseSideEffectOnlyBranchCount` | 37 |
| `sourceUseReExportNamespaceSkipBranchCount` | 126 |
| `sourceUseBroadNamespaceBranchCount` | 885 |
| `sourceUseDirectConsumerBranchCount` | 23,301 |

Namespace re-export use records are numerous, but their post-resolution handling
is not expensive in this run. Most namespace re-export branches are misses
after the target has already been resolved.

## Resolver Memo Counters

| Counter | Value |
|---|---:|
| `sourceUseResolverMemoHits` | 43,875 |
| `sourceUseResolverMemoMisses` | 24,092 |
| `sourceUseResolverMemoSize` | 24,847 |
| `symbolResolverMemoHits` | 44,028 |
| `symbolResolverMemoMisses` | 24,847 |
| `symbolResolverMemoSize` | 24,847 |

The source-use resolver cache hit rate was about 64.6%:

```text
43,875 / (43,875 + 24,092)
```

The cache is working, but 24,092 source-use misses still dominate wall time. The
next measurement needs to identify which resolver stages and path probes own
those misses.

## Interpretation

This run rules out several possible next targets:

- direct consumer insertion is not the immediate bottleneck;
- namespace/member propagation is not the immediate bottleneck;
- generated virtual handling is not the immediate bottleneck;
- unresolved diagnostic recording is not the immediate bottleneck.

The immediate bottleneck is resolver work inside source-use assembly. Because
the resolver already has run-local memoization, the next useful slice is not a
larger producer scheduler or a broad `js-facts` fusion. It is resolver-stage
telemetry for the symbol graph resolution path.

## Recommendation

Next WT-18 implementation slice:

1. Add resolver-stage counters/timings for symbol graph source-use resolution.
2. Count stage outcomes such as relative, scoped tsconfig path, scoped baseUrl,
   exact alias, wildcard alias, hash wildcard, root prefix, generated virtual,
   external, and unresolved internal.
3. Count path probe attempts and misses for expensive resolver stages if the
   stage timings show a single stage dominating.
4. Re-run this cal.diy measurement and decide whether the fix should be
   resolver algorithm work, stronger memoization keys, or pre-indexed workspace
   package/source target maps.

Do not start producer scheduler work, Rust/rayon work, or broad `js-facts`
fusion from this evidence. The measurement says the next unknown is inside the
resolver, not inside the source-use consumer maps.

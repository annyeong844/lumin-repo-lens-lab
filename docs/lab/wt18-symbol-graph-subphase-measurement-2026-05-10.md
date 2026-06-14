# WT-18 Symbol Graph Subphase Measurement - 2026-05-10

This note records the first cal.diy full-audit run after
`build-symbol-graph.mjs` started emitting named assembly subphase timings inside
the coarse `assemble-symbol-graph` producer phase.

The goal was to identify which assembly job owns the previously observed
`assemble-symbol-graph` bucket before starting scheduler, Rust/rayon, or broad
`js-facts` fusion work.

## Run

```powershell
node .\audit-repo.mjs `
  --root "C:\Users\endof\Downloads\cal.diy-main" `
  --profile full `
  --output "$env:TEMP\lumin-wt18-cal-diy-symbol-subphase-20260510173457" `
  --cache-root "$env:TEMP\lumin-wt18-cal-diy-symbol-subphase-cache-20260510173457" `
  --clear-incremental-cache
```

- Lumin checkout: `d37d63c`
- Package version: `0.9.0-beta.37`
- Corpus: `C:\Users\endof\Downloads\cal.diy-main`
- Output:
  `C:\Users\endof\AppData\Local\Temp\lumin-wt18-cal-diy-symbol-subphase-20260510173457`
- Cache root:
  `C:\Users\endof\AppData\Local\Temp\lumin-wt18-cal-diy-symbol-subphase-cache-20260510173457`
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
| Total wall time | 353,220 ms |
| Artifact count | 19 |
| Total artifact bytes | 45,074,127 |
| Orchestrator artifact reads | 34 |
| Orchestrator JSON parse time | 473 ms |
| Phase-supporting producers | 3 |

Top producer wall times:

| Producer | Wall time |
|---|---:|
| `build-symbol-graph.mjs` | 118,985 ms |
| `measure-topology.mjs` | 99,758 ms |
| `build-call-graph.mjs` | 52,295 ms |
| `triage-repo.mjs` | 17,228 ms |
| `build-function-clone-index.mjs` | 11,996 ms |
| `classify-dead-exports.mjs` | 11,897 ms |
| `checklist-facts.mjs` | 9,226 ms |
| `build-entry-surface.mjs` | 9,029 ms |

This run was faster than the earlier single cal.diy run, but the difference
should be treated as local variance unless repeated with a median benchmark.
The useful evidence here is the internal distribution of the symbol graph
producer.

## Symbol Graph Coarse Phases

`build-symbol-graph.mjs` wall time was 118,985 ms.

| Phase | Wall time | Share of symbol producer |
|---|---:|---:|
| `snapshot` | 4,078 ms | 3.4% |
| `cache-classification` | 13 ms | 0.0% |
| `extract-changed-files` | 8,228 ms | 6.9% |
| `assemble-symbol-graph` | 103,481 ms | 87.0% |
| `write-artifact` | 138 ms | 0.1% |

## Assembly Subphases

`assemble-symbol-graph` wall time was 103,481 ms.

| Subphase | Wall time | Share of assembly |
|---|---:|---:|
| `assemble-file-data` | 305 ms | 0.3% |
| `assemble-def-index` | 4 ms | 0.0% |
| `assemble-namespace-reexports` | 1,948 ms | 1.9% |
| `assemble-source-uses` | 100,421 ms | 97.0% |
| `assemble-mdx-uses` | 608 ms | 0.6% |
| `assemble-generated-blind-zones` | 4 ms | 0.0% |
| `assemble-dead-candidates` | 63 ms | 0.1% |
| `assemble-fan-in` | 64 ms | 0.1% |
| `assemble-any-contamination` | 35 ms | 0.0% |

The subphase sum is within timing overhead of the coarse assembly phase. The
result is decisive: the source-use processing loop owns almost the entire
symbol assembly bucket.

## Key Counters

| Counter | Value |
|---|---:|
| `snapshotFiles` | 5,056 |
| `changedFiles` | 5,056 |
| `reusedFiles` | 0 |
| `extractedFiles` | 5,056 |
| `parseErrorCount` | 0 |
| `definitionCount` | 8,672 |
| `useCount` | 67,967 |
| `sourceUseFilesProcessed` | 5,056 |
| `sourceUseRecordsProcessed` | 67,967 |
| `resolvedInternalUses` | 24,919 |
| `resolvedGeneratedVirtualUses` | 563 |
| `nonSourceAssetUses` | 218 |
| `externalUses` | 10,318 |
| `unresolvedInternalUses` | 69 |
| `namespaceReExportEntryCount` | 126 |
| `namedReExportEntryCount` | 701 |
| `mdxImportConsumerCandidateCount` | 16 |
| `barrelFileCount` | 4 |
| `deadCandidateCount` | 2,827 |
| `fanInIdentityCount` | 9,596 |
| `generatedConsumerBlindZoneCount` | 19 |
| `symbolsJsonBytes` | 15,995,801 |

## Interpretation

This run rules out several possible next targets for the immediate
`build-symbol-graph` bottleneck:

- namespace and named re-export index construction is visible but small
  at 1.9 seconds;
- MDX import consumer handling is small at 0.6 seconds;
- generated blind-zone construction, dead candidate construction, fan-in, and
  any-contamination facts are all negligible in this corpus;
- extraction is still visible at 8.2 seconds, but it is not the dominant symbol
  graph cost in this run.

The current bottleneck is the `assemble-source-uses` loop over 67,967 use
records. That loop includes resolver calls, generated virtual handling,
dependency import consumer recording, unresolved specifier diagnostics, direct
consumer insertion, namespace member propagation, and named re-export
propagation. Those jobs are still one timing bucket.

## Recommendation

Next WT-18 implementation slice:

1. Split `assemble-source-uses` into internal operation counters or timings.
2. Record resolver memo stats from the symbol graph resolver instance at the
   end of source-use assembly.
3. Measure whether time is dominated by resolution, consumer insertion,
   namespace/member propagation, named re-export propagation, or unresolved
   diagnostics.

Do not start producer scheduler work, Rust/rayon work, or broad `js-facts`
fusion from this evidence. The measurement says the immediate unknown is now
inside `assemble-source-uses`, not the outer producer scheduler or the broader
assembly wrapper.

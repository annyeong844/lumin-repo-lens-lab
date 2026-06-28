# WT-18 Symbol Graph Phase Measurement - 2026-05-10

This note records the first cal.diy full-audit run after
`build-symbol-graph.mjs` started emitting producer phase counters and graph
counters.

The goal was to decide whether the next WT-18 performance slice should focus on
parser extraction, JSON artifact reads, or the symbol graph assembly loop.

## Run

```powershell
node .\audit-repo.mjs `
  --root "C:\Users\endof\Downloads\cal.diy-main" `
  --profile full `
  --output "$env:TEMP\lumin-wt18-cal-diy-symbol-20260510165427" `
  --cache-root "$env:TEMP\lumin-wt18-cal-diy-symbol-cache-20260510165427" `
  --clear-incremental-cache
```

- Lumin checkout: `b152d7a`
- Package version: `0.9.0-beta.37`
- Corpus: `C:\Users\endof\Downloads\cal.diy-main`
- Output: `C:\Users\endof\AppData\Local\Temp\lumin-wt18-cal-diy-symbol-20260510165427`
- Cache root: `C:\Users\endof\AppData\Local\Temp\lumin-wt18-cal-diy-symbol-cache-20260510165427`
- Profile: `full`
- Cache mode: cold-style run with a fresh cache root and `--clear-incremental-cache`

This is a single local run, not a median benchmark. Treat absolute wall time as
local machine evidence, not a regression claim.

## Producer Summary

`producer-performance.json.summary`:

| Metric | Value |
|---|---:|
| Producers ok | 16 |
| Producers skipped | 3 |
| Total wall time | 543,064 ms |
| Artifact count | 19 |
| Total artifact bytes | 45,074,102 |
| Orchestrator artifact reads | 34 |
| Orchestrator JSON parse time | 1,007 ms |
| Phase-supporting producers | 3 |

Top producer wall times:

| Producer | Wall time |
|---|---:|
| `build-symbol-graph.mjs` | 178,594 ms |
| `measure-topology.mjs` | 152,034 ms |
| `build-call-graph.mjs` | 82,742 ms |
| `triage-repo.mjs` | 27,171 ms |
| `classify-dead-exports.mjs` | 19,830 ms |
| `build-function-clone-index.mjs` | 19,372 ms |

## Symbol Graph Breakdown

`build-symbol-graph.mjs` wall time was 178,594 ms.

| Phase | Wall time | Share of symbol producer |
|---|---:|---:|
| `snapshot` | 5,840 ms | 3.3% |
| `cache-classification` | 26 ms | 0.0% |
| `extract-changed-files` | 15,692 ms | 8.8% |
| `assemble-symbol-graph` | 152,428 ms | 85.3% |
| `write-artifact` | 151 ms | 0.1% |

Key counters:

| Counter | Value |
|---|---:|
| `snapshotFiles` | 5,056 |
| `changedFiles` | 5,056 |
| `reusedFiles` | 0 |
| `extractedFiles` | 5,056 |
| `parseErrorCount` | 0 |
| `definitionCount` | 8,672 |
| `useCount` | 67,967 |
| `reExportCount` | 1,069 |
| `typeEscapeCount` | 1,370 |
| `resolvedInternalUses` | 24,919 |
| `externalUses` | 10,318 |
| `unresolvedInternalUses` | 69 |
| `resolvedInternalEdgeCount` | 24,356 |
| `deadCandidateCount` | 2,827 |
| `trulyDeadCount` | 2,060 |
| `namespaceShadowedDeadCount` | 767 |
| `fanInIdentityCount` | 9,596 |
| `generatedConsumerBlindZoneCount` | 19 |
| `symbolsJsonBytes` | 15,995,792 |

## Interpretation

This run does not support making parser extraction the next symbol-graph
bottleneck assumption. The extraction phase was visible at 15.7 seconds, but
`assemble-symbol-graph` dominated at 152.4 seconds.

The current `assemble-symbol-graph` phase includes several different jobs:

- namespace re-export index construction,
- use resolution across 67,967 extracted use records,
- dependency import consumer recording,
- generated virtual surface handling,
- unresolved specifier diagnostics,
- consumer/fan-in maps,
- dead export candidate construction,
- generated consumer blind-zone construction,
- any-contamination facts.

Those jobs are still one timing bucket, so the next useful slice is not a broad
rewrite. It is a finer-grained assembly breakdown.

## JSON I/O Check

Orchestrator JSON parsing remained visible but not dominant:

| Artifact | Read count | Total bytes | JSON parse time |
|---|---:|---:|---:|
| `symbols.json` | 4 | 63,983,168 | 531 ms |
| `fix-plan.json` | 2 | 15,918,734 | 145 ms |
| `call-graph.json` | 2 | 8,970,318 | 103 ms |
| `topology.json` | 2 | 7,867,234 | 74 ms |
| `dead-classify.json` | 3 | 9,636,417 | 65 ms |

Total orchestrator JSON parse time was 1,007 ms out of a 543,064 ms run. This
confirms that orchestrator JSON parse elimination should not be the next primary
performance slice.

## Recommendation

Next WT-18 implementation slice:

1. Split `assemble-symbol-graph` into named subphase timings and counters.
2. Add counters for resolver calls/cache hits inside symbol assembly, if they
   are not already available from the resolver instance.
3. Re-run this cal.diy measurement and compare which assembly subphase owns the
   152-second bucket.

Do not start producer scheduler work, Rust/rayon work, or broad `js-facts`
fusion from this single run. The measurement says the immediate unknown is
inside symbol assembly, not the outer orchestrator.

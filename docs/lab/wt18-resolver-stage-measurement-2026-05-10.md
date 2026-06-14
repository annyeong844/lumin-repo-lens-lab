# WT-18 Resolver Stage Measurement - 2026-05-10

This note records the first cal.diy full-audit run after
`build-symbol-graph.mjs` started emitting resolver-stage counters and timings
for the source-use resolution loop.

The goal was to identify which resolver stage owns the previously observed
`assemble-source-use-resolve` bucket before starting producer scheduler,
Rust/rayon, broad `js-facts` fusion, or scanner expansion work.

## Run

```powershell
node .\audit-repo.mjs `
  --root "C:\Users\endof\Downloads\cal.diy-main" `
  --profile full `
  --output "$env:TEMP\lumin-wt18-cal-diy-resolver-stage-20260510184338" `
  --cache-root "$env:TEMP\lumin-wt18-cal-diy-resolver-stage-cache-20260510184338" `
  --clear-incremental-cache
```

- Lumin checkout: `816ad59`
- Package version: `0.9.0-beta.37`
- Corpus: `C:\Users\endof\Downloads\cal.diy-main`
- Output:
  `C:\Users\endof\AppData\Local\Temp\lumin-wt18-cal-diy-resolver-stage-20260510184338`
- Cache root:
  `C:\Users\endof\AppData\Local\Temp\lumin-wt18-cal-diy-resolver-stage-cache-20260510184338`
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
| Total wall time | 371,341 ms |
| Artifact count | 19 |
| Total artifact bytes | 45,074,126 |
| Orchestrator artifact reads | 34 |
| Orchestrator JSON parse time | 528 ms |
| Phase-supporting producers | 3 |

Top producer wall times:

| Producer | Wall time |
|---|---:|
| `build-symbol-graph.mjs` | 145,578 ms |
| `measure-topology.mjs` | 102,655 ms |
| `build-call-graph.mjs` | 51,289 ms |
| `classify-dead-exports.mjs` | 13,919 ms |
| `build-entry-surface.mjs` | 11,017 ms |
| `checklist-facts.mjs` | 10,963 ms |
| `build-function-clone-index.mjs` | 10,753 ms |
| `build-shape-index.mjs` | 8,181 ms |

## Symbol Graph Breakdown

`build-symbol-graph.mjs` wall time was 145,578 ms.

| Phase | Wall time | Share of symbol producer |
|---|---:|---:|
| `snapshot` | 3,432 ms | 2.4% |
| `cache-classification` | 11 ms | 0.0% |
| `extract-changed-files` | 9,398 ms | 6.5% |
| `assemble-symbol-graph` | 129,988 ms | 89.3% |
| `write-artifact` | 148 ms | 0.1% |

`assemble-source-uses` wall time was 127,211 ms, and
`assemble-source-use-resolve` accounted for 126,407 ms.

## Resolver Stage Timings

Resolver-stage counters live in
`.producer-phases/build-symbol-graph.mjs.json`.

| Stage | Attempts | Results | Wall time | Share of resolve bucket | Result rate | Avg ms / attempt |
|---|---:|---:|---:|---:|---:|---:|
| `scopedBaseUrl` | 15,342 | 495 | 105,418 ms | 83.4% | 3.2% | 6.871 |
| `scopedTsconfig` | 18,901 | 3,559 | 8,760 ms | 6.9% | 18.8% | 0.463 |
| `wildcardAlias` | 12,578 | 7,027 | 6,375 ms | 5.0% | 55.9% | 0.507 |
| `canonicalize` | 24,092 | 24,092 | 2,814 ms | 2.2% | 100.0% | 0.117 |
| `relative` | 5,191 | 5,191 | 2,025 ms | 1.6% | 100.0% | 0.390 |
| `rootPrefix` | 5,551 | 0 | 372 ms | 0.3% | 0.0% | 0.067 |
| `exactAlias` | 14,847 | 2,269 | 325 ms | 0.3% | 15.3% | 0.022 |
| `hashWildcard` | 5,551 | 0 | 43 ms | 0.0% | 0.0% | 0.008 |
| `memoHit` | 0 | 43,875 | 8 ms | 0.0% | n/a | n/a |
| `external` | 5,551 | 5,551 | 0 ms | 0.0% | 100.0% | 0.000 |

## Resolver Memo Counters

| Counter | Value |
|---|---:|
| `sourceUseResolverMemoHits` | 43,875 |
| `sourceUseResolverMemoMisses` | 24,092 |
| `sourceUseResolverMemoSize` | 24,847 |
| `symbolResolverMemoHits` | 44,028 |
| `symbolResolverMemoMisses` | 24,847 |
| `symbolResolverMemoSize` | 24,847 |

The source-use resolver cache hit rate was again about 64.6%:

```text
43,875 / (43,875 + 24,092)
```

The cache is working. The expensive path is not memo hits, canonicalization, or
external fallthrough. The decisive hot stage is `scopedBaseUrl`.

## Interpretation

This run changes the WT-18 priority.

Do not start producer scheduler work, Rust/rayon work, or broad `js-facts`
fusion from this evidence. The symbol graph bottleneck is inside a single
resolver stage:

```text
assemble-source-use-resolve  126.4s
  scopedBaseUrl              105.4s  (83.4% of the resolve bucket)
```

`scopedBaseUrl` is both expensive and low-yield in this corpus:

- 15,342 attempts;
- 495 results;
- 3.2% result rate;
- 6.871 ms per attempt on average.

The next implementation slice should inspect why scoped baseUrl probing is so
expensive. Likely candidates are repeated path probing, repeated filesystem
existence checks, tsconfig scope lookup, package/source fallback attempts, or a
missing negative cache for baseUrl candidates.

## Recommendation

Next WT-18 implementation slice:

1. Add focused scoped-baseUrl probe telemetry if the current stage aggregate is
   not enough to choose a fix.
2. Prefer a resolver-local baseUrl negative/positive probe cache or pre-indexed
   baseUrl candidate map over scheduler or parser-fusion work.
3. Re-run this cal.diy measurement after the scoped-baseUrl fix and compare
   `sourceUseResolverStageScopedBaseUrlMs`, attempts, result rate, and total
   `build-symbol-graph.mjs` wall time.


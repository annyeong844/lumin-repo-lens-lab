# WT-18 Scoped BaseUrl Probe Cache Measurement - 2026-05-10

This note records the first cal.diy full-audit run after adding a resolver-local
cache for scoped tsconfig `baseUrl` probes.

The goal was to verify whether caching `baseUrlDir + specifier` probe results
reduces the `scopedBaseUrl` stage that dominated the previous resolver-stage
measurement.

## Run

```powershell
node .\audit-repo.mjs `
  --root "C:\Users\endof\Downloads\cal.diy-main" `
  --profile full `
  --output "$env:TEMP\lumin-wt18-cal-diy-baseurl-cache-20260510185928" `
  --cache-root "$env:TEMP\lumin-wt18-cal-diy-baseurl-cache-cache-20260510185928" `
  --clear-incremental-cache
```

- Lumin checkout: local branch `codex/baseurl-probe-cache`
- Package version: `0.9.0-beta.37`
- Corpus: `C:\Users\endof\Downloads\cal.diy-main`
- Output:
  `C:\Users\endof\AppData\Local\Temp\lumin-wt18-cal-diy-baseurl-cache-20260510185928`
- Cache root:
  `C:\Users\endof\AppData\Local\Temp\lumin-wt18-cal-diy-baseurl-cache-cache-20260510185928`
- Profile: `full`
- Cache mode: cold-style run with a fresh cache root and
  `--clear-incremental-cache`

This is a single local run, not a median benchmark. Treat absolute wall time as
local machine evidence, not a regression or speedup claim.

## Producer Summary

`producer-performance.json.summary`:

| Metric | Before | After |
|---|---:|---:|
| Total wall time | 371,341 ms | 172,776 ms |
| `build-symbol-graph.mjs` | 145,578 ms | 44,540 ms |
| `measure-topology.mjs` | 102,655 ms | 41,122 ms |
| `build-call-graph.mjs` | 51,289 ms | 23,086 ms |
| Artifact count | 19 | 19 |
| Total artifact bytes | 45,074,126 | 45,074,122 |
| Orchestrator JSON parse time | 528 ms | 489 ms |

The before column is the immediately preceding resolver-stage run recorded in
`docs/lab/wt18-resolver-stage-measurement-2026-05-10.md`.

## Symbol Graph Breakdown

| Symbol graph metric | Before | After |
|---|---:|---:|
| `build-symbol-graph.mjs` wall time | 145,578 ms | 44,540 ms |
| `assemble-symbol-graph` | 129,988 ms | 30,476 ms |
| `assemble-source-uses` | 127,211 ms | 28,096 ms |
| `assemble-source-use-resolve` | 126,407 ms | 27,585 ms |

## Resolver Stage Timings

Resolver-stage counters live in
`.producer-phases/build-symbol-graph.mjs.json`.

| Stage | Attempts | Results | Cache hits | Wall time | Share of resolve bucket |
|---|---:|---:|---:|---:|---:|
| `scopedBaseUrl` | 15,342 | 495 | 13,441 | 12,437 ms | 45.1% |
| `scopedTsconfig` | 18,901 | 3,559 | 0 | 5,935 ms | 21.5% |
| `wildcardAlias` | 12,578 | 7,027 | 0 | 4,627 ms | 16.8% |
| `canonicalize` | 24,092 | 24,092 | 0 | 2,276 ms | 8.3% |
| `relative` | 5,191 | 5,191 | 0 | 1,590 ms | 5.8% |
| `rootPrefix` | 5,551 | 0 | 0 | 298 ms | 1.1% |
| `exactAlias` | 14,847 | 2,269 | 0 | 230 ms | 0.8% |
| `hashWildcard` | 5,551 | 0 | 0 | 34 ms | 0.1% |
| `memoHit` | 0 | 0 | 0 | 5 ms | 0.0% |
| `external` | 5,551 | 5,551 | 0 | 0 ms | 0.0% |

## Before / After Focus

| Metric | Before | After | Change |
|---|---:|---:|---:|
| `sourceUseResolverStageScopedBaseUrlMs` | 105,418 ms | 12,437 ms | -92,981 ms |
| `assemble-source-use-resolve` | 126,407 ms | 27,585 ms | -98,822 ms |
| `build-symbol-graph.mjs` | 145,578 ms | 44,540 ms | -101,038 ms |
| Full audit wall time | 371,341 ms | 172,776 ms | -198,565 ms |

The scoped baseUrl cache recorded 13,441 cache hits. This confirms the previous
hot stage was dominated by repeated equivalent `baseUrl` path probes across
different importer files.

## Interpretation

The resolver-local scoped baseUrl probe cache is the right first fix for the
measured bottleneck. It preserves resolver semantics while avoiding repeated
filesystem probing for the same `baseUrlDir + specifier` pair.

The hot path is no longer a single 105-second stage. Remaining symbol graph
time is spread across:

- scoped baseUrl probing at 12.4 seconds;
- scoped tsconfig path probing at 5.9 seconds;
- wildcard alias probing at 4.6 seconds;
- canonicalization at 2.3 seconds;
- relative probing at 1.6 seconds.

Further improvements should be measurement-led. The next resolver slices, if
needed, should inspect scoped tsconfig and wildcard alias probing before
returning to scheduler, Rust/rayon, or broad `js-facts` fusion work.


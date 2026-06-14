# WT-18 Beta.39 Public Cal.diy Verification - 2026-05-10

This note records the public-package verification run for the scoped `baseUrl`
cache miss telemetry added after beta.38. The goal was to verify the installed
slash-command path, confirm `cacheMisses` is emitted by the public package, and
identify the next measured resolver bottleneck before adding another cache.

## Run

Maintainer-provided run summary:

```text
/lumin-repo-lens-lab:full --root C:\Users\endof\Downloads\cal.diy-main
```

The slash command routed through the installed plugin cache:

```text
C:\Users\endof\.claude\plugins\cache\annyeong844-marketplace\lumin-repo-lens-lab\0.9.0-beta.39
```

- Package version: `0.9.0-beta.39`
- Version evidence:
  - installed `.claude-plugin/plugin.json`
  - installed `skills/lumin-repo-lens-lab/package.json`
  - maintainer `package.json`
- Corpus: `C:\Users\endof\Downloads\cal.diy-main`
- Output:
  `C:\Users\endof\Downloads\auditing-repo-structure\review-output-cal-diy-beta39-public-20260510`
- Profile: `full`
- Incremental mode: disabled
- Generated: `2026-05-10T11:34:01Z`

This is a single public-package verification run. Treat absolute wall time as
local machine evidence, not a median benchmark.

## Headline Metrics

| Metric | Value |
|---|---:|
| Total wall time | 195,513 ms |
| `build-symbol-graph.mjs` | 47,206 ms |
| `assemble-source-use-resolve` | 29,251 ms |
| `sourceUseResolverStageScopedBaseUrlMs` | 13,481 ms |
| `sourceUseResolverStageScopedBaseUrlCacheHits` | 13,441 |
| `sourceUseResolverStageScopedBaseUrlCacheMisses` | 2,303 |
| scoped baseUrl hit ratio | 85.4% |
| `scopedTsconfig` stage | 6,216 ms |
| `wildcardAlias` stage | 4,871 ms |

The new `cacheMisses` counter turns the beta.38 inferred miss count into a
direct measurement. On this corpus, scoped baseUrl probe caching had 13,441
hits and 2,303 misses, for an 85.4% hit ratio.

## Beta.38 Comparison

| Metric | beta.38 | beta.39 | Change |
|---|---:|---:|---:|
| Total wall time | 178,977 ms | 195,513 ms | +9.2% |
| `build-symbol-graph.mjs` | 45,578 ms | 47,206 ms | +3.6% |
| `assemble-source-use-resolve` | 26,913 ms | 29,251 ms | +8.7% |
| `sourceUseResolverStageScopedBaseUrlMs` | 12,217 ms | 13,481 ms | +10.4% |
| scoped baseUrl cache hits | 13,441 | 13,441 | 0 |
| scoped baseUrl cache misses | inferred 2,303 | measured 2,303 | 0 |

The scoped baseUrl hit/miss shape is deterministic across the two runs. The
wall-time movement is consistent with single-run local machine noise or small
instrumentation overhead, not an algorithmic workload regression.

## Remaining Resolver Stage Shape

| Stage | Wall time | Notes |
|---|---:|---|
| `scopedBaseUrl` | 13,481 ms | Cache is active; hit/miss counters now measured. |
| `scopedTsconfig` | 6,216 ms | Highest uncached stage and closest to scoped baseUrl's probe pattern. |
| `wildcardAlias` | 4,871 ms | Next candidate after scoped tsconfig. |
| `canonicalize` | 2,319 ms | Many attempts, low unit cost. |
| `relative` | 1,623 ms | Lower priority. |
| `rootPrefix` | 316 ms | Low priority. |
| `exactAlias` | 236 ms | Low priority. |

## Follow-Ups

1. Add scoped tsconfig probe-shape counters before caching, so the next cache
   can be justified from measured pattern matches, probe hits, probe misses,
   fallbacks, and unresolved internal results.
2. Implement scoped tsconfig probe caching only after the telemetry confirms
   the cache key shape.
3. For beta.40 cache verification, prefer three to five repeated cold runs and
   compare medians if the expected delta is close to local timing noise.

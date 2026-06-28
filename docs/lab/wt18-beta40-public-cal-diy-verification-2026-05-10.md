# WT-18 Beta.40 Public Cal.diy Verification - 2026-05-10

This note records the public-package verification run for the scoped tsconfig
probe cache added after beta.39. The goal was to verify the installed
slash-command path, confirm the scoped tsconfig cache counters are emitted by
the public package, and identify the next measured resolver bottleneck.

## Run

Maintainer-provided run summary:

```text
/lumin-repo-lens-lab:full --root C:\Users\endof\Downloads\cal.diy-main
```

The slash command routed through the installed plugin cache:

```text
C:\Users\endof\.claude\plugins\cache\annyeong844-marketplace\lumin-repo-lens-lab\0.9.0-beta.40
```

- Package version: `0.9.0-beta.40`
- Corpus: `C:\Users\endof\Downloads\cal.diy-main`
- Output:
  `C:\Users\endof\Downloads\auditing-repo-structure\review-output-cal-diy-beta40-public-20260510`
- Profile: `full`
- Incremental mode: disabled
- Generated: `2026-05-10T12:37:22Z`

This is a single public-package verification run. Treat absolute wall time as
local machine evidence, not a median benchmark.

## Headline Metrics

| Metric | beta.39 | beta.40 | Change |
|---|---:|---:|---:|
| Total wall time | 195,513 ms | 185,298 ms | -5.2% |
| `build-symbol-graph.mjs` | 47,206 ms | 47,069 ms | -0.3% |
| `assemble-source-use-resolve` | 29,251 ms | 27,913 ms | -1,338 ms |
| `sourceUseResolverStageScopedTsconfigMs` | 6,216 ms | 5,284 ms | -15.0% |
| `sourceUseResolverStageScopedTsconfigCacheHits` | 0 | 2,760 | new |
| `sourceUseResolverStageScopedTsconfigCacheMisses` | 0 | 799 | new |
| `sourceUseResolverStageScopedBaseUrlMs` | 13,481 ms | 13,076 ms | -3.0% |
| scoped baseUrl hits / misses | 13,441 / 2,303 | 13,441 / 2,303 | unchanged |
| `sourceUseResolverStageWildcardAliasMs` | 4,871 ms | 4,747 ms | -2.5% |

The scoped tsconfig probe cache hit ratio was 2,760 / (2,760 + 799) = 77.6%
for entry-resolution work. The scoped baseUrl hit/miss shape stayed identical,
which is the main regression guard for the previous resolver cache.

## Interpretation

The scoped tsconfig cache is active and did not regress scoped baseUrl caching.
The measured wall-time drop is smaller than the raw hit count may suggest
because most scoped tsconfig attempts exit through the cheap scope filter before
entry-resolution probing.

## Remaining Resolver Stage Shape

| Stage | beta.40 wall time | Notes |
|---|---:|---|
| `scopedBaseUrl` | 13,076 ms | Cache is active; hit/miss counters unchanged. |
| `scopedTsconfig` | 5,284 ms | Cache is active; hit ratio 77.6% at entry-resolution. |
| `wildcardAlias` | 4,747 ms | Highest uncached resolver stage after scoped tsconfig. |
| `canonicalize` | 2,414 ms | Many attempts, low unit cost. |
| `relative` | 1,618 ms | Lower priority. |

## Follow-Ups

1. Add a run-local wildcard alias probe cache keyed by `spec` inside a resolver
   instance.
2. Preserve all wildcard result classes: resolved file, generated virtual
   surface, `UNRESOLVED_INTERNAL`, and no-match fallback.
3. Freeze cached generated virtual surface objects so cache reuse cannot be
   corrupted by accidental caller mutation.
4. Verify the next beta through the public slash-command path on cal.diy.

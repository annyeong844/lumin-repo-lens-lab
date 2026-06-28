# WT-18 Beta.38 Public Cal.diy Verification - 2026-05-10

This note records a public-package verification run for the WT-18 scoped
`baseUrl` resolver probe cache work. The goal was to confirm that the packaged
plugin surface, not only the private checkout, carries the observed performance
improvement and telemetry.

## Run

Maintainer-provided run summary:

```powershell
node .\dist\lumin-repo-lens-lab-plugin\skills\lumin-repo-lens-lab\scripts\audit-repo.mjs `
  --root "C:\Users\endof\Downloads\cal.diy-main" `
  --profile full `
  --no-incremental `
  --output "C:\Users\endof\Downloads\auditing-repo-structure\review-output-cal-diy-beta38-public-20260510"
```

- Package version: `0.9.0-beta.38`
- Version evidence:
  - `dist/lumin-repo-lens-lab-plugin/.claude-plugin/plugin.json`
  - `dist/lumin-repo-lens-lab-plugin/skills/lumin-repo-lens-lab/package.json`
- Corpus: `C:\Users\endof\Downloads\cal.diy-main`
- Output:
  `C:\Users\endof\Downloads\auditing-repo-structure\review-output-cal-diy-beta38-public-20260510`
- Profile: `full`
- Incremental mode: disabled
- Generated: `2026-05-10T10:46:04Z`

This is a single public-package verification run. Treat absolute wall time as
local machine evidence, not a median benchmark.

## Headline Metrics

| Metric | Value |
|---|---:|
| Total wall time | 178,977 ms |
| Producers ok / failed / skipped | 16 / 0 / 3 |
| `build-symbol-graph.mjs` | 45,578 ms |
| `assemble-source-uses` | 27,401 ms |
| `assemble-source-use-resolve` | 26,913 ms |
| `sourceUseResolverStageScopedBaseUrlMs` | 12,217 ms |
| `sourceUseResolverStageScopedBaseUrlAttempts` | 15,342 |
| `sourceUseResolverStageScopedBaseUrlResults` | 495 |
| `sourceUseResolverStageScopedBaseUrlCacheHits` | 13,441 |

The run confirms the public installable package carries the scoped baseUrl
probe cache behavior. It does not isolate the beta.37 to beta.38 delta by
itself, because the comparable older cal.diy run predates multiple cumulative
WT-18 changes.

## Coarse Historical Comparison

The older cal.diy `.audit/manifest.json` from 2026-05-09 had no
`producer-performance.json`, so this comparison uses manifest command timings
only.

| Step | 2026-05-09 older run | 2026-05-10 beta.38 public | Change |
|---|---:|---:|---:|
| `build-symbol-graph.mjs` | 441,307 ms | 45,578 ms | -89.7% |
| `build-call-graph.mjs` | 344,350 ms | 21,812 ms | -93.7% |
| `measure-topology.mjs` | 147,495 ms | 38,092 ms | -74.2% |
| `checklist-facts.mjs` | 28,852 ms | 10,917 ms | -62.2% |
| `build-entry-surface.mjs` | 22,757 ms | 8,097 ms | -64.4% |
| Total | 1,065,002 ms | 178,977 ms | -83.2% |

This is enough to reject a public-package regression, but not enough to claim a
single-PR speedup. An apple-to-apple beta.37 cold run would be needed to isolate
the scoped baseUrl cache contribution alone.

## Remaining Resolver Stage Shape

| Stage | Wall time | Notes |
|---|---:|---|
| `scopedBaseUrl` | 12,217 ms | Still largest after caching; miss count was inferred before this note's follow-up counter. |
| `scopedTsconfig` | 5,826 ms | Most similar next cache candidate. |
| `wildcardAlias` | 4,445 ms | Second cache/lookup candidate. |
| `canonicalize` | 2,227 ms | Many attempts, low unit cost. |
| `relative` | 1,498 ms | Lower priority. |
| `rootPrefix` | 297 ms | Low priority. |
| `exactAlias` | 221 ms | Low priority. |

## Follow-Ups

1. Add an explicit `sourceUseResolverStageScopedBaseUrlCacheMisses` counter so
   the hit ratio no longer depends on inference from attempts and cache size.
2. Inspect scoped tsconfig path probing next if symbol graph resolution remains
   the active WT-18 bottleneck.
3. Use the slash-command entrypoint for the next public verification so the
   plugin command path is also covered.


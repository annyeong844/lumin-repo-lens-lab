# WT-18 Artifact Read Measurement - 2026-05-10

This note records the first real-corpus read of
`producer-performance.json.artifactReads` after the audit-repo orchestrator
started measuring JSON artifact read and parse costs.

The goal was to decide whether the next performance slice should prioritize
orchestrator-level artifact JSON parsing, child-producer internal artifact
loading, scanner expansion, or producer fusion.

## Method

Both runs used the maintainer checkout on `main` after PR #178:

```text
node audit-repo.mjs --root <repo> --output <temp-output> --profile full --no-incremental
```

Outputs were written under the local temp directory, outside the scanned repos.
No generated audit artifacts from these runs are tracked in git.

## Results

| Repo | Total wall | Artifact reads | Read bytes | JSON parse |
| --- | ---: | ---: | ---: | ---: |
| cal.diy-main | 447.6s | 34 | 121.1 MB | 730 ms |
| next.js-canary | 727.9s | 34 | 227.9 MB | 1001 ms |

Top producers by wall time:

| Repo | Producer | Wall |
| --- | --- | ---: |
| cal.diy-main | `build-symbol-graph.mjs` | 146.1s |
| cal.diy-main | `measure-topology.mjs` | 133.8s |
| cal.diy-main | `build-call-graph.mjs` | 73.3s |
| next.js-canary | `build-symbol-graph.mjs` | 150.3s |
| next.js-canary | `measure-topology.mjs` | 95.7s |
| next.js-canary | `classify-dead-exports.mjs` | 85.2s |
| next.js-canary | `triage-repo.mjs` | 70.5s |
| next.js-canary | `build-call-graph.mjs` | 69.8s |

Largest orchestrator reads:

| Repo | Artifact | Read count | Read bytes | JSON parse |
| --- | --- | ---: | ---: | ---: |
| cal.diy-main | `symbols.json` | 4 | 64.0 MB | 421 ms |
| cal.diy-main | `fix-plan.json` | 2 | 15.9 MB | 92 ms |
| cal.diy-main | `dead-classify.json` | 3 | 9.6 MB | 39 ms |
| next.js-canary | `symbols.json` | 4 | 107.8 MB | 503 ms |
| next.js-canary | `function-clones.json` | 2 | 39.0 MB | 97 ms |
| next.js-canary | `call-graph.json` | 2 | 28.0 MB | 177 ms |

## Interpretation

The orchestrator does re-read large artifacts, especially `symbols.json`.
However, measured JSON parse time is roughly one second or less on these runs.
That is visible and worth preserving as telemetry, but it does not explain the
multi-minute full-profile wall time.

The current bottleneck remains producer work:

- repeated AST parsing and traversal across `build-symbol-graph`,
  `measure-topology`, `build-call-graph`, `build-function-clone-index`, and
  `build-shape-index`;
- large graph/symbol classification loops inside individual producers;
- file walking and framework/resource classification on very large monorepos.

## Decision

Do not make orchestrator JSON parse elimination the next primary performance
slice. Keep the counters, but use them as a guardrail.

The next high-value performance work should either:

1. add richer phase/counter telemetry inside the heaviest child producers,
   starting with `build-symbol-graph.mjs`; or
2. continue the Lumin-Fused SAFER path by reducing repeated parse/traversal
   work across symbol, topology, call graph, shape, and clone producers.

Child-producer internal artifact read/parse counters may still be useful, but
the first corpus read suggests they are unlikely to be the dominant cost unless
they expose a producer-local hot loop that is not visible from orchestrator
reads.

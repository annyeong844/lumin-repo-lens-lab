# WT-18 Source-Use Rust Boundary Measurement - 2026-07-09

This note records the first local measurement after source-use resolver work was
moved out of the hot JS loop and namespace re-export misses were routed through
the Rust source-use assembly lane.

The goal was to decide whether the next `build-symbol-graph.mjs` slice should
continue moving small source-use candidate-building logic to Rust, or whether
the remaining time has shifted to a different boundary.

## Run

```powershell
node build-symbol-graph.mjs --root . --output $out --no-incremental

node build-symbol-graph.mjs `
  --root . `
  --output $out `
  --cache-root $cache `
  --clear-incremental-cache

node build-symbol-graph.mjs `
  --root . `
  --output $out `
  --cache-root $cache
```

- Lumin checkout: `3f67360`
- Corpus: this repository checkout
- Cold-style output:
  `C:\Users\endof\AppData\Local\Temp\lumin-symbol-current-420c5c3e5cdb40e0a1207639b02819ed`
- Incremental output:
  `C:\Users\endof\AppData\Local\Temp\lumin-symbol-incremental-52893e5ab10340eeb3eba4838542e117`
- Incremental cache:
  `C:\Users\endof\AppData\Local\Temp\lumin-symbol-incremental-52893e5ab10340eeb3eba4838542e117-cache`

These are single local runs, not median benchmarks. Treat absolute wall time as
local machine evidence. The useful signal is the internal distribution.

## Cold-Style Run

The no-incremental run scanned 729 files, including 721 JS-family files and 8
Python files.

| Phase or counter | Value |
|---|---:|
| `snapshot` | 2,113 ms |
| `extract-rust-js-batch` | 2,300 ms |
| `extract-changed-files` | 2,811 ms |
| `assemble-source-use-candidate-build` | 52 ms |
| `assemble-source-uses` | 55 ms |
| `assemble-symbol-graph` | 198 ms |
| `symbol-graph-artifact-command` | 937 ms |
| `write-artifact` | 949 ms |
| `changedFiles` | 729 |
| `changedJsFiles` | 721 |
| `rustJsExtractorBatchCount` | 1 |
| `rustJsExtractorExtractedFiles` | 721 |
| `rustJsExtractorInputBytes` | 226,491 |
| `rustJsExtractorSourceBytes` | 8,333,143 |
| `sourceUseRustAssemblyCandidateCount` | 10,310 |
| `sourceUseRustAssemblyUnhandledCount` | 0 |
| `sourceUseResolverCallCountFinal` | 0 |
| `symbolsJsonBytes` | 1,685,678 |

The source-use resolver bottleneck is closed for this corpus: the final resolver
call count is zero, and all 10,310 source-use records are handled by Rust
assembly without JS fallback.

The source-use candidate builder is not the next meaningful performance target.
It is 52 ms in this run. Moving that small classification loop to Rust would add
a larger request-shape change than the measured cost justifies.

## Warm Incremental Run

The second incremental run reused every file fact.

| Phase or counter | Value |
|---|---:|
| `snapshot` | 2,326 ms |
| `cache-classification` | 10 ms |
| `extract-changed-files` | 0 ms |
| `extract-rust-js-batch` | 0 ms |
| `assemble-source-use-candidate-build` | 62 ms |
| `assemble-source-uses` | 65 ms |
| `assemble-symbol-graph` | 236 ms |
| `symbol-graph-artifact-command` | 1,026 ms |
| `write-artifact` | 1,039 ms |
| `changedFiles` | 0 |
| `reusedFiles` | 729 |
| `rustJsExtractorBatchCount` | 0 |
| `rustJsExtractorExtractedFiles` | 0 |
| `sourceUseRustAssemblyCandidateCount` | 10,310 |
| `sourceUseRustAssemblyUnhandledCount` | 0 |
| `sourceUseResolverCallCountFinal` | 0 |
| `symbolsJsonBytes` | 1,685,781 |

Incremental caching removes the JS/TS extraction cost in the unchanged case.
The remaining warm-run cost is dominated by repository snapshotting and the
Rust symbol graph artifact command.

## Interpretation

This run changes the WT-18 target selection:

- source-use resolver work is no longer the hot path in this corpus;
- source-use candidate building is measurable but small;
- JS/TS extraction remains visible only on changed/cold runs;
- warm unchanged runs are dominated by snapshotting plus the Rust finalizer
  command;
- broad movement of file collection, package discovery, or tsconfig policy into
  Rust is still a boundary change and should not be smuggled in as a small
  source-use optimization.

The next slice should not keep chasing source-use micro-branches. The evidence
says the better candidates are:

1. a typed Rust symbol finalizer cache or smaller finalizer request/response
   contract, if artifact command time remains large on bigger repos;
2. snapshot/file inventory ownership design, with explicit parity for excludes,
   languages, and package boundaries;
3. JS/TS extractor cold-run improvements only after a corpus shows parser time,
   not source-use assembly, dominates.

## Recommendation

Mark the source-use resolver migration as performance-complete for this
checkpoint. Keep source-use assembly counters in place, but do not move the
remaining JS candidate builder to Rust unless a larger corpus shows it above the
noise floor.

The next implementation plan should be a separate boundary slice for either
snapshot ownership or symbol-finalizer cache/contract reduction. Both are larger
than the source-use branch migrations and need their own parity notes before
code.

## Follow-up Validation - 2026-07-10

Follow-up source-use work added final resolver lane, outcome, and language
counters, then split SFC and MDX consumers out of the previous `Other` language
bucket.

Latest checked commit: `469a6e3`.

Current-repo dogfood:

| Counter | Value |
|---|---:|
| `sourceUseResolverCallCountFinal` | 0 |
| `sourceUseResolverPostSourceUseCallCount` | 0 |
| `sourceUseResolverRawJsCallCountFinal` | 0 |
| `sourceUseRustAssemblyCandidateCount` | 10,298 |
| `sourceUseFallbackLoopMs` | 0 |

A targeted Vue/SFC alias fixture still produces three JS resolver calls, all
reported as `sourceUseResolverLanguageSfcCallCount`, with lanes
`out-of-band-import-consumer` and `sfc-template-component-ref`. That is expected:
the remaining calls in that fixture require tsconfig alias interpretation, which
is outside the Rust-owned source-use assembly boundary for this slice.

Conclusion: the Rust-owned source-use resolver hot loop is closed for the
current corpus. Remaining JS resolver calls are visible through lane/language
counters and correspond to JS-owned package, tsconfig, alias, or framework
interpretation boundaries.

## Snapshot Walk Follow-up - 2026-07-10

After the symbol-finalizer cache landed, a repeated unchanged run reduced the
Rust finalizer command from about 1.8 seconds to zero and restored byte-identical
`symbols.json` output. Peak combined Node plus audit-core working set was about
133 MiB on the cache hit. The remaining warm phase leader was `snapshot` at
2.6 seconds.

A focused breakdown showed that hashing was not the bottleneck:

| Operation | Local range |
|---|---:|
| `collectFiles` only | 2,087-2,377 ms |
| stat-only snapshot | 2,177-2,601 ms |
| content-hash snapshot | 2,218-2,681 ms |

The walker was descending into nested Cargo `target/` trees under
`experiments/` and the offline Rust basepack. Excluding those generated trees
reduced the same file collection from 2,121 ms to 500 ms while preserving the
exact 729-file result set. The checked fix therefore extends the existing
root-level `target` prune policy to nested directories whose parent owns a
`Cargo.toml`; authored directories that merely happen to be named `target`
remain in scope. A new Rust hash-batch
boundary was rejected because it would optimize the small remainder instead of
the measured directory-walk cost.

Post-change producer dogfood preserved all 729 scanned files and produced the
following single-run comparison:

| Measurement | Before | After |
|---|---:|---:|
| cold snapshot | 2,612 ms | 876 ms |
| cold process wall | 25,139 ms | 9,878 ms |
| unchanged warm snapshot | 2,616 ms | 868 ms |
| unchanged warm process wall | 6,394 ms | 4,321 ms |
| unchanged warm combined peak working set | 132.6 MiB | 133.3 MiB |
| warm finalizer command | 0 ms | 0 ms |

The unchanged warm run retained `symbolGraphFinalizerCacheHit = 1`, emitted no
finalizer request bytes, made zero JS resolver calls, and had zero unhandled
source-use records. Before/after `symbols.json` structural comparison found
only the expected generated timestamp, temporary cache-root path, and line/byte
identity shifts caused by the two added source comments. No graph, count, tier,
or source-file inventory difference was observed.

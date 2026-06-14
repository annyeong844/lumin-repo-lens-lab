# M2 Rust Topology Scanner Compare Baseline

Date: 2026-06-15

## Scope

- Lab source repo: `C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab`
- Lab source branch: `m2-rust-topology-sidecar`
- Rust sidecar binary: `experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe`
- Rust sidecar source: same lab branch, `experiments/rust-sidecar/topology-scanner`
- Cache status: `no-incremental`
- Stable plugin touched: no
- Private CI triggered: no

## Environment

- machine/OS: Microsoft Windows NT 10.0.26200.0
- Node: v25.7.0
- rustc: 1.96.0 (ac68faa20 2026-05-25)
- cargo: 1.96.0 (30a34c682 2026-05-25)

## Decision

M2 remains compare-only. JS topology output is still the oracle, and `prefer` stays disabled.

This baseline proves zero-mismatch compare parity for `geulbat-phase1`, the lab self-scan, and a clean stable-source scan. `nuxt-main` is intentionally recorded as not fully matched: three remaining samples are tied to JS scanner `scanner-state-ambiguous` behavior around complex template syntax. A broad Rust heuristic for that class was rejected because it produced false mismatches on the stable-source corpus.

## Results

| Corpus | Files | LOC | Runtime internal edges | Rust status | Compared | Mismatches | Wrapper ms | Sidecar ms |
| --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |
| `geulbat-phase1` | 11 | 1,386 | 1 | `matched` | 11 | 0 | 53 | 3 |
| `lumin-repo-lens-lab` self-scan | 701 | 220,633 | 1,241 | `matched` | 701 | 0 | 388 | 340 |
| `stable-source-clean` | 2,072 | 443,646 | 3,834 | `matched` | 2,071 | 0 | 1,111 | 1,053 |
| `nuxt-main` | 625 | 69,681 | 1,077 | `risk-mismatch` | 625 | 3 | 375 | 291 |

## Corpus: geulbat-phase1

- root: `C:\Users\endof\Downloads\geulbat-phase1`
- output: `C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\geulbat-phase1`
- command: `node measure-topology.mjs --root C:\Users\endof\Downloads\geulbat-phase1 --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\geulbat-phase1 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin <release-binary> --rust-topology-timeout-ms 10000`
- rust status: `matched`
- mismatches: 0

## Corpus: lumin-repo-lens-lab self-scan

- root: `C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab`
- output: `C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\lab-self`
- command: `node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\lab-self --exclude node_modules --exclude dist --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin <release-binary> --rust-topology-timeout-ms 60000`
- rust status: `matched`
- mismatches: 0

## Corpus: stable-source-clean

- root: `C:\Users\endof\Downloads\auditing-repo-structure`
- output: `C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\stable-source-clean`
- command: `node measure-topology.mjs --root C:\Users\endof\Downloads\auditing-repo-structure --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\stable-source-clean --exclude node_modules --exclude dist --exclude output --exclude audit-artifacts --exclude .audit --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin <release-binary> --rust-topology-timeout-ms 60000`
- rust status: `matched`
- mismatches: 0
- note: stable checkout was used read-only; output was written outside the stable repo.

## Corpus: nuxt-main

- root: `C:\Users\endof\Downloads\nuxt-main`
- output: `C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\nuxt-main`
- command: `node measure-topology.mjs --root C:\Users\endof\Downloads\nuxt-main --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\nuxt-main --exclude node_modules --exclude dist --exclude .nuxt --exclude .output --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin <release-binary> --rust-topology-timeout-ms 60000`
- rust status: `risk-mismatch`
- mismatches: 3
- remaining samples:
  - `packages/nitro-server/src/index.ts`: JS-only `scanner-state-ambiguous`
  - `packages/nuxt/src/compiler/plugins/keyed-functions.ts`: JS-only `scanner-state-ambiguous`
  - `packages/vite/src/plugins/decorators.ts`: JS fallback leaves no edges; Rust still sees static imports plus `typeof import('@babel/core')`

## Follow-up

- Keep `prefer` disabled.
- Do not add broad Rust heuristics for `scanner-state-ambiguous`; they already proved too noisy against stable-source-clean.
- If Nuxt parity is pursued further, start with a precise JS tokenizer parity fixture for the remaining three samples.
- Continue using real mismatch samples as TDD inputs.

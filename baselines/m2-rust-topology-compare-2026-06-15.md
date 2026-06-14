# M2 Rust Topology Scanner Compare Baseline

Date: 2026-06-15

## Scope

- Lab source repo: `C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab`
- Lab source branch: `m2-rust-topology-sidecar`
- Rust sidecar binary: `experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe`
- Rust sidecar source: same lab branch, `experiments/rust-sidecar/topology-scanner`
- Original reviewed implementation packet commit: `f2380cc`
- Cache status: `no-incremental`
- Stable plugin touched: no
- Private CI triggered: no; `.github/workflows/ci.yml` is manual-only (`workflow_dispatch` present, `push` absent, `pull_request` absent)
- Git status before review packet refresh: clean (`git status --short --branch` reported only `## m2-rust-topology-sidecar`)

## Environment

- machine/OS: Microsoft Windows NT 10.0.26200.0
- Node: v25.7.0
- rustc: 1.96.0 (ac68faa20 2026-05-25)
- cargo: 1.96.0 (30a34c682 2026-05-25)

## Decision

M2 remains compare-only. JS topology output is still the oracle, and `prefer` stays disabled.

This refreshed baseline proves zero-mismatch compare parity for all four M2 corpora: `geulbat-phase1`, the lab self-scan, a clean stable-source scan, and `nuxt-main`. The previous Nuxt 3-sample gap is closed by precise JS-tokenizer state parity fixtures, not by a broad `scanner-state-ambiguous` heuristic. A broad heuristic was tested and rejected because it produced false mismatches on the lab/stable corpora.

## Results

| Corpus | Files | LOC | Runtime internal edges | Rust status | Compared | Mismatches | Wrapper ms | Sidecar ms |
| --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: |
| `geulbat-phase1` | 11 | 1,386 | 1 | `matched` | 11 | 0 | 92 | 7 |
| `lumin-repo-lens-lab` self-scan | 701 | 220,633 | 1,241 | `matched` | 701 | 0 | 606 | 559 |
| `stable-source-clean` | 2,072 | 443,646 | 3,834 | `matched` | 2,071 | 0 | 1,409 | 1,344 |
| `nuxt-main` | 625 | 69,681 | 1,077 | `matched` | 625 | 0 | 255 | 199 |

`Files` is the topology summary file count. `Compared` is the Rust/JS scanner comparison set. For `stable-source-clean`, one collected topology file was outside the Rust JS/TS scanner comparison set, so `Files=2,072` and `Compared=2,071` is expected.

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
- rust status: `matched`
- mismatches: 0
- note: the previous three Nuxt mismatch samples are now covered by parity tests for JS-like scanner-state ambiguity around nested template literals and by a regression test that keeps single conditional escaped-backtick templates from becoming broad false positives.

## Follow-up

- Keep `prefer` disabled.
- Do not add broad Rust heuristics for `scanner-state-ambiguous`; the accepted fix mirrors JS tokenizer state more directly and is guarded by lab/stable corpus parity.
- Treat this as compare evidence only. `prefer` remains closed until the project deliberately defines a separate replacement gate.
- Continue using real mismatch samples as TDD inputs.

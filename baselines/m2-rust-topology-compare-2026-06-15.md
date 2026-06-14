# M2 Rust Topology Scanner Compare Baseline

Date: 2026-06-15

## Scope

- Lab source repo: `C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab`
- Lab source branch: `m2-rust-topology-sidecar`
- Rust sidecar binary: `experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe`
- Rust sidecar source: same lab branch, `experiments/rust-sidecar/topology-scanner`
- cache status: `no-incremental`
- stable plugin touched: no
- private CI triggered: no

## Environment

- machine/OS: Microsoft Windows NT 10.0.26200.0
- Node: v25.7.0
- rustc: 1.96.0 (ac68faa20 2026-05-25)
- cargo: 1.96.0 (30a34c682 2026-05-25)

## Decision

M2 remains compare-only. JS topology output is still the oracle.

Both real smoke runs attempted the Rust sidecar and matched the JS oracle. Rust is still not preferred in M2; no topology artifact is sourced from Rust. This baseline only proves compare parity on the selected corpora.

## Corpus: geulbat-phase1

- root: `C:\Users\endof\Downloads\geulbat-phase1`
- output: `C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\geulbat-phase1`
- command: `node measure-topology.mjs --root C:\Users\endof\Downloads\geulbat-phase1 --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\geulbat-phase1 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin <release-binary> --rust-topology-timeout-ms 10000`
- command wall time: 12.8 s
- JS topology edge count: 1
- rust status: `matched`
- files compared: 11
- mismatches: 0
- mismatch samples: 0
- timeoutMs: 10000
- wrapper elapsedMs: 428
- sidecar files: 11
- sidecar elapsedMs: 7
- first sample: none

## Corpus: lumin-repo-lens-lab self-scan

- root: `C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab`
- output: `C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\lab-self`
- command: `node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\lab-self --exclude node_modules --exclude dist --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin <release-binary> --rust-topology-timeout-ms 60000`
- command wall time: 22.4 s
- JS topology edge count: 1241
- rust status: `matched`
- files compared: 701
- mismatches: 0
- mismatch samples: 0
- timeoutMs: 60000
- wrapper elapsedMs: 615
- sidecar files: 701
- sidecar elapsedMs: 555
- first sample: none

## Follow-up

- Keep `prefer` disabled.
- Keep using real mismatch samples as TDD inputs.
- Do not add `prefer` until compare parity is repeated on a broader real corpus set and the artifact contract is reviewed deliberately.

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

Both real smoke runs attempted the Rust sidecar. `geulbat-phase1` now matches the JS oracle. The lab self-scan is down to two explicit `edge-mismatch` records. Both residual samples are dynamic-import line-number mismatches in `audit-repo.mjs` and its generated skill mirror; source and specifier agree. That is acceptable for this baseline: Rust is not preferred, no topology artifact is sourced from Rust, and mismatch samples are capped for follow-up parity work.

## Corpus: geulbat-phase1

- root: `C:\Users\endof\Downloads\geulbat-phase1`
- output: `C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\geulbat-phase1`
- command: `node measure-topology.mjs --root C:\Users\endof\Downloads\geulbat-phase1 --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\geulbat-phase1 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin <release-binary> --rust-topology-timeout-ms 10000`
- command wall time: 2.0 s
- JS topology edge count: 1
- rust status: `matched`
- files compared: 11
- mismatches: 0
- mismatch samples: 0
- timeoutMs: 10000
- wrapper elapsedMs: 93
- sidecar files: 11
- sidecar elapsedMs: 5
- first sample: none

## Corpus: lumin-repo-lens-lab self-scan

- root: `C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab`
- output: `C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\lab-self`
- command: `node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\lab-self --exclude node_modules --exclude dist --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin <release-binary> --rust-topology-timeout-ms 60000`
- command wall time: 3.9 s
- JS topology edge count: 1241
- rust status: `edge-mismatch`
- files compared: 701
- mismatches: 2
- mismatch samples: 2
- timeoutMs: 60000
- wrapper elapsedMs: 510
- sidecar files: 701
- sidecar elapsedMs: 451
- first sample: `audit-repo.mjs` has matching dynamic `node:child_process` sources with line numbers shifted between JS and Rust. The skills mirror has the same residual.

## Follow-up

- Keep `prefer` disabled.
- Decide whether dynamic-import line numbers should follow JS tokenized-code offsets, source line numbers, or be ignored for compare-only parity metadata.
- Keep using mismatch samples as TDD inputs; do not add `prefer` until residuals are resolved or deliberately documented.

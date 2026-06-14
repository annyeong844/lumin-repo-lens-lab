# M2 Rust Topology Scanner Compare Baseline

Date: 2026-06-15

## Scope

- Lab source repo: `C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab`
- Lab source commit: `658e7932d5818d5fcc5a1390aad97b80da032860`
- Rust sidecar binary: `experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe`
- Rust sidecar commit: same as lab source commit
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

Both real smoke runs attempted the Rust sidecar and produced explicit `edge-mismatch` metadata. That is acceptable for this baseline: Rust is not preferred, no topology artifact is sourced from Rust, and mismatch samples are capped for follow-up parity work.

## Corpus: geulbat-phase1

- root: `C:\Users\endof\Downloads\geulbat-phase1`
- output: `C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\geulbat-phase1`
- command: `node measure-topology.mjs --root C:\Users\endof\Downloads\geulbat-phase1 --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\geulbat-phase1 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin <release-binary> --rust-topology-timeout-ms 10000`
- command wall time: 1133 ms
- JS topology edge count: 1
- rust status: `edge-mismatch`
- files compared: 11
- mismatches: 8
- mismatch samples: 8
- timeoutMs: 10000
- wrapper elapsedMs: 67
- sidecar files: 11
- sidecar elapsedMs: 5
- first sample: `02-source/callback-tool-dispatcher.ts` has Rust-only edges; this is a parity gap, not a replacement signal.

## Corpus: lumin-repo-lens-lab self-scan

- root: `C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab`
- output: `C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\lab-self`
- command: `node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m2-rust-topology\lab-self --exclude node_modules --exclude dist --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin <release-binary> --rust-topology-timeout-ms 60000`
- command wall time: 3385 ms
- JS topology edge count: 1241
- rust status: `edge-mismatch`
- files compared: 701
- mismatches: 313
- mismatch samples: 10
- timeoutMs: 60000
- wrapper elapsedMs: 492
- sidecar files: 701
- sidecar elapsedMs: 428
- first sample: `_lib/alias-map.mjs` has a JS-only edge for `./generated-artifact-evidence.mjs`; this is a static import parity gap to address before any prefer mode.

## Follow-up

- Keep `prefer` disabled.
- Improve Rust import parsing parity before claiming performance wins.
- Use the mismatch samples as the next TDD inputs.

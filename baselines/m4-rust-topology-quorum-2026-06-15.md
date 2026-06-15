# M4 Rust Topology Quorum Evidence

Date: 2026-06-15

## Decision

This records quorum evidence for the Rust topology scanner. `prefer` remains disabled and JS remains authoritative.

## Commands

- `node measure-topology.mjs --root C:\Users\endof\Downloads\geulbat-phase1 --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\geulbat-phase1\run-001 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\lab-self\run-001 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-audit --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\stable-source-clean\run-001 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\nuxt-main --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\nuxt-main\run-001 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\geulbat-phase1 --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\geulbat-phase1\run-002 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\lab-self\run-002 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-audit --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\stable-source-clean\run-002 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\nuxt-main --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\nuxt-main\run-002 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\geulbat-phase1 --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\geulbat-phase1\run-003 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\lab-self\run-003 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-audit --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\stable-source-clean\run-003 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`
- `node measure-topology.mjs --root C:\Users\endof\Downloads\nuxt-main --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\nuxt-main\run-003 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000`

## Corpus Runs

| Corpus | Runs | Latest Three | Files Compared | Mismatches | Command Wall ms | Scanner Bridge ms | Sidecar ms |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| `geulbat-phase1` | 3 | clean | 11 | 0 | 863 | 58 | 5 |
| `lab-self` | 3 | clean | 708 | 0 | 2703 | 628 | 571 |
| `stable-source-clean` | 3 | clean | 326 | 0 | 1788 | 305 | 244 |
| `nuxt-main` | 3 | clean | 625 | 0 | 3180 | 365 | 294 |

## M3 Gate Verification

Command:

```bash
node measure-topology.mjs --root C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab --output C:\Users\endof\Downloads\lumin-perf-lab\baselines\m4-rust-topology-quorum\m3-gate-check --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin C:\Users\endof\Downloads\lumin-perf-lab\product\lumin-repo-lens-lab\experiments\rust-sidecar\topology-scanner\target\release\lumin-topology-scanner.exe --rust-topology-timeout-ms 120000 --rust-topology-prefer-gate --rust-topology-prefer-gate-corpus lab-self --rust-topology-prefer-quorum baselines/rust-topology-prefer-quorum.json
```

- `status`: `eligible`
- `preferEnabled`: `false`
- `jsRemainsOracle`: `true`

Private CI was not used.

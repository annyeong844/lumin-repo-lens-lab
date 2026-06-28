# M4 Rust Topology Quorum Evidence Design

Date: 2026-06-15

## Decision

M4 should build the quorum evidence collector for the Rust topology scanner.
It should not enable Rust replacement.

M3 built the dry-run prefer gate. The missing piece is now the evidence file
that gate reads: `baselines/rust-topology-prefer-quorum.json`.

Strong take: do not open `prefer` in M4. Opening it before the quorum collector
exists is backwards. The gate needs boring evidence first.

## Current Baseline

- M2 closure: `baselines/m2-rust-topology-closure-2026-06-15.md`
- M3 design: `docs/lab/m3-rust-topology-prefer-gate-design-2026-06-15.md`
- M3 merge commit: `2e98adeaffd3f28b179a3614973d19f0529119d1`
- Current mode contract: `off | compare`
- `prefer`: disabled
- JS topology output: authoritative
- M3 gate input path: `baselines/rust-topology-prefer-quorum.json`

## Goal

M4 should answer one question:

> Can we repeatedly collect complete, audit-friendly, no-incremental compare
> evidence for the fixed required corpus set without touching topology output?

If yes, M4 produces quorum evidence. It still does not produce replacement
approval.

## Non-Goals

- Do not enable `prefer`.
- Do not add a `prefer` CLI mode.
- Do not replace JS topology output.
- Do not mix Rust and JS output per file.
- Do not change topology artifact contracts.
- Do not ship Rust binaries in the stable plugin.
- Do not trigger private CI.
- Do not make broad Rust speed claims.

## Required Corpora

The fixed required corpus set remains:

| Corpus | Requirement |
| --- | --- |
| `geulbat-phase1` | 3 latest recorded runs are clean. |
| `lab-self` | 3 latest recorded runs are clean. |
| `stable-source-clean` | 3 latest recorded runs are clean. |
| `nuxt-main` | 3 latest recorded runs are clean. |

The run arrays are append-only chronological histories. The M3 gate evaluates
the latest three recorded runs with `slice(-3)`. M4 must preserve that meaning.

Do not let a command-line flag, config file, or hand-edited quorum file shrink
the required corpus list.

## Evidence File Contract

M4 owns this file:

- `baselines/rust-topology-prefer-quorum.json`

Schema:

```json
{
  "schemaVersion": 1,
  "requiredCorpora": ["geulbat-phase1", "lab-self", "stable-source-clean", "nuxt-main"],
  "policyVersion": "module-edge-scanner.fast.v1",
  "rustSidecarSourceCommit": "87116819c23d1e1adfbfca5def44552856e4f464",
  "runs": {
    "geulbat-phase1": [
      {
        "labSourceCommit": "2e98adeaffd3f28b179a3614973d19f0529119d1",
        "rustSidecarSourceCommit": "87116819c23d1e1adfbfca5def44552856e4f464",
        "rustSidecarBinary": "experiments/rust-sidecar/topology-scanner/target/release/lumin-topology-scanner.exe",
        "command": "node measure-topology.mjs --root C:/Users/endof/Downloads/geulbat-phase1 --output C:/Users/endof/Downloads/lumin-perf-lab/baselines/m4-rust-topology-quorum/geulbat-phase1/run-001 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-scanner-bin experiments/rust-sidecar/topology-scanner/target/release/lumin-topology-scanner.exe --rust-topology-timeout-ms 10000",
        "corpusRoot": "C:/Users/endof/Downloads/geulbat-phase1",
        "cacheMode": "no-incremental",
        "fileCount": 11,
        "filesCompared": 11,
        "mismatches": 0,
        "commandWallElapsedMs": 1200,
        "scannerBridgeElapsedMs": 100,
        "sidecarElapsedMs": 5,
        "sidecarStatus": "matched",
        "policyVersion": "module-edge-scanner.fast.v1",
        "machineOs": "Microsoft Windows NT 10.0.26200.0",
        "recordedAt": "2026-06-15T18:48:28+09:00",
        "outputDir": "C:/Users/endof/Downloads/lumin-perf-lab/baselines/m4-rust-topology-quorum/geulbat-phase1/run-001",
        "topologyJson": "C:/Users/endof/Downloads/lumin-perf-lab/baselines/m4-rust-topology-quorum/geulbat-phase1/run-001/topology.json",
        "collector": {
          "workingTreeClean": true,
          "sourceDirty": false
        }
      }
    ]
  }
}
```

The collector may add non-gate diagnostic fields under a clearly named
namespace such as `collector`, but the M3 gate must only rely on the fields it
already validates.

## Run Rules

Every quorum run must use full-coverage compare mode:

```bash
--no-incremental --clear-incremental-cache --rust-topology-scanner compare
```

Cached quorum runs are rejected in M4. That is intentional. Cache-aware quorum
can come later if someone proves `filesCompared` covers the full JS/TS scanner
comparison set. That proof does not exist today.

Each run must also pass:

- `rustTopologyScanner.status === "matched"`
- `rustTopologyScanner.mismatches === 0`
- JS and Rust policy versions match
- current corpus name is explicit
- required audit fields are present
- lab source and Rust sidecar source are clean, or dirty state is recorded and
  the run is not counted as clean evidence

Completed compare failures should still be recorded when `topology.json` exists
and `meta.rustTopologyScanner` is complete enough to form a real run record.
Those non-clean records break the latest-three clean streak, which is exactly
what we want.

Hard collector failures are different. Missing corpus roots, malformed quorum
JSON, non-zero `measure-topology.mjs` exits before topology metadata exists, and
setup failures must not invent quorum evidence. They should exit non-zero and
must not claim M4 completion.

## Collector Shape

Create a small collector rather than burying this in `measure-topology.mjs`.

Preferred file:

- `scripts/record-rust-topology-quorum.mjs`

Responsibilities:

- know the four corpus names
- receive every selected corpus root explicitly
- run `measure-topology.mjs` in compare mode with no incremental cache
- read `topology.json.meta.rustTopologyScanner`
- append one run record to `baselines/rust-topology-prefer-quorum.json`
- keep run arrays chronological and append-only
- write the quorum JSON atomically
- write a human-readable summary under `baselines/`
- never enable `prefer`

This script is lab-only. It is not a public plugin command.

## Suggested CLI

```bash
node scripts/record-rust-topology-quorum.mjs \
  --corpus geulbat-phase1 \
  --root C:/Users/endof/Downloads/geulbat-phase1 \
  --rust-topology-scanner-bin experiments/rust-sidecar/topology-scanner/target/release/lumin-topology-scanner.exe
```

All-required mode must receive all four roots explicitly:

```bash
node scripts/record-rust-topology-quorum.mjs \
  --all-required \
  --repeat 3 \
  --corpus-root geulbat-phase1=C:/Users/endof/Downloads/geulbat-phase1 \
  --corpus-root lab-self=C:/Users/endof/Downloads/lumin-perf-lab/product/lumin-repo-lens-lab \
  --corpus-root stable-source-clean=C:/Users/endof/Downloads/auditing-repo-structure \
  --corpus-root nuxt-main=C:/Users/endof/Downloads/nuxt-main \
  --rust-topology-scanner-bin experiments/rust-sidecar/topology-scanner/target/release/lumin-topology-scanner.exe
```

Root map files are allowed only for local convenience:

```bash
--roots-json baselines/rust-topology-corpus-roots.local.json
```

The root map may provide paths only. It must not redefine or shrink the required
corpus set.

Useful options:

| Option | Meaning |
| --- | --- |
| `--corpus <name>` | One required corpus name. Must be in the fixed set. |
| `--all-required` | Run all four required corpora once each. |
| `--corpus-root <name=path>` | Explicit root map entry for `--all-required`. Must be repeated for all four required corpora. |
| `--roots-json <path>` | Optional local-only root map. It supplies paths, not corpus policy. |
| `--repeat <n>` | Repeat each selected corpus. Default `1`; M4 validation should use `3`. |
| `--root <path>` | Corpus root for single-corpus mode. |
| `--output-root <path>` | Defaults to `C:/Users/endof/Downloads/lumin-perf-lab/baselines/m4-rust-topology-quorum`. |
| `--quorum <path>` | Defaults to `baselines/rust-topology-prefer-quorum.json`. |
| `--rust-topology-scanner-bin <path>` | Required unless a repo-local release binary is discoverable. |
| `--timeout-ms <ms>` | Passed through to `measure-topology.mjs`. |

No hidden corpus discovery. If a root is wrong, fail clearly.

## Source Cleanliness

Clean quorum evidence is only meaningful when the sources are identifiable.

Before appending a clean run, the collector must record:

- `git rev-parse HEAD` for the lab repo as `labSourceCommit`
- the Rust sidecar source commit as `rustSidecarSourceCommit`
- whether the lab working tree is clean
- whether the Rust sidecar source tree is clean

CLI:

```bash
--rust-sidecar-source-commit 87116819c23d1e1adfbfca5def44552856e4f464
```

If either source tree is dirty, the collector may append a non-clean diagnostic
run only when topology scanner metadata exists, but it must not count the run
as clean evidence. Record the dirty state under `collector.sourceDirty` or a
similarly explicit diagnostic field.

Do not quietly append clean evidence from dirty source. That would make the
commit fields decoration instead of evidence.

## Run Field Sources

| Quorum field | Source |
| --- | --- |
| `fileCount` | `topology.json.summary.files` when present; otherwise the documented topology summary file count for that run. |
| `filesCompared` | `topology.json.meta.rustTopologyScanner.filesCompared`. |
| `mismatches` | `topology.json.meta.rustTopologyScanner.mismatches`. |
| `commandWallElapsedMs` | Collector-measured wall time for the `measure-topology.mjs` process. |
| `scannerBridgeElapsedMs` | `topology.json.meta.rustTopologyScanner.elapsedMs`. |
| `sidecarElapsedMs` | `topology.json.meta.rustTopologyScanner.sidecarTiming.elapsedMs` when present. |
| `sidecarStatus` | `topology.json.meta.rustTopologyScanner.status`. |
| `policyVersion` | `topology.json.meta.rustTopologyScanner.policyVersion`. |
| `recordedAt` | Collector timestamp when appending the run record. |
| `outputDir` | The output directory passed to `measure-topology.mjs`. |
| `topologyJson` | Absolute or repo-readable path to the generated `topology.json`. |

Use these names. Do not overload `wrapperElapsedMs`; it is too vague.

This is a deliberate M4 contract update. The current M3 gate implementation
still names `wrapperElapsedMs` as a required quorum field. M4 must update that
required-field list to require `commandWallElapsedMs` and
`scannerBridgeElapsedMs` instead. That gate update is allowed because it changes
only quorum evidence validation, not topology output.

M4 must also update the M3 gate clean-run predicate to reject dirty-source
diagnostic runs. A run with `collector.sourceDirty === true`,
`collector.workingTreeClean !== true`, or more specific dirty flags such as
`collector.labWorkingTreeClean !== true` / `collector.rustSidecarWorkingTreeClean !== true`
must not contribute to the latest-three clean streak.

## Summary Artifact

M4 should also write:

- `baselines/m4-rust-topology-quorum-2026-06-15.md`

Minimum contents:

- lab source commit
- Rust sidecar source commit
- Rust binary path
- machine/OS
- exact commands
- corpus table with run count, latest-three status, files compared, mismatch
  count, command wall elapsed, scanner bridge elapsed, sidecar elapsed
- whether M3 gate now emits `eligible`
- explicit statement that `prefer` remains disabled
- private CI status: not used

The summary must include the exact M3 gate verification command:

```bash
node measure-topology.mjs \
  --root <current corpus root> \
  --output <gate-check-output> \
  --no-incremental \
  --clear-incremental-cache \
  --rust-topology-scanner compare \
  --rust-topology-scanner-bin <bin> \
  --rust-topology-prefer-gate \
  --rust-topology-prefer-gate-corpus lab-self \
  --rust-topology-prefer-quorum baselines/rust-topology-prefer-quorum.json
```

And the result:

- `topology.json.meta.rustTopologyPreferGate.status`
- `preferEnabled`
- `jsRemainsOracle`

## Error Handling

Collector failures should be loud and specific.

| Failure | Behavior |
| --- | --- |
| unknown corpus name | exit non-zero, do not edit quorum file |
| missing corpus root | exit non-zero, do not edit quorum file |
| missing Rust binary | record no run unless `measure-topology.mjs` produced scanner metadata |
| `measure-topology.mjs` exits non-zero | exit non-zero, do not append partial data |
| scanner status is not `matched` | append failed run if topology output exists and metadata is complete |
| malformed existing quorum JSON | exit non-zero, do not overwrite |
| required run field missing | exit non-zero before append |
| hard `measure-topology.mjs` failure with no scanner metadata | exit non-zero, do not append, and do not produce an eligible summary |

Appending failed compare runs is useful only when the topology artifact exists
and scanner metadata is complete enough to explain the failure. Do not invent
fields to make the schema pass.

## Test Strategy

Tests must prove real behavior, not missing-file setup noise.

Minimum tests:

1. Append one matched run to an existing realistic quorum file.
2. Preserve append-only chronological order across repeated runs.
3. Reject unknown corpus names.
4. Reject cached runs as quorum evidence.
5. Reject missing required audit fields.
6. Preserve failed run history so latest-three clean streak breaks.
7. Generate a Markdown summary from a realistic quorum file.
8. Verify M3 gate reads the produced quorum file and reports `eligible` only
   after latest-three clean runs for all four required corpora.
9. Verify hard `measure-topology.mjs` failure appends no quorum evidence and
   produces no eligible summary.

Test fixtures should be small but real:

- use minimal `topology.json` files with actual `meta.rustTopologyScanner`
  shapes the bridge can produce
- use temporary directories for quorum output
- use a fake `measure-topology` runner injected into the collector so tests
  exercise collector behavior instead of scaffolding presence

## Artifact Contract

Allowed M4 changes:

- create `baselines/rust-topology-prefer-quorum.json`
- create `baselines/m4-rust-topology-quorum-2026-06-15.md`
- add a lab-only quorum collector script
- add targeted tests for the collector
- update the M3 gate required quorum run fields from `wrapperElapsedMs` to
  `commandWallElapsedMs` plus `scannerBridgeElapsedMs`

Forbidden M4 changes:

- enabling `prefer`
- adding a public `prefer` command
- changing topology edge arrays
- changing topology counts
- changing Mermaid, SARIF, fix-plan, deadness, safe-fix, or export-action
  surfaces
- triggering private CI
- modifying stable `/lumin-repo-lens:*`

## M4 Exit Criteria

M4 is complete when:

- the quorum collector can append valid evidence for each required corpus
- three consecutive clean runs per required corpus are present in
  `baselines/rust-topology-prefer-quorum.json`
- the M3 gate reports `eligible` from that file
- the Markdown summary records exact commands and timings
- targeted tests cover append, reject, failed-run, and gate-read behavior
- documentation says `prefer` remains disabled
- private CI remains manual-only and unused

## What M4 Enables

M4 makes M5 possible.

M5 may design actual `prefer` semantics only after M4 records boring,
repeatable quorum evidence. Until then, Rust is still compare evidence, not
replacement.

## Final Line

M4 is paperwork for a dangerous door. Do the paperwork well, and opening the
door later becomes boring. Boring is the win.

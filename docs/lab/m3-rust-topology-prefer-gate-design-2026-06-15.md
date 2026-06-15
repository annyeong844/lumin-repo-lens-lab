# M3 Rust Topology Prefer Gate Design

Date: 2026-06-15

## Decision

M3 should build a dry-run prefer gate. It should not enable Rust replacement.

The right next step is to answer one question with evidence: "Would this run have been eligible for Rust prefer?" The actual topology output must still come from JS until a later phase deliberately opens `prefer`.

## Current Baseline

- M2 closure: `baselines/m2-rust-topology-closure-2026-06-15.md`
- M2 final implementation commit: `87116819c23d1e1adfbfca5def44552856e4f464`
- M2 merge commit: `472f188c8e10b5b0661d8dec430cbe5c43679561`
- M2 result: four selected corpora matched with zero mismatches.
- Current mode contract: `off | compare`
- `prefer`: disabled
- JS topology output: authoritative

## Strong Take

Do not start M3 by mixing Rust output into production topology artifacts.

Per-file prefer sounds clever, but it makes aggregate topology counts harder to explain and harder to audit. First replacement gate should be run-level: either the whole run is eligible for future prefer, or it is not. Mixed-source output can come later if the evidence says it is worth the complexity.

## Non-Goals

- Do not enable `prefer`.
- Do not replace JS topology output.
- Do not ship Rust binaries in the stable plugin.
- Do not trigger private CI.
- Do not claim broad Rust speed wins from compare evidence alone.
- Do not change artifact contracts except to add explicit dry-run metadata.

## M3 Output Contract

M3 adds dry-run metadata to `topology.json.meta`. The exact object should be named:

```json
{
  "rustTopologyPreferGate": {
    "status": "eligible",
    "mode": "compare",
    "scope": "run",
    "preferEnabled": false,
    "jsRemainsOracle": true,
    "reason": "all-required-corpora-matched",
    "requiredCorpora": ["geulbat-phase1", "lab-self", "stable-source-clean", "nuxt-main"],
    "currentCorpus": "lab-self",
    "currentCorpusSource": "cli",
    "quorumEvidence": "baselines/rust-topology-prefer-quorum.json",
    "cacheMode": "no-incremental",
    "mismatches": 0,
    "filesCompared": 701,
    "sidecarStatus": "matched",
    "policyVersion": "module-edge-scanner.fast.v1",
    "sidecarPolicyVersion": "module-edge-scanner.fast.v1"
  }
}
```

This metadata is advisory. It must not change the `runtimeInternalEdges`, topology summary, Mermaid output, or any downstream action lane.

`currentCorpus` must be explicit. Do not infer it from `root`, output path, repository name, or folder basename. M3 should add a CLI flag:

```bash
--rust-topology-prefer-gate-corpus lab-self
```

If the flag is missing while the gate is enabled, emit:

```json
{
  "status": "blocked-corpus-quorum",
  "reason": "current-corpus-not-declared"
}
```

## Gate Status Values

Use these status values exactly:

| Status | Meaning |
| --- | --- |
| `eligible` | This run satisfies the dry-run gate for future run-level prefer consideration. |
| `blocked-mode-off` | Rust scanner was not attempted. |
| `blocked-sidecar-failure` | Sidecar was missing, timed out, returned non-zero, or produced invalid JSON. |
| `blocked-policy-version` | JS and Rust scanner policy versions differ. |
| `blocked-count-mismatch` | JS and Rust compared different file sets. |
| `blocked-edge-mismatch` | JS and Rust edge output differs. |
| `blocked-risk-mismatch` | JS and Rust risk output differs. |
| `blocked-corpus-quorum` | Current run matched, but the required corpus history is incomplete. |
| `blocked-artifact-contract` | The run changed output fields outside approved metadata. |

No fuzzy status names. If the gate is blocked, say exactly why.

## Required Corpus Quorum

Future prefer cannot even be discussed until all required corpora have a clean history:

| Corpus | Requirement |
| --- | --- |
| `geulbat-phase1` | 3 consecutive clean compare runs on the same Rust sidecar source commit. |
| `lab-self` | 3 consecutive clean compare runs on the same Rust sidecar source commit. |
| `stable-source-clean` | 3 consecutive clean compare runs on the same Rust sidecar source commit. |
| `nuxt-main` | 3 consecutive clean compare runs on the same Rust sidecar source commit. |

Each run must record:

- lab source commit
- Rust sidecar source commit
- Rust sidecar binary path
- command
- corpus root
- cache mode
- file count
- compared file count
- mismatch count
- wrapper elapsed milliseconds
- sidecar elapsed milliseconds
- sidecar status
- policy version
- machine/OS

One clean run is a good sign. Three clean runs is a gate. Anything less is a vibe, not evidence.

Quorum evidence should live in one JSON file:

- `baselines/rust-topology-prefer-quorum.json`

Use this schema. The values below are examples; implementations must fill them from the current run.

```json
{
  "schemaVersion": 1,
  "requiredCorpora": ["geulbat-phase1", "lab-self", "stable-source-clean", "nuxt-main"],
  "policyVersion": "module-edge-scanner.fast.v1",
  "rustSidecarSourceCommit": "87116819c23d1e1adfbfca5def44552856e4f464",
  "runs": {
    "geulbat-phase1": [
      {
        "labSourceCommit": "c9f9dc7d52fdc93272dda9f8f72b3d7011f17253",
        "rustSidecarSourceCommit": "87116819c23d1e1adfbfca5def44552856e4f464",
        "rustSidecarBinary": "experiments/rust-sidecar/topology-scanner/target/release/lumin-topology-scanner.exe",
        "command": "node measure-topology.mjs --root C:/Users/endof/Downloads/geulbat-phase1 --output C:/Users/endof/Downloads/lumin-perf-lab/baselines/m3-rust-topology/geulbat-phase1 --no-incremental --clear-incremental-cache --rust-topology-scanner compare --rust-topology-prefer-gate-corpus geulbat-phase1",
        "corpusRoot": "C:/Users/endof/Downloads/geulbat-phase1",
        "cacheMode": "no-incremental",
        "fileCount": 11,
        "filesCompared": 11,
        "mismatches": 0,
        "wrapperElapsedMs": 92,
        "sidecarElapsedMs": 7,
        "sidecarStatus": "matched",
        "policyVersion": "module-edge-scanner.fast.v1",
        "machineOs": "Microsoft Windows NT 10.0.26200.0"
      }
    ]
  }
}
```

M3 quorum runs should use `--no-incremental --clear-incremental-cache`. Cached quorum runs are allowed only after a separate proof shows that `filesCompared` covers the full JS/TS scanner comparison set for that corpus. First implementation should not take that complexity. Cold full-coverage evidence is boring, and boring wins here.

## Replacement Gate Rule

M3 only records whether a run would be eligible for a future prefer mode.

The actual `prefer` mode remains out of scope until a later M4-style phase creates a separate approval document. That document must include:

- exact command-line interface for `prefer`
- fallback semantics
- artifact diff contract
- public package behavior
- rollback plan
- corpus evidence link

## Failure Handling

If Rust compare fails, JS still wins.

The run should finish with JS output unless the existing JS path fails. Rust gate failure must be visible in metadata, not hidden in logs.

Required failure mappings:

| Sidecar condition | Gate status |
| --- | --- |
| binary not found | `blocked-sidecar-failure` |
| unsupported platform | `blocked-sidecar-failure` |
| timeout | `blocked-sidecar-failure` |
| non-zero exit | `blocked-sidecar-failure` |
| invalid JSON | `blocked-sidecar-failure` |
| policy mismatch | `blocked-policy-version` |
| file set mismatch | `blocked-count-mismatch` |
| edge mismatch | `blocked-edge-mismatch` |
| risk mismatch | `blocked-risk-mismatch` |

Bridge-to-gate status mapping:

| M2 `rustTopologyScanner.status` | M3 `rustTopologyPreferGate.status` |
| --- | --- |
| `matched` | `eligible` or `blocked-corpus-quorum` |
| `binary-not-found` | `blocked-sidecar-failure` |
| `unsupported-platform` | `blocked-sidecar-failure` |
| `timeout` | `blocked-sidecar-failure` |
| `non-zero-exit` | `blocked-sidecar-failure` |
| `invalid-json-output` | `blocked-sidecar-failure` |
| `invalid-json-output` with `reason: policy-version-mismatch` | `blocked-policy-version` |
| `count-mismatch` | `blocked-count-mismatch` |
| `edge-mismatch` | `blocked-edge-mismatch` |
| `risk-mismatch` | `blocked-risk-mismatch` |
| `unsupported-file-type-or-syntax` | `blocked-sidecar-failure` |

Do not hardcode policy version strings in the gate. Read the JS value from `MODULE_EDGE_SCANNER_POLICY_VERSION` and compare it with the Rust compare metadata. Documentation examples may show `module-edge-scanner.fast.v1`, but code must use the exported constant.

## Artifact Contract

Allowed M3 artifact change:

- Add `topology.json.meta.rustTopologyPreferGate`.

Forbidden M3 artifact changes:

- changing existing topology edge arrays
- changing existing topology counts
- changing Mermaid output
- changing SARIF
- changing fix-plan, deadness, safe-fix, or export-action surfaces
- changing public plugin command names

M3 is a gate layer, not a topology rewrite.

`blocked-artifact-contract` is primarily a verifier/test status, not something a normal topology run can reliably self-diagnose. The artifact guard must compare gate-off and gate-on outputs:

1. Run topology with Rust compare metadata and the prefer gate disabled.
2. Run topology with Rust compare metadata and the prefer gate enabled.
3. Remove only `meta.rustTopologyPreferGate` from the second output.
4. Normalize naturally variable fields that already vary between runs.
5. Deep-compare topology JSON.
6. If anything else differs, the test fails and the verifier records `blocked-artifact-contract`.

## Test Strategy

Tests must prove real behavior. No fake "file missing, then create it" games.

Minimum tests:

1. Happy path: compare metadata says `matched`, corpus quorum record says all required corpora are clean, gate emits `eligible`.
2. Edge mismatch: compare metadata reports edge mismatch, gate emits `blocked-edge-mismatch`.
3. Risk mismatch: compare metadata reports risk mismatch, gate emits `blocked-risk-mismatch`.
4. Count mismatch: compare metadata reports file-set mismatch, gate emits `blocked-count-mismatch`.
5. Policy mismatch: JS and Rust policy versions differ, gate emits `blocked-policy-version`.
6. Sidecar failure: timeout or invalid JSON, gate emits `blocked-sidecar-failure`.
7. Quorum incomplete: current run matched but not enough corpus history exists, gate emits `blocked-corpus-quorum`.
8. Artifact guard: enabling the gate changes only `topology.json.meta.rustTopologyPreferGate`.

The happy path should use realistic topology metadata. Edge cases should be actual states the bridge can produce.

## Suggested Implementation Shape

Create one small gate module:

- `_lib/rust-topology-prefer-gate.mjs`

Responsibilities:

- read compare metadata
- read quorum evidence from `baselines/rust-topology-prefer-quorum.json`
- require explicit current corpus identity from `--rust-topology-prefer-gate-corpus`
- return the `rustTopologyPreferGate` object
- never mutate topology edges
- use `MODULE_EDGE_SCANNER_POLICY_VERSION` rather than a hardcoded string

Do not bury this logic inside `measure-topology.mjs`. That file already orchestrates too much. Keep the gate testable and boring.

Suggested test file:

- `tests/rust-topology-prefer-gate.test.mjs`

Suggested docs update:

- append M3 gate results to a new `baselines/m3-rust-topology-prefer-gate-YYYY-MM-DD.md`

## M3 Exit Criteria

M3 is complete when:

- `topology.json.meta.rustTopologyPreferGate` exists in compare runs.
- The gate can emit every blocked status from realistic inputs.
- Artifact guard proves no non-metadata topology output changed.
- Required corpus quorum can be recorded without enabling `prefer`.
- Documentation says `prefer` is still disabled.
- Private CI remains manual-only.

## Final Line

M3 should make future `prefer` boring to approve. If it makes replacement feel exciting, it is doing the wrong job.

# M5 Rust Topology Prefer Design

Date: 2026-06-15

## Decision

M5 may design explicit opt-in, run-level Rust topology `prefer` mode for the
lab plugin only.

This is not approval to enable Rust by default. It is not approval to ship
Rust replacement in the stable `/lumin-repo-lens:*` namespace. The correct
take is simple: Rust can be tried as a gated whole-run replacement candidate,
but JS remains the safe fallback and the public truth until the evidence is
boring for longer.

M5 should be conservative enough that a bad Rust run is annoying, not
dangerous.

## Current Evidence

M4 recorded quorum evidence in:

- `baselines/rust-topology-prefer-quorum.json`
- `baselines/m4-rust-topology-quorum-2026-06-15.md`

The latest M4 record has three clean no-incremental compare runs for every
required corpus:

| Corpus | Latest clean runs | Files compared | Mismatches |
| --- | ---: | ---: | ---: |
| `geulbat-phase1` | 3 | 11 | 0 |
| `lab-self` | 3 | 708 | 0 |
| `stable-source-clean` | 3 | 326 | 0 |
| `nuxt-main` | 3 | 625 | 0 |

The M3 dry-run gate reported:

- `status`: `eligible`
- `preferEnabled`: `false`
- `jsRemainsOracle`: `true`

Commit provenance for the M4 evidence:

- quorum collector implementation commit: `d7d5c6a`
- evidence record commit: `7ef87e4`

That distinction matters. The evidence was collected from the implementation
commit, and the later commit recorded the evidence and summary.

## Goal

Add a lab-only design for explicit Rust topology `prefer` mode that:

- requires a valid M3 gate `eligible` result;
- runs at whole-run granularity only;
- falls back to JS on any uncertainty;
- records whether Rust was attempted, used, or rejected;
- preserves the existing topology artifact contract unless an artifact guard
  explicitly proves the allowed difference;
- remains easy to turn off.

## Non-Goals

M5 must not:

- enable Rust by default;
- enable Rust in the stable `/lumin-repo-lens:*` namespace;
- mix Rust and JS output per file;
- silently fall back;
- claim broad speed wins;
- ship a public stable `prefer` command;
- trigger private CI;
- change Mermaid, SARIF, fix-plan, deadness, safe-fix, export-action, or other
  downstream surfaces as part of the design.

If a change needs those surfaces to move, that is not M5. That is a later
replacement project.

## Mode Contract

Current lab scanner modes:

- `off`
- `compare`

M5 may introduce:

- `prefer`

The `prefer` mode is explicit opt-in only:

```bash
node measure-topology.mjs \
  --root <repo> \
  --output <out> \
  --rust-topology-scanner prefer \
  --rust-topology-scanner-bin <bin> \
  --rust-topology-prefer-quorum baselines/rust-topology-prefer-quorum.json \
  --rust-topology-prefer-gate \
  --rust-topology-prefer-gate-corpus <required-corpus-name>
```

The exact flag shape can change during implementation, but these contracts
cannot:

- the user must explicitly request `prefer`;
- a quorum evidence file must be supplied;
- the current corpus identity must be explicit;
- `prefer` must be rejected if the M3 gate is not `eligible`;
- `off` and `compare` must remain available as instant rollback.

## Gate Requirements

Before Rust output may replace JS topology output for a run, all of these must
be true:

1. `--rust-topology-scanner prefer` was explicitly requested.
2. `meta.rustTopologyPreferGate.status === "eligible"`.
3. `meta.rustTopologyPreferGate.preferEnabled === false` in the gate evidence.
   M5 can use the gate result as evidence, but M5 must still make the actual
   replacement decision in its own prefer layer.
4. `meta.rustTopologyPreferGate.jsRemainsOracle === true` in the gate evidence.
5. The quorum evidence policy version matches the JS scanner policy version.
6. The Rust sidecar source commit in quorum evidence matches the sidecar being
   used, or the mismatch is blocked.
7. The sidecar binary path and source commit are recorded in metadata.
8. The current run has no sidecar failure, count mismatch, edge mismatch, or
   risk mismatch.
9. The artifact guard passes.

If any item fails, JS wins.

## Run-Level Only

M5 must be run-level prefer, not per-file prefer.

Allowed:

- all compared JS/TS topology scanner output for the run comes from Rust;
- or all topology scanner output for the run falls back to JS.

Forbidden:

- Rust output for some files and JS output for other files in the same topology
  artifact.

Per-file mixing would make counts, timing, cache semantics, and downstream
debugging a mess. It is not worth it.

## Fallback Semantics

Fallback is mandatory and visible.

JS must be used when any of these occur:

- quorum evidence missing;
- M3 gate missing;
- M3 gate not `eligible`;
- unsupported platform;
- sidecar binary not found;
- sidecar timeout;
- sidecar non-zero exit;
- invalid JSON output;
- scanner policy mismatch;
- sidecar source commit mismatch;
- count mismatch;
- edge mismatch;
- risk mismatch;
- artifact guard mismatch;
- any unknown sidecar status;
- any unknown prefer status.

Fallback metadata must say why. No silent fallback.

## Metadata Contract

M5 should add one prefer decision object under topology metadata, for example:

```json
{
  "rustTopologyPrefer": {
    "schemaVersion": 1,
    "requested": true,
    "mode": "prefer",
    "status": "used-rust",
    "usedRust": true,
    "fallbackUsed": false,
    "reason": "gate-eligible-artifact-guard-passed",
    "gateStatus": "eligible",
    "quorumEvidence": "baselines/rust-topology-prefer-quorum.json",
    "policyVersion": "module-edge-scanner.fast.v1",
    "rustSidecarSourceCommit": "d7d5c6a...",
    "rustSidecarBinary": "experiments/rust-sidecar/topology-scanner/target/release/lumin-topology-scanner.exe",
    "filesCompared": 708,
    "mismatches": 0,
    "sidecarTiming": {
      "files": 708,
      "elapsedMs": 571
    },
    "artifactGuard": {
      "status": "passed"
    }
  }
}
```

When JS fallback is used:

```json
{
  "rustTopologyPrefer": {
    "schemaVersion": 1,
    "requested": true,
    "mode": "prefer",
    "status": "fallback-js",
    "usedRust": false,
    "fallbackUsed": true,
    "reason": "blocked-risk-mismatch",
    "gateStatus": "eligible",
    "artifactGuard": {
      "status": "not-run"
    }
  }
}
```

The exact field names can be refined during implementation, but these meanings
must survive:

- was prefer requested;
- did Rust actually produce the topology scanner facts;
- if not, why not;
- which gate/quorum evidence was used;
- whether artifact guard passed;
- how to roll back.

## Artifact Guard

The artifact guard is the line between "Rust matched the scanner" and "Rust is
safe enough to own this run."

M5 should compare a prefer candidate against the JS path for the same input:

1. Run or synthesize the JS-owned topology result for the same root and options.
2. Run the Rust prefer candidate.
3. Remove only allowed metadata differences.
4. Deep-compare the topology JSON contract that downstream consumers rely on.
5. Block prefer if anything else differs.

Allowed differences:

- `meta.rustTopologyScanner`
- `meta.rustTopologyPreferGate`
- `meta.rustTopologyPrefer`
- timing fields explicitly scoped to Rust sidecar execution

Forbidden differences:

- topology edge arrays;
- topology counts;
- module file counts;
- scanner risk counts;
- Mermaid output;
- SARIF output;
- fix-plan, deadness, safe-fix, or export-action output;
- Markdown claims that imply different facts.

If the guard is too expensive for every run, M5 can make it a required
implementation validation step before allowing prefer mode to use Rust in that
build. It cannot be skipped silently.

## Quorum Evidence Use

M5 consumes `baselines/rust-topology-prefer-quorum.json`; it does not rewrite
the quorum policy.

The fixed required corpus set remains:

- `geulbat-phase1`
- `lab-self`
- `stable-source-clean`
- `nuxt-main`

The latest-three clean-run semantics remain M3/M4-owned. M5 only asks whether
the gate is eligible for the current corpus and current policy/source pair.

Do not infer corpus identity from root paths. Use an explicit corpus name.

## Public And Private CI

Private CI stays manual-only and should not be triggered for M5 design or lab
experiments.

Public lab package CI may be used when the package surface changes. That CI
can prove the package installs and smoke-runs. It does not prove Rust
replacement readiness by itself.

Record the distinction:

- private source CI: not triggered;
- public lab package CI: package validation only, when used;
- prefer activation: not approved by package CI.

## Rollback

Rollback must be boring:

- `--rust-topology-scanner off` uses JS only;
- `--rust-topology-scanner compare` records comparison metadata but JS remains
  authoritative;
- removing the quorum file or making the gate ineligible forces JS fallback;
- deleting the Rust sidecar binary forces JS fallback with visible metadata.

No persisted default should make future runs accidentally prefer Rust.

## Validation Shape

Implementation validation should use checked artifacts and realistic failure
paths. Do not pad this with file-existence or function-existence tests.

Minimum useful checks:

- happy path: explicit prefer + eligible gate + artifact guard pass uses Rust
  for the whole run and records `usedRust: true`;
- fallback path: missing binary uses JS and records `fallback-js`;
- mismatch path: risk or edge mismatch uses JS and records the exact block
  reason;
- gate path: ineligible quorum uses JS and records the gate status;
- artifact path: any non-metadata topology diff blocks prefer;
- rollback path: `off` and `compare` still behave exactly as before.

Those are real user and maintainer paths. Anything weaker is ceremony.

## M5 Success Criteria

M5 design is ready for implementation when review agrees that:

- explicit opt-in is mandatory;
- run-level only is mandatory;
- M3 `eligible` is mandatory;
- fallback is visible and JS-owned;
- artifact guard is strict enough;
- rollback is obvious;
- private CI remains unused;
- stable plugin behavior remains untouched.

M5 implementation is not complete until it proves:

- `off` behavior is unchanged;
- `compare` behavior is unchanged;
- `prefer` cannot run without eligible gate evidence;
- `prefer` either uses Rust for the full run or falls back to JS for the full
  run;
- fallback metadata is clear;
- artifact guard prevents accidental contract drift.

## Open Review Questions

1. Is run-level prefer strict enough for the first replacement gate? My take:
   yes. Per-file mixing is a trap.
2. Should M5 require artifact guard on every prefer run, or only as a release
   validation before prefer is allowed in that build?
3. Is the proposed `rustTopologyPrefer` metadata shape clear enough for a user
   to know whether Rust actually ran?
4. Do we need one more real-world corpus before implementation, or is the
   current M4 quorum enough for lab-only opt-in prefer?
5. Should single-corpus quorum CLI behavior be cleaned up before M5
   implementation, or can it remain a collector polish item?

## Follow-Up Notes From M4 Review

These are useful, but they do not block M5 design:

- clarify commit provenance in future summaries:
  - collector implementation: `d7d5c6a`
  - evidence record: `7ef87e4`
- improve single-corpus quorum collector UX with one of:
  - `--skip-gate-check`
  - `--gate-check-root`
  - optional gate check outside `--all-required`
- keep graceful missing-quorum behavior in topology metadata where practical:
  missing quorum should block, not crash, unless the caller explicitly asks for
  strict failure;
- add a source section to future markdown summaries:
  - lab source commit
  - Rust sidecar source commit
  - Rust binary
  - machine/OS

## Approval Boundary

Approval of this document would mean:

- M5 implementation planning can begin;
- `prefer` remains disabled until that implementation is separately reviewed;
- no stable plugin behavior changes;
- no replacement readiness claim;
- no broad speed claim.

That is the right next step. Open the door slowly, with a hand on the handle.

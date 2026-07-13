# Write Gate Runtime Runbook

Use this runbook when running or diagnosing JS/TS `pre-write` and
`post-write`. It separates lifecycle runtime from the editing interval and
prevents stale JS full-scan behavior from being mistaken for the current write
gate.

## Required Entrypoint

Use the orchestrator from the same checkout or generated package whose behavior
you intend to validate:

```bash
# Maintainer checkout
node audit-repo.mjs --root <repo> --output <output> ...

# Generated plugin package
node ${CLAUDE_PLUGIN_ROOT}/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs \
  --root <repo> --output <output> ...
```

Do not mix a source wrapper, binary from another checkout, and generated skill
from an older commit. The removed standalone write-gate scripts are not valid
entrypoints; measure the `audit-repo.mjs` lifecycle command end to end.

For a source-checkout diagnosis, record `node --version`, the wrapper checkout
commit, and both packaged helper contracts before changing product code. The
wrapper requires Node `^20.19.0 || >=22.12.0`. A one-off run with
`LUMIN_AUDIT_CORE_FULL_CONTRACT_PROBE=1` verifies executable behavior, including
`shapeTypeLiterals`, instead of trusting feature strings alone. Do not leave
that expensive fixture probe enabled for normal write-gate runs.

## Fresh JS/TS Contract

`--pre-write-engine auto` still selects the JS/TS lifecycle when the intent
does not declare `language: "rust"`. Audit-core discovers the scoped files,
parses them once with OXC, projects every lookup and cue, and writes the
advisory without a Node child or JS policy fallback.

After a normal fresh JS/TS run, read the invocation-specific advisory and
verify:

- `preWrite.rustEvidencePath` names
  `pre-write-evidence.<invocationId>.json`;
- `preWrite.rustEvidence.schemaVersion` is
  `lumin-js-ts-pre-write-evidence-response.v1`;
- `preWrite.rustEvidence.summary.fileCount` matches the requested scope;
- `preWrite.rustEvidence.summary.parseErrorFileCount` is `0`, or every parse
  failure is treated as incomplete evidence;
- the evidence artifact reports complete symbols, topology, and any-inventory
  metadata before an absence claim is made.

Missing `preWrite.rustEvidencePath` on the JS/TS route indicates a stale or
incompatible runtime. Stop validation,
identify the exact wrapper/package, and rebuild or reinstall it. Do not rerun a
legacy path or reinterpret old `symbols.json` as fresh Rust evidence.

Rust-source intent is separate. It writes
`rust-pre-write-artifact.<invocationId>.json`; that native artifact is neither
the JS/TS `rustEvidencePath` nor the post-write handoff.

## Normal Pair

```bash
<audit-repo> --root <repo> --output <output> \
  --pre-write --pre-write-engine auto --intent <file|->

<audit-repo> --root <repo> --output <output> \
  --post-write \
  --pre-write-advisory <output>/pre-write-advisory.<invocationId>.json
```

Use the printed invocation-specific advisory for the pair. Keep root, output,
and scan-range flags aligned.

Reusing the output and incremental cache is correct. Every scoped file is read
from the current worktree and identified from its exact bytes; reusable parse
facts do not make the inventory stale. Clear or disable the cache only for a
deliberate cold measurement or cache-compatibility diagnosis.

## Parallel Agent Ownership

The main controller owns the write-gate pair. Subagents and parallel workers do
not launch their own pre-write or post-write commands.

Partition planned work by evidence owner before creating a wave. A JS/TS wave
and a Rust wave are separate, non-overlapping transactions because one intent
has one top-level `language` selector and one pre-write owner. Do not combine
them into a mixed-language parallel wave. An all-Rust intent must preserve
`language: "rust"`; an explicit JS/TS selector remains `language: "js-ts"`.

Within one same-owner wave, merge the complete checked intent transport from
every worker, not only the five required arrays. This includes `language`,
`names` and their declarations, `shapes`, `files`, `dependencies`,
`plannedTypeEscapes`, `refactorSources`, and any supported transport metadata.
Run one pre-write for that merged intent and give every worker in the wave the
same invocation-specific advisory.

After all workers finish, run the matching post-write before starting broad
tests, builds, generators, installs, or another audit. A generator that creates
source-like files inside the scan range would otherwise appear as
`fileDelta.unexpectedNew`. If generated outputs are intentionally part of the
change, declare them in `intent.files` or apply an existing repository-owned
scan exclusion; do not invent a timing-only exclusion.

If a later worker needs files or intent lanes outside that transaction, finish
or stop the current wave and begin a new write-gate pair. Do not start an
overlapping pre-write against the same root/output/cache.

Treat pre-write and post-write as repository-scale I/O jobs. Do not overlap
them with another Lumin audit, package install, TypeScript build, Cargo build,
or broad test process over the same checkout. This scheduling rule is required
on WSL mounted worktrees, where competing directory walks and content reads can
turn a roughly ten-second compact scan into a minute-scale run.

## Timing Interpretation

Measure each command from process start to process exit. Time between the
pre-write invocation and the later post-write invocation is editing time, not
post-write runtime.

There is no wall-time cap. Compare the same checkout, entrypoint, root,
output/cache state, scan range, and intent. A large regression is a diagnostic
event, not permission to degrade evidence or skip files.

A 2026-07-12 mounted-WSL dogfood run over 2,591 JS/TS files established this
reference point for the compact path:

| Run | Wall time | Result |
| --- | ---: | --- |
| fresh pre-write | 10.2-11.5 s | complete, 0 parse-error files |
| matching post-write | 11.5 s | complete delta |

This is an observation, not a universal SLA. Historical legacy JS full-scan
pre-write runs over the same class of repository took 149-280 seconds and did
not contain `preWrite.rustEvidencePath`.

Fresh JS/TS evidence publishes contention and phase timings at
`preWrite.rustEvidence.summary.runtime` and mirrors them under
`anyInventory.meta.incremental`:

- `singleFlight.waitMs` / `timing.lockWaitMs` is time spent waiting for another
  current scan of the same canonical root to finish. A non-zero value proves
  overlapping callers were serialized; it is not parser time.
- `discoveryMs`, `sourceReadHashMs`, `parseMs`, `cacheWriteMs`, and
  `projectionMs` identify the owned scan phase that consumed time.
- `scanHeldMs` is the current invocation's admitted scan time after lock
  acquisition. `totalRuntimeMs` adds lock wait to that admitted scan time.

If process wall time is much larger than `totalRuntimeMs`, inspect wrapper
startup, binary contract probes, lifecycle rendering, and result transport.
If `lockWaitMs` is large, fix caller scheduling or the competing repository
reader; do not attribute the wait to OXC or weaken evidence collection.

## Diagnosis Order

1. Record the exact command, checkout commit or package provenance, root,
   output, platform, scan range, and intent lanes.
2. Measure pre-write and post-write separately at the process boundary.
3. Inspect the invocation-specific advisory, not `latest`.
4. For fresh JS/TS, verify `rustEvidencePath` and its schema before blaming the
   cache or repository size.
5. Read `preWrite.rustEvidence.summary` and
   `anyInventory.meta.incremental`: `loadStatus`, `changedFiles`,
   `reusedFiles`, `droppedFiles`, and `writeStatus`.
6. In WSL or override-heavy environments, rerun once with
   `LUMIN_AUDIT_CORE_CONTRACT_DEBUG=1` and, when feature strings and behavior
   disagree, once with `LUMIN_AUDIT_CORE_FULL_CONTRACT_PROBE=1`. Fix rejected
   binary/package selection; do not add a fallback classifier.
7. Exact object, literal-union, function-signature, and inline-refactor facts
   are Rust-owned current-run evidence. A missing normalization is a
   wrapper/helper provenance failure, not expected degradation.
8. Check the file inventory for accidental generated trees. Exclude a tree only
   when repository policy says it is out of scope, never just to improve time.

For post-write, also verify `baselineStatus`, `scanRangeParity`,
`typeEscapeDeltaStatus`, and `fileDeltaStatus`.

## Do Not

- Do not use `pre-write-advisory.latest.json` across task boundaries.
- Do not restore a stale-evidence or no-fresh mode to make a slow path appear fixed.
- Do not delete the incremental cache before every normal write gate.
- Do not raise a timeout, cap the repository, mute evidence, or switch to a JS
  fallback to make the command finish.
- Do not call the pair slow based on the human editing interval.
- Do not let each subagent run its own write-gate pair or overlap the gate with
  another repository-scale reader.
- Do not claim clean evidence when Rust evidence is absent or incomplete.

## Diagnostic Handoff

Include the exact command and entrypoint, commit/package provenance, per-command
wall time and exit code, invocation-specific advisory and delta paths, Rust
evidence and incremental summaries, platform, intent lanes, and scan-range
flags. Without those fields, "pre-write took minutes" is a symptom report, not
a reproducible performance finding.

# Rust `rank-fixes` Producer Migration

Status: design
Owner: Rust audit-core migration track
Date: 2026-07-03

## Purpose

Move `fix-plan.json` construction from `rank-fixes.mjs` and the fix-tier
predicate in `_lib/ranking.mjs` into `lumin-audit-core`, while preserving the
checked JS/TS artifact contract.

This is a projection-owner migration, not a language-analysis migration.
Rust may rank already-produced findings and merge already-produced evidence.
Rust must not parse JS/TS, resolve packages, classify dead exports, or prove
edit safety in this slice.

## Current Owner

`rank-fixes.mjs` currently:

1. requires `dead-classify.json`;
2. optionally reads `runtime-evidence.json`, `staleness.json`, `symbols.json`,
   `export-action-safety.json`, `call-graph.json`, `entry-surface.json`, and
   `module-reachability.json`;
3. flattens proposal buckets into findings;
4. materializes `dead-classify.excludedCandidates[]` as `MUTED`;
5. merges `export-action-safety` records by `dead-export:<file>:<symbol>:<line>`;
6. adds support evidence from module reachability and call-graph artifacts;
7. computes package/public-surface deep-import risk through JS package helpers;
8. applies `_lib/ranking.mjs::tierForFinding`;
9. emits `fix-plan.json`.

`_lib/ranking.mjs` is also imported by `emit-sarif.mjs` and several reference
tests. It cannot be deleted by this slice.

## Target Owner

Add `experiments/rust-main/lumin-audit-core/src/rank_fixes.rs`.

Rust owns:

- request validation for the rank-fixes producer input;
- proposal flattening from `dead-classify.json`;
- `excludedCandidates[]` materialization into `MUTED`;
- action-safety merge by finding id;
- runtime and staleness lookup by `file|symbol|line`;
- resolver blindness summary from `symbols.json`;
- `entry-unreachable` support from already-produced module reachability facts;
- `call-graph-no-observed-callers` support from already-produced call graph
  and symbol graph facts;
- HTML entry blind-zone matching from already-produced entry-surface facts;
- the four-tier predicate: `SAFE_FIX`, `REVIEW_FIX`, `DEGRADED`, `MUTED`;
- deterministic sorting, `summary`, `safeFixGroups`, and artifact-local
  projection for `fix-plan.json`.

Rust does not own:

- `classify-dead-exports.mjs` deadness classification;
- `_lib/export-action-safety.mjs` edit-range proof or source parsing;
- JS/TS source parsing, OXC behavior, or AST walking;
- package discovery, package export interpretation, deep-import policy, or
  public surface analysis;
- symbol graph, call graph, entry-surface, module reachability, runtime, or
  staleness producer semantics;
- SARIF emission.

## Request Contract

Add a CLI command:

```text
lumin-audit-core rank-fixes-artifact --input <path|-> [--result-output <path>]
```

Input schema:

```json
{
  "schemaVersion": "lumin-rank-fixes-producer-request.v1",
  "root": "/repo",
  "generated": "2026-07-03T00:00:00.000Z",
  "artifacts": {
    "deadClassify": {},
    "runtimeEvidence": null,
    "staleness": null,
    "symbols": null,
    "exportActionSafety": null,
    "callGraph": null,
    "entrySurface": null,
    "moduleReachability": null
  },
  "publicDeepImportRiskByFile": {
    "src/foo.ts": {
      "risk": false,
      "reason": "exports-map-closed"
    }
  }
}
```

Required fields:

- `schemaVersion`
- `root`
- `generated`
- `artifacts.deadClassify`
- `publicDeepImportRiskByFile`

Unknown request fields are ignored. Required top-level fields must be present
with the expected JSON type. Missing optional artifacts mean that evidence axis
is unavailable; missing required input is a hard request error.

`publicDeepImportRiskByFile` is supplied by the JS wrapper because the current
source of truth is `_lib/package-exports.mjs`. Rust must consume the supplied
per-file fact and must not read `package.json` or reimplement package export
policy in this slice.

## Public Deep Import Risk Coverage

`publicDeepImportRiskByFile` may contain a conservative superset of files.
Rust must only consume entries for emitted findings. Extra map entries are
allowed and must not affect output.

The JS wrapper may perform shallow file collection from `deadClassify` only to
compute package export facts. It must not rank, bucket, deduplicate, or
classify findings in a way that duplicates the Rust owner. Shallow file
collection means scanning proposal arrays and `excludedCandidates[]` for file
strings; it does not build finding identities or apply tier policy.

A missing per-file `publicDeepImportRiskByFile[file]` entry is unknown risk, not
`risk: false`. Unknown public contract risk must not promote a finding to
`SAFE_FIX`. The checked behavior for this slice is review-visible output, not a
silent safe-fix promotion.

## Shared Rust Producer CLI Contract

The command must accept `--input <path|->` and may write either to stdout or to
`--result-output <path>`. JS wrappers must use `--result-output` for normal
repository runs.

Rust must write artifact JSON only to the selected result channel. Diagnostics
must go to stderr. Invalid JSON, schema mismatch, invalid normalized paths, and
failed result-file writes must exit non-zero and must not produce a partial
success artifact.

Wrappers must treat a non-zero exit, missing result file, or malformed result
JSON as producer failure rather than falling back to JS classification.

## JS Wrapper Boundary

Keep `rank-fixes.mjs` as the compatibility entrypoint.

The wrapper may:

- parse `--root` / `--output`;
- load artifacts from the output directory;
- require `dead-classify.json`;
- compute `publicDeepImportRiskByFile` with existing JS package helpers for
  a shallow superset of files referenced by `deadClassify`;
- call `lumin-audit-core rank-fixes-artifact`;
- write `fix-plan.json`;
- print the existing console summary.

The wrapper must not keep or add:

- tier predicate logic;
- proposal bucket ranking;
- `supportedBy` computation;
- resolver blindness tier decisions;
- HTML entry blind-zone tier decisions;
- `safeFixGroups` summary math;
- fallback JS classification when Rust fails.

`_lib/ranking.mjs` stays as a reference/downstream helper until all consumers
that import it, including `emit-sarif.mjs`, have explicit replacement plans.
After this slice, it should not be the production owner for `fix-plan.json`.

## Artifact Contract

The output remains `fix-plan.json`.

Preserve existing top-level fields:

- `meta`
- `summary`
- `safeFixes`
- `safeFixGroups`
- `reviewFixes`
- `degraded`
- `muted`

Preserve tier strings:

- `SAFE_FIX`
- `REVIEW_FIX`
- `DEGRADED`
- `MUTED`

Preserve `meta.tool = "rank-fixes.mjs"` for compatibility unless the artifact
schema is versioned in the same change. If Rust ownership must be visible, add
an additive field such as:

```json
"executionOwner": "lumin-audit-core"
```

Do not change SARIF severity semantics in this slice. `emit-sarif.mjs` must
continue to read existing `fix-plan.json` tier names.

## Finding Identity

Rust must build finding identity with a single canonical builder shared by
proposal flattening, action-safety merge, runtime/staleness lookup,
excluded-candidate materialization, sorting, and deduplication.

The checked identity inputs are:

- normalized slash path;
- symbol string;
- line value using the same missing/non-numeric behavior as the checked JS
  producer.

The action-safety id remains `dead-export:<file>:<symbol>:<line>` and the
runtime/staleness lookup key remains `file|symbol|line`, but both must be
derived from the same canonical inputs. If duplicate records produce the same
finding identity, Rust must preserve checked JS precedence. `policy.excluded`
remains highest priority and yields `MUTED`.

The implementation tests must cover duplicate proposal records, proposal plus
`excludedCandidates[]` collisions, and missing or non-numeric line behavior.

## Ranking Predicate Port

The Rust predicate must preserve the checked order:

1. `policy.excluded` -> `MUTED`;
2. runtime `executed` -> `DEGRADED`;
3. blocking per-finding taint -> `DEGRADED`;
4. legacy global resolver blindness only when per-finding taint is absent;
5. `bucket = "unprocessed"` -> `DEGRADED`;
6. missing selected safe action proof -> `REVIEW_FIX`;
7. declaration export dependency not preserved -> `REVIEW_FIX`;
8. ordinary `B` bucket -> `REVIEW_FIX`;
9. HTML entry surface blind zone -> `REVIEW_FIX`;
10. public deep-import risk -> `REVIEW_FIX`;
11. selected safe action proof is complete, no selected action blockers exist,
    declaration-export binding requirements are preserved, no HTML blind zone
    exists, a per-file public contract fact is present with `risk !== true`, no
    soft taint exists, and no weak runtime status exists -> `SAFE_FIX`;
12. otherwise, selected safe action with soft taint or weak runtime status ->
    `REVIEW_FIX`;
13. otherwise `DEGRADED`.

The port must preserve structured `blockedBy` diagnostics for generated
artifact and resolver blind-zone soft taints. Do not collapse them to strings.

`clean safe action` means a selected safe action with complete proof and no
blocking or soft evidence that the checked JS predicate would use to prevent
`SAFE_FIX`. Soft taints, weak runtime status, HTML blind zones, public contract
risk, and unknown public contract risk must be evaluated before returning
`SAFE_FIX`, even if the Rust implementation organizes the predicate differently
from the numbered prose.

## Taint And Blocker Vocabulary

| Evidence | Tier effect | Shape |
|---|---|---|
| policy excluded | `MUTED` | existing `policy` shape |
| runtime executed | `DEGRADED` | existing runtime evidence shape |
| blocking per-finding taint | `DEGRADED` | existing `taintedBy[]` shape |
| legacy global resolver blindness | `DEGRADED` when per-finding taint is absent and unresolved ratio trips the checked gate | existing resolver summary shape |
| selected action blockers | `REVIEW_FIX` | `reason = "action-blockers: ..."` |
| generated artifact soft taint | `REVIEW_FIX` with `blockedPromotion` | preserve structured `blockedBy` |
| resolver blind-zone soft taint | `REVIEW_FIX` with `blockedPromotion` | preserve structured `blockedBy` |
| HTML entry blind zone | `REVIEW_FIX` with `blockedPromotion` | existing capped match shape |
| public deep-import risk | `REVIEW_FIX` | JS-supplied risk fact |
| unknown public deep-import risk | `REVIEW_FIX` | JS-supplied map missing the file |

Legacy global resolver blindness applies only when no per-finding resolver
taint is present. It must produce the same tier and reason text as the checked
JS predicate.

## Support Evidence Rules

### Entry Unreachable

Rust may add `supportedBy: [{ kind: "entry-unreachable", ... }]` only when all
of these hold:

- `moduleReachability` and `entrySurface` artifacts are present;
- the finding file is in `moduleReachability.unreachableFiles`;
- the file is not runtime reachable, type reachable, bounded out, or an entry
  file;
- the submodule completeness is `high`;
- no `symbols.dynamicImportOpacity` target directory could reach the file;
- `publicDeepImportRiskByFile[file].risk !== true`.

Bounded module reachability must not add this support. "Not observed" and
"bounded out" stay separate.

Support evidence must be claimed only when the source artifact explicitly
declares support for the required fact, or when the checked JS producer treated
that field as guaranteed for the artifact schema version. Missing support flags
are soft absence, not negative evidence.

Submodule completeness lookup must use the checked JS path-prefix selection
rule. If no matching submodule completeness record exists, completeness is not
`high`.

Dynamic import opacity checks must use normalized slash paths and the checked JS
directory-containment rule. A missing or unsupported opacity fact prevents
`entry-unreachable` support from being claimed.

### Call Graph No Observed Callers

Rust may add `supportedBy: [{ kind: "call-graph-no-observed-callers", ... }]`
only from already-produced call graph and symbols facts. It must preserve the
checked gates:

- finding is function-like;
- finding is not framework-callback-like;
- symbol graph fan-in is zero;
- call graph fan-in is zero by definition id or identity when the artifact
  declares support;
- bounded member-call stats exist;
- nearby bounded-out ratio is below `0.10`.

No new call graph analysis is allowed.

The nearby bounded-out ratio comparison is strict `< 0.10`, matching checked JS
behavior. Missing denominator, missing bounded stats, missing support flags, or
unsupported identity lookup prevents this support evidence from being emitted.
These cases must not be treated as zero observed callers.

### HTML Entry Blind Zone

Rust may compute the current suffix match from
`entrySurface.unresolvedHtmlEntrypoints[]` because this uses already-produced
artifact fields only. The emitted `blockedBy` object must preserve the existing
shape and capped match examples.

## Determinism

All output arrays must be deterministic:

- tier lists sort by `file`, then numeric `line`, then `symbol`;
- `safeFixGroups` sort by descending `count`, then `file`, then `actionKind`;
- copied evidence arrays preserve checked order where JS currently preserves
  it, or sort explicitly when JS sorted them;
- map iteration order must not leak into artifacts.

`safeFixGroups` must be derived only from emitted `safeFixes`. Group identity,
counting, representative `file`/`actionKind` fields, symbols, lines, and sort
order must preserve the existing JS artifact shape. JS must not recompute or
patch `safeFixGroups`.

## Error Handling

Hard errors:

- invalid JSON input;
- unsupported request schema;
- missing `deadClassify`;
- missing or invalid `publicDeepImportRiskByFile`;
- invalid result-file write;
- final artifact invariant failure.

Soft absence:

- optional runtime/staleness/symbol/call-graph/module-reachability artifacts
  are missing;
- optional artifact lacks a feature support flag;
- optional artifact is older and lacks an additive field.

Soft absence must not become zero evidence when that would promote to
`SAFE_FIX`.

## Optional Evidence Absence Semantics

Evidence absence is not negative evidence.

- Missing `exportActionSafety` or missing selected safe action proof prevents
  `SAFE_FIX` and yields `REVIEW_FIX` unless an earlier rule degrades or mutes.
- Missing `runtimeEvidence` must not be treated as `executed: false`.
- Missing `staleness` contributes `no-staleness` context only; it does not
  weaken or strengthen a tier by itself.
- Missing `symbols` prevents resolver-blindness and symbol fan-in support from
  being claimed.
- Missing `callGraph` prevents `call-graph-no-observed-callers` support from
  being claimed; it must not be treated as zero callers.
- Missing `moduleReachability` prevents `entry-unreachable` support from being
  claimed; it must not be treated as unreachable.
- Missing `entrySurface` prevents HTML blind-zone matching and entry-surface
  support; it must not be treated as "no blind zones" unless checked JS did so
  for that artifact schema.
- Missing `publicDeepImportRiskByFile[file]` is unknown public contract risk
  and must not promote to `SAFE_FIX`.
- Missing support flags or additive fields are soft absence. They are never
  proof of zero risk, zero callers, no runtime execution, no blind zone, or no
  public contract.

When absence would otherwise enable `SAFE_FIX`, Rust must choose the checked
JS review/degraded behavior and cover that case with a fixture.

## Acceptance Tests

Rust tests must cover product behavior, not module existence.

Required Rust cases:

- `SAFE_FIX` requires `safeAction.kind`, `proofComplete = true`, and no selected
  `actionBlockers`;
- runtime `executed` overrides all other evidence into `DEGRADED`;
- policy excluded findings become `MUTED`;
- missing safe action becomes `REVIEW_FIX`;
- blocking taints become `DEGRADED`;
- generated-artifact and resolver soft taints stay `REVIEW_FIX` with structured
  `blockedBy`;
- declaration export dependency can stay safe only with a binding-preserving
  action;
- module reachability can add `entry-unreachable` only under high complete,
  unbounded, non-entry, non-opaque, non-public-contract conditions;
- bounded module reachability does not add `entry-unreachable`;
- HTML unresolved entrypoint blind zone blocks `SAFE_FIX`;
- public deep-import risk from the JS-supplied request blocks `SAFE_FIX`;
- missing per-file public deep-import risk is unknown and blocks `SAFE_FIX`;
- call-graph no-observed-callers support requires both symbol and call graph
  zero fan-in plus bounded member-call stats;
- duplicate proposal records with the same finding id preserve checked JS
  precedence and deterministic output;
- a finding present in both proposal arrays and `excludedCandidates[]` becomes
  `MUTED`;
- missing or non-numeric line values preserve checked JS identity behavior;
- deterministic tier sorting and `safeFixGroups` sorting;
- `safeFixGroups` are derived from `safeFixes` only and preserve existing group
  object shape, count semantics, examples, and sort order.

Required compatibility fixture:

- feed the same synthetic findings/evidence vectors to checked JS
  `_lib/ranking.mjs::tierForFinding` and the Rust rank predicate;
- compare tier, reason, `blockedPromotion`, `blockedBy`, confidence, and
  confidence detail canonically;
- keep this fixture focused and do not require the full Node umbrella.

Required focused compatibility checks:

```powershell
cargo test --manifest-path experiments/Cargo.toml -p lumin-audit-core --locked --profile ci-test rank_fixes
node tests/test-rank-fixes.mjs
```

Do not run the full Node umbrella for this slice unless the user asks for it.
`tests/test-rank-fixes.mjs` is the focused JS compatibility lane.

## Dogfood

After implementation, run the focused compatibility fixture first. If it passes,
run one small real audit output that already contains:

- `dead-classify.json`;
- `export-action-safety.json`;
- `symbols.json`;
- `entry-surface.json`;
- `module-reachability.json`.

The dogfood check should compare the old JS `fix-plan.json` and the Rust-owned
`fix-plan.json` canonically. Any intentional difference requires an artifact
field, documented reason, and reviewer-visible explanation.

## Canonical Updates

Implementation must update:

- `canonical/audit-core.md` to add `rank_fixes.rs` as the owner of
  `fix-plan.json` artifact construction;
- generated skill-package source lists if a new Rust module or command is
  packaged;
- CLI usage text for `rank-fixes-artifact`;
- any maintainer docs that currently name `_lib/ranking.mjs` as the production
  `fix-plan.json` owner.

Do not remove `_lib/ranking.mjs` from docs where it remains a reference or
downstream owner for SARIF until that consumer migrates.

## Non-Goals

- No Rust JS/TS parser.
- No Rust package export resolver.
- No Rust public surface inference from `package.json`.
- No changes to `classify-dead-exports.mjs`.
- No changes to `_lib/export-action-safety.mjs` edit proof.
- No new timeouts, repo-size caps, or thresholds.
- No SARIF migration.
- No deletion of `_lib/ranking.mjs`.
- No full Node test-suite reshuffle.

## Implementation Slice

This is one implementation slice:

1. add typed Rust request/artifact projection;
2. add `rank-fixes-artifact` CLI command;
3. convert `rank-fixes.mjs` into a thin wrapper;
4. add Rust behavior tests and the focused JS compatibility check;
5. update canonical/package docs;
6. commit the migration.

If the implementation discovers that JS wrapper package facts cannot be
isolated cleanly, stop and keep `rank-fixes.mjs` as owner rather than moving
package policy into Rust.

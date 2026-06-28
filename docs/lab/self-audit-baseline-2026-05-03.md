# Self-Audit Baseline - 2026-05-03

This note records the first dogfood baseline after the public
`0.9.0-beta.12` cache-refresh release. It is maintainer evidence, not part of
the public skill entry surface.

## Baseline

- Repo head: `490a9d6` (`Merge pull request #28 from annyeong844/codex/bump-beta12-public-cache-key`)
- Audit profile: `full`
- Evidence directory: `.audit/self-baseline-20260503-234450/`
- Generated at: `2026-05-03T14:45:27.348Z`
- Scan range:
  - root: repository root
  - files: `254`
  - total LOC: `68050`
  - tests included: `true`
  - production-only mode: `false`
  - languages: `ts`, `js`
  - auto-excludes: `p6-corpus`, `output/corpus`, `review-output`,
    `audit-artifacts`, `.audit`, `test-harness`,
    `skills/lumin-repo-lens-lab/_engine`, `skills/lumin-repo-lens-lab/scripts`,
    `node_modules`
- Confidence:
  - parseErrors: `0`
  - unresolvedInternalRatio: `0`
  - resolvedInternal: `653`
  - unresolvedInternal: `0`
  - externalImports: `991`
  - blindZones: none

## Gate Snapshot

| Gate | Status | Evidence |
| --- | --- | --- |
| A2 function size | `watch` | `checklist-facts.json.A2_function_size = {big:1, medium:13, small:2827}` |
| A5 decoupling ratio | `ok` | `checklist-facts.json.A5_decoupling_ratio.gate = "ok"` |
| A6 circular deps | `ok` | `topology.json.summary.sccCount = 0` |
| B1 duplicate implementation | `watch` | `checklist-facts.json.B1_duplicate_implementation = {exactBodyGroups:0, structureGroupCandidates:1, nearFunctionCandidates:15}` |
| B3 dead code | `fix` | `fix-plan.json.summary = {SAFE_FIX:11, REVIEW_FIX:0, DEGRADED:0, MUTED:1, safeFixGroups:6}` |
| B1/B2 shape drift | `ok` | `checklist-facts.json.B1B2_shape_drift.gate = "ok"` |
| C5 lint enforcement | `ok` | `checklist-facts.json.C5_lint_enforcement.gate = "ok"` |
| C7 barrel amplification | `ok` | `checklist-facts.json.C7_barrel_amplification.gate = "ok"` |
| E2 silent catch | `watch` | `checklist-facts.json.E2_silent_catch = {count:0, emptyUndocumentedCount:0, documentedCount:21}` |

## PCEF / SAFE_FIX Snapshot

The `0.9.0-beta.12` self-audit produced `11` SAFE_FIX findings grouped into
`6` file/action groups.

| File | Action | Count | Symbols |
| --- | --- | ---: | --- |
| `_lib/classify-policies.mjs` | `delete_value_declaration` | 3 | `isCoreSentinel`, `detectNuxtNitro`, `isNuxtNitroSentinel` |
| `_lib/audit-manifest.mjs` | `demote_export_declaration` | 2 | `LIVING_AUDIT_DOC_CANDIDATES`, `detectLivingAuditDocs` |
| `_lib/classify-policies.mjs` | `remove_export_specifier` | 2 | `ACTION_NONE`, `ACTION_REVIEW_HINT` |
| `_lib/function-clone-artifact.mjs` | `demote_export_declaration` | 2 | `FUNCTION_CLONE_SCHEMA_VERSION`, `FUNCTION_CLONE_NORMALIZED_VERSION` |
| `_lib/definition-id.mjs` | `demote_export_declaration` | 1 | `makeDefinitionId` |
| `_lib/post-write-file-delta.mjs` | `demote_export_declaration` | 1 | `normalizeRepoRelativePath` |

Action-safety split:

- `remove_export_specifier`: `2`
- `delete_value_declaration`: `3`
- `demote_export_declaration`: `6`
- `actionBlockers`: none
- `strongerActionBlockers.local-refs-present`: `6`

Confidence split:

- `SAFE_FIX`: `11`
- `confidence: medium`: `11`
- `confidenceDetail: medium_with_evidence`: `9`
- `SAFE_FIX_high`: `0`

This is the intended PCEF posture at this stage: safe actions are present, but
high confidence remains reserved for stronger two-lens evidence.

## Reachability And Call-Graph Evidence

- `module-reachability.json.meta.mode = "full-bfs"`
- `module-reachability.json.meta.globalCompleteness = "high"`
- all listed submodules have `completenessBySubmodule = "high"`
- `module-reachability.json.summary = {runtimeReachable:25, typeReachable:25, reachable:25, boundedOut:0, unreachable:240, knownFiles:265}`
- `call-graph.json.meta.supports.callFanInByDefinitionId = true`
- `call-graph.json.meta.supports.callFanInByIdentity = true`
- `call-graph.json.meta.supports.truncationFix = true`

## Important Interpretation

`B3_dead_code.gate = "fix"` is now meaningful because every SAFE_FIX finding has
a concrete `safeAction`. It still should not be read as "delete these files" or
"blindly remove code." The action kind matters:

- `demote_export_declaration` preserves the local binding.
- `remove_export_specifier` removes only an export edge.
- `delete_value_declaration` is used only when the action-safety proof allows a
  stronger edit.

The only muted finding is `eslint.config.mjs::default`, excluded by
`config_FP22`.

## Next Review Slices

1. Inspect `_lib/classify-policies.mjs` SAFE_FIX group as the first concrete
   cleanup candidate. It contains both `delete_value_declaration` and
   `remove_export_specifier` actions.
2. Keep `_lib/audit-manifest.mjs` and `_lib/function-clone-artifact.mjs` grouped
   as demotion candidates. They are likely low-risk export-surface cleanup.
3. Treat `tierForFinding` in `_lib/ranking.mjs` as the next size-pressure watch
   item: `checklist-facts.json.A2_function_size.oversized[0].loc = 153`.
4. Do not change the known B1 clone cue yet:
   `classifyHelperGroup` / `classifyTypeNameGroup` remains review-only and was
   previously judged better left split by domain.


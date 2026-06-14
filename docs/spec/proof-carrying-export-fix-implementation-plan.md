# Proof-Carrying Export Fix Implementation Plan

> **Role:** maintainer-facing implementation plan for PCEF.
> **Status:** implementation plan, P0-P3.
> **Depends on:** `docs/spec/proof-carrying-export-fix.md`
> **Last updated:** 2026-05-02

This document implements the proof and ranking contract defined by
`proof-carrying-export-fix.md`. If this plan and PCEF disagree, PCEF owns the
contract and this plan must be updated.

PCEF answers:

```text
What makes a cleanup safe?
```

This plan answers:

```text
In what order should the engine collect that proof?
```

## 1. Context

The current ranking path can push too many dead-export candidates into
`REVIEW_FIX`. The root cause is not one threshold. It is four layers that stack:

1. Repo-global soft taint can affect unrelated findings.
2. Unresolved-spec matching can over-taint across package and alias scopes.
3. Deadness proof and edit-action safety are not separated strongly enough.
4. CommonJS consumers are under-modeled, so loosening ranking before CJS support
   would create false positives in mixed JS/TS repositories.

The implementation must recover meaningful `SAFE_FIX` output without lowering
the precision bar.

## 2. Principles

- Irrelevant taint removal restores the clean-deadness candidate pool.
- Public `SAFE_FIX` promotion under PCEF waits for `safeAction` proof.
- Reachability and call-graph evidence are confidence boosters, not direct
  promotion keys.
- Relevant taint blocks `SAFE_FIX` even when positive evidence exists.
- `strongerActionBlockers` never block `SAFE_FIX` when the selected weaker
  `safeAction` is valid.
- Clone, similarity, and review-hint signals are annotations, not ranking
  evidence.

## 3. Phase Map

```text
P0  candidate-deadness cleanup
    local-scope taint, tri-state unresolved matching, CJS consumers

P1  action-safety spine
    export-action-safety producer, safeAction ranking, action groups

P2  reachability booster
    resolved edges, entry-surface artifact, module reachability

P3  independent call-graph booster
    canonical definition ids, full fan-in maps, bounded member-call resolution
```

Each phase should be shippable independently. P0 may improve legacy `SAFE_FIX`
counts, but under PCEF ranking it primarily restores the clean-deadness pool.
Public PCEF `SAFE_FIX` promotion requires P1 `safeAction` proof.

## 4. P0: Candidate-Deadness Cleanup

### 4.1 Local-scope soft taint

Files:

- `_lib/finding-provenance.mjs`
- `classify-dead-exports.mjs`

`PARSE_ERRORS_ELSEWHERE` must stop acting as repo-wide disqualification unless
no local-scope information is available.

Add optional inputs:

```ts
computeFindingProvenance(finding, {
  submoduleOf,
  aliasMap
})
```

Attach parse-error soft taint only when one of these is true:

- the parse-error file is in the same submodule as the finding,
- the parse-error file contains an import shape that could target the finding
  file, or
- no submodule resolver is available, in which case preserve current behavior.

Do not implement dependency-cone taint in P0. Same-submodule scope is the first
safe narrowing.

### 4.2 Tri-state unresolved-spec matching

Replace boolean `specifierCouldMatchFile` with:

```ts
type SpecifierMatch = "match" | "no-match" | "unknown";

specifierCouldMatchFile(spec, relFile, {
  aliasMap,
  submoduleOf,
  fromHint
}): SpecifierMatch
```

Rules:

| Specifier form | Result |
|---|---|
| known alias prefix and target in alias scope | `match` or `no-match` after path normalization |
| known alias prefix and target outside alias scope | `no-match` |
| relative specifier matching importer-normalized path | `match` |
| relative specifier not matching importer-normalized path | `no-match` |
| bare package such as `react` | `no-match` |
| unknown alias-like specifier such as `@/x`, `~/x`, `#/x` | `unknown` only in same submodule; otherwise `no-match` |
| unknown non-relative specifier such as `app/foo` | `unknown` only when the finding is inside a plausible baseUrl scope; otherwise `no-match` |

Call-site behavior:

- `match`: attach `UNRESOLVED_SPEC_MATCH`.
- `no-match`: attach no taint.
- `unknown`: attach weak `UNRESOLVED_SPEC_MATCH_UNKNOWN` only when the unresolved
  consumer file is in the same submodule as the finding.

The weak taint record must include:

```json
{
  "kind": "UNRESOLVED_SPEC_MATCH_UNKNOWN",
  "consumerFile": "src/consumer.ts",
  "fromHint": "app/foo"
}
```

Run-level reporting must also preserve unresolved scope. If unresolved internal
imports remain high by absolute count or are dominated by one unresolved
workspace/alias prefix, emit a precision warning even when the unresolved ratio
is below the normal blind-zone threshold. This warning does not by itself block
candidate-local `SAFE_FIX`; it is a scope statement for users and reviewers.

### 4.3 CommonJS consumer extraction

Files:

- `extract-ts.mjs`
- `_lib/symbol-graph-artifact.mjs`
- `build-symbol-graph.mjs`

Recognize string-literal `require(...)` consumers in TS and JS.

| Pattern | Use kind | Symbol fan-in |
|---|---|---|
| `require("./x")` as statement | `cjs-side-effect-only` | no |
| `const { foo } = require("./x")` | `cjs-require-exact` | yes, `foo` |
| `const m = require("./x"); m.foo` | `cjs-namespace-member` | yes, `foo` |
| `require("./x").foo()` | `cjs-namespace-member` | yes, `foo` |
| `const m = require("./x"); use(m)` | `cjs-namespace-escape` | no, whole-file degraded |
| `module.exports = require("./x")` | `cjs-reexport-broad` | no, whole-file degraded |

`require("./x")` as a side-effect-only statement must not count as a named export
consumer. It should later flow into file-level reachability as
`cjs-side-effect`, not symbol-level fan-in.

Only treat `m.foo` as exact when `m` is a `const` require binding and the binding
is not shadowed, redeclared, reassigned, or escaped. If the guard fails, emit
`cjs-namespace-escape`.

### 4.4 P0 tests

Add or extend:

- `tests/test-finding-local-provenance.mjs`
- `tests/test-extract-cjs-consumer.mjs`
- `tests/test-cjs-classification.mjs`

Required cases:

- unrelated parse-error file does not taint a different submodule finding,
- same-submodule parse-error file remains relevant soft taint,
- known alias scope matches only files in that alias target,
- bare package specifiers are `no-match`,
- unknown alias-like specifiers weak-taint only within the same submodule,
- large absolute unresolved internal count below the ratio threshold still
  emits a run-level precision warning,
- unresolved prefix concentration below the ratio threshold still emits a
  run-level precision warning,
- bare `require("./x")` does not increase named export fan-in,
- CJS destructuring and safe namespace member reads increase exact fan-in,
- namespace binding shadow/reassign cases degrade instead of exact-protecting.

## 5. P1: Action-Safety Spine

### 5.1 New producer

Add:

```text
export-action-safety.mjs
```

Inputs:

- `dead-classify.json`
- `symbols.json`
- source files

Output:

```text
export-action-safety.json
```

Each finding should receive a selected `safeAction` or `null`.

```json
{
  "id": "dead-export:src/x.ts:foo:12",
  "safeAction": {
    "kind": "demote_export_declaration",
    "actionGroupId": "src/x.ts:VariableDeclaration:120-180",
    "target": {
      "file": "src/x.ts",
      "symbol": "foo",
      "nodeKind": "VariableDeclaration",
      "definitionId": "src/x.ts#VariableDeclaration:120-180"
    },
    "edits": [
      { "kind": "remove-token", "token": "export", "range": [120, 126] }
    ],
    "requiresModuleMarker": false,
    "preservesModuleSyntax": true,
    "preservesSideEffects": true,
    "preservesTypes": true,
    "actionBlockers": [],
    "strongerActionBlockers": ["side-effect-initializer"],
    "proofComplete": true
  }
}
```

### 5.2 Blocker split

`actionBlockers` block the selected `safeAction`; ranking reads these.

Examples:

- `partial-multi-declarator`
- `ambient-module-context`
- `re-export-from-source`
- `unrecognized-export-form`
- `declaration-merge-partner`
- `post-delete-import-cleanup-unknown`

`strongerActionBlockers` block stronger edits such as deletion; ranking must not
read these.

Examples:

- `side-effect-initializer`
- `local-refs-present`
- `identifier-initializer`
- `decorator-present`
- `class-extends`
- `class-static-field`
- `class-static-block`
- `class-computed-member`

### 5.3 Initial action rules

Use PCEF action names:

```text
remove_export_specifier
demote_export_declaration
delete_type_declaration
delete_value_declaration
```

Initial v1 policy:

- type/interface exports demote when locally referenced and delete only when no
  local type consumer remains,
- value exports demote when local refs or side effects exist,
- `export const x = call()` demotes but does not delete,
- identifiers are delete-unsafe by default,
- partial multi-declarator edits are review in v1,
- all declarators dead in one declaration may share one `actionGroupId`,
- delete actions must run post-delete import integrity after `actionGroupId`
  dedupe; if an imported binding is only used by deleted declarations, the
  action must either add an import-cleanup patch or fall back to demotion/review,
- re-export-from-source is review in v1 until side-effect-preserving patch
  generation is implemented and tested,
- default exports and star exports are review in v1,
- enums demote only,
- declaration merge partners are review in v1.

### 5.4 Module marker

When removing the last import/export would change module mode, the safe action
must include an `export {};` insertion patch. `requiresModuleMarker: true` is
metadata, not a ranking blocker.

### 5.5 Post-delete import integrity

Files:

- `export-action-safety.mjs`
- `_lib/definition-id.mjs` where action grouping needs canonical ids

For `delete_type_declaration` and `delete_value_declaration`, compute import
usage after grouping proposed edits by `actionGroupId`.

Required behavior:

- imports referenced outside deleted ranges remain untouched,
- imports referenced only inside deleted ranges receive a
  `remove_import_specifier` or `remove_import_declaration` patch when safe,
- side-effect-only import semantics are preserved when cleanup would otherwise
  stop evaluating a source module,
- final import/export removal composes with the module marker patch from §5.4,
- when cleanup cannot be proven, the delete action is blocked with
  `post-delete-import-cleanup-unknown`; a weaker demotion action may still be
  selected.

This is a P1 action-safety rule. It does not change deadness proof.

### 5.6 Ranking switch

Files:

- `_lib/ranking.mjs`
- `rank-fixes.mjs`
- `audit-repo.mjs`

Ranking should stop using mechanical bucket as the safe gate:

```js
const hasSafeAction =
  !!finding.safeAction?.kind &&
  finding.safeAction.actionBlockers.length === 0 &&
  finding.safeAction.proofComplete === true;

const deadnessClean =
  !hasBlockingTaint &&
  !hasRelevantSoftTaint &&
  !weakRuntimeStatus;

if (!hasSafeAction) return REVIEW_FIX;
if (declarationExportDependency && !isBindingPreservingAction(finding.safeAction)) {
  return REVIEW_FIX;
}
if (!deadnessClean || !contractClean) return REVIEW_FIX;
return SAFE_FIX_MEDIUM;
```

`declarationExportDependency` is not a blanket deadness blocker. It blocks
destructive declaration deletion unless the selected action preserves the local
type/value binding. A binding-preserving action such as
`demote_export_declaration` can still rank `SAFE_FIX` when the rest of the
deadness and contract proof is clean.

All P1 `SAFE_FIX` entries are `confidence: "medium"`. High confidence requires
P3.

### 5.7 P1 tests

Add or extend:

- `tests/test-export-action-safety.mjs`
- `tests/test-action-group-dedup.mjs`
- `tests/test-rank-fixes.mjs`

Required cases:

- side-effect initializer still ranks safe via demotion,
- `strongerActionBlockers` do not block ranking,
- local value refs prevent delete but allow demotion,
- local type refs prevent type deletion but allow demotion,
- pure local-unreferenced type declarations can delete,
- object/array spread, computed keys, getters, tagged templates, calls, `new`,
  and identifiers are delete-unsafe,
- module marker insertion keeps `SAFE_FIX`,
- deleting the last declarations that use an import either removes that import
  and preserves module syntax, or blocks delete with
  `post-delete-import-cleanup-unknown`,
- grouped deletes are evaluated together before import cleanup; a two-symbol
  declaration-file case should not make per-finding import decisions in
  isolation,
- re-export-from-source is review in v1,
- declaration merge partners are review,
- shared `actionGroupId` dedupes duplicate edits.

## 6. P2: Reachability Booster

### 6.1 Resolved internal edges

Add `resolvedInternalEdges` to `symbols.json`.

Keep symbol fan-in and file reachability separate:

- exact imports protect symbols,
- side-effect imports and broad CJS still create file edges,
- type-only edges are preserved with `typeOnly: true`.

Required edge kinds include:

```text
import-named
import-default
import-namespace
import-side-effect
reexport-named
reexport-broad
reexport-namespace
dynamic-literal
cjs-require-exact
cjs-namespace-member
cjs-side-effect
cjs-namespace-escape
cjs-reexport-broad
```

### 6.2 Entry surface artifact

Add `_lib/entry-surface.mjs` and emit:

```text
entry-surface.json
```

The artifact should contain:

- `publicApiFiles`
- `scriptEntrypointFiles`
- `htmlEntrypointFiles`
- `frameworkEntrypointFiles`
- `configEntrypointFiles`
- `entryFiles`
- `evidenceByFile`
- `globalCompleteness`
- `completenessBySubmodule`

Ranking uses submodule completeness, not global completeness, for local
decisions.

### 6.3 Module reachability

Add:

```text
build-module-reachability.mjs
module-reachability.json
```

Run full BFS over file edges with emergency caps. If a cap fires, unvisited
files become `boundedOutFiles`, never `unreachableFiles`.

Emit:

- `runtimeReachableFiles`
- `typeReachableFiles`
- `reachableFiles`
- `boundedOutFiles`
- `unreachableFiles`

### 6.4 Booster-only ranking

Attach `entry-unreachable` only when:

- submodule completeness is `high`,
- file is in `unreachableFiles`,
- file is neither runtime-reachable nor type-reachable,
- file is not public/framework/config/script/html entry,
- opaque dynamic imports cannot plausibly reach it,
- public deep-import risk is false.

P2 alone does not create `confidence: "high"`. It can only create
`confidenceDetail: "medium_with_evidence"`.

### 6.5 Public deep-import risk

Add `_lib/package-exports.mjs::hasPublicDeepImportRisk`.

Rules:

- `private: true` is not public deep-import risk,
- missing `exports` in a publishable package is risk,
- wildcard exports such as `./*` or `./src/*` are risk,
- explicit file exposure is risk,
- root-only conditional exports are not risk for non-exposed internals,
- `null` export leaves are explicit blockers.

### 6.6 P2 tests

Add or extend:

- `tests/test-resolved-edges.mjs`
- `tests/test-module-reachability.mjs`
- `tests/test-public-deep-import-risk.mjs`
- `tests/test-rank-high-confidence-public-package.mjs`

Required cases:

- entry-to-deep-file BFS is transitive,
- cap produces `boundedOutFiles`, not false unreachable,
- type-only edges affect type reachability,
- medium submodule completeness blocks entry-unreachable support,
- publishable wildcard exports block entry-unreachable support.

## 7. P3: Independent Call-Graph Booster

### 7.1 Canonical definition id

Add `_lib/definition-id.mjs`.

Use byte offsets, not line numbers:

```ts
makeDefinitionId(file, nodeKind, startOffset, endOffset)
```

`symbols.json::defIndex` is the canonical source. Other producers must reuse
that id rather than inventing their own.

### 7.2 Full fan-in maps

Extend `call-graph.json` with:

- `callFanInByDefinitionId`
- `callFanInByIdentity`
- `callSiteFanInByDefinitionId`
- `exportAliasMap`

`topCallees` remains display-only. Ranking must never infer zero callers from
absence in a truncated display list.

### 7.3 Bounded member-call resolution

Add `_lib/call-graph-bounded.mjs`.

Support depth-1 exported object member calls only. Depth-2 and unknown member
calls increment `boundedOutMemberCalls` and do not support high confidence.

Emit per-file:

- `boundedOutMemberCallsByFile`
- `memberCallsByFile`

### 7.4 Independent support ranking

Attach `call-graph-no-observed-callers` only when:

- symbol fan-in is zero,
- finding is function-like,
- definition-id call fan-in is zero,
- nearby bounded-out ratio is below threshold,
- finding is not a React component, hook, route handler, or framework callback.

Final confidence:

- both `entry-unreachable` and `call-graph-no-observed-callers`: `high`,
- either one: `medium_with_evidence`,
- neither: `medium`.

Relevant taint still blocks `SAFE_FIX` regardless of evidence.

### 7.5 P3 tests

Add or extend:

- `tests/test-definition-id-canonical.mjs`
- `tests/test-call-graph-bounded.mjs`
- `tests/test-call-graph-truncation-defense.mjs`
- `tests/test-rank-fixes.mjs`
- `tests/test-supported-by-allowlist.mjs`

Required cases:

- producer definition ids match,
- alias export fan-in resolves through definition id,
- 101st callee still has full-map fan-in,
- depth-1 object calls resolve,
- depth-2 object calls bounded-out,
- component/hook/route heuristics block call-graph booster,
- annotations are never read by ranking.

## 8. Non-Promoting Evidence

Signals that help review but do not prove deadness or action safety must go in
`annotations`, not `supportedBy`.

Example:

```json
{
  "supportedBy": ["entry-unreachable", "call-graph-no-observed-callers"],
  "annotations": {
    "cloneGroup": { "id": "cg-17", "similarity": 0.91 }
  }
}
```

Ranking must ignore `annotations`.

Module marker information is part of `safeAction`, not an annotation, because it
is fix-critical.

## 9. Verification

Run phase-local tests after each phase. By P3, the required unit set should
include:

```bash
node --test tests/test-finding-local-provenance.mjs \
             tests/test-extract-cjs-consumer.mjs \
             tests/test-cjs-classification.mjs \
             tests/test-export-action-safety.mjs \
             tests/test-action-group-dedup.mjs \
             tests/test-public-deep-import-risk.mjs \
             tests/test-rank-fixes.mjs \
             tests/test-module-reachability.mjs \
             tests/test-rank-high-confidence-public-package.mjs \
             tests/test-call-graph-bounded.mjs \
             tests/test-call-graph-truncation-defense.mjs \
             tests/test-definition-id-canonical.mjs \
             tests/test-supported-by-allowlist.mjs \
             tests/test-evidence-honesty.mjs
```

Also run:

```bash
bash test-harness/run-all.sh
node audit-repo.mjs --root . --output .audit --profile full
node emit-sarif.mjs --root . --output .audit
```

Expected behavior by phase:

- P1: `SAFE_FIX` exists and all entries have `safeAction`.
- P2: `SAFE_FIX_high` remains zero; `medium_with_entry_evidence` can be
  positive.
- P3: high confidence can appear only when two compatible evidence lenses agree.

## 10. Calibration

Measure false positives in two classes:

```text
deadness FP:
  SAFE_FIX has a real consumer that the graph missed.

action FP:
  deadness is correct, but the selected safeAction breaks semantics, types, or
  module syntax.
```

Accepted calibration corpus:

- false-positive budget remains zero.

Exploratory third-party corpus:

- `SAFE_FIX` deadness FP < 4%,
- `SAFE_FIX` action FP < 1%,
- overall `SAFE_FIX` FP < 5%,
- `SAFE_FIX(high)` overall FP < 2%.

For large-monorepo resolver tests, record unresolved reporting separately:

- absolute unresolved internal count warning emitted when threshold is exceeded,
- unresolved prefix-concentration warning emitted when one alias/workspace prefix
  dominates unresolved imports,
- `blindZones` or equivalent run-level precision output is not empty for such
  cases, even if the unresolved ratio is below the older confidence-gap
  threshold.

If deadness FP exceeds threshold, tighten P0/P2/P3 evidence. If action FP
exceeds threshold, tighten P1 action blockers or remove a risky action kind from
v1 safe promotion.

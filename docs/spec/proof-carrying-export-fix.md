# Proof-Carrying Export Fix (PCEF)

> **Role:** maintainer-facing design spec for increasing meaningful
> `SAFE_FIX` output without raising false positives.
> **Status:** design draft, v1. Implementation deferred; execution sequencing
> lives in `docs/spec/proof-carrying-export-fix-implementation-plan.md`.
> **Last updated:** 2026-05-02

---

## 1. Problem

`SAFE_FIX` must be useful. A dead-export tool that reports hundreds of
`REVIEW_FIX` entries and zero `SAFE_FIX` entries has moved the engineering
burden to the user. That is not a usable engine.

At the same time, widening `SAFE_FIX` by copying broad heuristics from tools
such as Knip would raise false positives. The target is stricter:

```text
Increase SAFE_FIX discovery by collecting stronger proof,
not by lowering the precision bar.
```

PCEF separates two questions that are currently too easy to conflate:

1. **Deadness proof:** is this export unobserved by the constructed graph?
2. **Action-safety proof:** what edit can be applied without changing runtime
   behavior or public contract?

The core design choice:

```text
"Unused export" is not the same as "delete declaration".
```

An export can be safe to demote even when deleting the declaration would be
unsafe.

Example:

```ts
export const token = initTelemetry();
```

If no external consumer exists, deleting the declaration can change runtime
behavior because `initTelemetry()` may have side effects. Demoting the export
preserves runtime behavior:

```ts
const token = initTelemetry();
```

That should be a `SAFE_FIX` when the proof packet is clean.

## 2. Goals

- Make `SAFE_FIX` a proof-carrying edit action, not merely a deadness label.
- Preserve the current false-positive budget: `SAFE_FIX` false positives must
  remain zero in accepted calibration corpora.
- Increase `SAFE_FIX` discovery in real TS/JS repositories by choosing weaker
  safe edits such as export demotion before declaration deletion.
- Keep policy muting conservative. Framework, public API, config, and entrypoint
  heuristics may hide known false-positive families, but broad heuristics must
  not directly promote `SAFE_FIX`.
- Make every non-safe candidate explain why it could not be promoted.

## 3. Non-goals

- Do not clone Knip, ts-prune, TypeScript Remove, Rollup, Webpack, or OXC DCE
  behavior wholesale.
- Do not make `SAFE_FIX` mean "definitely dead in all possible runtime
  executions".
- Do not use broad framework or namespace heuristics as direct safe evidence.
- Do not implement code edits in this spec. This document defines proof and
  ranking contracts; application of edits remains a later phase.
- Do not require coverage or git history for static cleanup. Runtime and
  staleness are supporting evidence, not prerequisites.

## 4. Definitions

### 4.1 Deadness proof

Evidence that the export has no observed consumer in the constructed graph and
no relevant blind zone can plausibly hide one.

### 4.2 Contract proof

Evidence that the export is not part of a public or framework-consumed surface:
package exports, entrypoints, config files, route modules, declaration sidecars,
script entrypoints, HTML module entrypoints, or policy-specified public APIs.

### 4.3 Action-safety proof

Evidence that a concrete edit preserves runtime behavior, module syntax, and
public-contract expectations under the recorded scan range.

### 4.4 Local-use proof

Evidence about whether the local binding or type declaration is still required
inside its defining file, and whether the selected action preserves or removes
that binding/type.

External consumer proof and local-use proof are separate. A declaration can be
externally unused while still being locally required. In that case demotion may
be safe while deletion is not.

### 4.5 Proof packet

The structured evidence attached to each cleanup candidate:

```json
{
  "deadnessProof": {},
  "contractProof": {},
  "blindZoneProof": {},
  "localUseProof": {},
  "actionProof": {}
}
```

`SAFE_FIX` requires a complete proof packet and a selected safe action.

## 5. Tier Contract

PCEF does not remove the four public tiers. It tightens what drives them:

```text
MUTED
  Known false-positive family or public/framework/entry policy exclusion.

DEGRADED
  Deadness cannot be trusted because a blocking blind zone could hide a consumer.

REVIEW_FIX
  Deadness is plausible or clean, but no safe action proof exists, or only
  non-proving runtime evidence is present.

SAFE_FIX
  Deadness proof clean + contract proof clean + no relevant blind zone +
  concrete action-safety proof.
```

This means a candidate can be deadness-clean but still not `SAFE_FIX` if the
engine cannot name an edit that preserves behavior.

### 5.1 Runtime evidence and demotion

`runtime=executed` is a trust blocker for static deadness, not a side-effect
claim. A demotion can preserve local runtime behavior, but observed execution
means the scan cannot prove the export edge is unused under the recorded runtime
evidence. Such candidates remain `DEGRADED` unless a narrower future runtime
proof separates declaration execution from export consumption.

## 6. Safe Action Lattice

PCEF chooses the weakest safe edit first. Higher numbered edit actions are more
aggressive; `review` is a terminal no-safe-action result, not an edit.

| Order | Action | Meaning |
|---:|---|---|
| 1 | `remove_export_specifier` | Remove a single `export { x }` item while preserving local binding. |
| 2 | `demote_export_declaration` | Remove only the `export` keyword/default export edge. Keep declaration/runtime effects. Applies to value and type declarations. |
| 3 | `delete_type_declaration` | Delete TS-only declarations only when no local type consumer remains and the declaration is not part of declaration/public surface. |
| 4 | `delete_value_declaration` | Delete value declaration only when the local binding is unused and side-effect-safe. |
| 5 | `review` | No safe edit can be proven. |

The selected action must be stored on the ranked finding. The action, not the
bucket, is what makes a cleanup safe.

Demotion preserves runtime behavior, not necessarily public contract. Removing
an `export` keyword still removes an externally observable import target, so
demotion must pass contract proof and public deep-import gates.

## 7. Action Table

| Export form | Safe action | Delete allowed only when |
|---|---|---|
| `export { foo }` | remove specifier | local binding is also unused and its declaration is delete-safe |
| `export { foo as bar }` | remove specifier | same as above |
| `export { foo } from "./x"` | review by default in v1; later remove re-export specifier with source-evaluation preservation | source evaluation is preserved by default or proven unnecessary |
| `export { default as Foo } from "./x"` | review by default in v1; later same as value re-export specifier cleanup | follows source-evaluation preservation rules |
| `export type { Foo } from "./x"` | review by default in v1; later remove type re-export specifier | no downstream type consumer and contract proof clean; no runtime source evaluation preservation needed |
| `export { type Foo, bar } from "./x"` | review by default in v1; later remove only proven-dead specifiers | value specifier removal follows source-evaluation preservation rules; type-only specifier removal does not create runtime evaluation |
| `export * from "./x"` | review | star export has precise downstream binding proof |
| `export * as ns from "./x"` | review | namespace re-export requires precise downstream namespace proof and source-evaluation proof |
| `export type * from "./x"` | review by default | type star export requires precise downstream type binding proof |
| `export function f() {}` | demote export, then maybe delete | no decorators and no local value references require the binding |
| `export const x = 1` | demote export, then maybe delete | local binding is unused and initializer is pure |
| `export const x = sideEffect()` | demote export only by default | deletion is not allowed unless local binding is unused, the call/new is explicitly trusted as pure, and all arguments are pure |
| `export const x = /*#__PURE__*/ pureFactory()` | demote export, then maybe delete | local binding unused, annotated call trusted, and arguments are pure |
| `export class C {}` | demote export, then maybe delete | local binding unused, no decorator, no extends, no static block, no static field initializer, no computed member |
| `export interface T {}` | demote export, then maybe delete | no local type consumer, no declaration merging/augmentation risk, and not public/declaration sidecar |
| `export type T = ...` | demote export, then maybe delete | no local type consumer and not public/declaration sidecar |
| `export enum E` | demote first | deletion requires local binding/type-use proof and TS emit mode proof |
| `export default expr` | review by default | anonymous/default expression delete/demote is proven safe |
| `export default function f(){}` | review by default; demote only with explicit default-demotion proof | named binding, module syntax, local binding behavior, and public contract can be preserved |

## 8. Consumer Proof

A finding passes consumer proof only if all of these are true:

- No exact named/default import consumer.
- No exact re-export consumer.
- No namespace broad consumer.
- No dynamic import broad consumer.
- No CommonJS broad consumer.
- No unresolved internal specifier can plausibly resolve to the candidate file.
- Defining file parsed successfully.
- Candidate-relevant consumer scan slice has no parse error that could hide a
  consumer, including reverse import/re-export paths that could target the
  candidate file.
- Candidate is not excluded by public/framework/config/entry policy.

### 8.1 Exact namespace member reads

The extractor should treat static member reads as exact uses:

```ts
ns.foo;
ns.foo();
const x = ns.foo;
[ns.foo];
<ns.Foo />;
ns?.foo;
ns.foo?.();
ns.foo!;
typeof ns.foo;
const { foo } = ns;
const { foo: alias } = ns;
```

These protect only `foo`/`Foo`, not every export in the namespace.

The extractor must retain broad/opaque namespace evidence for:

```ts
ns[key];
const x = ns;
{ ...ns };
Object.keys(ns);
for (const key in ns) {}
```

Broad evidence blocks `SAFE_FIX` for siblings because the graph cannot prove
which export was used.

Namespace destructuring with only named properties may protect exact members.
Namespace rest destructuring protects named properties exactly but keeps broad
evidence for siblings:

```ts
const { foo, ...rest } = ns;
```

### 8.2 Dynamic import exact destructuring

Literal dynamic imports can contribute exact named uses:

```ts
const { foo } = await import("./x");
const { foo: bar } = await import("./x");
const mod = await import("./x");
mod.foo();
const { baz } = mod;
import("./x").then(({ foo }) => foo());
import("./x").then((mod) => mod.foo);
```

Non-literal imports stay opaque:

```ts
import(pathFromConfig);
```

Opaque dynamic imports are not false positives; they are proof blockers.

Dynamic import aliases are exact only while the alias is local and unescaped. If
the alias is returned, exported, spread, passed to an unknown call, or assigned
to an object with unknown consumers, the import stays broad/opaque.

### 8.3 Type-space and value-space consumers

Consumer proof must distinguish value-space, type-space, and dual-space
symbols.

- `import type { T }` protects exported type declarations and the type side of
  dual symbols such as classes and enums.
- Value imports protect value-space exports.
- A demotion that removes an exported symbol is blocked by either value-space or
  type-space external consumers for that symbol.
- Type-only re-export cleanup must not be treated as value re-export cleanup.

Example:

```ts
// a.ts
export class User {}

// b.ts
import type { User } from "./a";
```

The type-only import does not execute `a.ts`, but it is still an external
consumer of the exported `User` type contract. `demote_export_declaration` is
not safe while that consumer exists.

### 8.4 CommonJS consumers

JS/TS repositories often mix ESM and CommonJS. Consumer proof must include:

```js
const { foo } = require("./x");
const mod = require("./x");
mod.foo();
require("./x").foo();
const { bar } = mod;
exports.foo = require("./x").foo;
```

Plain `require("./x")` is a side-effect import or broad module consumer. It may
protect the module/file even when it does not protect every named export.

Separated aliases such as `const mod = require("./x"); const { foo } = mod;`
should become exact uses when the alias is local and unescaped. If the alias is
exported, assigned into another object, returned, or passed to an unknown call,
the module stays broad/opaque and blocks sibling `SAFE_FIX`.

## 9. Contract Proof

The candidate must not be part of:

- `package.json` `exports`, `main`, `module`, `browser`, `types`, or `bin`.
- Barrel chains that form package entrypoints.
- Framework route/config conventions.
- Script entrypoints from package manager scripts or known tool configs.
- HTML module scripts.
- Declaration sidecars (`.d.ts`, generated declarations, or declaration emit
  dependencies).
- JSDoc or local metadata that marks a symbol public.

### 9.1 Public deep-import risk

If a package is public (`private !== true`) and does not use package `exports`
to constrain deep imports, external users may import internal files:

```ts
import { thing } from "pkg/src/internal/file";
```

For such packages, `SAFE_FIX` must be blocked unless one of these is true:

- The candidate is inside a workspace/package explicitly marked internal-only.
- A future config explicitly enables deep-import cleanup for that package.

Default: public deep-import risk blocks both demotion and deletion. Demotion
preserves local runtime behavior, but it still removes an externally observable
export contract. This gate prevents local graph cleanup from silently breaking
package consumers.

## 10. Blind-Zone Proof

Blind zones are classified by relevance to the candidate, not by repo-wide
presence alone.

### 10.1 Blocking taint

Blocks `SAFE_FIX` and usually produces `DEGRADED`:

- Defining file parse error.
- Unresolved internal specifier that could resolve to candidate file.
- Candidate-relevant consumer scan slice parse error that could hide a
  consumer.
- Opaque dynamic import with static directory overlap to candidate file.
- Broad namespace or CommonJS consumer that could include candidate export.

### 10.2 Soft taint

Does not automatically block `SAFE_FIX`; must be narrowed to relevance:

- Parse error in unrelated file.
- Repo-global unresolved ratio increase with no candidate-file match.
- Large absolute unresolved internal import count with no candidate-file match.
- Unresolved specifier prefix concentration with no candidate-file match.
- Unsupported language file outside the candidate package or relevant consumer
  scan slice.

Soft taint should become a blocking taint only after a matcher shows candidate
relevance.

### 10.3 Run-level precision warnings

Candidate-local taint and run-level precision warnings are separate.

A resolver gap that cannot be matched to a candidate should not automatically
block that candidate's `SAFE_FIX`, but it still must be visible to users when
the run contains a large unresolved area. Reporting must not rely only on
unresolved ratios; large monorepos can have a modest percentage and still hide
hundreds or thousands of missing import edges.

Emit a blind-zone or precision warning when either condition is true:

- unresolved internal imports exceed an absolute-count threshold, or
- one unresolved specifier prefix accounts for a large share of unresolved
  internal imports.

These warnings are not positive or negative proof for a specific finding. They
are run-level scope statements that keep "no consumer was found in the
constructed graph" distinct from "the whole repository was fully resolved."

## 11. Action-Safety Proof

Create an action-safety module in the implementation phase:

```text
_lib/export-action-safety.mjs
```

Suggested return shape:

```ts
type SafeAction =
  | {
      kind: "remove_export_specifier";
      actionGroupId: string;
      range: [number, number];
      preservesRuntime: true;
      preservesModuleSyntax: boolean;
      actionBlockers: string[];
      strongerActionBlockers: string[];
      proofComplete: boolean;
      patches?: SafePatch[];
      proof: string[];
    }
  | {
      kind: "demote_export_declaration";
      actionGroupId: string;
      exportKeywordRange: [number, number];
      preservesRuntime: true;
      preservesModuleSyntax: boolean;
      actionBlockers: string[];
      strongerActionBlockers: string[];
      proofComplete: boolean;
      patches?: SafePatch[];
      proof: string[];
    }
  | {
      kind: "delete_type_declaration";
      actionGroupId: string;
      declarationRange: [number, number];
      preservesRuntime: true;
      preservesModuleSyntax: boolean;
      actionBlockers: string[];
      strongerActionBlockers: string[];
      proofComplete: boolean;
      patches?: SafePatch[];
      proof: string[];
    }
  | {
      kind: "delete_value_declaration";
      actionGroupId: string;
      declarationRange: [number, number];
      preservesRuntime: true;
      preservesModuleSyntax: boolean;
      actionBlockers: string[];
      strongerActionBlockers: string[];
      proofComplete: boolean;
      patches?: SafePatch[];
      proof: string[];
    };

type SafePatch =
  | {
      kind: "preserve_side_effect_import";
      source: string;
      insertionRange: [number, number];
      statement: string;
      reason: "re-export source evaluation must be preserved";
    }
  | {
      kind: "remove_import_specifier";
      importRange: [number, number];
      specifierRange: [number, number];
      source: string;
      reason: "import binding is only used by deleted declarations";
    }
  | {
      kind: "remove_import_declaration";
      importRange: [number, number];
      source: string;
      reason: "import declaration is only used by deleted declarations";
    }
  | {
      kind: "insert_empty_export";
      insertionRange: [number, number];
      statement: "export {};";
      reason: "final import/export removed";
    };
```

An edit plan may contain multiple patches. For example, removing a re-export can
also require inserting `import "./x";`, and removing the last import/export can
also require inserting `export {};`.

Deletion safety is separate from export-edge deadness. A declaration may be
externally unused but still locally required. `delete_value_declaration` and
`delete_type_declaration` require local binding/type-use proof in addition to
consumer proof, contract proof, side-effect proof, and module-syntax proof.

For type/interface exports, demotion is the default weaker safe action when the
type remains locally referenced. Type declaration deletion requires proof that
the type has no external consumer, no local type consumer, no declaration
sidecar/public-surface role, and no declaration merging or augmentation risk.

Value re-export removal must preserve source module evaluation by default unless
the engine proves one of:

1. the source module is type-only in the emitted runtime path,
2. the source module is already evaluated by another remaining import/export in
   the same file, or
3. the source module is side-effect-free under the configured side-effect
   policy.

### 11.1 Action blockers

Action safety distinguishes blockers for the selected safe action from blockers
for stronger actions.

`actionBlockers` prevent the selected `safeAction` itself. Ranking must treat a
candidate with non-empty `actionBlockers` as not safe.

`strongerActionBlockers` explain why a stronger edit, especially declaration
deletion, is not allowed. Ranking must not use `strongerActionBlockers` to block
`SAFE_FIX` when the selected weaker action is safe.

Example: `export const x = registerTelemetry()` may block
`delete_value_declaration` because the initializer may have side effects, but it
can still be `SAFE_FIX` with `demote_export_declaration` when deadness and
contract proof are clean.

### 11.2 Action groups

`safeAction` should include `actionGroupId` when multiple findings can produce
the same edit. Fix-plan rendering must dedupe edits by `actionGroupId` and then
perform range-conflict checks before presenting or applying a plan.

### 11.3 Conservative purity

Initial `delete_value_declaration` support must be narrow.

Pure enough for deletion:

- Literals.
- Object/array literals whose children are pure.
- Function expressions.
- Arrow functions.
- Class expressions with no static effects.
- Type-only declarations.

Unsafe unless proven otherwise:

- Call expression.
- `new`.
- `await` / `yield`.
- Assignment/update.
- Member access with possible getter.
- Computed property.
- Tagged template.
- Dynamic import.

Pure annotations such as `/*#__PURE__*/` may support call/new deletion, but
only for the annotated call itself. They do not make argument expressions pure.

### 11.4 Module syntax preservation

Removing the final import/export can change a file from module to script.
Safe actions must preserve module syntax when that matters.

Allowed preservation strategy:

```ts
export {};
```

The proof packet must record whether this marker is needed.

When marker insertion is needed, the action must include:

```ts
{
  kind: "insert_empty_export",
  insertionRange: [number, number],
  statement: "export {};",
  reason: "final import/export removed"
}
```

`.mts`/`.cts` files and project emit settings may constrain whether an empty
export marker is appropriate. If the engine cannot prove the marker is valid for
the file's module mode, the candidate stays `REVIEW_FIX`.

### 11.5 Post-delete import integrity

Declaration deletion can make imports unnecessary even when deadness and local
binding proof are clean. That cleanup is part of action safety, not cosmetic
formatting.

For `delete_type_declaration` and `delete_value_declaration`, the selected
action must evaluate imported bindings after the whole `actionGroupId` edit set
is deduped:

- If an import binding is still used outside the deleted ranges, leave it.
- If an import binding is used only by deleted ranges and the import can be
  safely edited, include `remove_import_specifier` or
  `remove_import_declaration` patch evidence.
- If removing the import would stop required source-module evaluation, preserve
  that evaluation with a side-effect import or block the deletion.
- If removing the final import/export would change module mode, include the
  `insert_empty_export` marker patch from §11.4.
- If the engine cannot prove a safe import cleanup or safe intentional retention
  under the package's type/lint constraints, the stronger delete action is not
  allowed. Prefer a weaker demotion action when available; otherwise keep the
  finding `REVIEW_FIX`.

This rule is evaluated at action-group level. Two findings that delete the only
two declarations using `import type { Linter } from "eslint"` must be checked
together before deciding whether the import remains, is removed, or requires a
module marker.

## 12. Ranking Integration

Current `ranking.mjs` should eventually move from bucket-based promotion to
action-proof promotion.

Target predicate:

```js
if (policyExcluded) return MUTED;
if (blockingBlindZone) return DEGRADED;

const hasSafeAction =
  !!finding.safeAction?.kind &&
  finding.safeAction.actionBlockers.length === 0 &&
  finding.safeAction.proofComplete === true;

if (!hasSafeAction) return REVIEW_FIX;
if (declarationExportDependency && !isBindingPreservingAction(finding.safeAction)) {
  return REVIEW_FIX;
}
if (!deadnessClean || !contractClean) return REVIEW_FIX;

if (hasEntryReachSupport && hasIndependentSupport) {
  return {
    tier: "SAFE_FIX",
    confidence: "high",
    reason:
      "safe-action + clean-deadness + entry-unreachable + no-observed-callers"
  };
}

if (hasEntryReachSupport || hasIndependentSupport) {
  return {
    tier: "SAFE_FIX",
    confidence: "medium",
    confidenceDetail: "medium_with_evidence",
    reason: "safe-action + clean-deadness + single-lens evidence"
  };
}

return {
  tier: "SAFE_FIX",
  confidence: "medium",
  reason: "safe-action + clean-deadness"
};
```

Buckets remain useful as classifier facts, but they do not by themselves
authorize `SAFE_FIX`.

Updated bucket meaning:

```text
C = no external consumer observed.
A = external export edge removable while local use remains.
specifier = export specifier can be removed.
B = design/predicate judgment; review required.
```

`B` remains review-by-default. The narrow exception is a local type declaration
dependency where action safety proves a binding-preserving edit such as
`demote_export_declaration`. In that case the local declaration binding remains
available to exported declarations, while only the unused export edge is
removed.

The selected `safeAction.kind` determines edit safety.

### 12.1 Confidence

`SAFE_FIX` confidence describes supporting evidence strength, not edit safety.

- `medium`: safe action + clean deadness + clean contract proof.
- `medium_with_evidence`: `medium` plus one positive evidence lens, such as
  entry-unreachable or no-observed-callers.
- `high`: `medium` plus at least two compatible positive evidence lenses, such
  as entry-unreachable and call-graph no-observed-callers.

Positive evidence must not override relevant taint. If deadness proof is not
clean, the candidate is not `SAFE_FIX` regardless of confidence boosters.

### 12.2 Supported evidence and annotations

Positive evidence used by ranking must live in `supportedBy`.

Review hints that do not prove deadness or action safety must live in
`annotations`. Ranking must not read `annotations`.

Examples:

- `supportedBy.entry-unreachable`
- `supportedBy.call-graph-no-observed-callers`
- `annotations.cloneGroup`

## 13. Artifact Shape

Each ranked finding should be able to carry:

```json
{
  "finding": {
    "file": "src/a.ts",
    "symbol": "foo",
    "bucket": "A"
  },
  "tier": "SAFE_FIX",
  "supportedBy": ["entry-unreachable"],
  "annotations": {
    "cloneGroup": { "id": "cg-17", "similarity": 0.91 }
  },
  "action": {
    "kind": "demote_export_declaration",
    "actionGroupId": "src/a.ts:export-decl:120-180",
    "preservesRuntime": true,
    "preservesModuleSyntax": true,
    "actionBlockers": [],
    "strongerActionBlockers": ["side-effect-initializer"],
    "proofComplete": true,
    "patches": []
  },
  "proof": {
    "deadnessProof": {
      "noExactConsumers": true,
      "noBroadConsumers": true,
      "unresolvedSpecifiersCouldMatch": []
    },
    "contractProof": {
      "publicSurface": false,
      "frameworkSurface": false,
      "deepImportRisk": "not_applicable"
    },
    "blindZoneProof": {
      "blocking": [],
      "soft": []
    },
    "localUseProof": {
      "localValueReferences": ["src/a.ts::foo"],
      "localTypeReferences": [],
      "bindingPreserved": true,
      "deletionAllowed": false
    },
    "actionProof": {
      "selectedAction": "demote_export_declaration",
      "sideEffectProof": ["initializer preserved"],
      "moduleSyntaxProof": ["remaining import exists"]
    }
  }
}
```

The artifact should be explicit enough that a model can explain why the fix is
safe without inventing evidence.

## 14. Metrics

Do not measure only `SAFE_FIX / total findings`. That rewards noisy tools.

Required metrics:

```text
SafeFixDiscoveryRate =
  SAFE_FIX /
  (SAFE_FIX + REVIEW_FIX where deadness proof is clean)
```

Also record review blockers:

```text
review_reason.namespace_broad
review_reason.dynamic_import_opaque
review_reason.cjs_gap
review_reason.public_contract_unknown
review_reason.deep_import_risk
review_reason.local_value_use
review_reason.local_type_use
review_reason.action_delete_unsafe
review_reason.post_delete_import_cleanup_unknown
review_reason.action_missing
review_reason.parse_taint
review_reason.resolver_taint
```

Implementation work should target the largest review blocker first.

## 15. Acceptance Criteria

PCEF implementation is not ready until all are true:

- Accepted calibration corpus false-positive budget remains zero.
- A calibration corpus produces non-empty `SAFE_FIX`.
- `export const x = sideEffect()` ranks `SAFE_FIX` with
  `demote_export_declaration`, not `delete_value_declaration`.
- `export const x = 1` can rank `delete_value_declaration` only with pure
  initializer proof and local-binding-unused proof.
- A locally used value export ranks at most `demote_export_declaration`, not
  `delete_value_declaration`.
- A locally used type/interface export ranks at most
  `demote_export_declaration`, not `delete_type_declaration`.
- Type/interface deletion requires no local type consumer, no
  declaration-sidecar/public role, and no declaration merging or augmentation
  risk.
- Declaration deletion either preserves, removes, or intentionally retains
  imports made unused by the deleted declarations, with proof recorded in the
  action packet.
- Type-space and value-space consumers are distinct, and either one can block
  demotion of an exported dual-space symbol.
- Type-only re-export cleanup does not create runtime source evaluation, while
  value re-export cleanup preserves source evaluation by default.
- `runtime=executed` still forces `DEGRADED`.
- Explicit `runtime=uncovered` or `runtime=type-only` remains `REVIEW_FIX`
  unless another proof explicitly authorizes a non-runtime action.
- Namespace exact-member reads protect only named members.
- Namespace reflection/broad use blocks sibling `SAFE_FIX`.
- Literal dynamic import destructuring protects exact exports.
- Local, unescaped dynamic import aliases protect exact member reads.
- Non-literal dynamic import remains opaque.
- CommonJS exact consumers protect named exports.
- Public deep-import risk blocks public-package `SAFE_FIX` unless package is
  explicitly internal-only or constrained by exports.
- Public deep-import risk blocks demotion as well as deletion by default.
- Large absolute unresolved internal import counts or concentrated unresolved
  workspace/alias prefixes surface as run-level precision warnings even when
  their global unresolved ratio is below a normal confidence-gap threshold.
- Re-export cleanup explicitly preserves source module evaluation when removing
  the re-export would otherwise stop evaluating the source module.
- Module syntax preservation is proven or recorded.
- Exploratory third-party corpus reports deadness false positives and action
  false positives separately before broader release behavior is enabled.

## 16. Implementation Phases

Detailed implementation sequencing lives in
`docs/spec/proof-carrying-export-fix-implementation-plan.md`.

### P0: Candidate-deadness cleanup

- Narrow repo-global soft taint to candidate-relevant scope.
- Replace boolean unresolved-spec matching with tri-state
  match/no-match/unknown.
- Add CommonJS exact and broad consumer extraction.
- P0 restores the clean-deadness candidate pool, but PCEF `SAFE_FIX` promotion
  still requires P1 safeAction proof.

### P1: Action proof without edit application

- Add `export-action-safety` producer.
- Attach `safeAction`, `actionBlockers`, `strongerActionBlockers`, and
  `actionGroupId`.
- Update ranking to depend on `safeAction`, not mechanical bucket.
- No source editing.

### P2: Reachability evidence as booster

- Add resolved internal file edges.
- Add an entry-surface artifact with submodule completeness.
- Add a module-reachability artifact.
- Use entry-unreachable only as confidence support, never as direct promotion
  over taint.

### P3: Independent call-graph evidence

- Add canonical definition ids.
- Add full fan-in maps not limited by display truncation.
- Add bounded member-call resolution.
- Promote `SAFE_FIX(high)` only when clean deadness + safeAction has at least
  two compatible positive evidence lenses.

### P4: Calibration and release gating

- Measure deadness false positives and action false positives separately.
- Preserve zero false positives on accepted calibration corpus.
- Keep maintainer calibration notes for external repositories when they expose
  new proof obligations.
- Run third-party corpus sampling before enabling broader release behavior.

### P5: Optional fixer

- Apply proof-carrying safe actions only after proof packets, ranking, and
  calibration are stable.

## 17. External Reference Notes

The design borrows concepts, not policy thresholds:

- Knip unused exports:
  <https://knip.dev/typescript/unused-exports>
- Knip namespace imports:
  <https://knip.dev/guides/namespace-imports>
- Knip CommonJS conventions:
  <https://knip.dev/guides/working-with-commonjs>
- Knip handling issues:
  <https://knip.dev/guides/handling-issues>
- OXC dead-code elimination and pure annotations:
  <https://oxc.rs/docs/guide/usage/minifier/dead-code-elimination>
- OXC AST architecture:
  <https://oxc.rs/docs/contribute/parser/ast>
- TypeScript compiler options:
  <https://www.typescriptlang.org/docs/handbook/compiler-options.html>

Knip uses project entrypoints, framework/tool plugins, namespace handling, and
CommonJS support. PCEF should learn from these coverage ideas but keep stricter
`SAFE_FIX` gates. OXC DCE documents side-effect controls and pure annotations;
PCEF uses that separation to distinguish export demotion from declaration
deletion. TypeScript unused-local checks are not a substitute for package-level
export graph analysis.

## 18. Open Questions

- Which explicit default-demotion proof is sufficient for
  `export default function named(){}`? Initial default: review.
- Should pure annotations be trusted by default, or gated behind a
  `trustPureAnnotations` flag? Initial default: trusted only as supporting proof,
  never as sole proof if arguments are impure.
- Should type-only deletion require checker-grade validation in `.d.ts` emit
  contexts? Initial default: yes for public packages, no for private internal
  packages after contract proof.

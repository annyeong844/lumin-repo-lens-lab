# WT-SFC Svelte Action Directive Fixture Inventory

Date: 2026-05-29

## Decision

Decision tokens:
`svelte-action-directive-evidence-before-compiler-semantics`,
`explicit-action-binding-required`, `framework-convention-stays-muted`,
`scan-gap-stays`.

Svelte `use:action` directives are framework convention evidence, not import
graph proof. P1 records only directives whose action name resolves to an
explicit imported action or a local function binding already visible to the SFC
parser. The evidence remains muted and review-only.

## Surface

Records use `symbols.json.sfcFrameworkConventionComponents[]` with:

- `framework: "svelte"`;
- `conventionKind: "action-directive"`;
- `source` / `reason: "sfc-framework-svelte-action-directive"`;
- `confidence: "framework-convention-observed"`;
- `status: "muted"`;
- `eligibleForFanIn: false`;
- `eligibleForSafeFix: false`.

The record may include `consumerFile`, `tagName`, `directiveName`, `actionName`,
`bindingName`, `bindingSource`, `fromSpec`, `bindingKind`, `importedName`,
`line`, and `sfcBlockKind`.

## Accepted Shape

Fixture:

```svelte
<script context="module" lang="ts">
  import { enhance } from "../src/svelte-action";
</script>

<script lang="ts">
  function localAction(node) {
    return { destroy() {} };
  }

  const localConstAction = (node) => ({ destroy() {} });
</script>

<form use:enhance></form>
<div use:localAction></div>
<section use:localConstAction></section>
```

Expected:

- one muted record with `directiveName: "use:enhance"`;
- `actionName` / `bindingName: "enhance"`;
- `bindingSource: "../src/svelte-action"`;
- local muted records for `use:localAction` and `use:localConstAction`;
- local records use the SFC file as `bindingSource` and report
  `bindingKind: "local-function"` or `bindingKind: "local-const-function"`;
- no `resolvedInternalEdges[]` entry from this evidence.

## Rejected Shapes

Fixture shapes:

```svelte
<script lang="ts">
  const notActionValue = 1;
</script>

<button use:missingAction></button>
<button use:notActionValue></button>
<!-- <div use:commentAction></div> -->
```

Expected:

- no record for unbound `missingAction`;
- no record for a local non-function value such as `notActionValue`;
- no record for comment-only markup;
- no synthetic target, no unresolved component-style diagnostic, and no graph
  edge.

## Safety Contract

This lane must not enter `resolvedInternalEdges[]`, named export fan-in,
deadness, `SAFE_FIX`, `EXISTS`, fix-plan, export-action-safety, package edits,
or default action lanes. The `sfc-scan-gap` blind zone remains visible because
Svelte compiler behavior, `$store` auto-subscription, and runtime action effects
are still not modeled.

## Test Anchors

- [`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs)
- [`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs)

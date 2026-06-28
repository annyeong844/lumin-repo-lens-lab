# WT-SFC Astro Client Directive Fixture Inventory

This note selects the next narrow WT-SFC framework-magic lane from
[`wt-sfc-framework-magic-fixture-inventory-2026-05-27.md`](wt-sfc-framework-magic-fixture-inventory-2026-05-27.md):
Astro `client:*` directives on explicitly imported components.

This is an implementation inventory, not a broad Astro semantics claim. The
lane records that an Astro template attached a client hydration directive to an
explicit frontmatter component binding. It does not prove template consumption,
component target reachability, named export fan-in, or deadness.

## Decision

Decision: `astro-client-directive-before-astro-integration-inference`,
`explicit-binding-or-no-client-directive-record`, and
`client-directive-evidence-stays-review-only`.

Astro `client:*` directives are visible syntax, but their runtime behavior is
framework-owned hydration. The safe first slice is therefore muted evidence in
`symbols.json.sfcFrameworkConventionComponents[]`, sharing the review-only
framework convention surface used by Nuxt filesystem convention and
`unplugin-vue-components` config evidence.

## Accepted Shape

P1 accepts only this shape:

```astro
---
import { UsedByAstro } from "../src/astro-use";
---

<UsedByAstro client:load />
```

Requirements:

- file extension is `.astro`;
- the template tag is a component tag resolved through the existing
  frontmatter/import binding model;
- the tag has a literal Astro client directive attribute whose name starts with
  `client:`;
- the directive is recorded as muted review evidence with
  `reason: "sfc-framework-astro-client-directive"`;
- the record keeps the tag name, directive name, binding name, binding source,
  container file, and line.

## Rejected Shapes

P1 must not create records for:

- tags without a `client:*` directive;
- native/custom lowercase HTML tags;
- missing or unresolved component bindings;
- namespace component tags such as `<UI.Card client:load />`;
- dynamic component expressions;
- compiler/runtime integration-injected components.

These shapes remain covered by `sfcTemplateComponentRefs[]` when applicable, or
by the broader `sfc-scan-gap` when not observable.

## Review-Only Contract

Astro client directive evidence must stay review-only:

1. use `symbols.json.sfcFrameworkConventionComponents[]`;
2. emit `eligibleForFanIn: false` and `eligibleForSafeFix: false`;
3. never enter `resolvedInternalEdges[]`, named export fan-in, deadness,
   `SAFE_FIX`, `EXISTS`, fix-plan, export-action-safety, or package edits;
4. keep the grouped `sfc-scan-gap` visible;
5. do not infer component targets beyond the explicit binding metadata already
   visible in the SFC.

## Fixture Matrix

| #   | Fixture                                                  | Expected                                                                                          |
| --- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| F1  | `.astro` frontmatter import + `<Used client:load>`       | One muted `sfc-framework-astro-client-directive` record with tag, directive, binding, and source. |
| F2  | Same tag already present in `sfcTemplateComponentRefs[]` | Both surfaces may record evidence, but neither creates graph/fan-in/action-lane entries.          |
| F3  | Template fake import string                              | No directive record.                                                                              |
| F4  | Lowercase/native tag with `client:*`                     | No directive record.                                                                              |
| F5  | Missing binding with `client:*`                          | No framework convention record; missing binding stays in the template-ref lane if applicable.     |
| F6  | Runtime public-install verification                      | Installed beta must prove the record is present and review-only.                                  |

## Decision Tokens

`astro-client-directive-fixture-inventory`,
`client-directive-review-evidence-not-consumption`,
`explicit-binding-or-no-client-directive-record`, and `scan-gap-stays`.

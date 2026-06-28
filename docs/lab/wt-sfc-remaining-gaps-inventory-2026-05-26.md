# WT-SFC Remaining Gaps Inventory

> Historical note: this beta.67-era inventory is no longer the current WT-SFC
> status record. Use
> [`wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md`](wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md)
> for the beta.78-era accepted surface and remaining gaps.

This note records the remaining WT-SFC gaps after the beta.67 public-install
verification of template component target evidence:
[`wt-sfc-beta67-template-component-target-verification-2026-05-26.md`](wt-sfc-beta67-template-component-target-verification-2026-05-26.md).

It is an inventory, not an implementation plan. The purpose is to keep the next
SFC slice from accidentally treating broad framework semantics as already
understood.

## Current Accepted Surface

The current WT-SFC MVP is intentionally narrow:

- `.vue`, `.svelte`, and `.astro` files are counted and keep one grouped
  `sfc-scan-gap` blind zone;
- inline Vue/Svelte scripts and Astro frontmatter static imports feed ordinary
  source import evidence;
- literal relative Vue/Svelte `<script src>` creates file reachability, not
  named export fan-in;
- literal relative style `url()` and `@import` references create isolated asset
  evidence, not graph edges;
- explicit template component bindings create review-only
  `symbols.json.sfcTemplateComponentRefs[]` evidence;
- SFC-to-SFC template component targets stay `muted` as
  `sfc-template-component-non-source-binding` while preserving `resolvedFile`
  for reviewer navigation.

The source guards are
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs). The
policy boundary is
[`sfc-support-policy.md`](../spec/sfc-support-policy.md).

## Remaining Gaps

### Component Binding Sources

These can create real runtime component references but are not fully modeled:

- Vue global registration through `app.component(...)`;
- Vue plugin registration and app-level component installation;
- framework auto-import plugins such as `unplugin-vue-components`;
- Nuxt-style file-system component auto-registration;
- Svelte component imports hidden behind generated route/layout conventions;
- Astro integration-driven component availability.

### Vue Registration Shapes

The current surface handles narrow explicit bindings. It does not prove all
Vue registration forms:

- spread registrations inside `components: { ...components }`;
- computed component names;
- registration through variables imported from elsewhere;
- mixin/extends-provided component registration;
- non-literal Options API `components` objects;
- macro or compiler transform registration outside ordinary script syntax.

### Template Reference Forms

These forms are intentionally weak or absent:

- dynamic `<component :is="...">` values beyond review-only muted evidence;
- namespace member tags such as `<UI.Card />`;
- framework/native/custom elements that may be globally valid without an
  explicit local binding;
- template-only prop, event, slot, action, directive, and member-use semantics;
- conditionally rendered component names assembled at runtime.

### SFC Container Semantics

The following remain outside deadness and fan-in claims:

- Svelte stores, actions, transitions, and compiler-owned reactivity;
- Vue macros and compiler-owned bindings beyond ordinary imports;
- Astro island/client directives beyond frontmatter imports;
- style preprocessor dependency semantics;
- generated virtual SFC build output and route manifests.

## Required Gates Before The Next SFC Slice

Any future SFC lane must define:

1. the exact source syntax it accepts;
2. stable muted/unresolved reason codes for every rejected shape;
3. Node and Vitest fixture coverage;
4. runtime public-install verification;
5. an explicit statement that the lane does or does not affect graph edges,
   named export fan-in, deadness, `SAFE_FIX`, `EXISTS`, package edits, and
   default action lanes;
6. whether `sfc-scan-gap` remains visible.

## Non-Goals

Do not remove `sfc-scan-gap` from the current MVP. That would reward narrow
evidence with a broad absence claim: every new lane proves a little more, not
everything.

Do not promote template component references into named export fan-in until a
future contract proves which symbol identity is consumed. File navigation
evidence is not symbol consumption.

## Decision

Decision: `remaining-gaps-inventory-before-next-sfc-lane` and
`scan-gap-stays-until-framework-semantics-are-proven`.

WT-SFC remains `MVP`, not `DONE`. The next SFC implementation slice should
choose one gap from this inventory, add a fixture inventory first, and keep the
review-only boundary unless the spec proves a stronger claim.

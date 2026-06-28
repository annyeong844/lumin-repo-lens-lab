# WT-SFC MVP Status And Remaining Gaps

This note records the current WT-SFC MVP boundary after the beta.78 public
verification of the audit brief surface:
[`wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md`](wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md).
It was updated after the beta.79 SvelteKit local action regression:
[`wt-sfc-beta79-sveltekit-local-action-regression-2026-05-31.md`](wt-sfc-beta79-sveltekit-local-action-regression-2026-05-31.md).
It was refreshed again after the Nuxt resolver/config follow-up lanes through
module package config evidence. Those lanes are tracked in the Nuxt app-dir and
custom resolver follow-up inventory:
[`wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md`](wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md).
The first beta.85 Nuxt corpus calibration is recorded in
[`wt-sfc-nuxt-main-corpus-calibration-2026-06-01.md`](wt-sfc-nuxt-main-corpus-calibration-2026-06-01.md).
The beta.86 public verification closes the `componentNames` helper-export
false positive from that calibration:
[`wt-sfc-beta86-nuxt-alias-helper-filter-verification-2026-06-02.md`](wt-sfc-beta86-nuxt-alias-helper-filter-verification-2026-06-02.md).
The policy contract lives in
[`sfc-support-policy.md`](../spec/sfc-support-policy.md).

It supersedes the older beta.67-era gap inventory
[`wt-sfc-remaining-gaps-inventory-2026-05-26.md`](wt-sfc-remaining-gaps-inventory-2026-05-26.md)
for current status. The older note remains useful as historical design
context, but several items from that inventory have since shipped as
review-only SFC evidence lanes.

## Current MVP Surface

WT-SFC is `MVP`, not `DONE`. The accepted surface is useful, but deliberately
bounded:

- SFC file presence is counted and still creates the `sfc-scan-gap` absence
  guard when `.vue`, `.svelte`, or `.astro` files are present
  ([policy](../spec/sfc-support-policy.md#current-contract)).
- Vue/Svelte inline script imports and Astro frontmatter imports feed ordinary
  import consumers and fan-in when they are static imports
  ([beta.63](wt-sfc-beta63-script-import-consumers-verification-2026-05-25.md)).
- Literal relative Vue/Svelte `<script src>` creates file reachability, not
  named export fan-in
  ([beta.64](wt-sfc-beta64-script-src-reachability-verification-2026-05-25.md)).
- Literal relative style `url()` and `@import` references create isolated style
  asset evidence, not graph edges or symbol fan-in
  ([beta.65](wt-sfc-beta65-style-assets-verification-2026-05-26.md)).
- Explicit template component bindings create review-only
  `symbols.json.sfcTemplateComponentRefs[]`; SFC-to-SFC targets stay muted while
  preserving `resolvedFile`
  ([beta.67](wt-sfc-beta67-template-component-target-verification-2026-05-26.md)).
- Explicit Vue global/app/plugin registration creates review-only
  `symbols.json.sfcGlobalComponentRegistrations[]`; async factories and
  duplicate registrations stay muted when the evidence is weak
  ([beta.68](wt-sfc-beta68-global-component-registration-verification-2026-05-27.md),
  [global registration P2 inventory](wt-sfc-global-registration-p2-fixture-inventory-2026-05-30.md)).
- Generated component manifests, Nuxt root/app filesystem conventions, Nuxt
  `#components` aliases, Nuxt literal component-dir config, unplugin config
  evidence, Astro `client:*`, Svelte `use:action`, Svelte `$store`
  auto-subscription evidence, Vue
  `defineOptions({ components })`, and Vue Options API `components` evidence
  are review-only framework convention or availability evidence
  ([framework magic inventory](wt-sfc-framework-magic-fixture-inventory-2026-05-27.md),
  [Nuxt resolver inventory](wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md),
  [beta.71](wt-sfc-beta71-nuxt-convention-verification-2026-05-28.md),
  [beta.80](wt-sfc-beta80-nuxt-app-dir-verification-2026-05-31.md),
  [beta.81](wt-sfc-beta81-nuxt-components-alias-verification-2026-05-31.md),
  [beta.72](wt-sfc-beta72-unplugin-config-verification-2026-05-28.md),
  [beta.73](wt-sfc-beta73-astro-client-directive-verification-2026-05-28.md),
  [beta.74](wt-sfc-beta74-svelte-action-directive-verification-2026-05-28.md),
  [beta.79 SvelteKit regression](wt-sfc-beta79-sveltekit-local-action-regression-2026-05-31.md),
  [SFC source guards](../../tests/test-sfc-consumers.mjs),
  [SFC Vitest guards](../../tests/sfc-consumers.test.mjs),
  [beta.75](wt-sfc-beta75-vue-macro-registration-verification-2026-05-28.md),
  [beta.76](wt-sfc-beta76-vue-options-registration-verification-2026-05-29.md)).
- Nuxt custom resolver hooks, layer `extends`, and `modules` package config now
  create high-level `status: "unavailable"` review-only records when the config
  shape is observed. They disclose that per-component facts are not statically
  available in this lane; they do not execute hooks, merge layers, run modules,
  or infer component targets
  ([Nuxt resolver inventory](wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md),
  [policy](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
  [source guards](../../tests/test-sfc-consumers.mjs),
  [Vitest guards](../../tests/sfc-consumers.test.mjs)).
- `manifest.json.sfcEvidence`, `audit-summary.latest.md`, and
  `audit-review-pack.latest.md` surface only shallow counts and pointers, not
  raw SFC record payloads
  ([beta.78](wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md)).
- Nuxt corpus calibration confirms the current MVP remains useful but not
  complete: beta.85 kept graph/action/Markdown surfaces clean on `nuxt-main`;
  beta.86 filters the `componentNames` helper export while preserving
  component-like `#components` alias records as review-only evidence
  ([Nuxt corpus calibration](wt-sfc-nuxt-main-corpus-calibration-2026-06-01.md),
  [beta.86 helper filter verification](wt-sfc-beta86-nuxt-alias-helper-filter-verification-2026-06-02.md)).

The source guard spine remains
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs),
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs),
[`tests/audit-repo-artifact-brief.test.mjs`](../../tests/audit-repo-artifact-brief.test.mjs),
[`tests/test-audit-manifest-export-surface.mjs`](../../tests/test-audit-manifest-export-surface.mjs),
and
[`tests/audit-manifest-export-surface.test.mjs`](../../tests/audit-manifest-export-surface.test.mjs).

## Remaining Gaps

These remain outside the MVP and must not be treated as proven:

- broad template semantics: props, events, slots, directives, dynamic
  components, namespace member tags, conditional rendering, and custom element
  policy;
- strong absence claims for framework conventions when generated manifests or
  explicit registrations are missing;
- custom resolver functions for Nuxt, unplugin, Vite, Webpack, Astro
  integrations, and user-defined component auto-import rules. The Nuxt subset
  is now bucketed in the
  [Nuxt app-dir/custom resolver inventory](wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md);
- Nuxt returned component lists, layer-merged component availability, module
  execution, and module-injected virtual registries. Nuxt `app/components/**`,
  `#components` imports, literal component-dir config, custom resolver hook
  presence, layer `extends`, and `modules` package config are covered only as
  review-only navigation or unavailable evidence, not as template consumption
  or absence proof;
- compiler/runtime magic such as Svelte semantics beyond fixture-pinned store
  auto-subscription syntax, Vue macro rewrites beyond fixture-pinned syntax,
  Astro integration injection, and virtual route/layout manifests;
- style preprocessor dependency semantics beyond literal relative asset
  references;
- promotion of review-only SFC evidence into named export fan-in, deadness,
  `SAFE_FIX`, `EXISTS`, package edits, fix-plan, export-action, or SARIF
  findings.

## Next Work Rule

The next WT-SFC slice should start from a fixture inventory, not from broad
framework inference. It must name accepted syntax, rejected shapes, reason
codes, Node and Vitest coverage, public-install verification, and whether
`sfc-scan-gap` remains visible.

## Decision

Decision: `wt-sfc-mvp-status-recorded`,
`sfc-current-surface-linked-to-policy-and-guards`,
`nuxt-unavailable-config-signals-recorded`,
`componentNames-helper-filter-public-verified`,
`svelte-store-subscription-stays-review-only`,
`remaining-gaps-stay-explicit`, and `wt-sfc-not-done`.

WT-SFC is ready to be used as a bounded MVP evidence surface. It is not ready
to remove `sfc-scan-gap` or make full-framework absence claims.

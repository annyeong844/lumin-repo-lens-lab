# Lumin Wiki Log

## [2026-06-02] implementation | WT-SFC Svelte store auto-subscription evidence

Recorded Svelte `$store` auto-subscription evidence as muted review-only SFC
framework-convention evidence. The policy boundary is
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
the current status is
[`WT-SFC MVP status`](../lab/wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md),
and the implementation guards are
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `svelte-store-subscription-stays-review-only`,
`explicit-store-binding-required`, `scan-gap-stays`, and
`no-action-surface`. Svelte `$store` syntax now produces
`sfc-framework-svelte-store-subscription` records only when the store name is
grounded in an explicit import or local `svelte/store` factory binding. Plain
text `$name`, comment-only markup, missing stores, and non-store local values
stay out. The lane does not create graph edges, fan-in, deadness, `SAFE_FIX`,
`EXISTS`, fix-plan, export-action, SARIF findings, package edits, or full
Svelte compiler-runtime proof.

## [2026-06-02] status | WT-SFC MVP boundary after beta.86

Refreshed the WT-SFC MVP status note after the beta.86 Nuxt `#components`
helper filter verification. The status note is
[`WT-SFC MVP Status And Remaining Gaps`](../lab/wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md),
the beta.86 runtime evidence is
[`WT-SFC beta.86 Nuxt alias helper filter verification`](../lab/wt-sfc-beta86-nuxt-alias-helper-filter-verification-2026-06-02.md),
and the earlier Nuxt corpus calibration is
[`WT-SFC Nuxt main corpus calibration`](../lab/wt-sfc-nuxt-main-corpus-calibration-2026-06-01.md).
The policy boundary remains
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
with source guards in
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `wt-sfc-mvp-status-refreshed-after-beta86`,
`componentNames-helper-filter-no-longer-open-gap`,
`remaining-gaps-stay-explicit`, `scan-gap-stays`, and `wt-sfc-not-done`.
The MVP status now treats the beta.85 `componentNames` helper-export finding as
closed by beta.86 while keeping full-framework absence claims and SFC evidence
promotion outside the MVP.

## [2026-06-02] verification | WT-SFC Nuxt #components helper filter public package

Verified beta.86 public-install behavior for the Nuxt `#components` helper
filter. The runtime note is
[`WT-SFC beta.86 Nuxt alias helper filter verification`](../lab/wt-sfc-beta86-nuxt-alias-helper-filter-verification-2026-06-02.md),
the source filter landed in
[`PR #599`](https://github.com/annyeong844/lumin_lab/pull/599), the beta.86
metadata landed in
[`PR #600`](https://github.com/annyeong844/lumin_lab/pull/600), and the
public package came from
[`1de657a`](https://github.com/annyeong844/lumin-repo-lens-lab/commit/1de657a6b11ab320375ce2b3c94f0a6075cb1338),
whose
[`Public Package CI`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26762576211)
passed on Node 20 and Node 22. The policy boundary remains
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
and the source guards remain
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `beta86-public-verified`,
`componentNames-helper-filter-public-verified`,
`component-like-alias-evidence-preserved`, `scan-gap-stays`, and
`no-action-surface`. The installed beta.86 audit of `nuxt-main` produced zero
`componentNames` hits in SFC convention evidence, SFC import lanes, action
lanes, SARIF, and default Markdown, while preserving seven unresolved
component-like `#components` alias records as review-only evidence. SARIF
reported `tool.driver.version: "0.9.0-beta.86"`.

## [2026-06-01] fix | WT-SFC Nuxt #components helper export filter

Filtered known Nuxt `#components` virtual helper exports from component alias
evidence. The calibration source is
[`WT-SFC Nuxt main corpus calibration`](../lab/wt-sfc-nuxt-main-corpus-calibration-2026-06-01.md),
the Nuxt boundary is
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md),
the policy contract is
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
and the source guards are
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `componentNames-is-helper-not-component`,
`nuxt-components-alias-helper-filtered`, `scan-gap-stays`, and
`no-action-surface`. Manifest-backed and unresolved component-like
`#components` imports remain review-only evidence; the virtual helper export
`componentNames` is ignored rather than recorded as a noisy component
diagnostic.

## [2026-06-01] calibration | WT-SFC Nuxt main corpus

Recorded the beta.85 Nuxt corpus calibration:
[`WT-SFC Nuxt main corpus calibration`](../lab/wt-sfc-nuxt-main-corpus-calibration-2026-06-01.md).
The run follows the
[`WT-SFC corpus calibration plan`](../lab/wt-sfc-corpus-calibration-plan-2026-05-31.md),
the
[`WT-SFC MVP status`](../lab/wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md),
and the
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md).

Decision: `keep-current-mvp`, `custom-resolver-still-gap`,
`scan-gap-stays`, `no-action-surface`, and
`tighten-nuxt-components-alias-helper-exports-before-stronger-wording`. The
beta.85 public package produced 51 SFC evidence records across 315 Vue SFCs in
`nuxt-main`, including 8 unresolved Nuxt `#components` alias records and 2
Nuxt module-package unavailable records. Graph, fan-in, deadness, strict
SFC-policy action, and default Markdown raw-payload checks stayed clean. The
review also found one harmless alias-lane false positive:
`componentNames` from `#components` is a virtual helper export, not a component
name, so stronger Nuxt alias wording should wait for a filter or a separate
classification.

## [2026-06-01] status | WT-SFC MVP boundary after Nuxt config lanes

Refreshed the current WT-SFC MVP boundary after the Nuxt resolver/config
follow-up lanes. The status record is
[`WT-SFC MVP status and remaining gaps`](../lab/wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md),
the contract is
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
the Nuxt matrix is
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md),
and the tracker summary is
[`Current WT-SFC Design Note`](../spec/lumin-work-tracker.md#current-wt-sfc-design-note).
The source guard spine remains
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `wt-sfc-not-done`,
`nuxt-unavailable-config-signals-recorded`, `scan-gap-stays`, and
`no-action-surface`. The current boundary separates graph-capable SFC script
imports, isolated review-only availability/navigation evidence, and Nuxt
`status: "unavailable"` config-shape disclosures for custom resolver hooks,
layer `extends`, and `modules`. Those unavailable records disclose that a
resolver/layer/module path exists; they do not execute framework code, infer
component targets, strengthen absence claims, or enter graph, fan-in, deadness,
`SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF, package-edit, or raw
Markdown surfaces.

## [2026-06-01] implementation | WT-SFC Nuxt module package unavailable evidence

Added source support for high-level Nuxt `modules` package config evidence in
`symbols.json.sfcFrameworkConventionComponents[]`. The implementation follows
the
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md)
and the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
with contract coverage in
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `module-package-stays-unavailable`, `scan-gap-stays`, and
`no-action-surface`. Literal, tuple-literal, and nonliteral Nuxt `modules`
config entries now produce
`sfc-framework-nuxt-module-package-unavailable` records with config-file and
config-property metadata only. Literal entries preserve the configured module
package string; nonliteral entries preserve only the unavailable signal. The
lane does not execute modules, infer module-provided component names, infer
target files, or enter graph, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan,
export-action, SARIF, package-edit, or raw Markdown surfaces.

## [2026-06-01] implementation | WT-SFC Nuxt layer extends unavailable evidence

Added source support for high-level Nuxt layer `extends` presence evidence in
`symbols.json.sfcFrameworkConventionComponents[]`. The implementation follows
the
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md)
and the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
with contract coverage in
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `layer-extends-stays-unavailable`, `scan-gap-stays`, and
`no-action-surface`. Literal and nonliteral Nuxt `extends` config entries now
produce `sfc-framework-nuxt-layer-extends-unavailable` records with config-file
and config-property metadata only. Literal entries preserve the configured
source string; nonliteral entries preserve only the unavailable signal. The
lane does not evaluate layers, infer component names, infer target files, or
enter graph, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, export-action,
SARIF, package-edit, or raw Markdown surfaces.

## [2026-06-01] implementation | WT-SFC Nuxt custom resolver unavailable evidence

Added source support for high-level Nuxt component hook presence evidence in
`symbols.json.sfcFrameworkConventionComponents[]`. The implementation follows
the
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md)
and the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
with contract coverage in
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `custom-resolver-stays-unavailable`, `scan-gap-stays`, and
`no-action-surface`. Literal Nuxt `hooks["components:dirs"]` and
`hooks["components:extend"]` entries now produce
`sfc-framework-nuxt-custom-resolver-unavailable` records with config-file and
hook-name metadata only. The lane does not execute hooks, infer component
names, infer target files, or enter graph, fan-in, deadness, `SAFE_FIX`,
`EXISTS`, fix-plan, export-action, SARIF, package-edit, or raw Markdown
surfaces.

## [2026-05-31] implementation | WT-SFC Nuxt literal component-dir config

Added source support for literal Nuxt component directory config evidence in
`symbols.json.sfcFrameworkConventionComponents[]`. The implementation follows
the
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md)
and the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
with contract coverage in
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `literal-component-dir-config-stays-directory-evidence`,
`no-component-target-inference`, `scan-gap-stays`, and `no-action-surface`.
Literal `components` / `components.dirs` entries now produce muted
`sfc-framework-nuxt-components-dir-config` records with config-file,
directory, prefix/path-prefix/global, and optional resolved-directory
navigation. Literal `~/...` and `@/...` paths resolve through Nuxt `srcDir`
semantics, including explicit `srcDir: "app/"` and the Nuxt 4 default `app/`
source directory. The lane does not scan configured directories into component
records and does not enter graph, fan-in, deadness, `SAFE_FIX`, `EXISTS`,
fix-plan, export-action, SARIF, package-edit, or raw Markdown surfaces.

## [2026-05-31] verification | WT-SFC beta.81 Nuxt `#components` alias

Recorded the beta.81 public install verification:
[`wt-sfc-beta81-nuxt-components-alias-verification-2026-05-31.md`](../lab/wt-sfc-beta81-nuxt-components-alias-verification-2026-05-31.md).
It verifies the implementation described by the
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md),
the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
and the source regression coverage in
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `nuxt-components-alias-public-verified`,
`generated-manifest-backed-alias-stays-muted`,
`unmapped-alias-stays-unresolved`, `scan-gap-stays`, and
`no-action-surface`. The beta.81 installed package records manifest-backed
Nuxt `#components` imports as muted
`sfc-framework-nuxt-components-alias` evidence with `resolvedFile`
navigation, records unmapped alias imports as unresolved advisory evidence
without guessing a target, and keeps dependency, unresolved-internal, graph,
fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF,
package-edit, and raw Markdown lanes clean.

## [2026-05-31] implementation | WT-SFC Nuxt `#components` alias evidence

Added source support for static SFC script imports from Nuxt `#components` when
the imported name is backed by `.nuxt/components.d.ts`. The implementation
follows the
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md)
and the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
with contract coverage in
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `nuxt-components-alias-requires-generated-manifest`,
`generated-manifest-before-alias-resolution`, `scan-gap-stays`, and
`no-action-surface`. Manifest-backed alias imports now produce review-only
`sfc-framework-nuxt-components-alias` evidence with manifest navigation and
`resolvedFile`; alias imports without a mapping produce
`sfc-framework-nuxt-components-alias-unresolved` without guessing a target.
Dependency, graph, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan,
export-action, SARIF, package-edit, and raw Markdown surfaces stay out of this
lane.

## [2026-05-31] verification | WT-SFC beta.80 Nuxt app-dir convention

Recorded the beta.80 public install verification:
[`wt-sfc-beta80-nuxt-app-dir-verification-2026-05-31.md`](../lab/wt-sfc-beta80-nuxt-app-dir-verification-2026-05-31.md).
It verifies the implementation described by the
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md)
and the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).

Decision: `nuxt-app-dir-public-verified`,
`nuxt-app-dir-requires-app-src-signal`,
`nuxt3-dependency-only-does-not-emit-app-dir`, `scan-gap-stays`, and
`no-action-surface`. The beta.80 installed package emits muted
`sfc-framework-nuxt-app-dir-convention` evidence for a Nuxt 4/app `srcDir`
fixture and emits zero app-dir records for a Nuxt 3 dependency-only fixture.
Graph, fan-in, deadness, fix-plan, export-action, SARIF, package-edit, and raw
Markdown lanes stayed clean.

## [2026-05-31] implementation | WT-SFC Nuxt app-dir convention evidence

Added source support for Nuxt `app/components/**` convention evidence in the
same review-only SFC framework-convention surface. The implementation follows
the
[`Nuxt app-dir/custom resolver inventory`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md)
and the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).

Decision: `nuxt-app-dir-convention-stays-muted`,
`nuxt-app-dir-requires-app-src-signal`, `scan-gap-stays`, and
`no-action-surface`. When a Nuxt app-dir signal is present (Nuxt 4 dependency
range or explicit `srcDir: "app"` per
[`srcDir` configuration](https://nuxt.com/docs/4.x/api/nuxt-config#srcdir)),
`.vue` files under `app/components/**` now produce muted
`sfc-framework-nuxt-app-dir-convention` records with
`conventionKind: "nuxt-app-components-directory"`, path-derived Nuxt names,
and `sourceFile` / `resolvedFile` navigation. The slice does not implement
`#components`, literal component-dir config inference, custom resolver
functions, Nuxt layers, graph edges, fan-in, deadness, fix-plan, export-action,
SARIF, or package edits.

## [2026-05-31] design | WT-SFC Nuxt resolver gap inventory

Recorded the Nuxt app-dir and custom resolver inventory:
[`wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md).
It follows the current
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration)
and the supplemental Nuxt finding in the
[`Vue Options corpus calibration`](../lab/wt-sfc-vue-options-corpus-calibration-2026-05-31.md#supplemental-nuxt-run).

Decision: `nuxt-resolver-inventory-before-implementation`,
`generated-manifest-before-alias-resolution`,
`nuxt-app-dir-convention-stays-muted`,
`custom-resolver-stays-unavailable`, `scan-gap-stays`, and
`no-action-surface`. The inventory separates generated manifests, root
`components/` convention evidence, `app/components/**`, `#components` alias
imports, literal `components.dirs` config, custom resolver functions, and Nuxt
layers into explicit-supportable, muted-observed, or unavailable buckets. No
behavior changes landed; the next Nuxt slice still needs failing fixtures,
reason codes, public-install verification, and proof that graph, fan-in,
deadness, fix-plan, export-action, SARIF, and package-edit lanes stay clean.

## [2026-05-31] regression | WT-SFC SvelteKit local action evidence

Recorded the beta.79 SvelteKit corpus regression run:
[`wt-sfc-beta79-sveltekit-local-action-regression-2026-05-31.md`](../lab/wt-sfc-beta79-sveltekit-local-action-regression-2026-05-31.md).
It follows up the beta.78
[`SvelteKit corpus calibration`](../lab/wt-sfc-sveltekit-corpus-calibration-2026-05-31.md)
and the current
[`SFC support policy`](../spec/sfc-support-policy.md#acceptance-for-current-mvp).

Decision: `sveltekit-corpus-fully-covered-with-local-action-evidence`,
`local-svelte-action-gap-closed`, `scan-gap-stays`, `no-action-surface`, and
`mvp-now-extended-to-local-bindings`. The beta.79 public package recovered the
two beta.78 missed local action records, `enhanceWrapper` and `focusAndScroll`,
as muted `sfc-framework-svelte-action-directive` evidence with
`bindingKind: "local-function"` and `bindingKind: "local-const-function"`.
`missedUsefulEvidenceCount` fell from 2 to 0 while `falsePositiveCount` and
`actionLeakCount` stayed at 0. Default Markdown stayed count-only, graph,
fan-in, deadness, fix-plan, and export-action surfaces had zero SFC-policy
matches, and `sfc-scan-gap` remained present.

## [2026-05-31] calibration | WT-SFC Storybook Vue corpus

Recorded a Vue corpus calibration pass against the Storybook Vue template:
[`wt-sfc-storybook-vue-corpus-calibration-2026-05-31.md`](../lab/wt-sfc-storybook-vue-corpus-calibration-2026-05-31.md).
The run follows the
[`WT-SFC corpus calibration plan`](../lab/wt-sfc-corpus-calibration-plan-2026-05-31.md)
and the current
[`SFC support policy`](../spec/sfc-support-policy.md#acceptance-for-current-mvp).
It completes the requirement left open by the
[`Vue Options corpus calibration`](../lab/wt-sfc-vue-options-corpus-calibration-2026-05-31.md),
which did not include global app registration.

Decision: `storybook-vue-corpus-covered-with-runtime-registry-gap`,
`current-mvp-safe-on-vue-corpus`, `scan-gap-stays`, and
`no-action-surface`. The beta.78 public package produced 17 SFC evidence
records across 28 Vue files, including 8 muted template refs, 2 muted Vue
Options API records, and 1 muted `sfc-global-component-registration` record
for `setup((app) => app.component(...))`. Default Markdown stayed count-only,
`sfc-scan-gap` remained present, and graph, fan-in, deadness, fix-plan, and
export-action surfaces had zero SFC-policy matches. The Vue corpus leg is now
covered for the current MVP, while runtime registries, Nuxt custom resolvers,
and stronger absence/action claims remain out of scope.

## [2026-05-31] calibration | WT-SFC Vue Options/template corpus

Recorded a Vue-focused partial corpus calibration pass against the Storybook
Vue CLI template:
[`wt-sfc-vue-options-corpus-calibration-2026-05-31.md`](../lab/wt-sfc-vue-options-corpus-calibration-2026-05-31.md).
The run follows the
[`WT-SFC corpus calibration plan`](../lab/wt-sfc-corpus-calibration-plan-2026-05-31.md)
and the current
[`SFC support policy`](../spec/sfc-support-policy.md#acceptance-for-current-mvp).

Decision: `vue-options-template-corpus-covered`,
`vue-corpus-leg-still-open`, `nuxt-custom-resolver-still-gap`,
`scan-gap-stays`, and `no-action-surface`. The beta.78 public package produced
6 SFC evidence records across 3 Vue files, including 2 muted
`sfc-framework-vue-options-registration` records and 4 muted template
component refs. Default Markdown stayed count-only, `sfc-scan-gap` remained
present, and graph, deadness, fix-plan, and export-action surfaces had zero
SFC-policy matches. A supplemental `nuxt-main` run produced 41 SFC evidence
records across 315 Vue files but no framework-convention records, confirming
that Nuxt `#components` and app-dir component convention semantics remain
custom-resolver/framework gaps. The Vue Options/template surface is reviewed,
but the full Vue corpus leg remains open until app/global registration and a
broader convention or generated-manifest surface are covered by a suitable
corpus.

## [2026-05-31] calibration | WT-SFC SvelteKit corpus

Recorded the WT-SFC SvelteKit corpus calibration pass against
`kit-main/packages/kit/test/apps/basics`:
[`wt-sfc-sveltekit-corpus-calibration-2026-05-31.md`](../lab/wt-sfc-sveltekit-corpus-calibration-2026-05-31.md).
The run follows the
[`WT-SFC corpus calibration plan`](../lab/wt-sfc-corpus-calibration-plan-2026-05-31.md)
and the current
[`SFC support policy`](../spec/sfc-support-policy.md#acceptance-for-current-mvp).

Decision: `sveltekit-corpus-covered-with-local-action-gap`,
`current-mvp-safe-on-svelte-corpus`, `scan-gap-stays`, `no-action-surface`,
and `consider-local-svelte-action-evidence`. The beta.78 public package
produced 29 SFC evidence records across 525 Svelte files, including 17 muted
`sfc-framework-svelte-action-directive` records for imported `use:enhance`
bindings and one isolated style asset record. Default Markdown stayed
count-only, `sfc-scan-gap` remained present, and graph, deadness, fix-plan, and
export-action surfaces had zero SFC-policy matches. The corpus also exposed
two useful local action wrappers (`enhanceWrapper`, `focusAndScroll`) that the
current imported-binding-only action lane does not record. The Svelte corpus
leg is now reviewed for the MVP; WT-SFC remains `MVP`, not `DONE`, until the
Vue corpus leg is reviewed too.

## [2026-05-31] calibration | WT-SFC Astro client directive corpus

Recorded the WT-SFC Astro client directive corpus calibration pass against
`astro-main/examples/with-nanostores`:
[`wt-sfc-astro-client-corpus-calibration-2026-05-31.md`](../lab/wt-sfc-astro-client-corpus-calibration-2026-05-31.md).
The run follows the
[`WT-SFC corpus calibration plan`](../lab/wt-sfc-corpus-calibration-plan-2026-05-31.md)
and the current
[`SFC support policy`](../spec/sfc-support-policy.md#acceptance-for-current-mvp).
It also completes the requirement left open by the
[`IMA2 Astro corpus calibration`](../lab/wt-sfc-ima2-astro-corpus-calibration-2026-05-31.md),
which did not exercise `client:*` evidence.

Decision: `astro-client-directive-corpus-covered`,
`current-mvp-safe-on-astro-client-corpus`, `scan-gap-stays`, and
`no-action-surface`. The beta.78 public package produced 14 SFC evidence
records across 3 Astro files, including 3 muted
`sfc-framework-astro-client-directive` records for imported components with
`client:load`. Default Markdown stayed count-only, `sfc-scan-gap` remained
present, and graph, deadness, fix-plan, and export-action surfaces had zero
SFC-policy matches. The Astro `client:*` corpus requirement is now covered;
WT-SFC remains `MVP`, not `DONE`, until the Vue and Svelte corpus legs are
reviewed too.

## [2026-05-31] calibration | WT-SFC IMA2 Astro corpus

Recorded the first dedicated WT-SFC Astro corpus calibration pass against
`repo/ima2-gen-main`:
[`wt-sfc-ima2-astro-corpus-calibration-2026-05-31.md`](../lab/wt-sfc-ima2-astro-corpus-calibration-2026-05-31.md).
The run follows the
[`WT-SFC corpus calibration plan`](../lab/wt-sfc-corpus-calibration-plan-2026-05-31.md)
and the current
[`SFC support policy`](../spec/sfc-support-policy.md#acceptance-for-current-mvp).

Decision: `astro-corpus-partial-script-and-template-only`,
`current-mvp-safe-on-astro-corpus`, `scan-gap-stays`, `no-action-surface`, and
`needs-astro-client-directive-corpus`. The beta.78 public package produced 67 SFC
evidence records across 19 Astro files, kept Astro-to-Astro template evidence
muted with `resolvedFile`, kept default Markdown count-only, preserved
`sfc-scan-gap`, and produced zero SFC matches in graph, deadness, fix-plan, and
export-action surfaces. This is an Astro-only partial pass, not a completed
Astro leg, because `frameworkConventionComponents` stayed 0 and no `client:*`
directive evidence was exercised. WT-SFC remains `MVP`, not `DONE`, until the
Astro `client:*` requirement plus the Vue and Svelte corpus legs are reviewed
too.

## [2026-05-31] calibration | WT-SFC Vite mixed SFC corpus

Recorded the first WT-SFC corpus calibration pass against `vite-main`:
[`wt-sfc-vite-corpus-calibration-2026-05-31.md`](../lab/wt-sfc-vite-corpus-calibration-2026-05-31.md).
The run follows the
[`WT-SFC corpus calibration plan`](../lab/wt-sfc-corpus-calibration-plan-2026-05-31.md)
and the current
[`SFC support policy`](../spec/sfc-support-policy.md#acceptance-for-current-mvp).

Decision: `mixed-sfc-smoke-corpus-useful`,
`current-mvp-safe-on-vite-corpus`, `scan-gap-stays`, `no-action-surface`, and
`needs-vue-svelte-astro-corpus-set`. The beta.78 public package produced 32
SFC evidence records across 31 SFC files in `vite-main`, kept the default
Markdown count-only, preserved `sfc-scan-gap`, and produced zero SFC matches in
graph, deadness, fix-plan, and export-action surfaces. WT-SFC remains `MVP`,
not `DONE`, until dedicated Vue, Svelte, and Astro app corpora are reviewed.

## [2026-05-31] planning | WT-SFC corpus calibration worksheet

Recorded the WT-SFC corpus calibration worksheet:
[`wt-sfc-corpus-calibration-plan-2026-05-31.md`](../lab/wt-sfc-corpus-calibration-plan-2026-05-31.md).
The worksheet follows the current MVP boundary in
[`wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md`](../lab/wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md)
and the policy contract in
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#follow-up-checklist).

Decision: `sfc-corpus-calibration-before-stronger-claims`,
`vue-svelte-astro-corpus-required`, `scan-gap-stays`, and
`no-action-surface`. The next implementation-affecting WT-SFC PR should cite
a corpus report covering at least one Vue app, one Svelte app, and one Astro
app before changing SFC absence claims, default Markdown wording, action lanes,
or broad framework semantics.

## [2026-05-31] status | WT-SFC MVP boundary and remaining gaps

Recorded the beta.78-era WT-SFC MVP status and remaining gaps:
[`wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md`](../lab/wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md).
The current policy boundary is
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#acceptance-for-current-mvp),
and the latest public-install audit-brief verification is
[`wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md`](../lab/wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md).
The older beta.67 gap inventory remains available as historical context:
[`wt-sfc-remaining-gaps-inventory-2026-05-26.md`](../lab/wt-sfc-remaining-gaps-inventory-2026-05-26.md).

Decision: `wt-sfc-mvp-status-recorded`,
`sfc-current-surface-linked-to-public-verification`,
`remaining-gaps-stay-explicit`, and `wt-sfc-not-done`. WT-SFC is now recorded
as a bounded MVP evidence surface with public verification links through
beta.78, but it still does not support broad template semantics, custom
framework resolvers, compiler/runtime magic, strong framework absence claims,
or promotion of review-only SFC evidence into fan-in, deadness, `SAFE_FIX`,
`EXISTS`, fix-plan, export-action, package edits, or SARIF findings.

## [2026-05-31] verification | WT-SFC SFC evidence audit brief public package

Verified beta.78 public-install behavior for the WT-SFC SFC evidence audit
brief surface:
[`wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md`](../lab/wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md).
The surface is specified in
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#audit-brief-surface).
The source implementation is tracked by
[`lumin-audit` PR #567](https://github.com/annyeong844/lumin_lab/pull/567),
and the beta.78 metadata/changelog/SARIF version bump is tracked by
[`lumin-audit` PR #568](https://github.com/annyeong844/lumin_lab/pull/568).
The installed package came from public main
[`2fa1fc5`](https://github.com/annyeong844/lumin-repo-lens-lab/commit/2fa1fc5d4d08f284f547c562c953d40c4ab937d7),
whose
[`Public Package CI`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26685684469)
passed.

Decision: `sfc-evidence-brief-public-verified`,
`sfc-evidence-stays-count-only`,
`sfc-raw-names-stay-out-of-markdown`, `sfc-scan-gap-stays`, and
`sarif-version-sync-public-verified`. The public package surfaces SFC counts in
`manifest.json.sfcEvidence`, `audit-summary.latest.md`, and
`audit-review-pack.latest.md`, keeps raw component names, tag names, directive
names, macro/API names, and action wording out of default Markdown, preserves
`sfc-scan-gap`, and leaves graph edges, named export fan-in, deadness,
`SAFE_FIX`, `EXISTS`, fix-plan, and export-action lanes unchanged. WT-SFC
remains `MVP`, not `DONE`.

## [2026-05-30] design | WT-SFC global registration P2 inventory

Recorded the next WT-SFC global component registration refinement:
[`wt-sfc-global-registration-p2-fixture-inventory-2026-05-30.md`](../lab/wt-sfc-global-registration-p2-fixture-inventory-2026-05-30.md).
The existing global-registration lane and public verification are anchored by
[`wt-sfc-global-component-registration-fixture-inventory-2026-05-26.md`](../lab/wt-sfc-global-component-registration-fixture-inventory-2026-05-26.md)
and
[`wt-sfc-beta68-global-component-registration-verification-2026-05-27.md`](../lab/wt-sfc-beta68-global-component-registration-verification-2026-05-27.md).

Decision: `global-registration-p2-before-template-consumption`,
`plugin-install-syntax-is-not-runtime-install-proof`,
`async-registration-stays-muted`,
`duplicate-registration-stays-ambiguous`, and `scan-gap-stays`. The P2
inventory selects plugin `install(app) { app.component(...) }` syntax, literal
async component factories, and duplicate literal registrations as the next
fixture set. The lane must stay review-only and out of graph edges, named
export fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, export-action,
template refs, and default action lanes. WT-SFC remains `MVP`, not `DONE`.

## [2026-05-29] verification | WT-SFC Vue Options API registration public package

Verified beta.76 public-install behavior for WT-SFC Vue Options API
`export default { components: { ... } }` registration evidence:
[`wt-sfc-beta76-vue-options-registration-verification-2026-05-29.md`](../lab/wt-sfc-beta76-vue-options-registration-verification-2026-05-29.md).
The installed package came from public main `8995a4f`, whose
[`Public Package CI`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26638211663)
passed. The source implementation is tracked by
[`lumin-audit` PR #561](https://github.com/annyeong844/lumin_lab/pull/561),
and the beta.76 metadata/changelog/SARIF version bump is tracked by
[`lumin-audit` PR #562](https://github.com/annyeong844/lumin_lab/pull/562).

Decision: `vue-options-registration-public-verified`,
`explicit-value-binding-required-for-vue-options-evidence`,
`sarif-version-sync-public-verified`, and
`vue-options-evidence-stays-review-only`. The public package records
`symbols.json.sfcFrameworkConventionComponents[]` entries for literal Vue
Options API `components` members backed by explicit non-type imports only,
keeps all records muted, rejects computed keys, unbound identifiers, type-only
bindings, comment-only text, and template text, keeps SARIF
`tool.driver.version` on beta.76, and leaves graph edges, named export fan-in,
deadness, `SAFE_FIX`, `EXISTS`, fix-plan, and export-action lanes unchanged.
WT-SFC remains `MVP`, not `DONE`.

## [2026-05-28] implementation | WT-SFC Vue Options API registration evidence

Added muted WT-SFC framework convention evidence for Vue Options API local
component registration:
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).
The source contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `vue-options-registration-is-muted-evidence`,
`explicit-binding-required-for-vue-options-evidence`, and
`vue-options-evidence-stays-review-only`. Literal
`export default { components: { ... } }` entries in ordinary Vue `<script>`
blocks now emit `symbols.json.sfcFrameworkConventionComponents[]` records only
when the component value resolves to an explicit non-type import binding.
Dynamic/computed names, unbound identifiers, comment-only text, and template
text stay out. The lane keeps graph edges, named export fan-in, deadness,
`SAFE_FIX`, `EXISTS`, fix-plan, and export-action unchanged. WT-SFC stays
`MVP`, not `DONE`, pending the remaining framework/custom-resolver/compiler-runtime
gaps.

## [2026-05-28] verification | WT-SFC Vue macro registration public package

Verified beta.75 public-install behavior for WT-SFC Vue
`defineOptions({ components })` macro registration evidence:
[`wt-sfc-beta75-vue-macro-registration-verification-2026-05-28.md`](../lab/wt-sfc-beta75-vue-macro-registration-verification-2026-05-28.md).
The installed package came from public main `18589cc`, whose
[`Public Package CI`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26597965914)
passed. The source implementation is tracked by
[`lumin-audit` PR #558](https://github.com/annyeong844/lumin_lab/pull/558),
and the beta.75 metadata/changelog/SARIF version bump is tracked by
[`lumin-audit` PR #559](https://github.com/annyeong844/lumin_lab/pull/559).

Decision: `vue-macro-registration-public-verified`,
`explicit-binding-required-for-vue-macro-evidence`,
`sarif-version-sync-public-verified`, and
`vue-macro-evidence-stays-review-only`. The public package records
`symbols.json.sfcFrameworkConventionComponents[]` entries for literal Vue
`defineOptions({ components })` members backed by explicit imports only, keeps
all records muted, rejects computed keys, unbound identifiers, comment-only
text, and template text, keeps SARIF `tool.driver.version` on beta.75, and
leaves graph edges, named export fan-in, deadness, `SAFE_FIX`, `EXISTS`,
fix-plan, and export-action lanes unchanged. WT-SFC remains `MVP`, not `DONE`.

## [2026-05-28] verification | WT-SFC Svelte action directive public package

Verified beta.74 public-install behavior for WT-SFC Svelte `use:action`
directive evidence:
[`wt-sfc-beta74-svelte-action-directive-verification-2026-05-28.md`](../lab/wt-sfc-beta74-svelte-action-directive-verification-2026-05-28.md).
The installed package came from public main `eb4e560`, whose
[`Public Package CI`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26590569991)
passed. The source implementation is tracked by
[`lumin-audit` PR #555](https://github.com/annyeong844/lumin_lab/pull/555),
and the beta.74 metadata/SARIF version bump is tracked by
[`lumin-audit` PR #556](https://github.com/annyeong844/lumin_lab/pull/556).

Decision: `svelte-action-directive-public-verified`,
`explicit-binding-required-for-svelte-action-evidence`,
`sarif-version-sync-public-verified`, and
`svelte-action-evidence-stays-review-only`. The public package records
`symbols.json.sfcFrameworkConventionComponents[]` entries for explicitly bound
Svelte `use:action` directives only, keeps all records muted, rejects unbound
and comment-only actions, keeps SARIF `tool.driver.version` on beta.74, and
leaves graph edges, named export fan-in, deadness, `SAFE_FIX`, `EXISTS`,
fix-plan, and export-action lanes unchanged. WT-SFC remains `MVP`, not `DONE`.

## [2026-05-28] implementation | WT-SFC unplugin config convention evidence

Added muted WT-SFC framework convention evidence for
`unplugin-vue-components` config usage:
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).
The source contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `unplugin-config-is-muted-evidence`,
`component-targets-not-guessed-from-config`, and
`graph-action-lanes-stay-clean`. Vite/Webpack config files that import
`unplugin-vue-components`, or require it from CommonJS Webpack config, and call
the plugin function now record
`symbols.json.sfcFrameworkConventionComponents[]` entries with
`reason: "sfc-framework-auto-import-plugin-config"`. These entries expose the
config file, plugin import specifier, and call site only; they do not create
component targets, graph edges, named export fan-in, deadness proof,
`SAFE_FIX`, `EXISTS`, fix-plan, or export-action entries. Public-install
verification is still required before this slice is treated as verified.

## [2026-05-28] verification | WT-SFC Nuxt convention public package

Verified beta.71 public-install behavior for WT-SFC Nuxt filesystem convention
evidence:
[`wt-sfc-beta71-nuxt-convention-verification-2026-05-28.md`](../lab/wt-sfc-beta71-nuxt-convention-verification-2026-05-28.md).
The installed package came from public main `998b1cc`, whose
[`Public Package CI`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26517500561)
passed.

Decision: `nuxt-convention-public-verified`,
`nuxt-signal-required-for-convention-evidence`, and
`path-derived-nuxt-names-public-verified`. The public package records
`symbols.json.sfcFrameworkConventionComponents[]` only when a Nuxt signal is
present, derives nested component names such as `BaseButton` and `UserIndex`
from path segments, keeps all records muted, and leaves graph edges, named
export fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, and export-action lanes
unchanged. WT-SFC remains `MVP`, not `DONE`.

## [2026-05-27] implementation | WT-SFC Nuxt filesystem convention evidence

Added a dedicated muted convention surface for Nuxt `components/` files:
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).
The source contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `nuxt-fs-convention-is-muted-evidence`,
`generated-manifest-surface-stays-separate`, and `graph-action-lanes-stay-clean`.
The implementation emits
`symbols.json.sfcFrameworkConventionComponents[]` for observed `.vue` files
under root `components/` only when a Nuxt signal exists, records `sourceFile` /
`resolvedFile` navigation with path-derived Nuxt component names such as
`BaseButton`, and keeps graph edges, named export fan-in, deadness, `SAFE_FIX`,
`EXISTS`, fix-plan, and export-action lanes unchanged. Public-install
verification is still required before marking this convention slice complete.

## [2026-05-27] implementation | WT-SFC generated manifest nonliteral visibility

Extended the generated component-manifest surface from
[`sfc-generated-component-manifest-evidence.md`](../spec/sfc-generated-component-manifest-evidence.md)
so computed/nonliteral members are no longer silently dropped. The behavior is
pinned by [`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs)
and [`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs), and
documented in
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).

Decision: `nonliteral-generated-manifest-is-skipped-evidence`,
`package-manifest-imports-stay-out`, and `graph-action-lanes-stay-clean`.
Computed manifest members now surface as `status: "skipped"` with
`sfc-framework-generated-manifest-nonliteral`; package/nonrelative imports are
still excluded. Public-install verification is still required before marking
this visibility slice complete.

## [2026-05-27] implementation | WT-SFC generated component manifest evidence

Implemented the P1 generated component-manifest surface specified in
[`sfc-generated-component-manifest-evidence.md`](../spec/sfc-generated-component-manifest-evidence.md).
The source contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs), and the
SFC policy links the implemented lane:
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).

Decision: `generated-manifest-evidence-before-convention-inference`,
`manifest-allow-list-read-exception`, `sfc-target-stays-muted-with-resolvedFile`,
`missing-manifest-target-is-unresolved`, and `scan-gap-stays`. The source
implementation emits `symbols.json.sfcGeneratedComponentManifests[]` for
allow-listed `components.d.ts` and `.nuxt/components.d.ts` manifests while
keeping graph edges, named export fan-in, deadness, `SAFE_FIX`, `EXISTS`, and
action lanes unchanged. Public-install verification is still required before
marking the lane complete.

## [2026-05-27] design | WT-SFC generated component manifest evidence

Selected the P1 generated component-manifest contract from the framework magic
inventory:
[`sfc-generated-component-manifest-evidence.md`](../spec/sfc-generated-component-manifest-evidence.md).
The decision is linked from the framework magic inventory:
[`wt-sfc-framework-magic-fixture-inventory-2026-05-27.md`](../lab/wt-sfc-framework-magic-fixture-inventory-2026-05-27.md).

Decision: `generated-manifest-evidence-before-convention-inference`,
`manifest-allow-list-read-exception`, `manifest-availability-not-absence`,
`sfc-target-stays-muted-with-resolvedFile`,
`missing-manifest-target-is-unresolved`, and `scan-gap-stays`. P1 will use a
dedicated `symbols.json.sfcGeneratedComponentManifests[]` surface for Nuxt
`.nuxt/components.d.ts` and `unplugin-vue-components` `components.d.ts`, keep
SFC targets muted with `resolvedFile`, and record stale manifest targets as
`unresolved` review evidence. Broad convention/config inference remains future
work.

## [2026-05-27] design | WT-SFC framework magic inventory

Recorded the WT-SFC framework magic fixture inventory:
[`wt-sfc-framework-magic-fixture-inventory-2026-05-27.md`](../lab/wt-sfc-framework-magic-fixture-inventory-2026-05-27.md).
The SFC policy now records
[`sfc-framework-magic`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration)
as a `SPEC` lane.

Decision: `framework-magic-inventory-before-implementation`,
`generated-manifest-is-availability-not-absence`,
`convention-and-compiler-magic-stay-muted-or-unavailable`, and
`scan-gap-stays`. The next safe implementation candidate is generated
component-manifest evidence, not broad framework inference. Nuxt
`.nuxt/components.d.ts` and `unplugin-vue-components` `components.d.ts` may
become review-only availability evidence when present, but missing generated
manifests must not become absence proof.

## [2026-05-27] verification | WT-SFC global component registration public package

Verified beta.68 public-install behavior for WT-SFC explicit global component
registration evidence:
[`wt-sfc-beta68-global-component-registration-verification-2026-05-27.md`](../lab/wt-sfc-beta68-global-component-registration-verification-2026-05-27.md).
The installed package came from public main `52be4b9`, whose
[`Public Package CI`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26458546860)
passed.

Decision: `global-registration-public-verified` and
`registration-availability-not-template-consumption`. Explicit Vue
`app.component(...)`, `createSSRApp(...)`, and app-returning
`createApp(...).use(...)` registration receivers now appear in
`symbols.json.sfcGlobalComponentRegistrations[]`. `mount()` chains remain
excluded. SFC targets stay muted with `resolvedFile` reviewer navigation,
source targets may resolve, and the lane still does not enter graph edges,
named export fan-in, deadness, `SAFE_FIX`, `EXISTS`, package edits, or default
action lanes.

## [2026-05-26] design | WT-SFC global component registration inventory

Selected the next WT-SFC design lane from the remaining gaps inventory:
[`wt-sfc-global-component-registration-fixture-inventory-2026-05-26.md`](../lab/wt-sfc-global-component-registration-fixture-inventory-2026-05-26.md).
The SFC policy now records
[`sfc-global-component-registration`](../spec/sfc-support-policy.md#p5-candidate-explicit-global-component-registration)
as a `SPEC` lane.

Decision: `explicit-registration-before-framework-convention` and
`registration-evidence-is-not-template-consumption`. Explicit Vue
`app.component(...)` and plugin registration syntax can become review-only
availability evidence, but it must not enter graph edges, named export fan-in,
deadness, `SAFE_FIX`, `EXISTS`, package edits, or default action lanes.

## [2026-05-26] design | WT-SFC remaining gaps inventory

Recorded the post-beta.67 WT-SFC remaining gaps inventory:
[`wt-sfc-remaining-gaps-inventory-2026-05-26.md`](../lab/wt-sfc-remaining-gaps-inventory-2026-05-26.md).

Decision: `remaining-gaps-inventory-before-next-sfc-lane` and
`scan-gap-stays-until-framework-semantics-are-proven`. WT-SFC stays `MVP`, not
`DONE`; automatic/global component registration, complex Vue registration,
dynamic component semantics, template prop/event/member use, and framework
magic need lane-specific fixtures before they affect absence claims.

## [2026-05-26] verification | WT-SFC template component target public package

Verified beta.67 public-install behavior for WT-SFC template component target
evidence:
[`wt-sfc-beta67-template-component-target-verification-2026-05-26.md`](../lab/wt-sfc-beta67-template-component-target-verification-2026-05-26.md).

Decision: `template-ref-public-verified` and
`sfc-target-file-evidence-without-graph-claim`. SFC-to-SFC component refs now
remain `muted` as non-source bindings while carrying `resolvedFile`, and they
still do not enter graph edges, named export fan-in, deadness, `SAFE_FIX`,
`EXISTS`, package edits, or default action lanes.

## [2026-05-26] implementation | WT-SFC template component target evidence

Tightened the WT-SFC template component reference surface pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `sfc-target-file-evidence-without-graph-claim`. When an explicit
template component binding points at another existing SFC file, the ref remains
`muted` as `sfc-template-component-non-source-binding`, but now carries
`resolvedFile` so the reviewer can inspect the target. It still does not enter
graph edges, named export fan-in, deadness, `SAFE_FIX`, `EXISTS`, package edits,
or default action lanes.

## [2026-05-26] implementation | WT-SFC template component refs source surface

Implemented the WT-SFC template component reference source surface described by
[`wt-sfc-template-component-ref-fixture-inventory-2026-05-26.md`](../lab/wt-sfc-template-component-ref-fixture-inventory-2026-05-26.md)
and
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p4-candidate-template-component-refs).
The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Decision: `binding-aware-review-only-surface`. Explicit Vue/Svelte/Astro
component bindings now appear in `symbols.json.sfcTemplateComponentRefs[]`, but
they do not enter graph edges, named export fan-in, deadness, `SAFE_FIX`,
`EXISTS`, package edits, or default action lanes.

## [2026-05-26] design | WT-SFC template component refs inventory

Recorded the next WT-SFC lane as a fixture inventory, not an implementation:
[`wt-sfc-template-component-ref-fixture-inventory-2026-05-26.md`](../lab/wt-sfc-template-component-ref-fixture-inventory-2026-05-26.md).
The SFC policy now marks
[`sfc-template-component-refs`](../spec/sfc-support-policy.md#p4-candidate-template-component-refs)
as `SPEC`.

Decision: `template-binding-inventory-before-evidence` and
`binding-aware-or-no-claim`. Template tags can become review-only evidence only
after a fixture-pinned binding model connects them to explicit component
bindings. They still do not feed graph edges, named export fan-in, deadness,
`SAFE_FIX`, `EXISTS`, or package edits.

## [2026-05-26] verification | WT-SFC style asset public package

Verified the WT-SFC
[`sfc-style-assets`](../spec/sfc-support-policy.md#p3-candidate-style-assets)
lane against the public beta.65 install, not just source tests. The runtime
matrix is recorded in
[`wt-sfc-beta65-style-assets-verification-2026-05-26.md`](../lab/wt-sfc-beta65-style-assets-verification-2026-05-26.md),
with public package publication in
[`lumin-repo-lens-lab#3`](https://github.com/annyeong844/lumin-repo-lens-lab/pull/3)
and the packaged CSS-escape fix in
[`lumin-repo-lens-lab#5`](https://github.com/annyeong844/lumin-repo-lens-lab/pull/5).

Decision: `sfc-style-assets-public-verified` and
`asset-evidence-not-symbol-fan-in`. Style asset references stay grounded in
`symbols.json.sfcStyleAssetReferences[]`, while JS/TS graph edges, symbol
fan-in, deadness, and action lanes remain untouched.

## [2026-05-26] implementation | WT-SFC style asset source surface

Implemented the WT-SFC
[`sfc-style-assets`](../spec/sfc-support-policy.md#p3-candidate-style-assets)
source surface in
[`sfc-consumers.mjs`](../../_lib/sfc-consumers.mjs),
[`build-symbol-graph.mjs`](../../build-symbol-graph.mjs), and
[`symbol-graph-artifact.mjs`](../../_lib/symbol-graph-artifact.mjs). Literal
relative SFC style `url()` and style `@import` references now emit
`symbols.json.sfcStyleAssetReferences[]` records, while missing relative style
assets stay diagnostic-only with reason `sfc-style-asset-unresolved`.

The contract is pinned in
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs): style assets do
not enter `resolvedInternalEdges[]`, do not affect named export fan-in, and do
not leak package/URL/dynamic/comment/template-attribute forms into concrete
source edges. The follow-up public-install verification is recorded in
[`wt-sfc-beta65-style-assets-verification-2026-05-26.md`](../lab/wt-sfc-beta65-style-assets-verification-2026-05-26.md).

## [2026-05-25] design | WT-SFC style asset next-lane inventory

Recorded the WT-SFC
[`style asset fixture inventory`](../lab/wt-sfc-style-asset-fixture-inventory-2026-05-25.md)
and linked it from the
[`SFC support policy`](../spec/sfc-support-policy.md#p3-candidate-style-assets).
The decision is `style-assets-before-template-refs` and
`asset-reachability-not-symbol-fan-in`: literal SFC style `url()` / `@import`
references may become non-source asset evidence, but they must not become JS/TS
module graph edges, named export fan-in, `SAFE_FIX`, `EXISTS`, package edits, or
dead-export ranking.

The existing SFC source guards remain
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs). WT-SFC stays
`MVP`, not `DONE`; template component refs remain parked until a binding-aware
fixture inventory exists.

## [2026-05-25] verification | WT-SFC script src beta.64

Recorded beta.64 public-install verification for the WT-SFC
[`sfc-script-src`](../spec/sfc-support-policy.md#script-src-contract) lane at
[`wt-sfc-beta64-script-src-reachability-verification-2026-05-25.md`](../lab/wt-sfc-beta64-script-src-reachability-verification-2026-05-25.md).
The installed package from public main `77bc7e1e` passed all seven runtime
checks: Vue/Svelte literal relative `<script src>` records
`resolvedInternalEdges[].kind === "sfc-script-src"`,
`symbols.uses.sfcScriptSrcReachability === 2`, script-sourced named exports keep
fan-in `0`, package/URL/dynamic/empty sources do not create concrete edges,
missing relative sources stay diagnostic-only, and `sfc-scan-gap` remains
visible.

The source contract remains pinned by
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs). WT-SFC returns
to `MVP`, not `DONE`: script imports and script-source reachability are
grounded, while template refs, style assets, and framework magic remain future
lanes.

## [2026-05-25] implementation | WT-SFC script src reachability

Implemented the WT-SFC
[`sfc-script-src`](../spec/sfc-support-policy.md#script-src-contract) source
lane in
[`sfc-consumers.mjs`](../../_lib/sfc-consumers.mjs) and
[`build-symbol-graph.mjs`](../../build-symbol-graph.mjs). Literal relative
Vue/Svelte `<script src>` references now emit `resolvedInternalEdges[]` with
`kind: "sfc-script-src"` and `symbols.uses.sfcScriptSrcReachability`, while
named exports in the referenced file still keep `fanInByIdentity` at zero unless
they are consumed by a real import.

The contract is pinned in
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs): package,
URL/data, non-literal, and missing script-source forms do not become concrete
import edges. WT-SFC moves to `VERIFY` for this lane until public install
verification confirms the beta package; the follow-up beta.64 runtime
verification is recorded above.

## [2026-05-25] design | WT-SFC script src fixture inventory

Added the WT-SFC
[`script src fixture inventory`](../lab/wt-sfc-script-src-fixture-inventory-2026-05-25.md)
and linked it from the
[`SFC support policy`](../spec/sfc-support-policy.md#script-src-contract).
The decision is `script-src-reachability-first` and `no-symbol-fan-in`: literal
Vue/Svelte `<script src>` can prove source-file reachability, but it must not
protect named exports by itself.

This keeps the beta.63
[`script import consumer`](../lab/wt-sfc-beta63-script-import-consumers-verification-2026-05-25.md)
lane intact while making the next implementation slice narrower than broad
template parsing. WT-SFC remains `MVP`, not `DONE`.

## [2026-05-25] design | WT-SFC support boundary

Recorded the SFC support boundary in
[`sfc-support-policy.md`](../spec/sfc-support-policy.md) after the beta.63
[`script import consumer verification`](../lab/wt-sfc-beta63-script-import-consumers-verification-2026-05-25.md).
The current implemented lane is deliberately narrow: Vue/Svelte inline script
imports and Astro frontmatter imports can feed graph and fan-in evidence, while
template text, `<script src>`, style assets, and framework magic remain outside
the modeled surface.

The supporting source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs). WT-SFC stays
`MVP`, not `DONE`: the next slice should pick one lane, likely `sfc-script-src`,
and prove its reachability semantics before implementation.

## [2026-05-25] verification | WT-09 beta.61 cap/noise v2

Recorded beta.61 public-install verification for
[`block-clone-threshold-policy-v2`](../spec/block-clone-detection.md#cap-allocation)
in
[`wt09-beta61-block-clone-cap-noise-v2-verification-2026-05-25.md`](../lab/wt09-beta61-block-clone-cap-noise-v2-verification-2026-05-25.md).
The installed artifact emitted v2 cap fields, preserved legacy `maxGroups` when
explicitly supplied, mirrored only shallow threshold/saturation metadata into
`manifest.blockClones`, and kept clone evidence out of Markdown/action lanes.

The self-dogfood corpus moved from beta.60's 7 review / 93 muted groups to
beta.61's 49 review / 100 muted groups, with `reviewCapSaturated: false` and
`mutedCapSaturated: true`. That confirms the core invariant: review groups are
not displaced by muted noise. WT-09 remains `MVP`, not `DONE`, because broader
corpus calibration and P3 Markdown wording remain deferred.

## [2026-05-25] implementation | WT-09 cap/noise v2 allocation

Implemented the
[`block-clone-threshold-policy-v2`](../spec/block-clone-detection.md#thresholds)
cap allocation contract in
[`block-clone-artifact.mjs`](../../_lib/block-clone-artifact.mjs). The producer
now ranks candidates, applies the internal `maxCandidateGroups` guard, classifies
noise, then fills independent review and muted budgets before any deprecated
`maxGroups` total compatibility cap.

The new BC12c/BC12d checks in
[`test-build-block-clone-index.mjs`](../../tests/test-build-block-clone-index.mjs)
and the
[`Vitest mirror`](../../tests/build-block-clone-index.test.mjs) pin both critical
contracts: muted noise cannot displace a lower-ranked review group, and legacy
`maxGroups` still limits total emitted groups. The manifest mirror is pinned in
[`test-audit-manifest-export-surface.mjs`](../../tests/test-audit-manifest-export-surface.mjs)
and
[`audit-manifest-export-surface.test.mjs`](../../tests/audit-manifest-export-surface.test.mjs).

WT-09 remains `MVP`, not `DONE`: the follow-up beta.61 public-install
verification is recorded above, but broader corpus reruns are still required
before weak P3 Markdown is considered.

## [2026-05-25] verification | WT-09 beta.60 block clone noise policy

Recorded beta.60 public-install verification for
[`block-clone-noise-policy-v1`](../spec/block-clone-detection.md#noise-and-mute-policy)
in
[`wt09-beta60-block-clone-noise-policy-verification-2026-05-25.md`](../lab/wt09-beta60-block-clone-noise-policy-verification-2026-05-25.md).
The installed artifact emitted `noisePolicy` with 7 review groups and 93 muted
groups, mirrored only shallow counts/reasons into `manifest.blockClones`, and
kept clone group ids plus clone evidence fields out of Markdown, fix-plan, and
export-action-safety lanes.

Decision: `noise-policy-public-verified` and
`p3-markdown-still-deferred`. WT-09 remains `MVP`, not `DONE`, because
`maxGroups: 100` saturated and P4 corpus calibration still needs to decide
whether cap/noise ordering or weak P3 Markdown should change.

## [2026-05-24] implementation | WT-09 block clone noise policy

Implemented `block-clone-noise-policy-v1` from
[`block-clone-detection.md`](../spec/block-clone-detection.md#noise-and-mute-policy).
Raw `block-clones.json` groups now carry review/muted visibility, muted groups
record stable reasons, and `manifest.blockClones` mirrors only shallow
review/muted counts and reason totals. The policy was motivated by the beta.59
[`block clone noise review`](../lab/wt09-beta59-block-clone-noise-review-2026-05-24.md).

Node and Vitest coverage both pin the new classification contract. Default
Markdown remains off; WT-09 is still `MVP`, not `DONE`.

## [2026-05-24] design | WT-09 block clone noise policy

Updated [`block-clone-detection.md`](../spec/block-clone-detection.md) with the
`block-clone-noise-policy-v1` contract after the beta.59
[`block clone noise review`](../lab/wt09-beta59-block-clone-noise-review-2026-05-24.md).
The policy keeps raw `block-clones.json` evidence auditable while allowing
Node/Vitest mirror pairs, broad test scaffolding, and same-file repeats to be
classified as muted before any default P3 Markdown.

WT-09 remains `MVP`: the next implementation slice is noise classification in
the raw artifact plus shallow manifest counts, not renderer wording.

## [2026-05-24] lab | WT-09 block clone noise review

Added the beta.59 WT-09 block clone noise review at
[`wt09-beta59-block-clone-noise-review-2026-05-24.md`](../lab/wt09-beta59-block-clone-noise-review-2026-05-24.md).
The installed artifact had useful signal, but the capped top 100 groups were
dominated by test migration noise: 58 Node/Vitest mirror-pair groups, 18 test
cross-file groups, 17 same-file repeats, and 7 engine cross-file groups.

Decision: `p3-default-markdown-not-ready`, `needs-noise-policy`,
`engine-signal-present`, and `cap-saturated`. Keep default Markdown off until a
named noise/mute policy handles mirror pairs and broad test scaffolding.

## [2026-05-24] verification | WT-09 beta.59 block clone manifest

Recorded beta.59 public-install verification for the WT-09/P4 P2 manifest
mirror in
[`wt09-beta59-block-clone-manifest-verification-2026-05-24.md`](../lab/wt09-beta59-block-clone-manifest-verification-2026-05-24.md).
The installed package exposed `build-block-clone-index.mjs`,
`block-clone-artifact.mjs`, emitted `block-clones.json` in full profile, and
mirrored only shallow `manifest.blockClones` metadata.

The runtime artifact was not empty: raw `block-clones.json` contained 100
groups and 210 instances, but `manifest.blockClones` still omitted raw
`groups[]`, `instances[]`, source spans, and per-instance files. Markdown also
remained unchanged, so WT-09 stays `MVP`, not `DONE`; P3 review-pack wording,
broader corpus calibration, cap/noise review, and threshold/rendering decisions
remain separate slices.

## [2026-05-24] implementation | WT-09 block clone manifest mirror

Added the WT-09/P4 P2 manifest mirror for the
[`block-clone-detection.md`](../spec/block-clone-detection.md) surface.
`manifest.blockClones` now mirrors shallow `block-clones.json` status,
review-only, normalization, threshold, and summary-count metadata while keeping
raw groups, instances, and source spans out of the manifest. The evidence is
pinned by
[`test-audit-manifest-export-surface.mjs`](../../tests/test-audit-manifest-export-surface.mjs)
and
[`audit-manifest-export-surface.test.mjs`](../../tests/audit-manifest-export-surface.test.mjs),
with end-to-end producer coverage still anchored by
[`test-build-block-clone-index.mjs`](../../tests/test-build-block-clone-index.mjs)
and
[`build-block-clone-index.test.mjs`](../../tests/build-block-clone-index.test.mjs).

WT-09 remains `MVP`, not `DONE`: public install verification, corpus
calibration, P3 review-pack wording, and threshold/rendering decisions remain
separate slices.

## [2026-05-24] implementation | WT-09 block clone artifact P1

[`PR #504`](https://github.com/annyeong844/lumin_lab/pull/504) implemented
the WT-09/P4 P1 artifact-only slice described by
[`block-clone-detection.md`](../spec/block-clone-detection.md) and the
[`WT-09 block clone fixture inventory`](../lab/wt09-block-clone-fixture-inventory-2026-05-24.md).
The new `build-block-clone-index.mjs` producer emits review-only
`block-clones.json` evidence and keeps repeated token-region evidence out of
`function-clones.json`, Markdown, fix-plan, SAFE, EXISTS, and pre-write cue
lanes. The supporting guards are
[`test-build-block-clone-index.mjs`](../../tests/test-build-block-clone-index.mjs)
and
[`build-block-clone-index.test.mjs`](../../tests/build-block-clone-index.test.mjs),
which now cover BC1-BC11, including destructured binding normalization and
object-pattern key safety.

WT-09 remains `MVP`, not `DONE`: public install verification, corpus
calibration, P2 manifest mirroring, and any review-pack wording are still
separate slices.

## [2026-05-24] design | WT-09 block clone fixture inventory

Added the WT-09 fixture inventory at
[`wt09-block-clone-fixture-inventory-2026-05-24.md`](../lab/wt09-block-clone-fixture-inventory-2026-05-24.md).
It turns the
[`block-clone-detection.md`](../spec/block-clone-detection.md) P1 test
requirements into named BC1-BC9 fixture contracts before runtime behavior starts.
The [performance workstream](workstreams/performance.md) now links this
inventory so future suffix-array/LCP work cannot stop at happy-path top-level
function clones.

## [2026-05-24] design | WT-09 block clone detection boundary

Added the WT-09/P4 block clone detection spec at
[`block-clone-detection.md`](../spec/block-clone-detection.md). The spec records
the decision to keep future suffix-array/LCP repeated-region detection in a
separate review-only `block-clones.json` artifact instead of widening
[`function-clones.json`](../spec/lumin-work-tracker.md#current-wt-09p4-block-clone-design-note).
The [performance workstream](workstreams/performance.md) now links the spec and
tracks the guardrail that block clone evidence must not leak into top-level
function clone, SAFE, fix-plan, or pre-write cue lanes.

## [2026-05-24] design | WT-17 import.meta.glob scan-policy expansion

Added the WT-17 scan-policy expansion spec for
[`import.meta.glob`](../spec/import-meta-glob-scan-policy-expansion.md). The
spec keeps the current unsupported diagnostic contract from
[`test-import-meta-glob-diagnostics.mjs`](../../tests/test-import-meta-glob-diagnostics.mjs)
and
[`import-meta-glob-diagnostics.test.mjs`](../../tests/import-meta-glob-diagnostics.test.mjs)
intact while defining the P1 fixture matrix for future concrete dynamic edges.
The tracker now points future WT-17 implementation work at that spec instead of
allowing broad glob expansion from one fixture.

## [2026-05-24] maintenance | Parked suite dogfooding guide refresh

Refreshed
[`parked-suite-dogfooding.md`](concepts/parked-suite-dogfooding.md) after the
[`Vitest Mirror Lane Closure Audit`](vitest-mirror-closure-audit.md) reduced
the parked remainder to two Node-authoritative umbrella suites:
[`tests/test-audit-repo.mjs`](../../tests/test-audit-repo.mjs) and
[`tests/test-pre-write-cue-tiers.mjs`](../../tests/test-pre-write-cue-tiers.mjs).
The guide now routes completed performance, incremental, deadness, ranking,
calibration, scanner, and producer-artifact lanes back to the closure audit
instead of treating them as active parked candidates.

## [2026-05-24] cleanup | WT-25 dependency hygiene DONE status

Marked the WT-25 dependency hygiene surface complete for its current
review-only scope after beta.57 artifact verification and beta.58
summary/review-pack verification. Source evidence is linked through the
[`unused-deps-producer.md`](../spec/unused-deps-producer.md) and
[`unused-deps-review-surface.md`](../spec/unused-deps-review-surface.md) specs,
the
[`dependency-hygiene.md`](workstreams/dependency-hygiene.md) workstream, and the
Node/Vitest guards for
[`test-unused-deps-producer.mjs`](../../tests/test-unused-deps-producer.mjs),
[`unused-deps-producer.test.mjs`](../../tests/unused-deps-producer.test.mjs),
[`test-audit-manifest-export-surface.mjs`](../../tests/test-audit-manifest-export-surface.mjs),
[`audit-manifest-export-surface.test.mjs`](../../tests/audit-manifest-export-surface.test.mjs),
[`test-audit-repo.mjs`](../../tests/test-audit-repo.mjs), and
[`audit-repo-artifact-brief.test.mjs`](../../tests/audit-repo-artifact-brief.test.mjs).
The tracker now separates the completed surface from future
dependency-specific configuration, lockfile semantics, and broader corpus
calibration: `review-unused` remains inspect-only evidence, not package removal
proof.

## [2026-05-24] verification | Package script runtime entry beta.56

Closed the package-script runtime entry surface. The beta.56 public install
confirmed that `tsx src/server.ts`, `tsx watch src/server.ts`, and
`node src/main.ts` seed `entry-surface.json` with package-script runtime
evidence, and the follow-up diagnostics slice now records unsupported wrappers
in `entry-surface.json.unsupportedScriptEntrypoints[]` without adding concrete
entry files.

The verification also pinned the argv boundary: in
`node src/main.ts src/config.ts`, only `src/main.ts` becomes entry evidence.
`src/config.ts` remains a script argument and may still appear in
`module-reachability.json.unreachableFiles` when no real import reaches it.

## [2026-05-24] verification | Dependency hygiene review surface beta.58

Closed the `unused-deps.json` review-surface slice against the beta.58 public
install. The installed package verification confirmed that
`manifest.json.unusedDependencies` mirrors review counts plus capped
package-name examples for navigation, while `audit-summary.latest.md` and
`audit-review-pack.latest.md` surface only review counts and artifact paths.
Package-edit wording is absent, and dependency hygiene evidence does not leak
into fix-plan, action-safety, dead-classify, SARIF, `SAFE_FIX`, `EXISTS`, or
`SAFE_CUE` lanes.

The install-cache `node_modules` check was also clarified. Public skill
packages intentionally lazy-install parser/runtime dependencies on first run in
`skills/lumin-repo-lens-lab`; with `LUMIN_REPO_LENS_NO_AUTO_INSTALL=1`, the
installed wrapper emits the manual
`npm ci --omit=dev --ignore-scripts --no-audit --fund=false` setup command.
Missing `node_modules` in a fresh plugin cache is therefore not a publish
failure by itself.

## [2026-05-24] implement | unused-deps Markdown review surface

Implemented the P2c wording slice from
[`unused-deps-review-surface.md`](../spec/unused-deps-review-surface.md).
`audit-summary.latest.md` and `audit-review-pack.latest.md` now surface
dependency hygiene as review-only counts that point to
`manifest.json.unusedDependencies` and `unused-deps.json`, while package names
stay in JSON evidence. Coverage is pinned by
[`test-audit-repo.mjs`](../../tests/test-audit-repo.mjs) and
[`audit-repo-artifact-brief.test.mjs`](../../tests/audit-repo-artifact-brief.test.mjs).

## [2026-05-24] implement | unused-deps manifest mirror

Implemented the P2b manifest-only slice from
[`unused-deps-review-surface.md`](../spec/unused-deps-review-surface.md).
`manifest.json.unusedDependencies` now mirrors shallow `unused-deps.json`
status, counts, reason distribution, and capped review-only examples while
leaving summary Markdown and review-pack wording untouched. Coverage is pinned
by
[`test-audit-manifest-export-surface.mjs`](../../tests/test-audit-manifest-export-surface.mjs)
and
[`audit-manifest-export-surface.test.mjs`](../../tests/audit-manifest-export-surface.test.mjs).

## [2026-05-24] design | unused-deps review surface

Added
[`unused-deps-review-surface.md`](../spec/unused-deps-review-surface.md) as the
P2 dependency hygiene wording and leakage contract before any
`unused-deps.json` Markdown surfacing. The spec links back to
[`unused-deps-producer.md`](../spec/unused-deps-producer.md), keeps
`review-unused` as inspect-only evidence, defines the
`manifest.json.unusedDependencies` mirror shape, names weak summary/review-pack
wording, and requires leakage guards for fix-plan, action-safety, SARIF, and
safe cue lanes before implementation.

## [2026-05-24] cleanup | WT-24 closure note

Refreshed
[`vitest-mirror-closure-audit.md`](vitest-mirror-closure-audit.md) and
[`vitest-mirror-goal.md`](vitest-mirror-goal.md) after the grouped Node runner
landed. The WT-24 closure state now records 165 Node suites, 176 focused Vitest
mirrors, two intentionally Node-authoritative parked suites, and
`npm run test:node:groups` as an opt-in maintainer shortcut rather than a
replacement for `npm test`. Source evidence is linked to
[`scripts/run-tests-grouped.mjs`](../../scripts/run-tests-grouped.mjs),
[`tests/test-run-tests-grouped.mjs`](../../tests/test-run-tests-grouped.mjs),
and [`tests/run-tests-grouped.test.mjs`](../../tests/run-tests-grouped.test.mjs).

## [2026-05-24] implement | grouped Node test runner

Added the opt-in grouped Node test runner described in
[`2026-05-24-grouped-node-test-runner-design.md`](../superpowers/specs/2026-05-24-grouped-node-test-runner-design.md).
The default `npm test` serial lane remains unchanged; the new
`npm run test:node:groups` command runs deterministic groups with bounded jobs,
serial execution inside each group, compact passing output, and failed-group
replay commands. The implementation is covered by
[`test-run-tests-grouped.mjs`](../../tests/test-run-tests-grouped.mjs) and
[`run-tests-grouped.test.mjs`](../../tests/run-tests-grouped.test.mjs). A local
dogfood run with `--jobs 3` passed 165 suites across 12 groups in 362.8 seconds.

## [2026-05-23] cleanup | pre-write cue-tier parked coverage

Clarified that `tests/test-pre-write-cue-tiers.mjs` remains parked only as a
direct Node-authoritative umbrella. Its current T1-T10 contracts are covered by
focused split mirrors for exact/signature safe cues, class-method cues,
suppressed diagnostics, service-operation cues, local-operation cues,
unavailable/policy cues, and file/token/inline cues. Future cue-tier behavior
still needs a fresh split review before a new Vitest mirror. Source links are
recorded in
[`vitest-pre-write-cue-tiers.md`](pilot-reviews/vitest-pre-write-cue-tiers.md)
and
[`vitest-mirror-closure-audit.md`](vitest-mirror-closure-audit.md).

## [2026-05-23] cleanup | parked remainder wording

Refreshed the parked remainder wording after the audit-repo full-profile split
mirror landed. The wiki now says the known `test-audit-repo.mjs` split mirrors
are complete while the direct umbrella suite remains intentionally
Node-authoritative; any future audit-repo product-pass behavior needs a fresh
split review before another mirror.

## [2026-05-23] implement | audit-repo full-profile Vitest mirror

Added `tests/audit-repo-full-profile-staleness.test.mjs` and
`npm run test:vitest:audit-repo-full-profile-staleness` as the focused mirror
for the O10a-O10e/H full-profile staleness/artifacts split track inside
`tests/test-audit-repo.mjs`. The mirror uses a real full-profile git fixture to
verify staleness, optional support artifacts, review-pack wording, shape-drift,
function-clone, and any-contamination evidence without absorbing scan-range,
lifecycle, producer-performance, blind-zone, ranking, or action-safety
semantics.

## [2026-05-23] review | audit-repo full-profile staleness

Added `pilot-reviews/vitest-audit-repo-full-profile-staleness.md` as the
suite-specific review for the O10a-O10e/H split track inside
`tests/test-audit-repo.mjs`. The review keeps the future mirror scoped to a
real full-profile git fixture that verifies staleness, optional support
artifacts, review-pack wording, shape-drift, function-clone, and
any-contamination evidence without absorbing scan-range, lifecycle,
producer-performance, blind-zone, ranking, or action-safety semantics.

## [2026-05-23] implement | audit-repo lifecycle artifacts Vitest mirror

Added `tests/audit-repo-lifecycle-artifacts.test.mjs` and
`npm run test:vitest:audit-repo-lifecycle-artifacts` as the focused mirror for
the O12 lifecycle artifact collection split track inside
`tests/test-audit-repo.mjs`. The mirror uses a real `audit-repo.mjs` temp-repo
run to verify `pre-write-advisory.latest.json`, timestamped
`any-inventory.pre.*.json`, and `canon-drift.json` artifact registration after
their opt-in lifecycle modes run.

## [2026-05-23] implement | audit-repo scan range Vitest mirror

Added `tests/audit-repo-scan-range.test.mjs` and
`npm run test:vitest:audit-repo-scan-range` as the focused mirror for the
O9/O11/O13 scan range and self-audit exclusion split track inside
`tests/test-audit-repo.mjs`. The mirror uses real `audit-repo.mjs` temp-repo
runs to verify user excludes, generated-artifact mode, production aliases, and
maintainer auto-excludes without absorbing lifecycle or staleness tracks.

## [2026-05-23] implement | audit-repo blind-zone Vitest mirror

Added `tests/audit-repo-blind-zones.test.mjs` and
`npm run test:vitest:audit-repo-blind-zones` as the focused mirror for the
B-series and O5/O6/O8 split track inside `tests/test-audit-repo.mjs`. The
mirror preserves unsupported-language, parser, CJS, resolver, generated-consumer,
affected-package, and blocked-absence confidence contracts while keeping the
umbrella Node suite authoritative for the remaining split tracks.

## [2026-05-23] review | audit-repo blind-zone confidence

Added `pilot-reviews/vitest-audit-repo-blind-zone-confidence.md` as the
suite-specific review for the B-series and O5/O6/O8 split track inside
`tests/test-audit-repo.mjs`. The review records fresh 97/97 passing Node
evidence and names the unsupported-language, parser, CJS, resolver,
generated-consumer, affected-package, and blocked-absence confidence contracts
that any future focused Vitest mirror must preserve.

## [2026-05-23] implement | P6 SAFE_FIX calibration mirror

Added `tests/p6-safe-fix-calibration.test.mjs` and
`npm run test:vitest:p6-safe-fix-calibration` as the focused mirror for
`tests/test-p6-safe-fix-calibration.mjs`. The mirror preserves all 15
calibration contracts covering candidate emission, runtime evidence, SAFE_FIX
ranking reachability, and P6 measurement readiness through the production
script chain.

## [2026-05-23] review | P6 SAFE_FIX calibration

Added `pilot-reviews/vitest-p6-safe-fix-calibration.md` as the
suite-specific review for `tests/test-p6-safe-fix-calibration.mjs`. The review
records all 15 calibration contracts covering static candidate emission,
runtime evidence merge, SAFE_FIX ranking reachability, and P6 measurement
readiness.

## [2026-05-23] implement | P6 member precision mirror

Added `tests/p6-member-precision.test.mjs` and
`npm run test:vitest:p6-member-precision` as the focused mirror for
`tests/test-p6-member-precision.mjs`. The mirror preserves all 12 namespace,
dynamic import binding, lexical shadowing, and conservative namespace
degradation contracts through the production `build-symbol-graph.mjs` path.

## [2026-05-23] review | P6 member precision

Added `pilot-reviews/vitest-p6-member-precision.md` as the suite-specific
review for `tests/test-p6-member-precision.mjs`. The review records all 12
member fan-in and lexical shadowing contracts, keeps the Node command
authoritative, and marks the suite ready only for a narrow production
`build-symbol-graph.mjs` mirror.

## [2026-05-22] implement | P6 measurement mirror

Added `tests/p6-measurement.test.mjs` and
`npm run test:vitest:p6-measurement` as the focused deadness/ranking
calibration mirror for `tests/test-p6-measurement.mjs`. The mirror preserves
all 26 P6 measurement contracts covering candidate counts, adjudication
denominators, readiness blockers, schema round-trip precedence,
dirty-worktree safety, multi-corpus merge behavior, and CLI artifact output.

## [2026-05-22] review | P6 measurement

Added `pilot-reviews/vitest-p6-measurement.md` as the suite-specific
deadness/ranking calibration review for `tests/test-p6-measurement.mjs`. The
review keeps `node tests/test-p6-measurement.mjs` authoritative, records fresh
26/26 passing evidence, and names the candidate-count, adjudication
denominator, readiness-gate, schema round-trip, dirty-worktree,
multi-corpus-merge, and CLI artifact contracts that any future focused Vitest
mirror must preserve.

## [2026-05-22] implement | precision corpus mirror

Added `tests/corpus.test.mjs` and `npm run test:vitest:corpus` as the
focused deadness/ranking precision-corpus mirror for
`tests/test-corpus.mjs`. The mirror preserves all 78 corpus assertions plus the
zero false-positive budget gate while running the real symbol graph,
dead-classify, export-action-safety, and rank-fixes producers.

## [2026-05-22] review | precision corpus

Added `pilot-reviews/vitest-corpus.md` as the suite-specific
deadness/ranking precision-corpus review for `tests/test-corpus.mjs`. The
review keeps `node tests/test-corpus.mjs` authoritative, records fresh 78/78
passing evidence plus the zero false-positive budget gate, and names the
dynamic import opacity, AST reference counting, resolver taint locality,
test-pinned contract, public/package surface, framework policy, and declaration
safety contracts that any future focused Vitest mirror must preserve.

## [2026-05-21] implement | rank fixes mirror

Added `tests/rank-fixes.test.mjs` and `npm run test:vitest:rank-fixes`
as the focused deadness/ranking action-proof mirror for
`tests/test-rank-fixes.mjs`. The mirror preserves tier ranking,
action-proof blockers, public deep-import risk, generated blind-zone
blockers, call-graph support, framework callback guards, muted candidates,
and `fix-plan.json` grouping while keeping the preserved Node suite
runnable.

## [2026-05-21] review | rank fixes

Added `pilot-reviews/vitest-rank-fixes.md` as the suite-specific
deadness/ranking action-proof review for `tests/test-rank-fixes.mjs`. The
review keeps `node tests/test-rank-fixes.mjs` authoritative, records fresh
45/45 passing evidence, and names the tier ranking, action-proof blocker,
public deep-import, generated blind-zone, call-graph support, framework
callback, muted candidate, and `fix-plan.json` grouping contracts that any
future focused Vitest mirror must preserve.

## [2026-05-21] implement | finding-local provenance mirror

Added `tests/finding-local-provenance.test.mjs` and
`npm run test:vitest:finding-local-provenance` as the focused
deadness/ranking provenance mirror for
`tests/test-finding-local-provenance.mjs`. The mirror preserves scoped
specifier matching, per-finding taint, resolver confidence,
generated-artifact blind-zone relevance, and ranking tier demotion while
keeping the preserved Node suite runnable.

## [2026-05-21] review | finding-local provenance

Added `pilot-reviews/vitest-finding-local-provenance.md` as the
suite-specific deadness/ranking provenance review for
`tests/test-finding-local-provenance.mjs`. The review keeps
`node tests/test-finding-local-provenance.mjs` authoritative, records fresh
48/48 passing evidence, and names the scoped specifier matching,
per-finding taint, resolver confidence, generated-artifact blind-zone, and
ranking tier contracts that any future focused Vitest mirror must preserve.

## [2026-05-21] implement | export action safety mirror

Added `tests/export-action-safety.test.mjs` and
`npm run test:vitest:export-action-safety` as the focused deadness/ranking
action-proof mirror for `tests/test-export-action-safety.mjs`. The mirror
preserves demote/delete safe actions, stronger-action blockers, local
value/type reference preservation, B-bucket declaration dependency demotion,
partial multi-declarator review behavior, re-export review behavior, and
module-marker insertion while keeping the preserved Node suite runnable.

## [2026-05-21] review | export action safety

Added `pilot-reviews/vitest-export-action-safety.md` as the suite-specific
deadness/ranking action-proof review for
`tests/test-export-action-safety.mjs`. The review keeps
`node tests/test-export-action-safety.mjs` authoritative, records fresh 14/14
passing evidence, and names the side-effect initializer, local value/type
reference, type deletion, B-bucket demotion, partial multi-declarator,
re-export-from-source, and module-marker patch contracts that any future
focused Vitest mirror must preserve.

## [2026-05-21] implement | namespace re-export deadness mirror

Added `tests/namespace-reexport-deadness.test.mjs` and
`npm run test:vitest:namespace-reexport-deadness` as the focused
deadness/ranking graph-lens mirror for
`tests/test-namespace-reexport-deadness.mjs`. The mirror preserves direct
namespace fan-in, chained namespace fan-in, unused sibling deadness,
broad-shadow absence for precise member usage, and opaque namespace escape
diagnostics while keeping the preserved Node suite runnable.

## [2026-05-21] review | namespace re-export deadness

Added `pilot-reviews/vitest-namespace-reexport-deadness.md` as the
suite-specific deadness/ranking graph-lens review for
`tests/test-namespace-reexport-deadness.mjs`. The review keeps
`node tests/test-namespace-reexport-deadness.mjs` authoritative, records fresh
12/12 passing evidence, and names the direct namespace, chained namespace,
unused sibling, broad-shadow, and opaque namespace escape diagnostic contracts
that any future focused Vitest mirror must preserve.

## [2026-05-21] implement | classify performance metadata mirror

Added `tests/classify-performance-metadata.test.mjs` and
`npm run test:vitest:classify-performance-metadata` as the focused Lane F
mirror for `tests/test-classify-performance-metadata.mjs`. The mirror preserves
classify-dead-exports performance metadata, AST file batching, text-zero
shortcuts, provenance cache entries, candidate-limit incompleteness,
time-budget degraded proposals, and file-size degradation behavior while
keeping the preserved Node suite runnable.

## [2026-05-21] review | classify performance metadata

Added `pilot-reviews/vitest-classify-performance-metadata.md` as the
suite-specific Lane F review for
`tests/test-classify-performance-metadata.mjs`. The review keeps
`node tests/test-classify-performance-metadata.mjs` authoritative, records
fresh 13/13 passing evidence, and names the performance metadata,
AST-batching, text-zero, candidate-limit, time-budget, file-size degradation,
and degraded unprocessed proposal contracts that any future focused Vitest
mirror must preserve.

## [2026-05-20] implement | any-inventory incremental mirror

Added `tests/any-inventory-incremental.test.mjs` and
`npm run test:vitest:any-inventory-incremental` as the focused Lane F mirror
for `tests/test-any-inventory-incremental.mjs`. The mirror preserves strict
any-inventory incremental cache identity, cold/warm public fact equivalence,
changed/deleted type escape evidence, scan-range invalidation from production
back to default include-tests behavior, malformed unrelated cache tolerance,
and disabled-cache metadata while keeping the preserved Node suite runnable.

## [2026-05-20] review | any-inventory incremental cache identity

Added `pilot-reviews/vitest-any-inventory-incremental.md` as the
suite-specific Lane F review for `tests/test-any-inventory-incremental.mjs`.
The review keeps `node tests/test-any-inventory-incremental.mjs`
authoritative, records fresh 13/13 passing evidence, and names the cold/warm,
changed/deleted file, scan-range invalidation, malformed-cache tolerance, and
disabled-cache contracts that any future focused Vitest mirror must preserve.

## [2026-05-20] implement | symbol-graph incremental mirror

Added `tests/symbol-graph-incremental.test.mjs` and
`npm run test:vitest:symbol-graph-incremental` as the focused Lane F mirror for
`tests/test-symbol-graph-incremental.mjs`. The mirror preserves strict symbol
graph incremental cache identity, cold/warm public fact equivalence, changed
consumer fan-in refresh, deleted definition cleanup, disabled-cache metadata,
legacy CJS export/require cache invalidation, stale JSON require opacity
removal, and old CJS extractor identity invalidation while keeping the
preserved Node suite runnable.

## [2026-05-20] review | symbol-graph incremental cache identity

Added `pilot-reviews/vitest-symbol-graph-incremental.md` as the suite-specific
Lane F review for `tests/test-symbol-graph-incremental.mjs`. The review keeps
`node tests/test-symbol-graph-incremental.mjs` authoritative, records fresh
13/13 passing evidence, and names the cold/warm, changed/deleted file, CJS
legacy-cache invalidation, dynamic require opacity, stale JSON require opacity,
and extractor identity contracts that any future focused Vitest mirror must
preserve.

## [2026-05-20] implement | shape-index incremental mirror

Added `tests/shape-index-incremental.test.mjs` and
`npm run test:vitest:shape-index-incremental` as the focused Lane F mirror for
`tests/test-shape-index-incremental.mjs`. The mirror preserves strict
shape-index incremental cache identity, cold/warm public fact equivalence,
current-run `observedAt` stamping, changed/deleted file evidence, disabled
cache metadata, and `audit-repo.mjs` forwarding of `--no-incremental` and
`--cache-root`.

## [2026-05-20] review | shape-index incremental cache identity

Added `pilot-reviews/vitest-shape-index-incremental.md` as the suite-specific
Lane F review for `tests/test-shape-index-incremental.mjs`. The review keeps
`node tests/test-shape-index-incremental.mjs` authoritative, records fresh
12/12 passing evidence, and names the cold/warm, changed/deleted file,
`observedAt`, and audit-repo incremental forwarding contracts that any future
focused Vitest mirror must preserve.

## [2026-05-20] cleanup | vitest closure inventory refresh

Refreshed `vitest-mirror-closure-audit.md`, `vitest-mirror-goal.md`, and the
test migration candidate board after the function-clone incremental and module
reachability mirrors landed. The active parked remainder now records 164 Node
test suites, 159 focused Vitest mirrors, and 14 Node-authoritative parked
suites.

## [2026-05-20] implement | module reachability mirror

Added `tests/module-reachability.test.mjs` and
`npm run test:vitest:module-reachability` as the focused Lane E graph-lens
mirror for `tests/test-module-reachability.mjs`. The mirror keeps runtime and
type reachability separate, preserves bounded-out uncertainty, and keeps
entry-unreachable SCCs surfaced as review evidence rather than export
`SAFE_FIX` proof.

## [2026-05-20] review | module reachability graph lens

Added `pilot-reviews/vitest-module-reachability.md` as the first Lane E
deadness/ranking graph-lens review page. The review keeps
`node tests/test-module-reachability.mjs` authoritative, records fresh 19/19
passing evidence, and names the runtime/type BFS, bounded-out, audit hook, and
entry-unreachable SCC review-evidence contracts that any future focused Vitest
mirror must preserve.

## [2026-05-20] implement | function clone incremental mirror

Added `tests/function-clone-incremental.test.mjs` and
`npm run test:vitest:function-clone-incremental` as the focused Lane F mirror
for `tests/test-function-clone-incremental.mjs`. The mirror keeps strict
incremental cache identity, current-run observedAt stamping, changed/deleted
file evidence, relPath move behavior, clear-cache behavior, disabled-cache
metadata, and mixed fresh/reused exact clone group rebuilds visible.

## [2026-05-19] review | function clone incremental cache identity

Added `pilot-reviews/vitest-function-clone-incremental.md` as the first
parked-suite dogfooding review after the mirror-lane closure. The review keeps
`node tests/test-function-clone-incremental.mjs` authoritative, records fresh
15/15 passing evidence, and names the local cache invalidation and global clone
group rebuild failures that any future focused Vitest mirror must preserve.

## [2026-05-19] implement | audit-repo manifest performance split

Added `tests/audit-repo-manifest-performance.test.mjs` and
`npm run test:vitest:audit-repo-manifest-performance` as the focused O0-O3 /
O1-O1f4 mirror from the parked audit-repo umbrella suite. The mirror keeps
output location notes, `manifest.json.performance`, `producer-performance.json`,
artifact sizes, artifact read/parse counters, orchestrator memory snapshots,
heavy producer phase counters, source-use resolver timings, and quick-profile
producer boundaries visible.

## [2026-05-19] review | audit-repo manifest performance split

Added `pilot-reviews/vitest-audit-repo-manifest-performance.md` as the reviewed
O0-O3/O1-O1f4 split track for `tests/test-audit-repo.mjs`. The page keeps the
future mirror scoped to manifest metadata, `producer-performance.json`, artifact
sizes, artifact read/parse counters, orchestrator memory snapshots, heavy
producer phase counters, and quick-profile producer boundaries.

## [2026-05-19] cleanup | audit-repo umbrella board gate

Updated the parked `tests/test-audit-repo.mjs` umbrella row after the
artifact-brief split had already landed. The board now points remaining
audit-repo work at separate review pages for blind zones, manifest evidence,
producer-performance counters, scan range, lifecycle artifacts, and staleness
instead of asking for another artifact-brief review page.

## [2026-05-19] implement | pre-write file token inline cue lane

Added `tests/pre-write-file-inline-cues.test.mjs` and
`npm run test:vitest:pre-write-file-inline` as the focused T7-T10 mirror from
the parked pre-write cue-tier suite. The mirror keeps exact file evidence as
claim-only `SAFE_CUE`, preserves important token stems, keeps inline-pattern
matches as review-only extraction cues, and keeps missing inline artifacts in
`unavailableEvidence[]`.

## [2026-05-19] implement | pre-write evidence gap cue lanes

Added `tests/pre-write-evidence-gaps.test.mjs` and
`npm run test:vitest:pre-write-evidence-gaps` as the focused T5-T6b mirror from
the parked pre-write cue-tier suite. The mirror keeps unavailable lookup results
in `unavailableEvidence[]` and keeps policy-excluded exact-symbol evidence in
suppressed evidence instead of cue cards.

## [2026-05-19] implement | pre-write local-operation cue adapter

Added `tests/pre-write-local-op-cues.test.mjs` and
`npm run test:vitest:pre-write-local-op-cues` as the focused T4h-T4j mirror
from the parked pre-write cue-tier suite. The mirror keeps promoted nested local
operations as review-only cue cards and keeps mutation-family mismatches in
suppressed evidence.

## [2026-05-19] implement | pre-write service-operation cue adapter

Added `tests/pre-write-service-op-cues.test.mjs` and
`npm run test:vitest:pre-write-service-op-cues` as the focused T4c-T4g mirror
from the parked pre-write cue-tier suite. The mirror keeps promoted
service-operation siblings as review-only cue cards and keeps muted,
class-method, and generated service-operation candidates in suppressed evidence.

## [2026-05-19] implement | pre-write suppressed cue diagnostics

Added `tests/pre-write-cue-muted.test.mjs` and
`npm run test:vitest:pre-write-cue-muted` as the focused T4-T4b mirror from the
parked pre-write cue-tier suite. The mirror keeps muted semantic and near-name
diagnostics in `suppressedCues[]` and separate from exact-symbol,
function-signature, class-method, service-operation, local-operation,
unavailable, file, token, and inline-pattern cue lanes.

## [2026-05-19] implement | pre-write class-method cue lane

Added `tests/pre-write-class-method-cues.test.mjs` and
`npm run test:vitest:pre-write-class-method-cues` as the focused T3c-T3d mirror
from the parked pre-write cue-tier suite. The mirror keeps `classMethodIndex`
review cue adaptation separate from exact-symbol, function-signature,
suppressed, service-operation, local-operation, unavailable, file, token, and
inline-pattern cue lanes.

## [2026-05-18] implement | pre-write exact and signature safe cues

Added `tests/pre-write-exact-safe-cues.test.mjs` and
`npm run test:vitest:pre-write-exact-safe-cues` as the focused T1-T3 mirror
from the parked pre-write cue-tier suite. The mirror keeps exact-symbol and
function-signature `SAFE_CUE` adaptation separate from class-method, suppressed,
service-operation, local-operation, unavailable, file, token, and inline-pattern
cue lanes.

## [2026-05-18] review | pre-write file token and inline cues

Added `pilot-reviews/vitest-pre-write-file-token-inline-cues.md` as the
file/token/inline split-track review from the parked pre-write cue-tier suite.
The review isolates assertions T7-T10: exact file hits create `SAFE_CUE`
evidence, token policy preserves important stems, inline-pattern matches remain
review-only extraction cues, and missing inline-pattern artifacts stay
`unavailableEvidence[]`.

## [2026-05-18] review | pre-write unavailable and policy cues

Added `pilot-reviews/vitest-pre-write-unavailable-policy-cues.md` as the
unavailable/policy-excluded split-track review from the parked pre-write
cue-tier suite. The review isolates assertions T5-T6b: missing artifacts remain
`unavailableEvidence[]`, and policy-excluded exact evidence stays suppressed
without creating cue cards, `SAFE_CUE`, `EXISTS`, or `SAFE_FIX` proof.

## [2026-05-18] review | pre-write class-method cue lane

Added `pilot-reviews/vitest-pre-write-class-method-cues.md` as the
class-method split-track review from the parked pre-write cue-tier suite. The
review isolates assertions T3c-T3d: `classMethodIndex` near-name evidence may
create only review cue cards, must cite `classMethodIndex`, and must not become
top-level `defIndex`, `SAFE_CUE`, `EXISTS`, or `SAFE_FIX` proof.

## [2026-05-18] review | pre-write exact and signature safe cues

Added `pilot-reviews/vitest-pre-write-exact-safe-cues.md` as the
exact/signature split-track review from the parked pre-write cue-tier suite. The
review isolates assertions T1-T3: exact identities and normalized function
signatures may create `SAFE_CUE` records, exact-symbol safe cues stay
claim-only, and mixed safe/review candidates render at `AGENT_REVIEW_CUE`
without dropping either cue record.

## [2026-05-18] review | pre-write local-operation cue adapter

Added `pilot-reviews/vitest-pre-write-local-operation-cues.md` as the third
split-track review from the parked pre-write cue-tier suite. The review
isolates assertions T4h-T4j: promoted nested local operations create only
`AGENT_REVIEW_CUE` cards with copied container/surface/locality policy evidence,
and mutation-family mismatches stay muted in `suppressedCues[]` instead of
rendering cue cards or safe/action proof.

## [2026-05-18] review | pre-write service-operation cue adapter

Added `pilot-reviews/vitest-pre-write-service-operation-cues.md` as the second
split-track review from the parked pre-write cue-tier suite. The review
isolates assertions T4c-T4g: promoted service-operation siblings create only
`AGENT_REVIEW_CUE` cards with copied policy evidence, original suppressed
diagnostics remain muted, and muted/class-method/generated candidates stay out
of `cueCards[]`.

## [2026-05-18] review | pre-write suppressed cue diagnostics

Added `pilot-reviews/vitest-pre-write-cue-suppressed-diagnostics.md` as the
first split-track review from the parked pre-write cue-tier suite. The review
isolates assertions T4-T4b: suppressed semantic and near-name candidates must
stay `MUTED` in `suppressedCues[]`, preserve reason/lane/score/distance/locality
metadata, and never create `cueCards[]` or safe/action proof.

## [2026-05-18] review | pre-write cue-tier split tracks

Added `pilot-reviews/vitest-pre-write-cue-tiers.md` after reviewing the
preserved `tests/test-pre-write-cue-tiers.mjs` Node command. The suite remains
parked as a direct Vitest mirror because it protects exact safe cues,
class-method review cues, suppressed diagnostics, service-operation sibling
cues, local-operation sibling cues, unavailable evidence, policy exclusions,
file cues, token policy, and inline-pattern cues in one adapter boundary.
Future work must split one cue lane at a time before adding a focused mirror.

## [2026-05-18] consolidation | dependency hygiene workstream

Added the dependency hygiene workstream and unused-deps Vitest pilot review
after beta.57 public verification. The wiki now records `unused-deps.json` as a
review-only artifact chain that depends on package-script runtime entry
evidence and must not leak into package edits, fix-plan/SARIF output, Markdown
removal wording, or `SAFE_FIX` claims.

## [2026-05-18] verification | unused dependencies producer beta.57

Verified the installed beta.57 public package against a temporary dependency
hygiene fixture. The run emitted `unused-deps.json` with schema
`unused-deps.v1`, policy `unused-deps-review-policy-v1`, and complete status.
Observed imports classified as `used`, `tsx` was muted by package-script tool
evidence, `@types/node` stayed muted as ambient types, and unobserved packages
remained `review-unused` without deletion or safe-fix wording in fix-plan,
action-safety, SARIF, summary Markdown, or review-pack Markdown.

## [2026-05-18] spec | unused dependencies producer

Added `docs/spec/unused-deps-producer.md` as the WT-25 design seed. The spec
turns the dogfood finding "external import and manifest data exists, but no
producer surfaces unused dependencies" into a review-only dependency hygiene
artifact plan. It explicitly depends on the package-script runtime entry
baseline, keeps peer/optional/@types/script/framework explanations
conservative, and forbids package removal or `SAFE_FIX` claims in the first
slices.

## [2026-05-18] implementation | package script runtime entry surface

Added the first runtime package-script entry extractor for `tsx`, `ts-node`,
`node`, and `bun` commands that directly name JS/TS entry files. The
entry-surface regression fixture now proves `tsx src/server.ts` enters
`scriptEntrypointFiles` / `entryFiles`, seeds `module-reachability.json`, and no
longer appears as unreachable, while unknown script wrappers remain unmodeled.

## [2026-05-18] spec | package script runtime entry surface

Added `docs/spec/package-script-runtime-entry-surface.md` after dogfood showed
that runtime package scripts such as `tsx src/server.ts` can be missed by the
current entry-surface extractor. The spec separates the urgent
false-unreachable reachability gap from the broader `unused-deps` producer
idea, keeps the first slice conservative, and requires script evidence to feed
`entryFiles` without becoming dead-export or `SAFE_FIX` proof.

## [2026-05-17] verification | WT-23 service operation type-name filter beta.55

Verified the installed beta.55 public package against the same VNplayer
pre-write corpus used for beta.54. TypeScript-only service-operation false
positives such as `ListLibraryDocsOptions` and `ListLibraryOutlineOptions` no
longer render as related service-operation cues; service-operation review lines
dropped from 2 to 0. The local-operation lane stayed stable at nine review-only
cues, no `SAFE_CUE`, `EXISTS`, or `SAFE_FIX` cue leaked, and muted service/local
evidence stayed out of default Markdown.

## [2026-05-17] verification | WT-23 local operation support reason beta.54

Verified the installed beta.54 public package against the VNplayer
`createRepository()` corpus. All nine promoted local-operation review cues now
carry `local-operation-same-file-domain-overlap` from policy evidence through
cue evidence and Markdown, replacing the beta.53 `unknown` fallback. The run
kept the cue tier review-only, emitted no `SAFE_CUE`, `EXISTS`, or `SAFE_FIX`,
and kept muted local-operation evidence out of default Markdown.

## [2026-05-17] implementation | WT-23 local operation support reason

Added a stable `local-operation-same-file-domain-overlap` support reason for
promoted `localOperationSiblingPolicy` entries. This keeps local-operation cue
Markdown from falling back to `unknown` while preserving the review-only cue
lane and the existing service/local policy separation.

## [2026-05-17] calibration | WT-23 VNplayer local operation corpus

Recorded the beta.53 VNplayer corpus rerun for the WT-23 local-operation cue
bridge. The installed public package rendered nine review-only local-operation
cues across four read/query intents, produced zero promoted cues for the
mutation intent, leaked no `SAFE_CUE`, `EXISTS`, or `SAFE_FIX` claims, and kept
muted local-operation evidence out of default Markdown. The decision is
`useful-enough` for the v1 bridge with follow-up for the `unknown` support
reason and a separate service-policy type-name false positive.

## [2026-05-17] verification | WT-23 local operation cue beta.53

Verified the installed beta.53 public package through the runtime
`audit-repo.mjs --pre-write` entrypoint, not source-only inspection. The
temporary `createRepository()` fixture confirmed `getWorld` renders as a
review-only local-operation `AGENT_REVIEW_CUE`, never as `SAFE_CUE`, `EXISTS`,
or `SAFE_FIX`, while muted local-operation evidence stays in JSON and remains
hidden from default Markdown.

## [2026-05-16] implementation | WT-23 local operation index

Added the first artifact-only WT-23 local operation index guard. Nested
read/query operations inside exported repository factories now appear in
`symbols.json.preWriteLocalOperationIndex` while staying out of `defIndex`,
`classMethodIndex`, formal lookup result lanes, and mutation/generic helper
surfaces.

## [2026-05-16] planning | WT-23 nested service-operation surface

Added the WT-23 nested local service-operation surface spec. The design targets
VNplayer-style repository factories without adding nested functions to
`defIndex`, dead-export ranking, `SAFE_FIX`, or `EXISTS`; unavailable local
operation evidence stays non-absence evidence.

## [2026-05-16] calibration | WT-23 corpus run

Recorded the first WT-23 service-operation corpus calibration report across
VNplayer-main and hono-main. The CLI route produced zero
service-operation cue cards because name intents do not preserve owner-file
locality. Owner-aware controls showed useful Hono helper siblings, while
VNplayer's relevant repository operations are mostly nested inside
`createRepository()` and outside the current `defIndex` service-operation
surface.

## [2026-05-16] planning | WT-23 corpus calibration

Added the WT-23 service-operation corpus calibration worksheet. The plan keeps
P2b review cues unchanged, requires at least one service-heavy app and one
library/noise-heavy corpus before policy expansion, and blocks mutation-family,
signature-weighted, or threshold changes until a reviewed report supports them.

## [2026-05-16] implementation | Audit repo artifact brief Vitest mirror

Added the focused Vitest mirror for the reviewed `tests/test-audit-repo.mjs`
artifact brief split track. The mirror covers A0/A0pre renderer and option
contracts plus O4/O7/O10c2 real `audit-repo.mjs` artifact-output behavior while
keeping the umbrella Node suite authoritative.

## [2026-05-16] review | Audit repo artifact brief split track

Reviewed the first `test-audit-repo.mjs` split track for future focused mirror
work. The new page scopes the artifact brief/review-pack track to A0, O4, O7,
and O10c2 assertions, separates direct renderer fixtures from real
`audit-repo.mjs` runs, and keeps the umbrella Node suite authoritative.

## [2026-05-16] review | Audit repo split-track dogfooding

Reviewed the parked `tests/test-audit-repo.mjs` umbrella suite through the
parked-suite dogfooding rules. The new split-track page keeps the suite
Node-authoritative, maps the A/B/O/H assertion clusters into future review
tracks, and recommends starting with the artifact brief/review-pack track before
any focused mirror or helper extraction.

## [2026-05-16] design | Parked suite dogfooding

Added a dogfooding concept page for the 16 Node-authoritative suites left after
the WT-24 Vitest mirror lane. The page keeps direct mirrors blocked, maps the
parked suites into review lanes, and defines the review/helper contract required
before future split, helper, or runner work.

## [2026-05-16] audit | Vitest mirror lane closure

Added a closure audit for the WT-24 mirror lane. The audit maps the remaining
Node-authoritative suites to parked risk categories, records the
`test-incremental.mjs` mirror-name exception, and defines the review-page gate
required before any parked analyzer-sensitive suite may move to Vitest.

## [2026-05-16] implementation | Symlink aliasing Vitest mirror

Added a focused Vitest mirror for `tests/test-symlink-aliasing.mjs`. The mirror
keeps the original Node entrypoint runnable, preserves the local Windows
symlink-privilege skip path, and mirrors the resolver realpath assertions for
platforms that can create symlinks.

## [2026-05-16] review | Symlink aliasing Vitest mirror

Reviewed `tests/test-symlink-aliasing.mjs` as a focused Lane D resolver/surface
mirror candidate. The preserved Node command reports a clean Windows privilege
skip locally and keeps the Linux/Developer-Mode realpath assertions scoped to
resolver canonicalization, not broader resolver expansion or deadness proof.

## [2026-05-16] implementation | Classification gates Vitest mirror

Added a focused Vitest mirror for `tests/test-classification-gates.mjs`. The
mirror preserves canonical type/helper/topology/naming label sets, classifier
rule precedence, facade import guards, and canon-drift parser contracts while
keeping the original Node entrypoint runnable.

## [2026-05-16] review | Classification gates Vitest mirror

Reviewed `tests/test-classification-gates.mjs` as a Lane B canon classifier
matrix mirror candidate. The preserved Node command passes with 105 assertions
covering type/helper/topology/naming labels, rule precedence, facade import
guards, and canon-drift parser contracts.

## [2026-05-16] review | Audit repo umbrella split

Reviewed `tests/test-audit-repo.mjs` after the Python conventions mirror. The
Node suite still passes with 97 assertions, but it mixes artifact-summary,
blind-zone, manifest, performance, scan-range, lifecycle, and staleness
contracts. Added a split/park review so the goal track does not create one
oversized `audit-repo.test.mjs` mirror.

## [2026-05-16] implementation | Python conventions Vitest mirror

Added a focused Vitest mirror for `tests/test-python-conventions.mjs`. The
mirror preserves Python self-reference import resolution, `__all__`
public-surface filtering, framework decorator dispatch evidence, dunder
runtime-dispatch exclusion, and the `python3` availability skip while keeping
the original Node entrypoint runnable.

## [2026-05-16] review | Python conventions Vitest mirror

Reviewed `tests/test-python-conventions.mjs` as the next single-suite Lane D
surface candidate. The review keeps the future mirror focused on Python
self-reference imports, `__all__` public-surface filtering, framework decorator
registration, dunder runtime-dispatch exclusion, and the current `python3`
availability skip while keeping symlink aliasing, generated/framework resource
packs, deadness/ranking, resolver expansion, and performance/incremental cache
identity out of scope.

## [2026-05-16] implementation | Entry surface artifact Vitest mirror

Added a focused Vitest mirror for `tests/test-entry-surface-artifact.mjs`.
The mirror preserves `entry-surface.json` entry union evidence, quick audit
pipeline hook coverage, HTML static-root blind-zone confidence limits, nested
HTML app-root resolution, phantom extension-probe prevention, and excluded HTML
scan-policy behavior while keeping the original Node entrypoint runnable.

## [2026-05-16] review | Entry surface artifact Vitest mirror

Reviewed `tests/test-entry-surface-artifact.mjs` as the next Lane D
surface-artifact candidate. The review keeps the future mirror focused on
`entry-surface.json` public/script/HTML/framework/config entries, quick audit
pipeline hook evidence, HTML-entry blind-zone confidence limits, nested HTML
root resolution, phantom extension-probe prevention, and excluded HTML scan
policy while keeping resolver expansion, reachability ranking, SAFE_FIX action
proof, generated/framework resource packs, Python conventions, symlink
resolution, and performance/incremental cache identity out of scope.

## [2026-05-16] implementation | Pre-write advisory lifecycle Vitest mirror batch

Added focused Vitest mirrors for `tests/test-pre-write-advisory-artifact.mjs`,
`tests/test-pre-write-bootstrap.mjs`, `tests/test-pre-write-cli.mjs`,
`tests/test-pre-write-drift.mjs`, and
`tests/test-pre-write-integration.mjs`. The mirrors preserve advisory
id/hash/artifact writes, prerequisite support flags, direct CLI cold-cache and
unavailable-evidence behavior, pure canonical drift projection, and direct
end-to-end advisory rendering while keeping the original Node entrypoints
runnable.

## [2026-05-16] review | Pre-write advisory lifecycle Vitest mirror batch

Reviewed five direct pre-write advisory lifecycle suites for a future Lane C
Vitest mirror batch: `tests/test-pre-write-advisory-artifact.mjs`,
`tests/test-pre-write-bootstrap.mjs`, `tests/test-pre-write-cli.mjs`,
`tests/test-pre-write-drift.mjs`, and
`tests/test-pre-write-integration.mjs`. The review keeps the future mirror
focused on advisory id/hash/artifact writes, prerequisite support flags, direct
CLI cold-cache and unavailable-evidence behavior, pure canonical drift
projection, and direct end-to-end advisory rendering while keeping lookup
policy, cue-tier promotion, renderer wording, resolver behavior,
deadness/ranking, generated/framework surfaces, and performance/incremental
cache identity out of scope.

## [2026-05-16] review | Pre-write inline extraction Vitest mirror batch

Reviewed `tests/test-inline-pattern-index.mjs` and
`tests/test-pre-write-inline-patterns.mjs` for a future Lane C Vitest mirror
batch. The review keeps the batch focused on `inline-patterns.json` repeated
catch-block artifact evidence, deterministic occurrence grouping,
pre-write-only `inline-extraction` review cues, and unavailable-evidence
behavior while keeping cue-tier promotion, name lookup thresholds,
deadness/ranking, resolver behavior, function clone semantics, and
performance/incremental cache identity out of scope.

## [2026-05-16] implementation | Pre-write inline extraction Vitest mirror batch

Added focused Vitest mirrors for `tests/test-inline-pattern-index.mjs` and
`tests/test-pre-write-inline-patterns.mjs`. The mirrors preserve
`inline-patterns.json` schema/support flags, threshold policy metadata,
deterministic catch-block grouping, noisy-catch suppression, pre-write
`inline-extraction` review cues, and unavailable-evidence behavior while
keeping cue-tier promotion, name lookup thresholds, deadness/ranking, resolver
behavior, function clone semantics, and performance/incremental cache identity
out of scope.

## [2026-05-16] review | Aliased export classification Vitest mirror

Reviewed `tests/test-alias.mjs` for a future single-suite Vitest mirror. The
review keeps the batch focused on aliased export-specifier local-name evidence,
specifier-aware dead-export action wording, local-reference counts, and
local-also-dead signaling while keeping broader deadness/ranking, namespace
reachability, public API blockers, resolver behavior, and SAFE_FIX calibration
out of scope. This review also corrects the current mirror inventory to 134
Vitest mirrors and 29 unmigrated Node suites because
`incremental-legacy-cache.test.mjs` is a renamed mirror for
`tests/test-incremental.mjs`, not a separate Node-suite match.

## [2026-05-16] implementation | Aliased export classification Vitest mirror

Added a focused Vitest mirror for `tests/test-alias.mjs`. The mirror preserves
aliased export-specifier `localName` evidence, non-aliased local-name noise
suppression, specifier-aware action wording, local-reference counts, and
local-also-dead signaling while keeping broader deadness/ranking, namespace
reachability, public API blockers, resolver behavior, and SAFE_FIX calibration
out of scope.

## [2026-05-16] review | Pre-write render Vitest mirror

Reviewed `tests/test-pre-write-render.mjs` for a future single-suite Vitest
mirror. The review keeps the batch focused on advisory Markdown/JSON renderer
wording, citation coverage, grounded/degraded/unavailable evidence lanes,
canonical drift placement, planned type-escape rendering, cue-card rendering,
muted cue suppression, service-operation sibling review wording, and
`renderJson` pass-through shape while keeping lookup policy, cue-tier
promotion, resolver behavior, deadness/ranking, and audit orchestration out of
scope.

## [2026-05-16] implementation | Pre-write render Vitest mirror

Added a focused Vitest mirror for `tests/test-pre-write-render.mjs`. The mirror
preserves advisory Markdown/JSON renderer wording, citation coverage,
grounded/degraded/unavailable evidence lanes, canonical drift placement,
planned type-escape rendering, cue-card rendering, muted cue suppression,
service-operation sibling review wording, and `renderJson` pass-through shape
while keeping lookup policy, cue-tier promotion, resolver behavior,
deadness/ranking, and audit orchestration out of scope.

## [2026-05-15] review | Type escape evidence Vitest mirror batch

Reviewed two type-escape evidence suites for a future Vitest mirror batch:
`tests/test-extract-ts-escapes.mjs` and `tests/test-any-inventory.mjs`. The
review keeps the batch focused on canonical escape-kind emission, precedence,
code-shape normalization, occurrence keys, exported-identity ownership,
parse-error completeness, default/test versus production scan scope,
shell-sensitive paths, required fact fields, and custom artifact naming while
keeping `any-inventory` incremental cache identity, pre-write inventory hook
stamping, resolver behavior, deadness/ranking, and action-safety out of scope.

## [2026-05-15] implementation | Type escape evidence Vitest mirror batch

Added focused Vitest mirrors for the reviewed type escape evidence suites:
`tests/test-extract-ts-escapes.mjs` and `tests/test-any-inventory.mjs`. The
mirrors preserve canonical escape-kind emission, precedence, code-shape
normalization, occurrence keys, exported-identity ownership, parse-error
completeness, default/test versus production scan scope, shell-sensitive paths,
required fact fields, and custom artifact naming while keeping
`any-inventory` incremental cache identity, pre-write inventory hook stamping,
resolver behavior, deadness/ranking, and action-safety out of scope.

## [2026-05-12] scaffold | maintainer wiki and test reform contract

Created the first maintainer wiki scaffold. The initial pages record the wiki
boundary, workstream map, core evidence concepts, and the test reform rule that
future TDD should fail on concrete edge cases rather than missing helpers.

## [2026-05-12] inventory | pre-write test family

Added a risk-based inventory for pre-write-related suites. The inventory names
each suite's protected invariant and the edge case or negative guard that should
survive future test reform.

## [2026-05-12] inventory | resolver test family

Added a risk-based inventory for resolver-related suites. The inventory names
resolved, candidate, unsupported-family, generated, output-layout, dynamic
module, and workspace resolver invariants before any test-file movement.

## [2026-05-12] inventory | deadness test family

Added a risk-based inventory for deadness, reachability, ranking, action-safety,
consumer extraction, and SAFE_FIX calibration suites. The inventory separates
review evidence from automated action proof before any test-file movement.

## [2026-05-12] inventory | performance test family

Added a risk-based inventory for performance, incremental cache, scanner,
producer measurement, and resolver-cache-overlap suites. The inventory separates
measurement evidence from correctness claims before any test-file movement.

## [2026-05-12] inventory | public-package test family

Added a risk-based inventory for plugin package, skill package, publish, public
CI, export-surface boundary, threshold policy, generated docs, behavior corpus,
and installed-package verification evidence before any test-file movement.

## [2026-05-12] comparison | fixture shapes across inventories

Added a fixture-shape comparison page that names repeated temporary repo,
resolver unsupported-family, generated/framework surface, consumer/member,
incremental, public/internal package, and Markdown mirror shapes before any
test-file movement.

## [2026-05-12] spec | shared temporary repo fixture helper

Added a setup-only shared fixture helper spec that limits the first test-reform
extraction to temporary repo creation, file/JSON helpers, safe path containment,
and cleanup before any broad test movement.

## [2026-05-12] implementation | shared temporary repo fixture helper

Added the setup-only temporary repo fixture helper with path containment,
JSON/file helper, and cleanup tests, then migrated only the low-risk saved-answer
behavior corpus verifier temp directory setup.

## [2026-05-12] concept | structure review charter

Added a structure review charter for shape, function, and helper decisions so
future suite reform can name boundaries, anti-patterns, failure modes, and the
first safe fix before moving code.

## [2026-05-12] implementation | second helper migration

Migrated the generated test README suite to the setup-only temp repo fixture
helper. The suite keeps its drift, regeneration, count-leak, and maintainer-note
assertions local while sharing only temporary repo setup and cleanup.

## [2026-05-12] tracking | wiki milestones

Added a wiki milestone board that records completed wiki/test-reform slices,
the setup-only helper migration boundary, and the gate that Vitest or Bun test
runner work needs a spec before implementation.

## [2026-05-12] spec | test runner migration

Added the test runner migration spec. It selects Vitest as the first pilot while
preserving Node entrypoints, keeps Bun as a parked future evaluation, and records
rollback and CI boundaries before any runner implementation.

## [2026-05-12] implementation | Vitest pilot

Added the first Vitest pilot suite for the setup-only temp repo fixture helper.
The pilot keeps the existing Node test entrypoint intact, adds a focused
`npm run test:vitest` command, and leaves Bun parked as a future evaluation.

## [2026-05-12] review | Vitest pilot

Reviewed the first Vitest pilot before any further runner migration. The review
records that Vitest improved focused execution and per-case assertion reporting
for the setup-only temp repo helper while preserving the Node test entrypoint,
`npm test`, and public package runtime boundaries.

## [2026-05-12] implementation | behavior corpus Vitest pilot

Added a parallel Vitest suite for the behavior corpus verifier while keeping
the Node verifier entrypoint intact. The pilot covers jargon rejection,
caveated dead-export wording, overclaim rejection, CLI success/failure, and
read-trace missing-artifact behavior. `vitest.config.mjs` scopes the runner to
`tests/*.test.mjs` so behavior corpora and generated fixture repositories are
not accidentally collected as test suites.

## [2026-05-12] review | behavior corpus Vitest pilot

Reviewed the behavior corpus Vitest pilot before further runner migration. The
review records the saved-answer invariants, the preserved Node verifier
entrypoint, the focused Vitest command, and the discovery-scope guard that keeps
behavior corpora and generated fixture repositories as test data rather than
test suites.

## [2026-05-12] implementation | generated test README Vitest pilot

Added a parallel Vitest suite for the generated test README drift guard while
keeping the Node test entrypoint intact. The pilot covers check-mode success,
drift failure wording, regeneration, do-not-edit/count-leak guards, maintainer
note surfacing, and real README hermeticity.

## [2026-05-12] review | generated test README Vitest pilot

Reviewed the generated test README Vitest pilot before further runner
migration. The review records the generated README drift invariant, preserved
Node test entrypoint, repository-level doc check, focused Vitest command, and
hermetic fixture boundary.

## [2026-05-12] planning | test migration candidate board

Added a candidate board for future runner migration. The board records reviewed
Vitest pilots, identifies test-harness verifier suites as the next reviewable
low-risk candidates, and parks pre-write, resolver, deadness, ranking,
performance, and public-package-sensitive suites until their invariants and
failure boundaries have review pages.

## [2026-05-12] review | citation verifier Vitest pilot candidate

Reviewed `tests/test-citation-verifier.mjs` as the next low-risk Vitest pilot
candidate. The review records the grounded-citation invariant, current Node
entrypoint, proposed focused Vitest command, concrete failure cases, CLI
behavior, stdin behavior, and temporary artifact fixture boundary before any
runner implementation.

## [2026-05-12] implementation | citation verifier Vitest pilot

Added a parallel Vitest suite for the grounded citation verifier while keeping
`node tests/test-citation-verifier.mjs` intact. The pilot covers valid scalar,
bracket, length, object, and root-package citations plus mismatched values,
missing paths, placeholder values, unfalsifiable citations, CLI failure, and
stdin behavior.

## [2026-05-13] review | refactor-plan verifier Vitest pilot candidate

Reviewed `tests/test-refactor-plan-verifier.mjs` as the next low-risk Vitest
pilot candidate. The review records the refactor-plan section, evidence,
pre-write handoff, coding-agent prompt, tone, CLI success/failure, and temporary
Markdown fixture boundaries before any runner implementation.

## [2026-05-13] implementation | refactor-plan verifier Vitest pilot

Added a parallel Vitest suite for the refactor-plan verifier while keeping
`node tests/test-refactor-plan-verifier.mjs` intact. The pilot covers valid
SHORT and FULL plans plus missing pre-write handoff, missing coding-agent
prompt, raw JSON, discouraging tone, missing evidence anchor, CLI success, and
CLI failure behavior.

## [2026-05-13] review | maintainer-scripts Vitest pilot candidate

Reviewed `tests/test-maintainer-scripts.mjs` as the next low-risk Vitest pilot
candidate. The review records that this suite should keep source-text guards
for child-process spawn error handling and optional JSON read safety rather
than introducing broader script/package behavior fixtures in the pilot.

## [2026-05-13] implementation | maintainer-scripts Vitest pilot

Added a parallel Vitest suite for the maintainer-scripts source guards while
keeping `node tests/test-maintainer-scripts.mjs` intact. The pilot covers
explicit `spawnSync(...).error` handling in the syntax and test runners plus
optional JSON read safety in the public-package publisher.

## [2026-05-13] review | threshold policy drift guard Vitest pilot candidate

Reviewed `tests/test-threshold-policy-drift-guard.mjs` as the next Vitest pilot
candidate. The review records that threshold policy migration must preserve the
exact ordered snapshot of policy ids, versions, classes, threshold hashes,
calibration corpora, and calibration notes so numeric threshold changes cannot
silently pass without an explicit policy snapshot review.

## [2026-05-13] implementation | threshold policy drift guard Vitest pilot

Added a parallel Vitest suite for the threshold policy drift guard while
keeping `node tests/test-threshold-policy-drift-guard.mjs` intact. The pilot
mirrors the ordered policy-id check and the exact threshold policy snapshot so
threshold version, hash, or calibration drift still requires explicit review.

## [2026-05-13] tracking | wiki v1 consolidation

Updated the wiki overview, milestones, and test migration candidate board after
WT-23 public verification and the service-operation sibling cue policy spec.
The wiki is now treated as a maintained v1 index rather than only a scaffold,
while analyzer-sensitive runner migration remains gated behind review pages and
active policy slices.

## [2026-05-13] tracking | WT-23 P2 cue readiness

Updated the service-operation sibling cue spec, work tracker, pre-write
workstream, and test migration board after the beta.48 P1 policy object landed.
P2 cue rendering is now gated behind a focused fixture matrix, review-only
wording contract, and corpus checklist before any `AGENT_REVIEW_CUE` behavior
changes.

## [2026-05-13] implementation | WT-23 P2a JSON cue cards

Added the first P2 service-operation sibling cue-tier adapter. The JSON cue
layer now copies `serviceOperationSiblingPolicy.promoted[]` into review-only
`AGENT_REVIEW_CUE` cards, preserves suppressed diagnostics, mirrors policy
`muted[]` entries into `suppressedCues[]`, and keeps generated/framework and
class-method candidates out of the service-operation sibling cue lane.

## [2026-05-13] implementation | WT-23 P2b Markdown service cues

Added default Markdown wording for `service-operation-sibling` review cues.
Rendered rows now say `Review related service operation`, cite the P2 policy
evidence path, show shared domain tokens, operation family, locality, and
supporting suppressed reasons, while keeping muted policy entries hidden by
default and avoiding stronger reuse/safety wording.

## [2026-05-14] verification | WT-23 P2b beta.50 public install

Recorded beta.50 public install verification for WT-23 P2b. The installed
renderer produced the expected `Review related service operation` Markdown
row, cited `pre-write-advisory.json /
lookups[].serviceOperationSiblingPolicy.promoted`, preserved
`heuristic-review` / `AGENT_REVIEW_CUE`, hid muted policy entries by default,
and kept the service-operation cue body free of reuse/equivalence/safety
wording. See
`docs/lab/wt23-beta50-service-operation-markdown-verification-2026-05-14.md`.

## [2026-05-14] implementation | pre-write lookup-name Vitest pilot

Added a parallel Vitest mirror for `tests/test-pre-write-lookup-name.mjs` while
keeping the Node suite runnable. The mirror covers exact identity, canonical
lookup, fan-in capability states, suppressed near/semantic diagnostics,
service-operation sibling policy evidence, noise-floor behavior, and resolver
confidence demotion without migrating cue-tier or Markdown rendering semantics.

## [2026-05-14] implementation | pre-write suite README descriptions

Added explicit generated `tests/README.md` descriptions for the pre-write suite
family and pinned the generator with a regression check so pre-write suites do
not return to anonymous maintainer-note entries after the wiki inventory named
their protected invariants.

## [2026-05-14] implementation | complete suite README descriptions

Added explicit generated `tests/README.md` descriptions for the remaining
anonymous JSONC edge-case and mode-dispatch suites. The generator is now pinned
with Node and Vitest guards so the current suite inventory cannot keep
maintainer-note description gaps checked in.

## [2026-05-14] review | Node imports unsupported Vitest candidate

Added the first resolver unsupported-family Vitest pilot review page for
`tests/test-node-imports-unsupported.mjs`. The review names the Node `#imports`
contracts that a future mirror must preserve: unsupported output levels, no
fake graph edges, resolver diagnostics lanes, candidate preservation, and
explicit blind-zone scope.

## [2026-05-14] implementation | Node imports unsupported Vitest pilot

Added a parallel Vitest mirror for
`tests/test-node-imports-unsupported.mjs` while keeping the Node suite runnable.
The mirror covers package-local `#imports` with no supported imports map and
unsupported condition-profile maps, preserving unsupported diagnostics, absence
of concrete graph edges, candidate preservation, and explicit blind-zone scope.

## [2026-05-14] tracking | Vitest pilot status consolidation

Updated the Node `#imports` pilot review page and wiki status pages after the
mirror landed. The board now records that no reviewed unimplemented suite is
open, so the next test-runner step should be a review-only page rather than a
direct migration into parked analyzer-sensitive suites.

## [2026-05-14] review | import.meta.glob diagnostics Vitest candidate

Added a dynamic-module Vitest pilot review page for
`tests/test-import-meta-glob-diagnostics.mjs`. The review names the contracts a
future mirror must preserve: `import-meta-glob-unsupported` diagnostics,
absence of concrete graph edges, unsupported diagnostic lanes, affected route
surface scope, and candidate-relevant blind-zone behavior.

## [2026-05-14] implementation | import.meta.glob diagnostics Vitest mirror

Added the focused Vitest mirror for
`tests/test-import-meta-glob-diagnostics.mjs`. The mirror preserves the Node
entrypoint, keeps literal `import.meta.glob` diagnostic-only, asserts that no
concrete graph edge is created even when a matching route file exists, and
records the suite as a completed pilot in the candidate board.

## [2026-05-14] review | output-to-source layout diagnostics Vitest candidate

Added an output-to-source mapping Vitest pilot review page for
`tests/test-output-source-layout-diagnostics.mjs`. The review names the
contracts a future mirror must preserve: unsupported output-layout diagnostics,
no fake graph edge to build output or source files, candidate-scoped blind-zone
behavior, and blocked candidate hints for the affected package surface.

## [2026-05-14] implementation | output-to-source layout diagnostics Vitest mirror

Added the focused Vitest mirror for
`tests/test-output-source-layout-diagnostics.mjs`. The mirror preserves the
Node entrypoint, keeps unsupported build-output layouts diagnostic-only,
asserts that no fake graph edge is created, and records the suite as a
completed pilot in the candidate board.

## [2026-05-14] review | generated artifact evidence Vitest candidate

Added a generated-artifact evidence Vitest pilot review page for
`tests/test-generated-artifact-evidence.mjs`. The review names the contracts a
future mirror must preserve: strong generated evidence requires package/script
proof, files-only and path-segment hints stay weak/supporting, workspace subpath
evidence stays normalized, and generated identity constants remain centralized.

## [2026-05-14] implementation | generated artifact evidence Vitest mirror

Added the focused Vitest mirror for
`tests/test-generated-artifact-evidence.mjs`. The mirror preserves the Node
entrypoint, keeps build/static/local generated evidence quorum checks local to
the generated artifact policy module, keeps path-segment evidence
supporting-only, and pins generated artifact identity constants against
downstream hardcoding. The full Vitest pilot lane now disables file-level
parallelism so producer-backed pilot suites do not contend and trip hook
timeouts during `npm run test:vitest`.

## [2026-05-14] review | generated blind-zone relevance Vitest candidate

Added a generated blind-zone relevance Vitest pilot review page for
`tests/test-generated-blind-zone-relevance.mjs`. The review names the contracts
a future mirror must preserve: generated misses stay candidate-scoped,
consumer-file overlap alone is not provider proof, present-but-excluded
generated files remain blind zones with unknown stale provenance, and
structured generated taint remains review evidence rather than `SAFE_FIX` or
deadness proof.

## [2026-05-14] implementation | generated blind-zone relevance Vitest mirror

Added the focused Vitest mirror for
`tests/test-generated-blind-zone-relevance.mjs`. The mirror preserves the Node
entrypoint, keeps provider-surface and generated-consumer relevance assertions
local to `generated-blind-zone-relevance.mjs`, keeps present-but-excluded
generated files as blind zones with unknown stale provenance, and keeps
structured generated taint as review evidence rather than `SAFE_FIX` or
deadness proof.

## [2026-05-14] review | generated consumer blind-zones Vitest candidate

Added a generated consumer blind-zones Vitest pilot review page for
`tests/test-generated-consumer-blind-zones.mjs`. The review names the producer
contracts a future mirror must preserve: `build-symbol-graph.mjs` emits
`symbols.json.generatedConsumerBlindZones[]`, generated consumer misses stay
blind-zone inventory rather than observed source consumers, prepared mode keeps
unknown stale provenance, and virtual surfaces remain a separate suite.

## [2026-05-14] implementation | generated consumer blind-zones Vitest mirror

Added the focused Vitest mirror for
`tests/test-generated-consumer-blind-zones.mjs`. The mirror preserves the Node
entrypoint, keeps the `build-symbol-graph.mjs` producer fixture local, verifies
`symbols.json.generatedConsumerBlindZones[]` support and inventory shape, and
keeps prepared generated artifact mode paired with unknown stale provenance.

## [2026-05-14] review | generated virtual surface Vitest candidate

Added a generated virtual surface Vitest pilot review page for
`tests/test-generated-virtual-surface.mjs`. The review names the contracts a
future mirror must preserve: Prisma enum virtual surfaces require explicit
schema generator evidence, remain partial, keep `runtimeEquivalence: false`,
and do not resolve missing enum exports or replace generated consumer
blind-zone inventory.

## [2026-05-14] implementation | generated virtual surface Vitest mirror

Added the focused Vitest mirror for
`tests/test-generated-virtual-surface.mjs`. The mirror preserves the Node
entrypoint, keeps Prisma enum parser and schema-provider evidence assertions
local, verifies supported virtual surfaces remain partial with
`runtimeEquivalence: false`, and keeps missing provider or missing enum export
cases unresolved rather than virtual import consumers.

## [2026-05-14] review | hash imports Vitest candidate

Added a hash-imports Vitest pilot review page for
`tests/test-hash-imports.mjs`. The review names the contracts a future mirror
must preserve: supported Node `#imports` exact and wildcard resolution, suffix
wildcard graph/deadness protection, output-pattern mapping helper behavior,
authored JS and directory-index targets, and malformed workspace package
resilience.

## [2026-05-14] implementation | hash imports Vitest mirror

Added the focused Vitest mirror for `tests/test-hash-imports.mjs`. The mirror
preserves the Node entrypoint, keeps exact and wildcard `#imports` resolver
fixtures local, verifies suffix wildcard type/value deadness protection,
pins output-pattern helper behavior, and keeps authored JS, directory-index,
and malformed workspace package resilience cases visible.

## [2026-05-14] review | resolver diagnostics artifacts Vitest candidate

Added a resolver diagnostics artifacts Vitest pilot review page for
`tests/test-resolver-diagnostics-artifacts.mjs`. The review names the contracts
a future mirror must preserve: resolver capability and per-run diagnostics
artifacts stay separate, candidate targets remain diagnostic-only, blind zones
stay candidate-relevant, blocked hints do not become action proof, and summary
pivots remain machine-readable.

## [2026-05-14] implementation | resolver diagnostics artifacts Vitest mirror

Added the focused Vitest mirror for
`tests/test-resolver-diagnostics-artifacts.mjs`. The mirror preserves the Node
entrypoint, keeps the synthetic `symbols.json` artifact-shape fixture local,
verifies capability and diagnostics artifacts remain separate, keeps candidate
targets diagnostic-only, preserves candidate-relevant blind-zone policies, and
keeps blocked hints as absence-claim limitations rather than action proof.

## [2026-05-14] review | resolver blind-zone relevance Vitest candidate

Added a resolver blind-zone relevance Vitest pilot review page for
`tests/test-resolver-blind-zone-relevance.mjs`. The review names the contracts
a future mirror must preserve: candidate-relevant resolver taint is scoped by
target candidate package, affected package scope, or exact target file;
unrelated unresolved imports do not become repo-global blockers; generated
artifact relevance stays owned by generated helpers; and resolver taint demotes
`SAFE_FIX` to review rather than becoming action proof.

## [2026-05-14] implementation | resolver blind-zone relevance Vitest mirror

Added the focused Vitest mirror for
`tests/test-resolver-blind-zone-relevance.mjs`. The mirror preserves the Node
entrypoint, keeps relevance fixtures local, verifies candidate-relevant
resolver taint remains scoped, keeps generated artifact records delegated to
generated relevance helpers, and preserves the `SAFE_FIX` to `REVIEW_FIX`
demotion path for relevant resolver blind zones.

## [2026-05-14] review | resolved edges Vitest candidate

Added a resolved edges Vitest pilot review page for
`tests/test-resolved-edges.mjs`. The review names the contracts a future mirror
must preserve: `symbols.json.resolvedInternalEdges[]` is file-level
reachability evidence, symbol fan-in stays separate, side-effect and broad CJS
edges do not keep named exports alive, type-only edges keep their lens, and
non-source asset imports do not become unresolved internal JavaScript modules.

## [2026-05-14] implementation | resolved edges Vitest mirror

Added the focused Vitest mirror for `tests/test-resolved-edges.mjs`. The mirror
preserves the Node entrypoint, keeps the symbol-graph fixture local, verifies
ESM, type-only, dynamic literal, and CommonJS edge kinds, and keeps side-effect
reachability separate from named export fan-in.

## [2026-05-14] review | JSONC edge cases Vitest candidate

Added a JSONC edge-cases Vitest pilot review page for
`tests/test-jsonc-edge-cases.mjs`. The review names the contracts a future
mirror must preserve: `$schema` URLs and comment-looking string literals must
not be stripped as comments, JSONC comments/trailing commas must parse, missing
`extends` targets must not erase local path aliases, and duyet-shaped multi-app
fixtures must discover every scoped path entry.

## [2026-05-14] planning | Vitest mirror goal

Added `docs/lumin-wiki/vitest-mirror-goal.md` to stop treating the remaining
Vitest migration as one PR per suite. The goal records the current inventory
count, groups the 143 unmigrated Node suites into risk lanes, defines batch
rules, and keeps Node entrypoints, runner discovery, and analyzer-sensitive
semantics protected while review pages and mirrors are generated or batched.

## [2026-05-14] implementation | JSONC edge cases Vitest mirror

Added the focused Vitest mirror for `tests/test-jsonc-edge-cases.mjs`. The
mirror preserves the Node entrypoint, keeps JSONC parser edge cases local,
covers `$schema` URLs, real comments, trailing commas, comment-looking string
literals, BOM-prefixed tsconfigs, missing `extends` targets, and duyet-shaped
multi-app scoped path discovery, and records the suite as complete in the
goal lane.

## [2026-05-14] review | CLI helper Vitest candidate

Added a Lane A CLI helper Vitest pilot review page for `tests/test-cli.mjs`.
The review names the contracts a future mirror must preserve: boolean
`includeTests` parsing, negation aliases, production precedence, default output
location, test-path naming conventions, substring false-positive protection,
and Windows-safe dynamic import URLs.

## [2026-05-15] implementation | CLI helper Vitest mirror

Added the focused Vitest mirror for `tests/test-cli.mjs`. The mirror preserves
the Node entrypoint, keeps `parseCliArgs(...)` and `isTestLikePath(...)`
contracts local, covers boolean include-tests parsing, negation aliases,
production precedence, default output, test-path naming conventions,
substring false-positive protection, and Windows-safe cache-busted `file://`
dynamic imports.

## [2026-05-15] review | Vocab Vitest mirror candidate

Added `pilot-reviews/vitest-vocab.md` and marked `tests/test-vocab.mjs` as the
next reviewed Lane A candidate. The review keeps the future mirror focused on
`_lib/vocab.mjs` literal evidence labels, taint labels, severity sets,
provenance forwarding, delta-label enumeration, and the
`requiredAcknowledgements(...)` silent-new contract while keeping classifier,
deadness, ranking, resolver, post-write matching, renderer, and producer suites
out of scope.

## [2026-05-15] implementation | Vocab Vitest mirror

Added the focused Vitest mirror for `tests/test-vocab.mjs`. The mirror preserves
the Node entrypoint, keeps `_lib/vocab.mjs` vocabulary contracts local, and
covers evidence labels, taint labels, severity groups, frozen export
containers, provenance forwarding, fresh field-name copies, delta-label
enumeration, and the `requiredAcknowledgements(...)` silent-new filter.

## [2026-05-15] review | Collect files Vitest mirror candidate

Added `pilot-reviews/vitest-collect.md` and marked `tests/test-collect.mjs` as
the next reviewed Lane A candidate. The review keeps the future mirror focused
on `_lib/collect-files.mjs` language filtering, root-level Python/Go discovery,
`includeTests` filtering, JS/TS root-entry preservation, cross-language leakage
guards, user excludes, and repo-relative path matching while keeping
orchestrator, resolver, deadness, ranking, performance, and renderer suites out
of scope.

## [2026-05-15] implementation | Collect files Vitest mirror

Added the focused Vitest mirror for `tests/test-collect.mjs`. The mirror
preserves the Node entrypoint, keeps `_lib/collect-files.mjs` contracts local,
and covers Python/Go/JS/TS language filtering, root-level file discovery,
`includeTests` filtering, JS/TS root entry preservation, cross-language leakage
guards, user exclude behavior, and repo-relative vendor exclude matching.

## [2026-05-15] review | Shape hash Vitest mirror candidate

Added `pilot-reviews/vitest-shape-hash.md` and marked
`tests/test-shape-hash.mjs` as the next reviewed Lane A candidate. The review
keeps the future mirror focused on `_lib/shape-hash.mjs` pure normalization and
diagnostic contracts: field-order-insensitive object shapes, hash-bearing
optional/readonly modifiers, literal-safe type-text normalization, unsupported
shape diagnostics, canonical fact metadata, alias identity handling,
deterministic grouping, declaration-merge rejection, generated-file evidence,
and literal union hashing while keeping shape-index producers, pre-write lookup,
deadness, ranking, resolver, performance, and renderer suites out of scope.

## [2026-05-15] implementation | Shape hash Vitest mirror

Added the focused Vitest mirror for `tests/test-shape-hash.mjs`. The mirror
preserves the Node entrypoint, keeps `_lib/shape-hash.mjs` contracts local, and
covers field-order-insensitive object/interface hashes, hash-bearing field
changes and modifiers, literal-safe type normalization, unsupported-shape
diagnostics, canonical fact metadata, parse-error suppression, exported alias
identity, deterministic grouping, declaration-merge rejection, generated-file
evidence, and literal union hashing.

## [2026-05-15] review | Export surface guards Vitest mirror batch

Added `pilot-reviews/vitest-export-surface-guards.md` and marked four tiny
export-surface guard suites as reviewed Lane A candidates:
`tests/test-definition-id-export.mjs`, `tests/test-file-delta-export.mjs`,
`tests/test-function-clone-export-surface.mjs`, and
`tests/test-classify-policies-export-surface.mjs`. The review keeps the future
mirror batch focused on direct module export contracts while preserving every
Node entrypoint and keeping algorithm behavior, resolver, ranking,
pre/post-write workflow, function-clone grouping, and classification behavior
suites out of scope.

## [2026-05-15] implementation | Export surface guards Vitest mirror batch

Added focused Vitest mirrors for the four reviewed export-surface guard suites:
`tests/definition-id-export.test.mjs`, `tests/file-delta-export.test.mjs`,
`tests/function-clone-export-surface.test.mjs`, and
`tests/classify-policies-export-surface.test.mjs`. The mirrors preserve the
Node entrypoints and keep direct module export contracts local: public helpers
remain exported, raw builders/path normalizers/version constants/legacy
sentinels/non-public policy actions stay unexported, and no algorithm behavior
or analyzer evidence suite is absorbed into this batch.

## [2026-05-15] review | Parser and AST guards Vitest mirror batch

Added `pilot-reviews/vitest-parser-ast-guards.md` and marked
`tests/test-classify-facts-ast.mjs` and `tests/test-lang-matrix.mjs` as the
next reviewed Lane A candidates. The review keeps the future mirror batch
focused on AST identifier reference counting, exported declaration-surface
references, scope shadowing, JSX handling, language dispatch, and
mixed-extension symbol ingest while keeping deadness/ranking, resolver,
topology, performance, and public-package suites out of scope.

## [2026-05-15] implementation | Parser and AST guards Vitest mirror batch

Added focused Vitest mirrors for `tests/test-classify-facts-ast.mjs` and
`tests/test-lang-matrix.mjs`. The mirrors preserve the Node entrypoints and
keep parser/AST contracts local: raw-text false references stay rejected,
real value/type/JSX references stay counted, scope shadowing remains visible,
exported declaration-surface evidence stays distinct from implementation
bodies, batch reference counting matches single-symbol semantics, and
mixed-extension language dispatch continues to walk and classify supported
JS/TS families.

## [2026-05-15] review | Hardcoding guards Vitest mirror candidate

Added `pilot-reviews/vitest-hardcoding.md` and marked
`tests/test-hardcoding.mjs` as the next reviewed Lane A candidate. The review
keeps the future mirror focused on workspace-derived dead-export labels and
`resolve-method-calls.mjs --focus-class` output gating while keeping broader
deadness/ranking, method-call precision, call-graph, artifact schema, and audit
orchestration behavior out of scope.

## [2026-05-15] implementation | Hardcoding guards Vitest mirror

Added the focused Vitest mirror for `tests/test-hardcoding.mjs`. The mirror
preserves the Node entrypoint, uses the setup-only temp repo fixture helper,
keeps workspace label expectations local to the synthetic `packages/alpha` and
`apps/beta` monorepo, blocks legacy repo-specific labels, and preserves
`resolve-method-calls.mjs --focus-class` console and `level2-methods.json`
behavior.

## [2026-05-15] review | Audit manifest export-surface Vitest mirror candidate

Added `pilot-reviews/vitest-audit-manifest-export-surface.md` and marked
`tests/test-audit-manifest-export-surface.mjs` as the next reviewed Lane A
candidate. The review keeps the future mirror focused on `_lib/audit-manifest`
public exports and manifest evidence summary shapes while keeping full audit
orchestration, resolver correctness, generated/framework producer behavior,
deadness/ranking, performance, public-package install, and Markdown rendering
out of scope.

## [2026-05-15] implementation | Audit manifest export-surface Vitest mirror

Added the focused Vitest mirror for
`tests/test-audit-manifest-export-surface.mjs`. The mirror preserves the Node
entrypoint, keeps temporary JSON artifacts as setup-only fixtures, and pins
`_lib/audit-manifest.mjs` public exports plus generated, framework/resource,
generated-consumer, present/prepared generated-file, and resolver diagnostics
summary shapes without running the full audit pipeline.

## [2026-05-15] review | Definition ID canonical Vitest mirror candidate

Added `pilot-reviews/vitest-definition-id-canonical.md` and marked
`tests/test-definition-id-canonical.mjs` as the next reviewed Lane A candidate.
The review keeps the future mirror focused on canonical `definitionId`
continuity across symbol graph, call graph, and action-safety while keeping
general call-graph precision, deadness/ranking, resolver behavior, public API
policy, performance, and full audit orchestration out of scope.

## [2026-05-15] implementation | Definition ID canonical Vitest mirror

Added the focused Vitest mirror for
`tests/test-definition-id-canonical.mjs`. The mirror preserves the Node
entrypoint and keeps the temporary two-file alias fixture local while pinning
canonical `definitionId` continuity across `symbols.json`, `call-graph.json`,
and `export-action-safety.json` without broadening into call-graph precision,
deadness/ranking, resolver, or full audit orchestration behavior.

## [2026-05-15] review | Shell safety Vitest mirror candidate

Added `pilot-reviews/vitest-shell-safety.md` and marked
`tests/test-shell-safety.mjs` as the next reviewed Lane A candidate. The review
keeps the future mirror focused on shell-metacharacter path safety, triage
language counts, single-pass file collection telemetry, root-only Python
detection, Go counting, and staleness records while keeping broad git history,
deadness/ranking, resolver behavior, performance benchmarking, public package
behavior, and full audit orchestration out of scope.

## [2026-05-15] implementation | Shell safety Vitest mirror

Added the focused Vitest mirror for `tests/test-shell-safety.mjs`. The mirror
preserves the Node entrypoint and keeps shell-metacharacter fixtures local while
pinning triage language counts, single-pass file collection telemetry,
root-only Python discovery, Go counting, top-dir summaries, and staleness
records for `$`-named files without broadening into deadness/ranking, resolver,
performance, public-package, or full audit orchestration behavior.

## [2026-05-15] review | Evidence honesty Vitest mirror candidate

Added `pilot-reviews/vitest-evidence-honesty.md` and marked
`tests/test-evidence-honesty.mjs` as the next reviewed Lane A candidate. The
review keeps the future mirror focused on `compare-repos.mjs` artifact deltas,
missing-artifact null semantics, `scripts/check-doc-script-refs.mjs` missing
`.mjs` references, remediation wording, and `_lib/` helper resolution while
keeping resolver behavior, deadness/ranking, performance timing, public package
install, generated/framework surfaces, and full audit orchestration out of
scope.

## [2026-05-15] implementation | Evidence honesty Vitest mirror

Added the focused Vitest mirror for `tests/test-evidence-honesty.mjs`. The
mirror preserves the Node entrypoint and keeps synthetic artifact directories
and doc-ref fixtures local while pinning `compare-repos.mjs` numeric deltas,
missing-artifact null semantics, `scripts/check-doc-script-refs.mjs` failure
wording, and `_lib/` helper resolution without broadening into resolver,
deadness/ranking, performance, public-package, generated/framework, or full
audit orchestration behavior.

## [2026-05-15] review | Canon helper-registry Vitest mirror batch

Added `pilot-reviews/vitest-canon-helper-registry.md` and marked four Lane B
helper-registry canon suites as reviewed:
`tests/test-canon-draft-helpers.mjs`,
`tests/test-canon-draft-helper-registry.mjs`,
`tests/test-generate-canon-draft-cli-helpers.mjs`, and
`tests/test-check-canon-helpers.mjs`. The review keeps the future batch focused
on helper classifier precedence, helper fan-in and identity aggregation,
helper-registry CLI draft behavior, and helper drift evidence gates while
keeping topology, naming, type-ownership, resolver, full audit, performance,
and analyzer-sensitive ranking behavior out of scope.

## [2026-05-15] implementation | Canon helper-registry Vitest mirror batch

Added the focused Vitest mirrors for the four reviewed Lane B helper-registry
canon suites. The mirrors preserve the Node entrypoints and keep helper
classifier precedence, helper fan-in aggregation, helper-registry CLI draft
behavior, and helper drift evidence gates local while leaving topology, naming,
type-ownership, resolver, full audit, performance, and analyzer-sensitive
ranking behavior out of scope.

## [2026-05-15] review | Canon naming Vitest mirror batch

Reviewed the next Lane B canon source family for a batched Vitest mirror. The
naming batch covers pure convention detection and basename normalization,
cohort aggregation and rendering, the `generate-canon-draft.mjs --source
naming` CLI path, and naming drift evidence gates. The review keeps topology,
type-ownership, helper-registry, integration, resolver, full audit,
performance, and analyzer-sensitive ranking behavior out of scope.

## [2026-05-15] implementation | Canon naming Vitest mirror batch

Added the focused Vitest mirrors for the four reviewed Lane B naming canon
suites. The mirrors preserve the Node entrypoints and keep naming convention
detection, basename normalization, cohort aggregation and rendering, naming CLI
draft behavior, and naming drift evidence gates local while leaving topology,
type-ownership, helper-registry, integration, resolver, full audit,
performance, and analyzer-sensitive ranking behavior out of scope.

## [2026-05-15] review | Canon type-ownership Vitest mirror batch

Reviewed the next Lane B canon source family for a batched Vitest mirror. The
type-ownership batch covers pure type classifier rules and Markdown cell
helpers, type identity aggregation and rendering, the
`generate-canon-draft.mjs --source type-ownership` CLI path, and type drift
evidence gates. The review keeps helper-registry, naming, topology,
integration, resolver, generated/framework, full audit, performance, and
deadness/ranking behavior out of scope.

## [2026-05-15] implementation | Canon type-ownership Vitest mirror batch

Added the focused Vitest mirrors for the four reviewed Lane B type-ownership
canon suites. The mirrors preserve the Node entrypoints and keep pure type
classifier rules, type identity aggregation and rendering, type-ownership CLI
draft behavior, and type drift evidence gates local while leaving
helper-registry, naming, topology, integration, resolver, generated/framework,
full audit, performance, and deadness/ranking behavior out of scope.

## [2026-05-15] review | Canon topology Vitest mirror batch

Reviewed the next Lane B canon source family for a batched Vitest mirror. The
topology batch covers pure topology classifier rules, topology structure
aggregation and rendering, the `generate-canon-draft.mjs --source topology` CLI
path, and topology drift evidence gates. The review keeps helper-registry,
naming, type-ownership, integration, resolver, generated/framework, full audit,
performance, incremental cache, deadness, and ranking behavior out of scope.

## [2026-05-15] implementation | Canon topology Vitest mirror batch

Added the focused Vitest mirrors for the four reviewed Lane B topology canon
suites. The mirrors preserve the Node entrypoints and keep topology classifier
rules, structure aggregation and rendering, topology CLI draft behavior, and
topology drift evidence gates local while leaving helper-registry, naming,
type-ownership, integration, resolver, generated/framework, full audit,
performance, incremental cache, deadness, and ranking behavior out of scope.

## [2026-05-15] review | Canon draft integration Vitest mirror batch

Reviewed the next Lane B canon integration batch for a future Vitest mirror.
The batch covers type-ownership, helper-registry, and topology end-to-end
fixture-to-Markdown integration through the real canon draft CLIs. The review
keeps check-canon integration, audit-repo orchestration, resolver expansion,
generated/framework, deadness/ranking, performance, and incremental cache
behavior out of scope.

## [2026-05-15] implementation | Canon draft integration Vitest mirror batch

Added focused Vitest mirrors for the three reviewed Lane B canon draft
integration suites. The mirrors preserve type-ownership, helper-registry, and
topology fixture-to-Markdown integration through the real canon draft CLIs
while keeping check-canon integration, audit-repo orchestration, resolver
expansion, generated/framework, deadness/ranking, performance, and incremental
cache behavior out of scope.

## [2026-05-15] review | Canon drift contract Vitest mirror batch

Reviewed the next Lane B canon drift contract batch for a future Vitest mirror.
The batch covers canon renderer table header contracts against
`canonical/canon-drift.md` §5 and the `canonical/fact-model.md` §3.9
type-escape schema drift guard. The review keeps producer integration,
resolver behavior, generated/framework surfaces, deadness/ranking,
performance, incremental cache, and full audit orchestration out of scope.

## [2026-05-15] implementation | Canon drift contract Vitest mirror batch

Added focused Vitest mirrors for the two reviewed Lane B canon drift contract
suites. The mirrors preserve renderer table header drift checks and fact-model
type-escape schema drift checks while keeping producer integration, resolver
behavior, generated/framework surfaces, deadness/ranking, performance,
incremental cache, and full audit orchestration out of scope.

## [2026-05-15] review | Check-canon core Vitest mirror batch

Reviewed the next Lane B check-canon core batch for a future Vitest mirror. The
batch covers parser strictness, canon loader and drift artifact writer I/O, the
`generate-check-canon.mjs` CLI exit/output matrix, and end-to-end drift fixture
outputs. The review keeps audit-repo check-canon orchestration, resolver
expansion, generated/framework surfaces, deadness/ranking, performance, and
incremental cache behavior out of scope.

## [2026-05-15] implementation | Check-canon core Vitest mirror batch

Added focused Vitest mirrors for the four reviewed Lane B check-canon core
suites. The mirrors preserve parser strictness, canon loader/writer I/O,
check-canon CLI exit/output policy, and end-to-end drift fixture outputs while
keeping audit-repo orchestration, resolver expansion, generated/framework,
deadness/ranking, performance, and incremental cache behavior out of scope.

## [2026-05-15] review | Public package publish Vitest mirror batch

Reviewed the next Lane G public package batch for a future Vitest mirror. The
batch covers plugin package build output, the local public publish workflow,
and GitHub Actions CI policy. The review keeps public skill-surface text
suites, hook runtime suites, analyzer behavior, resolver behavior,
generated/framework surfaces, deadness/ranking, performance, and incremental
cache behavior out of scope.

## [2026-05-15] implementation | Public package publish Vitest mirror batch

Added focused Vitest mirrors for the three reviewed Lane G public package
publish suites. The mirrors preserve plugin package build output, local public
publish git fixtures, and GitHub Actions CI routing policy while keeping public
skill-surface text suites, hook runtime suites, analyzer behavior, resolver
behavior, generated/framework surfaces, deadness/ranking, performance, and
incremental cache behavior out of scope.

## [2026-05-15] review | Public skill-surface Vitest mirror batch

Reviewed `tests/test-skill-surface.mjs` for a future Lane G Vitest mirror. The
review covers root package metadata, README install and evidence wording, split
SKILL surfaces, command-routing docs, template docs, and public/private doc
staging while keeping `test-skill-package.mjs`, package publishing, hook
runtime suites, analyzer behavior, resolver behavior, deadness/ranking, and
performance/incremental cache behavior out of scope.

## [2026-05-15] implementation | Public skill-surface Vitest mirror batch

Added the focused Vitest mirror for the reviewed Lane G public skill-surface
suite. The mirror preserves package metadata, README, split SKILL, command
routing, template, and doc-staging text contracts while keeping
`test-skill-package.mjs`, package publishing, hook runtime suites, analyzer
behavior, resolver behavior, deadness/ranking, and performance/incremental
cache behavior out of scope.

## [2026-05-15] review | Generated skill-package Vitest mirror batch

Reviewed `tests/test-skill-package.mjs` for a future Lane G Vitest mirror. The
review covers `scripts/build-skill.mjs` generated wrapper scripts, shared
engine relocation, packaged skill surfaces, packaged references/templates and
canonical spine, generated package metadata, smoke test, Codex wrapper
metadata, and dependency setup behavior while keeping `test-skill-surface.mjs`,
plugin packaging, package publishing, hook runtime suites, analyzer behavior,
resolver behavior, deadness/ranking, and performance/incremental cache behavior
out of scope.

## [2026-05-15] implementation | Generated skill-package Vitest mirror batch

Added the focused Vitest mirror for the reviewed Lane G generated
skill-package suite. The mirror preserves generated wrapper scripts, shared
engine relocation, packaged skill surfaces, references/templates/canonical
spine, package metadata, smoke test, Codex wrapper metadata, and dependency
setup behavior while keeping `test-skill-surface.mjs`, plugin packaging,
package publishing, hook runtime suites, analyzer behavior, resolver behavior,
deadness/ranking, and performance/incremental cache behavior out of scope.

## [2026-05-15] review | Host hook runtime Vitest mirror batch

Reviewed the nine `tests/test-hook-*.mjs` runtime suites for a future Lane G
Vitest mirror batch. The review covers hook doctor/manifest evidence, host
runner stdin/output behavior, path and id safety, event-store delivery and
lock recovery, preimage privacy, ACK observation, reminder drain/rendering, and
post-write-lite silent-new reminders while keeping
`test-pre-write-inventory-hook.mjs`, pre/post-write advisory tests, package
publishing, skill package/surface tests, analyzer behavior, resolver behavior,
deadness/ranking, and performance/incremental cache behavior out of scope.

## [2026-05-15] implementation | Host hook runtime Vitest mirror batch

Added focused Vitest mirrors for the nine reviewed host hook runtime suites.
The mirrors preserve hook doctor/manifest evidence, host runner stdin/output
behavior, path and id safety, event-store delivery and lock recovery, preimage
privacy, ACK observation, reminder drain/rendering, and post-write-lite
silent-new reminders while keeping `test-pre-write-inventory-hook.mjs`,
pre/post-write advisory tests, package publishing, skill package/surface tests,
analyzer behavior, resolver behavior, deadness/ranking, and
performance/incremental cache behavior out of scope.

## [2026-05-15] review | Post-write lifecycle Vitest mirror batch

Reviewed the six `tests/test-post-write-*.mjs` lifecycle suites for a future
Lane C Vitest mirror batch. The review covers post-write delta artifact
identity, direct post-write CLI behavior, pure delta classification,
after-snapshot incremental routing, end-to-end pre-write/post-write lifecycle
behavior, and Markdown/JSON delta rendering while keeping pre-write advisory
shape tests, `test-pre-write-inventory-hook.mjs`, cue-tier policy tests,
broader audit-repo lifecycle tests, analyzer behavior, resolver behavior,
deadness/ranking, and performance/incremental cache behavior out of scope.

## [2026-05-15] implementation | Post-write lifecycle Vitest mirror batch

Added focused Vitest mirrors for the six reviewed post-write lifecycle suites.
The mirrors preserve post-write delta artifact identity, direct post-write CLI
behavior, pure delta classification, after-snapshot incremental routing,
end-to-end pre-write/post-write lifecycle behavior, and Markdown/JSON delta
rendering while keeping pre-write advisory shape tests,
`test-pre-write-inventory-hook.mjs`, cue-tier policy tests, broader audit-repo
lifecycle tests, analyzer behavior, resolver behavior, deadness/ranking, and
performance/incremental cache behavior out of scope.

## [2026-05-15] review | Class method pre-write Vitest mirror batch

Reviewed `tests/test-class-method-index-prototype-names.mjs` and
`tests/test-class-method-prewrite-surface.mjs` for a future Lane C Vitest
mirror batch. The review covers prototype-named method dictionary safety,
class method index metadata, `defIndex` non-promotion, and pre-write near-name
review cue visibility while keeping cue-tier routing, Markdown rendering,
service-operation policy tests, pre-write advisory artifacts, resolver
behavior, deadness/ranking, and performance/incremental cache behavior out of
scope.

## [2026-05-15] implementation | Class method pre-write Vitest mirror batch

Added focused Vitest mirrors for the two reviewed class-method pre-write
suites. The mirrors preserve prototype-named method dictionary safety, class
method index metadata, `defIndex` non-promotion, and pre-write near-name review
cue visibility while keeping cue-tier routing, Markdown rendering,
service-operation policy tests, pre-write advisory artifacts, resolver
behavior, deadness/ranking, and performance/incremental cache behavior out of
scope.

## [2026-05-15] review | CJS surface Vitest mirror batch

Reviewed the five CJS surface suites for a future Lane D Vitest mirror batch:
`tests/test-extract-cjs-consumer.mjs`,
`tests/test-extract-cjs-export-surface.mjs`,
`tests/test-cjs-export-surface-artifact.mjs`,
`tests/test-cjs-classification.mjs`, and `tests/test-cjs-integration.mjs`.
The review keeps the batch focused on exact CJS consumer evidence, CJS export
surface facts, broad/opaque opacity, artifact metadata, and integrated fan-in
classification while keeping broader resolver expansion, generated/framework
surfaces, action-safety promotion, performance, and pre-write cue policy out of
scope.

## [2026-05-15] implementation | CJS surface Vitest mirror batch

Added focused Vitest mirrors for the five reviewed CJS surface suites. The
mirrors preserve exact CJS consumer extraction, CJS export surface extraction,
`symbols.json` CJS export-surface metadata, exact-vs-broad fan-in
classification, and integrated dynamic require opacity while keeping broader
resolver expansion, generated/framework surfaces, action-safety promotion,
performance, and pre-write cue policy out of scope.

## [2026-05-15] review | Framework/resource surfaces Vitest mirror batch

Reviewed four Lane D framework/resource suites for a future Vitest mirror
batch: `tests/test-framework-resource-surfaces.mjs`,
`tests/test-build-framework-resource-surfaces.mjs`,
`tests/test-framework-policy-facts.mjs`, and
`tests/test-framework-policy-matrix.mjs`. The review keeps the batch focused on
framework/resource surface artifacts, capability-pack summaries, Hono route
registration facts, package-scoped framework policy, workspace pattern merging,
and framework sentinel/review-hint counters while keeping broader resolver
expansion, deadness/ranking, action-safety promotion, performance, pre-write
cue policy, and full audit orchestration out of scope.

## [2026-05-15] implementation | Framework/resource surfaces Vitest mirror batch

Added focused Vitest mirrors for the four reviewed framework/resource suites.
The mirrors preserve framework/resource surface classification, producer
artifact and manifest routing, Hono route registration facts, package-scoped
framework policy, workspace pattern merging, and framework sentinel/review-hint
counters while keeping broader resolver expansion, deadness/ranking,
action-safety promotion, performance, pre-write cue policy, and full audit
orchestration out of scope.

## [2026-05-15] review | Public/workspace surfaces Vitest mirror batch

Reviewed four Lane D public/workspace surface suites for a future Vitest mirror
batch: `tests/test-public-surface.mjs`,
`tests/test-public-deep-import-risk.mjs`,
`tests/test-workspace-no-exports.mjs`, and
`tests/test-mdx-consumers.mjs`. The review keeps the batch focused on package
public surfaces, package `files` and npm always-included deep-import risk,
legacy workspace subpath fallback, output-to-source aliasing, and MDX consumer
fan-in evidence while keeping unsupported resolver families, generated
blind-zones, deadness/ranking, action-safety promotion, performance,
pre-write cue policy, and full audit orchestration out of scope.

## [2026-05-15] implementation | Public/workspace surfaces Vitest mirror batch

Added focused Vitest mirrors for the four reviewed public/workspace surface
suites. The mirrors preserve package public surface collection, public
deep-import risk details, legacy workspace subpath fallback, output-to-source
aliasing, and MDX consumer fan-in evidence while keeping unsupported resolver
families, generated blind-zones, deadness/ranking, action-safety promotion,
performance, pre-write cue policy, and full audit orchestration out of scope.

## [2026-05-15] review | Artifact-output presentation Vitest mirror batch

Reviewed two Lane H artifact-output presentation suites for a future Vitest
mirror batch: `tests/test-topology-mermaid.mjs` and
`tests/test-sarif-fix-plan.mjs`. The review keeps the batch focused on
topology Markdown/Mermaid companion rendering and SARIF fix-plan tier output
while keeping SCC computation, resolver behavior, dead-export classification,
ranking policy selection, full audit orchestration, public package install
behavior, and performance measurement out of scope.

## [2026-05-15] implementation | Artifact-output presentation Vitest mirror batch

Added focused Vitest mirrors for the two reviewed artifact-output presentation
suites. The mirrors preserve topology Markdown/Mermaid companion rendering and
SARIF fix-plan tier output while keeping SCC computation, resolver behavior,
dead-export classification, ranking policy selection, full audit orchestration,
public package install behavior, and performance measurement out of scope.

## [2026-05-15] review | Call-graph evidence Vitest mirror batch

Reviewed three Lane H call-graph evidence suites for a future Vitest mirror
batch: `tests/test-call-graph-bounded.mjs`,
`tests/test-call-graph-parse-errors.mjs`, and
`tests/test-call-graph-truncation-defense.mjs`. The review keeps the batch
focused on bounded imported member-call fan-in evidence, parse-error
completeness warnings, and full fan-in preservation outside the `topCallees`
display slice while keeping ranking, deadness, action-safety, resolver
expansion, full audit orchestration, performance, and incremental cache
behavior out of scope.

## [2026-05-15] implementation | Call-graph evidence Vitest mirror batch

Added focused Vitest mirrors for the three reviewed call-graph evidence suites.
The mirrors preserve bounded imported member-call fan-in evidence, parse-error
completeness warnings, and full fan-in preservation outside the `topCallees`
display slice while keeping ranking, deadness, action-safety, resolver
expansion, full audit orchestration, performance, and incremental cache
behavior out of scope.

## [2026-05-15] review | Threshold metadata Vitest mirror batch

Reviewed two metadata-only threshold suites for a future Vitest mirror batch:
`tests/test-threshold-policies.mjs` and
`tests/test-calibration-corpora.mjs`. The review keeps the batch focused on
threshold policy ids, versions, classes, numeric values, hashes, calibration
corpus references, corpus ids, metrics, compact summaries, and unknown-corpus
errors while keeping threshold drift snapshots, ranking, deadness, resolver
confidence behavior, cue-tier behavior, calibration quality, and performance
out of scope.

## [2026-05-15] implementation | Threshold metadata Vitest mirror batch

Added focused Vitest mirrors for the two reviewed metadata-only threshold
suites. The mirrors preserve threshold policy ids, versions, classes, numeric
values, hashes, calibration corpus references, corpus ids, metrics, compact
summaries, and unknown-corpus errors while keeping threshold drift snapshots,
ranking, deadness, resolver confidence behavior, cue-tier behavior,
calibration quality, and performance out of scope.

## [2026-05-15] review | Pre-write inventory hook Vitest mirror

Reviewed `tests/test-pre-write-inventory-hook.mjs` for a future Lane C Vitest
mirror. The review keeps the suite focused on pre-write
`any-inventory.pre.<invocationId>.json` snapshot creation, advisory
`preWrite.anyInventoryPath` stamping, `--no-fresh-audit` absence semantics, P1
advisory field preservation, snapshot type-escape capability metadata, and
shared `any-inventory.json` non-clobbering while keeping broader pre-write
advisory shape tests, cue-tier policy, host hook runtime, post-write lifecycle,
resolver behavior, deadness/ranking, and performance out of scope.

## [2026-05-15] implementation | Pre-write inventory hook Vitest mirror

Added the focused Vitest mirror for `tests/test-pre-write-inventory-hook.mjs`
while keeping the Node suite runnable. The mirror preserves invocation-specific
`any-inventory.pre.<invocationId>.json` snapshot creation, advisory
`preWrite.anyInventoryPath` stamping, `--no-fresh-audit` absence semantics, P1
advisory field preservation, snapshot type-escape capability metadata, and
shared `any-inventory.json` non-clobbering while keeping broader pre-write
advisory shape tests, cue-tier policy, host hook runtime, post-write lifecycle,
resolver behavior, deadness/ranking, and performance out of scope.

## [2026-05-15] review | Pre-write lookup contracts Vitest mirror batch

Reviewed four Lane C pre-write lookup suites for a future Vitest mirror batch:
`tests/test-pre-write-lookup-dep.mjs`,
`tests/test-pre-write-lookup-file.mjs`,
`tests/test-pre-write-lookup-shape.mjs`, and
`tests/test-pre-write-shape-index.mjs`. The review keeps the batch focused on
dependency availability labels, file status labels, exact shape-hash evidence,
and shape-index integration while keeping lookup-name service-operation cues,
cue-tier policy, renderer wording, deadness/ranking, resolver expansion, and
performance cache identity out of scope.

## [2026-05-15] implementation | Pre-write lookup contracts Vitest mirror batch

Added focused Vitest mirrors for the four reviewed pre-write lookup contract
suites. The mirrors preserve dependency availability labels, package-root
normalization, import-graph confidence, file status labels, topology
completeness requirements, parse-error handling, boundary non-evaluation,
domain-cluster watch cues, exact shape-hash evidence, `typeLiteral`
normalization, malformed shape-index handling, and shape-index integration
while keeping lookup-name service-operation cues, cue-tier policy, renderer
wording, deadness/ranking, resolver expansion, and performance cache identity
out of scope.

## [2026-05-15] review | Pre-write input contracts Vitest mirror batch

Added review metadata for the next Lane C pre-write input contract batch:
`tests/test-pre-write-intent.mjs` and
`tests/test-pre-write-canonical-parser.mjs`. The review keeps the future mirror
limited to intent schema normalization, planned type-escape validation,
refactor source safety, canonical owner-claim parsing, free-form prose
rejection, and group-level canonical row exclusion while excluding bootstrap,
mode dispatch, CLI/advisory orchestration, cue tiers, renderer wording,
resolver behavior, deadness/ranking, and performance cache identity.

## [2026-05-15] implementation | Pre-write input contracts Vitest mirror batch

Added focused Vitest mirrors for the reviewed pre-write input contract suites:
`tests/test-pre-write-intent.mjs` and
`tests/test-pre-write-canonical-parser.mjs`. The mirrors preserve intent
schema normalization, planned type-escape validation, structured name and
dependency declarations, refactor source path safety, canonical owner-claim
parsing, free-form prose rejection, severely-any-contaminated owner rows, and
group-level duplicate/common row exclusion while keeping bootstrap, mode
dispatch, CLI/advisory orchestration, cue tiers, renderer wording, resolver
behavior, deadness/ranking, and performance cache identity out of scope.

## [2026-05-15] review | Audit-repo command lifecycle Vitest mirror batch

Reviewed four `audit-repo.mjs` command lifecycle wrapper suites for a future
Vitest mirror batch: `tests/test-audit-repo-canon-draft.mjs`,
`tests/test-audit-repo-check-canon.mjs`,
`tests/test-audit-repo-pre-write.mjs`, and
`tests/test-audit-repo-post-write.mjs`. The review keeps the batch focused on
manifest command blocks, command-result summaries, source scoping, advisory
versus strict exit-code matrices, pre/write mutexes, evidence availability
mirrors, post-write delta summary fields, stdout/stderr ordering, and
shell-safe paths while keeping direct component suites, cue-tier policy,
renderer wording, resolver behavior, deadness/ranking, and performance cache
identity out of scope.

## [2026-05-15] implementation | Audit-repo command lifecycle Vitest mirror batch

Added focused Vitest mirrors for the reviewed `audit-repo.mjs` command
lifecycle wrapper suites: `tests/test-audit-repo-canon-draft.mjs`,
`tests/test-audit-repo-check-canon.mjs`,
`tests/test-audit-repo-pre-write.mjs`, and
`tests/test-audit-repo-post-write.mjs`. The mirrors preserve manifest command
blocks, command-result summaries, source scoping, advisory versus strict
exit-code matrices, pre/write mutexes, evidence availability mirrors,
post-write delta summary fields, stdout/stderr ordering, and shell-safe paths
while keeping direct component suites, cue-tier policy, renderer wording,
resolver behavior, deadness/ranking, and performance cache identity out of
scope.

## [2026-05-15] review | Mode-dispatch Vitest mirror batch

Added review metadata for `tests/test-mode-dispatch.mjs` as a narrow Lane C
mode-dispatch batch. The review keeps the future mirror limited to the pure
`dispatchMode(userText, cwdMeta)` contract: canonical trigger vocabulary,
guard-only non-triggers, repo-context precedence, prose-rewrite and
comment-typo non-triggers, compound guard-plus-verb firing, return-shape
sanity, and deterministic repeat calls. Broader pre-write advisory,
bootstrap, CLI, renderer, cue-tier, resolver, deadness/ranking, generated, and
performance suites remain out of scope.

## [2026-05-15] implementation | Mode-dispatch Vitest mirror batch

Added a focused Vitest mirror for `tests/test-mode-dispatch.mjs`. The mirror
preserves canonical trigger vocabulary drift checks, guard-only non-triggers,
repo-context precedence, prose-rewrite and comment-typo non-triggers,
compound guard-plus-verb firing, trigger return-shape sanity, and deterministic
repeat calls while keeping broader pre-write advisory, bootstrap, CLI,
renderer, cue-tier, resolver, deadness/ranking, generated, and performance
suites separate.

## [2026-05-15] review | Resolver path lookup Vitest mirror batch

Reviewed three Lane D resolver path lookup suites for a future Vitest mirror
batch: `tests/test-resolver-paths.mjs`,
`tests/test-tsconfig-paths-scoped.mjs`, and `tests/test-wildcard.mjs`. The
review keeps the batch focused on extensionless relative paths, resolver
sentinels, scoped baseUrl/tsconfig path lookup, package exports wildcard
lookup, generated-artifact target reasons, and resolver-stage cache identity
while keeping unsupported-family diagnostics, generated blind-zone relevance,
deadness/ranking, topology graph lenses, pre-write cue policy, and broader
performance/incremental suites out of scope.

## [2026-05-15] implementation | Resolver path lookup Vitest mirror batch

Added focused Vitest mirrors for `tests/test-resolver-paths.mjs`,
`tests/test-tsconfig-paths-scoped.mjs`, and `tests/test-wildcard.mjs`. The
mirrors preserve extensionless relative imports, directory indexes,
resource-query asset sentinels, generated asset explanations,
`isResolvedFile()` sentinel discrimination, resolver memo/stage caches,
scoped baseUrl and tsconfig path identity, package exports wildcard matching,
generated virtual surface freezing, unresolved reason summaries, and workspace
fallback behavior while keeping unsupported-family diagnostics, generated
blind-zone relevance, deadness/ranking, topology graph lenses, pre-write cue
policy, and broader performance/incremental suites out of scope.

## [2026-05-15] review | Topology edge lens Vitest mirror batch

Reviewed three topology edge-lens suites for a future Vitest mirror batch:
`tests/test-dynamic-import.mjs`, `tests/test-type-only-reexport.mjs`, and
`tests/test-topology-producer-cross-edges.mjs`. The review keeps the batch
focused on literal dynamic import topology edges, scanner fallback counters,
type-only re-export runtime lens filtering, mixed/runtime SCC survival,
`reExportsByFile` precision, and `crossSubmoduleEdges` full-list artifact shape
while keeping resolver unsupported-family diagnostics, module reachability,
deadness/ranking, action-safety, full audit orchestration, and broader
performance/incremental cache identity out of scope.

## [2026-05-15] implementation | Topology edge lens Vitest mirror batch

Added focused Vitest mirrors for `tests/test-dynamic-import.mjs`,
`tests/test-type-only-reexport.mjs`, and
`tests/test-topology-producer-cross-edges.mjs`. The mirrors preserve literal
dynamic import topology edges, scanner fallback counters, type-only re-export
runtime lens filtering, mixed/runtime SCC survival, exact `reExportsByFile`
coverage, structured `crossSubmoduleEdges`, and the legacy `crossSubmoduleTop`
display shape while keeping resolver unsupported-family diagnostics, module
reachability, deadness/ranking, action-safety, full audit orchestration, and
broader performance/incremental cache identity out of scope.

## [2026-05-16] review | Smoke-uncovered Vitest mirror batch

Reviewed `tests/test-smoke-uncovered.mjs` as a narrow Lane H artifact smoke
batch. The review keeps the future mirror limited to shallow script-entrypoint
execution and recognizable artifact-shape checks for call graph, barrel
discipline, discipline metrics, SARIF empty/classifier/warning flows,
package-lock drift detection, and runtime evidence merge output while keeping
deeper producer semantics, resolver behavior, deadness/ranking, performance,
incremental cache identity, and full audit orchestration out of scope.

## [2026-05-16] implementation | Smoke-uncovered Vitest mirror batch

Added a focused Vitest mirror for `tests/test-smoke-uncovered.mjs`. The mirror
preserves shallow script-entrypoint execution and recognizable artifact-shape
checks for call graph, barrel discipline, discipline metrics, SARIF
empty/classifier/warning flows, package-lock drift detection, and runtime
evidence merge output while keeping deeper producer semantics, resolver
behavior, deadness/ranking, performance, incremental cache identity, and full
audit orchestration out of scope.

## [2026-05-16] review | Classification label emission Vitest mirror batch

Reviewed `tests/test-classification-label-emission-corpus.mjs` as a narrow
Lane B canon label-emission batch. The review keeps the future mirror limited
to the synthetic TS corpus, symbol graph `anyContamination` support, generated
type-ownership Markdown table parsing, and canonical labels emitted through the
public producer path while keeping the larger classifier matrix, classifier
predicate changes, deadness/ranking, resolver behavior, performance, and public
package behavior out of scope.

## [2026-05-16] implementation | Classification label emission Vitest mirror batch

Added a focused Vitest mirror for
`tests/test-classification-label-emission-corpus.mjs`. The mirror preserves the
synthetic TS corpus, symbol graph `anyContamination` support, generated
type-ownership Markdown table parsing, and canonical labels emitted through the
public producer path while keeping the larger classifier matrix, classifier
predicate changes, deadness/ranking, resolver behavior, performance, and public
package behavior out of scope.

## [2026-05-16] review | Audit-repo incremental forwarding Vitest mirror batch

Reviewed `tests/test-audit-repo-symbol-incremental.mjs` and
`tests/test-function-clone-audit-forwarding.mjs` as a narrow Lane H/F
incremental-forwarding batch. The review keeps the future mirror limited to
`audit-repo.mjs` flag forwarding, explicit cache-root propagation, cache-root
paths with spaces, and shared incremental-cache clearing metadata while keeping
symbol graph extraction, function clone normalization, cache identity,
performance counters, deadness/ranking, resolver behavior, and full audit
semantics out of scope.

## [2026-05-16] implementation | Audit-repo incremental forwarding Vitest mirror batch

Added focused Vitest mirrors for
`tests/test-audit-repo-symbol-incremental.mjs` and
`tests/test-function-clone-audit-forwarding.mjs`. The mirrors preserve
`audit-repo.mjs` incremental flag forwarding, explicit cache-root propagation,
cache-root paths with spaces, and shared incremental-cache clearing metadata
while keeping symbol graph extraction, function clone normalization, cache
identity, performance counters, deadness/ranking, resolver behavior, and full
audit semantics out of scope.

## [2026-05-16] review | Incremental core helpers Vitest mirror batch

Reviewed `tests/test-incremental-cache-store.mjs`,
`tests/test-incremental-snapshot.mjs`, and `tests/test-incremental.mjs` as a
narrow Lane F core-helper batch. The review keeps the future mirror limited to
strict cache-store hits/misses, malformed cache handling, cache clearing,
repo-relative snapshot identity, include-tests filtering, package scope,
content hashing, unreadable-file visibility, legacy stat-first-cut behavior,
dropped files, stale cache versions, and cache banner text while keeping
producer-level incremental reuse, scanner fallback, performance counters,
deadness/ranking, resolver behavior, and full audit orchestration out of scope.

## [2026-05-16] implementation | Incremental core helpers Vitest mirror batch

Added focused Vitest mirrors for `tests/test-incremental-cache-store.mjs`,
`tests/test-incremental-snapshot.mjs`, and `tests/test-incremental.mjs`. The
mirrors preserve strict cache-store hits/misses, malformed cache handling,
cache clearing, repo-relative snapshot identity, include-tests filtering,
package scope, content hashing, unreadable-file visibility, legacy
stat-first-cut behavior, dropped files, stale cache versions, and cache banner
text while keeping producer-level incremental reuse, scanner fallback,
performance counters, deadness/ranking, resolver behavior, and full audit
orchestration out of scope.

## [2026-05-16] review | JS module edge scanner Vitest mirror batch

Reviewed `tests/test-js-module-edge-scanner.mjs` as a narrow Lane F scanner
batch. The review keeps the future mirror limited to tokenizer-state scanner
equivalence against Oxc topology edges, fake syntax skipping, import/export
attribute handling, line-number preservation, stable fallback reason codes, JSX
fallback, and the many-string-literal non-quadratic guard while keeping
resolver behavior, topology graph/SCC outputs, producer incremental reuse,
performance counters, deadness/ranking, and public package behavior out of
scope.

## [2026-05-16] implementation | JS module edge scanner Vitest mirror batch

Added a focused Vitest mirror for `tests/test-js-module-edge-scanner.mjs`. The
mirror preserves tokenizer-state scanner equivalence against Oxc topology
edges, fake syntax skipping, import/export attribute handling, line-number
preservation, fallback reason codes for unsupported dynamic/CommonJS/TypeScript
module forms, JSX fallback, and the many-string-literal non-quadratic guard
while keeping scanner implementation, resolver behavior, graph/SCC outputs,
producer incremental reuse, performance counters, deadness/ranking, and public
package behavior out of scope.

## [2026-05-16] review | Producer artifact builders Vitest mirror batch

Reviewed `tests/test-build-shape-index.mjs` and
`tests/test-build-function-clone-index.mjs` as a narrow producer artifact
builder batch. The review keeps the future mirrors limited to
`shape-index.json` and `function-clones.json` producer artifact contracts,
including schema/support metadata, parse-error handling, production scan scope,
shell-safe paths, unsupported shape diagnostics, generated-file evidence,
literal-union grouping, function clone review-only wording, exact body groups,
near-function policy metadata, signature groups, and small exact-body clone
groups while keeping producer-level incremental reuse, resolver behavior,
deadness/ranking, cache identity, performance counters, and full audit
orchestration out of scope.

## [2026-05-16] implementation | Producer artifact builders Vitest mirror batch

Added focused Vitest mirrors for `tests/test-build-shape-index.mjs` and
`tests/test-build-function-clone-index.mjs`. The mirrors preserve
`shape-index.json` and `function-clones.json` producer artifact contracts,
including schema/support metadata, parse-error handling, production scan scope,
shell-safe paths, unsupported shape diagnostics, generated-file evidence,
literal-union grouping, function clone review-only wording, exact body groups,
near-function policy metadata, signature groups, and small exact-body clone
groups while keeping producer-level incremental reuse, resolver behavior,
deadness/ranking, cache identity, performance counters, and full audit
orchestration out of scope.

## [2026-05-16] review | Checklist facts Vitest mirror batch

Reviewed `tests/test-checklist-facts.mjs` as a narrow Lane H checklist
pre-compute artifact batch. The review keeps the future mirror limited to
`checklist-facts.json` graceful degradation, fresh AST-backed A2/E2 facts,
missing-input availability gates, `_not_computed` visibility, citation hints,
context-check flags, role-aware function-size buckets, full-list
cross-submodule evidence, review-only shape/function duplicate cues, silent
catch subcategories, C5 lint-boundary evidence, and pipeline `inputsPresent`
bits while keeping producer implementations, deadness/ranking, resolver
behavior, cache identity, performance counters, pre-write cue tiers, and full
audit orchestration out of scope.

## [2026-05-16] implementation | Checklist facts Vitest mirror batch

Added a focused Vitest mirror for `tests/test-checklist-facts.mjs`. The mirror
preserves `checklist-facts.json` graceful degradation, fresh AST-backed A2/E2
facts, missing-input availability gates, `_not_computed` visibility, citation
hints, context-check flags, role-aware function-size buckets, full-list
cross-submodule evidence, review-only shape/function duplicate cues, silent
catch subcategories, C5 lint-boundary evidence, and pipeline `inputsPresent`
bits while keeping producer implementations, deadness/ranking, resolver
behavior, cache identity, performance counters, pre-write cue tiers, and full
audit orchestration out of scope.

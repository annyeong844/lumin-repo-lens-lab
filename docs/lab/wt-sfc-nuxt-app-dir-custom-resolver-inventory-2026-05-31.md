# WT-SFC Nuxt App-Dir And Custom Resolver Inventory - 2026-05-31

This note records the next WT-SFC Nuxt gap inventory. It follows the
[`SFC support policy`](../spec/sfc-support-policy.md), the earlier
[`framework magic fixture inventory`](wt-sfc-framework-magic-fixture-inventory-2026-05-27.md),
and the supplemental Nuxt finding in the
[`Vue Options corpus calibration`](wt-sfc-vue-options-corpus-calibration-2026-05-31.md).

This began as a fixture and policy inventory for future slices around Nuxt
`#components`, app-dir component conventions, literal component-dir config,
and custom component resolvers. The app-dir, manifest-backed `#components`
alias, literal component-dir config, and high-level custom resolver hook rows
now have source implementations; layer `extends` presence now has high-level
unavailable evidence. Nuxt module package config presence now has high-level
unavailable evidence as well. Returned component lists, layer-merged component
facts, and module-injected virtual registries remain unavailable.

## Decision

Decision: `nuxt-resolver-inventory-before-implementation`,
`generated-manifest-before-alias-resolution`,
`nuxt-app-dir-convention-stays-muted`,
`nuxt-app-dir-requires-app-src-signal`,
`nuxt-components-alias-requires-generated-manifest`,
`literal-component-dir-config-stays-directory-evidence`,
`custom-resolver-stays-unavailable`, `layer-extends-stays-unavailable`,
`module-package-stays-unavailable`, `scan-gap-stays`, and
`no-action-surface`.

The current WT-SFC MVP has useful Nuxt-adjacent evidence:

- generated `.nuxt/components.d.ts` manifest entries in
  `symbols.json.sfcGeneratedComponentManifests[]`;
- root `components/` filesystem convention evidence in
  `symbols.json.sfcFrameworkConventionComponents[]`;
- unplugin/Nuxt-adjacent config evidence when a known plugin call is observed.

Those lanes are not enough to claim full Nuxt component availability. Nuxt
`#components`, `app/components`, layers, and custom resolver functions may all
make components available without leaving a simple static file-to-name mapping.
They need their own bounded fixtures before any absence claim gets stronger.

## Bucket Semantics

Use the same three buckets as the framework magic inventory:

- `explicit-supportable`: a concrete source or generated artifact can name the
  component and the target. Presence may create review-only availability
  evidence. Absence is not proof.
- `muted-observed`: source/config/convention shape is visible, but the analyzer
  must not turn it into graph edges, fan-in, deadness, `SAFE_FIX`, or action
  evidence.
- `unavailable`: Nuxt/compiler/runtime behavior is not statically observable in
  this lane. Do not invent per-component targets.

## Nuxt Gap Matrix

| Shape | Bucket | Future evidence rule |
| ----- | ------ | -------------------- |
| `.nuxt/components.d.ts` literal declaration | `explicit-supportable` | Existing generated-manifest lane. SFC targets stay muted with `resolvedFile`; source targets may resolve; missing targets are `unresolved`. |
| Root `components/` files with a Nuxt signal | `muted-observed` | Existing Nuxt filesystem convention lane with `sfc-framework-nuxt-fs-convention`; path-derived names only; no graph/fan-in/action effect. |
| `app/components/**` files with a Nuxt 4 dependency or explicit `srcDir: "app"` signal | `muted-observed` | App-dir convention lane records navigation evidence with `sfc-framework-nuxt-app-dir-convention`; no consumption claim. A generic Nuxt 3 signal is not enough because [`srcDir`](https://nuxt.com/docs/4.x/api/nuxt-config#srcdir) defaults to `app` for Nuxt 4 and `.` for Nuxt 3 compatibility mode. |
| Static `#components` import backed by `.nuxt/components.d.ts` | `explicit-supportable` | Alias lane connects imported binding names to generated-manifest entries with `sfc-framework-nuxt-components-alias-manifest`, but remains review-only and has no graph/fan-in/action effect. |
| Static `#components` import without a generated manifest | `muted-observed` | Alias diagnostic records component-like imports as observed but unresolved with `sfc-framework-nuxt-components-alias-unresolved`; no guessed target. Known virtual helper exports such as `componentNames` are ignored instead of being recorded as component evidence. |
| Literal `components.dirs` config | `muted-observed` | Implemented config evidence records literal directory signals with `sfc-framework-nuxt-components-dir-config`; it preserves directory/prefix metadata only, resolves `~/` and `@/` through Nuxt `srcDir` semantics, and does not scan arbitrary dirs into component targets. |
| Custom resolver functions or Nuxt module hooks | `unavailable` | Implemented hook-presence evidence records literal `hooks["components:dirs"]` and `hooks["components:extend"]` with `sfc-framework-nuxt-custom-resolver-unavailable`; function execution, returned component lists, and module-provided virtual registries remain unavailable. |
| Nuxt layers, `extends`, and generated build virtuals | `unavailable` | Implemented config-presence evidence records literal and nonliteral `extends` entries with `sfc-framework-nuxt-layer-extends-unavailable`; layer merge execution and per-component availability remain unavailable. Generated manifests may still be read when present. |
| Nuxt `modules` package config | `unavailable` | Implemented config-presence evidence records literal module packages, tuple module packages, and nonliteral module entries with `sfc-framework-nuxt-module-package-unavailable`; module execution and module-injected component registries remain unavailable. |

## Fixture Matrix

Future implementation slices should start with failing fixtures that prove the
boundary before adding behavior.

| ID | Fixture | Expected result |
| -- | ------- | --------------- |
| N1 | `.nuxt/components.d.ts` maps `BaseButton` to `components/BaseButton.vue`. | Existing generated-manifest record: muted SFC target with `resolvedFile`; no graph edge. |
| N2 | `components/base/Button.vue` under a Nuxt project. | Existing convention record with path-derived `BaseButton`; muted review-only. |
| N3 | `app/components/base/Button.vue` with a Nuxt 4 dependency or `nuxt.config.ts` literal `srcDir: "app"`. | App-dir convention record with path-derived `BaseButton`; muted review-only. |
| N4 | `app/components/base/Button.vue` with only a Nuxt 3 dependency signal. | No app-dir convention record; root `components/` evidence may still emit if present. |
| N5 | `import { BaseButton } from "#components"` with a generated manifest mapping. | Alias evidence connects the binding to the manifest target and records `resolvedFile`; still review-only for SFC targets. |
| N6 | `import { BaseButton } from "#components"` without a generated manifest. | Observed unresolved alias diagnostic with `sfc-framework-nuxt-components-alias-unresolved`; no guessed file path. |
| N7 | `import { componentNames } from "#components"`. | No alias component evidence is emitted; `componentNames` is a virtual helper export, not a component name. |
| N8 | `nuxt.config.ts` contains literal `components: [{ path: "~/shared/components", prefix: "Shared" }]`. | Config evidence records the literal directory, prefix, and optional resolved directory only; no component target inference. |
| N9 | `nuxt.config.ts` uses `hooks: { "components:dirs"() {}, "components:extend": () => {} }`. | High-level unavailable hook records are emitted with `sfc-framework-nuxt-custom-resolver-unavailable`; no component names, target files, graph edges, or fan-in. |
| N10 | `nuxt.config.ts` uses `extends: ["../layer-a", layerPreset]`. | High-level unavailable layer records are emitted with `sfc-framework-nuxt-layer-extends-unavailable`; literal entries preserve the configured source, nonliteral entries preserve only the unavailable signal, and neither form creates component names, target files, graph edges, or fan-in. |
| N11 | `nuxt.config.ts` uses `modules: ["@nuxt/image", ["@nuxtjs/tailwindcss", {}], customModule]`. | High-level unavailable module records are emitted with `sfc-framework-nuxt-module-package-unavailable`; literal and tuple literal entries preserve the package source, nonliteral entries preserve only the unavailable signal, and no module is executed. |
| N12 | Any future Nuxt record is emitted. | `eligibleForFanIn: false`, `eligibleForSafeFix: false`, no `resolvedInternalEdges[]`, no named export fan-in, no deadness/action/SARIF/package-edit effect. |

## Required Future Reason Codes

Keep existing reason codes stable:

- `sfc-framework-generated-manifest-non-source-binding`;
- `sfc-framework-generated-manifest-unresolved`;
- `sfc-framework-generated-manifest-nonliteral`;
- `sfc-framework-nuxt-fs-convention`;
- `sfc-framework-nuxt-app-dir-convention`;
- `sfc-framework-nuxt-components-alias-manifest`;
- `sfc-framework-nuxt-components-alias-unresolved`;
- `sfc-framework-auto-import-plugin-config`;
- `sfc-framework-nuxt-components-dir-config`;
- `sfc-framework-nuxt-custom-resolver-unavailable`;
- `sfc-framework-nuxt-layer-extends-unavailable`;
- `sfc-framework-nuxt-module-package-unavailable`.

Future Nuxt slices should add new reason codes only when they have fixtures:

- Additional Nuxt module or virtual-registry reason codes for observed shapes
  that cannot yield static per-component facts.

## Safety Contract

All Nuxt convention and resolver evidence remains review-only until a future
spec explicitly proves stronger semantics:

1. Do not feed Nuxt convention evidence into `resolvedInternalEdges[]`.
2. Do not create named export fan-in from generated manifests, app-dir files,
   or `#components` alias evidence.
3. Do not rank Nuxt convention evidence as deadness proof.
4. Do not create `SAFE_FIX`, `EXISTS`, fix-plan, export-action-safety, SARIF,
   or package-edit output from these records.
5. Keep `sfc-scan-gap` visible. These records narrow review navigation; they do
   not mean the analyzer understands all Nuxt SFC semantics.

## Current Calibration Signal

The supplemental `nuxt-main` run in the
[`Vue Options corpus calibration`](wt-sfc-vue-options-corpus-calibration-2026-05-31.md#supplemental-nuxt-run)
is the trigger for this inventory. It produced explicit Vue template evidence
but no framework-convention records, and it identified Nuxt `#components` plus
app-dir component convention semantics as custom-resolver/framework gaps.

That is the right failure mode. The MVP should say "we saw the gap" before it
tries to guess Nuxt's build-time behavior.

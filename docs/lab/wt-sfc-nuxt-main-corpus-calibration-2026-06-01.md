# WT-SFC Nuxt Main Corpus Calibration - 2026-06-01

This report records a WT-SFC corpus calibration run against `nuxt-main` after
the beta.85 public package. It follows the
[`WT-SFC corpus calibration plan`](wt-sfc-corpus-calibration-plan-2026-05-31.md),
the current
[`WT-SFC MVP status`](wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md),
and the
[`Nuxt app-dir/custom resolver inventory`](wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md).

This is corpus calibration only. It does not change analyzer behavior and does
not mark WT-SFC `DONE`.

## Run Metadata

| Field | Value |
| ----- | ----- |
| Corpus | `C:/Users/endof/Downloads/nuxt-main` |
| Corpus shape | Nuxt monorepo, Vue SFCs only |
| Package route | Public installed marketplace clone |
| Package version | `0.9.0-beta.85` |
| Command | `node C:/Users/endof/.claude/plugins/marketplaces/annyeong844-marketplace/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root C:/Users/endof/Downloads/nuxt-main --output C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-nuxt-main-beta85-20260601 --profile full` |
| Node | `v24.14.0` |
| Output | `C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-nuxt-main-beta85-20260601/` |
| Result | PASS, 22 artifacts produced |

## Aggregate Metrics

| Metric | Value |
| ------ | ----: |
| `sfcFileCount` | 315 |
| `byLanguage` | `{ vue: 315 }` |
| `scriptImportConsumerCount` | 17 |
| `scriptSrcReachabilityCount` | 0 |
| `styleAssetReferenceCount` | 0 |
| `templateRefCount` | 24 |
| `globalRegistrationCount` | 0 |
| `generatedManifestCount` | 0 |
| `frameworkConventionCount` | 10 |
| `reviewOnlyEvidenceCount` | 34 |
| `totalSfcEvidenceCount` | 51 |
| `rawMarkdownLeakCount` | 0 for sampled SFC names/reasons |
| `actionLeakCount` | 0 strict SFC-policy matches |
| `falsePositiveCount` | 1 sampled record |
| `missedUsefulEvidenceCount` | 0 for the current MVP contract; one remaining manifest-less `#components` family stays a gap |

`manifest.json.sfcEvidence` was `status: "complete"` with
`scanGapStillApplies: true`. `manifest.json.blindZones[]` preserved one
`sfc-scan-gap` entry with `files: 315` and `languages: { "vue": 315 }`.

## Framework Convention Distribution

`symbols.json.sfcFrameworkConventionComponents[]` contained 10 records:

| Reason | Status | Count |
| ------ | ------ | ----: |
| `sfc-framework-nuxt-components-alias-unresolved` | `unresolved` | 8 |
| `sfc-framework-nuxt-module-package-unavailable` | `unavailable` | 2 |

The two module records came from nonliteral function entries in the root
`nuxt.config.ts` `modules` array. They correctly preserved only the
module-execution capability-gap signal:
`reason: "sfc-framework-nuxt-module-package-unavailable"`,
`moduleSourceKind: "nonliteral"`, and no `moduleSource`.

The eight `#components` alias records came from SFC script imports without a
generated `.nuxt/components.d.ts` manifest in the corpus checkout. They stayed
review-only with `eligibleForFanIn: false` and `eligibleForSafeFix: false`.

## Sampled Evidence Review

| Lane | Source | Status | Reason | Review label | Note |
| ---- | ------ | ------ | ------ | ------------ | ---- |
| template ref | `packages/nuxt/src/app/components/nuxt-root.vue` `<AppComponent />` | `unresolved` | `sfc-template-component-unresolved` | `useful-review-evidence` | Points at `#build/app-component.mjs`; this is a real Nuxt virtual/build gap, not fan-in proof. |
| template ref | `packages/nuxt/test/components-fixture/components/client/WithClientOnlySetup.vue` `<HelloWorld />` | `muted` | `sfc-template-component-non-source-binding` | `useful-review-evidence` | Preserves `resolvedFile` for an SFC target while staying out of fan-in. |
| framework convention | `WithClientOnlySetup.vue` import `{ ClientImport }` from `#components` | `unresolved` | `sfc-framework-nuxt-components-alias-unresolved` | `useful-review-evidence` | Correctly records a Nuxt alias import without guessing a manifest target. |
| framework convention | `test/fixtures/basic/app/pages/nuxt-link/use-link.vue` import `{ NuxtLink }` from `#components` | `unresolved` | `sfc-framework-nuxt-components-alias-unresolved` | `useful-review-evidence` | Shows the manifest-less built-in component alias gap. |
| framework convention | `nuxt.config.ts` `modules: [function () { ... }]` | `unavailable` | `sfc-framework-nuxt-module-package-unavailable` | `useful-review-evidence` | Correctly discloses module execution without executing the module. |
| framework convention | `test/fixtures/minimal/app.vue` import `{ componentNames }` from `#components` | `unresolved` | `sfc-framework-nuxt-components-alias-unresolved` | `false-positive` | `componentNames` is a virtual helper export, not a component name. The record is harmless but noisy. |

## False-Positive Table

| Record | Why It Is Noise | Impact |
| ------ | --------------- | ------ |
| `componentName: "componentNames"` from `test/fixtures/minimal/app.vue` | `#components` can export helper data such as `componentNames`; treating every named import as a component record is too broad. | Harmless because the record is unresolved, review-only, and not eligible for fan-in or safe fixes. It should still be tightened before stronger Nuxt alias wording. |

## Missed Useful Evidence

| Gap | Current Behavior | Decision |
| --- | ---------------- | -------- |
| Manifest-less Nuxt built-ins and virtual aliases, such as `NuxtLink` from `#components` | Recorded as unresolved review-only alias evidence because no generated `.nuxt/components.d.ts` mapping exists. | Keep as a gap. Do not guess built-in or virtual component targets without a new fixture-pinned spec. |

## Leakage Checks

| Surface | Result |
| ------- | ------ |
| `resolvedInternalEdges[]` | 0 SFC framework/template/global/source hits |
| `fanInByIdentity` | 0 SFC / `#components` / Nuxt alias keys |
| `fix-plan.json` | 0 strict SFC-policy matches; 2 raw `#components` mentions appear only in resolver top-unresolved metadata |
| `export-action-safety.json` | 0 strict SFC-policy matches |
| `dead-classify.json` | 0 strict SFC-policy matches |
| `audit-summary.latest.md` | SFC count-only line present; sampled raw SFC component/alias names absent |
| `audit-review-pack.latest.md` | SFC count-only line present; sampled raw SFC component/alias names absent |

The raw `#components` mentions in `fix-plan.json` are not SFC evidence leakage:
they come from general resolver metadata for non-SFC TypeScript imports and are
reported under `hash-imports-unsupported`.

## Decision

Decision: `keep-current-mvp`, `custom-resolver-still-gap`,
`scan-gap-stays`, `no-action-surface`, and
`tighten-nuxt-components-alias-helper-exports-before-stronger-wording`.

Beta.85 is safe on this Nuxt corpus for the current review-only SFC boundary:
SFC evidence is useful, default Markdown remains count-only, and action/graph
surfaces stay clean. The run also found a real calibration signal:
manifest-less `#components` aliases are helpful as unresolved review evidence,
but the alias lane should filter or separately classify non-component helper
exports such as `componentNames` before any stronger Nuxt alias wording is
enabled.

Follow-up source guards now pin that boundary in
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs):
`componentNames` from `#components` is filtered instead of emitted as Nuxt
component alias evidence.

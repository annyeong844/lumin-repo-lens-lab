# WT-SFC Vue Options Corpus Calibration - 2026-05-31

This report applies the
[`WT-SFC corpus calibration plan`](wt-sfc-corpus-calibration-plan-2026-05-31.md)
to Vue-focused corpus runs with the beta.78 public package. It does not change
analyzer behavior.

The primary run uses the Storybook Vue CLI template because it has real `.vue`
files with local component imports, template component usage, and Vue Options
API `components` registration. A supplemental Nuxt run is also recorded because
it shows a useful custom-resolver gap: Vue template evidence is visible, but
Nuxt `#components` and app-dir convention semantics are still not modeled.

This is a Vue partial calibration pass. It covers Vue template refs and Vue
Options API convention evidence, but it does not close the full Vue corpus leg
because the selected corpus does not exercise app/global registration and the
Nuxt corpus produced no framework-convention records.

## Primary Run

| Field | Value |
| ----- | ----- |
| Corpus | `storybook-next/code/renderers/vue3/template/cli/js` |
| Framework/language mix | Vue CLI template with Options API component registration |
| Root path class | local checkout under `C:/Users/endof/Downloads/storybook-next/code/renderers/vue3/template/cli/js` |
| Public package | `0.9.0-beta.78` from `C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab` |
| Command route | `node <public-package-clone>/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root <repo> --output <out> --profile full` |
| Output path | `C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-storybook-vue-cli-js-beta78` |
| Node | `v24.14.0` |
| Profile | `full` |
| Result | PASS, 22 artifacts produced |

Command:

```text
node C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root C:/Users/endof/Downloads/storybook-next/code/renderers/vue3/template/cli/js --output C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-storybook-vue-cli-js-beta78 --profile full
```

## Aggregate Metrics

| Metric | Value |
| ------ | ----- |
| `sfcFileCount` | 3 |
| `byLanguage` | `vue: 3` |
| `scriptImportConsumerCount` | 0 |
| `scriptSrcReachabilityCount` | 0 |
| `styleAssetReferenceCount` | 0 |
| `templateRefCount` | 4 |
| `globalRegistrationCount` | 0 |
| `generatedManifestCount` | 0 |
| `frameworkConventionCount` | 2 |
| `vueOptionsRegistrationCount` | 2 |
| `reviewOnlyEvidenceCount` | 6 |
| `totalSfcEvidenceCount` | 6 |
| `rawMarkdownLeakCount` | 0 of 4 sampled SFC advisory names |
| `actionLeakCount` | 0 SFC-policy matches in `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` |
| `falsePositiveCount` | 0 among sampled Vue Options and template evidence records |
| `missedUsefulEvidenceCount` | 0 in the selected primary template corpus |

`manifest.json.sfcEvidence` reported:

```json
{
  "status": "complete",
  "scriptImportConsumerCount": 0,
  "reachabilityOnlyCount": 0,
  "reviewOnlyEvidenceCount": 6,
  "totalEvidenceCount": 6,
  "byLane": {
    "scriptImportConsumers": 0,
    "scriptSrcReachability": 0,
    "styleAssetReferences": 0,
    "templateComponentRefs": 4,
    "globalComponentRegistrations": 0,
    "generatedComponentManifests": 0,
    "frameworkConventionComponents": 2
  },
  "scanGapStillApplies": true
}
```

## Blind Zone

The run emitted one `sfc-scan-gap` blind zone:

```json
{
  "area": "sfc-scan-gap",
  "severity": "scan-gap",
  "details": {
    "files": 3,
    "languages": {
      "vue": 3
    },
    "reason": "sfc-extractor-not-registered"
  }
}
```

That is the correct MVP behavior. The analyzer records bounded Vue template
and Options API review evidence, but it still does not claim full Vue template
semantics.

## Sampled Evidence

`symbols.json.sfcFrameworkConventionComponents[]` contained two Vue Options API
records:

| Component | Consumer | Binding Source | Status | Reason |
| --------- | -------- | -------------- | ------ | ------ |
| `MyButton` | `Header.vue` | `./Button.vue` | `muted` | `sfc-framework-vue-options-registration` |
| `MyHeader` | `Page.vue` | `./Header.vue` | `muted` | `sfc-framework-vue-options-registration` |

Both records used `source: "sfc-framework-vue-options-registration"`,
`confidence: "framework-convention-observed"`,
`eligibleForFanIn: false`, and `eligibleForSafeFix: false`.

`symbols.json.sfcTemplateComponentRefs[]` contained four review-only template
records:

| Tag | Consumer | Target | Status | Reason |
| --- | -------- | ------ | ------ | ------ |
| `my-button` | `Header.vue` | `Button.vue` | `muted` | `sfc-template-component-non-source-binding` |
| `my-header` | `Page.vue` | `Header.vue` | `muted` | `sfc-template-component-non-source-binding` |

The repeated `my-button` records correspond to repeated visible uses in the
template. They are useful review navigation evidence and not fan-in or action
proof.

## Supplemental Nuxt Run

The supplemental run used `nuxt-main` to test a broader Vue/Nuxt repository:

```text
node C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root C:/Users/endof/Downloads/nuxt-main --output C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-nuxt-main-vue-beta78 --profile full
```

The run produced 22 artifacts and reported:

| Metric | Value |
| ------ | ----- |
| `sfcFileCount` | 315 |
| `byLanguage` | `vue: 315` |
| `scriptImportConsumerCount` | 17 |
| `templateRefCount` | 24 |
| `frameworkConventionCount` | 0 |
| `reviewOnlyEvidenceCount` | 24 |
| `totalSfcEvidenceCount` | 41 |
| `rawMarkdownLeakCount` | 0 of 15 sampled SFC advisory names |
| `actionLeakCount` | 0 SFC-policy matches |

The Nuxt run is useful but not a completed Vue leg. It shows that current
beta.78 records explicit Vue template bindings, including muted SFC targets
with `resolvedFile`, but it does not model Nuxt `#components`, generated app
component aliases, or app-dir component convention semantics. Those remain
custom-resolver/framework-convention gaps, so stronger absence claims are still
blocked.

## False-Positive Review

The sampled Vue evidence records were reviewed for noisy or misleading claims.
No false-positive SFC evidence was found in the primary Storybook Vue template
run.

| Lane | Sample | Review Label | Result |
| ---- | ------ | ------------ | ------ |
| `framework-convention` | `MyButton` in `Header.vue` Options API `components` | `useful-review-evidence` | Correct muted Options API evidence. |
| `framework-convention` | `MyHeader` in `Page.vue` Options API `components` | `useful-review-evidence` | Correct muted Options API evidence. |
| `template-ref` | `<my-button>` in `Header.vue` | `useful-review-evidence` | Correct muted SFC-to-SFC navigation evidence. |
| `template-ref` | `<my-header>` in `Page.vue` | `useful-review-evidence` | Correct muted SFC-to-SFC navigation evidence. |

False-positive table:

| Evidence | Reason |
| -------- | ------ |
| None | Sampled Vue Options and template evidence matched visible source syntax and stayed review-only. |

## Missed Useful Evidence

The primary Storybook Vue template did not expose missed evidence in the
current MVP lanes. The supplemental Nuxt run did expose a broader limitation:

| Missing Evidence | Corpus | Why It Was Missed |
| ---------------- | ------ | ----------------- |
| Nuxt `#components` resolution and app-dir component conventions | `nuxt-main` | The current MVP records explicit imports and selected framework evidence only; Nuxt generated aliases and component convention semantics remain custom-resolver/framework gaps. |
| Vue app/global registration corpus coverage | both runs | The selected primary corpus does not include `app.component(...)`; this needs a separate Vue app corpus before the Vue leg is closed. |

## Markdown And Action Surface Checks

`audit-summary.latest.md` rendered the count-only brief for the primary run:

```text
SFC evidence: 6 records across template refs 4, framework conventions 2; 6 review-only records. Read `manifest.json.sfcEvidence` and SFC arrays in `symbols.json`; review-only SFC lanes are not fan-in or action-tier proof, and sfc-scan-gap still applies.
```

`audit-review-pack.latest.md` rendered:

```text
SFC evidence review: inspect manifest.json.sfcEvidence and SFC arrays in symbols.json before treating SFC absence as deadness. template-refs=4; framework-conventions=2; review-only=6; sfc-scan-gap still applies.
```

Checks:

- Four advisory names sampled from SFC review-only arrays had zero occurrences
  in `audit-summary.latest.md` and `audit-review-pack.latest.md`.
- `resolvedInternalEdges[]` had zero SFC-specific graph edges.
- `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` had
  zero SFC-policy matches in both the primary Storybook run and supplemental
  Nuxt run.

## Decision

Decision: `vue-options-template-corpus-covered`,
`vue-corpus-leg-still-open`, `nuxt-custom-resolver-still-gap`,
`scan-gap-stays`, and `no-action-surface`.

The beta.78 MVP behaved correctly on a Vue Options API/template corpus:
Options API and template evidence appeared as muted review-only records,
default Markdown stayed count-only, `sfc-scan-gap` remained present, and no SFC
review-only evidence entered graph, fan-in, deadness, or action lanes.

This does not complete the full Vue corpus leg. Before WT-SFC can claim the
Vue corpus gate is covered, a future Vue app run should exercise app/global
registration and at least one broader framework convention or generated
manifest surface in the same corpus, or explicitly record why no such corpus is
available.

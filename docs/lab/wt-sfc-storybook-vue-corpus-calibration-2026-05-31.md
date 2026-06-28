# WT-SFC Storybook Vue Corpus Calibration - 2026-05-31

This report applies the
[`WT-SFC corpus calibration plan`](wt-sfc-corpus-calibration-plan-2026-05-31.md)
to a Vue corpus that exercises template refs, Options API component
registration, and global app registration in one public-package run. It uses
the beta.78 public package and does not change analyzer behavior.

This run complements the earlier
[`Vue Options corpus calibration`](wt-sfc-vue-options-corpus-calibration-2026-05-31.md),
which covered Vue Options and template evidence but left the full Vue leg open
because it did not include `app.component(...)` registration.

## Run

| Field | Value |
| ----- | ----- |
| Corpus | `storybook-next/code/renderers/vue3/template` |
| Framework/language mix | Storybook Vue template with `.vue` files, Options API registrations, and `setup((app) => app.component(...))` |
| Root path class | local checkout under `C:/Users/endof/Downloads/storybook-next/code/renderers/vue3/template` |
| Public package | `0.9.0-beta.78` from `C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab` |
| Command route | `node <public-package-clone>/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root <repo> --output <out> --profile full` |
| Output path | `C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-storybook-vue-global-beta78` |
| Node | `v24.14.0` |
| Profile | `full` |
| Result | PASS, 22 artifacts produced |

Command:

```text
node C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root C:/Users/endof/Downloads/storybook-next/code/renderers/vue3/template --output C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-storybook-vue-global-beta78 --profile full
```

## Aggregate Metrics

| Metric | Value |
| ------ | ----- |
| `sfcFileCount` | 28 |
| `byLanguage` | `vue: 28` |
| `scriptImportConsumerCount` | 6 |
| `scriptSrcReachabilityCount` | 0 |
| `styleAssetReferenceCount` | 0 |
| `templateRefCount` | 8 |
| `globalRegistrationCount` | 1 |
| `generatedManifestCount` | 0 |
| `frameworkConventionCount` | 2 |
| `vueOptionsRegistrationCount` | 2 |
| `reviewOnlyEvidenceCount` | 11 |
| `totalSfcEvidenceCount` | 17 |
| `rawMarkdownLeakCount` | 0 of 5 sampled SFC advisory names |
| `actionLeakCount` | 0 SFC-policy matches in `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` |
| `falsePositiveCount` | 0 among sampled Vue template, Options, and global-registration evidence records |
| `missedUsefulEvidenceCount` | 0 for the selected gate; one runtime registry target remains intentionally unresolved |

`manifest.json.sfcEvidence` reported:

```json
{
  "status": "complete",
  "scriptImportConsumerCount": 6,
  "reachabilityOnlyCount": 0,
  "reviewOnlyEvidenceCount": 11,
  "totalEvidenceCount": 17,
  "byLane": {
    "scriptImportConsumers": 6,
    "scriptSrcReachability": 0,
    "styleAssetReferences": 0,
    "templateComponentRefs": 8,
    "globalComponentRegistrations": 1,
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
    "files": 28,
    "languages": {
      "vue": 28
    },
    "reason": "sfc-extractor-not-registered"
  }
}
```

That remains the correct MVP behavior. The analyzer records grounded Vue
review evidence, but it still does not claim full Vue template or framework
semantics.

## Sampled Evidence

`symbols.json.sfcGlobalComponentRegistrations[]` contained one global
registration record:

| Component | Registration File | Status | Reason |
| --------- | ----------------- | ------ | ------ |
| `GlobalButton` | `stories/preview.js` | `muted` | `sfc-global-component-value-unsupported` |

The source shape is `setup((app) => app.component('GlobalButton',
globalThis.__TEMPLATE_COMPONENTS__.Button))`. The record is useful because it
proves a visible Vue app registration exists. It stays muted because the target
comes from a runtime `globalThis.__TEMPLATE_COMPONENTS__` registry, so the
analyzer must not invent a `resolvedFile` or fan-in edge.

`symbols.json.sfcFrameworkConventionComponents[]` contained two Vue Options API
records:

| Component | Consumer | Binding Source | Status | Reason |
| --------- | -------- | -------------- | ------ | ------ |
| `MyButton` | `cli/js/Header.vue` | `./Button.vue` | `muted` | `sfc-framework-vue-options-registration` |
| `MyHeader` | `cli/js/Page.vue` | `./Header.vue` | `muted` | `sfc-framework-vue-options-registration` |

`symbols.json.sfcTemplateComponentRefs[]` contained eight review-only template
records, including:

| Tag | Consumer | Target | Status | Reason |
| --- | -------- | ------ | ------ | ------ |
| `my-button` | `cli/js/Header.vue` | `cli/js/Button.vue` | `muted` | `sfc-template-component-non-source-binding` |
| `my-header` | `cli/js/Page.vue` | `cli/js/Header.vue` | `muted` | `sfc-template-component-non-source-binding` |
| `my-button` | `cli/ts/Header.vue` | `cli/ts/Button.vue` | `muted` | `sfc-template-component-non-source-binding` |
| `my-header` | `cli/ts/Page.vue` | `cli/ts/Header.vue` | `muted` | `sfc-template-component-non-source-binding` |

All sampled review-only records kept `eligibleForFanIn: false` and
`eligibleForSafeFix: false`.

## False-Positive Review

The sampled Vue evidence records were reviewed for noisy or misleading claims.
No false-positive SFC evidence was found.

| Lane | Sample | Review Label | Result |
| ---- | ------ | ------------ | ------ |
| `global-registration` | `GlobalButton` from `stories/preview.js` | `useful-review-evidence` | Correct muted registration evidence; target value is unsupported and no `resolvedFile` is invented. |
| `framework-convention` | `MyButton` in `cli/js/Header.vue` Options API `components` | `useful-review-evidence` | Correct muted Options API evidence. |
| `framework-convention` | `MyHeader` in `cli/js/Page.vue` Options API `components` | `useful-review-evidence` | Correct muted Options API evidence. |
| `template-ref` | `<my-button>` in `cli/js/Header.vue` | `useful-review-evidence` | Correct muted SFC-to-SFC navigation evidence. |
| `template-ref` | `<my-header>` in `cli/js/Page.vue` | `useful-review-evidence` | Correct muted SFC-to-SFC navigation evidence. |

False-positive table:

| Evidence | Reason |
| -------- | ------ |
| None | Sampled Vue template, Options, and global registration evidence matched visible source syntax and stayed review-only. |

## Missed Useful Evidence

No required evidence was missing for this Vue corpus gate. The only important
limitation is target precision for the `GlobalButton` registration:

| Limitation | Reason |
| ---------- | ------ |
| `GlobalButton` has no `resolvedFile` | The component value comes from `globalThis.__TEMPLATE_COMPONENTS__.Button`; resolving that runtime registry would require Storybook-specific runtime modeling outside the current MVP. |

This limitation is acceptable for the MVP because the global registration
record remains visible and muted, and it does not create fan-in, deadness, or
action-tier proof.

## Markdown And Action Surface Checks

`audit-summary.latest.md` rendered the count-only brief:

```text
SFC evidence: 17 records across script imports 6, template refs 8, global registrations 1, framework conventions 2; 11 review-only records. Read `manifest.json.sfcEvidence` and SFC arrays in `symbols.json`; review-only SFC lanes are not fan-in or action-tier proof, and sfc-scan-gap still applies.
```

`audit-review-pack.latest.md` rendered:

```text
SFC evidence review: inspect manifest.json.sfcEvidence and SFC arrays in symbols.json before treating SFC absence as deadness. script-imports=6; template-refs=8; global-registrations=1; framework-conventions=2; review-only=11; sfc-scan-gap still applies.
```

Checks:

- Five advisory names sampled from SFC review-only arrays had zero occurrences
  in `audit-summary.latest.md` and `audit-review-pack.latest.md`.
- `resolvedInternalEdges[]` had 21 entries, but zero SFC-specific graph edges.
- No `GlobalButton`, `app.component`, or SFC-policy match appeared in
  `fix-plan.json`, `export-action-safety.json`, or `dead-classify.json`.
- No sampled SFC evidence created fan-in keys.

## Decision

Decision: `storybook-vue-corpus-covered-with-runtime-registry-gap`,
`current-mvp-safe-on-vue-corpus`, `scan-gap-stays`, and `no-action-surface`.

The beta.78 MVP behaved correctly on a Vue corpus that includes local SFC
template refs, Vue Options API registration, and Vue app/global registration:
evidence appeared as muted review-only records, default Markdown stayed
count-only, `sfc-scan-gap` remained present, and no SFC review-only evidence
entered graph, fan-in, deadness, or action lanes.

This covers the Vue corpus leg for the current MVP. WT-SFC remains `MVP`, not
`DONE`: Nuxt custom resolvers, runtime registries, broad template semantics,
and stronger absence/action claims remain out of scope.

# WT-SFC Vite Corpus Calibration - 2026-05-31

This report applies the
[`WT-SFC corpus calibration plan`](wt-sfc-corpus-calibration-plan-2026-05-31.md)
to a first mixed SFC smoke corpus. It uses the beta.78 public package and does
not change analyzer behavior.

This run is useful evidence, but it does not satisfy the full calibration gate:
WT-SFC still needs separate Vue, Svelte, and Astro app corpus reports before
stronger absence claims or action surfaces are considered.

## Run

| Field | Value |
| ----- | ----- |
| Corpus | `vite-main` |
| Root path class | local checkout under `C:/Users/endof/Downloads/vite-main` |
| Public package | `0.9.0-beta.78` from `C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab` |
| Command route | `node <public-package-clone>/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root <repo> --output <out> --profile full` |
| Output path | `C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-vite-beta78` |
| Profile | `full` |
| Result | PASS, 22 artifacts produced |

Command:

```text
node C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root C:/Users/endof/Downloads/vite-main --output C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-vite-beta78 --profile full
```

## Aggregate Metrics

| Metric | Value |
| ------ | ----- |
| `sfcFileCount` | 31 |
| `byLanguage` | `vue: 26`, `svelte: 4`, `astro: 1` |
| `scriptImportConsumerCount` | 4 |
| `scriptSrcReachabilityCount` | 0 |
| `styleAssetReferenceCount` | 0 |
| `templateRefCount` | 28 |
| `globalRegistrationCount` | 0 |
| `generatedManifestCount` | 0 |
| `frameworkConventionCount` | 0 |
| `reviewOnlyEvidenceCount` | 28 |
| `totalSfcEvidenceCount` | 32 |
| `rawMarkdownLeakCount` | 0 of 21 sampled SFC advisory names |
| `actionLeakCount` | 0 SFC-policy matches in `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` |

`manifest.json.sfcEvidence` reported:

```json
{
  "status": "complete",
  "scriptImportConsumerCount": 4,
  "reachabilityOnlyCount": 0,
  "reviewOnlyEvidenceCount": 28,
  "totalEvidenceCount": 32,
  "byLane": {
    "scriptImportConsumers": 4,
    "scriptSrcReachability": 0,
    "styleAssetReferences": 0,
    "templateComponentRefs": 28,
    "globalComponentRegistrations": 0,
    "generatedComponentManifests": 0,
    "frameworkConventionComponents": 0
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
    "files": 31,
    "languages": {
      "vue": 26,
      "svelte": 4,
      "astro": 1
    },
    "reason": "sfc-extractor-not-registered"
  }
}
```

This is the right result. The current MVP exposes bounded SFC evidence but does
not claim full SFC understanding.

## Sampled Evidence

`symbols.json.sfcTemplateComponentRefs[]` contained 28 review-only records:

| Status | Count | Reason |
| ------ | ----- | ------ |
| `external` | 14 | `sfc-template-component-external-binding` |
| `muted` | 14 | `sfc-template-component-non-source-binding` |

Representative samples:

| Tag | Status | Reason | Consumer |
| --- | ------ | ------ | -------- |
| `VPDocAsideSponsors` | `external` | `sfc-template-component-external-binding` | `docs/.vitepress/theme/components/AsideSponsors.vue` |
| `Icon` | `external` | `sfc-template-component-external-binding` | `docs/.vitepress/theme/landing/Community.vue` |
| `RiveAnimation` | `external` | `sfc-template-component-external-binding` | `docs/.vitepress/theme/landing/FeatureGrid1.vue` |
| `Footer` | `external` | `sfc-template-component-external-binding` | `docs/.vitepress/theme/landing/Layout.vue` |
| `HeadingSection` | `external` | `sfc-template-component-external-binding` | `docs/.vitepress/theme/landing/Layout.vue` |

All sampled review-only records kept `eligibleForFanIn: false` and
`eligibleForSafeFix: false`.

## Markdown And Action Surface Checks

`audit-summary.latest.md` rendered the count-only brief:

```text
SFC evidence: 32 records across script imports 4, template refs 28; 28 review-only records. Read `manifest.json.sfcEvidence` and SFC arrays in `symbols.json`; review-only SFC lanes are not fan-in or action-tier proof, and sfc-scan-gap still applies.
```

`audit-review-pack.latest.md` rendered the review-pack cue:

```text
SFC evidence review: inspect manifest.json.sfcEvidence and SFC arrays in symbols.json before treating SFC absence as deadness. script-imports=4; template-refs=28; review-only=28; sfc-scan-gap still applies.
```

Checks:

- 21 advisory names sampled from SFC review-only arrays had zero occurrences in
  `audit-summary.latest.md` and `audit-review-pack.latest.md`.
- `resolvedInternalEdges[]` had 3,547 entries, but zero SFC-specific graph
  edges.
- `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` had
  zero SFC-policy matches.

## Decision

Decision: `mixed-sfc-smoke-corpus-useful`,
`current-mvp-safe-on-vite-corpus`, `scan-gap-stays`, `no-action-surface`, and
`needs-vue-svelte-astro-corpus-set`.

The beta.78 MVP behaved correctly on this mixed Vite corpus: SFC evidence was
visible in counts and raw `symbols.json` arrays, default Markdown stayed
count-only, `sfc-scan-gap` remained present, and no SFC review-only evidence
entered graph, fan-in, deadness, or action lanes.

This is not enough to mark WT-SFC `DONE`. It is the first corpus pass for the
calibration plan. The next reports should cover a dedicated Vue app, a
dedicated Svelte app, and a dedicated Astro app.

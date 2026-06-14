# WT-SFC Astro Client Corpus Calibration - 2026-05-31

This report applies the
[`WT-SFC corpus calibration plan`](wt-sfc-corpus-calibration-plan-2026-05-31.md)
to an Astro corpus that explicitly exercises the Astro `client:*` framework
convention lane. It uses the beta.78 public package and does not change
analyzer behavior.

This run complements the
[`IMA2 Astro corpus calibration`](wt-sfc-ima2-astro-corpus-calibration-2026-05-31.md).
The IMA2 pass covered Astro frontmatter and template evidence, but it did not
exercise `client:*` directives. This pass closes that specific Astro
client-directive requirement while keeping WT-SFC at `MVP`, not `DONE`.

## Run

| Field | Value |
| ----- | ----- |
| Corpus | `astro-main/examples/with-nanostores` |
| Framework/language mix | Astro example app with imported client components |
| Root path class | local checkout under `C:/Users/endof/Downloads/astro-main/examples/with-nanostores` |
| Public package | `0.9.0-beta.78` from `C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab` |
| Command route | `node <public-package-clone>/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root <repo> --output <out> --profile full` |
| Output path | `C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-astro-with-nanostores-client-beta78` |
| Profile | `full` |
| Result | PASS, 22 artifacts produced |

Command:

```text
node C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root C:/Users/endof/Downloads/astro-main/examples/with-nanostores --output C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-astro-with-nanostores-client-beta78 --profile full
```

## Aggregate Metrics

| Metric | Value |
| ------ | ----- |
| `sfcFileCount` | 3 |
| `byLanguage` | `astro: 3` |
| `scriptImportConsumerCount` | 6 |
| `scriptSrcReachabilityCount` | 0 |
| `styleAssetReferenceCount` | 0 |
| `templateRefCount` | 5 |
| `globalRegistrationCount` | 0 |
| `generatedManifestCount` | 0 |
| `frameworkConventionCount` | 3 |
| `astroClientDirectiveCount` | 3 |
| `reviewOnlyEvidenceCount` | 8 |
| `totalSfcEvidenceCount` | 14 |
| `rawMarkdownLeakCount` | 0 of 6 sampled SFC advisory names |
| `actionLeakCount` | 0 SFC-policy matches in `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` |

`manifest.json.sfcEvidence` reported:

```json
{
  "status": "complete",
  "scriptImportConsumerCount": 6,
  "reachabilityOnlyCount": 0,
  "reviewOnlyEvidenceCount": 8,
  "totalEvidenceCount": 14,
  "byLane": {
    "scriptImportConsumers": 6,
    "scriptSrcReachability": 0,
    "styleAssetReferences": 0,
    "templateComponentRefs": 5,
    "globalComponentRegistrations": 0,
    "generatedComponentManifests": 0,
    "frameworkConventionComponents": 3
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
      "astro": 3
    },
    "reason": "sfc-extractor-not-registered"
  }
}
```

That is the correct MVP behavior. The analyzer records grounded Astro
frontmatter, template, and client-directive review evidence, but it still does
not claim full Astro semantics.

## Sampled Evidence

`symbols.json.sfcFrameworkConventionComponents[]` contained three Astro
`client:*` directive records. Each sampled record used
`source: "sfc-framework-astro-client-directive"`,
`status: "muted"`, `confidence: "framework-convention-observed"`,
`eligibleForFanIn: false`, and `eligibleForSafeFix: false`.

Representative records:

| Tag | Directive | Binding Source | Consumer |
| --- | --------- | -------------- | -------- |
| `CartFlyout` | `client:load` | `../components/CartFlyout` | `src/layouts/Layout.astro` |
| `CartFlyoutToggle` | `client:load` | `../components/CartFlyoutToggle` | `src/layouts/Layout.astro` |
| `AddToCartForm` | `client:load` | `../components/AddToCartForm` | `src/pages/index.astro` |

These records are useful review evidence: they show that Astro hydrated an
explicitly imported component. They are not fan-in, deadness, or action-tier
proof.

## Markdown And Action Surface Checks

`audit-summary.latest.md` rendered the count-only brief:

```text
SFC evidence: 14 records across script imports 6, template refs 5, framework conventions 3; 8 review-only records. Read `manifest.json.sfcEvidence` and SFC arrays in `symbols.json`; review-only SFC lanes are not fan-in or action-tier proof, and sfc-scan-gap still applies.
```

`audit-review-pack.latest.md` rendered the review-pack cue:

```text
SFC evidence review: inspect manifest.json.sfcEvidence and SFC arrays in symbols.json before treating SFC absence as deadness. script-imports=6; template-refs=5; framework-conventions=3; review-only=8; sfc-scan-gap still applies.
```

Checks:

- Six advisory names sampled from SFC review-only arrays had zero occurrences
  in `audit-summary.latest.md` and `audit-review-pack.latest.md`.
- `resolvedInternalEdges[]` had 12 entries, but zero SFC-specific graph edges.
- `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` had
  zero SFC-policy matches.

## Decision

Decision: `astro-client-directive-corpus-covered`,
`current-mvp-safe-on-astro-client-corpus`, `scan-gap-stays`, and
`no-action-surface`.

The beta.78 MVP behaved correctly on an Astro corpus that includes imported
components with `client:*` directives: framework convention evidence appeared
as muted review-only records, default Markdown stayed count-only,
`sfc-scan-gap` remained present, and no SFC review-only evidence entered graph,
fan-in, deadness, or action lanes.

This completes the Astro `client:*` requirement from the corpus calibration
plan. WT-SFC remains `MVP`, not `DONE`, until the Vue and Svelte corpus legs
are reviewed too.

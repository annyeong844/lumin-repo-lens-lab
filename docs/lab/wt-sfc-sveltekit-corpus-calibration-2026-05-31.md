# WT-SFC SvelteKit Corpus Calibration - 2026-05-31

This report applies the
[`WT-SFC corpus calibration plan`](wt-sfc-corpus-calibration-plan-2026-05-31.md)
to a SvelteKit corpus with many `.svelte` files and Svelte action directives.
It uses the beta.78 public package and does not change analyzer behavior.

The selected corpus is the upstream SvelteKit `test/apps/basics` application.
It is not a production product checkout, but it is a broad framework app with
hundreds of Svelte routes and realistic SvelteKit action usage. The run is
strong enough to cover the Svelte corpus leg, while also recording a local
action-wrapper gap for future policy work.

## Run

| Field | Value |
| ----- | ----- |
| Corpus | `kit-main/packages/kit/test/apps/basics` |
| Framework/language mix | SvelteKit app corpus |
| Root path class | local checkout under `C:/Users/endof/Downloads/kit-main/packages/kit/test/apps/basics` |
| Public package | `0.9.0-beta.78` from `C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab` |
| Command route | `node <public-package-clone>/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root <repo> --output <out> --profile full` |
| Output path | `C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-sveltekit-basics-beta78` |
| Profile | `full` |
| Result | PASS, 22 artifacts produced |

Command:

```text
node C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root C:/Users/endof/Downloads/kit-main/packages/kit/test/apps/basics --output C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-sveltekit-basics-beta78 --profile full
```

## Aggregate Metrics

| Metric | Value |
| ------ | ----- |
| `sfcFileCount` | 525 |
| `byLanguage` | `svelte: 525` |
| `scriptImportConsumerCount` | 11 |
| `scriptSrcReachabilityCount` | 0 |
| `styleAssetReferenceCount` | 1 |
| `templateRefCount` | 0 |
| `globalRegistrationCount` | 0 |
| `generatedManifestCount` | 0 |
| `frameworkConventionCount` | 17 |
| `svelteActionDirectiveCount` | 17 |
| `reviewOnlyEvidenceCount` | 18 |
| `totalSfcEvidenceCount` | 29 |
| `rawMarkdownLeakCount` | 0 SFC-specific advisory names; one generic `form` word hit came from non-SFC prose |
| `actionLeakCount` | 0 SFC-policy matches in `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json`; one `enhance` hit was a normal route filename, not SFC evidence |
| `falsePositiveCount` | 0 among sampled Svelte action and style evidence records |
| `missedUsefulEvidenceCount` | 2 local action wrappers not recorded (`enhanceWrapper`, `focusAndScroll`) |

`manifest.json.sfcEvidence` reported:

```json
{
  "status": "complete",
  "scriptImportConsumerCount": 11,
  "reachabilityOnlyCount": 0,
  "reviewOnlyEvidenceCount": 18,
  "totalEvidenceCount": 29,
  "byLane": {
    "scriptImportConsumers": 11,
    "scriptSrcReachability": 0,
    "styleAssetReferences": 1,
    "templateComponentRefs": 0,
    "globalComponentRegistrations": 0,
    "generatedComponentManifests": 0,
    "frameworkConventionComponents": 17
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
    "files": 525,
    "languages": {
      "svelte": 525
    },
    "reason": "sfc-extractor-not-registered"
  }
}
```

That is the correct MVP behavior. The analyzer records grounded Svelte script,
style, and action-directive review evidence, but it still does not claim full
Svelte compiler semantics.

## Sampled Evidence

`symbols.json.sfcFrameworkConventionComponents[]` contained 17 Svelte action
directive records. Every record used
`source: "sfc-framework-svelte-action-directive"`,
`status: "muted"`, `confidence: "framework-convention-observed"`,
`eligibleForFanIn: false`, and `eligibleForSafeFix: false`.

Representative records:

| Action | Tag | Binding Source | Consumer |
| ------ | --- | -------------- | -------- |
| `enhance` | `form` | `$app/forms` | `src/routes/actions/enhance/+page.svelte` |
| `enhance` | `form` | `$app/forms` | `src/routes/actions/redirect/+page.svelte` |
| `enhance` | `form` | `$app/forms` | `src/routes/cookies/enhanced/basic/+page.svelte` |

The action surface is useful review evidence: it shows SvelteKit form
enhancement directives that are explicitly bound through imports. It is not
fan-in, deadness, or action-tier proof.

The run also included one style asset reference:

| Source | Spec | Status | Resolved File |
| ------ | ---- | ------ | ------------- |
| `src/routes/asset-preload/prerendered/+page.svelte` | `../styles.css` | `resolved` | `src/routes/asset-preload/styles.css` |

## False-Positive Review

The sampled Svelte evidence records were reviewed for noisy or misleading
claims. No false-positive SFC evidence was found.

| Lane | Sample | Review Label | Result |
| ---- | ------ | ------------ | ------ |
| `framework-convention` | `use:enhance` bound from `$app/forms` on SvelteKit forms | `useful-review-evidence` | Correct muted action-directive evidence. |
| `style-asset` | `../styles.css` from `src/routes/asset-preload/prerendered/+page.svelte` | `useful-review-evidence` | Correct isolated style asset evidence. |

False-positive table:

| Evidence | Reason |
| -------- | ------ |
| None | Sampled Svelte action and style evidence matched visible source syntax and stayed review-only. |

## Missed Useful Evidence

Two `use:` directives were visible in source but not recorded:

| Action | File | Why It Was Missed |
| ------ | ---- | ----------------- |
| `enhanceWrapper` | `src/routes/actions/invalidate-all/+page.svelte` | Local wrapper function calls imported `enhance`, but the current lane records only direct imported action bindings. |
| `focusAndScroll` | `src/routes/use-action/focus-and-scroll/+page.svelte` | Local action function is declared in the same Svelte file, not imported. |

This is a useful calibration result. The current MVP is safe, but a future
Svelte action policy could add local action binding evidence as muted
review-only data.

## Markdown And Action Surface Checks

`audit-summary.latest.md` rendered the count-only brief:

```text
SFC evidence: 29 records across script imports 11, style assets 1, framework conventions 17; 18 review-only records. Read `manifest.json.sfcEvidence` and SFC arrays in `symbols.json`; review-only SFC lanes are not fan-in or action-tier proof, and sfc-scan-gap still applies.
```

`audit-review-pack.latest.md` rendered the review-pack cue:

```text
SFC evidence review: inspect manifest.json.sfcEvidence and SFC arrays in symbols.json before treating SFC absence as deadness. script-imports=11; style-assets=1; framework-conventions=17; review-only=18; sfc-scan-gap still applies.
```

Checks:

- SFC-specific advisory names had zero occurrences in
  `audit-summary.latest.md` and `audit-review-pack.latest.md`.
- The only raw-name Markdown hit was the generic word `form` in unrelated
  non-SFC prose.
- `resolvedInternalEdges[]` had 49 entries, but zero SFC-specific graph edges.
- `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` had
  zero SFC-policy matches.
- The only `enhance` matches in action artifacts were normal route filenames
  such as `src/routes/actions/enhance/+page.server.js`, not SFC action
  evidence.

## Decision

Decision: `sveltekit-corpus-covered-with-local-action-gap`,
`current-mvp-safe-on-svelte-corpus`, `scan-gap-stays`,
`no-action-surface`, and `consider-local-svelte-action-evidence`.

The beta.78 MVP behaved correctly on a broad SvelteKit corpus: imported
`use:enhance` directives appeared as muted review-only framework convention
records, style asset evidence was isolated, default Markdown stayed count-only,
`sfc-scan-gap` remained present, and no SFC review-only evidence entered graph,
fan-in, deadness, or action lanes.

This covers the Svelte corpus leg for the current MVP. WT-SFC remains `MVP`,
not `DONE`, until the Vue corpus leg is reviewed and the local Svelte action
gap is either accepted as an explicit limitation or addressed by a future
review-only lane.

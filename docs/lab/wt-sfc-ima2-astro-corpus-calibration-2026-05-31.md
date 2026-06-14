# WT-SFC IMA2 Astro Corpus Calibration - 2026-05-31

This report applies the
[`WT-SFC corpus calibration plan`](wt-sfc-corpus-calibration-plan-2026-05-31.md)
to the first dedicated Astro corpus. It uses the beta.78 public package and
does not change analyzer behavior.

This run is an Astro-only partial corpus pass. It does not satisfy the Astro
leg of the Vue/Svelte/Astro corpus gate because it has no observed
`client:*`/framework-convention evidence. A future Astro corpus must include at
least one explicitly imported component with a `client:*` directive.

## Run

| Field | Value |
| ----- | ----- |
| Corpus | `repo/ima2-gen-main` |
| Framework/language mix | Astro-only SFC corpus |
| Root path class | local checkout under `C:/Users/endof/Downloads/repo/ima2-gen-main` |
| Public package | `0.9.0-beta.78` from `C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab` |
| Command route | `node <public-package-clone>/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root <repo> --output <out> --profile full` |
| Output path | `C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-ima2-astro-beta78` |
| Profile | `full` |
| Result | PASS, 22 artifacts produced |

Command:

```text
node C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root C:/Users/endof/Downloads/repo/ima2-gen-main --output C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-ima2-astro-beta78 --profile full
```

## Aggregate Metrics

| Metric | Value |
| ------ | ----- |
| `sfcFileCount` | 19 |
| `byLanguage` | `astro: 19` |
| `scriptImportConsumerCount` | 26 |
| `scriptSrcReachabilityCount` | 0 |
| `styleAssetReferenceCount` | 0 |
| `templateRefCount` | 41 |
| `globalRegistrationCount` | 0 |
| `generatedManifestCount` | 0 |
| `frameworkConventionCount` | 0 |
| `reviewOnlyEvidenceCount` | 41 |
| `totalSfcEvidenceCount` | 67 |
| `rawMarkdownLeakCount` | 0 of 15 sampled SFC advisory names |
| `actionLeakCount` | 0 SFC-policy matches in `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` |

`manifest.json.sfcEvidence` reported:

```json
{
  "status": "complete",
  "scriptImportConsumerCount": 26,
  "reachabilityOnlyCount": 0,
  "reviewOnlyEvidenceCount": 41,
  "totalEvidenceCount": 67,
  "byLane": {
    "scriptImportConsumers": 26,
    "scriptSrcReachability": 0,
    "styleAssetReferences": 0,
    "templateComponentRefs": 41,
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
    "files": 19,
    "languages": {
      "astro": 19
    },
    "reason": "sfc-extractor-not-registered"
  }
}
```

That is the correct Astro MVP behavior: the analyzer records grounded script
and template evidence, but it still does not claim full Astro semantics.
It also means this corpus does not exercise the Astro `client:*` framework
convention lane.

## Sampled Evidence

`symbols.json.sfcTemplateComponentRefs[]` contained 41 review-only Astro
template records. All sampled records kept `eligibleForFanIn: false` and
`eligibleForSafeFix: false`.

Representative samples:

| Tag | Status | Reason | Resolved File | Consumer |
| --- | ------ | ------ | ------------- | -------- |
| `InlineCode` | `muted` | `sfc-template-component-non-source-binding` | `site/src/components/InlineCode.astro` | `site/src/components/FAQ.astro` |
| `InlineCode` | `muted` | `sfc-template-component-non-source-binding` | `site/src/components/InlineCode.astro` | `site/src/components/FAQPage.astro` |
| `LangToggle` | `muted` | `sfc-template-component-non-source-binding` | `site/src/components/LangToggle.astro` | `site/src/components/Header.astro` |
| `CodeCopy` | `muted` | `sfc-template-component-non-source-binding` | `site/src/components/CodeCopy.astro` | `site/src/components/Hero.astro` |
| `Screenshot` | `muted` | `sfc-template-component-non-source-binding` | `site/src/components/Screenshot.astro` | `site/src/components/Hero.astro` |

The repeated `muted` status is expected. Astro-to-Astro template targets
preserve `resolvedFile` for navigation, but they do not create named export
fan-in.

## Markdown And Action Surface Checks

`audit-summary.latest.md` rendered the count-only brief:

```text
SFC evidence: 67 records across script imports 26, template refs 41; 41 review-only records. Read `manifest.json.sfcEvidence` and SFC arrays in `symbols.json`; review-only SFC lanes are not fan-in or action-tier proof, and sfc-scan-gap still applies.
```

`audit-review-pack.latest.md` rendered the review-pack cue:

```text
SFC evidence review: inspect manifest.json.sfcEvidence and SFC arrays in symbols.json before treating SFC absence as deadness. script-imports=26; template-refs=41; review-only=41; sfc-scan-gap still applies.
```

Checks:

- 15 advisory names sampled from SFC review-only arrays had zero occurrences in
  `audit-summary.latest.md` and `audit-review-pack.latest.md`.
- `resolvedInternalEdges[]` had 1,274 entries, but zero SFC-specific graph
  edges.
- `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` had
  zero SFC-policy matches.

## Decision

Decision: `astro-corpus-partial-script-and-template-only`,
`current-mvp-safe-on-astro-corpus`, `scan-gap-stays`, `no-action-surface`, and
`needs-astro-client-directive-corpus`.

The beta.78 MVP behaved correctly on this dedicated Astro corpus: SFC evidence
was visible in counts and raw `symbols.json` arrays, Astro-to-Astro template
targets stayed muted with `resolvedFile`, default Markdown stayed count-only,
`sfc-scan-gap` remained present, and no SFC review-only evidence entered graph,
fan-in, deadness, or action lanes.

This does not complete the Astro leg. WT-SFC remains `MVP`, not `DONE`, until
the Astro `client:*` corpus requirement plus the Vue and Svelte corpus legs are
reviewed too.

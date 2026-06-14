# WT-SFC Beta.79 SvelteKit Local Action Regression

This report follows up the
[`WT-SFC SvelteKit corpus calibration`](wt-sfc-sveltekit-corpus-calibration-2026-05-31.md),
which found two useful local Svelte action directives that beta.78 did not
record. It reruns the same SvelteKit corpus with the beta.79 public package
after local `use:action` bindings were added to the review-only framework
convention lane.

The policy boundary remains the current
[`SFC support policy`](../spec/sfc-support-policy.md#acceptance-for-current-mvp):
local action evidence is advisory, muted, and not fan-in or action-tier proof.

## Run

| Field | Value |
| ----- | ----- |
| Corpus | `kit-main/packages/kit/test/apps/basics` |
| Framework/language mix | SvelteKit app corpus |
| SFC files | 525 `.svelte` files |
| Public package | `0.9.0-beta.79` from `C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab` |
| Public package commit | `380d3cd` |
| Public package CI | `26695427844`, Node 20 and Node 22 smoke jobs passed |
| Command route | `node <public-package-clone>/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root <repo> --output <out> --profile full` |
| Output path | `C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-sveltekit-basics-beta79` |
| Profile | `full` |
| Result | PASS, 23 artifacts produced |

Command:

```text
node C:/Users/endof/Downloads/lumin-repo-lens-lab-public/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root C:/Users/endof/Downloads/kit-main/packages/kit/test/apps/basics --output C:/Users/endof/AppData/Local/Temp/lumin-sfc-corpus-sveltekit-basics-beta79 --profile full
```

## Beta.78 To Beta.79 Comparison

| Metric | beta.78 | beta.79 | Delta | Result |
| ------ | ------: | ------: | ----: | ------ |
| `sfcFileCount` | 525 | 525 | 0 | Same corpus. |
| `scriptImportConsumerCount` | 11 | 11 | 0 | No script-import drift. |
| `styleAssetReferenceCount` | 1 | 1 | 0 | No style-asset drift. |
| `svelteActionDirectiveCount` | 17 | 19 | +2 | Local action evidence added. |
| `frameworkConventionCount` | 17 | 19 | +2 | Same delta as Svelte actions. |
| `reviewOnlyEvidenceCount` | 18 | 20 | +2 | Both new records stay review-only. |
| `totalSfcEvidenceCount` | 29 | 31 | +2 | Expected increase only. |
| `falsePositiveCount` | 0 | 0 | 0 | Sampled evidence stayed correct. |
| `missedUsefulEvidenceCount` | 2 | 0 | -2 | Both beta.78 misses were recovered. |
| `actionLeakCount` | 0 | 0 | 0 | No action surface regression. |

`manifest.json.sfcEvidence` reported:

```json
{
  "status": "complete",
  "scriptImportConsumerCount": 11,
  "reachabilityOnlyCount": 0,
  "reviewOnlyEvidenceCount": 20,
  "totalEvidenceCount": 31,
  "byLane": {
    "scriptImportConsumers": 11,
    "scriptSrcReachability": 0,
    "styleAssetReferences": 1,
    "templateComponentRefs": 0,
    "globalComponentRegistrations": 0,
    "generatedComponentManifests": 0,
    "frameworkConventionComponents": 19
  },
  "scanGapStillApplies": true
}
```

## Recovered Local Action Evidence

The two beta.78 missed-useful records are now present in
`symbols.json.sfcFrameworkConventionComponents[]`.

| Action | File | Tag | Binding Kind | Result |
| ------ | ---- | --- | ------------ | ------ |
| `enhanceWrapper` | `src/routes/actions/invalidate-all/+page.svelte` | `form` | `local-function` | Recovered as muted review-only evidence. |
| `focusAndScroll` | `src/routes/use-action/focus-and-scroll/+page.svelte` | `input` | `local-const-function` | Recovered as muted review-only evidence. |

Both records use:

- `source: "sfc-framework-svelte-action-directive"`;
- `confidence: "framework-convention-observed"`;
- `status: "muted"`;
- `reason: "sfc-framework-svelte-action-directive"`;
- `eligibleForFanIn: false`;
- `eligibleForSafeFix: false`;
- `bindingSource` and `fromSpec` equal to the owning Svelte file.

Representative evidence:

```json
{
  "consumerFile": "src/routes/actions/invalidate-all/+page.svelte",
  "tagName": "form",
  "directiveName": "use:enhanceWrapper",
  "actionName": "enhanceWrapper",
  "bindingSource": "src/routes/actions/invalidate-all/+page.svelte",
  "bindingKind": "local-function",
  "status": "muted",
  "reason": "sfc-framework-svelte-action-directive"
}
```

```json
{
  "consumerFile": "src/routes/use-action/focus-and-scroll/+page.svelte",
  "tagName": "input",
  "directiveName": "use:focusAndScroll",
  "actionName": "focusAndScroll",
  "bindingSource": "src/routes/use-action/focus-and-scroll/+page.svelte",
  "bindingKind": "local-const-function",
  "status": "muted",
  "reason": "sfc-framework-svelte-action-directive"
}
```

## Leak Checks

The regression run kept the review-only boundary intact:

- `symbols.json.resolvedInternalEdges[]` had 49 entries and zero SFC-prefixed
  `kind` or `source` values.
- `fix-plan.json`, `export-action-safety.json`, and `dead-classify.json` had
  zero `enhanceWrapper`, `focusAndScroll`, or
  `sfc-framework-svelte-action-directive` matches.
- `audit-summary.latest.md` and `audit-review-pack.latest.md` had zero
  `enhanceWrapper`, `focusAndScroll`, or
  `sfc-framework-svelte-action-directive` matches.
- The summary still said review-only SFC lanes are not fan-in or action-tier
  proof and that `sfc-scan-gap` still applies.

The rendered count-only summary changed as expected:

```text
SFC evidence: 31 records across script imports 11, style assets 1, framework conventions 19; 20 review-only records. Read `manifest.json.sfcEvidence` and SFC arrays in `symbols.json`; review-only SFC lanes are not fan-in or action-tier proof, and sfc-scan-gap still applies.
```

The review-pack line also stayed count-only:

```text
SFC evidence review: inspect manifest.json.sfcEvidence and SFC arrays in symbols.json before treating SFC absence as deadness. script-imports=11; style-assets=1; framework-conventions=19; review-only=20; sfc-scan-gap still applies.
```

## Blind Zone

The run still emitted one `sfc-scan-gap` blind zone:

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

That is still correct. Local action evidence closes a useful review gap, but it
does not model full Svelte compiler semantics.

## Source Guard

The source contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs). The
beta.79 package metadata and SARIF version sync are guarded by
[`tests/publish-public-plugin.test.mjs`](../../tests/publish-public-plugin.test.mjs)
and
[`tests/smoke-uncovered.test.mjs`](../../tests/smoke-uncovered.test.mjs).

## Decision

Decision: `sveltekit-corpus-fully-covered-with-local-action-evidence`,
`local-svelte-action-gap-closed`, `scan-gap-stays`, `no-action-surface`, and
`mvp-now-extended-to-local-bindings`.

The beta.79 public package closes the beta.78 SvelteKit local-action gap:
`missedUsefulEvidenceCount` fell from 2 to 0 while `falsePositiveCount` and
`actionLeakCount` stayed at 0. WT-SFC remains `MVP`, not `DONE`, because
`sfc-scan-gap` and the broader framework/compiler gaps still apply.

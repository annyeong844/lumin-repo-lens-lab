# WT-SFC Corpus Calibration Plan - 2026-05-31

## Purpose

WT-SFC now has a bounded MVP evidence surface, recorded in
[`wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md`](wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md)
and governed by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md). The next step is not
turning `sfc-scan-gap` off. That would be the wrong move.

The next step is corpus calibration: run the current public package on real
Vue, Svelte, and Astro repositories, record which SFC lanes produce useful
evidence or noise, and use that evidence before promoting review-only lanes
into stronger claims.

This worksheet does not change analyzer behavior and does not mark WT-SFC
`DONE`.

## Current Baseline

- beta.63 verified inline Vue/Svelte script and Astro frontmatter imports.
- beta.64 verified literal Vue/Svelte `<script src>` reachability.
- beta.65 verified style asset references.
- beta.67 verified template component reference evidence and SFC-to-SFC
  target navigation.
- beta.68 and beta.77 verified explicit global/app/plugin registration
  evidence.
- beta.71 through beta.76 verified generated/framework convention evidence for
  Nuxt, unplugin, Astro `client:*`, Svelte `use:action`, Vue macros, and Vue
  Options API registration.
- beta.78 verified count-only SFC audit brief summaries.
- The current checkout contains no `.vue`, `.svelte`, or `.astro` files, so
  self-dogfood cannot measure positive SFC evidence.

## Corpus Set

Minimum calibration requires three corpora:

| Corpus Type | Purpose | Required Shape |
| ----------- | ------- | -------------- |
| Vue app | Measure Vue script imports, template refs, global registration, Nuxt/unplugin conventions, macros, and Options API evidence. | Multiple `.vue` files with local imports, app registration, and at least one framework/convention surface. |
| Svelte app | Measure Svelte script imports, style assets, `use:action`, and remaining compiler-owned gaps. | Multiple `.svelte` files with imported actions/components and at least one store or runtime-only shape that should remain a gap. |
| Astro app | Measure Astro frontmatter imports, `client:*` directives, style assets, and integration gaps. | Multiple `.astro` files with imported components and at least one island/client directive. |

Recommended optional corpora:

| Corpus Type | Purpose |
| ----------- | ------- |
| Mixed monorepo | Confirm SFC evidence stays package-scoped and does not create cross-package absence claims. |
| Generated-heavy app | Confirm `.nuxt`, generated declarations, virtual routes, and build artifacts stay review-only or unavailable. |
| Design-system repo | Measure noise from many globally available components and custom elements. |

If a corpus is unavailable, record the missing corpus and reason instead of
substituting a fixture. Fixtures prove mechanics; they do not prove corpus
usefulness.

## Run Shape

Use the installed public package whenever possible. Use the maintainer checkout
only when a public install is unavailable, and label that run as
maintainer-only.

Command skeleton:

```text
lumin-repo-lens-lab --root <repo> --output <out> --profile full
node <public-package-clone>/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --root <repo> --output <out> --profile full
```

Use the first form when the installed public package exposes the
`lumin-repo-lens-lab` bin on `PATH`. Use the second form when verifying from a
checked-out or cached public package clone. Do not run `node audit-repo.mjs`
from the corpus repository root unless that file actually exists there; that
would test the corpus checkout, not the installed package entrypoint.

For each corpus:

1. Record package version, command route, root path class, output path class,
   Node version, and whether the run used the public install.
2. Keep `manifest.json`, `symbols.json`, `blind-zones.json`,
   `audit-summary.latest.md`, `audit-review-pack.latest.md`, `fix-plan.json`,
   `export-action-safety.json`, and SARIF if emitted.
3. Record whether `manifest.json.sfcEvidence` is `null`, `complete`, or
   partial.
4. Record whether `sfc-scan-gap` appears and whether its file/language counts
   match the corpus shape.
5. Sample rendered Markdown for count-only wording and raw SFC name leakage.

## Metrics To Record

For each corpus, record:

| Metric | Meaning |
| ------ | ------- |
| `sfcFileCount` | Total `.vue`, `.svelte`, and `.astro` files observed by triage. |
| `byLanguage` | SFC counts by language. |
| `scriptImportConsumerCount` | Static SFC script/frontmatter import consumers. |
| `scriptSrcReachabilityCount` | Literal `<script src>` reachability records. |
| `styleAssetReferenceCount` | Style `url()` and `@import` asset records. |
| `templateRefCount` | Template component reference evidence records. |
| `globalRegistrationCount` | Explicit global/app/plugin registration records. |
| `generatedManifestCount` | Generated component manifest records. |
| `frameworkConventionCount` | Framework convention evidence records. |
| `reviewOnlyEvidenceCount` | SFC evidence records that must not feed fan-in/action lanes. |
| `rawMarkdownLeakCount` | Raw component/tag/directive/API names in default Markdown. |
| `actionLeakCount` | SFC evidence in `SAFE_FIX`, `EXISTS`, fix-plan, export-action, package edits, or SARIF findings. |
| `falsePositiveCount` | Human-reviewed SFC evidence records that should not have been emitted. |
| `missedUsefulEvidenceCount` | Human-reviewed useful SFC facts absent from the current surface. |

For each sampled SFC evidence record, record:

| Field | Required Value |
| ----- | -------------- |
| `lane` | `script-import`, `script-src`, `style-asset`, `template-ref`, `global-registration`, `generated-manifest`, or `framework-convention`. |
| `sourceFile` | The SFC or config/manifest file that produced evidence. |
| `status` | `resolved`, `muted`, `unresolved`, `skipped`, or lane-specific equivalent. |
| `reason` | Stable reason code, if present. |
| `resolvedFile` | Target path when the lane preserves navigation evidence. |
| `eligibleForFanIn` | Must be `false` for review-only lanes. |
| `eligibleForSafeFix` | Must be `false` for review-only lanes. |
| `reviewLabel` | `useful-review-evidence`, `noisy-but-harmless`, `false-positive`, or `missing-important-evidence`. |
| `reviewNote` | One concise sentence explaining the label. |

## Decision Gates

The corpus result may support keeping the current MVP as-is when:

- review-only lanes are useful for navigation but do not create action proof;
- default Markdown exposes counts and pointers only;
- `sfc-scan-gap` remains present for SFC corpora;
- raw SFC record payloads stay in `symbols.json`, not summary/review-pack
  prose;
- graph edges, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan,
  export-action, package edits, and SARIF findings stay clean.

The result must block stronger absence claims when:

- a generated manifest is missing but framework convention files exist;
- custom resolvers or compiler transforms provide component availability that
  the analyzer cannot observe;
- template semantics require runtime or compiler evaluation;
- SFC evidence is useful only as review navigation, not as consumption proof.

The result must block default action wording when:

- reviewers cannot tell whether a record means availability, reachability,
  or actual template use;
- a lane produces false positives from custom elements, global components,
  namespace tags, or generated files;
- any sampled SFC record would need wording stronger than review-only
  inspection.

## Output Shape

The calibration report should include:

```text
corpus name
framework/language mix
package version or maintainer commit
command route
artifact paths
aggregate metrics table
sampled SFC evidence table
false-positive table
missed-useful-evidence table
raw Markdown/action leakage checks
decision
next action
```

Allowed decisions:

| Decision | Meaning |
| -------- | ------- |
| `keep-current-mvp` | Current WT-SFC surface is useful and safe as-is. |
| `tighten-lane-policy` | A lane emits noisy or misleading review evidence. |
| `add-corpus-before-next-lane` | Corpus set is insufficient or unbalanced. |
| `custom-resolver-still-gap` | Framework/custom resolver semantics remain unmodeled. |
| `scan-gap-stays` | `sfc-scan-gap` remains required. |
| `no-action-surface` | SFC evidence remains out of action lanes. |

## Non-Goals

- Do not remove `sfc-scan-gap` from this plan.
- Do not promote review-only SFC evidence into fan-in, deadness, `SAFE_FIX`,
  `EXISTS`, fix-plan, export-action, package edits, or SARIF findings.
- Do not treat generated manifest absence as proof that a component is unused.
- Do not infer custom resolver or compiler/runtime behavior without a
  lane-specific spec.
- Do not count a synthetic fixture as corpus calibration.

## Verdict

WT-SFC should stay `MVP`, not `DONE`, until at least one Vue app, one Svelte
app, and one Astro app are reviewed with this worksheet. The next
implementation-affecting SFC PR should cite that report before changing
absence claims, default Markdown wording, action lanes, or broad framework
semantics.

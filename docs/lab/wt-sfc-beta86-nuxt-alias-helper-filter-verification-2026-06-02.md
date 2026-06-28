# WT-SFC Beta.86 Nuxt `#components` Helper Filter Verification

This report verifies the beta.86 public package after the Nuxt
`#components` alias helper-export filter landed. It follows the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration),
the
[`Nuxt app-dir/custom resolver inventory`](wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md),
and the beta.85
[`Nuxt main corpus calibration`](wt-sfc-nuxt-main-corpus-calibration-2026-06-01.md).

## Run

| Field | Value |
| ----- | ----- |
| Public package | `0.9.0-beta.86` |
| Source PRs | [#599 helper filter](https://github.com/annyeong844/lumin_lab/pull/599), [#600 beta.86 metadata](https://github.com/annyeong844/lumin_lab/pull/600) |
| Public package commit | [`1de657a`](https://github.com/annyeong844/lumin-repo-lens-lab/commit/1de657a6b11ab320375ce2b3c94f0a6075cb1338) |
| Public package CI | [26762576211](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26762576211), Public Package CI passed on Node 20 and Node 22 |
| Installed cache | `C:/Users/endof/.claude/plugins/cache/annyeong844-marketplace/lumin-repo-lens-lab/0.9.0-beta.86/` |
| Audit target | `C:/Users/endof/Downloads/nuxt-main` |
| Command route | Installed public `/lumin-repo-lens-lab:audit` command |
| Output | `.lumin-verify-beta86/` quick audit; `.lumin-verify-beta86-ci/` CI/SARIF audit |
| Result | PASS |

## Checkpoints

| # | Checkpoint | Result | Evidence |
| - | ---------- | ------ | -------- |
| 1 | Installed package is `0.9.0-beta.86` | PASS | Cache folder exists at `0.9.0-beta.86`; installed `.claude-plugin/plugin.json` and `skills/lumin-repo-lens-lab/package.json` both report beta.86. |
| 2 | Public package CI passed | PASS | Public Package CI run `26762576211` completed successfully for `1de657a` on Node 20 and Node 22. |
| 3 | `componentNames` is filtered from SFC alias evidence | PASS | `symbols.json.sfcFrameworkConventionComponents[]` has zero `componentNames` entries. |
| 4 | `componentNames` does not leak into SFC import lanes | PASS | `dependencyImportConsumers[]`, `unresolvedInternalSpecifierRecords[]`, and `resolvedInternalEdges[]` each have zero `componentNames` hits. |
| 5 | `componentNames` does not leak into action or Markdown outputs | PASS | `fix-plan.json`, `dead-classify.json`, `export-action-safety.json`, `checklist-facts.json`, `module-reachability.json`, SARIF, and `audit-summary.latest.md` each have zero `componentNames` hits. |
| 6 | Component-like `#components` alias evidence remains | PASS | `sfcFrameworkConventionComponents[]` has 9 records; 7 are `nuxt-components-alias-import` records from `#components`, all `status: "unresolved"` with `reason: "sfc-framework-nuxt-components-alias-unresolved"`. |
| 7 | SFC scan-gap still applies | PASS | `manifest.sfcEvidence.scanGapStillApplies: true`; `manifest.blindZones[]` includes `area: "sfc-scan-gap"` for 315 Vue SFC files. |
| 8 | SARIF version is beta.86 | PASS | `.lumin-verify-beta86-ci/lumin-repo-lens-lab.sarif` has `tool.driver.name: "lumin-repo-lens-lab"` and `tool.driver.version: "0.9.0-beta.86"`. |

## SFC Evidence Summary

The beta.86 run produced this `manifest.sfcEvidence` summary:

```json
{
  "status": "complete",
  "scriptImportConsumerCount": 17,
  "reviewOnlyEvidenceCount": 33,
  "totalEvidenceCount": 50,
  "byLane": {
    "scriptImportConsumers": 17,
    "templateComponentRefs": 24,
    "frameworkConventionComponents": 9
  },
  "scanGapStillApplies": true
}
```

The beta.85 Nuxt corpus calibration found one false-positive
`componentNames` alias record. Beta.86 removes that record, so the total SFC
evidence count drops from 51 to 50 while the useful component-like alias
records remain.

## Helper Filter Sweep

The verification swept the SFC evidence, import, action, SARIF, and Markdown
surfaces for `componentNames`:

| Surface | `componentNames` hits |
| ------- | --------------------- |
| `symbols.json.sfcFrameworkConventionComponents[]` | 0 |
| `symbols.json.unresolvedInternalSpecifierRecords[]` | 0 |
| `symbols.json.dependencyImportConsumers[]` | 0 |
| `symbols.json.resolvedInternalEdges[]` | 0 |
| `fix-plan.json` | 0 |
| `dead-classify.json` | 0 |
| `export-action-safety.json` | 0 |
| `checklist-facts.json` | 0 |
| `module-reachability.json` | 0 |
| `lumin-repo-lens-lab.sarif` | 0 |
| `audit-summary.latest.md` | 0 |

Full-text `symbols.json` still contains unrelated TypeScript symbol identities
such as `componentNamesTemplate` and `resolveComponentNameSegments`. Those are
ordinary source symbols in `nuxt-main`; they are not Nuxt `#components` alias
evidence and do not touch SFC advisory, import, or action lanes.

## Alias Evidence

The Nuxt corpus has no generated `.nuxt/components.d.ts` manifest in the
checked output, so every retained `#components` alias record stays bounded as
unresolved review-only evidence:

```json
{
  "conventionKind": "nuxt-components-alias-import",
  "fromSpec": "#components",
  "source": "sfc-framework-nuxt-components-alias",
  "status": "unresolved",
  "reason": "sfc-framework-nuxt-components-alias-unresolved",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

The manifest-backed branch remains guarded by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs), where
`NuxtManifest` still expects `status: "muted"`, `resolvedFile`, and
`reason: "sfc-framework-nuxt-components-alias-manifest"`.

## `#components` Resolver Boundary

The SFC `.vue` alias path no longer leaks `#components` into unresolved import
diagnostics. The remaining `#components` unresolved records are outside the SFC
convention lane:

| Surface | `#components` hits | Decision |
| ------- | ------------------ | -------- |
| `dependencyImportConsumers[]` | 0 | Clean |
| `resolvedInternalEdges[]` | 0 | Clean |
| `unresolvedInternalSpecifierRecords[]` | 24 | Non-SFC `.ts` resolver diagnostics with `hash-imports-unsupported`; not framework evidence leakage |
| `fix-plan.json` | 2 | Metadata summaries of the same non-SFC unresolved specifiers |
| `dead-classify.json`, `export-action-safety.json`, SARIF, Markdown | 0 | Clean |

## Decision

Decision: `beta86-public-verified`,
`componentNames-helper-filter-public-verified`,
`component-like-alias-evidence-preserved`, `scan-gap-stays`, and
`no-action-surface`.

Beta.86 closes the beta.85 Nuxt corpus false positive: `componentNames` from
`#components` is no longer emitted as component alias evidence, while
component-like `#components` imports remain review-only evidence and graph,
fan-in, deadness, action, SARIF, and default Markdown boundaries stay clean.

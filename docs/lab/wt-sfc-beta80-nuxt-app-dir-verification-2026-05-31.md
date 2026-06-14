# WT-SFC Beta.80 Nuxt App-Dir Convention Verification

This report verifies the beta.80 public package after the Nuxt app-dir
convention lane was added and then narrowed to an explicit app source-dir
signal. It follows the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration)
and the
[`Nuxt app-dir/custom resolver inventory`](wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md).

## Run

| Field | Value |
| ----- | ----- |
| Public package | `0.9.0-beta.80` |
| Public package commit | `19be3b6` |
| Public package CI | `26712278901`, Public Package CI passed |
| Source PRs | #582 inventory, #583 implementation, #584 beta.80 metadata |
| Command route | Installed package `skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --profile full` |
| Fixtures | `Temp/lumin-beta80-nuxt4-appdir/`, `Temp/lumin-beta80-nuxt3-deponly/` |
| Result | PASS |

## Checkpoints

| # | Checkpoint | Result | Evidence |
| - | ---------- | ------ | -------- |
| 1 | Installed package is `0.9.0-beta.80` | PASS | Marketplace clone `HEAD` matched public commit `19be3b6`; plugin, marketplace, skill package, and skill lockfile versions reported beta.80. |
| 2 | Public package CI passed | PASS | Public Package CI run `26712278901` completed successfully. |
| 3 | Nuxt 4 / app `srcDir` emits app-dir convention evidence | PASS | Fixture A used `nuxt: "^4.0.0"`, `srcDir: "app"`, and `app/components/AppDirCard.vue`; `symbols.json.sfcFrameworkConventionComponents[]` recorded one muted `sfc-framework-nuxt-app-dir-convention` entry. |
| 4 | Nuxt 3 dependency-only projects do not emit app-dir evidence | PASS | Fixture B used `nuxt: "^3.13.0"` with no `srcDir` config and `app/components/AppDirCard.vue`; `sfcFrameworkConventionComponents[]` stayed empty. |
| 5 | App-dir evidence remains review-only | PASS | The emitted record had `status: "muted"`, `confidence: "framework-convention-observed"`, `eligibleForFanIn: false`, and `eligibleForSafeFix: false`. |
| 6 | Graph/action lanes stay clean | PASS | `resolvedInternalEdges[]` had no Nuxt app-dir kind/source entries; no fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF, or package-edit lane was affected. |
| 7 | Markdown remains count-only | PASS | The summary surfaced only SFC evidence counts and the review-only/sfc-scan-gap warning; raw `AppDirCard` names did not leak into Markdown. |

## Fixture A: Nuxt 4 App Dir

The Nuxt 4/app-src fixture produced the expected app-dir record:

```json
{
  "framework": "nuxt",
  "conventionKind": "nuxt-app-components-directory",
  "componentName": "AppDirCard",
  "normalizedTagNames": ["AppDirCard", "app-dir-card"],
  "sourceFile": "app/components/AppDirCard.vue",
  "resolvedFile": "app/components/AppDirCard.vue",
  "source": "sfc-framework-nuxt-app-dir-convention",
  "reason": "sfc-framework-nuxt-app-dir-convention",
  "confidence": "framework-convention-observed",
  "status": "muted",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false,
  "componentPathSegments": ["AppDirCard"]
}
```

`manifest.json.sfcEvidence` reported one review-only framework convention
record, and the audit summary repeated that review-only SFC lanes are not
fan-in or action-tier proof while `sfc-scan-gap` still applies.

## Fixture B: Nuxt 3 Dependency Only

The Nuxt 3 dependency-only fixture used the same `app/components/AppDirCard.vue`
shape without a Nuxt 4 dependency or explicit `srcDir: "app"` config.
`sfcFrameworkConventionComponents[]` stayed empty and `manifest.sfcEvidence`
stayed `null`.

This is the important beta.80 regression guard: a generic Nuxt 3 dependency is
not enough to claim the Nuxt 4 app-dir component convention.

## Decision

Decision: `nuxt-app-dir-public-verified`,
`nuxt-app-dir-requires-app-src-signal`,
`nuxt3-dependency-only-does-not-emit-app-dir`, `scan-gap-stays`, and
`no-action-surface`.

Beta.80 keeps Nuxt app-dir evidence useful but bounded: Nuxt 4 or explicit
`srcDir: "app"` can produce muted navigation evidence, while Nuxt 3
dependency-only projects do not. The lane remains advisory and does not enter
graph, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF,
or package-edit surfaces.

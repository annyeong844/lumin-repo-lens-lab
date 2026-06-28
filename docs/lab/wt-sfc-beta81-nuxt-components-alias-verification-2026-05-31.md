# WT-SFC Beta.81 Nuxt `#components` Alias Verification

This report verifies the beta.81 public package after the Nuxt `#components`
alias evidence lane landed. It follows the
[`SFC support policy`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration)
and the
[`Nuxt app-dir/custom resolver inventory`](wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md).

## Run

| Field | Value |
| ----- | ----- |
| Public package | `0.9.0-beta.81` |
| Source PRs | #586 implementation, #587 beta.81 metadata |
| Public package commit | `82d57db` |
| Public package CI | `26714357264`, Public Package CI passed on Node 20 and Node 22 |
| Command route | Installed package `skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --profile full` |
| Fixture | `Temp/lumin-beta81-nuxt-hash-components/` |
| Result | PASS |

## Checkpoints

| # | Checkpoint | Result | Evidence |
| - | ---------- | ------ | -------- |
| 1 | Installed package is `0.9.0-beta.81` | PASS | Marketplace clone `HEAD` matched public commit `82d57db`; plugin, marketplace, skill package, and skill lockfile versions reported beta.81. |
| 2 | Public package CI passed | PASS | Public Package CI run `26714357264` completed successfully on Node 20 and Node 22. |
| 3 | Manifest-backed `#components` alias emits review-only evidence | PASS | `KnownCard` was backed by `.nuxt/components.d.ts` and produced a muted `sfc-framework-nuxt-components-alias` record with `resolvedFile: "components/KnownCard.vue"`. |
| 4 | Unmapped `#components` alias emits unresolved review-only evidence | PASS | `UnknownCard` produced `reason: "sfc-framework-nuxt-components-alias-unresolved"`, `status: "unresolved"`, and no `resolvedFile`. |
| 5 | Alias imports do not leak into import/dependency lanes | PASS | `dependencyImportConsumers[]`, `unresolvedInternalSpecifierRecords[]`, and `resolvedInternalEdges[]` had zero `#components` / Nuxt alias entries. |
| 6 | Action and deadness lanes stay clean | PASS | Fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF, and package-edit lanes had zero Nuxt alias evidence. |
| 7 | SFC scan-gap and Markdown boundaries remain intact | PASS | `sfc-scan-gap` remained present for the fixture SFCs; default Markdown stayed count-only with zero raw `KnownCard`, `UnknownCard`, or `#components` mentions. |

## Manifest-Backed Alias

The fixture used `.nuxt/components.d.ts` to define `KnownCard`:

```ts
declare module 'vue' {
  export interface GlobalComponents {
    KnownCard: typeof import('../components/KnownCard.vue')['default']
  }
}
export {};
```

The installed package recorded the expected advisory entry:

```json
{
  "framework": "nuxt",
  "conventionKind": "nuxt-components-alias-import",
  "consumerFile": "components/Consumer.vue",
  "componentName": "KnownCard",
  "manifestFile": ".nuxt/components.d.ts",
  "manifestKind": "nuxt-components-dts",
  "resolvedFile": "components/KnownCard.vue",
  "bindingName": "KnownCard",
  "bindingSource": "../components/KnownCard.vue",
  "fromSpec": "#components",
  "source": "sfc-framework-nuxt-components-alias",
  "confidence": "generated-manifest-availability",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false,
  "status": "muted",
  "reason": "sfc-framework-nuxt-components-alias-manifest",
  "importedName": "KnownCard",
  "sfcBlockKind": "vue-script-setup",
  "line": 8
}
```

## Unmapped Alias

The same fixture imported `UnknownCard` from `#components` without a generated
manifest mapping. The installed package recorded a bounded unresolved advisory:

```json
{
  "framework": "nuxt",
  "conventionKind": "nuxt-components-alias-import",
  "consumerFile": "components/Consumer.vue",
  "componentName": "UnknownCard",
  "bindingName": "UnknownCard",
  "fromSpec": "#components",
  "source": "sfc-framework-nuxt-components-alias",
  "confidence": "framework-convention-observed",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false,
  "status": "unresolved",
  "reason": "sfc-framework-nuxt-components-alias-unresolved",
  "importedName": "UnknownCard",
  "sfcBlockKind": "vue-script-setup",
  "line": 8
}
```

No `resolvedFile` is emitted for the unmapped alias. That is the important
boundary: the producer records that a Nuxt alias import was observed, but it
does not guess a target.

## Lane Isolation

The verification checked the import and action surfaces directly:

| Lane | `#components` / Nuxt alias hits |
| ---- | ------------------------------- |
| `dependencyImportConsumers[]` | 0 |
| `unresolvedInternalSpecifierRecords[]` | 0 |
| `resolvedInternalEdges[]` | 0 |
| `fanInByIdentity` | 0 |
| `fix-plan.json` | 0 |
| `export-action-safety.json` | 0 |
| `dead-classify.json` | 0 |
| Markdown summaries | 0 raw component or alias names |

The emitted records stay in
`symbols.json.sfcFrameworkConventionComponents[]` with
`eligibleForFanIn: false` and `eligibleForSafeFix: false`.

## Decision

Decision: `nuxt-components-alias-public-verified`,
`generated-manifest-backed-alias-stays-muted`,
`unmapped-alias-stays-unresolved`, `scan-gap-stays`, and
`no-action-surface`.

Beta.81 safely surfaces Nuxt `#components` alias evidence as review-only SFC
framework convention records. Manifest-backed aliases keep navigation details;
unmapped aliases stay unresolved; neither path enters dependency, unresolved
internal, graph, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan,
export-action, SARIF, package-edit, or raw Markdown surfaces.

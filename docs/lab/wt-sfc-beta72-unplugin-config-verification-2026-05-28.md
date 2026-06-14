# WT-SFC Beta.72 Unplugin Config Verification

This note records the beta.72 public-install verification for the WT-SFC
`unplugin-vue-components` config evidence lane described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).

The source fix landed in
[`lumin-audit` PR #550](https://github.com/annyeong844/lumin_lab/pull/550).
The public package landed in
[`lumin-repo-lens-lab` PR #6](https://github.com/annyeong844/lumin-repo-lens-lab/pull/6)
at public main
[`022e83e`](https://github.com/annyeong844/lumin-repo-lens-lab/commit/022e83e200c8e0c99ce41fde9a3f89bb91c59100).
Public Package CI passed for the updated beta.72 branch:
[`annyeong844/lumin-repo-lens-lab/actions/runs/26524997546`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26524997546).

The source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

## Result

PASS. The installed beta.72 package records
`unplugin-vue-components` Vite/Webpack config usage in
`symbols.sfcFrameworkConventionComponents[]` as muted review-only evidence. The
lane covers ESM imports, assigned CommonJS `require(...)` calls, direct
CommonJS `require(...)` calls, and binding-free inline plugin calls such as
`plugins: [require("unplugin-vue-components/vite")()]`.

| #   | Checkpoint                                 | Result | Evidence                                                                                                                                |
| --- | ------------------------------------------ | ------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.72`       | PASS   | `/plugin` updated the installed package; four version files reported beta.72.                                                           |
| 2   | Public package publish chain is grounded   | PASS   | Source PR #550 and public PR #6 are merged; public main `022e83e` matches the installed marketplace clone.                              |
| 3   | Existing config forms still work           | PASS   | ESM import, assigned `require`, and direct `require` fixtures still produce `sfc-framework-auto-import-plugin-config` records.          |
| 4   | Binding-free inline `require` is recorded  | PASS   | `plugins: [require("unplugin-vue-components/vite")()]` records `pluginName: "require"` and `fromSpec: "unplugin-vue-components/vite"`.  |
| 5   | Config records stay muted and review-only  | PASS   | Records use `status: "muted"`, `reason: "sfc-framework-auto-import-plugin-config"`, and `confidence: "framework-convention-observed"`.  |
| 6   | Graph, fan-in, and action lanes stay clean | PASS   | The config evidence does not enter `resolvedInternalEdges[]`, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, or export-action lanes. |
| 7   | `sfc-scan-gap` remains visible             | PASS   | The grouped SFC scan-gap blind zone remains wired and source/installed `blind-zones.mjs` copies are byte-identical.                     |

## Safety Notes

The `unplugin-vue-components` config lane is convention evidence, not generated
component-manifest evidence. Seeing the plugin call means the project may use
auto-registration, but this lane does not infer component names, component
targets, template usage, or named export consumption.

Beta.72 therefore keeps the lane isolated:

```json
{
  "source": "sfc-framework-auto-import-plugin-config",
  "confidence": "framework-convention-observed",
  "status": "muted",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

The binding-free inline `require(...)()` regression mattered because the
traversal already intended to recognize direct CommonJS plugin calls. Returning
early when there were no import or variable bindings silently skipped the common
`plugins: [require("unplugin-vue-components/vite")()]` shape. Beta.72 keeps
that call reachable without broadening the lane into resolver or component
target inference.

## Decision

Decision: `unplugin-config-public-verified`,
`inline-require-plugin-config-public-verified`, and
`framework-config-evidence-stays-review-only`.

WT-SFC remains `MVP`, not `DONE`: SFC script imports, script-source
reachability, style assets, explicit template component bindings, explicit
global component registrations, generated component manifests, Nuxt filesystem
conventions, and `unplugin-vue-components` config evidence now have grounded
review evidence. Other framework magic, custom resolvers, compiler/runtime
conventions, and stronger absence claims still require lane-specific contracts.

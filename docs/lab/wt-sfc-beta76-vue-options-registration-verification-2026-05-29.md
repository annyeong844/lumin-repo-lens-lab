# WT-SFC Beta.76 Vue Options API Registration Verification

This note records the beta.76 public-install verification for the WT-SFC Vue
Options API local component registration evidence lane described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).

The source implementation landed in
[`lumin-audit` PR #561](https://github.com/annyeong844/lumin_lab/pull/561).
The beta.76 metadata, changelog, and SARIF version bump landed in
[`lumin-audit` PR #562](https://github.com/annyeong844/lumin_lab/pull/562).
The public package landed on public main at
[`8995a4f`](https://github.com/annyeong844/lumin-repo-lens-lab/commit/8995a4f04229d696170b236efa34c221b1aa9dff).
Public Package CI passed for the public main push:
[`annyeong844/lumin-repo-lens-lab/actions/runs/26638211663`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26638211663).

The source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs),
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs),
[`audit-repo-blind-zones.test.mjs`](../../tests/audit-repo-blind-zones.test.mjs),
[`sarif-fix-plan.test.mjs`](../../tests/sarif-fix-plan.test.mjs),
[`smoke-uncovered.test.mjs`](../../tests/smoke-uncovered.test.mjs), and
[`publish-public-plugin.test.mjs`](../../tests/publish-public-plugin.test.mjs).

## Result

PASS. The installed beta.76 package records literal Vue ordinary `<script>`
`export default { components: { ... } }` registrations only when each component
name is backed by an explicit non-type import binding. The record stays in
`symbols.sfcFrameworkConventionComponents[]` as muted review-only evidence. The
lane does not treat Vue Options API registrations as module reachability,
template component consumption, named export fan-in, or action-lane evidence.

| #   | Checkpoint                                       | Result | Evidence                                                                                                                                  |
| --- | ------------------------------------------------ | ------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.76`             | PASS   | `plugin.json`, `marketplace.json`, skill `package.json`, and skill `package-lock.json` all reported beta.76.                              |
| 2   | Public package publish chain is grounded         | PASS   | Source PRs #561/#562 are merged; public main `8995a4f` exists and Public Package CI run `26638211663` passed.                             |
| 3   | Literal Options API `components` records         | PASS   | `OptionsCard` emitted a `sfc-framework-vue-options-registration` record in `sfcFrameworkConventionComponents[]`.                          |
| 4   | Records stay muted and review-only               | PASS   | Records use `status: "muted"`, `confidence: "framework-convention-observed"`, `eligibleForFanIn: false`, and `eligibleForSafeFix: false`. |
| 5   | Static alias keys remain supported               | PASS   | A string-literal component key produced review-only evidence with the explicit import source.                                             |
| 6   | Type-only imports do not create evidence         | PASS   | Declaration-level and specifier-level type imports are excluded before Options API registration matching.                                 |
| 7   | Dynamic, unbound, and comment shapes do not leak | PASS   | Computed keys, unbound identifiers, template text, and comment-only Options API examples produced no Vue Options-registration records.    |
| 8   | Graph, fan-in, deadness, and action lanes clean  | PASS   | No Options API evidence entered `resolvedInternalEdges[]`, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, or export-action lanes.      |
| 9   | SARIF tool version matches beta.76               | PASS   | Root and packaged `emit-sarif.mjs` both report `TOOL_VERSION` as `0.9.0-beta.76`; the installed producer matched the packaged copy.       |
| 10  | Focused source guards pass                       | PASS   | Five focused Vitest files passed with 35 tests, including SFC, blind-zone, SARIF, smoke, and public-package guards.                       |

## Safety Notes

Vue Options API component registration is availability evidence, not template
consumption evidence. The first safe contract is therefore narrow: if
`export default { components: { ... } }` contains a literal object member whose
value is an explicit non-type import binding, the analyzer records that the
binding appears in Vue Options API registration syntax. It does not infer
runtime component availability, template usage, or export consumption.

Beta.76 keeps the lane isolated:

```json
{
  "framework": "vue",
  "conventionKind": "options-registration",
  "optionName": "components",
  "source": "sfc-framework-vue-options-registration",
  "reason": "sfc-framework-vue-options-registration",
  "confidence": "framework-convention-observed",
  "status": "muted",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

The explicit non-type binding requirement is part of the contract. Computed
component keys, unbound identifiers, comments, template text, type-only
imports, runtime option composition, mixins, extends, and ungrounded framework
behavior stay out of this surface.

## Decision

Decision: `vue-options-registration-public-verified`,
`explicit-value-binding-required-for-vue-options-evidence`,
`sarif-version-sync-public-verified`, and
`vue-options-evidence-stays-review-only`.

WT-SFC remains `MVP`, not `DONE`: SFC script imports, script-source
reachability, style assets, explicit template component bindings, explicit
global component registrations, generated component manifests, Nuxt filesystem
conventions, `unplugin-vue-components` config evidence, Astro `client:*`
directive evidence, Svelte `use:action` directive evidence, Vue
`defineOptions({ components })` macro evidence, and Vue Options API component
registration evidence now have grounded review evidence. Other framework magic,
custom resolvers, compiler/runtime conventions, mixin/extends registration, and
stronger absence claims still require lane-specific contracts.

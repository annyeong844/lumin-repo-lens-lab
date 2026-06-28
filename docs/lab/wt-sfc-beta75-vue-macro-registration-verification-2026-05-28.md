# WT-SFC Beta.75 Vue Macro Registration Verification

This note records the beta.75 public-install verification for the WT-SFC Vue
`defineOptions({ components })` macro registration evidence lane described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).

The source implementation landed in
[`lumin-audit` PR #558](https://github.com/annyeong844/lumin_lab/pull/558).
The beta.75 metadata, changelog, and SARIF version bump landed in
[`lumin-audit` PR #559](https://github.com/annyeong844/lumin_lab/pull/559).
The public package landed on public main at
[`18589cc`](https://github.com/annyeong844/lumin-repo-lens-lab/commit/18589ccbe6ed53ec38a0d573e1fb7f426661e9f4).
Public Package CI passed for the public main push:
[`annyeong844/lumin-repo-lens-lab/actions/runs/26597965914`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26597965914).

The source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs),
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs),
[`audit-repo-blind-zones.test.mjs`](../../tests/audit-repo-blind-zones.test.mjs),
[`sarif-fix-plan.test.mjs`](../../tests/sarif-fix-plan.test.mjs),
[`smoke-uncovered.test.mjs`](../../tests/smoke-uncovered.test.mjs), and
[`publish-public-plugin.test.mjs`](../../tests/publish-public-plugin.test.mjs).

## Result

PASS. The installed beta.75 package records literal Vue `<script setup>`
`defineOptions({ components })` registrations only when each component name is
backed by an explicit import binding. The record stays in
`symbols.sfcFrameworkConventionComponents[]` as muted review-only evidence. The
lane does not treat Vue macro registrations as module reachability, template
component consumption, named export fan-in, or action-lane evidence.

| #   | Checkpoint                                       | Result | Evidence                                                                                                                                                   |
| --- | ------------------------------------------------ | ------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.75`             | PASS   | `plugin.json`, `marketplace.json`, skill `package.json`, and skill `package-lock.json` all reported beta.75.                                               |
| 2   | Public package publish chain is grounded         | PASS   | Source PRs #558/#559 are merged; public main `18589cc` exists and Public Package CI run `26597965914` passed on Node 20 and Node 22.                       |
| 3   | Literal `defineOptions({ components })` records  | PASS   | `MacroCard` emitted a `sfc-framework-vue-macro-registration` record in `sfcFrameworkConventionComponents[]`.                                               |
| 4   | Records stay muted and review-only               | PASS   | Records use `status: "muted"`, `confidence: "framework-convention-observed"`, `eligibleForFanIn: false`, and `eligibleForSafeFix: false`.                  |
| 5   | Static alias keys remain supported               | PASS   | A string-literal component key such as `"macro-alias": MacroAlias` produced review-only evidence with normalized tag names and the explicit import source. |
| 6   | Dynamic, unbound, and comment shapes do not leak | PASS   | Computed keys, unbound identifiers, template text, and comment-only `defineOptions` examples produced no Vue macro-registration records.                   |
| 7   | Graph, fan-in, deadness, and action lanes clean  | PASS   | No macro evidence entered `resolvedInternalEdges[]`, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, or export-action lanes.                             |
| 8   | SARIF tool version matches beta.75               | PASS   | Root and packaged `emit-sarif.mjs` both report `TOOL_VERSION` as `0.9.0-beta.75`; the installed producer matched the packaged copy.                        |
| 9   | Node and Vitest source guards pass               | PASS   | Five focused Vitest files passed with 35 tests, including SFC, blind-zone, SARIF, smoke, and public-package guards.                                        |

## Safety Notes

Vue compiler macros can expand into framework behavior that is not visible as a
plain import graph. The first safe contract is therefore narrow: if
`defineOptions({ components })` contains a literal object member whose value is
an explicit non-type import binding, the analyzer records that the binding
appears in Vue macro registration syntax. It does not infer macro expansion,
runtime component availability, template consumption, or export consumption.

Beta.75 keeps the lane isolated:

```json
{
  "framework": "vue",
  "conventionKind": "macro-registration",
  "macroName": "defineOptions",
  "source": "sfc-framework-vue-macro-registration",
  "reason": "sfc-framework-vue-macro-registration",
  "confidence": "framework-convention-observed",
  "status": "muted",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

The explicit binding requirement is part of the contract. Computed component
keys, unbound identifiers, comments, template text, other macros, compiler
runtime magic, and ungrounded framework behavior stay out of this surface.

## Decision

Decision: `vue-macro-registration-public-verified`,
`explicit-binding-required-for-vue-macro-evidence`,
`sarif-version-sync-public-verified`, and
`vue-macro-evidence-stays-review-only`.

WT-SFC remains `MVP`, not `DONE`: SFC script imports, script-source
reachability, style assets, explicit template component bindings, explicit
global component registrations, generated component manifests, Nuxt filesystem
conventions, `unplugin-vue-components` config evidence, Astro `client:*`
directive evidence, Svelte `use:action` directive evidence, and Vue
`defineOptions({ components })` macro evidence now have grounded review
evidence. Other framework magic, custom resolvers, compiler/runtime
conventions, and stronger absence claims still require lane-specific contracts.

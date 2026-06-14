# WT-SFC Beta.73 Astro Client Directive Verification

This note records the beta.73 public-install verification for the WT-SFC Astro
`client:*` directive evidence lane described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).

The source implementation landed in
[`lumin-audit` PR #552](https://github.com/annyeong844/lumin_lab/pull/552).
The beta.73 metadata bump landed in
[`lumin-audit` PR #553](https://github.com/annyeong844/lumin_lab/pull/553).
The public package landed in
[`lumin-repo-lens-lab` PR #7](https://github.com/annyeong844/lumin-repo-lens-lab/pull/7)
at public main
[`9ee4f04`](https://github.com/annyeong844/lumin-repo-lens-lab/commit/9ee4f04549d79b70cdeb5fa140f4e97e26c9bae6).
Public Package CI passed for the public main push:
[`annyeong844/lumin-repo-lens-lab/actions/runs/26576766750`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26576766750).

The source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

## Result

PASS. The installed beta.73 package records Astro `client:*` directives on
explicitly imported components in
`symbols.sfcFrameworkConventionComponents[]` as muted review-only evidence. The
lane does not treat hydration directives as module reachability, template
consumption, named export fan-in, or action-lane evidence.

| #   | Checkpoint                                      | Result | Evidence                                                                                                                                  |
| --- | ----------------------------------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.73`            | PASS   | `plugin.json`, `marketplace.json`, skill `package.json`, and skill `package-lock.json` all reported beta.73.                              |
| 2   | Public package publish chain is grounded        | PASS   | Source PRs #552/#553 and public PR #7 are merged; the installed marketplace clone matched public main `9ee4f04`.                          |
| 3   | Bound Astro `client:*` directive is recorded    | PASS   | `<UsedByAstro client:load />` emitted a `sfc-framework-astro-client-directive` record in `sfcFrameworkConventionComponents[]`.            |
| 4   | Records stay muted and review-only              | PASS   | Records use `status: "muted"`, `confidence: "framework-convention-observed"`, `eligibleForFanIn: false`, and `eligibleForSafeFix: false`. |
| 5   | Unbound and native tags do not leak             | PASS   | `<MissingAstroClient client:load />` and native `<div client:load>` produced no Astro client-directive convention records.                |
| 6   | Graph, fan-in, deadness, and action lanes clean | PASS   | No directive evidence entered `resolvedInternalEdges[]`, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, or export-action lanes.        |
| 7   | `sfc-scan-gap` remains visible                  | PASS   | The grouped SFC scan-gap blind zone remains wired; installed `sfc-consumers.mjs` and `blind-zones.mjs` matched the source/skill copies.   |
| 8   | Node and Vitest source guards pass              | PASS   | `test-sfc-consumers.mjs` and `sfc-consumers.test.mjs` passed together with 11 assertions for the checked SFC evidence paths.              |

## Safety Notes

Astro hydration directives are framework convention evidence. A directive such
as `client:load` can tell a reviewer that an explicitly imported component is
used in an Astro island-like position, but this lane deliberately does not
infer runtime hydration reachability, integration-injected components, or
strong named export consumption.

Beta.73 therefore keeps the lane isolated:

```json
{
  "framework": "astro",
  "conventionKind": "client-directive",
  "source": "sfc-framework-astro-client-directive",
  "reason": "sfc-framework-astro-client-directive",
  "confidence": "framework-convention-observed",
  "status": "muted",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

The binding requirement is part of the contract. The directive evidence is only
emitted when the tag resolves through the existing Astro frontmatter/import
binding model. Missing imports, native tags, integration-provided components,
and other ungrounded framework magic stay out of this surface.

## Decision

Decision: `astro-client-directive-public-verified`,
`explicit-binding-required-for-astro-client-evidence`, and
`hydration-directive-evidence-stays-review-only`.

WT-SFC remains `MVP`, not `DONE`: SFC script imports, script-source
reachability, style assets, explicit template component bindings, explicit
global component registrations, generated component manifests, Nuxt filesystem
conventions, `unplugin-vue-components` config evidence, and Astro `client:*`
directive evidence now have grounded review evidence. Other framework magic,
custom resolvers, compiler/runtime conventions, and stronger absence claims
still require lane-specific contracts.

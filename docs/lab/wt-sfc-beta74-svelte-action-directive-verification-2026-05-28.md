# WT-SFC Beta.74 Svelte Action Directive Verification

This note records the beta.74 public-install verification for the WT-SFC Svelte
`use:action` directive evidence lane described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).

The source implementation landed in
[`lumin-audit` PR #555](https://github.com/annyeong844/lumin_lab/pull/555).
The beta.74 metadata and SARIF version bump landed in
[`lumin-audit` PR #556](https://github.com/annyeong844/lumin_lab/pull/556).
The public package landed in
[`lumin-repo-lens-lab` PR #8](https://github.com/annyeong844/lumin-repo-lens-lab/pull/8)
at public main
[`eb4e560`](https://github.com/annyeong844/lumin-repo-lens-lab/commit/eb4e56052f11a6e97b6b66933c802f8e14f72983).
Public Package CI passed for the public main push:
[`annyeong844/lumin-repo-lens-lab/actions/runs/26590569991`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26590569991).

The source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs),
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs),
[`smoke-uncovered.test.mjs`](../../tests/smoke-uncovered.test.mjs), and
[`publish-public-plugin.test.mjs`](../../tests/publish-public-plugin.test.mjs).

## Result

PASS. The installed beta.74 package records Svelte `use:action` directives when
the action name resolves to an explicit Svelte script binding. The record stays
in `symbols.sfcFrameworkConventionComponents[]` as muted review-only evidence.
The lane does not treat Svelte action directives as module reachability,
template component consumption, named export fan-in, or action-lane evidence.

| #   | Checkpoint                                      | Result | Evidence                                                                                                                                  |
| --- | ----------------------------------------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.74`            | PASS   | `plugin.json`, `marketplace.json`, skill `package.json`, and skill `package-lock.json` all reported beta.74.                              |
| 2   | Public package publish chain is grounded        | PASS   | Source PRs #555/#556 and public PR #8 are merged; the installed marketplace clone matched public main `eb4e560`.                          |
| 3   | Bound Svelte `use:action` directive is recorded | PASS   | `<form use:enhance>` emitted a `sfc-framework-svelte-action-directive` record in `sfcFrameworkConventionComponents[]`.                    |
| 4   | Records stay muted and review-only              | PASS   | Records use `status: "muted"`, `confidence: "framework-convention-observed"`, `eligibleForFanIn: false`, and `eligibleForSafeFix: false`. |
| 5   | Unbound and comment actions do not leak         | PASS   | `<button use:missingAction>` and comment-only `use:commentAction` produced no Svelte action-directive convention records.                 |
| 6   | Graph, fan-in, deadness, and action lanes clean | PASS   | No directive evidence entered `resolvedInternalEdges[]`, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, or export-action lanes.        |
| 7   | SARIF tool version matches beta.74              | PASS   | Root and packaged `emit-sarif.mjs` both report `TOOL_VERSION` as `0.9.0-beta.74`; the installed producer matched the packaged copy.       |
| 8   | Node and Vitest source guards pass              | PASS   | Five focused Vitest files passed with 35 tests, including SFC, blind-zone, SARIF, smoke, and public-package guards.                       |

## Safety Notes

Svelte actions can run arbitrary framework/runtime behavior. The first safe
contract is therefore narrow: if a directive name such as `use:enhance` is
grounded in an explicit Svelte script binding, the analyzer records that the
binding participates in Svelte action syntax. It does not infer action effects,
DOM reachability, compiler reactivity, `$store` subscriptions, or export
consumption.

Beta.74 keeps the lane isolated:

```json
{
  "framework": "svelte",
  "conventionKind": "action-directive",
  "source": "sfc-framework-svelte-action-directive",
  "reason": "sfc-framework-svelte-action-directive",
  "confidence": "framework-convention-observed",
  "status": "muted",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

The binding requirement is part of the contract. Missing action bindings,
comment-only markup, compiler/runtime magic, and ungrounded framework behavior
stay out of this surface.

## Decision

Decision: `svelte-action-directive-public-verified`,
`explicit-binding-required-for-svelte-action-evidence`,
`sarif-version-sync-public-verified`, and
`svelte-action-evidence-stays-review-only`.

WT-SFC remains `MVP`, not `DONE`: SFC script imports, script-source
reachability, style assets, explicit template component bindings, explicit
global component registrations, generated component manifests, Nuxt filesystem
conventions, `unplugin-vue-components` config evidence, Astro `client:*`
directive evidence, and Svelte `use:action` directive evidence now have
grounded review evidence. Other framework magic, custom resolvers,
compiler/runtime conventions, and stronger absence claims still require
lane-specific contracts.

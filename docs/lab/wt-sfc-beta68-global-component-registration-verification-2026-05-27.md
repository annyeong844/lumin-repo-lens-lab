# WT-SFC Beta.68 Global Component Registration Verification

This note records the beta.68 public-install verification for the WT-SFC
`sfc-global-component-registration` lane described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p5-candidate-explicit-global-component-registration).

The verification used the installed public package from public main
`52be4b9`, not only source tests. Public Package CI passed for that commit:
[`annyeong844/lumin-repo-lens-lab/actions/runs/26458546860`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26458546860).
The source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

## Result

PASS. The installed beta.68 package records explicit Vue global component
registrations as isolated review evidence in
`symbols.sfcGlobalComponentRegistrations[]`. It recognizes `createApp(...)`,
`createSSRApp(...)`, and app-returning chains such as
`createApp(...).use(router)`, while excluding `createApp(...).mount(...)`.

| #   | Checkpoint                                      | Result | Evidence                                                                                                          |
| --- | ----------------------------------------------- | ------ | ----------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.68`            | PASS   | `plugin.json`, `marketplace.json`, skill `package.json`, and skill `package-lock.json` all reported beta.68.      |
| 2   | `app.component(...)` registrations are recorded | PASS   | `app.component` registration for `Card` appeared in `symbols.sfcGlobalComponentRegistrations[]`; `supports` true. |
| 3   | `createSSRApp(...)` is a receiver               | PASS   | `ssr.component` registration for `Panel` appeared in the registration surface.                                    |
| 4   | App-returning chains are receivers              | PASS   | `createApp(...).use(router)` produced an `app2.component` registration for `Hero`.                                |
| 5   | `mount()` chains are not receivers              | PASS   | `createApp(...).mount(...)` did not produce the `Nope` registration.                                              |
| 6   | SFC targets stay muted; source targets resolve  | PASS   | `.vue` targets stayed muted with `resolvedFile`; the `.ts` control target `Gadget` resolved to `src/gadget.ts`.   |
| 7   | Graph, fan-in, and action lanes stay clean      | PASS   | No registration evidence entered `resolvedInternalEdges[]`, fan-in, `SAFE_FIX`, `EXISTS`, or action lanes.        |
| 8   | `sfc-scan-gap` remains visible                  | PASS   | The grouped scan-gap blind zone remained visible for the SFC fixture.                                             |

## Safety Notes

Global component registration is availability evidence, not template
consumption. A registered component may be usable from templates, but the
registration does not prove that any template actually uses it.

Beta.68 therefore keeps the lane isolated:

```json
{
  "source": "sfc-global-component-registration",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

SFC targets are intentionally muted because SFC files are still outside the
source graph. The `resolvedFile` field is reviewer navigation evidence only.
It does not authorize graph reachability, named export fan-in, deadness,
`SAFE_FIX`, `EXISTS`, package edits, or default action lanes.

## Decision

Decision: `global-registration-public-verified` and
`registration-availability-not-template-consumption`.

WT-SFC remains `MVP`, not `DONE`: SFC script imports, script-source
reachability, style assets, explicit template component bindings, and explicit
global component registrations now have grounded lane-specific evidence, but
framework magic and convention-driven registration still require their own
contracts before they affect absence claims.

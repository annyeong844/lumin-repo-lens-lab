# WT-SFC Beta.71 Nuxt Convention Verification

This note records the beta.71 public-install verification for the WT-SFC Nuxt
filesystem convention evidence lane described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md#p6-candidate-framework-magic-and-convention-registration).

The verification used the installed public package from public main
`998b1cc`, not only source tests. Public Package CI passed for that commit:
[`annyeong844/lumin-repo-lens-lab/actions/runs/26517500561`](https://github.com/annyeong844/lumin-repo-lens-lab/actions/runs/26517500561).
The source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

## Result

PASS. The installed beta.71 package records Nuxt root `components/` filesystem
convention files in `symbols.sfcFrameworkConventionComponents[]` only when a
Nuxt signal is present. Records stay muted review evidence and do not enter
graph edges, named export fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, or
export-action lanes.

| #   | Checkpoint                                | Result | Evidence                                                                                                                    |
| --- | ----------------------------------------- | ------ | --------------------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.71`      | PASS   | `plugin.json`, `marketplace.json`, skill `package.json`, and skill `package-lock.json` all reported beta.71.                |
| 2   | Nuxt signal gates convention evidence     | PASS   | The Nuxt fixture emitted convention records; the no-Nuxt-signal fixture emitted zero records and `uses` stayed `0`.         |
| 3   | Nested path names are derived correctly   | PASS   | `components/base/Button.vue` produced `BaseButton`; `components/user/index.vue` produced `UserIndex`.                       |
| 4   | Convention records stay muted             | PASS   | Records used `status: "muted"`, `reason: "sfc-framework-nuxt-fs-convention"`, and `confidence: "framework-convention-observed"`. |
| 5   | Graph, fan-in, and action lanes stay clean | PASS   | No convention evidence entered `resolvedInternalEdges[]`, fan-in, deadness, `SAFE_FIX`, fix-plan, or export-action lanes.   |
| 6   | `sfc-scan-gap` remains visible            | PASS   | The grouped SFC scan-gap blind zone remained visible for the verification fixture.                                          |

## Safety Notes

Nuxt filesystem convention evidence is availability evidence, not template
consumption. A file under `components/` may be auto-registered by Nuxt, but this
lane does not prove that a template uses it and does not prove that any named
export is consumed.

Beta.71 therefore keeps the lane isolated:

```json
{
  "source": "sfc-framework-nuxt-fs-convention",
  "confidence": "framework-convention-observed",
  "status": "muted",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

The no-Nuxt-signal fixture is part of the contract. A plain Vue project with a
root `components/` directory must not receive Nuxt convention evidence just
because the directory name matches.

## Decision

Decision: `nuxt-convention-public-verified`,
`nuxt-signal-required-for-convention-evidence`, and
`path-derived-nuxt-names-public-verified`.

WT-SFC remains `MVP`, not `DONE`: SFC script imports, script-source
reachability, style assets, explicit template component bindings, explicit
global component registrations, generated component manifests, and the first
Nuxt filesystem convention lane now have grounded review evidence. Other
framework magic, custom resolvers, and compiler/runtime conventions still
require lane-specific contracts before they affect absence claims.

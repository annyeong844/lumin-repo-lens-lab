# WT-SFC Beta.64 Script Src Reachability Verification

This note records the beta.64 public-install verification for the WT-SFC
`sfc-script-src` reachability slice described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md).

The verification used the installed public package from public main
`77bc7e1e`, not only source tests. The fixture and output were written outside
the repo under temporary directories.

## Result

PASS. The installed beta.64 package records literal relative Vue/Svelte
`<script src>` references as source-file reachability while keeping named export
fan-in unchanged.

| #   | Checkpoint                                                     | Result | Evidence                                                                                                                                       |
| --- | -------------------------------------------------------------- | ------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.64`                           | PASS   | `plugin.json`, bundled and live `marketplace.json`, skill `package.json`, and skill `package-lock.json` all reported `0.9.0-beta.64`.          |
| 2   | Vue/Svelte literal relative `<script src>` creates src edges   | PASS   | Runtime `resolvedInternalEdges` included `App.vue -> external-logic.ts` and `Widget.svelte -> svelte-logic.ts` with `kind: "sfc-script-src"`.  |
| 3   | `uses.sfcScriptSrcReachability` is counted                     | PASS   | Runtime `symbols.uses.sfcScriptSrcReachability` reported `2`.                                                                                  |
| 4   | Script-sourced file named exports keep zero fan-in             | PASS   | The referenced `srcLogic` and `svelteSrcLogic` exports both kept fan-in `0` and remained in `deadProdList`.                                    |
| 5   | Package, URL, dynamic, and empty `src` create no concrete edge | PASS   | `some-package`, `cdn.example.com`, and `dynamicPath` were absent from concrete edges; the only script-source edges were the two relative ones. |
| 6   | Missing relative `src` is diagnostic-only                      | PASS   | `./does-not-exist.ts` appeared in unresolved diagnostics with reason `sfc-script-src-unresolved` and did not create an edge.                   |
| 7   | `sfc-scan-gap` remains visible                                 | PASS   | The grouped blind zone reported `severity: "scan-gap"` with four SFC files and language counts `{ vue: 3, svelte: 1 }`.                        |

## Safety Notes

The important distinction is reachability versus consumption:
`<script src="./x.ts">` can make `x.ts` reachable, but it does not import or
consume the named exports inside `x.ts`. Beta.64 keeps that distinction visible
by using `resolvedInternalEdges[].kind === "sfc-script-src"` and leaving
`fanInByIdentity` at zero for script-sourced exports unless a real import
consumes them.

The run also confirmed that package, URL, dynamic, empty, and missing script
sources do not become concrete import edges. Missing relative paths are
diagnostic-only, not fake resolved files.

The broader `sfc-scan-gap` remains intentional. Beta.64 still does not model
template component references, style assets, Vue/Svelte/Astro framework magic,
or full SFC semantics.

## Decision

Decision: `sfc-script-src-public-verified` and `reachability-not-consumption`.

WT-SFC is `MVP`, not `DONE`: script imports and script-source reachability are
grounded, but broader SFC semantics still need lane-specific specs, fixtures,
and corpus checks before they affect absence claims.

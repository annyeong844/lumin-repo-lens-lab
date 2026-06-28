# WT-SFC Beta.65 Style Asset Verification

This note records the beta.65 public-install verification for the WT-SFC
`sfc-style-assets` lane described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md).

The verification used the installed public package from the
`annyeong844/lumin-repo-lens-lab` beta.65 cache, not only source tests. Public
package PRs
[`annyeong844/lumin-repo-lens-lab#3`](https://github.com/annyeong844/lumin-repo-lens-lab/pull/3)
and
[`annyeong844/lumin-repo-lens-lab#5`](https://github.com/annyeong844/lumin-repo-lens-lab/pull/5)
published the beta.65 package and the packaged CSS-escape fix. The source
guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

## Result

PASS. The installed beta.65 package records literal relative SFC style
`url()` and `@import` references as isolated asset evidence while keeping JS/TS
graph edges, named export fan-in, and broader SFC scan-gap warnings unchanged.

| #   | Checkpoint                                             | Result | Evidence                                                                                                      |
| --- | ------------------------------------------------------ | ------ | ------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.65`                   | PASS   | `plugin.json`, `marketplace.json`, skill `package.json`, and skill `package-lock.json` all reported beta.65.  |
| 2   | Public install path executed the fixture               | PASS   | The run used the beta.65 cache `scripts/audit-repo.mjs`, not a source checkout.                               |
| 3   | Vue/Svelte/Astro style references are recorded         | PASS   | Runtime `symbols.sfcStyleAssetReferences[]` contained Vue `url`/`@import`, Svelte `url`, and Astro `@import`. |
| 4   | Resolved assets carry `status: "resolved"`             | PASS   | `logo.svg`, `theme.css`, and `my icon.svg` had `resolvedFile` values.                                         |
| 5   | Missing relative assets are diagnostic-only            | PASS   | `../assets/missing.svg` carried reason `sfc-style-asset-unresolved`.                                          |
| 6   | CSS escapes are decoded before resolution              | PASS   | `url(../assets/my\ icon.svg)` resolved to `src/assets/my icon.svg`.                                           |
| 7   | Style assets do not enter `resolvedInternalEdges[]`    | PASS   | Runtime graph edges contained no style-asset entries.                                                         |
| 8   | Style assets do not affect named export fan-in/deadness | PASS   | Runtime fan-in did not mention assets; asset evidence did not call source consumer or graph edge paths.       |
| 9   | Unsupported style forms do not leak                    | PASS   | Package, URL, dynamic, comment, and template-attribute forms were absent from concrete asset records.         |
| 10  | `sfc-scan-gap` remains visible                         | PASS   | The grouped scan-gap blind zone remained with language counts for Vue, Svelte, and Astro.                    |

## Safety Notes

The style-asset lane is grounded but isolated. It records resource references
for future asset hygiene, but it does not claim source reachability, named
export use, `SAFE_FIX`, `EXISTS`, package edits, or dead-export ranking.

CSS escape decoding is part of that grounded evidence contract: a valid CSS URL
such as `url(../assets/my\ icon.svg)` must resolve the same file that the
runtime stylesheet would request. Without that decoding, the lane would create
false unresolved diagnostics for valid asset paths.

The broader `sfc-scan-gap` remains intentional. Beta.65 still does not model
template component references, Vue/Svelte/Astro framework magic, or full SFC
semantics.

## Decision

Decision: `sfc-style-assets-public-verified` and
`asset-evidence-not-symbol-fan-in`.

WT-SFC remains `MVP`, not `DONE`: script imports, script-source reachability,
and style asset evidence are grounded, but template component references and
framework-specific SFC semantics remain parked until they get their own
contracts and public-install verification.

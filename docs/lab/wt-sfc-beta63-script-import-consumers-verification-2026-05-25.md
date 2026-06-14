# WT-SFC Beta.63 Script Import Consumers Verification

This note records the beta.63 public-install verification for the first SFC
script-import consumer slice described by
[`sfc-support-policy.md`](../spec/sfc-support-policy.md).

The verification used the installed public package, not only source tests. The
fixture and output were written outside the repo under temporary directories.

## Result

PASS. The installed beta.63 package recognizes static imports from supported SFC
script regions, keeps template-only text out of the graph, preserves the
repo-wide SFC scan-gap warning, and does not overclaim full SFC support.

| #   | Checkpoint                                            | Result | Evidence                                                                                                                                                                                      |
| --- | ----------------------------------------------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Installed version is `0.9.0-beta.63`                  | PASS   | Installed package metadata reported `0.9.0-beta.63`.                                                                                                                                          |
| 2   | Vue and Svelte inline script imports feed symbols     | PASS   | Runtime `resolvedInternalEdges` included `App.vue -> util.ts` and `Widget.svelte -> util.ts` edges.                                                                                           |
| 3   | Astro frontmatter imports feed symbols                | PASS   | Runtime `resolvedInternalEdges` included `Page.astro -> util.ts`.                                                                                                                             |
| 4   | Vue `<script lang="tsx">` preserves JSX/TSX imports   | PASS   | Runtime `resolvedInternalEdges` included `Fancy.vue -> util.ts`; the TSX parser path preserved the import instead of dropping the file.                                                       |
| 5   | Template fake imports and `<script src>` stay ignored | PASS   | Fake template/script-source specifiers were absent from edges and unresolved specifier lists.                                                                                                 |
| 6   | SFC consumers are counted in the intended lanes       | PASS   | External `vue` and `svelte` imports appeared in `dependencyImportConsumers` with source `sfc-script-import`; four resolved internal SFC script imports appeared in `uses.sfcScriptConsumers`. |
| 7   | `sfc-scan-gap` remains visible                        | PASS   | The grouped blind zone reported `severity: "scan-gap"` with four SFC files across Vue, Svelte, and Astro.                                                                                     |

## Safety Notes

The run also confirmed that exports consumed only through the SFC script-import
lane stayed alive in dead-export analysis. That is the intended P1 behavior.

The scan-gap remains intentional. Beta.63 does not model template component
references, `<script src>`, style assets, Vue/Svelte/Astro framework magic, or
non-script SFC semantics.

## Decision

Decision: `sfc-script-import-consumers-public-verified` and
`sfc-scan-gap-still-required`.

WT-SFC is `MVP`, not `DONE`: script import consumers are grounded, but broader
SFC semantics need lane-specific specs, fixtures, and corpus checks before they
affect absence claims.

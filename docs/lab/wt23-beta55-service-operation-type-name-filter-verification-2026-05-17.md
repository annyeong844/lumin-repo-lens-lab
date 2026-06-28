# WT-23 Beta 55 Service-Operation Type-Name Filter Verification

Public install verification for the WT-23 service-operation type-name filter.
This run verifies that TypeScript-only declaration names no longer render as
related service-operation review cues.

## Run Summary

| Field             | Value                                                                                         |
| ----------------- | --------------------------------------------------------------------------------------------- |
| Installed version | `0.9.0-beta.55`                                                                               |
| Engine route      | `/lumin-repo-lens-lab:pre-write` routing through the installed public package                     |
| Entry point       | `node <plugin-cache>/0.9.0-beta.55/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --pre-write` |
| Corpus            | `C:/Users/endof/Downloads/VNplayer-main`                                                      |
| Output path class | temporary path under `C:/tmp/lrl-beta55-typefilter-1101/`, removed after verification         |
| Related fix       | PR #415, commit `38abab6`, `Mute service type-name pre-write cues`                            |

The run used the same VNplayer corpus and structured intents as the beta.54
local-operation support-reason verification so the result can be compared
directly.

## Verification Matrix

| Checkpoint                                           | Result | Evidence                                                                                                                                                         |
| ---------------------------------------------------- | :----: | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Installed package is `0.9.0-beta.55`                 |  PASS  | `plugin.json.version = 0.9.0-beta.55`.                                                                                                                           |
| Type-name false positives leave service promotions   |  PASS  | `queryLibraryDoc` service promotions dropped from 2 to 0; `ListLibraryDocsOptions` and `ListLibraryOutlineOptions` no longer appear in promoted service entries. |
| Non-callable declarations stay muted with kind       |  PASS  | Service muted entries include `service-sibling-non-callable-definition` and carry `definitionKind`, including TypeScript declaration kinds.                       |
| Type-name service Markdown disappears                |  PASS  | Service-operation review lines dropped from 2 in beta.54 to 0 in beta.55.                                                                                        |
| Local-operation behavior does not regress            |  PASS  | Local-operation promoted entries stayed at 9, all with `local-operation-same-file-domain-overlap`; local-operation Markdown stayed at 9 lines and `unknown` at 0. |
| Cue-tier and lane safety remain intact               |  PASS  | Cue tally remained review-only: `{ AGENT_REVIEW_CUE: 24 }`; no `SAFE_CUE`, `EXISTS`, or `SAFE_FIX` cue was emitted.                                              |
| Muted evidence stays hidden from default Markdown    |  PASS  | Local and service mute reasons stayed in JSON evidence and did not render in default Markdown.                                                                    |
| Service and local operation lanes remain independent |  PASS  | `service-operation-sibling` lane disappeared for this corpus while `local-operation-sibling` remained at 9; cross-feed identities stayed at 0.                    |

## Beta 54 To Beta 55 Delta

| Surface                         | beta.54                                                                    | beta.55                                             |
| ------------------------------- | -------------------------------------------------------------------------- | --------------------------------------------------- |
| Cue-tier tally                  | `{ AGENT_REVIEW_CUE: 26 }`                                                 | `{ AGENT_REVIEW_CUE: 24 }`                          |
| Evidence lane tally             | `{ intent-token: 15, local-operation-sibling: 9, service-operation: 2 }`    | `{ intent-token: 15, local-operation-sibling: 9 }`   |
| Service-operation Markdown cues | 2, both type-name options declarations                                     | 0                                                   |
| Local-operation promoted cues   | 9, all review-only                                                         | 9, all review-only                                  |
| SAFE cue leakage                | 0                                                                          | 0                                                   |
| Muted Markdown leakage          | 0                                                                          | 0                                                   |

## Scope Note

Type names such as `ListLibraryDocsOptions` can still appear in weaker semantic
hint surfaces such as `lookup.semanticHints[]` and the `Search hints (not reuse
candidates)` Markdown section. That lane is intentionally weaker than the
service-operation sibling lane: it is an intent-token hint, not a grounded
reuse or related-operation claim. If that weaker surface becomes too noisy, it
needs a separate slice.

## Decision

Decision: `type-name-filter-public-verified`.

The service-operation type-name filter is verified in the public install. The
VNplayer regression that promoted TypeScript-only option names as related
service operations is closed, local-operation review cues stayed stable, muted
details stayed hidden from default Markdown, and no safe-action cue leaked.

# WT-09 Block Clone Fixture Inventory - 2026-05-24

## Decision

Decision: `fixtures-before-implementation`.

Block clone work must not start by widening `function-clones.json`. P1 must
first add edge-case tests for a separate review-only `block-clones.json`
surface, then implement the smallest artifact producer that satisfies those
tests.

## Source Contract

This inventory refines
[`docs/spec/block-clone-detection.md`](../spec/block-clone-detection.md). The
spec already fixes the artifact shape, suffix-array/LCP direction,
normalization policy, threshold policy, and review-only boundary.

The implementation must preserve these hard boundaries:

- `function-clones.json` remains top-level function/helper evidence.
- `block-clones.json` is the only place for repeated token/block regions.
- P1 emits no Markdown, SAFE cue, EXISTS cue, fix-plan, or
  export-action-safety entry.
- Empty `groups[]` is proof only when `status: "complete"`.
- Thresholds in the artifact include `minTokens`, `minLines`,
  `minOccurrences`, `maxInstancesPerGroup`, `maxGroups`, and
  `maxTokensPerFile`.

## Fallow Anti-Pattern Notes

`C:/Users/endof/Downloads/fallow-main` has the useful algorithmic spine:
tokenization, normalization, suffix array, LCP, interval extraction, and
filtering. The part Lumin should not copy is the product contract: repeated
regions must not be collapsed into stronger refactor or reuse claims before
Lumin has corpus evidence.

The Lumin version should therefore be algorithmic in collection, conservative
in presentation.

## P1 Fixture Matrix

| ID  | Fixture                                                                            | Required Positive Evidence                                                                                                  | Required Negative Guard                                                                                              |
| --- | ---------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| BC1 | Two nested `if/for` blocks in different functions, same structure, renamed locals. | One `block-clones.json.groups[]` entry with two instances, `normalizationMode: "alpha-identifier"`, and `reviewOnly: true`. | No new `function-clones.json.exactBodyGroups`, `structureGroups`, `signatureGroups`, or near-function candidate.     |
| BC2 | A top-level exact function clone plus a separate nested repeated block.            | Function pair remains in `function-clones.json`; nested block appears in `block-clones.json`.                               | The block clone must not be mirrored into any function-clone lane.                                                   |
| BC3 | Repeated import declaration blocks.                                                | Either skipped by a named import policy or emitted with an explicit import-policy reason.                                   | Import boilerplate must not silently dominate top groups.                                                            |
| BC4 | Same-file overlapping repeated ranges.                                             | Overlap filtering keeps occurrence count honest.                                                                            | Overlapping windows in one file must not satisfy `minOccurrences` by themselves.                                     |
| BC5 | A repeated token sequence adjacent to a file boundary.                             | Sentinels prevent a cross-file group.                                                                                       | No instance span may cross from one file into another.                                                               |
| BC6 | A smaller repeated range fully contained in a longer repeated range.               | Longer maximal group survives.                                                                                              | Contained subset group is removed unless a future policy names why it stays.                                         |
| BC7 | Generated or bundled file with an otherwise obvious repeated block.                | File appears in `skipped[]` or makes artifact `confidence-limited`.                                                         | Artifact must not claim `complete` while silently ignoring policy-skipped files.                                     |
| BC8 | Missing or unsupported tokenizer input.                                            | Artifact status is `unavailable` or `confidence-limited` with a reason.                                                     | Empty `groups[]` must not be interpreted as proof of no clone evidence.                                              |
| BC9 | Repeated block detected in a normal full-profile run.                              | `block-clones.json` is produced as an artifact.                                                                             | No package edit, SARIF warning, `SAFE_FIX`, `SAFE_CUE`, `EXISTS`, pre-write cue, or Markdown line is produced in P1. |

## Test Shape

P1 should add both Node and Vitest coverage before behavior is considered done:

- Node suite: `tests/test-build-block-clone-index.mjs`.
- Vitest mirror: `tests/build-block-clone-index.test.mjs`.
- Producer smoke should run against a temp repo and read `block-clones.json`.
- Review-only leakage checks should grep/read `fix-plan.json`,
  `export-action-safety.json`, and pre-write/advisory outputs only when those
  artifacts exist in the selected run.

The tests should fail because behavior is wrong or absent, not because a helper
name is missing. Keep fixture assertions focused on artifact contracts and
leakage boundaries.

## Implementation Readiness

P1 is ready to start when the implementation PR can name which BC fixtures are
covered by its first commit. A partial P1 is acceptable only if the artifact is
`unavailable` or `confidence-limited` for unsupported cases and the missing
fixture IDs are listed in the PR body.

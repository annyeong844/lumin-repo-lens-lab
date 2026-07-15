# Block Clone Detection

> **Role:** maintainer-facing design spec for future token/block-level clone
> detection that can find repeated code substrings beyond top-level function
> bodies.
> **Status:** MVP. P1 artifact-only implementation merged in
> [`PR #504`](https://github.com/annyeong844/lumin_lab/pull/504); P2 manifest
> mirroring is implemented in `_lib/audit-manifest.mjs` and beta.59 public
> install verification passed. The first beta.59 noise review found useful
> engine signal but heavy Node/Vitest mirror noise, so P3 review-pack wording,
> broader corpus calibration, and threshold/rendering decisions remain open.
> P4 cap/noise calibration now specifies independent review/muted output caps
> under `block-clone-threshold-policy-v2`.
> **Last updated:** 2026-07-15.

## Problem

`function-clones.json` is intentionally narrow. It records deterministic
review cues for exported top-level functions and helpers. That surface is good
at exact body, structure, signature, and near-function evidence, but it cannot
see repeated blocks inside functions, classes, factory closures, route handlers,
or test bodies unless those blocks happen to be the whole top-level function.

That is a real recall gap. A suffix-array/LCP detector can find repeated token
substrings after normalization, including clones where local variable names
changed. But dropping that kind of detector into `function-clones.json` would
be dumb: it would mix two different evidence contracts, break existing
pre-write expectations, and invite users to treat arbitrary repeated snippets
as semantic duplicates.

The fix is a separate block-clone surface with a stricter contract:

```text
function-clones.json = top-level function/helper clone cues
block-clones.json    = token/block repeated-region review evidence
```

## Algorithmic Basis

The detector uses a conventional token-clone pipeline:

- AST tokenization into normalized token streams;
- optional identifier/literal normalization modes;
- file concatenation with unique sentinels;
- suffix-array construction;
- LCP scan;
- maximal interval extraction;
- token and line subset filtering;
- min-token and min-line thresholds.

The implementation is Lumin-owned rather than copied from another detector.
Its product contract remains deliberately narrower than the algorithm: repeated
regions are review evidence with explicit policy versions, thresholds,
unsupported states, and "not semantic equivalence" wording.

## Goals

1. Keep `block-clones.json` as the artifact for repeated token/block regions.
2. Detect repeated source substrings, not only complete top-level functions.
3. Support identifier/literal normalization through named policies.
4. Preserve source spans, line ranges, token counts, and normalization evidence.
5. Keep the surface review-only in the first implementation slices.
6. Keep `function-clones.json` behavior and schema unchanged.
7. Record enough status/completeness metadata that absence is not overclaimed.

## Non-Goals

- Do not change `function-clones.json`.
- Do not feed block clones into `nearNames`, `semanticHints`, `EXISTS`,
  `SAFE_CUE`, `SAFE_FIX`, export ranking, or package edit surfaces.
- Do not claim semantic equivalence.
- Do not emit refactor instructions such as "extract function" in P1.
- Do not scan generated, bundled, or framework/resource files without an
  explicit scan-policy reason.
- Do not build a free-form regex/string clone detector.
- Do not treat an empty artifact as proof unless `status: "complete"`.

## Artifact Contract

P1 artifact:

```json
{
  "schemaVersion": "block-clones.v1",
  "policyVersion": "block-clone-review-policy-v1",
  "status": "complete",
  "generated": "2026-05-24T00:00:00.000Z",
  "root": "/repo",
  "scanRange": {
    "includeTests": true,
    "exclude": []
  },
  "normalization": {
    "policyId": "block-clone-normalization-v1",
    "mode": "alpha-identifier",
    "preservePropertyNames": true,
    "preserveImportSpecifiers": true,
    "literalPolicy": "classify"
  },
  "thresholds": {
    "policyId": "block-clone-threshold-policy-v2",
    "minTokens": 50,
    "minLines": 5,
    "minOccurrences": 2,
    "maxInstancesPerGroup": 20,
    "maxCandidateGroups": 1000,
    "maxReviewGroups": 100,
    "maxMutedGroups": 100,
    "maxTokensPerFile": 200000
  },
  "summary": {
    "fileCount": 42,
    "tokenCount": 120000,
    "groupCount": 3,
    "instanceCount": 7,
    "skippedFileCount": 4,
    "unavailableFileCount": 0
  },
  "groups": [
    {
      "id": "block-clone:sha256:...",
      "claim": "repeated normalized token region",
      "confidence": "heuristic-review",
      "tokenCount": 73,
      "lineCount": 12,
      "occurrenceCount": 2,
      "normalizationMode": "alpha-identifier",
      "reasons": ["suffix-array-lcp-repeat", "line-threshold-met"],
      "instances": [
        {
          "file": "src/a.ts",
          "startLine": 10,
          "endLine": 21,
          "startToken": 120,
          "endToken": 193,
          "container": {
            "kind": "function",
            "name": "loadUser"
          }
        }
      ],
      "reviewOnly": true,
      "eligibleForSafeFix": false
    }
  ],
  "skipped": [
    {
      "file": "dist/app.js",
      "reason": "bundled-build-artifact"
    }
  ]
}
```

Status values:

- `complete`: scanner/tokenizer ran for the selected corpus.
- `unavailable`: required parser/tokenizer inputs were missing or unsupported.
- `confidence-limited`: some files were skipped for policy reasons, but the
  artifact still contains partial review evidence.
- `not-run`: producer was intentionally not invoked.

An empty `groups[]` array is evidence only when `status: "complete"`. If
`status !== "complete"`, consumers must treat absence as unknown.

## Algorithm

P1 should implement this pipeline, not a bag of string heuristics:

```text
collect source files under scan policy
  -> parse/tokenize JS/TS source
  -> map source tokens to normalized token ids
  -> split generated/bundled/framework-resource files by policy
  -> concatenate token ids with unique file sentinels
  -> build suffix array
  -> build LCP array with sentinel boundaries
  -> extract maximal LCP intervals above thresholds
  -> map token intervals back to source spans and containers
  -> remove overlapping/subset groups
  -> rank candidates (tokenCount, occurrenceCount, id) and apply the internal
     maxCandidateGroups guard (record candidateCapSaturated)
  -> classify every surviving candidate through the noise policy
  -> split into review and muted lanes
  -> apply maxReviewGroups and maxMutedGroups independently
     (record reviewCapSaturated / mutedCapSaturated)
  -> if deprecated maxGroups was supplied, apply it as a final total output
     ceiling with review groups emitted before muted groups
  -> emit review groups before muted groups, deterministically
```

Required properties:

- Sentinels must prevent matches from crossing file boundaries.
- LCP extraction must produce maximal repeated regions, not every nested
  suffix match.
- Subset removal must remove groups fully contained in longer groups.
- Same-file overlapping instances must not inflate occurrence counts.
- Output ordering must be deterministic.
- Group ids must include normalization and threshold policy identity.
- Noise classification must run on the full ranked candidate set, before the
  review/muted output caps are applied.
- The review output cap and the muted output cap are independent: a muted group
  must never displace a review group from the artifact.
- Deprecated `maxGroups` compatibility must preserve the previous total output
  ceiling across review plus muted groups. It must not be mapped to
  `maxReviewGroups` in a way that allows `maxReviewGroups + maxMutedGroups` to
  exceed the caller's old total cap.
- The internal `maxCandidateGroups` guard is noise-agnostic and exists only to
  bound work on pathological repos; its saturation must be reported via
  `candidateCapSaturated`, not hidden.

### Performance Contract

- Production suffix-array construction uses Lumin's SA-IS implementation over
  the dense normalized token alphabet. Prefix-doubling remains test-only as an
  independent differential oracle.
- LCP identity uses interval coordinates, not materialized token-signature
  strings for every candidate.
- Containment checks use an indexed interval query and materialize stable group
  ids only for surviving candidates. Candidate count alone must not reintroduce
  pairwise group comparison into the hot path.
- These implementation choices must preserve the artifact contract exactly;
  performance work does not change thresholds, ranking, ids, or review policy.

## Normalization Policy

P1 must name its normalization policy instead of hiding behavior behind a
generic "semantic" mode.

Recommended P1:

- preserve keywords, operators, punctuation, and control-flow structure;
- normalize local binding names and references through alpha slots;
- preserve property names and method names by default;
- preserve import/export specifier strings by default;
- classify primitive literals as `STRING`, `NUMBER`, `BOOLEAN`, `NULL` unless
  a future policy preserves exact literal values;
- skip comments and whitespace;
- optionally skip import declarations behind a named threshold policy, not as a
  hidden default.

Identifier normalization must not collapse everything to one token when a
stable local binding map is available. Use coarse `$ID` only as a lower
confidence fallback and record it in `normalization.mode`.

## Boundary With `function-clones.json`

Hard invariants:

1. `block-clones.json` does not alter `function-clones.json`.
2. `function-clones.json` does not read `block-clones.json`.
3. Pre-write function signature and top-level function cues keep using
   `function-clones.json`.
4. Block clone groups do not enter `exactBodyGroups`, `structureGroups`,
   `signatureGroups`, or near-function candidates.
5. Block clone groups do not enter cue tiers in P1.
6. If a future renderer mentions block clones, it must use wording like
   "Review repeated code region", not "reuse", "safe", or "equivalent".

This separation is non-negotiable. Mixing the surfaces would make every
existing function-clone test less meaningful.

## Scan Policy

P1 should inherit the existing audit scan range:

- `--root`;
- `--include-tests` / production profile;
- explicit `--exclude`;
- generated artifact policy;
- framework/resource capability packs;
- bundled-build-artifact lanes.

Files skipped for generated/bundle/framework reasons should be recorded in
`skipped[]`. If a large class of files cannot be classified, use
`confidence-limited` rather than pretending the artifact is complete.

## Thresholds

Default thresholds (`block-clone-threshold-policy-v2`):

- `minTokens: 50`;
- `minLines: 5`;
- `minOccurrences: 2`;
- `maxInstancesPerGroup: 20`;
- `maxCandidateGroups: 1000` — internal, noise-agnostic performance guard on
  the ranked candidate set, applied before noise classification;
- `maxReviewGroups: 100` — output cap on `visibility: "review"` groups;
- `maxMutedGroups: 100` — output cap on `visibility: "muted"` groups;
- `maxTokensPerFile: 200000` before confidence-limited fallback.

`maxGroups` is retained only as a deprecated input compatibility cap. When
supplied, it preserves the old total output ceiling across review plus muted
groups. The producer must emit review groups first and then fill any remaining
legacy `maxGroups` slots with muted groups. It must not translate
`maxGroups: 20` into `maxReviewGroups: 20` plus the default
`maxMutedGroups: 100`, because that would unexpectedly allow 120 emitted
groups for a caller that requested a total cap of 20.

These are policy defaults, not universal truth. They must live behind
`block-clone-threshold-policy-v2` and appear in the artifact.

## Noise And Mute Policy

Before P3 renders block clone evidence in default Markdown, the artifact needs
a named noise policy. The policy must classify review groups without deleting
raw evidence.

Recommended first policy:

```json
{
  "noisePolicy": {
    "policyId": "block-clone-noise-policy-v1",
    "reviewGroupCount": 7,
    "mutedGroupCount": 93,
    "mutedByReason": {
      "node-vitest-mirror-pair": 58,
      "test-scaffold-repeat": 18,
      "same-file-repeat": 17
    },
    "candidateCapSaturated": false,
    "reviewCapSaturated": false,
    "mutedCapSaturated": false
  }
}
```

### Cap Allocation

Noise classification runs before the output caps. The producer then fills two
independent budgets:

- up to `maxReviewGroups` groups with `visibility: "review"`;
- up to `maxMutedGroups` groups with `visibility: "muted"`.

**Core invariant: a review group is never pushed out of the artifact by muted
noise.** Muted groups compete only for the muted budget; review groups compete
only for the review budget. The earlier single `maxGroups` cap, applied before
noise classification, is replaced because it let high-token mirror-pair and
test-scaffold groups consume the cap and truncate genuine review groups.

`summary.groupCount` remains the emitted group count: review groups plus muted
groups after both output caps are applied. Saturation is reported per lane:
`reviewCapSaturated`, `mutedCapSaturated`, and the internal
`candidateCapSaturated`.

If deprecated `maxGroups` is supplied, it overlays a final compatibility total
cap after review-first ordering. This keeps old callers bounded by their old
total artifact-size expectation while preserving the new invariant that muted
noise never displaces review groups.

Initial mute reasons:

- `node-vitest-mirror-pair`: the group is dominated by the reviewed Node test
  and Vitest mirror pair for the same suite.
- `test-scaffold-repeat`: all instances are test files and the repeated region
  is broad fixture/setup/assertion scaffolding.
- `same-file-repeat`: the group repeats inside one file and is more likely
  local fixture/assertion repetition than cross-file design debt.

The producer may keep muted groups in `groups[]` with a `visibility: "muted"`
field, or move them to `mutedGroups[]`, but it must keep the raw artifact
auditable. The manifest may mirror only shallow counts and reason totals.

Hard rules:

1. Muting a group is not deletion and not proof that the group is unimportant.
2. Muted groups never enter `function-clones.json`, pre-write cues, SAFE lanes,
   fix-plan entries, or package/edit surfaces.
3. If all emitted groups are muted, default Markdown still must not claim that
   there are no block clone review candidates.
4. The artifact and manifest must preserve per-lane saturation signals
   (`reviewCapSaturated`, `mutedCapSaturated`, `candidateCapSaturated`) before
   any default renderer summarizes the data.

## P1 Fixture Matrix

P1 started with edge-case tests, not "helper missing" tests.
[`PR #504`](https://github.com/annyeong844/lumin_lab/pull/504) covers the BC
matrix below in `tests/test-build-block-clone-index.mjs` and
`tests/build-block-clone-index.test.mjs`.

Required fixtures:

1. Two nested blocks inside different functions with identical structure and
   renamed variables are grouped.
2. A top-level exact function clone remains in `function-clones.json`, while a
   nested block clone appears only in `block-clones.json`.
3. A repeated import block is ignored or classified according to the named
   import policy.
4. Same-file overlapping repeats do not create fake occurrence counts.
5. A clone spanning a file boundary is impossible because sentinels stop LCP.
6. A smaller repeated range contained in a longer repeated range is removed.
7. Generated/bundled files are skipped or make the artifact
   `confidence-limited`, not complete.
8. Empty `groups[]` with non-complete status is not treated as proof.
9. The artifact never creates `SAFE_FIX`, `EXISTS`, `SAFE_CUE`, fix-plan, or
   export-action-safety entries.
10. Destructured local bindings are normalized as local alpha slots.
11. Object-pattern keys such as `{ id: userId }` are not treated as local
    bindings.
12. Review-group preservation under noise pressure: with `maxCandidateGroups`
    large and small output caps (`maxReviewGroups: 1`, `maxMutedGroups: 1`), a
    corpus of two high-`tokenCount` muted groups (one `same-file-repeat`, one
    test-scaffold mirror pair) plus one lower-`tokenCount` cross-file non-test
    review group must classify all three, and the review group must survive in
    `groups[]` with `visibility: "review"`. Under the old pre-noise
    `maxGroups` cap the review group would be truncated; the new policy keeps
    it. Assert review survival, per-lane saturation flags, and no `SAFE_FIX` /
    `EXISTS` / `SAFE_CUE` / fix-plan / export-action-safety leakage.

## Implementation Slices

### P0: Spec And Fixture Inventory

Document the artifact contract, normalization policy, and negative fixtures.
No runtime behavior.

### P1: Artifact Only

Add `_lib/block-clone-artifact.mjs` and `build-block-clone-index.mjs`.
Emit `block-clones.json` in full profile only. Do not render it in Markdown.

Status: implemented by
[`PR #504`](https://github.com/annyeong844/lumin_lab/pull/504). The slice
remains `MVP`, not `DONE`; beta.59 verified the installed artifact and manifest
mirror, but broader corpus calibration still needs to confirm noise and cap
behavior before stronger surfaces.

### P2: Manifest Mirror

Mirror only shallow counts, status, threshold policy, and artifact path in
`manifest.json`. Do not include raw source fragments in the manifest.

Status: implemented through `manifest.blockClones`. The mirror exposes
artifact/schema/policy/status, review-only status, normalization policy,
threshold policy/default values, and summary counts. It intentionally omits
`groups[]`, `instances[]`, and source spans.

Beta.59 public-install verification is recorded in
[`wt09-beta59-block-clone-manifest-verification-2026-05-24.md`](../lab/wt09-beta59-block-clone-manifest-verification-2026-05-24.md).
The run produced non-empty raw evidence (`groups[]` capped at 100 and 210
instances) while `manifest.blockClones` stayed metadata-only. The artifact
status was `confidence-limited` because three files were skipped.

### P2b: Noise Classification

Add `block-clone-noise-policy-v1` to the raw artifact and mirror only shallow
review/muted counts and reason totals in `manifest.blockClones`. Keep default
Markdown off in this slice.

The beta.59 noise review is the baseline corpus:
[`wt09-beta59-block-clone-noise-review-2026-05-24.md`](../lab/wt09-beta59-block-clone-noise-review-2026-05-24.md).

Status: implemented. `block-clones.json` groups now carry `visibility:
"review" | "muted"` plus `muteReason` for muted groups, and the artifact-level
`noisePolicy` records `reviewGroupCount`, `mutedGroupCount`, `mutedByReason`,
and `capSaturated`. `manifest.blockClones` mirrors only those shallow
navigation counts, not raw groups or source spans.

Beta.60 public-install verification is recorded in
[`wt09-beta60-block-clone-noise-policy-verification-2026-05-25.md`](../lab/wt09-beta60-block-clone-noise-policy-verification-2026-05-25.md).
The installed artifact emitted 7 review groups and 93 muted groups, kept clone
group ids and clone evidence fields out of Markdown/action lanes, and confirmed
that `manifest.blockClones` remains a shallow mirror.

### P3: Review Pack Wording

Add weak wording only:

```text
Block clone review: inspect block-clones.json for repeated normalized regions.
```

No package edits, no fix-plan, no SAFE lanes.

Status: not ready for default rendering. The beta.59 noise review in
[`wt09-beta59-block-clone-noise-review-2026-05-24.md`](../lab/wt09-beta59-block-clone-noise-review-2026-05-24.md)
found 58 of the capped 100 groups were Node/Vitest mirror pairs. Add a named
noise/mute policy before surfacing block clone wording in default Markdown.
P3 may proceed only after P2b keeps mirror-pair/test-scaffold groups out of the
default reader path while preserving raw artifact auditability.

### P4: Corpus Calibration

Run real repositories with known clone cases and record false-positive/noise
patterns before changing thresholds or rendering more detail.

The first self-dogfood review recorded `p3-default-markdown-not-ready`,
`needs-noise-policy`, `engine-signal-present`, and `cap-saturated`.

The beta.60 noise-policy verification recorded
`noise-policy-public-verified` and `p3-markdown-still-deferred`, and confirmed
that the implementation ranked and capped the top 100 raw groups before
applying the noise policy.

Resolution: cap/noise ordering moves to noise-classification-first with
independent review/muted output caps under `block-clone-threshold-policy-v2`
(see Cap Allocation). The internal `maxCandidateGroups` guard stays
noise-agnostic with a `candidateCapSaturated` diagnostic.

Status: public-install verified in beta.61. BC12c/BC12d now pin review-group
preservation under muted-noise pressure and the deprecated `maxGroups` total cap
compatibility contract in both Node and Vitest. The beta.61 verification at
[`wt09-beta61-block-clone-cap-noise-v2-verification-2026-05-25.md`](../lab/wt09-beta61-block-clone-cap-noise-v2-verification-2026-05-25.md)
confirmed review-group recovery from 7 to 49 in the self-dogfood corpus,
legacy `maxGroups` artifact/manifest preservation, and no Markdown/action-lane
leakage. Broader corpus reruns are still required before P3 Markdown wording is
reconsidered.

## Acceptance Gate

No implementation PR may be marked done until:

- Node and Vitest tests cover the P1 fixtures;
- `function-clones.json` existing tests are unchanged and passing;
- `block-clones.json` proves review-only behavior;
- generated/bundled skip behavior is visible;
- a small corpus run confirms output is readable and not noisy enough to hide
  the useful groups.

## Decision

Proceed with a separate block-clone artifact when work resumes. Do not retrofit
suffix-array/LCP detection into `function-clones.json`.

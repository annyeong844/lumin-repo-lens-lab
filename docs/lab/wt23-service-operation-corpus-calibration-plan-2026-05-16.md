# WT-23 Service-Operation Corpus Calibration Plan - 2026-05-16

## Purpose

WT-23 P2b made `serviceOperationSiblingPolicy.promoted[]` visible as a
review-only Markdown cue. The next step is not another threshold change. The
next step is corpus calibration: run the existing policy on real repositories,
record when the cue is useful or noisy, and use that evidence before adding
mutation families, signature weighting, or broader advisory wording changes.

This note is a calibration worksheet. It does not change analyzer behavior and
does not justify marking WT-23 `DONE` by itself.

## Current Baseline

- beta.47 verified suppressed near-name and semantic diagnostics.
- beta.48 verified the P1 `serviceOperationSiblingPolicy` JSON object.
- beta.50 verified the P2b Markdown renderer for promoted read/query siblings.
- The current public cue remains review-only:
  `Review related service operation`.
- Muted service-operation entries remain hidden by default.
- `nearNames[]`, `semanticHints[]`, `EXISTS`, `SAFE_FIX`, and `SAFE_CUE` remain
  unchanged.

## Corpus Set

Minimum calibration requires two corpora:

| Corpus Type                 | Purpose                                                                                                                | Required Shape                                                                                                       |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Service-heavy app           | Measure useful read/query sibling cues in ordinary feature code.                                                       | Multiple service or repository modules with verbs such as `fetch`, `get`, `search`, `list`, `lookup`, and `resolve`. |
| Library or noise-heavy repo | Measure false positives where exported helpers, public API, generated files, or generic domain nouns can look related. | Public or reusable modules with many helpers and package boundaries.                                                 |

Recommended optional corpora:

| Corpus Type                    | Purpose                                                                                        |
| ------------------------------ | ---------------------------------------------------------------------------------------------- |
| OO-heavy app                   | Confirm class-method lanes stay separate from service-operation sibling cues.                  |
| Generated/framework-heavy repo | Confirm generated, bundled, framework, scaffold, and vendor candidates stay muted or excluded. |

If a corpus is unavailable, record the missing corpus and reason instead of
silently substituting a smaller fixture.

## Run Shape

Use the installed public package when possible. Use the maintainer checkout only
when a public install is unavailable, and label that run as maintainer-only.

Command skeleton:

```text
node audit-repo.mjs --root <repo> --output <out> --profile full
node audit-repo.mjs --root <repo> --output <out> --pre-write --intent <intent.json>
```

Intent skeleton:

```json
{
  "names": ["searchUser"],
  "files": [],
  "dependencies": [],
  "shapes": [],
  "plannedTypeEscapes": []
}
```

For each corpus:

1. Produce or reuse grounded baseline artifacts in a stable output directory.
2. Run pre-write intents against that same output directory so evidence is not
   cold or misleading.
3. Save each `pre-write-advisory*.json` and the rendered Markdown, if present.
4. Record package/version, command route, root path class, output path class,
   and whether evidence availability was `available`, `missing`, or partial.

Intent examples should be concrete planned names, not generic probes:

```text
searchUser
lookupCustomer
listInvoices
createUser
deleteInvoice
formatTimestamp
```

Do not count an intent if the existing corpus has no plausible service or
repository surface near the intended domain.

## Metrics To Record

For each corpus, record:

| Metric                           | Meaning                                                               |
| -------------------------------- | --------------------------------------------------------------------- |
| `intentCount`                    | Number of pre-write intents reviewed.                                 |
| `promotedCount`                  | Total `serviceOperationSiblingPolicy.promoted[]` entries.             |
| `mutedCount`                     | Total `serviceOperationSiblingPolicy.muted[]` entries.                |
| `suppressedNearNameCount`        | Total suppressed near-name candidates.                                |
| `suppressedSemanticCount`        | Total suppressed semantic candidates.                                 |
| `cueCardCount`                   | Total service-operation `AGENT_REVIEW_CUE` cards.                     |
| `generatedOrPolicyExcludedCount` | Candidates blocked by generated/vendor/framework/class-method policy. |
| `falsePositiveCount`             | Reviewed promoted cues that should not have rendered.                 |
| `missedUsefulSiblingCount`       | Human-reviewed useful siblings not promoted.                          |

For each promoted cue sampled by a human, record:

| Field                | Required Value                                                  |
| -------------------- | --------------------------------------------------------------- |
| `intentName`         | Planned name, e.g. `searchUser`.                                |
| `candidateName`      | Existing operation, e.g. `fetchUser`.                           |
| `candidateIdentity`  | Full identity from the advisory.                                |
| `operationFamily`    | Usually `read-query` for current policy.                        |
| `sharedDomainTokens` | Shared tokens copied from policy evidence.                      |
| `locality`           | sameFile, sameDir, or package/service locality.                 |
| `supportingReasons`  | Suppressed reasons that admitted the candidate.                 |
| `reviewLabel`        | `useful-review-cue`, `noisy-but-harmless`, or `false-positive`. |
| `reviewNote`         | One concise sentence explaining the label.                      |

## Decision Gates

The corpus result may support keeping the current P2b policy as-is when:

- reviewed false positives are rare and explainable;
- generated, bundled, framework, scaffold, vendor, and class-method candidates
  do not appear as service-operation cue cards;
- read/query promotions cite at least one suppressed reason plus locality;
- muted entries stay out of default Markdown.

The result must block mutation-family promotion when:

- `create*`, `update*`, `delete*`, `save*`, `send*`, or `dispatch*` examples
  show mixed semantics;
- reviewers cannot consistently tell whether the candidate should be inspected
  without signature facts;
- a mutation candidate would require wording stronger than review-only
  inspection.

The result must block threshold relaxation when:

- useful siblings are mostly visible only because of generic noun overlap;
- unrelated candidates with shared nouns appear in the sampled promotions;
- lowering global near-name or semantic thresholds would affect non-service
  intents.

## Output Shape

The calibration report should include:

```text
corpus name
package/version or maintainer commit
command route
intent list
aggregate metrics table
sampled promoted cue table
false-positive table
missed-useful-sibling table
decision
next action
```

Allowed decisions:

| Decision                               | Meaning                                                                               |
| -------------------------------------- | ------------------------------------------------------------------------------------- |
| `keep-current-policy`                  | P2b review cue remains acceptable as-is.                                              |
| `tighten-policy`                       | Current policy is too noisy; add blockers before expansion.                           |
| `add-signature-facts-before-expansion` | Reviewers need shape/signature compatibility before mutation or cross-file expansion. |
| `mutation-family-still-muted`          | Mutation families remain muted and unrendered.                                        |
| `needs-more-corpus`                    | Corpus evidence is insufficient or unbalanced.                                        |

## Non-Goals

- Do not add new verbs or mutation-family promotion from this plan alone.
- Do not change `NEAR_NAME_MAX_DISTANCE`, `SEMANTIC_HINT_MIN_SCORE`, or global
  token thresholds.
- Do not change cue-tier rendering, `SAFE_FIX`, or `EXISTS` behavior.
- Do not treat a passing fixture as corpus evidence.
- Do not call `serviceOperationSiblingPolicy.promoted[]` reuse, equivalence, or
  action proof.

## Verdict

WT-23 should stay `MVP` until at least one service-heavy app corpus and one
library/noise-heavy corpus are reviewed with this worksheet. The next
implementation-affecting PR should cite that report before changing mutation
families, signature weighting, or thresholds.

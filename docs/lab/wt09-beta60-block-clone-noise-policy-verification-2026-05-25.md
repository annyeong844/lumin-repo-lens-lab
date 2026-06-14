# WT-09 Beta.60 Block Clone Noise Policy Verification

This note records the beta.60 public-install verification for the WT-09/P4
`block-clone-noise-policy-v1` slice defined in
[`block-clone-detection.md`](../spec/block-clone-detection.md#noise-and-mute-policy).

The run used the installed public package, not only source tests. The output was
written outside the repo at
`C:\Users\endof\AppData\Local\Temp\lumin-bc-verify-beta60\`.

## Result

PASS. The installed beta.60 artifact classified block clone groups into review
and muted lanes, mirrored only shallow navigation counts into the manifest, and
kept block clone evidence out of Markdown and action lanes.

| Check | Result | Evidence |
| --- | --- | --- |
| `block-clones.json.noisePolicy` exists | PASS | `policyId: block-clone-noise-policy-v1`, `reviewGroupCount: 7`, `mutedGroupCount: 93`, `capSaturated: true`. |
| Group visibility and mute reasons exist | PASS | All 100 groups carried `visibility`; 93 muted groups carried `muteReason`; groups retained `reviewOnly: true` and `eligibleForSafeFix: false`. |
| `manifest.blockClones` is shallow | PASS | Manifest mirrored `noisePolicyId`, review/muted counts, reason totals, and `capSaturated`; it did not expose `groups[]`, `instances[]`, source spans, or per-instance files. |
| Markdown/action lanes stay clean | PASS | `audit-summary.latest.md` and `audit-review-pack.latest.md` did not render block clone wording. `fix-plan.json` and `export-action-safety.json` contained no clone group ids or clone evidence fields. |

The muted reason totals were:

| Reason | Count |
| --- | ---: |
| `node-vitest-mirror-pair` | 58 |
| `test-scaffold-repeat` | 18 |
| `same-file-repeat` | 17 |

## Action-Lane False Alarm

The first broad grep found `block-clone` strings in `fix-plan.json` and
`export-action-safety.json`, but those matches were source-code symbol names
from auditing Lumin itself, such as `block-clone-artifact.mjs` and
`BLOCK_CLONE_*`. Follow-up checks found:

- clone group ids like `block-clone:<sha>`: 0 matches;
- clone evidence fields such as `occurrenceCount`, `normalizationMode`,
  `noisePolicy`, `muteReason`, and `reviewOnly`: 0 matches;
- residual block clone result leakage into action lanes: 0 matches.

So CP4 is a pass. The action lanes did not receive clone result evidence.

## Calibration Observation

The artifact saturated `maxGroups: 100`. The current implementation ranks and
caps groups before applying the noise policy; see
[`collectCloneGroups()`](../../_lib/block-clone-artifact.mjs) and
[`applyBlockCloneNoisePolicy()`](../../_lib/block-clone-artifact.mjs).

That is acceptable for beta.60 verification, but it is a P4 calibration topic:
if the capped top 100 is mostly muted noise, review-worthy lower-ranked groups
may be hidden before muting has a chance to narrow the reader surface.

## Decision

Decision: `noise-policy-public-verified` and `p3-markdown-still-deferred`.

WT-09 remains `MVP`, not `DONE`. The next decision is not whether the beta.60
slice works; it does. The next decision is whether P4 corpus calibration supports
changing cap/noise ordering or enabling weak P3 Markdown wording.

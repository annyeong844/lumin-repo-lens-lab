# WT-09 Beta.61 Block Clone Cap/Noise V2 Verification

This note records the beta.61 public-install verification for the WT-09/P4
`block-clone-threshold-policy-v2` cap allocation slice defined in
[`block-clone-detection.md`](../spec/block-clone-detection.md#cap-allocation).

The run used the installed public package, not only source tests. The output was
written outside the repo at
`C:\Users\endof\AppData\Local\Temp\lumin-bc-verify-beta61\`.

## Result

PASS. The installed beta.61 artifact applies the v2 cap/noise ordering, preserves
review groups independently from muted noise, mirrors only shallow manifest
metadata, and keeps block clone evidence out of Markdown and action lanes.

| # | Checkpoint | Result | Evidence |
| - | ---------- | ------ | -------- |
| 1 | Installed version is `0.9.0-beta.61` | PASS | Installed package `package.json` reported `0.9.0-beta.61`. |
| 2 | `thresholds.policyId` is `block-clone-threshold-policy-v2` | PASS | `block-clones.json.thresholds.policyId` and `manifest.blockClones.thresholdPolicyId` both used v2. |
| 3 | V2 cap fields exist | PASS | `maxCandidateGroups: 1000`, `maxReviewGroups: 100`, and `maxMutedGroups: 100` appeared in the raw artifact and manifest mirror. Normal runs correctly omitted deprecated `maxGroups` because it is input-only compatibility. |
| 4 | Legacy `maxGroups` is preserved when supplied | PASS | A temp script invoked the installed `assembleBlockCloneArtifact({ thresholds: { maxGroups: 5 } })`; the resulting `block-clones.json.thresholds.maxGroups` was `5` while v2 fields remained present. |
| 5 | Manifest mirrors legacy `maxGroups` when present | PASS | The temp script wrote that artifact to a temp output directory and called installed `buildManifestEvidence`; `manifest.blockClones.thresholds.maxGroups` was `5`. No-legacy artifacts omitted the field. |
| 6 | Saturation flags are mirrored shallowly | PASS | Artifact and manifest both reported `candidateCapSaturated: true`, `reviewCapSaturated: false`, and `mutedCapSaturated: true`; legacy `capSaturated` was absent. |
| 7 | Markdown/action lanes stay clean | PASS | `audit-summary.latest.md` and `audit-review-pack.latest.md` did not render block clone wording. Action lanes contained no clone group ids or clone evidence fields. |

## Cap/Noise Effect

The same self-dogfood corpus shows the intended behavior change:

| Version | Group Count | Review Groups | Muted Groups | Interpretation |
| ------- | ----------: | ------------: | -----------: | -------------- |
| beta.60 | 100 | 7 | 93 | Raw top-100 cap was mostly consumed by muted noise. |
| beta.61 | 149 | 49 | 100 | Review groups survived independently; muted groups filled the muted budget. |

The beta.61 run emitted `reviewCapSaturated: false`, so all 49 review groups in
the surviving candidate set reached the artifact. It emitted
`mutedCapSaturated: true`, so muted noise was capped at the muted budget instead
of displacing review evidence. It also emitted `candidateCapSaturated: true`,
which honestly reports that the internal candidate guard was reached.

## Legacy MaxGroups Check

The normal audit CLI does not expose threshold overrides, so CP4/CP5 used a
repo-external temp script against the installed beta.61 package:

1. import installed `block-clone-artifact.mjs`;
2. call `assembleBlockCloneArtifact()` with `thresholds: { maxGroups: 5 }`;
3. write that artifact to a temp `block-clones.json`;
4. import installed `audit-manifest.mjs`;
5. call `buildManifestEvidence({ root, outDir })`;
6. assert `artifact.thresholds.maxGroups === 5` and
   `manifest.blockClones.thresholds.maxGroups === 5`.

This directly validates the compatibility contract that source tests pin in
[`test-build-block-clone-index.mjs`](../../tests/test-build-block-clone-index.mjs)
and
[`build-block-clone-index.test.mjs`](../../tests/build-block-clone-index.test.mjs).

## Decision

Decision: `cap-noise-v2-public-verified` and `p3-markdown-still-deferred`.

The cap/noise v2 allocation works on the installed package and fixes the beta.60
review-group truncation problem for the self-dogfood corpus. WT-09 remains
`MVP`, not `DONE`: broader corpus calibration and default P3 Markdown wording
are still separate decisions.

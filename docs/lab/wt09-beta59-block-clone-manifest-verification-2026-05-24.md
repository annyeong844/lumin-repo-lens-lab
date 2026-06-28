# WT-09 Beta.59 Block Clone Manifest Verification

## Scope

This note records the beta.59 public-install verification for the WT-09/P4
block clone manifest mirror.

The checked behavior is P2 only: `manifest.blockClones` mirrors shallow
metadata from `block-clones.json`. It does not validate P3 Markdown wording or
mark the block clone surface `DONE`.

## Runtime Source

- Installed package version: `0.9.0-beta.59`
- Runtime entry: installed public plugin package
- Output location used during verification:
  `C:\Users\endof\AppData\Local\Temp\lumin-bc-verify-beta59\`
- Working tree impact: none

## Checkpoints

| # | Checkpoint | Result | Evidence |
| - | ---------- | ------ | -------- |
| 1 | Installed version is `0.9.0-beta.59` | PASS | Installed `plugin.json` and skill `package.json` agree. |
| 2 | Producer and artifact builder are present | PASS | `build-block-clone-index.mjs` and `block-clone-artifact.mjs` exist in the installed package. |
| 3 | Full profile creates `block-clones.json` | PASS | Full run executed `build-block-clone-index.mjs` in about 114s and emitted a 103KB `block-clones.json`. |
| 4 | `manifest.blockClones` mirrors shallow metadata | PASS | It includes artifact path, schema, policy, status, review-only flag, normalization policy/mode, threshold policy/defaults, and summary counts. |
| 5 | Raw groups, instances, and spans do not leak into manifest | PASS | Raw `block-clones.json` contained 100 groups and 210 instances, while `manifest.blockClones` exposed only metadata and counts. |
| 6 | Markdown remains unchanged for P2 | PASS | `audit-summary.latest.md` and `audit-review-pack.latest.md` did not render block clone wording. |

## Observations

- The artifact status was `confidence-limited`, not `complete`, because
  `skippedFileCount` was 3. That is the expected honest status when policy
  skips are present.
- `groupCount` reached `maxGroups: 100`. The cap behavior is working, but this
  repo saturates the P1/P2 default cap, so threshold and rendering decisions
  need more corpus review before any stronger surface.
- This verification is stronger than an empty-artifact smoke test: the raw
  artifact had populated groups and instances, and the manifest still stayed
  shallow.

## Decision

Decision: `p2-public-verified` and `still-mvp`.

The beta.59 installed package validates the manifest mirror contract. WT-09
remains `MVP`, not `DONE`, because broader corpus calibration, cap/noise
review, P3 review-pack wording, and threshold/rendering decisions are still
separate work.

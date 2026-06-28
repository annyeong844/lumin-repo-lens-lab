# WT-09 Beta.59 Block Clone Noise Review

## Scope

This note reviews the first beta.59 self-dogfood corpus for
`block-clones.json` after the P2 manifest mirror was verified.

The question is narrow: should P3 default Markdown/review-pack wording be
enabled now?

## Input

- Runtime artifact:
  `C:\Users\endof\AppData\Local\Temp\lumin-bc-verify-beta59\block-clones.json`
- Installed version: `0.9.0-beta.59`
- Artifact status: `confidence-limited`
- Summary:
  - files scanned: 529
  - tokens scanned: 951,424
  - groups emitted: 100
  - instances emitted: 210
  - skipped files: 3
  - unavailable files: 0

Skipped files:

| File | Reason | Evidence |
| ---- | ------ | -------- |
| `_lib/shape-hash.mjs` | `generated-file` | `header:generated-marker` |
| `tests/build-framework-resource-surfaces.test.mjs` | `generated-file` | `header:generated-marker` |
| `tests/test-build-framework-resource-surfaces.mjs` | `generated-file` | `header:generated-marker` |

## Distribution

The emitted groups were capped at `maxGroups: 100`.

| Category | Groups | Instances | Group Tokens | Notes |
| -------- | -----: | --------: | -----------: | ----- |
| Node/Vitest mirror pairs | 58 | 126 | 25,482 | Mostly `tests/*.test.mjs` versus `tests/test-*.mjs` migration mirrors. |
| Test cross-file repeats | 18 | 36 | 4,343 | Shared fixture/helper scaffolding across test suites. |
| Same-file repeats | 17 | 34 | 4,310 | Repeated local fixture blocks or repeated assertion scaffolds. |
| Engine cross-file repeats | 7 | 14 | 2,044 | Real maintainer-code review candidates. |

The top six groups were all Node/Vitest mirror pairs, with the largest group
covering `tests/hook-event-store.test.mjs` and
`tests/test-hook-event-store.mjs` across 2,239 normalized tokens.

Representative engine candidates:

| Rank | Tokens | Files |
| ---- | -----: | ----- |
| 14 | 442 | `_lib/check-canon-helpers.mjs` ↔ `_lib/check-canon-naming.mjs` |
| 15 | 417 | `build-function-clone-index.mjs` ↔ `build-shape-index.mjs` |
| 45 | 274 | `_lib/extract-ts.mjs` ↔ `build-call-graph.mjs` |
| 59 | 240 | `_lib/check-canon-helpers.mjs` ↔ `_lib/check-canon-topology.mjs` |
| 66 | 235 | `_lib/export-action-safety.mjs` ↔ `build-call-graph.mjs` |

## Interpretation

The detector is finding real repeated regions. It correctly finds large
Node/Vitest mirror blocks and smaller engine-code repetitions. That is useful
review evidence.

It is not ready for default Markdown. The top of the artifact is dominated by
test migration mirrors and shared test scaffolding, not user-facing production
code. Rendering a generic "block clone review" row today would be true but
noisy: the first reader action would be to mentally filter out known mirror
work.

The `maxGroups: 100` cap was saturated, so the artifact may hide lower-ranked
engine candidates behind test-heavy groups. The next useful slice is a noise
policy or presentation policy, not stronger wording.

## Decision

Decision: `p3-default-markdown-not-ready`,
`needs-noise-policy`, `engine-signal-present`, and `cap-saturated`.

Recommended next slice:

1. Keep `manifest.blockClones` shallow and keep default Markdown off.
2. Add a named classification or mute policy for known Node/Vitest mirror pairs
   and broad test scaffolding before rendering review-pack wording.
3. Preserve engine candidates as review-only evidence; do not feed them into
   `function-clones.json`, pre-write cues, SAFE, fix-plan, or extraction advice.
4. Re-run corpus after the noise policy and decide whether P3 wording is useful
   enough.

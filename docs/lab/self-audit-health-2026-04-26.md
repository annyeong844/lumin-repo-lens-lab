# Self-Audit Health Note - 2026-04-26

This note records the dogfood pass that followed the April 26 self-audit
cleanup sequence. It is maintainer evidence, not part of the public skill
entry surface.

## Baseline

- Repo head: `c04f769` (`Merge pull request #7 from <contributor>/codex/refine-a5-layered-flow`)
- Audit profile: `quick`
- Evidence directory: `review-output-a5-20260426-184318/`
- Generated at: `2026-04-26T09:43:28.856Z`
- Scan range:
  - root: repository root
  - includeTests: `true`
  - production: `false`
  - excludes: `output`, `p6-corpus`, `canonical-draft`, `review-output`, `review-output-main-20260426-171535`, `review-output-e2-20260426-172148`
- Confidence:
  - parseErrors: `0`
  - unresolvedInternalRatio: `0`
  - resolvedInternal: `516`
  - unresolvedInternal: `0`
  - externalImports: `737`
  - blindZones: none

## Gate Snapshot

| Gate | Status | Evidence |
| --- | --- | --- |
| A2 function size | `ok` | `checklist-facts.json.A2_function_size.gate` |
| A5 decoupling ratio | `ok` | `checklist-facts.json.A5_decoupling_ratio.gate` |
| A6 circular deps | `ok` | `checklist-facts.json.A6_circular_deps.gate` |
| B3 dead code | `ok` | `SAFE_FIX = 0`, `REVIEW_FIX = 7`, `MUTED = 1` |
| C5 lint enforcement | `ok` | own `eslint.config.mjs` provides boundary evidence |
| C7 barrel amplification | `ok` | single-package mode, no workspace barrels to discipline |
| E2 silent catch | `ok` | undocumented empty catches = `0`, documented catches = `20` |

## Cleanup Sequence

The pass was produced after these focused PRs landed:

- #4 `Split check-canon section parsers`
  - Reduced the tool-surface function-size pressure by splitting large
    topology/naming parser functions.
- #5 `Enforce tool boundary imports`
  - Added the tool's own `_lib/*.mjs` -> root-script import guard and taught
    triage/checklist evidence to recognize `no-restricted-imports`.
- #6 `Separate documented empty catches`
  - Kept documented empty catches visible while counting only undocumented
    empty catches against E2.
- #7 `Treat layered tool edges as healthy`
  - Used full `crossSubmoduleEdges` evidence for A5 and separated healthy
    `root/scripts/tests -> _lib` flows from review-worthy coupling.

## Important Interpretation

The A5 raw threshold still reports `rawGate = fix` with a ratio of `0.717`.
That signal is preserved deliberately. The final A5 gate is `ok` because all
cross-submodule edges in the dogfood run were healthy layered flows:

| From | To | Count |
| --- | --- | ---: |
| `root` | `_lib` | 149 |
| `tests` | `_lib` | 77 |

The refined output records:

- `crossSubmoduleEdgeSource = full-list`
- `crossSubmoduleEdgesSum = 226`
- `healthyLayeredEdgesSum = 226`
- `reviewedEdgesSum = 0`

This means the high ratio remains auditable, but it no longer claims that the
intended public/test surface calling into the engine is a structural defect.

## Residual Notes

- B3 still reports `REVIEW_FIX = 7`, but `SAFE_FIX = 0`. The tool should not
  claim automatic cleanup from this run.
- The review-fix findings are review prompts, not removal claims.
- Generated local evidence directories remain lab artifacts and should not be
  treated as shipping surface.

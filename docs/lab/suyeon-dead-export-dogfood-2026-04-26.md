# Suyeon Dead Export Dogfood Note - 2026-04-26

This note records the external-corpus dogfood pass against the Suyeon
workspace after the April 26 false-positive cleanup sequence. It is
maintainer evidence, not part of the public skill entry surface.

## Baseline

- Corpus: external Suyeon workspace
- Local root:
  `<maintainer-scratch>\suyeon-daemon-followup-p-work-next-20260426`
- Handoff state: after PR #50 was merged into `p-work-from-current`
- Git metadata caveat: Windows `git` could not resolve this worktree because
  `.git` points at a WSL-style worktree path:
  `<maintainer-scratch>/suyeon-daemon-followup-p-work/.git/worktrees/suyeon-daemon-followup-p-work-next-20260426`
- Audit profile: `quick --production`
- Evidence directory:
  `review-output-suyeon-next-main-after-pr11-20260426/`
- Generated at: `2026-04-26T13:47:45.263Z`
- Scan range:
  - root: Suyeon local root above
  - includeTests: `false`
  - production: `true`
  - excludes: none
- Confidence:
  - parseErrors: `0`
  - unresolvedInternalRatio: `0`
  - resolvedInternal: `2338`
  - unresolvedInternal: `0`
  - externalImports: `318`
  - blindZones: none

## Outcome

| Tier | Count | Interpretation |
| --- | ---: | --- |
| `SAFE_FIX` | 0 | No automatic cleanup candidate. |
| `REVIEW_FIX` | 0 | No review-visible cleanup candidate remains. |
| `DEGRADED` | 4 | Evidence says do not make a removal claim. |
| `MUTED` | 96 | Policy-excluded findings such as test consumers, public API, config, and declaration sidecars. |

The important product signal is:

```text
review-visible cleanup candidates = SAFE_FIX + REVIEW_FIX = 0
```

That means the tool should not ask a user to remove any Suyeon export from
this run. The remaining findings are evidence-improvement or policy buckets,
not cleanup claims.

## Cleanup Sequence

The pass was produced after these focused fixes landed:

- #9 `fix: reduce TS audit false positives`
  - Added direct code-consumer and declaration-dependency protections.
- #10 `fix: tighten MDX import consumer parsing`
  - Prevented fenced MDX examples from becoming live import consumers and
    covered default-plus-namespace MDX imports.
- #11 `fix: resolve suffix hash-import aliases`
  - Fixed Node `#imports` suffix wildcard resolution, including patterns such
    as `#web/request/*.js`.
  - Restored runtime suffix normalization for unsuffixed hash wildcards such
    as `#feat/*` consumed as `#feat/alpha.js`.
  - Counted exported function signature references as declaration dependency
    evidence without counting function bodies as public declarations.

## Before And After

| Evidence directory | SAFE_FIX | REVIEW_FIX | DEGRADED | MUTED | Total | resolvedInternal | external | unresolvedInternal |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `review-output-suyeon-next-20260426/` | 0 | 3 | 3 | 104 | 110 | 2305 | 347 | 0 |
| `review-output-suyeon-next-after-hash-wildcard-20260426/` | 0 | 1 | 3 | 96 | 100 | 2337 | 315 | 0 |
| `review-output-suyeon-next-after-hash-and-signature-20260426/` | 0 | 0 | 4 | 96 | 100 | 2337 | 315 | 0 |
| `review-output-suyeon-next-main-after-pr11-20260426/` | 0 | 0 | 4 | 96 | 100 | 2338 | 318 | 0 |

The first drop (`REVIEW_FIX 3 -> 1`) came from resolving real `#imports`
consumers. The second drop (`REVIEW_FIX 1 -> 0`) came from recognizing exported
function signature dependencies as a reason to downgrade rather than propose a
cleanup.

## Remaining DEGRADED Findings

| Symbol | File | Reason | Declaration refs |
| --- | --- | --- | --- |
| `ApprovalDecision` | `apps/daemon/src/daemon/agent/runtime/approval-gate.ts:8` | `exported-declaration-dependency (2 refs)` | lines 31, 36 |
| `ProviderAuthLoadError` | `apps/daemon/src/daemon/auth/runtime-state.ts:4` | `exported-declaration-dependency (4 refs)` | lines 14, 15, 16, 17 |
| `ProjectRegistryEntry` | `apps/daemon/src/daemon/files/project-registry-state.ts:6` | `exported-declaration-dependency (1 ref)` | line 14 |
| `LoadThreadIndexOptions` | `apps/daemon/src/daemon/sessions/threads-index.ts:13` | `exported-declaration-dependency (2 refs)` | lines 27, 117 |

These are healthy degraded findings. Each symbol is referenced by another
exported declaration in the same file, so removing or demoting the symbol could
change the public or file-level contract. Until checker-grade public surface
precision exists, the honest behavior is to keep them out of `SAFE_FIX` and
`REVIEW_FIX`.

## Interpretation

This dogfood run validates two important P6 principles:

- A raw dead-export classifier bucket is not a removal claim.
- `DEGRADED` is a successful safety outcome when evidence says the static graph
  is not precise enough to recommend cleanup.

The run also shows the value of external-corpus feedback. Suyeon exposed real
false positives around hash import suffixes and exported function signatures.
After those fixes, the tool produces no review-visible cleanup candidates on
this snapshot.

## Residual Notes

- `MUTED = 96` is expected for this corpus because policy exclusions cover
  test-consumed exports, public API surfaces, config entrypoints, and generated
  declaration sidecars.
- `unresolvedInternal = 0` means this pass did not rely on resolver blindness
  to suppress claims.
- The local evidence directories are lab artifacts and are ignored by default
  through `review-output*/`.

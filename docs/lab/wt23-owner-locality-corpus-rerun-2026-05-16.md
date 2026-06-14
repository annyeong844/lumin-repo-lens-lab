# WT-23 Owner-Locality Corpus Rerun - 2026-05-16

## Purpose

This report reruns the WT-23 service-operation corpus after PR #400 preserved
structured pre-write name locality through the normal CLI intent path.

This is calibration evidence only. It does not change analyzer behavior, does
not relax similarity thresholds, and does not mark WT-23 `DONE`.

## Run Summary

| Field             | Value                                                                                                  |
| ----------------- | ------------------------------------------------------------------------------------------------------ |
| Engine route      | maintainer checkout                                                                                    |
| Engine version    | `skills/lumin-repo-lens-lab/package.json` = `0.9.0-beta.50`                                                |
| Maintainer commit | `5318775`                                                                                              |
| Run date          | 2026-05-16                                                                                             |
| Output root       | `C:/Users/endof/Downloads/wt23-owner-locality-corpus-20260516/`                                        |
| Baseline command  | `node audit-repo.mjs --root <repo> --output <out> --profile full`                                      |
| Pre-write command | `node audit-repo.mjs --root <repo> --output <out> --pre-write --intent <intent.json> --no-fresh-audit` |

Both pre-write runs reported `evidenceAvailability.status = "available"` for
the baseline artifacts and reused their baseline output directories.

## Intent Locality Input

The rerun used the same intent names as the first corpus pass, but each
structured `names[]` entry carried an `ownerFile`.

### VNplayer-main

All four intents used `apps/server/src/repository.ts`:

- `searchWorld`
- `lookupSession`
- `findTurn`
- `createSession`

### hono-main

The Hono intents used the owner files from the earlier owner-aware control:

| Intent         | `ownerFile`                   |
| -------------- | ----------------------------- |
| `lookupCookie` | `src/helper/cookie/index.ts`  |
| `findRuntime`  | `src/helper/adapter/index.ts` |
| `searchMime`   | `src/utils/mime.ts`           |
| `queryPath`    | `src/utils/url.ts`            |

The emitted advisories preserved those owner-file declarations in
`intent.nameDeclarations[]`.

## Corpus 1: VNplayer-main

| Field              | Value                                                          |
| ------------------ | -------------------------------------------------------------- |
| Corpus type        | service-heavy app                                              |
| Root path class    | local external checkout                                        |
| Output path class  | local external lab output                                      |
| Baseline wall time | 24.9s                                                          |
| Advisory           | `vnplayer/pre-write-advisory.2026-05-16T10-10-02Z-e6f6fa.json` |
| Intent list        | `searchWorld`, `lookupSession`, `findTurn`, `createSession`    |

### Aggregate Metrics

| Metric                                     | Count |
| ------------------------------------------ | ----: |
| `intentCount`                              |     4 |
| `promotedCount`                            |     0 |
| `mutedCount`                               |    20 |
| `suppressedNearNameCount`                  |    17 |
| `suppressedSemanticCount`                  |    20 |
| service-operation `cueCardCount`           |     0 |
| total `cueCards[]`                         |     7 |
| service-operation `suppressedCues[]`       |    20 |
| total `suppressedCues[]`                   |    57 |
| reviewed service-operation false positives |     0 |
| reviewed missed useful siblings            |     4 |

### Intent Outcomes

| Intent          | Policy outcome      | Main mute reasons                                                                       | Human review                                                                                        |
| --------------- | ------------------- | --------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `searchWorld`   | 0 promoted, 5 muted | `service-sibling-unknown-operation`, `service-sibling-domain-mismatch`                  | No rendered service-operation cue; repository operations remain nested inside `createRepository()`. |
| `lookupSession` | 0 promoted, 5 muted | `service-sibling-unknown-operation`, `service-sibling-domain-mismatch`                  | Same nested-surface limitation.                                                                     |
| `findTurn`      | 0 promoted, 5 muted | `service-sibling-unknown-operation`, `service-sibling-domain-mismatch`, family mismatch | Same nested-surface limitation.                                                                     |
| `createSession` | 0 promoted, 5 muted | `service-sibling-unknown-operation`, `service-sibling-insufficient-suppressed-support`  | Correctly stayed muted; no mutation-family promotion.                                               |

### App-Corpus Finding

The owner-file signal now reaches the CLI path, but VNplayer still produces no
service-operation cue cards. The useful repository operations remain local
nested functions inside `createRepository()` and are not part of the current
service-operation candidate surface. The policy therefore sees type/input alias
surfaces in the same file, not the repository operations a reviewer would want.

This confirms the previous finding: VNplayer requires a separate design for a
local nested service-operation review surface. That surface must not enter the
dead-export `defIndex`.

## Corpus 2: hono-main

| Field              | Value                                                      |
| ------------------ | ---------------------------------------------------------- |
| Corpus type        | library/noise-heavy repo                                   |
| Root path class    | local external checkout                                    |
| Output path class  | local external lab output                                  |
| Baseline wall time | 30.3s                                                      |
| Advisory           | `hono/pre-write-advisory.2026-05-16T10-10-08Z-626a88.json` |
| Intent list        | `lookupCookie`, `findRuntime`, `searchMime`, `queryPath`   |

### Aggregate Metrics

| Metric                                      | Count |
| ------------------------------------------- | ----: |
| `intentCount`                               |     4 |
| `promotedCount`                             |     5 |
| `mutedCount`                                |    18 |
| `suppressedNearNameCount`                   |    16 |
| `suppressedSemanticCount`                   |    20 |
| service-operation `cueCardCount`            |     5 |
| total `cueCards[]`                          |    16 |
| service-operation `suppressedCues[]`        |    18 |
| total `suppressedCues[]`                    |    54 |
| reviewed service-operation false positives  |     0 |
| reviewed useful service-operation cue cards |     5 |

### Intent Outcomes

| Intent         | Promoted service-operation cue cards | Muted evidence retained | Human review                                                                                                         |
| -------------- | ------------------------------------ | ----------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `lookupCookie` | `getCookie`, `getSignedCookie`       | 4 muted                 | Useful sibling cues in the same helper file. Mutation-like `deleteCookie` stayed muted by operation-family mismatch. |
| `findRuntime`  | `getRuntimeKey`                      | 4 muted                 | Useful sibling cue in the same helper file. Cross-file SSG helpers stayed muted by locality mismatch.                |
| `searchMime`   | `getMimeType`                        | 5 muted                 | Useful sibling cue in the same utility file. Type-like and unrelated helpers stayed muted.                           |
| `queryPath`    | `getPath`                            | 5 muted                 | Useful sibling cue in the same utility file. Other same-file helpers stayed muted as unknown-operation evidence.     |

### Rendered Cue Shape

The rendered Hono cues used the P2b review-only wording:

```text
Review related service operation: `getCookie` in `src/helper/cookie/index.ts`.
[heuristic-review, pre-write-advisory.json / lookups[].serviceOperationSiblingPolicy.promoted; cueTier=AGENT_REVIEW_CUE]
policy prewrite-service-operation-sibling-cue-v1
shared domain tokens: `cookie`; operation family: `read-query`; locality: sameDir, sameFile.
supporting suppressed reasons: `single-non-weak-token-only`, `near-length-delta-exceeded`.
action: inspect this related operation before creating parallel service code.
```

The policy still did not create `SAFE_FIX`, `EXISTS`, or action-forcing claims.
It produced review cues only.

## Comparison With First Corpus Pass

| Corpus        | First pass service-operation cue cards | Owner-locality rerun service-operation cue cards | Meaning                                                                                          |
| ------------- | -------------------------------------: | -----------------------------------------------: | ------------------------------------------------------------------------------------------------ |
| VNplayer-main |                                      0 |                                                0 | Owner locality is no longer the blocker; nested local service-operation surface is the blocker.  |
| hono-main     |                                      0 |                                                5 | Owner locality was the blocker; CLI path now surfaces useful review-only service-operation cues. |

## Decision

Decision: `owner-locality-cli-proven`, `hono-useful`, `vnplayer-nested-surface-needed`, and `mutation-family-still-muted`.

The owner-file input path is now calibrated through the normal CLI route. Hono
shows useful review-only service-operation cues with no reviewed
service-operation false positives in this corpus pass. VNplayer still does not
measure useful repository siblings because those operations are nested inside a
factory function and outside the current candidate surface.

Do not relax thresholds, do not add mutation-family promotion, and do not add
signature-weighted promotion from this report.

## Next Action

1. Keep WT-23 P2 rendering review-only.
2. Add more corpus before broadening the policy beyond read/query same-file
   helper functions.
3. Design nested local service-operation review evidence separately if VNplayer
   remains an important target corpus. The surface must not feed dead-export
   ranking or `SAFE_FIX`.
4. Keep mutation families and signature-weighted promotion blocked until a
   follow-up corpus report includes rendered promoted examples and reviewed
   false-positive counts for those policy branches.

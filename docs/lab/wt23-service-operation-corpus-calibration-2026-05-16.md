# WT-23 Service-Operation Corpus Calibration - 2026-05-16

## Purpose

This report applies the WT-23 corpus worksheet to two real repositories after
the beta.50 service-operation Markdown verification. It records what the
current CLI pre-write route can prove before any mutation-family promotion,
signature weighting, or threshold change.

This is calibration evidence only. It does not change analyzer behavior and it
does not mark WT-23 `DONE`.

## Run Summary

| Field             | Value                                                                                                  |
| ----------------- | ------------------------------------------------------------------------------------------------------ |
| Engine route      | maintainer checkout                                                                                    |
| Engine version    | `skills/lumin-repo-lens-lab/package.json` = `0.9.0-beta.50`                                                |
| Maintainer commit | `d7f8946`                                                                                              |
| Run date          | 2026-05-16                                                                                             |
| Output root       | `C:/Users/endof/Downloads/wt23-corpus-calibration-20260516/`                                           |
| Baseline command  | `node audit-repo.mjs --root <repo> --output <out> --profile full`                                      |
| Pre-write command | `node audit-repo.mjs --root <repo> --output <out> --pre-write --intent <intent.json> --no-fresh-audit` |

Both pre-write runs reported `evidenceAvailability.status = "available"` for
`symbols.json` and reused the baseline output directory.

## Corpus 1: VNplayer-main

| Field              | Value                                                          |
| ------------------ | -------------------------------------------------------------- |
| Corpus type        | service-heavy app                                              |
| Root path class    | local external checkout                                        |
| Output path class  | local external lab output                                      |
| Baseline wall time | 18.8s                                                          |
| Advisory           | `vnplayer/pre-write-advisory.2026-05-16T08-16-23Z-d5e50f.json` |
| Intent list        | `searchWorld`, `lookupSession`, `findTurn`, `createSession`    |

### Aggregate Metrics

| Metric                               | Count |
| ------------------------------------ | ----: |
| `intentCount`                        |     4 |
| `promotedCount`                      |     0 |
| `mutedCount`                         |    35 |
| `suppressedNearNameCount`            |    24 |
| `suppressedSemanticCount`            |   101 |
| service-operation `cueCardCount`     |     0 |
| total `cueCards[]`                   |     7 |
| service-operation `suppressedCues[]` |    20 |
| total `suppressedCues[]`             |    57 |
| `generatedOrPolicyExcludedCount`     |     0 |
| reviewed `falsePositiveCount`        |     0 |
| reviewed `missedUsefulSiblingCount`  |     4 |

### Intent Outcomes

| Intent          | Policy outcome       | Main mute reason                           | Human review                                                                          |
| --------------- | -------------------- | ------------------------------------------ | ------------------------------------------------------------------------------------- |
| `searchWorld`   | 0 promoted, 10 muted | `service-sibling-locality-mismatch`        | Missed useful locality because CLI intent did not preserve an owner file.             |
| `lookupSession` | 0 promoted, 7 muted  | `service-sibling-locality-mismatch`        | Missed useful locality for the same owner-file reason.                                |
| `findTurn`      | 0 promoted, 10 muted | `service-sibling-locality-mismatch`        | Missed repository-local read/query functions; nested functions are not in `defIndex`. |
| `createSession` | 0 promoted, 8 muted  | locality mismatch and insufficient support | Correctly kept mutation-family evidence muted.                                        |

### Sampled Policy Evidence

| Intent          | Candidate                                         | Reason                                            | Shared domain tokens | Locality          | Supporting reasons       | Review label       | Note                                                                         |
| --------------- | ------------------------------------------------- | ------------------------------------------------- | -------------------- | ----------------- | ------------------------ | ------------------ | ---------------------------------------------------------------------------- |
| `findTurn`      | `packages/core/src/types.ts::StoredTurn`          | `service-sibling-locality-mismatch`               | `turn`               | none              | `near-distance-exceeded` | noisy-but-harmless | A type record is a plausible diagnostic but not a service-operation cue.     |
| `createSession` | `apps/server/src/repository.ts::createRepository` | `service-sibling-insufficient-suppressed-support` | none                 | none in CLI route | `domain-token-overlap`   | noisy-but-harmless | Correctly stayed muted; this is not a sibling operation for `createSession`. |

### App-Corpus Finding

VNplayer's relevant repository operations are mostly nested inside
`createRepository()` (`getWorld`, `getSession`, `getCurrentTurn`,
`listLibraryDocs`, `createWorld`, and similar functions). The current policy
enumerates `defIndex` and `classMethodIndex`; it does not have a local nested
service-operation surface. As a result, the app corpus does not yet measure the
usefulness of actual repository siblings. It mostly measures exported type and
input aliases near the intended domain.

## Corpus 2: hono-main

| Field              | Value                                                      |
| ------------------ | ---------------------------------------------------------- |
| Corpus type        | library/noise-heavy repo                                   |
| Root path class    | local external checkout                                    |
| Output path class  | local external lab output                                  |
| Baseline wall time | 26.2s                                                      |
| Advisory           | `hono/pre-write-advisory.2026-05-16T08-17-22Z-2f4a73.json` |
| Intent list        | `lookupCookie`, `findRuntime`, `searchMime`, `queryPath`   |

### Aggregate Metrics

| Metric                               | Count |
| ------------------------------------ | ----: |
| `intentCount`                        |     4 |
| `promotedCount`                      |     0 |
| `mutedCount`                         |    31 |
| `suppressedNearNameCount`            |    50 |
| `suppressedSemanticCount`            |   143 |
| service-operation `cueCardCount`     |     0 |
| total `cueCards[]`                   |    11 |
| service-operation `suppressedCues[]` |    20 |
| total `suppressedCues[]`             |    56 |
| `generatedOrPolicyExcludedCount`     |     4 |
| reviewed `falsePositiveCount`        |     0 |
| reviewed `missedUsefulSiblingCount`  |     4 |

### Intent Outcomes

| Intent         | Policy outcome      | Main mute reason                               | Human review                                                                                 |
| -------------- | ------------------- | ---------------------------------------------- | -------------------------------------------------------------------------------------------- |
| `lookupCookie` | 0 promoted, 9 muted | `service-sibling-locality-mismatch`            | `getCookie`/`getSignedCookie` are useful siblings but cannot promote without owner locality. |
| `findRuntime`  | 0 promoted, 7 muted | `service-sibling-locality-mismatch`            | `getRuntimeKey` is a useful sibling in the same helper file under owner-aware control.       |
| `searchMime`   | 0 promoted, 6 muted | `service-sibling-locality-mismatch`            | `getMimeType` is a useful sibling in the same file under owner-aware control.                |
| `queryPath`    | 0 promoted, 9 muted | surface-kind unsupported and locality mismatch | Class-method and unrelated path/query helpers stayed out of service-operation cue cards.     |

### Sampled Policy Evidence

| Intent       | Candidate                                                | Reason                                     | Shared domain tokens | Locality          | Supporting reasons                                     | Review label       | Note                                                                    |
| ------------ | -------------------------------------------------------- | ------------------------------------------ | -------------------- | ----------------- | ------------------------------------------------------ | ------------------ | ----------------------------------------------------------------------- |
| `searchMime` | `src/utils/mime.ts::BaseMime`                            | `service-sibling-locality-mismatch`        | `mime`               | none in CLI route | `single-non-weak-token-only`, `near-distance-exceeded` | noisy-but-harmless | Type-like exported surface stayed muted, which is correct.              |
| `queryPath`  | `src/router/reg-exp-router/prepared-router.ts::#addPath` | `service-sibling-surface-kind-unsupported` | none                 | none              | `single-non-weak-token-only`, `near-distance-exceeded` | noisy-but-harmless | Class/private method evidence stayed out of the service-operation lane. |

## Owner-Aware Control

The CLI corpus run above is the authoritative current-product route.
However, a direct control call to `lookupName()` with an injected
`intentDeclaration.ownerFile` shows why the missing owner signal matters.

This control is not counted as corpus pass/fail because it bypasses the CLI
intent normalization path.

| Corpus        | Intent owner file               | Useful promoted examples                                                                                |
| ------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------- |
| VNplayer-main | `apps/server/src/repository.ts` | none; relevant repository operations are nested inside `createRepository()` and absent from `defIndex`. |
| hono-main     | `src/helper/cookie/index.ts`    | `lookupCookie` promoted `getCookie` and `getSignedCookie`.                                              |
| hono-main     | `src/helper/adapter/index.ts`   | `findRuntime` promoted `getRuntimeKey`.                                                                 |
| hono-main     | `src/utils/mime.ts`             | `searchMime` promoted `getMimeType`.                                                                    |
| hono-main     | `src/utils/url.ts`              | `queryPath` promoted `getPath` and `getPathNoStrict`.                                                   |

The control supports the policy idea for library helper files, but it also
shows that the CLI pre-write input shape needs an owner-file path before the
cue can be calibrated in normal agent use.

## False Positives

No service-operation sibling cue cards rendered in either CLI run, so
there were no rendered service-operation false positives. This is a safety
result, not a usefulness result.

The normal pre-write advisory still rendered non-service review cues from the
existing near-name and semantic lanes. Those are outside WT-23 and were not
counted as service-operation false positives.

## Missed Useful Siblings

| Corpus        | Miss                                                                                                   | Why it was missed                                                                             | Required follow-up                                                                             |
| ------------- | ------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| VNplayer-main | nested repository operations such as `getWorld`, `getSession`, `getCurrentTurn`, and `listLibraryDocs` | nested function declarations inside `createRepository()` are not service-operation candidates | Decide whether to add a local service-operation surface, separate from dead-export `defIndex`. |
| hono-main     | `lookupCookie` -> `getCookie` / `getSignedCookie`                                                      | CLI intent did not preserve owner locality                                                    | Preserve `ownerFile`/`targetFile` in normalized `names[]` declarations.                        |
| hono-main     | `findRuntime` -> `getRuntimeKey`                                                                       | CLI intent did not preserve owner locality                                                    | Same owner-file input follow-up.                                                               |
| hono-main     | `searchMime` -> `getMimeType`                                                                          | CLI intent did not preserve owner locality                                                    | Same owner-file input follow-up.                                                               |
| hono-main     | `queryPath` -> `getPath` / `getPathNoStrict`                                                           | CLI intent did not preserve owner locality                                                    | Same owner-file input follow-up, then rerun corpus.                                            |

## Decision

Decision: `needs-more-corpus` and `mutation-family-still-muted`.

The current P2b cue remains safe because no service-operation cue cards render
without the required locality evidence. The corpus run does not yet prove that
the cue is useful in normal CLI use, because the CLI intent normalization
path drops the owner-file signal that the policy needs for promotion.

Do not relax thresholds and do not add mutation-family promotion from this
report.

## Next Action

1. Add or specify a CLI pre-write intent shape that preserves owner locality
   for `names[]` entries, such as `ownerFile`, `file`, or `targetFile`.
2. Rerun this corpus worksheet through the CLI path after owner locality
   is preserved.
3. Decide separately whether nested local repository functions should have a
   review-only service-operation surface. That surface must not enter
   dead-export `defIndex`.
4. Keep mutation families and signature-weighted promotion blocked until a
   follow-up corpus report includes rendered promoted examples and reviewed
   false-positive counts.

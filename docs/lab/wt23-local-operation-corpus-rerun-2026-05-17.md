# WT-23 Local Operation Corpus Rerun - 2026-05-17

## Purpose

This report reruns the VNplayer-style WT-23 corpus after beta.53 added the
local-operation cue bridge for nested repository factory operations.

This is calibration evidence only. It does not relax thresholds, does not add
mutation-family promotion, and does not mark WT-23 `DONE`.

## Run Summary

| Field             | Value                                                                                                      |
| ----------------- | ---------------------------------------------------------------------------------------------------------- |
| Installed version | `0.9.0-beta.53`                                                                                            |
| Engine route      | installed public package                                                                                   |
| Entry point       | `node <plugin-cache>/0.9.0-beta.53/skills/lumin-repo-lens-lab/scripts/audit-repo.mjs --pre-write`              |
| Corpus            | `C:/Users/endof/Downloads/VNplayer-main`                                                                   |
| Reference file    | `apps/server/src/repository.ts`, `createRepository(db: DatabaseSync)` at line 802                          |
| Output path class | temporary path under `C:/tmp/lrl-corpus-vnplayer-444/output/`, removed after verification                  |
| Runtime           | 4.1s cold-cache pre-write advisory generation; plugin dependencies were already installed from first-run use |

The run produced `pre-write-advisory.<invocationId>.json`,
`pre-write-advisory.latest.json`, `symbols.json`, `manifest.json`, and stdout
Markdown. The temporary fixture/output tree was removed after the run and the
working tree stayed clean.

## Intent Input

The advisory used five structured name declarations. Every entry carried
`ownerFile: apps/server/src/repository.ts`.

| Intent            | Family     | Purpose                                                              |
| ----------------- | ---------- | -------------------------------------------------------------------- |
| `searchWorld`     | read/query | search a world record by id from the repository factory              |
| `findSession`     | read/query | find an existing session record from the repository factory          |
| `findCurrentTurn` | read/query | find the current turn for a session in the repository factory        |
| `queryLibraryDoc` | read/query | query library docs for a world from the repository factory           |
| `removeWorld`     | mutation   | remove a world record from the repository factory                    |

The installed package emitted five lookup results. `symbols.json` advertised
`meta.supports.nestedLocalOperationIndex = true`, and
`symbols.preWriteLocalOperationIndex.status = "complete"`.

## Local Operation Policy Results

| Intent            | Promoted | Promoted names and shared tokens                                                                 | Muted | Main muted reason                   |
| ----------------- | -------: | ------------------------------------------------------------------------------------------------ | ----: | ----------------------------------- |
| `searchWorld`     |        2 | `getWorld` (`world`), `listWorlds` (`world`)                                                     |     5 | `local-operation-domain-mismatch`   |
| `findSession`     |        1 | `getSession` (`session`)                                                                         |     5 | `local-operation-domain-mismatch`   |
| `findCurrentTurn` |        4 | `getCurrentTurn` (`current`, `turn`), `getTurn` (`turn`), `listVisibleTurns` (`turn`), `getCgAssetForTurn` (`turn`) |     5 | `local-operation-domain-mismatch`   |
| `queryLibraryDoc` |        2 | `listLibraryDocs` (`library`, `doc`), `listLibraryOutline` (`library`)                           |     5 | `local-operation-domain-mismatch`   |
| `removeWorld`     |        0 | none                                                                                             |     5 | `local-operation-domain-mismatch`   |

Aggregate:

| Metric                                  | Count |
| --------------------------------------- | ----: |
| `intentCount`                           |     5 |
| local-operation `promoted[]` entries    |     9 |
| local-operation `muted[]` entries       |    25 |
| local-operation rendered review lines   |     9 |
| local-operation reviewed false positives |     0 |
| `SAFE_CUE` / `EXISTS` / `SAFE_FIX` cues |     0 |
| default Markdown muted-reason leaks     |     0 |

All promoted local-operation entries were read/query operations nested inside
`createRepository()`, had `sameFile = true`, and shared at least one domain
token with the intent. The mutation intent `removeWorld` produced zero promoted
local-operation cues.

The local-operation surface stayed separate from the older
`serviceOperationSiblingPolicy`: nested identities such as
`apps/server/src/repository.ts::createRepository#getWorld` did not cross-feed
into `serviceOperationSiblingPolicy.promoted[]` or `.muted[]`.

## Rendered Cue Shape

The installed beta.53 package rendered one local-operation review row for each
promoted local operation. A representative stdout Markdown cue:

```md
- Review related local service operation: `getCgAssetForTurn` inside `createRepository` in `apps/server/src/repository.ts`.
  [heuristic-review, pre-write-advisory.json / lookups[].localOperationSiblingPolicy.promoted; cueTier=AGENT_REVIEW_CUE]
  policy prewrite-local-operation-sibling-v1
  shared domain tokens: `turn`; operation family: `read-query`; locality: sameFile, sameDir.
  supporting local-operation reasons: `unknown`.
  action: inspect this local operation before creating parallel service code.
```

All local-operation cue cards stayed in `AGENT_REVIEW_CUE`. None became
`SAFE_CUE`, `EXISTS`, or `SAFE_FIX`.

Muted local-operation entries stayed in `localOperationSiblingPolicy.muted[]`
and the JSON `suppressedCues[]` evidence lane. The default Markdown did not
render `local-operation-domain-mismatch`.

## Reviewed Findings

### Useful Local-Operation Signal

The bridge is useful enough for the VNplayer-style repository factory target:

- the four read/query intents produced nine relevant same-file review cues;
- no reviewed local-operation cue was a false positive;
- the mutation intent stayed at zero promoted local-operation cues;
- muted local-operation evidence remained hidden from default Markdown;
- the cue tier stayed review-only.

`findCurrentTurn -> getCgAssetForTurn` is a borderline review cue because it
shares the `turn` token but adds the asset domain. This remains acceptable for
the current contract because the cue is explicitly review evidence, not reuse or
safe-fix proof.

### Cosmetic Follow-Up

Local-operation cues currently render:

```text
supporting local-operation reasons: `unknown`.
```

This happens because local-operation policy intentionally bypasses suppressed
near-name/semantic evidence. A follow-up should emit a stable default support
reason such as `local-operation-same-file-domain-overlap`.

### Adjacent Service-Policy Finding

The same corpus exposed an older service-operation policy false positive:
`queryLibraryDoc` promoted `ListLibraryDocsOptions` and
`ListLibraryOutlineOptions` through `serviceOperationSiblingPolicy`.

Those are TypeScript type/interface-like names, not service functions. This is
outside the local-operation bridge, but it suggests a separate service-policy
candidate-kind filter or calibration slice.

## Decision

Decision: `useful-enough` for the local-operation bridge v1 and
`policy-adjustment-needed` for two follow-ups.

Do not relax thresholds, do not add mutation-family promotion, and do not add
signature-weighted promotion from this report.

## Next Action

1. Add a stable local-operation support reason so renderer output does not fall
   back to `unknown`.
2. Track the service-operation type-name false positive separately from the
   local-operation bridge.
3. Add more corpus only before broadening the policy to mutation families,
   signature-weighted matching, or other factory-name patterns beyond this
   VNplayer reference fixture.

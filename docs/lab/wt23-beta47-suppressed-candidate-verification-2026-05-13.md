# WT-23 beta.47 Suppressed Candidate Verification

Maintainer lab note for the WT-23 pre-write semantic sibling recall slice. This
note records the local corpus check and the follow-up public installed
`0.9.0-beta.47` package verification.

## Scope

- Feature: WT-23 suppressed pre-write candidate diagnostics.
- Package label in repo: `0.9.0-beta.47`.
- Local corpus: small TypeScript service fixture with existing `fetchUser` /
  `fetchPost` exports and an intent to add `searchUser`.
- Expected behavior: plausible service sibling evidence is recorded as muted
  diagnostics without entering formal cue cards.

## Local Corpus Command

The local run used the maintainer checkout scripts:

```text
node build-symbol-graph.mjs --root <fixture> --output <output>
node pre-write.mjs --root <fixture> --output <output> --intent <output>/intent.json
```

Fixture shape:

```text
package.json
src/services/user.ts      exports fetchUser
src/services/post.ts      exports fetchPost
```

Intent:

```json
{
  "names": [
    {
      "name": "searchUser",
      "kind": "function",
      "why": "search user data"
    }
  ],
  "shapes": [],
  "files": [
    "src/services/user-search.ts"
  ],
  "dependencies": [],
  "plannedTypeEscapes": []
}
```

## Local Results

| Check | Result | Evidence |
|---|---:|---|
| `intentTokens` are exposed | PASS | `["search", "user", "data"]` |
| `fetchUser` does not enter `nearNames` | PASS | `nearNames: []` |
| `fetchUser` does not enter `semanticHints` | PASS | `semanticHints: []` |
| `fetchUser` enters `suppressedNearNames` | PASS | reason `near-distance-exceeded`, `distance: 3`, `matchedField: "defIndex"` |
| `fetchUser` enters `suppressedSemanticHints` | PASS | reason `single-non-weak-token-only`, `score: 1`, `matchedTokens: ["user"]` |
| muted evidence is preserved in `suppressedCues` | PASS | two `MUTED_CUE` entries for `near-name` and `intent-token` |
| no review/action cue is promoted | PASS | `cueCards: []` |

Observed suppressed cue payloads included:

```json
{
  "cueTier": "MUTED_CUE",
  "evidenceLane": "near-name",
  "reason": "near-distance-exceeded",
  "tokens": ["user"],
  "distance": 3,
  "ownerFile": "src/services/user.ts",
  "exportedName": "fetchUser",
  "identity": "src/services/user.ts::fetchUser",
  "matchedField": "defIndex"
}
```

```json
{
  "cueTier": "MUTED_CUE",
  "evidenceLane": "intent-token",
  "reason": "single-non-weak-token-only",
  "tokens": ["user"],
  "score": 1,
  "ownerFile": "src/services/user.ts",
  "exportedName": "fetchUser",
  "identity": "src/services/user.ts::fetchUser",
  "matchedField": "defIndex"
}
```

## Initial Public Install Status

Public installed beta.47 was not available in the local Claude plugin cache
during the first verification attempt. The local cache contained beta.46 and
earlier beta builds, but no `0.9.0-beta.47` directory under:

```text
%USERPROFILE%\.claude\plugins\cache\annyeong844-marketplace\lumin-repo-lens-lab\
```

That initial check established the corpus checklist and confirmed that the
maintainer checkout emitted the intended muted evidence, but it did not close
public install verification.

## Public Install Verification

After publishing beta.47 to the public package repository, the corpus was
repeated with the installed plugin path. The public install check passed all
expected WT-23 first-slice assertions:

| Check | Result | Evidence |
|---|---:|---|
| installed version is `0.9.0-beta.47` across plugin metadata and skill `package.json` | PASS | `installed_plugins.json`, install `plugin.json`, install `marketplace.json`, and install skill `package.json` all reported `0.9.0-beta.47` |
| `fetchUser` does not enter `nearNames` | PASS | `nearNames: []` |
| `fetchUser` does not enter `semanticHints` | PASS | `semanticHints: []` |
| `fetchUser` enters `suppressedNearNames` | PASS | reason `near-distance-exceeded`, `distance: 3`, `ownerFile: "src/services/user.ts"` |
| `fetchUser` enters `suppressedSemanticHints` | PASS | reason `single-non-weak-token-only`, `score: 1`, `matchedTokens: ["user"]` |
| muted evidence is preserved without promotion | PASS | `suppressedCues` contained two muted entries and `cueCards: []` |

The public install also confirmed install/dev parity for
`pre-write-lookup-name.mjs` and `pre-write-cue-tiers.mjs`.

## Follow-Up Boundary

This closes WT-23 public install verification for the first slice. It does not
promote service-operation siblings to `AGENT_REVIEW_CUE`, relax thresholds, or
claim that `searchUser` and `fetchUser` are equivalent. Follow-up work should
use the suppressed-candidate evidence to design a named, versioned cue policy
for verb-family, locality, and signature-sensitive promotion.

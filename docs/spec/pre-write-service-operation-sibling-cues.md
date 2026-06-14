# Pre-Write Service Operation Sibling Cues

> **Role:** maintainer-facing design spec for promoting selected suppressed
> pre-write service-operation siblings into review cues without relaxing global
> similarity thresholds.
> **Status:** P2b renderer wording public-verified in beta.50; owner-locality
> CLI rerun recorded `owner-locality-cli-proven`, `hono-useful`,
> `vnplayer-nested-surface-needed`, and `mutation-family-still-muted`.
> **Last updated:** 2026-05-16

---

## 1. Problem

WT-23 proved that pre-write can now explain why plausible service siblings were
not surfaced. In the public beta.47 fixture, an intent to add `searchUser`
correctly left the existing `fetchUser` out of formal `nearNames[]`,
`semanticHints[]`, and `cueCards[]`, while recording the miss as muted evidence:

```text
suppressedNearNames[]:
  fetchUser, reason near-distance-exceeded, distance 3

suppressedSemanticHints[]:
  fetchUser, reason single-non-weak-token-only, matchedTokens ["user"], score 1
```

That was the right first slice. The public beta.48 fixture then added a
versioned `serviceOperationSiblingPolicy` object that can promote
`searchUser` -> `fetchUser` internally while leaving formal `nearNames[]`,
`semanticHints[]`, and `cueCards[]` unchanged. The next product question is
narrower:

```text
When may promoted sibling-policy evidence render as AGENT_REVIEW_CUE?
```

The answer must not be "lower the thresholds." A broad threshold relaxation
would affect every intent and invite noisy matches such as unrelated
`handle*`, `create*`, or generic `user` candidates. The answer also must not
be "render every promoted P1 policy entry immediately." P2 needs a readiness
gate that proves cue rendering stays review-only, scoped, and quiet on negative
fixtures.

## 2. Goals

- Promote a small, explainable set of service-operation sibling candidates to
  `AGENT_REVIEW_CUE`.
- Keep exact reuse, semantic equivalence, and action safety out of scope.
- Use existing suppressed evidence as the input surface, rather than changing
  global `nearNames` or `semanticHints` thresholds.
- Require multiple independent signals: operation family, domain token,
  locality, and optional signature compatibility.
- Emit a versioned policy object so future calibration can explain why a
  candidate was promoted or kept muted.
- Preserve deterministic caps and sorting.

## 3. Non-Goals

- Do not emit `SAFE_CUE`, `EXISTS`, `SAFE_FIX`, or "reuse this" wording.
- Do not claim `searchUser` and `fetchUser` are equivalent.
- Do not introduce embeddings, fuzzy semantic search, or broad synonym tables.
- Do not relax `NEAR_NAME_MAX_DISTANCE`, `SEMANTIC_HINT_MIN_SCORE`, or existing
  token thresholds globally.
- Do not promote candidates from generated, bundled, vendor, or policy-excluded
  paths.
- Do not make a generic domain noun such as `user`, `task`, or `item`
  sufficient by itself.

## 4. Evidence Contract

Allowed claim:

```text
An existing service operation with the same domain token and compatible
operation family was found nearby. Review it before creating a new helper.
```

Disallowed claims:

```text
The existing operation does the same thing.
The new helper should call this operation.
The planned helper is unnecessary.
It is safe to delete, demote, or reuse either symbol.
```

The cue tier must be:

```json
{
  "cueTier": "AGENT_REVIEW_CUE",
  "evidenceLane": "service-operation-sibling",
  "confidence": "heuristic-review",
  "claim": "related service operation sibling"
}
```

## 5. Input Surface

The v1 policy consumes existing `lookupName()` output:

```text
intentTokens[]
suppressedNearNames[]
suppressedSemanticHints[]
nearNames[]
semanticHints[]
```

The primary source is suppressed evidence. A candidate is eligible only if it
has at least one suppressed lane that explains why the ordinary path rejected
it. This keeps the policy scoped to "plausible but currently hidden" siblings
instead of broadening the normal search result set.

This also means the policy is not a second broad candidate enumerator. A
candidate with no suppressed near-name or semantic hint remains outside the
policy object, even if a human can name a later gate it would fail. For example,
`searchPost` and `fetchUser` normally share no name/domain signal, so `fetchUser`
may stay absent from both `promoted[]` and `muted[]`. A
`service-sibling-domain-mismatch` mute is expected only after another supported
signal has already brought the candidate into the suppressed-evidence input set.

Required candidate fields:

```json
{
  "name": "fetchUser",
  "ownerFile": "src/services/user.ts",
  "identity": "src/services/user.ts::fetchUser",
  "matchedField": "defIndex",
  "matchedTokens": ["user"],
  "reason": "single-non-weak-token-only",
  "locality": {
    "sameDir": true,
    "sameFile": false
  }
}
```

If required fields are unavailable, the policy should keep the candidate muted
and record `reason: "service-sibling-insufficient-metadata"`.

## 6. Operation Family

The policy should split an identifier into:

```text
operation verb(s) + domain tokens
```

For `searchUser`, the operation is `search` and the domain token is `user`.
For `fetchUser`, the operation is `fetch` and the domain token is `user`.

### Read/Query Family

The v1 promotable family is read/query only:

```text
fetch, get, load, read, find, search, lookup, query, list, resolve
```

These verbs may be compatible with each other for review purposes when the
other gates also pass. This does not mean they are synonyms; it means they are
close enough to ask an agent to inspect.

### Mutation Families

Mutation verbs are not cross-family promotable in v1:

```text
create/add
update/patch/set
delete/remove/destroy
save/write
send/dispatch/emit
```

They may become future families, but each family needs separate fixtures. A
`deleteUser` intent must not promote `fetchUser` simply because both contain
`user`.

### Unknown Verbs

If either side has no recognized operation verb, keep the candidate muted with:

```text
service-sibling-unknown-operation
```

This fail-closed behavior prevents ordinary noun overlap from becoming a review
cue.

## 7. Promotion Gates

A candidate may be promoted only when all required gates pass.

### Gate A: Suppressed Evidence

At least one suppressed lane must exist for the candidate:

- `single-non-weak-token-only`
- `near-distance-exceeded`
- `near-length-delta-exceeded`

`domain-token-overlap` alone is supporting evidence, not sufficient evidence.

### Gate B: Domain Token

The intent and candidate must share at least one normalized non-verb domain
token. The shared token must be present in the candidate name, not only in free
text prose.

Examples:

```text
searchUser -> fetchUser      PASS domain token user
searchPost -> fetchUser      FAIL domain token mismatch, if already evaluated
searchData -> fetchUser      FAIL generic prose/token only
```

### Gate C: Operation Family

Both operation verbs must be in the read/query family for v1.

Examples:

```text
searchUser -> fetchUser      PASS read/query
lookupUser -> findUser       PASS read/query
createUser -> fetchUser      FAIL create vs read/query
deleteUser -> removeUser     MUTED in v1; future mutation-family policy
```

### Gate D: Locality

At least one locality signal must be true:

- same file;
- same directory;
- same nearest package/service folder when that metadata exists;
- intent planned file path and candidate owner path share the same immediate
  domain folder, such as `src/services/`.

If no intent file is known, same directory or same file is required. Locality is
what makes a broad token such as `user` useful rather than noisy.

### Gate E: Surface Kind

V1 may promote top-level function/helper candidates from `defIndex`.
`classMethodIndex` candidates should continue through the class-method lane
unless a later spec explicitly merges method and service-sibling policies.
Candidates whose `defIndex` record identifies a TypeScript-only declaration,
such as `TSInterfaceDeclaration`, `TSTypeAliasDeclaration`,
`TSEnumDeclaration`, or `TSModuleDeclaration`, must stay muted with
`service-sibling-non-callable-definition`; names such as
`ListLibraryDocsOptions` are not service operations even when their leading
token looks like a read/query verb.

### Gate F: Signature Support

Signature compatibility is supporting evidence in v1, not a hard requirement.
If normalized function signature facts are available, record:

```json
{
  "signatureSupport": {
    "status": "compatible | incompatible | unavailable",
    "reason": "arity-compatible | async-mismatch | no-signature-facts"
  }
}
```

An incompatible signature keeps the candidate muted. An unavailable signature
may still promote only if Gates A-E pass.

## 8. Policy Output

Add a policy summary to each lookup result:

```json
{
  "serviceOperationSiblingPolicy": {
    "policyId": "prewrite-service-operation-sibling-cue",
    "policyVersion": "prewrite-service-operation-sibling-cue-v1",
    "evaluatedCandidateCount": 2,
    "promotedCandidateCount": 1,
    "mutedCandidateCount": 1,
    "promoted": [
      {
        "identity": "src/services/user.ts::fetchUser",
        "name": "fetchUser",
        "ownerFile": "src/services/user.ts",
        "operationFamily": "read-query",
        "sharedDomainTokens": ["user"],
        "supportingReasons": [
          "near-distance-exceeded",
          "single-non-weak-token-only"
        ],
        "locality": {
          "sameDir": true,
          "sameFile": false
        },
        "signatureSupport": {
          "status": "unavailable",
          "reason": "no-signature-facts"
        }
      }
    ],
    "muted": [
      {
        "identity": "src/services/post.ts::fetchPost",
        "reason": "service-sibling-domain-mismatch"
      }
    ]
  }
}
```

The cue-tier adapter may then render promoted entries as:

```json
{
  "cueTier": "AGENT_REVIEW_CUE",
  "evidenceLane": "service-operation-sibling",
  "claim": "related service operation sibling",
  "evidence": [
    {
      "artifact": "pre-write-advisory.json",
      "matchedField": "lookups[].serviceOperationSiblingPolicy.promoted",
      "policyVersion": "prewrite-service-operation-sibling-cue-v1",
      "candidateIdentity": "src/services/user.ts::fetchUser",
      "operationFamily": "read-query",
      "sharedDomainTokens": ["user"]
    }
  ]
}
```

## 9. Renderer Wording

Default Markdown should keep the cue clearly review-only:

```text
Review related service operation: fetchUser in src/services/user.ts.
Shared domain token user; compatible read/query operation family.
This is not reuse proof and does not change safe action evidence.
```

Do not render muted policy details by default unless the advisory already has a
debug/verbose lane for suppressed evidence.

## 10. P1 Baseline

P1 is a pure evidence-object slice. It does not render cue cards.

P1 invariants that must remain true before P2:

1. `searchUser` intent with existing `fetchUser` in the same service directory
   produces a `serviceOperationSiblingPolicy.promoted[]` entry while keeping
   formal `nearNames[]`, `semanticHints[]`, and `cueCards[]` empty.
2. `createUser` intent with existing `fetchUser` stays in
   `serviceOperationSiblingPolicy.muted[]` with
   `service-sibling-operation-family-mismatch`.
3. `searchPost` intent with existing `fetchUser` is never promoted. If no
   suppressed evidence admits `fetchUser` into policy evaluation, it remains
   outside both `promoted[]` and `muted[]` as noise-floor behavior.
4. Muted policy entries may still mirror into `suppressedCues[]` for JSON
   readers, but muted evidence must not become user-facing cue cards.
5. Promoted and muted arrays are sorted by locality support, operation family,
   candidate name, owner file, and identity.
6. Candidate arrays are capped, with raw counts preserved.

## 11. P2 Readiness Gate

P2 may render `serviceOperationSiblingPolicy.promoted[]` as
`AGENT_REVIEW_CUE` only after the focused fixture matrix below exists and
passes. This gate is intentionally stricter than P1 because it changes what an
agent sees in the normal pre-write advisory.

Positive render fixtures:

1. `searchUser` intent with existing `fetchUser` in the same service directory
   renders exactly one `AGENT_REVIEW_CUE` in lane `service-operation-sibling`.
2. `lookupUser` intent with existing `findUser` in the same file or directory
   renders exactly one `AGENT_REVIEW_CUE` in lane `service-operation-sibling`.
3. The cue evidence includes `policyId`, `policyVersion`, `operationFamily`,
   `sharedDomainTokens`, locality, and original suppressed reasons from
   `pre-write-advisory.json`.

Negative no-render fixtures:

4. `createUser` intent with existing `fetchUser` remains muted and renders no
   service-operation sibling cue.
5. `deleteUser` intent with existing `removeUser` remains muted in v1 and
   renders no service-operation sibling cue.
6. `searchPost` intent with existing `fetchUser` remains outside the policy
   object when no suppressed evidence admits it; if a synthetic supporting
   signal admits it, it stays muted with `service-sibling-domain-mismatch`.
7. A generated, bundled, vendor, scaffold, or framework-resource candidate
   stays muted and renders no service-operation sibling cue.
8. A `classMethodIndex` candidate continues through the `class-method-name`
   lane and is not rendered by the service-operation sibling adapter.
9. No promoted candidate may become `SAFE_CUE`, `EXISTS`, `SAFE_FIX`, or
   exact-symbol evidence.
10. Existing suppressed diagnostics remain present when a candidate is rendered
    as review. Rendering adds a cue; it does not erase the diagnostic trail.

Renderer wording:

- The default Markdown may say `Review related service operation`.
- The default Markdown must not say `reuse`, `equivalent`, `safe`, `exists`,
  `should call`, `blocking failure`, or equivalent certainty wording.
- Muted policy entries remain hidden by default unless the advisory already has
  a debug or verbose suppressed-evidence lane.

Data contract:

- The cue-tier adapter consumes `lookups[].serviceOperationSiblingPolicy`.
- The adapter must copy policy decisions into cue evidence; it must not
  re-evaluate operation families, domain tokens, locality, or signatures.
- Each rendered cue references `pre-write-advisory.json`,
  `lookups[].serviceOperationSiblingPolicy.promoted`, `policyId`,
  `policyVersion`, candidate identity, operation family, shared domain tokens,
  locality, and supporting suppressed reasons.

Corpus readiness:

- Run at least one service-heavy application corpus and one library or
  noise-heavy corpus before enabling P2 by default.
- Record promoted count, muted count, suppressed count, top promoted names,
  generated/vendor/framework suppressed count, and reviewed false positives.
- If a corpus is unavailable, document the missing corpus explicitly in
  `docs/lab/` and keep P2 behind a review-only implementation gate.
- Mutation-family cues and signature-weighted promotion stay out of scope until
  a later calibration slice.

## 12. Implementation Slices

P0 documentation:

- Land this policy spec and point WT-23 at it.

P1 policy evaluator (implemented in beta.48):

- Add a pure function that consumes a lookup result and intent metadata.
- Emit `serviceOperationSiblingPolicy`.
- Add unit tests for the P1 positive/negative fixtures above.
- Do not render cue cards.

P2 readiness documentation:

- Lock this readiness gate and fixture matrix before cue-tier behavior changes.
- Add a lab note for the exact P2 fixture/corpus checklist.

P2a JSON cue-tier integration:

- Convert promoted policy entries to `AGENT_REVIEW_CUE` in `cueCards[]`.
- Keep muted policy entries in `suppressedCues[]`.
- Preserve P1 policy output and suppressed diagnostics.
- Keep Markdown rendering unchanged.

P2b renderer wording:

- Add default Markdown wording for `service-operation-sibling` cues.
- Keep muted policy entries hidden by default.
- Add renderer regression tests for the allowed and disallowed wording.

P2c public verification:

- Completed for beta.50. The installed public package rendered the
  `Review related service operation` cue, cited the policy evidence path,
  preserved `heuristic-review` / `AGENT_REVIEW_CUE`, hid muted policy entries,
  and kept the cue body free of reuse/equivalence/safety/action-forcing
  wording.
- Evidence note:
  `docs/lab/wt23-beta50-service-operation-markdown-verification-2026-05-14.md`.

P3 corpus calibration:

- Run on at least one service-heavy app and one library corpus.
- Record false positives before adding mutation families or signature-weighted
  promotion.
- Use the worksheet in
  `docs/lab/wt23-service-operation-corpus-calibration-plan-2026-05-16.md` to
  record corpus shape, command route, evidence availability, aggregate counts,
  sampled promoted cues, false positives, missed useful siblings, and the next
  policy decision.
- Treat fixture results as regression coverage, not corpus evidence.
- Keep mutation-family cues, signature weighting, and global threshold changes
  blocked until a calibration report explicitly supports them.
- The first maintainer-run corpus report,
  `docs/lab/wt23-service-operation-corpus-calibration-2026-05-16.md`, used
  VNplayer-main and hono-main. It found zero service-operation cue cards in the
  current CLI route because normalized `names[]` intent declarations do
  not preserve owner-file locality. Owner-aware controls produced useful Hono
  helper siblings, while VNplayer's relevant repository operations were mostly
  nested inside `createRepository()` and therefore outside the current
  `defIndex` service-operation input surface. This keeps WT-23 in corpus
  calibration rather than policy expansion.
- The owner-locality rerun,
  `docs/lab/wt23-owner-locality-corpus-rerun-2026-05-16.md`, used the same
  corpus with structured `names[]` owner files preserved through the CLI path.
  Hono rendered five useful review-only service-operation cue cards and no
  reviewed service-operation false positives. VNplayer still rendered zero
  service-operation cue cards because the useful repository operations remain
  nested inside `createRepository()` and outside the current candidate surface.
  This proves owner-locality transport but keeps mutation families, signature
  weighting, and nested local operation surfaces as separate follow-up designs.
- The nested local operation follow-up is specified in
  `docs/spec/pre-write-nested-service-operation-surface.md`. That design keeps
  VNplayer-style repository factory operations out of `defIndex`, dead-export
  ranking, `SAFE_FIX`, and `EXISTS` paths, and permits only review-only
  service-operation evidence after explicit local-surface fixtures pass.

## 13. Open Questions

- Should mutation families ever promote to review, or should they remain muted
  until signature facts are available?
- Should class-method service siblings be handled by this policy, or should
  `class-method-name` stay separate?
- Should signature facts become required for cross-file promotion once
  `function-clones.json` signature coverage is available?
- Should nested local functions inside service/repository factories have a
  separate review-only operation surface, without entering dead-export
  `defIndex`? Proposed answer:
  `docs/spec/pre-write-nested-service-operation-surface.md`.

# Pre-Write Nested Service Operation Surface

> **Role:** maintainer-facing design spec for exposing nested local service
> operations as pre-write review evidence without expanding dead-export
> candidates.
> **Status:** SPEC.
> **Last updated:** 2026-05-16

---

## 1. Problem

WT-23 owner-locality corpus rerun proved that the service-operation sibling
policy can produce useful review-only cues when candidate operations are already
visible in `symbols.json.defIndex`. Hono rendered five useful cues through the
normal CLI route.

VNplayer still rendered zero service-operation cues. The owner file was present,
but the relevant repository operations are nested inside `createRepository()`:

```text
apps/server/src/repository.ts
  createRepository()
    getWorld()
    getSession()
    getCurrentTurn()
    listLibraryDocs()
    createWorld()
```

Those local functions are not exports, so they are intentionally absent from
`defIndex`. Adding them to `defIndex` would be wrong: `defIndex` participates in
dead-export and public-surface reasoning, while these nested functions are only
pre-write review evidence.

The next slice therefore needs a separate surface.

## 2. Goals

- Expose selected nested local functions as pre-write review evidence.
- Keep the evidence out of dead-export ranking, `SAFE_FIX`, public API, and
  absence claims.
- Preserve the WT-23 proof discipline: review cue only, no reuse proof, no
  equivalence claim, no threshold relaxation.
- Make the surface explicit in artifacts so readers can see when nested local
  operation evidence was available or unavailable.
- Calibrate first on VNplayer-style repository factories before broadening to
  more patterns.

## 3. Non-Goals

- Do not add nested local functions to `symbols.json.defIndex`.
- Do not classify nested local functions as exports, dead exports, public API,
  or safe actions.
- Do not promote mutation-family cues in this slice.
- Do not add signature-weighted promotion in this slice.
- Do not add a generic "all nested functions" search surface.
- Do not inspect function bodies for semantic equivalence.
- Do not make the service-operation cue mandatory or blocking.

## 4. Proposed Artifact Surface

Add a new optional artifact section, either as a standalone artifact or as a
section under `symbols.json`, named:

```text
preWriteLocalOperationIndex
```

Recommended standalone artifact if implementation complexity grows:

```text
pre-write-local-operations.json
```

The surface is intentionally not named `defIndex`.

Minimum shape:

```json
{
  "schemaVersion": "pre-write-local-operations.v1",
  "status": "complete",
  "meta": {
    "supports": {
      "nestedLocalOperationIndex": true
    }
  },
  "byOwnerFile": {
    "apps/server/src/repository.ts": [
      {
        "identity": "apps/server/src/repository.ts::createRepository#getWorld",
        "name": "getWorld",
        "ownerFile": "apps/server/src/repository.ts",
        "containerName": "createRepository",
        "containerKind": "function-declaration",
        "scopeKind": "nested-function",
        "matchedField": "preWriteLocalOperationIndex",
        "line": 842,
        "operationFamily": "read-query",
        "domainTokens": ["world"],
        "visibility": "local-only",
        "eligibleForDeadExportRanking": false,
        "eligibleForSafeFix": false
      }
    ]
  },
  "summary": {
    "ownerFileCount": 1,
    "operationCount": 1
  }
}
```

Completeness semantics are part of the contract:

```json
{
  "status": "complete | unavailable | not-run",
  "reason": "no-js-symbol-facts | no-pre-write-intent | extraction-disabled"
}
```

`unavailable` means the producer ran but could not produce trustworthy local
operation facts from the current inputs, such as missing JS symbol facts or a
missing pre-write intent owner file. `not-run` means the producer or section was
not executed for this invocation, such as an explicit extraction disablement or
a mode that does not request the local operation surface.

An empty `byOwnerFile` map is proof only when `status: "complete"`. If the
section is `unavailable` or `not-run`, consumers must behave exactly as they do
today and must not treat the missing local surface as absence evidence.

When this artifact is required by a pre-write invocation but has
`status: "unavailable"` or `status: "not-run"`, it should participate in the
same `evidenceAvailability` reporting path as other pre-write artifacts. The
reader should see that local operation evidence was not grounded.

The standalone artifact is the preferred implementation if the section grows
large or if only pre-write needs it. Embedding the section in `symbols.json` is
acceptable only if it preserves the same status and safety fields.

V1 `containerKind` is a closed enum:

```text
function-declaration
const-function-expression
const-arrow-function
```

Additional container kinds require a spec update because they may change scope
and identity semantics.

Producer-side `domainTokens[]` are advisory cache facts. The canonical token
policy remains the consumer policy in `lookupName()` so token normalization,
operation-family grouping, and future calibration stay in one place. If the
producer and consumer disagree, the consumer policy decides promotion and the
artifact token list is diagnostic metadata.

## 5. Eligibility Rules

V1 should be narrow.

Eligible nested local operations:

- function declarations or const-assigned function expressions inside a
  top-level exported factory/service function;
- nested functions whose names begin with a known service-operation verb;
- nested functions in the same `ownerFile` as the pre-write intent;
- nested functions with at least one domain token shared with the intent;
- nested functions whose containing function has a service/factory/repository
  shape, such as `createRepository`, `makeRepository`, `createService`,
  `makeService`, or `buildService`.

Ineligible in V1:

- anonymous callbacks;
- inline event handlers;
- test helper closures unless test files are explicitly included and the
  intent owner file is a test file;
- generated, bundled, vendor, dist, build, coverage, or policy-excluded paths;
- nested functions in files outside the intent owner file;
- nested functions inside class methods; those remain a class-method surface
  question;
- mutation-family operations unless a future corpus report enables that family.

Rejecting too many candidates is acceptable. Admitting generic nested helpers is
not.

## 6. Candidate Identity

Nested local identities must include the owner file and container path:

```text
<ownerFile>::<containerName>#<localName>
```

Examples:

```text
apps/server/src/repository.ts::createRepository#getWorld
apps/server/src/repository.ts::createRepository#listLibraryDocs
```

If nested containers become necessary later, extend the identity with a stable
container path:

```text
<ownerFile>::<outer>#<inner>#<localName>
```

Do not use name-only keys.

If two local functions have the same name inside the same container, the
implementation must add a deterministic discriminator such as a declaration
line or ordinal:

```text
<ownerFile>::<containerName>#<localName>@line:<line>
```

The discriminator is identity metadata only. It must not imply export identity,
public API identity, or safe-action identity.

The entry `line` field is always declaration metadata. It becomes part of the
identity only when needed to disambiguate duplicate local names in the same
container.

## 7. Policy Integration

`lookupName()` may consume `preWriteLocalOperationIndex` alongside `defIndex`
and `classMethodIndex`, but it must keep the candidate lane distinct:

```json
{
  "matchedField": "preWriteLocalOperationIndex",
  "surfaceKind": "nested-local-operation"
}
```

Consumers must check `preWriteLocalOperationIndex.status` before using an empty
local operation result as evidence. Only `status: "complete"` can mean "no local
operations were observed"; `unavailable` and `not-run` mean "no grounded local
operation evidence was available."

The first policy slice must not feed nested candidates into
`serviceOperationSiblingPolicy.promoted[]`. That object is already wired to the
cue-tier adapter, so doing so would change `cueCards[]` and Markdown before the
local-operation cue contract is separately verified.

P2a therefore records nested candidates in a distinct lookup evidence surface:

```json
{
  "localOperationSiblingPolicy": {
    "policyId": "prewrite-local-operation-sibling",
    "policyVersion": "prewrite-local-operation-sibling-v1",
    "status": "complete",
    "promoted": [
      {
        "matchedField": "preWriteLocalOperationIndex",
        "surfaceKind": "nested-local-operation"
      }
    ],
    "muted": []
  }
}
```

This separate surface may promote nested candidates only when the local-operation
gates pass:

- operation family is read/query;
- shared domain token exists;
- locality is same file;
- path is not generated or policy-excluded;
- signature is compatible or unavailable, not incompatible.

The normal `nearNames[]` and `semanticHints[]` arrays should remain unchanged in
the first implementation slice. Local-operation candidates should enter only the
separate `localOperationSiblingPolicy` evidence object, never the formal name
lookup result lanes and never the existing `serviceOperationSiblingPolicy`
object.

The P2b cue-integration slice renders promoted nested candidates as:

```text
AGENT_REVIEW_CUE
evidenceLane: local-operation-sibling
confidence: heuristic-review
claim: related local service operation
```

The cue text should identify the container:

```text
Review related local service operation: `getWorld` inside
`createRepository` in `apps/server/src/repository.ts`.
```

## 8. Muted Evidence

In P2a, muted nested candidates remain in
`lookups[].localOperationSiblingPolicy.muted[]`, not `suppressedCues[]` or
`cueCards[]`. In P2b, muted local-operation policy entries are mirrored into
`suppressedCues[]` with `evidenceLane: "local-operation-sibling"` for JSON
readers, while default Markdown still hides them. Promoted entries render only
as review cues.

Recommended mute reasons:

```text
service-sibling-local-operation-unsupported-container
service-sibling-local-operation-domain-mismatch
service-sibling-local-operation-operation-family-mismatch
service-sibling-local-operation-generated-or-policy-excluded
service-sibling-local-operation-insufficient-suppressed-support
```

The policy must preserve:

```json
{
  "containerName": "createRepository",
  "surfaceKind": "nested-local-operation",
  "matchedField": "preWriteLocalOperationIndex",
  "eligibleForDeadExportRanking": false,
  "eligibleForSafeFix": false
}
```

## 9. Safety Invariants

These invariants are mandatory:

1. The producer emits nested local operations only into
   `preWriteLocalOperationIndex`; it must not also mirror them into
   `symbols.json.defIndex`.
2. Nested local operations never appear in `dead-classify.json` as export
   candidates.
3. Nested local operations never appear in `export-action-safety.json` as
   `SAFE_FIX`, `REVIEW_FIX`, `DEGRADED`, or `MUTED` export actions.
4. The cue-tier adapter never creates `SAFE_CUE`, `EXISTS`, or `SAFE_FIX` from
   nested local operation evidence.
5. The renderer never says "reuse", "equivalent", "safe", "exists", "should
   call", or "blocking failure" for nested local operation cues.
6. Absence of a nested local operation is not deadness evidence.
7. If the nested local operation artifact is unavailable, pre-write behaves as
   it does today and records no false absence claim.
8. Nested local operations never appear in `lookups[].nearNames[]` or
   `lookups[].semanticHints[]`.
9. Nested local operations never appear in `classMethodIndex`.

Invariant 6 applies to complete local-operation evidence. Invariant 7 applies
when `status !== "complete"`. Future nested local operation cues, if enabled,
should reuse the existing service-operation sibling review adapter;
implementations must not add a new nested-operation adapter that calls
`safeCue()`.

## 10. Acceptance Fixtures

### Positive Fixture

```ts
export function createRepository() {
  function getWorld(id: string) {
    return db.world.find(id);
  }

  function listLibraryDocs(worldId: string) {
    return db.docs.list(worldId);
  }

  return { getWorld, listLibraryDocs };
}
```

Intent:

```json
{
  "names": [
    {
      "name": "searchWorld",
      "kind": "function",
      "ownerFile": "apps/server/src/repository.ts"
    }
  ]
}
```

Expected:

- `preWriteLocalOperationIndex.byOwnerFile["apps/server/src/repository.ts"]`
  includes `getWorld`.
- `lookupName("searchWorld")` records `getWorld` in
  `localOperationSiblingPolicy.promoted[]` with
  `matchedField: "preWriteLocalOperationIndex"` and
  `surfaceKind: "nested-local-operation"`.
- P2a leaves `serviceOperationSiblingPolicy.promoted[]`, `cueCards[]`, and
  Markdown output unchanged.
- P2b renders the promoted local-operation candidate as a review-only
  `AGENT_REVIEW_CUE` with `evidenceLane: "local-operation-sibling"`.
- No `defIndex`, dead-export, or safe-action output changes.

### Negative Fixture: Generic Helper

```ts
export function createRepository() {
  function normalizeInput(value: string) {
    return value.trim();
  }

  return { normalizeInput };
}
```

Intent `searchWorld` must not render `normalizeInput` as a service-operation
cue.

### Negative Fixture: Mutation Family

```ts
export function createRepository() {
  function deleteWorld(id: string) {
    return db.world.delete(id);
  }

  return { deleteWorld };
}
```

Intent `searchWorld` must keep `deleteWorld` muted. Mutation-family promotion
remains blocked.

### Negative Fixture: Export Safety

A nested `getWorld` candidate must not create or alter any entry in
`dead-classify.json`, `export-action-safety.json`, or `fix-plan.json`.

## 11. Implementation Slices

P0 spec only:

- Land this design.
- Keep WT-23 tracker pointed at this surface as the next implementation
  candidate.

P1 artifact-only prototype:

- Emit `pre-write-local-operations.json` or `symbols.preWriteLocalOperationIndex`.
- Add fixtures proving positive and negative candidate extraction.
- Do not connect to cue cards yet.

P2 policy integration:

- P2a: let `lookupName()` consume the local operation surface into a separate
  `localOperationSiblingPolicy` evidence object.
- Preserve `matchedField: "preWriteLocalOperationIndex"`.
- Promoted entries carry stable local-operation `supportingReasons[]`, starting
  with `local-operation-same-file-domain-overlap`, so renderer output does not
  fall back to `unknown`.
- Keep `serviceOperationSiblingPolicy` unchanged.
- P2b: add `suppressedCues[]`, `cueCards[]`, and Markdown tests before rendering
  local-operation cues. The renderer must use `Review related local service
  operation`, cite
  `pre-write-advisory.json / lookups[].localOperationSiblingPolicy.promoted`,
  and keep muted local-operation details hidden by default.

P3 corpus rerun:

- Rerun VNplayer owner-locality corpus.
- Record whether `searchWorld`, `lookupSession`, and `findTurn` now receive
  useful review-only service-operation cues.
- Do not enable mutation-family or signature-weighted promotion from P3 unless
  a separate calibration report supports it.

## 12. Open Questions

- Should const-assigned object method values inside factory return objects be
  part of this surface, or only named local function declarations?
- Should same-directory nested local operations ever promote, or should V1
  require same-file locality only?
- Should local operation extraction live in the symbol graph producer or a
  pre-write-only producer?
- Should this artifact be emitted in full audits, pre-write-only runs, or only
  when `--pre-write` is requested?

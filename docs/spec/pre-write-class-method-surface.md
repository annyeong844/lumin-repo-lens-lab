# Pre-Write Class Method Surface

> **Role:** maintainer-facing design spec for making pre-write name search see
> TypeScript/JavaScript class methods without polluting export/deadness evidence.
> **Status:** SPEC.
> **Last updated:** 2026-05-09

---

## 1. Problem

`pre-write` currently looks for planned names mostly through
`symbols.json.defIndex`. That index is intentionally top-level/export oriented:
it supports dead-export, fan-in, canonical ownership, and public surface work.

This misses a large class of real reuse candidates in object-oriented
codebases. In the reported `ai-project-planner` case, an intent for
`handleBulkDelete` should have surfaced an existing `handleDelete` method in
`event-dispatcher.ts`. Instead, pre-write returned unrelated top-level
`handle*` candidates such as `handleApiError`, `handleSignIn`, and
`handleSignUp`.

Root cause:

```text
class TaskControlEventDispatcher {
  handleDelete(...) { ... }
}
```

The class and instance binding are indexed, but methods such as
`handleDelete` and `handleRegenerate` are not part of the pre-write search
surface. In class-heavy repositories, pre-write can therefore say
`NOT_OBSERVED` while the relevant method exists.

This is not primarily a parser problem. The measured report says parsing
finishes in about 1.5s; the expensive portion is traversal/classification and
candidate scoring in Node. A separate measurement compared Lumin at about 30s
with `fallow` at about 0.3s for the same style of search. Treat that as a
performance warning, not yet as a benchmark acceptance number.

## 2. Goals

- Add a pre-write search surface for class methods.
- Keep class methods out of `defIndex` unless they are actually exported
  top-level definitions.
- Surface method matches as review cues, not deadness or public API proof.
- Report an honest blind zone when method search is unavailable.
- Improve name search so weak common prefixes such as `handle`, `create`, and
  `get` do not crowd out stronger domain tokens such as `delete`.
- Keep the first implementation bounded to TS/JS class methods and class-field
  function properties.

## 3. Non-Goals

- Do not use class methods as `SAFE_FIX`, dead-export, fan-in, or public-surface
  evidence.
- Do not infer semantic equivalence.
- Do not index every object-literal method in v1.
- Do not introduce embeddings, synonym dictionaries, or broad semantic search.
- Do not rewrite the engine in Rust as part of this feature.
- Do not make `handleDelete` a grounded reuse command. It is a cue for an agent
  to inspect.

## 4. Evidence Contract

Allowed claims:

```text
Existing class method `TaskControlEventDispatcher.handleDelete` was found.
Name tokens overlap with the planned `handleBulkDelete` intent.
```

Allowed review wording:

```text
Inspect `event-dispatcher.ts:481` before creating `handleBulkDelete`.
```

Disallowed claims:

```text
This method does the same thing.
Reuse is safe.
No class method exists.
```

If method evidence is absent because the producer did not emit a method index,
pre-write must say that class-method search is unavailable. It must not phrase
the result as a grounded absence.

## 5. Artifact Shape

Preferred artifact:

```text
pre-write-member-index.json
```

Rationale: class methods are a search/review surface, not export ownership.
Keeping them out of `symbols.json.defIndex` prevents accidental reuse by
dead-export and ranking logic.

Suggested shape:

```json
{
  "meta": {
    "schemaVersion": "pre-write-member-index.v1",
    "producer": "build-pre-write-member-index.mjs",
    "language": "ts-js",
    "methodSearchPolicyVersion": "class-method-search-policy-v1",
    "supports": {
      "classMethods": true,
      "classFieldFunctionProperties": true,
      "objectLiteralMethods": false,
      "privateNames": true
    },
    "performance": {
      "parseMs": 1500,
      "traversalMs": 28500,
      "candidateScoringMs": 0
    }
  },
  "methods": [
    {
      "identity": "src/event-dispatcher.ts::TaskControlEventDispatcher#handleDelete",
      "ownerFile": "src/event-dispatcher.ts",
      "className": "TaskControlEventDispatcher",
      "methodName": "handleDelete",
      "memberKind": "method",
      "visibility": "public",
      "static": false,
      "computed": false,
      "line": 481,
      "endLine": 520,
      "loc": 40,
      "signatureText": "handleDelete(event: DeleteEvent): void",
      "signatureHash": "sha256:..."
    }
  ],
  "diagnostics": []
}
```

`signatureText` and `signatureHash` are optional in v1, but the fields should be
reserved now because standalone-function vs class-method signature matching is
one of the reported gaps.

## 6. Extraction Rules

Supported in v1:

- `MethodDefinition` / `MethodDeclaration`.
- `PropertyDefinition` / class fields initialized with arrow functions or
  function expressions.
- `static` methods.
- private names such as `#handleDelete`, recorded with `visibility: "private"`
  and the display name `#handleDelete`.

Deferred:

- Object literal methods.
- Decorator semantics beyond preserving a diagnostic field.
- Inheritance-aware method override resolution.
- Call graph or fan-in for methods.
- Semantic body comparison.

Exclusions:

- Generated/vendor/policy-excluded files should not appear in default Markdown.
- Computed names with no stable string name should be recorded in diagnostics,
  not default-surfaced as method candidates.
- Constructor methods should be excluded from name reuse search unless a future
  feature specifically needs constructor cues.

## 7. Pre-Write Lookup Integration

Add a method-aware lookup lane after normal name lookup:

```text
intent name -> defIndex exact/near search
            -> method index exact/near/token search
            -> cue-tier adapter
```

Method hits must produce `AGENT_REVIEW_CUE`, not `SAFE_CUE`.

Suggested cue:

```json
{
  "cueTier": "AGENT_REVIEW_CUE",
  "evidenceLane": "class-method-name",
  "claim": "existing class method with related name tokens",
  "grounding": "method-index-review",
  "evidence": [
    {
      "artifact": "pre-write-member-index.json",
      "matchedField": "methods[].methodName",
      "identity": "src/event-dispatcher.ts::TaskControlEventDispatcher#handleDelete",
      "matchedTokens": ["delete"],
      "policyVersion": "class-method-search-policy-v1"
    }
  ]
}
```

If `pre-write-member-index.json` is missing and the intent contains a
function/helper-like planned name, emit:

```json
{
  "evidenceLane": "class-method-search",
  "status": "UNAVAILABLE",
  "reason": "member-index-missing",
  "artifact": "pre-write-member-index.json"
}
```

This should also feed `evidenceAvailability` so `NOT_OBSERVED` cannot be read as
grounded class-method absence.

## 8. Candidate Ranking Policy

The current near-name path overweights shallow edit distance and common
prefixes. Method search needs a named policy:

```json
{
  "policyId": "class-method-search-policy",
  "policyVersion": "class-method-search-policy-v1",
  "weakVerbTokens": ["handle", "create", "get", "set", "make", "load", "update"],
  "requiredStrongTokenOverlap": 1,
  "maxDefaultReviewCues": 5
}
```

Ordering should be deterministic:

1. exact method name;
2. strong token overlap count;
3. same file or same directory as `refactorSources` / planned file when known;
4. signature hash match when available;
5. edit distance;
6. owner file;
7. class name;
8. method name.

Important regression target:

```text
handleBulkDelete -> handleDelete should outrank handleApiError,
handleSignIn, and handleSignUp because `delete` is a strong domain token while
`handle` alone is weak.
```

For `deleteMultipleTasks`, `handleDelete` may be a review cue when `delete`
normalization is shared. It must not be a grounded semantic match.

## 9. Blind-Zone Reporting

Class-method search must become a self-reported pre-write capability.

Recommended surfaces:

- `pre-write-advisory.latest.json.unavailableEvidence[]`
- `pre-write-advisory.latest.json.evidenceAvailability`
- optional manifest summary when invoked through `audit-repo --pre-write`

Example wording:

```text
Class method search unavailable: pre-write may miss OO method reuse candidates.
Run with a member-index-capable baseline before treating method absence as
grounded.
```

This gap is different from Python method-resolution limits. It applies to
TS/JS class methods and should be tracked separately.

## 10. Performance Contract

This feature should not hide cost behind broad full-audit behavior.

Record timings by phase when possible:

```json
{
  "parseMs": 1500,
  "traversalMs": 28500,
  "candidateScoringMs": 120,
  "methodCount": 2140,
  "fileCount": 135
}
```

Use the reported `Lumin ~30s` vs `fallow ~0.3s` measurement as motivation for
profiling, not as a release blocker. Before considering Rust/rayon, the engine
should measure:

- parser time;
- AST traversal time;
- normalization time;
- candidate scoring time;
- JSON serialization time;
- cold vs warm artifact reuse.

Rust/rayon may be justified later if profiling proves traversal/classification
dominates and incremental caching cannot bring warm pre-write latency into an
agent-loop budget. It should not be used to mask an incomplete evidence model.

## 11. Implementation Phases

### P0: Spec And Fixtures

- Keep this spec as the contract.
- Add a minimal fixture with a class containing `handleDelete`,
  `handleRegenerate`, and unrelated `handle*` methods.
- Add a regression intent for `handleBulkDelete`.

### P1: Member Index Producer

- Add `build-pre-write-member-index.mjs`.
- Emit `pre-write-member-index.json`.
- Support TS/JS class methods and class-field function properties.
- Record method search capability and timing metadata.

### P2: Pre-Write Lookup Lane

- Add method-index loading and cold-cache production.
- Add class-method review cues.
- Add unavailable evidence when the artifact is missing.
- Keep `defIndex` behavior unchanged.

### P3: Ranking And Noise Calibration

- Add `class-method-search-policy-v1`.
- Downweight weak common verb tokens.
- Add regression tests proving `handleDelete` outranks unrelated `handle*`
  prefix matches for delete intents.

### P4: Signature Bridge

- Use method signatures to compare standalone helper intents against class
  methods.
- Keep signature matches as review cues unless another proof lane justifies a
  stronger claim.

### P5: Performance Profiling

- Add timing metadata and corpus measurements.
- Decide whether Node optimization, incremental cache reuse, worker threads, or
  Rust/rayon is justified by measured bottlenecks.

## 12. Acceptance Criteria

- A TS class fixture indexes `handleDelete` as a method candidate.
- `handleBulkDelete` surfaces `handleDelete` as an `AGENT_REVIEW_CUE`.
- Unrelated `handleApiError`, `handleSignIn`, and `handleSignUp` do not outrank
  `handleDelete` when the intent includes delete semantics.
- Missing `pre-write-member-index.json` emits `UNAVAILABLE`, not grounded
  absence.
- `defIndex` remains unchanged for class methods.
- Dead-export and SAFE_FIX ranking do not consume method-index records.
- Renderer wording never claims semantic equivalence, safe reuse, or no method
  exists.
- Policy metadata records token/ranking thresholds.
- Performance metadata separates parse, traversal, scoring, and serialization
  time.

## 13. Open Questions

- Should v1 include private methods in default Markdown, or keep them JSON-only
  unless the planned edit is in the same file?
- Should method search be always produced in quick profile, or only when
  `pre-write` intent names look function/helper-like?
- Which token normalizations are safe beyond obvious morphology such as
  `delete` / `deletion`?
- Should class method signature facts live in `pre-write-member-index.json` or a
  shared function-signature artifact later?

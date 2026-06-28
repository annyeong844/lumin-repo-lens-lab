# Vitest Class Method Pre-Write Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-class-method-index-prototype-names.mjs`
> - `tests/test-class-method-prewrite-surface.mjs`

---

## Purpose

This review decides whether the class-method pre-write suites can move together
as one narrow Lane C Vitest mirror batch. It does not add the Vitest suites.

The batch is acceptable because both suites protect the same pre-write method
surface boundary:

- class methods must be indexed for pre-write review cues;
- prototype-named methods must not crash the class method index;
- class methods must not be promoted into export `defIndex` or dead-export
  proof;
- class method hints must stay review-only and must not relax cue-tier,
  semantic, or Markdown wording policy.

This batch exists because real TypeScript projects often place meaningful
operations on class methods. If the mirror loses that edge case, pre-write can
again fail to surface relevant reuse-review cues in OO codebases, or crash on
ordinary JavaScript prototype names.

## Reviewed Evidence

| Suite                                               | Preserved Node Command                                   | Proposed Focused Vitest Command                          | Surface Under Review                             |
| --------------------------------------------------- | -------------------------------------------------------- | -------------------------------------------------------- | ------------------------------------------------ |
| `tests/test-class-method-index-prototype-names.mjs` | `node tests/test-class-method-index-prototype-names.mjs` | `npm run test:vitest:class-method-index-prototype-names` | class method index dictionary safety             |
| `tests/test-class-method-prewrite-surface.mjs`      | `node tests/test-class-method-prewrite-surface.mjs`      | `npm run test:vitest:class-method-prewrite-surface`      | class method pre-write lookup review cue surface |

Current Node evidence checked for this review:

```text
node tests/test-class-method-index-prototype-names.mjs # 2 passed, 0 failed
node tests/test-class-method-prewrite-surface.mjs      # 5 passed, 0 failed
```

Goal lane: Lane C, pre/post-write lifecycle. This review covers only the class
method pre-write surface subset of that lane.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add both focused mirrors together because they
share the same class-method pre-write boundary. The mirror must keep every Node
entrypoint runnable and must not turn class method review evidence into
existence proof, deadness proof, or action-safety proof.

## Protected Invariants

The future Vitest batch must preserve these class-method contracts:

- `symbols.json.meta.supports.classMethodIndex` advertises class method index
  support when the symbol artifact includes the class method surface.
- `symbols.classMethodIndex` records methods by owner file and method name while
  preserving class owner, method identity, line, static/computed/visibility, and
  member kind metadata.
- Prototype-named methods such as `constructor`, `toString`,
  `hasOwnProperty`, `valueOf`, and `__proto__` are stored as own dictionary
  entries and do not crash artifact generation.
- Class methods remain outside export `defIndex`; indexing a method for
  pre-write lookup must not make it an exported symbol or dead-export
  candidate.
- `lookupName()` can surface class methods as near-name review hints through
  `matchedField: "classMethodIndex"`.
- Domain-specific method matches such as `handleBulkDelete` -> `handleDelete`
  outrank unrelated shared-prefix methods such as `handleApiError`,
  `handleSignIn`, and `handleSignUp`.
- Class method hints stay diagnostic. They do not become `EXISTS`, `SAFE_FIX`,
  semantic equivalence, or cue-tier policy proof.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- A plain-object accumulator such as `{}` must not be able to treat
  `constructor`, `toString`, or `__proto__` as inherited prototype entries.
- A fixture helper must not hide prototype-name crashes by omitting the exact
  prototype method names.
- A class method fixture must include unrelated `handle*` siblings so the
  domain-specific `handleDelete` ordering remains observable.
- A class method fixture must prove the method is present in `classMethodIndex`
  and absent from export `defIndex`.
- The suite must not assert against Markdown rows. Rendering belongs to
  `tests/test-pre-write-render.mjs`.
- The suite must not assert against cue-card routing. Cue tiers belong to
  `tests/test-pre-write-cue-tiers.mjs`.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- The fixture boundary is in-memory symbol artifact construction for prototype
  method names plus a temporary TypeScript repo for pre-write lookup behavior.
- Shared helpers may create temporary roots, write fixture files, read JSON, and
  clean up directories.
- Shared helpers must not decide class-method identity, dictionary safety,
  export `defIndex` membership, near-name ordering, or cue-tier meaning.
- The mirror must not absorb `tests/test-pre-write-cue-tiers.mjs`,
  `tests/test-pre-write-render.mjs`, service-operation sibling policy tests,
  pre-write advisory artifact tests, resolver behavior, deadness/ranking,
  performance/incremental cache identity, or public package verification.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/class-method-index-prototype-names.test.mjs`,
2. `tests/class-method-prewrite-surface.test.mjs`,
3. `npm run test:vitest:class-method-index-prototype-names`,
4. `npm run test:vitest:class-method-prewrite-surface`,
5. candidate-board updates moving the two suites from `REVIEWED` to `DONE`.

The implementation PR should keep the current Node assertion groups represented
as named Vitest `it(...)` blocks. It may share setup-only temp-root and
JSON-read helpers inside test files, but no shared helper should decide method
identity, prototype-name handling, export membership, near-name ranking, or cue
policy.

Run the preserved Node commands and focused Vitest commands when changing this
batch. Also run `npm run test:vitest`, doc-script checks, and formatting checks
so the reviewed runner discovery boundary and wiki references stay current.

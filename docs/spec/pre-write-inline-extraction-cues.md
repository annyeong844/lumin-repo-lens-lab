# Pre-Write Inline Extraction Cues

> **Role:** maintainer-facing design spec for detecting repeated inline code
> patterns before an agent extracts a helper.
> **Status:** P1/P2 implemented; P3 deferred.
> **Last updated:** 2026-05-09

---

## 1. Problem

The pre-write gate currently answers a reuse question:

```text
Is the name, file, dependency, or type shape I am about to add already present?
```

That is useful for preventing duplicate helpers such as a second
`formatTimestamp`. It does not answer the opposite refactoring question:

```text
Do repeated inline statements already exist, making this extraction worth
reviewing?
```

Example from `web-shell-bench`:

```ts
try {
  writeWebSocketTextMessage(connection.socket, payload);
} catch {
  connection.socket.destroy();
}
```

The planned helper was `writeOrDestroyConnection`, motivated by repeated
`catch { connection.socket.destroy(); }` patterns. The current pre-write lookup
reported:

- planned helper name not observed
- planned result type not observed
- planned file is new
- planned result shape not observed

All of those claims were true. The missing evidence was that an inline
statement pattern already appeared several times in the source file. That is a
different evidence lane, not a failure of name lookup.

## 2. Goals

- Add an evidence lane for repeated inline statement/block patterns that can
  support helper extraction review.
- Keep the claim narrow: repeated syntax exists; semantic equivalence is not
  proven.
- Integrate with pre-write cue tiers as `AGENT_REVIEW_CUE`, not `SAFE_CUE`.
- Prefer AST-normalized hashes and explicit refactor-source intent over broad
  name/token heuristics.
- Keep the first implementation small enough for agent-loop use.

## 3. Non-goals

- Do not infer semantic equivalence.
- Do not use embeddings, synonym dictionaries, or intent-word expansion.
- Do not automatically recommend extraction or block writing.
- Do not treat every repeated `catch`, `return`, or logging statement as useful.
- Do not make function-clone evidence responsible for statement-level clones.
  Function clones remain top-level helper/function evidence.

## 4. Definitions

**Inline extraction cue:** Evidence that a repeated statement or small block
already exists and may justify extracting a helper.

**Pattern occurrence:** A concrete source range matching a normalized inline
pattern.

**Pattern group:** A set of at least `minOccurrences` occurrences with the same
normalized pattern hash.

**Refactor source:** Optional intent-side evidence naming the file/ranges the
agent plans to extract from.

## 5. Evidence Contract

Inline extraction cues are review cues only.

Allowed claim:

```text
Repeated inline statement pattern found 4 times in src/server.ts.
```

Disallowed claims:

```text
These statements do the same thing.
You should extract this helper.
The planned helper is safe to add.
```

The cue tier should be:

```json
{
  "cueTier": "AGENT_REVIEW_CUE",
  "evidenceLane": "inline-extraction",
  "claim": "repeated inline statement pattern",
  "grounding": "ast-normalized-review"
}
```

If the pattern is too generic, generated-only, policy-excluded, or below
threshold, it should become `MUTED_CUE` or remain absent from default Markdown.

## 6. Producer Shape

The preferred artifact is separate from `function-clones.json`:

```text
inline-patterns.json
```

Suggested shape:

```json
{
  "meta": {
    "schemaVersion": "inline-patterns.v1",
    "normalizerVersion": "inline-statement-normalizer-v1",
    "minOccurrences": 3,
    "maxPatternStatements": 2,
    "supports": {
      "catchBlockPatterns": true,
      "statementSequencePatterns": false
    }
  },
  "groups": [
    {
      "patternHash": "sha256:...",
      "kind": "catch-block",
      "size": 4,
      "ownerFiles": ["src/server.ts"],
      "normalizedPattern": "catch { <member>.socket.destroy(); }",
      "occurrences": [
        {
          "file": "src/server.ts",
          "line": 498,
          "endLine": 500,
          "enclosingFunction": "handleClientSocketData"
        }
      ],
      "reviewReason": "same normalized catch block; verify control-flow and socket ownership before extracting"
    }
  ],
  "mutedGroups": []
}
```

The artifact should be deterministic:

- Sort groups by size descending, then file/range identity.
- Sort occurrences by file, line, end line, enclosing function.
- Derive group ids from normalized pattern content, not discovery order.

## 7. Normalization

The v1 normalizer should be conservative.

Recommended supported patterns:

- `catch` block body with 1-2 statements.
- Statement sequence of 2-3 adjacent statements inside the same block.
- Member-call statements such as `connection.socket.destroy()`, with receiver
  identifiers anonymized but property names preserved.

Recommended exclusions:

- Single `return`, `throw`, `break`, or `continue`.
- Single generic logging calls.
- Blocks containing declarations with nontrivial initializers.
- Blocks containing `await`, `yield`, assignments to outer bindings, or
  mutation patterns that the v1 normalizer does not model.
- Generated/vendor/policy-excluded files by default.

This is not a purity proof. Exclusions keep the first lane low-noise.

## 8. Pre-Write Intent Integration

The current pre-write intent shape has no place to express extraction source.
Add an optional field:

```json
{
  "refactorSources": [
    {
      "file": "src/server.ts",
      "lines": [498, 577, 661, 689],
      "why": "extract repeated catch-destroy handling into writeOrDestroyConnection"
    }
  ]
}
```

Validation rules:

- `file` is required and repository-relative.
- `lines` is optional but must contain positive integers when present.
- `why` is optional but recommended for renderer wording.

Lookup rules:

- If `inline-patterns.json` is absent, report lane `UNAVAILABLE`, not
  `MUTED_CUE`.
- If `refactorSources` intersects a pattern group, emit an
  `AGENT_REVIEW_CUE`.
- If there is no `refactorSources`, default pre-write may surface only high
  support groups near the planned file/directory, capped and labeled as review.

## 9. Renderer Contract

Markdown should be explicit:

```text
Agent review cue: repeated inline catch-destroy pattern appears 4 times in
src/server.ts. Read inline-patterns.json and verify control-flow/socket
ownership before extracting writeOrDestroyConnection.
```

The renderer must not say:

```text
Duplicate behavior found.
Safe to extract.
Use this helper.
```

JSON should preserve structured evidence:

```json
{
  "candidate": {
    "identity": "inline-pattern:sha256:..."
  },
  "renderTier": "AGENT_REVIEW_CUE",
  "cues": [
    {
      "cueTier": "AGENT_REVIEW_CUE",
      "evidenceLane": "inline-extraction",
      "claim": "repeated inline statement pattern",
      "evidence": [
        {
          "artifact": "inline-patterns.json",
          "matchedField": "patternHash",
          "occurrenceCount": 4
        }
      ]
    }
  ]
}
```

## 10. Relationship To Existing Artifacts

`function-clones.json` remains focused on top-level exported and file-local helper/function
clone cues. It should not be stretched to cover arbitrary inline statements.

`inline-patterns.json` may reuse helper code for parsing, range handling, and
normalization, but the artifact contract is separate because:

- the unit of evidence is a statement/block, not a function declaration;
- the default cue tier is review, not grounded reuse;
- occurrence ranges are more important than exported identity.

## 11. Implementation Phases

### P0: Spec And Fixture

- Keep this spec as the contract.
- Add a small fixture based on repeated `catch { connection.socket.destroy(); }`
  patterns.

### P1: Producer

- Add `build-inline-pattern-index.mjs`.
- Emit `inline-patterns.json`.
- Support catch-block patterns first.
- Keep general short statement sequences deferred until a separate noise policy
  and fixture set exists.
- Implemented in v1.

### P2: Pre-Write Lane

- Add optional `intent.refactorSources`.
- Add pre-write lookup/classification for inline extraction cues.
- Render as `AGENT_REVIEW_CUE`.
- Implemented in v1. The default pre-write lane requires explicit
  `refactorSources`; nearby-file automatic surfacing remains deferred.

### P3: Summary / Review Pack

- Surface counts in `audit-summary.latest.md` and review pack as unranked cues.
- Keep full details in `inline-patterns.json`.

### P4: Block-Level Exact Groups

- Add a general exact block detector after the catch-block lane is stable.
- Candidate algorithm: tokenize normalized statement/block streams per file,
  use suffix-array/LCP or an equivalent deterministic repeated-subsequence
  index, then emit exact repeated block groups.
- This lane should find contiguous repeated statement blocks that are not whole
  functions and not limited to `catch` bodies.
- The claim remains narrow: exact normalized block repetition only.
- Default tier remains `AGENT_REVIEW_CUE`; no semantic-equivalence, safe
  extraction, or auto-fix claim.
- Thresholds such as minimum block length, minimum occurrence count, and output
  caps must be named policy fields with a versioned calibration note before
  changing default Markdown behavior.

## 12. Acceptance Criteria

- A fixture with four repeated `catch { connection.socket.destroy(); }` blocks
  produces one inline pattern group.
- Pre-write with matching `refactorSources` emits an `AGENT_REVIEW_CUE`.
- Pre-write without `inline-patterns.json` reports `UNAVAILABLE`.
- Generic single-statement patterns remain muted or absent from default
  Markdown.
- Renderer wording never claims semantic equivalence or safe extraction.
- Generated/vendor-only groups are not default-surface cues.
- All output ordering is deterministic.
- Future suffix-array/LCP block groups must be exact normalized block groups,
  not fuzzy similarity or intent-token expansion.

## 13. Open Questions

- Should v1 require explicit `refactorSources`, or may it show high-support
  inline groups near planned files without them? Answer: v1 requires explicit
  `refactorSources`.
- What is the first minimum threshold: 3 occurrences or 4?
- Which general statement-sequence families are low-noise enough to enable
  after catch-block patterns?
- For P4, what minimum block length and occurrence threshold keep suffix-array
  block groups useful without turning common boilerplate into default-surface
  noise?
- Should `post-write` later verify that planned repeated ranges were actually
  reduced after extraction?

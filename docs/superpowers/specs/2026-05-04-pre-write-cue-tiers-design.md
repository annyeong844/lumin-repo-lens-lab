# Pre-Write Cue Tiers Design

Date: 2026-05-04
Status: design approved for implementation planning

## Goal

Make pre-write warnings useful for AI coding agents without pretending that
weak name or token similarity proves reuse.

The pre-write gate should classify suspicious reuse signals into explicit cue
tiers:

- `SAFE_CUE`
- `AGENT_REVIEW_CUE`
- `MUTED_CUE`

This is a cue contract, not an auto-fix contract. A safe cue means the stated
evidence is mechanically true. It does not mean the existing code is
semantically equivalent to the planned code.

Cue tiers attach to individual evidence items, not necessarily to whole
candidates. One candidate may carry a grounded exact-signature cue, a
review-only near-name cue, and one or more muted weak-token cues at the same
time. JSON must preserve each cue separately so AI agents do not infer cue
strength from a candidate-level bucket or from prose.

## Problem

The current pre-write path has two different issues:

1. Strong evidence is valuable but hidden behind mixed wording.
   Exact symbol/file matches and exact function signature hashes are grounded
   facts. AI agents can use them directly as evidence.

2. Weak token hints can become noisy.
   A planned `createLogger` can surface `createStore` or
   `createJSONStorage` because `create` is a common verb. That is not a
   useful reuse cue. It should either require stronger supporting evidence or
   be muted with an explicit reason.

The design should avoid a growing semantic token dictionary. Without embeddings
or explicit structural evidence, the tool should not try to infer meaning from
names alone.

## Non-Goals

- Do not add `ast-grep` as a dependency or fallback in this slice.
- Do not add semantic embeddings.
- Do not expand intent-token heuristics into a large synonym dictionary.
- Do not change dead-export `SAFE_FIX` / `REVIEW_FIX` / `MUTED` semantics.
- Do not claim that same signature or same shape means same behavior.
- Do not hide all weak signals; preserve them in artifacts when useful for
  debugging or tuning.

## Terminology

### `SAFE_CUE`

The cue's claim is mechanically grounded and can be trusted by an AI coding
agent as a fact.

`SAFE_CUE` is safe only for the exact claim it states. It is not safe for
semantic equivalence, auto-reuse, auto-fix, or deletion claims. Default Markdown
should render this tier as `Grounded facts` rather than `Safe` to avoid
confusion with dead-export `SAFE_FIX`.

Examples:

- exact exported symbol exists
- exact file exists
- exact normalized body hash match
- exact type/shape hash match
- exact function signature hash match

Allowed wording:

> Existing function with the same normalized type signature was found.

Disallowed wording:

> Existing function does the same thing.

### `AGENT_REVIEW_CUE`

The cue is suspicious enough to ask an AI coding agent to inspect the cited
file or symbol before writing new code.

Examples:

- near-name match
- domain cluster match
- same function signature but distant name or owner domain
- weak token overlap plus at least one stronger supporting signal

Allowed wording:

> Review this candidate before creating a parallel helper.

Disallowed wording:

> Reuse this candidate.

### `MUTED_CUE`

The cue is likely noise and should not be shown in the default chat surface.

Examples:

- common verb token only, such as `create`, `get`, `set`, `make`, `load`
- stop-token-only match
- token creates too many candidates
- generated/vendor/policy-excluded candidate

Muted cues should remain visible in JSON as `suppressedCues` with a reason,
token list, and candidate count when practical. This keeps tuning transparent
without burdening the AI agent's default review surface.

### `UNAVAILABLE`

`UNAVAILABLE` is not a cue tier. It is a lane-level evidence status for a lane
that could not be evaluated, such as a missing artifact, malformed artifact,
unsupported normalizer input, parse blind zone, or resolver blind zone.

Unavailable lanes must be reported separately from `suppressedCues`. Suppressed
cues are known-but-muted signals; unavailable lanes were not evaluated.

## Evidence Lanes

Pre-write should classify cues by evidence lane before rendering.

### Exact Identity Lane

Exact symbol and exact file matches are `SAFE_CUE`.

Inputs:

- `symbols.json.defIndex`
- identity-keyed fan-in where available
- file inventory and topology metadata where available

Failure mode:

- missing artifacts or resolver blind zones downgrade to unavailable evidence,
  not weak semantic hints.

### Exact Structural Lane

Exact shape hash, function signature hash, and normalized function body hash
matches are `SAFE_CUE` for the exact claim they make.

Inputs:

- `shape-index.json`
- `function-clones.json`
- normalized hash/schema version fields

Failure mode:

- unsupported shape or signature normalization returns `UNAVAILABLE`.
- no heuristic grep or semantic fallback in this slice.

### Name Similarity Lane

Near-name matches are `AGENT_REVIEW_CUE` unless the only shared evidence is a
weak common token.

Examples:

- `useShallowFromState` near `useShallow`: review cue
- `MergeWithValuesV2` near `MergeWithValues`: review cue
- `createLogger` near `createStore`: muted if `create` is the only meaningful
  overlap

### Intent Token Lane

Intent-token hints are the riskiest lane and must be conservative.

Rules:

- Weak common verbs do not count as sufficient evidence by themselves.
- A single weak token match becomes `MUTED_CUE`.
- Weak token overlap can become `AGENT_REVIEW_CUE` only when supported by a
  second non-weak signal, such as owner-domain overlap, rare name token, exact
  file/domain cluster, or explicit shape/signature evidence.
- Token hints never become `SAFE_CUE`.

Weak-token suppression must use a small deterministic policy, not a semantic
synonym dictionary. JSON diagnostics should expose:

- `tokenizerVersion`
- `tokenPolicyVersion`
- matched tokens
- suppression reason
- candidate count

Initial tokenizer policy:

- split camel, snake, kebab, path, and digit boundaries deterministically;
- normalize case to lowercase;
- do not infer synonyms.

### Domain Cluster Lane

Domain cluster cues are `AGENT_REVIEW_CUE`.

They tell the AI agent that the planned file appears to belong near an existing
cluster. They do not prove a reusable function or type exists.

### Policy Exclusion Precedence

Policy exclusions are applied after evidence extraction but before default
rendering.

Generated, vendor, and policy-excluded candidates must not be promoted into
the default pre-write review surface solely because they have exact or near
evidence. Their evidence may remain visible in JSON diagnostics with a
policy-exclusion reason. Default Markdown should show them only when a caller
explicitly requests debug or policy-excluded output.

## Rendering Contract

The default Markdown output should separate cue kinds clearly:

1. `Already exists`
   - exact symbol/file identity
2. `Grounded facts`
   - exact body/type/signature hashes
3. `Agent review cues`
   - near names, domain clusters, supported weak-token hints
4. `Muted noise`
   - omitted from Markdown by default, present in JSON `suppressedCues`

Existing section names may remain if changing them is too disruptive, but the
JSON artifact should carry explicit cue tier fields so downstream AI agents do
not infer tier from prose.

Cue tiers do not by themselves change CLI exit behavior in this slice. This
design changes classification, artifact fields, and default rendering. Any
future blocking policy must be designed separately and must consume cue tiers
explicitly.

Rendering and suppression must be deterministic. Candidate ordering must not
depend on filesystem enumeration order, artifact insertion order, or JavaScript
object key iteration. If output is capped, the cap must use stable
tie-breakers:

1. evidence lane priority;
2. candidate owner file;
3. exported name or identity;
4. deterministic score or pair id.

## Artifact Shape

Each candidate should expose cue items. `renderTier` is only the renderer's
default placement. Downstream agents must inspect `cues[]`,
`suppressedCues[]`, and `unavailableEvidence[]` instead of treating
`renderTier` as the whole truth.

```json
{
  "candidate": {
    "identity": "src/shallow.ts::useShallow",
    "ownerFile": "src/shallow.ts",
    "exportedName": "useShallow"
  },
  "renderTier": "AGENT_REVIEW_CUE",
  "cues": [
    {
      "cueTier": "SAFE_CUE",
      "safeMeaning": "claim-only",
      "notSafeFor": ["semantic-equivalence", "auto-reuse", "auto-fix"],
      "evidenceLane": "function-signature",
      "claim": "same normalized function signature",
      "confidence": "grounded",
      "evidence": [
        {
          "artifact": "function-clones.json",
          "matchedField": "normalizedSignatureHash",
          "algorithmVersion": "function-signature.normalized.v1"
        }
      ]
    },
    {
      "cueTier": "AGENT_REVIEW_CUE",
      "evidenceLane": "near-name",
      "claim": "near exported name",
      "confidence": "heuristic-review",
      "evidence": [
        {
          "artifact": "symbols.json",
          "matchedField": "defIndex",
          "algorithmVersion": "near-name.v1"
        }
      ]
    }
  ]
}
```

Suppressed cues should use a separate field. They are diagnostic evidence, not
default review items:

```json
{
  "suppressedCues": [
    {
      "cueTier": "MUTED_CUE",
      "evidenceLane": "intent-token",
      "reason": "weak-common-token-only",
      "tokens": ["create"],
      "candidateCount": 7,
      "tokenizerVersion": "camel-snake-kebab-digit-v1",
      "tokenPolicyVersion": "prewrite-token-policy-v1"
    }
  ]
}
```

Unavailable evidence should use a third field:

```json
{
  "unavailableEvidence": [
    {
      "evidenceLane": "function-signature",
      "status": "UNAVAILABLE",
      "reason": "missing-artifact",
      "artifact": "function-clones.json"
    }
  ]
}
```

JSON evidence should be structured rather than prose-only. Markdown renderers
may turn structured evidence into chat-readable citations, but downstream
agents should not have to parse citation strings to recover artifact names,
matched fields, policy versions, or exact claims.

## Error Handling

- Missing artifacts produce `UNAVAILABLE`, not `MUTED_CUE`.
- Unsupported shape/signature normalization produces `UNAVAILABLE`.
- Malformed artifacts produce `UNAVAILABLE` with a citation.
- Resolver or parse blind zones may downgrade a cue but must not fabricate
  semantic evidence.
- If a cue has both safe and review evidence, preserve both but render the
  safest exact claim separately from the review instruction.

## Testing Strategy

Add tests before implementation.

Required cases:

- exact symbol exists -> `SAFE_CUE`
- exact file exists -> `SAFE_CUE`
- function signature hash match -> `SAFE_CUE`
- one candidate with both exact signature and near-name evidence preserves
  both cue items
- near-name `useShallowFromState` -> `AGENT_REVIEW_CUE`
- near-name `MergeWithValuesV2` -> `AGENT_REVIEW_CUE`
- `createLogger` with only `create` overlap -> `MUTED_CUE` /
  `suppressedCues`
- `createLogger` suppression includes `tokenPolicyVersion`
- weak token plus non-weak supporting token -> `AGENT_REVIEW_CUE`
- fields-only unsupported shape -> `UNAVAILABLE`, not token fallback
- missing `function-clones.json` for function signature intent ->
  `unavailableEvidence`
- generated/vendor/policy-excluded exact evidence stays out of default
  Markdown and remains visible in JSON diagnostics
- muted cue appears in JSON and not in default Markdown
- renderer wording does not contain `does the same thing`, `semantically
  equivalent`, or `reuse this`
- deterministic ordering is stable when input artifact order changes

## Implementation Notes

This design should be implemented as a small layer around existing pre-write
lookup results, not as a broad rewrite.

Preferred order:

1. Add cue item vocabulary and renderer/artifact fields:
   `cues[]`, `suppressedCues[]`, `unavailableEvidence[]`, and optional
   `renderTier`.
2. Add weak common-token suppression for intent-token hints.
3. Add tests for `createLogger` noise suppression.
4. Keep exact signature/shape/body lanes unchanged except for cue tier labels.
5. Update README wording so the headline claim says Lumin surfaces grounded
   reuse cues and review tasks for AI agents, not semantic duplicate certainty.

Do not add a generic semantic matcher. If future work needs structural fallback,
it should arrive as a separate design with explicit dependency, timeout,
availability, and JSON evidence contracts.

## Acceptance Criteria

- AI agents can distinguish factual reuse evidence from review-only suspicion
  without parsing prose.
- A single candidate can carry multiple cue items without collapsing exact
  evidence into review wording or review evidence into exact claims.
- Common-token-only candidates no longer pollute the default pre-write output.
- Exact hash/signature/file/name matches remain visible and grounded.
- Muted noise remains transparent in JSON diagnostics.
- Unavailable evidence is distinct from muted evidence.
- No new semantic equivalence claim is introduced.

# Vitest Pre-Write Lookup Name Pilot Review

> **Status:** IMPLEMENTED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-pre-write-lookup-name.mjs`.

---

## Purpose

This review decided whether `tests/test-pre-write-lookup-name.mjs` was ready
for a Vitest mirror. The mirror now exists at
`tests/pre-write-lookup-name.test.mjs`. The goal remains to name the lookup
contracts that the runner migration must preserve before any pre-write
threshold, service-operation sibling, or cue policy work continues.

This suite is analyzer-sensitive. It protects whether `lookupName()` says
"already exists", "not observed", "near match", "suppressed diagnostic", or
"policy evidence". A migration must improve test ergonomics without weakening
those evidence distinctions.

## Reviewed Evidence

- Preserved Node command: `node tests/test-pre-write-lookup-name.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:pre-write-lookup-name`.
- Current unit under test: `_lib/pre-write-lookup-name.mjs`.
- Adjacent cue-tier suite: `node tests/test-pre-write-cue-tiers.mjs`.
- Adjacent renderer suite: `node tests/test-pre-write-render.mjs`.
- WT-23 policy spec:
  `docs/spec/pre-write-service-operation-sibling-cues.md`.
- WT-23 public verification notes:
  - `docs/lab/wt23-beta47-suppressed-candidate-verification-2026-05-13.md`,
  - `docs/lab/wt23-beta48-service-operation-policy-verification-2026-05-13.md`,
  - `docs/lab/wt23-beta50-service-operation-markdown-verification-2026-05-14.md`.

## Result

The suite is acceptable as the next pre-write Vitest review candidate, but only
as a behavior-preserving mirror. It should remain separate from cue-tier
rendering, Markdown output, and corpus calibration.

The current Node suite protects more than happy-path lookup. It pins exact
evidence boundaries:

- canonical-first identity lookup;
- identity-keyed fan-in and fan-in-space capability states;
- `EXISTS_MULTIPLE` owner preservation;
- near-name hints as search hints, not reuse proof;
- suppressed semantic/near diagnostics that explain why plausible candidates
  fell below thresholds;
- service-operation sibling policy evidence that promotes only inside the
  policy object;
- explicit noise-floor behavior for unrelated intents.

Those are the reasons this suite should not be folded into a broad pre-write
helper or rewritten into loose shape assertions.

## Protected Invariants

The Vitest pilot must preserve these lookup-name contracts:

- `symbols.json.defIndex` exact identities produce `EXISTS` or
  `EXISTS_MULTIPLE` without silently selecting one owner.
- `fanInByIdentity` is authoritative for identity fan-in; `topSymbolFanIn` must
  not be used as a replacement.
- `supports.identityFanIn` and `supports.identityFanInSpace` control whether
  absent counts are grounded or unavailable.
- Canonical claims are consulted before AST fallback, and canonical/AST
  disagreement remains a structured state rather than free-text drift wording.
- Near-name results stay capped and diagnostic. They do not become existence,
  equivalence, or reuse proof.
- Create-only/domain-only token overlap remains suppressed evidence, not a
  formal semantic hint.
- `searchUser` -> `fetchUser` records both suppressed semantic and suppressed
  near-name evidence while keeping `fetchUser` out of formal `nearNames[]` and
  `semanticHints[]`.
- `serviceOperationSiblingPolicy` remains a versioned policy evidence object
  with `policyId`, `policyVersion`, candidate identity, operation family,
  shared domain tokens, locality, supporting suppressed reasons, and signature
  limitation metadata.
- `serviceOperationSiblingPolicy.promoted[]` may promote `fetchUser` only inside
  policy evidence. It must not relax formal near-name or semantic thresholds.
- `createUser` -> `fetchUser` stays muted with
  `service-sibling-operation-family-mismatch`.
- `searchPost` -> `fetchUser` may be muted with
  `service-sibling-domain-mismatch` only when existing suppressed evidence
  admits the candidate into policy evaluation.
- Unrelated intents such as `xyzzy` keep suppressed candidate arrays and policy
  counts empty.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A runner helper must not build symbols fixtures that accidentally populate
  `topSymbolFanIn` and hide a missing `fanInByIdentity` regression.
- A shared fixture helper must not default capability flags to "supported" when
  a case is supposed to test unavailable evidence.
- The service-operation sibling fixture must retain both the positive
  `searchUser` case and the negative `createUser`, `searchPost`, and `xyzzy`
  cases.
- The `searchPost` domain-mismatch case must keep the supporting prose signal
  that admits `fetchUser` into policy evaluation. A separate no-signal case is
  needed when testing noise-floor absence.
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
- `node tests/test-pre-write-lookup-name.mjs` remains runnable.
- The Vitest file imports `lookupName()` directly, as the Node suite does.
- Temporary repo helpers are not needed for this suite. The fixture boundary is
  in-memory `symbols.json`-shaped objects plus optional canonical claims and
  intent declarations.
- Do not move cue-tier, renderer, resolver, deadness, ranking, performance, or
  public-package verification cases into this pilot.
- Do not introduce corpus calibration or mutation-family expansion in the
  runner migration PR.

## Recommendation

The narrow implementation PR added:

1. `tests/pre-write-lookup-name.test.mjs`,
2. `npm run test:vitest:pre-write-lookup-name`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR keeps the Node suite and represents the current contract
as named Vitest `describe(...)` / `it(...)` blocks grouped by evidence
lane:

- exact identity and canonical lookup;
- fan-in and capability availability;
- near-name and semantic hints;
- suppressed diagnostics;
- service-operation sibling policy;
- noise-floor behavior.

Run both commands when changing this suite:

- `node tests/test-pre-write-lookup-name.mjs`
- `npm run test:vitest:pre-write-lookup-name`

Do not migrate `tests/test-pre-write-cue-tiers.mjs` or any other
analyzer-sensitive suite as part of this pilot.

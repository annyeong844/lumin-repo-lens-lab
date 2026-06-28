# Vitest Resolver Blind-Zone Relevance Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-resolver-blind-zone-relevance.mjs`.

---

## Purpose

This review decides whether `tests/test-resolver-blind-zone-relevance.mjs` is
ready for a narrow Vitest mirror. It does not add the Vitest suite. The goal is
to name the resolver relevance contracts that runner migration must preserve
before unsupported-family diagnostics or absence-claim blockers are expanded.

This suite is analyzer-sensitive. It protects the difference between a resolver
blind zone that plausibly affects a candidate and an unrelated unresolved import
that should remain a run-level confidence limitation. A migration must improve
test ergonomics without turning local uncertainty into repo-global blocking or
automated action proof.

## Reviewed Evidence

- Preserved Node command:
  `node tests/test-resolver-blind-zone-relevance.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:resolver-blind-zone-relevance`.
- Relevance policy under review:
  `skills/lumin-repo-lens-lab/_engine/lib/resolver-blind-zone-relevance.mjs`.
- Provenance integration under review:
  `skills/lumin-repo-lens-lab/_engine/lib/finding-provenance.mjs`.
- Ranking integration under review:
  `skills/lumin-repo-lens-lab/_engine/lib/ranking.mjs`.
- Companion artifact-shape suite:
  `node tests/test-resolver-diagnostics-artifacts.mjs`.
- Companion generated relevance suite:
  `node tests/test-generated-blind-zone-relevance.mjs`.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.
- Evidence contract guardrail:
  `docs/lumin-wiki/concepts/evidence-contract.md`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as a
behavior-preserving mirror.

The current Node suite is intentionally small and mostly pure-function based.
It constructs resolver misses and candidate findings directly, then verifies
how relevance policy, provenance, and ranking interact. The fixture does not
need a temporary repository because the protected behavior is the scoped
relevance decision itself.

The future Vitest mirror must keep the same separation:

- resolver relevance decides whether an unresolved surface affects a candidate;
- provenance records candidate-relevant taint without polluting unrelated
  findings;
- ranking may demote `SAFE_FIX` to review when the candidate is affected;
- generated artifact blind zones remain owned by generated relevance helpers.

## Protected Invariants

The future Vitest pilot must preserve these resolver relevance contracts:

- `RESOLVER_BLIND_ZONE_RELEVANCE_POLICY_VERSION` stays exported for diagnostics;
- target candidate package scope is relevant only to findings in the same
  package or submodule;
- explicit `affectedPackageScope` scopes relevance without becoming a
  repo-global blocker;
- exact target candidate file matches remain blocking unresolved matches;
- generated artifact records return `null` from generic resolver relevance and
  stay owned by generated relevance helpers;
- `resolverBlindZoneRelevantTaint(...)` emits structured soft taint with
  `kind`, `reason`, `family`, `impact`, `relevance`, and count fields;
- `computeFindingProvenance(...)` lowers resolver confidence only for affected
  candidates;
- unrelated findings keep high resolver confidence and no resolver blind-zone
  taint;
- generic resolver soft taint demotes `SAFE_FIX` to `REVIEW_FIX` with
  blocker details rather than promoting action.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A helper must not treat every unresolved import as candidate-relevant.
- A helper must not drop `affectedPackageScope`, because that is the guard
  against repo-global blocking.
- A helper must not merge generated artifact relevance into generic resolver
  relevance.
- A helper must not convert soft resolver taint into `SAFE_FIX` proof.
- A helper must not hide the difference between package-scope relevance and
  exact-file blocking relevance.
- A helper must not collapse unrelated findings into medium resolver confidence.
- The mirror must not combine this policy suite with
  `test-resolver-diagnostics-artifacts.mjs`,
  `test-generated-blind-zone-relevance.mjs`,
  unsupported-family diagnostics suites, generated-artifact suites,
  deadness/ranking suites, or performance/incremental suites.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-resolver-blind-zone-relevance.mjs` remains runnable.
- The pilot may share only generic test assertion style. Relevance fixtures,
  taint payloads, ranking inputs, and expected blocker details stay local to
  this suite.
- The pilot must not change resolver relevance policy behavior.
- The pilot must not add resolver unsupported-family behavior.
- The pilot must not change generated blind-zone relevance policy.
- The pilot must not change ranking thresholds, action-safety, or
  `SAFE_FIX` promotion rules.
- The pilot must not relax exact object equality assertions into broad
  presence checks.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/resolver-blind-zone-relevance.test.mjs`,
2. `npm run test:vitest:resolver-blind-zone-relevance`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the Node suite and represent the current
contract as named Vitest `describe(...)` / `it(...)` blocks grouped by policy
lane:

- policy version and direct relevance decisions;
- affected package scope and exact target file matching;
- generated artifact handoff to generated relevance helpers;
- structured resolver taint;
- provenance confidence scoping;
- ranking demotion from `SAFE_FIX` to `REVIEW_FIX`.

Run both commands when changing this suite:

- `node tests/test-resolver-blind-zone-relevance.mjs`
- `npm run test:vitest:resolver-blind-zone-relevance`

Do not migrate any other resolver, generated, deadness, ranking, cue-tier, or
performance suite as part of the resolver blind-zone relevance pilot.

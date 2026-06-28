# Vitest Generated Blind-Zone Relevance Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-generated-blind-zone-relevance.mjs`.

---

## Purpose

This review decides whether `tests/test-generated-blind-zone-relevance.mjs` is
ready for a narrow Vitest mirror. It does not add the Vitest suite. The goal is
to name the generated blind-zone relevance contracts that a runner migration
must preserve before generated consumer or virtual-surface suites move.

This suite is analyzer-sensitive. It decides when a missing or excluded
generated artifact is relevant enough to limit an absence claim. If the
migration weakens that scope, generated misses can become repo-global blockers;
if it widens evidence, unrelated generated misses can suppress valid deadness
findings.

## Reviewed Evidence

- Preserved Node command:
  `node tests/test-generated-blind-zone-relevance.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:generated-blind-zone-relevance`.
- Policy module under review:
  `skills/lumin-repo-lens-lab/_engine/lib/generated-blind-zone-relevance.mjs`.
- Generated artifact policy dependency:
  `skills/lumin-repo-lens-lab/_engine/lib/generated-artifact-evidence.mjs`.
- Companion generated suites:
  `node tests/test-generated-artifact-evidence.mjs`,
  `node tests/test-generated-consumer-blind-zones.mjs`, and
  `node tests/test-generated-virtual-surface.mjs`.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as a
behavior-preserving mirror.

The current Node suite protects the relevance boundary between a generated miss
and a candidate finding. It is not a generated artifact classifier, not a
resolver expansion test, and not a virtual generated surface test. Its job is
to keep relevant generated uncertainty scoped to the affected provider or
generated consumer surface.

## Protected Invariants

The future Vitest pilot must preserve these generated blind-zone relevance
contracts:

- a candidate inside the generated package root is relevant provider-surface
  evidence;
- a target-candidate submodule is relevant when package-root evidence is absent;
- a consumer submodule alone is not relevant provider-surface proof;
- a consumer-only generated miss does not create finding taint;
- generated consumer blind-zone inventory records the missing generated target
  scope, source reason, matched package, generator family, and candidate path;
- a generated file that is present but excluded by scan policy is recorded as
  `present-but-out-of-scope`, not silently resolved;
- prepared/excluded generated files keep `staleStatus: "unknown"` and
  `staleReason: "generator-input-hash-not-recorded"`;
- generated consumer blind-zone relevance is scoped to the generated package
  surface, not the unrelated consuming application file;
- structured soft taint uses
  `kind: "generated-artifact-missing-relevant"` and preserves reason, impact,
  and relevance fields.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A generated miss consumed by `apps/web` must not globally taint unrelated
  `apps/web` component candidates.
- A provider candidate under `packages/prisma` must stay relevant to a generated
  miss for `@scope/prisma/enums`.
- A package-root-free generated artifact must still match by target-candidate
  submodule rather than falling through to unrelated.
- A present generated file excluded by scan policy must remain a blind zone
  with stale provenance unknown, not a concrete source consumer.
- A generated consumer blind zone may create review taint for affected provider
  candidates, but the suite must not promote that taint to `SAFE_FIX` or
  deadness proof.
- The mirror must not combine this relevance fixture with generated artifact
  classification, generated consumer producer integration, generated virtual
  surfaces, output-to-source layouts, dynamic modules, deadness/ranking, or
  performance fixtures.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-generated-blind-zone-relevance.mjs` remains runnable.
- The pilot may use temporary directories for present/excluded generated files,
  but the helper boundary is setup only. Relevance rules, scan-policy status,
  stale provenance, and structured taint assertions stay local to this suite.
- The pilot must not add new generated artifact evidence heuristics.
- The pilot must not change generated consumer blind-zone producer behavior.
- The pilot must not change virtual generated surfaces.
- The pilot must not change resolver diagnostics, deadness, ranking, or
  `SAFE_FIX` behavior.
- The pilot must not make generated blind zones repo-global by default.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/generated-blind-zone-relevance.test.mjs`,
2. `npm run test:vitest:generated-blind-zone-relevance`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the Node suite and represent the current
contract as named Vitest `describe(...)` / `it(...)` blocks grouped by
generated relevance lane:

- provider package-root relevance;
- target-candidate submodule relevance;
- consumer-only non-relevance;
- generated consumer blind-zone inventory shape;
- scan-policy excluded generated files with stale provenance;
- candidate-scoped generated consumer relevance;
- structured soft taint.

Run both commands when changing this suite:

- `node tests/test-generated-blind-zone-relevance.mjs`
- `npm run test:vitest:generated-blind-zone-relevance`

Do not migrate generated consumer, generated virtual surface, resolver,
deadness/ranking, or performance suites as part of the generated blind-zone
relevance pilot.

# Vitest Generated Consumer Blind-Zones Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-generated-consumer-blind-zones.mjs`.

---

## Purpose

This review decides whether `tests/test-generated-consumer-blind-zones.mjs` is
ready for a narrow Vitest mirror. It does not add the Vitest suite. The goal is
to name the artifact contract for generated consumer blind zones before any
runner migration rewrites the producer-backed fixture shape.

This suite is analyzer-sensitive. It runs `build-symbol-graph.mjs` against
temporary workspace fixtures and verifies that generated consumer misses are
recorded as blind-zone inventory, not as observed source consumers and not as
fake resolved graph edges.

## Reviewed Evidence

- Preserved Node command:
  `node tests/test-generated-consumer-blind-zones.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:generated-consumer-blind-zones`.
- Producer under review:
  `build-symbol-graph.mjs`.
- Artifact under review:
  `symbols.json`.
- Policy helper dependency:
  `skills/lumin-repo-lens-lab/_engine/lib/generated-blind-zone-relevance.mjs`.
- Companion generated suites:
  `node tests/test-generated-artifact-evidence.mjs`,
  `node tests/test-generated-blind-zone-relevance.mjs`, and
  `node tests/test-generated-virtual-surface.mjs`.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as a
behavior-preserving producer-backed mirror.

The current Node suite protects generated consumer blind-zone inventory emitted
by `build-symbol-graph.mjs`. It is not a generated artifact classifier, not a
virtual generated surface test, and not a deadness/ranking test. Its job is to
prove that unresolved generated workspace subpath consumers are visible in
`symbols.json.generatedConsumerBlindZones[]` with enough scope and provenance
for downstream diagnostics.

## Protected Invariants

The future Vitest pilot must preserve these generated consumer blind-zone
contracts:

- `symbols.json.meta.supports.generatedConsumerBlindZones` is `true`.
- A missing generated workspace subpath emits exactly a consumer blind-zone
  inventory record.
- The inventory record keeps `reason: "generated-consumer-blind-zone"`.
- The inventory record keeps
  `sourceReason: "workspace-generated-artifact-missing"`.
- The inventory record preserves `specifier`, `consumerFile`, `matchedPackage`,
  `targetSubpath`, `status`, and `scopePackageRoot`.
- Default mode records the missing generated target as `status: "missing"`.
- `--generated-artifacts prepared` is forwarded into the generated consumer
  zone as `mode: "prepared"`.
- Prepared mode without trusted generator input hashes keeps
  `staleStatus: "unknown"` and
  `staleReason: "generator-input-hash-not-recorded"`.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A missing generated subpath must not disappear from `symbols.json`.
- A generated consumer blind zone must not become a concrete resolved source
  consumer.
- A generated consumer blind zone must not become a virtual surface; virtual
  generated surfaces are covered by `tests/test-generated-virtual-surface.mjs`.
- Prepared mode must not claim freshness without trusted generator provenance.
- The producer fixture must keep workspace package shape explicit:
  `apps/web` consumes `@scope/prisma/enums`, while `packages/prisma` owns the
  generated package surface.
- The mirror must not combine this producer-backed fixture with generated
  artifact classification, blind-zone relevance helpers, virtual surfaces,
  output-to-source layouts, dynamic modules, deadness/ranking, or performance
  fixtures.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-generated-consumer-blind-zones.mjs` remains runnable.
- The pilot may use the setup-only temporary repo helper, but the helper must
  not absorb workspace, generated consumer, resolver, stale provenance, or
  `build-symbol-graph.mjs` semantics.
- The pilot must not change generated artifact evidence heuristics.
- The pilot must not change generated blind-zone relevance behavior.
- The pilot must not change virtual generated surfaces.
- The pilot must not change resolver diagnostics, deadness, ranking, or
  `SAFE_FIX` behavior.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/generated-consumer-blind-zones.test.mjs`,
2. `npm run test:vitest:generated-consumer-blind-zones`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the Node suite and represent the current
contract as named Vitest `describe(...)` / `it(...)` blocks grouped by producer
lane:

- missing generated workspace subpath inventory;
- generated consumer blind-zone support metadata;
- prepared generated artifact mode forwarding;
- unknown stale provenance for prepared mode.

Run both commands when changing this suite:

- `node tests/test-generated-consumer-blind-zones.mjs`
- `npm run test:vitest:generated-consumer-blind-zones`

Do not migrate generated artifact evidence, generated blind-zone relevance,
generated virtual surface, resolver, deadness/ranking, or performance suites as
part of the generated consumer blind-zones pilot.

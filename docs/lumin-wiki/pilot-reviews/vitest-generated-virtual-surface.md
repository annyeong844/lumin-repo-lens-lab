# Vitest Generated Virtual Surface Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-generated-virtual-surface.mjs`.

---

## Purpose

This review decides whether `tests/test-generated-virtual-surface.mjs` is ready
for a narrow Vitest mirror. It does not add the Vitest suite. The goal is to
name the virtual generated surface contracts before any runner migration
rewrites the Prisma enum fixture shape.

This suite is analyzer-sensitive. It is the generated-surface exception to the
default "missing generated artifact is a blind zone" rule. When the engine has
explicit Prisma enum generator evidence, it may create a conservative virtual
surface, but that virtual surface must remain partial and must not claim runtime
equivalence or body/call evidence.

## Reviewed Evidence

- Preserved Node command:
  `node tests/test-generated-virtual-surface.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:generated-virtual-surface`.
- Virtual surface helper under review:
  `skills/lumin-repo-lens-lab/_engine/lib/generated-virtual-surface.mjs`.
- Producer under review:
  `build-symbol-graph.mjs`.
- Artifact under review:
  `symbols.json`.
- Companion generated suites:
  `node tests/test-generated-artifact-evidence.mjs`,
  `node tests/test-generated-blind-zone-relevance.mjs`, and
  `node tests/test-generated-consumer-blind-zones.mjs`.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as a
behavior-preserving producer-backed mirror.

The current Node suite protects the virtual generated surface contract for
Prisma enum imports. It is not a general generated artifact resolver, not a
generated consumer blind-zone inventory test, and not a runtime-equivalence
claim. Its job is to prove that supported virtual enum surfaces are explicit,
partial, scoped, and conservative.

## Protected Invariants

The future Vitest pilot must preserve these virtual generated surface
contracts:

- `schemaUsesPrismaEnumGenerator(...)` recognizes the Prisma enum generator
  provider.
- `parsePrismaEnums(...)` extracts enum names and values without value
  attributes.
- A supported `@scope/prisma/enums` import resolves through
  `generatedVirtualSurfaces[]` and `generatedVirtualImportConsumers[]`.
- Resolved virtual imports remove the matching unresolved internal specifier.
- `symbols.uses.resolvedGeneratedVirtual` counts the virtual consumer.
- `symbols.json.meta.supports.generatedVirtualSurfaces` is `true`.
- Virtual surfaces keep `source: "generated-virtual"` and `virtual: true`.
- Virtual surfaces keep `runtimeEquivalence: false`.
- Virtual surfaces keep `surfaceCompleteness: "partial"`.
- Exported enum facts include both value and type spaces when supported.
- A schema without the enum generator provider does not create a virtual
  surface.
- A requested enum absent from the schema remains unresolved and does not create
  a virtual import consumer.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A virtual surface must not exist without explicit schema generator evidence.
- A virtual surface must not claim full runtime equivalence.
- A virtual surface must not resolve names absent from the schema enum surface.
- A virtual surface must not erase unrelated unresolved internal imports.
- A virtual surface must not be confused with generated consumer blind-zone
  inventory.
- The mirror must not combine this producer-backed fixture with generated
  artifact classification, blind-zone relevance helpers, generated consumer
  inventory, output-to-source layouts, dynamic modules, deadness/ranking, or
  performance fixtures.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-generated-virtual-surface.mjs` remains runnable.
- The pilot may use the setup-only temporary repo helper, but the helper must
  not absorb Prisma schema parsing, virtual surface, resolver, generated
  consumer, or `build-symbol-graph.mjs` semantics.
- The pilot must not add new generated artifact evidence heuristics.
- The pilot must not change generated blind-zone relevance behavior.
- The pilot must not change generated consumer blind-zone inventory.
- The pilot must not change resolver diagnostics, deadness, ranking, or
  `SAFE_FIX` behavior.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/generated-virtual-surface.test.mjs`,
2. `npm run test:vitest:generated-virtual-surface`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the Node suite and represent the current
contract as named Vitest `describe(...)` / `it(...)` blocks grouped by virtual
surface lane:

- Prisma enum parser behavior;
- supported generated virtual surface resolution;
- no-provider negative case;
- missing schema export negative case.

Run both commands when changing this suite:

- `node tests/test-generated-virtual-surface.mjs`
- `npm run test:vitest:generated-virtual-surface`

Do not migrate generated artifact evidence, generated blind-zone relevance,
generated consumer blind zones, resolver, deadness/ranking, or performance
suites as part of the generated virtual surface pilot.

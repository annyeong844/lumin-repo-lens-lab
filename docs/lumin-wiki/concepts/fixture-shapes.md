# Fixture Shapes

Fixture-shape comparison is the bridge between suite inventory and test-file
movement. This page names repeated fixture patterns across the inventoried
workstreams so future refactors can merge setup code without merging unrelated
risk claims.

## Comparison Rules

- Compare fixture shapes before moving test files.
- Preserve the original failure mode when sharing setup helpers.
- Keep artifact-shape assertions near the artifact they protect.
- Keep analyzer correctness, public packaging, and lab evidence separate even
  when they use similar temporary repos.
- Do not turn a shared helper into a shared interpretation of evidence.

## Repeated Shapes

| Fixture Shape | Seen In | Shared Setup Candidate | Must Stay Separate |
|---|---|---|---|
| Temporary repo plus `.audit` output directory | [Pre-Write](../workstreams/pre-write.md), [Deadness](../workstreams/deadness.md), [Performance](../workstreams/performance.md) | A small helper that creates a repo root, output root, `package.json`, writes files, and runs one producer or audit command. | The assertion lens: pre-write evidence availability, deadness review proof, and performance counters are different claims. |
| Unsupported resolver family mini repo | [Resolver](../workstreams/resolver.md), [Deadness](../workstreams/deadness.md) | A helper for internal-looking unresolved imports, unsupported family records, blocked candidate hints, and no concrete graph edge. | Family identity must remain explicit: generated artifacts, output-to-source layouts, Node `#imports`, and dynamic modules are not interchangeable. |
| Generated or framework/resource surface package | [Resolver](../workstreams/resolver.md), [Deadness](../workstreams/deadness.md), [Public Package](../workstreams/public-package.md) | A builder for `package.json`, framework dependencies, generated-looking paths, bundled files, declarations, templates, and codemod resources. | Resolver blind-zone evidence, deadness blockers, and public package allowlist checks protect different contracts. |
| Consumer/member precision graph | [Pre-Write](../workstreams/pre-write.md), [Deadness](../workstreams/deadness.md) | A fixture with exported siblings, namespace or class-member consumers, and one unused sibling. | Pre-write class methods are review cues; deadness member precision is export-consumer evidence. They must not share ranking expectations. |
| Prototype-name dictionary edge case | [Pre-Write](../workstreams/pre-write.md), [Performance](../workstreams/performance.md) | A tiny class/function fixture that includes names such as `constructor`, `toString`, `hasOwnProperty`, `valueOf`, and `__proto__`. | The same shape can guard different grouping bugs: class-method indexing, clone grouping, symbol graph maps, or cache dictionaries. |
| Cold/warm incremental repo | [Performance](../workstreams/performance.md), [Pre-Write](../workstreams/pre-write.md) | A helper that runs cold, mutates one file, reruns warm, and compares refreshed versus reused facts. | Post-write and pre-write lifecycle caches have different baseline semantics; a helper must not hide that difference. |
| Public/internal export-surface package | [Public Package](../workstreams/public-package.md), [Deadness](../workstreams/deadness.md) | A package fixture with explicit public files, internal files, generated package output, and manifest summaries. | Public package tests protect shipped surface. Deadness tests protect absence claims. One must not justify the other. |
| Markdown renderer and manifest mirror | [Resolver](../workstreams/resolver.md), [Deadness](../workstreams/deadness.md), [Public Package](../workstreams/public-package.md) | A helper that writes raw JSON artifacts, builds manifest summaries, and asserts summary/review-pack reader guidance. | Markdown visibility is reader guidance. It is not proof that the underlying analyzer fact is correct. |

## First Refactor Candidate

The safest first extraction candidate is a temporary repo helper with these
operations:

- create an isolated root and output directory
- write `package.json`
- write source files by relative path
- read JSON artifacts by relative path
- clean up after the test

This helper should not know about resolver families, pre-write intents,
deadness tiers, package publishing, or performance interpretation. Those remain
owned by the specific suites named in the workstream inventories.

The setup-only helper contract is specified in
[`docs/spec/shared-test-fixture-helper.md`](../../spec/shared-test-fixture-helper.md).

## Shapes Not Ready To Merge

- Resolver unsupported-family fixtures should not collapse until each family
  keeps a named reason, output level, and no-fake-edge assertion.
- Public install verification notes under `docs/lab/` are evidence records, not
  reusable unit-test fixtures.
- Scanner equivalence fixtures should stay close to the scanner until accepted
  syntax and fallback reasons are stable.
- SAFE_FIX calibration fixtures should stay separate from review-evidence
  fixtures until ranking behavior has a narrower design.

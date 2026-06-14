# Vitest Definition ID Canonical Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-definition-id-canonical.mjs`.

---

## Purpose

This review decides whether `tests/test-definition-id-canonical.mjs` can move
as a narrow Lane A Vitest mirror. It does not add a Vitest suite. The goal is
to preserve the PCEF P3 identity contract where `symbols.json` owns the
canonical `definitionId` for an exported identity and the downstream
`call-graph.json` and `export-action-safety.json` evidence attach to that same
id.

The candidate is acceptable as a single-suite mirror because it creates one
small temporary fixture with a local function exported through an alias,
executes only the producer scripts needed to build identity evidence, and
injects a minimal `dead-classify.json` proposal so action-safety can consume the
same alias target. It does not run the full audit orchestrator or claim broad
deadness behavior.

The future mirror should keep that identity contract local. It must not expand
into general call-graph precision, dead-export ranking, resolver behavior,
public API policy, performance, or full audit pipeline coverage.

## Reviewed Evidence

| Suite                                    | Preserved Node Command                        | Proposed Focused Vitest Command               | Surface Under Review                                                            |
| ---------------------------------------- | --------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------------------------- |
| `tests/test-definition-id-canonical.mjs` | `node tests/test-definition-id-canonical.mjs` | `npm run test:vitest:definition-id-canonical` | canonical definitionId continuity across symbols, call graph, and action-safety |

Current suite description is in `tests/README.md`.

Goal lane: Lane A, low-risk core/helper identity-contract guard.

## Result

This suite is acceptable as one narrow Vitest mirror.

The future implementation PR should preserve the same temporary fixture and
producer subprocess behavior without changing symbol graph, call graph,
dead-classify, or action-safety semantics. The Node entrypoint must remain
runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `symbols.json.defIndex["src/lib.ts"].publicApi.definitionId` exists;
- the canonical `definitionId` points at the local `FunctionDeclaration`, not
  the `ExportSpecifier` alias node;
- `call-graph.json.exportAliasMap["src/lib.ts::publicApi"]` equals the
  `symbols.json` canonical `definitionId`;
- `callFanInByDefinitionId[definitionId]` counts the call through the aliased
  exported name;
- `callFanInByIdentity["src/lib.ts::publicApi"]` still counts the exported
  identity;
- `export-action-safety.json.findings[0].safeAction.target.definitionId` equals
  the `symbols.json` canonical `definitionId`;
- the fabricated `dead-classify.json` proposal retains both the exported
  symbol name and the local alias target name.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- assigning the export alias identity to the `ExportSpecifier` instead of the
  local declaration must fail;
- building call fan-in by exported name only, without the canonical
  `definitionId`, must fail;
- drifting `callFanInByDefinitionId` and `callFanInByIdentity` apart must fail;
- attaching the action-safety target to the wrong export identity must fail;
- losing the `localName: "impl"` alias context in the dead-classify proposal
  must fail;
- changing producer ordering so action-safety runs without the required symbol
  and call-graph evidence must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is a single temporary two-file TypeScript repo plus a
  minimal `dead-classify.json` proposal.
- A future mirror may use setup-only helpers for temp directories and cleanup,
  but the helper must not decide identity, fan-in, alias, or action-safety
  meaning.
- The mirror may keep producer subprocesses via `execFileSync`.
- The mirror must not add broad assertions about call graph precision,
  dead-export classification, ranking, resolver behavior, public API behavior,
  performance, Markdown rendering, or full audit orchestration.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/definition-id-canonical.test.mjs`,
2. `npm run test:vitest:definition-id-canonical`,
3. candidate-board updates moving the suite from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves every
current Node assertion as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, and `npm run test:vitest`.

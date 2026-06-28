# Vitest Resolver Diagnostics Artifacts Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-resolver-diagnostics-artifacts.mjs`.

---

## Purpose

This review decides whether `tests/test-resolver-diagnostics-artifacts.mjs`
is ready for a narrow Vitest mirror. It does not add the Vitest suite. The goal
is to name the artifact contracts that runner migration must preserve before
future resolver work adds more unsupported families or capability-pack lanes.

This suite is analyzer-sensitive. It protects the split between the static
resolver capability matrix and per-run resolver diagnostics. A migration must
improve test ergonomics without mixing analyzer capability metadata with
repository-specific unresolved imports, blind zones, candidate targets, or
blocked absence hints.

## Reviewed Evidence

- Preserved Node command:
  `node tests/test-resolver-diagnostics-artifacts.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:resolver-diagnostics-artifacts`.
- Diagnostics producer under review:
  `skills/lumin-repo-lens-lab/_engine/producers/build-resolver-diagnostics.mjs`.
- Capability artifact code under review:
  `skills/lumin-repo-lens-lab/_engine/lib/resolver-capabilities.mjs`.
- Blind-zone relevance code under review:
  `skills/lumin-repo-lens-lab/_engine/lib/resolver-blind-zone-relevance.mjs`.
- Generated blind-zone relevance code under review:
  `skills/lumin-repo-lens-lab/_engine/lib/generated-blind-zone-relevance.mjs`.
- Companion unsupported-family suite:
  `node tests/test-node-imports-unsupported.mjs`.
- Companion blind-zone relevance suite:
  `node tests/test-resolver-blind-zone-relevance.mjs`.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.
- Architecture guardrail:
  `docs/spec/lumin-architecture-realignment.md`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as a
behavior-preserving mirror.

The current Node suite writes a synthetic `symbols.json` into a temporary audit
output directory, then runs `build-resolver-diagnostics.mjs`. The fixture is
intentionally artifact-shaped rather than source-shaped: it verifies how the
diagnostics producer transforms unresolved records, generated consumer blind
zones, candidate targets, relevance policies, blocked hints, and summaries into
two separate output artifacts.

The two output artifacts must stay distinct:

- `resolver-capabilities.json` describes what this analyzer version supports.
- `resolver-diagnostics.json` describes what this repository run exposed.

## Protected Invariants

The future Vitest pilot must preserve these resolver diagnostics contracts:

- `resolver-capabilities.json` is written with
  `schemaVersion: "resolver-capabilities.v1"`;
- `resolver-capabilities.json.resolverVersion` is deterministic and matches
  the diagnostics artifact;
- the capability matrix records static families such as `node-imports` and
  `tsconfig-paths` with status, reason codes, condition profiles, and
  absence-claim policy;
- `resolver-diagnostics.json` is written with
  `schemaVersion: "resolver-diagnostics.v1"`;
- `resolver-diagnostics.json.capabilityReference` points back to
  `resolver-capabilities.json` with matching schema and resolver version;
- unresolved imports preserve `family`, `reason`, `outputLevel`, specifier,
  consumer file, resolver stage, and generated artifact metadata when present;
- candidate targets remain diagnostic-only with
  `proofUse: "diagnostic-only"` and `createsGraphEdge: false`;
- blind zones declare candidate-relevant blocking policy rather than becoming
  repo-global blockers;
- generated consumer blind zones declare generated relevance policy separately
  from ordinary resolver relevance policy;
- `blockedCandidateHints[]` points reviewers at affected candidate paths and
  package scopes without becoming action proof;
- `summary` stays machine-readable and sorted enough for manifest/report
  consumers to compare family, reason, affected-scope, blind-zone, and blocked
  hint counts.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A helper must not fold `resolver-capabilities.json` into
  `resolver-diagnostics.json` or vice versa.
- A helper must not treat candidate targets as resolved graph edges.
- A helper must not drop `capabilityReference`, because downstream readers use
  it to distinguish analyzer capability from repo evidence.
- A helper must not collapse generated consumer blind-zone relevance into the
  ordinary resolver relevance policy.
- A helper must not turn `blockedCandidateHints[]` into `SAFE_FIX`,
  deadness, or ranking evidence.
- A helper must not make blocked hints repo-global; candidate relevance and
  affected package scope must remain visible.
- The fixture must keep multiple families in one run so summary grouping stays
  cross-family and deterministic.
- The mirror must not combine this artifact-shape suite with
  `test-resolver-blind-zone-relevance.mjs`,
  `test-node-imports-unsupported.mjs`,
  `test-output-source-layout-diagnostics.mjs`, generated-artifact suites,
  deadness/ranking suites, or performance/incremental suites.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-resolver-diagnostics-artifacts.mjs` remains runnable.
- The pilot may use temporary repo fixtures, but the helper boundary is setup
  only. The synthetic `symbols.json` shape, diagnostics producer invocation,
  artifact parsing, and artifact assertions stay local to this suite.
- The pilot must not change resolver capability family definitions.
- The pilot must not add resolver heuristics or unsupported-family behavior.
- The pilot must not change manifest/report rendering.
- The pilot must not relax artifact-shape assertions into broad presence
  checks.
- Shared resolver diagnostics assertion helpers remain a separate design
  question. This mirror may reuse setup-only fixture helpers, but it must not
  introduce semantic helpers that hide which artifact field proves which
  contract.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/resolver-diagnostics-artifacts.test.mjs`,
2. `npm run test:vitest:resolver-diagnostics-artifacts`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the Node suite and represent the current
contract as named Vitest `describe(...)` / `it(...)` blocks grouped by artifact
lane:

- capability artifact schema and static family metadata;
- diagnostics artifact schema and capability reference;
- unresolved imports and generated artifact metadata;
- candidate targets and no graph-edge proof;
- candidate-relevant blind-zone policies;
- blocked candidate hints and summary pivots.

Run both commands when changing this suite:

- `node tests/test-resolver-diagnostics-artifacts.mjs`
- `npm run test:vitest:resolver-diagnostics-artifacts`

Do not migrate any other resolver, generated, deadness, ranking, or performance
suite as part of the resolver diagnostics artifacts pilot.

# Vitest Artifact Output Presentation Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:** `tests/test-topology-mermaid.mjs`,
> `tests/test-sarif-fix-plan.mjs`.

---

## Purpose

This review decides whether two artifact-output presentation suites can move as
one narrow Lane H Vitest mirror batch. It does not add Vitest suites. The goal
is to preserve renderer/output contracts for already-computed evidence without
turning the mirror into a topology, ranking, deadness, or full audit pipeline
test.

The candidates are acceptable as one batch because both suites validate output
adapters fed by controlled in-memory or fixture JSON inputs:

- `tests/test-topology-mermaid.mjs` imports `renderTopologyMermaid(...)`
  directly and checks the Markdown/Mermaid companion artifact contract.
- `tests/test-sarif-fix-plan.mjs` synthesizes a small `fix-plan.json`, runs
  `emit-sarif.mjs`, and checks SARIF tier-to-level output semantics.

The future mirrors should keep those contracts local. They must not expand into
SCC computation, resolver behavior, dead-export classification, ranking policy
selection, full audit orchestration, public package install behavior, or
performance measurement.

## Reviewed Evidence

| Suite                             | Preserved Node Command                 | Proposed Focused Vitest Command        | Surface Under Review                                   |
| --------------------------------- | -------------------------------------- | -------------------------------------- | ------------------------------------------------------ |
| `tests/test-topology-mermaid.mjs` | `node tests/test-topology-mermaid.mjs` | `npm run test:vitest:topology-mermaid` | topology Markdown/Mermaid companion artifact rendering |
| `tests/test-sarif-fix-plan.mjs`   | `node tests/test-sarif-fix-plan.mjs`   | `npm run test:vitest:sarif-fix-plan`   | SARIF fix-plan tier mapping and result property output |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane H, artifact/output presentation guard.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR should preserve the same direct-renderer and
fixture-artifact behavior without changing `topology-mermaid.mjs`,
`emit-sarif.mjs`, `fix-plan.json` semantics, SARIF severity mapping, or
topology graph evidence semantics. The Node entrypoints must remain runnable.

## Protected Invariants

The future Vitest mirrors must preserve these contracts:

- `renderTopologyMermaid(...)` emits a Markdown artifact that starts with
  `# Topology Mermaid` and contains a fenced Mermaid block;
- the topology Mermaid artifact keeps the stable reader sections:
  `How To Read This`, `Cross-Submodule Edges`, `Runtime Cycles`, `Hub Files`,
  `Omitted Detail / Limits`, and `Citation Contract`;
- cross-submodule edges render as Mermaid `flowchart LR` nodes and labeled
  edges using stable node ids;
- runtime cycle diagrams use only SCC members and internal runtime edges;
- hub-file Markdown cites `topology.json.topFanIn` and
  `topology.json.topFanOut`;
- empty topology data renders explicit no-edge, no-cycle, and no-hub notes;
- Mermaid labels are escaped for quoted labels;
- edge and cycle caps report shown counts versus source counts;
- dangling topology edges do not produce `undefined` Mermaid node ids;
- `emit-sarif.mjs` takes the `fix-plan.json` branch when present and emits
  result properties carrying `tier`;
- SARIF output excludes `MUTED` fix-plan entries;
- `SAFE_FIX` emits as SARIF `warning`;
- `REVIEW_FIX` and `DEGRADED` emit as SARIF `note`;
- SARIF result properties preserve proposal bucket, ranking reason, and runtime
  `hitsInSymbol` evidence;
- the SARIF level distribution remains one warning and two notes for the
  controlled tier fixture.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- removing the Markdown reader sections must fail;
- changing Mermaid graph syntax, labels, or stable node ids must fail;
- rendering missing or type-only-only cycle edges as dangling Mermaid edges must
  fail;
- hiding cap truncation details must fail;
- treating `topology-mermaid.md` as citation authority instead of a visual
  companion must fail;
- emitting `MUTED` findings to SARIF must fail;
- promoting runtime-executed `DEGRADED` findings to SARIF warnings must fail;
- dropping fix-plan result properties used by downstream filtering must fail;
- bypassing `emit-sarif.mjs` with hand-built SARIF JSON must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node commands remain runnable.
- The fixture boundary is direct renderer input for topology Mermaid and a
  temporary fixture repo plus synthetic `fix-plan.json` for SARIF.
- A future mirror may use setup-only temp helpers, but helper code must not
  decide topology, ranking, fix-plan, SARIF severity, or citation meaning.
- The mirror must not run the full audit pipeline.
- The mirror must not change ranking, classifier, resolver, deadness,
  performance, or public package behavior.
- The mirror must not absorb `tests/test-rank-fixes.mjs`,
  `tests/test-module-reachability.mjs`, `tests/test-checklist-facts.mjs`,
  call-graph suites, or broader audit-repo suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/topology-mermaid.test.mjs`,
2. `tests/sarif-fix-plan.test.mjs`,
3. `npm run test:vitest:topology-mermaid`,
4. `npm run test:vitest:sarif-fix-plan`,
5. candidate-board updates moving both suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors that preserve every
current Node assertion as named Vitest cases. It should run the preserved Node
commands, the focused Vitest commands, and `npm run test:vitest`.

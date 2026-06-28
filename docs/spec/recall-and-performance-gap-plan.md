# Recall And Performance Gap Plan

> **Role:** maintainer-facing debt spec for analyzer recall gaps and production-scale performance work.
> **Status:** SPEC.
> **Last updated:** 2026-05-12

---

## 1. Problem

Recent corpus checks exposed two separate product risks:

1. **Recall gaps.** Lumin can be conservative without being accurate. If a
   namespace, dynamic module surface, or class method surface is not modeled,
   the tool may either keep truly dead code alive or fail to surface relevant
   reuse candidates.
2. **Runtime and memory cost.** Large monorepos can spend tens of seconds in
   the audit pipeline and approach high peak RSS. Optimizing this without
   measurement risks making memory pressure worse.

The immediate lesson is not "relax ranking." The correct response is to add
explicit recall surfaces and explicit bottleneck evidence.

## 2. Current Reported Gaps

### 2.1 Namespace Re-Export Precision

The reported bug:

```ts
// barrel.ts
export * as ns from "./source";

// consumer.ts
import { ns } from "./barrel";
ns.used();
```

Older behavior treated the namespace re-export as broad source usage. That kept
every export in `source.ts` alive, including truly unused members such as
`nsUnusedFunc` and `nsUnusedConst`.

Contract:

- Exact namespace member reads may protect only the observed member.
- Broad/opaque namespace usage may keep the source confidence-limited.
- Namespace evidence must not silently become repo-global "alive" evidence.

### 2.2 `import.meta.glob`

Frameworks such as Vite use `import.meta.glob(...)` as a module discovery and
entry/consumer surface. If Lumin ignores this family, it can miss consumers or
entry roots.

Contract:

- Literal glob patterns may become concrete file edges only when expansion is
  deterministic under the current scan policy.
- Non-literal or unsupported glob expressions must become resolver diagnostics
  and relevant blind zones, not fake resolved edges.
- The resolver artifact should identify the family as `import-meta-glob`.

### 2.3 Class Member Search

Pre-write currently relies heavily on top-level/export-oriented symbol data.
Class-heavy repositories can hide the relevant reuse candidate inside methods:

```ts
class TaskControlEventDispatcher {
  handleDelete(...) { ... }
}
```

An intent such as `handleBulkDelete` should surface `handleDelete` as an
agent-review cue before weaker `handle*` top-level matches crowd it out.

Contract:

- Class methods belong in a pre-write member index, not in `defIndex`.
- Method evidence is a review cue only.
- If method evidence is unavailable, pre-write must say so. It must not imply
  no method exists.

### 2.4 Package Output-To-Source Mapping

Workspace package public-surface recovery currently depends on a small set of
output-directory to source-directory probes such as `dist/ -> src/`. That works
for conventional packages, especially when a conditional export target such as
`import` points at a clean compiled path and a transitive barrel walker can then
follow re-exports.

It is not a general source map.

If a package uses non-standard layouts such as `compiled/ -> source/`,
`bazel-out/... -> lib/`, or source files that only appear through generated
declaration trees, the first public-surface seed may be missing. Once the seed
is missing, the transitive re-export walker has nothing reliable to propagate.

Contract:

- Output-to-source probing is resolver support, not proof of completeness.
- Unsupported output layouts must become capability diagnostics when they block
  public-surface or absence claims.
- Do not keep growing an unversioned list of directory pairs as an invisible
  magic-number policy. Any expansion needs fixtures and a named resolver policy.

### 2.5 Entry-Unreachable Cycles And File-Level Deadness

`module-reachability.json` records files that are not reachable from known entry
surfaces. That is useful evidence, but the current dead-export pipeline is still
primarily symbol/fan-in oriented. A mutually importing cycle can therefore keep
its own exports alive even when the whole SCC is unreachable from every entry:

```ts
// components/App.ts
import { Modal } from "./Modal";
export function App() { return Modal(); }

// components/Modal.ts
import { App } from "./App";
export function Modal() { return App; }
```

If no entry reaches either file, the SCC is a dead file group. Treating each
export as live because another unreachable file imports it misses the actual
product issue.

Contract:

- Entry-rooted reachability and symbol fan-in are different evidence lenses.
- Imports from files that are themselves entry-unreachable should not by
  themselves prove user-reachable liveness.
- Unreachable SCCs should be reported as file/group review evidence before any
  export-level `SAFE_FIX` promotion. The first slice should prefer an explicit
  artifact or review lane over a broad ranking relaxation.
- `SAFE_FIX` remains action-scoped: deleting a file group needs stronger proof
  than demoting an export, and type/public/framework surfaces still block.

### 2.6 Pre-Write Semantic Sibling Recall

Pre-write currently has two lightweight recall paths:

- near-name edit/prefix matching;
- exact token overlap from intent name, declaration kind, and intent prose.

That can miss a reuse cue such as `fetchUser` for an intent like `searchUser`:
the names share only a broad domain noun (`user`), while the verbs differ and
may not reach the current semantic score threshold. Returning
`nearNames: []` and `semanticHints: []` is honest, but it makes the “kind buddy”
reuse cue ineffective for common service-operation siblings.

Contract:

- This must remain an `AGENT_REVIEW_CUE`, not a reuse proof.
- Name-sibling recall should be calibrated against noise: broad nouns such as
  `user`, `task`, or `item` are useful only when supported by nearby file/domain
  evidence, compatible operation class, signature shape, or directory/package
  locality.
- The artifact should expose suppressed candidates and reasons so users can see
  whether a candidate was missed because of token policy, score threshold,
  missing method/function index, or scan availability.
- Any threshold change belongs to a named policy object per
  `agent-entry-resolver-calibration.md`; do not silently relax magic numbers.

Current first slice: `lookupName()` records suppressed semantic and near-name
candidates with reason codes, matched tokens, capped raw counts, and locality
metadata. These diagnostics explain misses such as `searchUser` vs `fetchUser`
without changing formal `nearNames`, `semanticHints`, or cue-card promotion.

### 2.7 Corpus Feedback Inventory

Keep the observed reports attached to this spec so later implementation work
does not blur them into generic "resolver improvements."

| Corpus / shape | Observed behavior | Product risk | Tracker |
|---|---|---|---|
| next.js-canary nested apps: `apps/bundle-analyzer/app/layout.tsx`, `bench/heavy-npm-deps/app/page.js`, `bench/module-cost/app/app/commonjs/route.js` | Framework sentinel did not activate for Next.js apps whose `package.json` lives under non-workspace `apps/*` or `bench/*`. | Next route/layout/page handlers were flagged as removable dead exports. | WT-19 |
| Namespace re-export: `export * as ns from "./source"` plus `ns.used()` | Older behavior treated namespace reachability as broad source liveness. | Truly unused source exports were missed, reducing recall. | WT-16 |
| `import.meta.glob(...)` surfaces | Dynamic module discovery was not modeled. | Route/entry modules can be missed, causing false absence claims. | WT-17 |
| Class-heavy OO codebase: `TaskControlEventDispatcher#handleDelete` | Pre-write `defIndex` did not include class methods. | Relevant reuse candidates were absent from pre-write search results. | WT-15 |
| Entry-unreachable SCC: `components/App.ts` ↔ `components/Modal.ts` | Exports import each other, so symbol fan-in keeps them alive even though the SCC is not reachable from any entry. | Dead file groups hidden inside cycles are missed; file-level unused differs from export-level unused. | WT-22 |
| Unimported hook file: `useDebounce.ts` | Exports are dead because the file itself has no import path from entries. | This is the simple case that works; it highlights the unreachable-cycle contrast. | WT-22 |
| Pre-write service sibling: `searchUser` intent vs existing `fetchUser` | `nearNames` and `semanticHints` can both be empty because edit distance/prefix and exact token overlap are too strict for verb-different service siblings; first-slice diagnostics now preserve the suppressed candidate reason instead of dropping it silently. | The pre-write “already exists nearby” cue still needs a calibrated review cue, but reviewers can now inspect why the candidate was suppressed. | WT-23 |
| Conventional package exports such as Hono conditional exports | Conditional export targets plus `dist -> src` probing and transitive barrel walking can work when the compiled target maps cleanly. | This is useful but should not be described as a complete source-map resolver. | WT-20 |
| Non-standard compiled/source layouts or missing build artifacts | If the first public entry seed is absent, the transitive walker cannot recover downstream public surface. | Public-surface shrinkage can create false dead-export confidence. | WT-20 |
| Large monorepo runs such as cal.diy / next.js-canary | Sequential producer orchestration, repeated JSON parsing, repeated AST parsing, and high peak RSS were reported. | Agent-loop latency and memory pressure can make the skill impractical. | WT-18 |

Use this table as provenance, not as a permanent benchmark. Any row promoted to
implementation must get a minimal fixture or a recorded corpus check before it
can move beyond `SPEC`/`VERIFY`.

## 3. Performance Diagnosis Contract

Reported measurements point at architectural cost:

- sequential `spawnSync` producer orchestration,
- repeated JSON parse of large artifacts,
- repeated AST parse across producers,
- process startup cost,
- high peak memory on large repositories.

These are plausible, but optimization must be evidence-led.

Before parallelizing or moving producers into one process, the engine needs a
stable timing and memory artifact. Without it, a 4-way scheduler can reduce wall
time while making peak RSS unacceptable.

## 4. Required Measurement Artifact

Add a future artifact such as:

```text
producer-performance.json
```

Minimum shape:

```json
{
  "schemaVersion": "producer-performance.v1",
  "root": "/repo",
  "profile": "full",
  "totalWallMs": 60000,
  "peakRssMb": 886,
  "producers": [
    {
      "name": "build-symbol-graph",
      "wallMs": 12000,
      "exitCode": 0,
      "artifactBytesWritten": 16777216,
      "phaseTimings": {
        "fileWalkMs": 100,
        "astParseMs": 1500,
        "analysisMs": 6000,
        "jsonReadMs": 0,
        "jsonWriteMs": 300,
        "serializeMs": 200
      },
      "resourceUsage": {
        "maxRssMb": 512,
        "userCpuMs": 9000,
        "systemCpuMs": 800
      }
    }
  ],
  "artifactReads": [
    {
      "artifact": "symbols.json",
      "readCount": 4,
      "totalBytesRead": 67108864,
      "totalParseMs": 1800
    }
  ]
}
```

Implementation can start with orchestrator-level wall time and artifact byte
counts, then add producer-level phase timings as each heavy producer is touched.

## 5. Optimization Order

Do not start with Rust, rayon, or a broad single-process rewrite.

Preferred order:

1. **Measurement first.** Emit producer timing, artifact size, parse/read counts,
   and memory metadata.
2. **Dependency graph declaration.** Record which producers depend on which
   artifacts. This is prerequisite for safe parallelism.
3. **Bounded parallel scheduler.** Parallelize only independent producers and
   cap concurrency by memory budget.
4. **Shared artifact read cache.** Avoid parsing `symbols.json` and other large
   artifacts repeatedly in the same process when the orchestrator can safely
   share them.
5. **Shared AST cache or single-process runner.** Only after producer contracts
   are function-call friendly and memory behavior is measured.
6. **Rust/rayon exploration.** Only if measured CPU-bound traversal remains the
   dominant cost after orchestration and redundant I/O are reduced.

## 6. Acceptance Criteria

Recall acceptance:

- Namespace re-export exact-member fixtures prove that unused source members
  remain dead.
- `import.meta.glob` literal fixtures either create concrete edges or emit
  explicit unsupported-family diagnostics.
- Class-method pre-write fixtures surface existing methods before weak common
  prefix matches.
- Missing support for any of these surfaces appears in artifacts as a blind
  zone or unavailable evidence, not as grounded absence.

Performance acceptance:

- Full audit emits per-producer wall time and at least coarse memory metadata.
- Reports distinguish algorithm time, JSON read/parse time, AST parse time, and
  orchestration overhead when available.
- A large monorepo run records peak RSS and artifact sizes.
- No parallelism PR is accepted without a dependency graph, a memory cap, and
  before/after measurements on at least one large corpus.

## 7. Non-Goals

- Do not turn weak recall surfaces into positive `SAFE_FIX` evidence.
- Do not make `import.meta.glob` non-literal expressions resolved by guessing.
- Do not add embeddings or synonym dictionaries to compensate for missing
  method surfaces.
- Do not claim performance parity with another tool from a single corpus run.
  Record the measurement as a warning until the benchmark harness is stable.


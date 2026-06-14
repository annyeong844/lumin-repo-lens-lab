# Lumin-Fused SAFER Graph

> **Role:** maintainer-facing performance architecture spec for reducing repeated
> JS/TS parsing and graph work without weakening evidence contracts.
> **Status:** SPEC.
> **Last updated:** 2026-05-10

---

## 1. Problem

Large-repo feedback shows that Lumin's slow path is not primarily the SCC
algorithm. The larger cost is architectural:

- several producers walk the same file set,
- several producers read the same files,
- several producers call `parseOxcOrThrow(...)` on the same JS/TS files,
- later producers repeatedly parse large JSON artifacts,
- `audit-repo.mjs` still launches producers as sequential child processes,
- topology has a legacy incremental mode, but public audit flow does not route
  the shared incremental flags through it.

This matters because Lumin is a plugin/skill. It can be technically accurate
and still fail the product if full audits are too expensive for agent loops or
public CI verification.

## 2. North Star

Move from:

```text
Full AST parse first, in each producer.
```

to:

```text
Prove cheap module-edge facts lexically where safe.
Parse only ambiguous files.
Fuse parse-heavy facts when full-profile work needs the AST anyway.
Reuse edge hashes before rebuilding graph and SCC outputs.
```

The goal is not to invent a second JavaScript parser. The goal is to avoid
building an AST when the producer only needs import/export edge strings and the
scanner can prove that it has seen all relevant module edges. Ambiguous files
must fall back to the existing Oxc path.

## 3. Design Principles

- **Correctness before speed.** Fast paths are admissible only when their output
  is equivalent to the existing producer contract for that file. Otherwise,
  fallback to `parseOxcOrThrow(...)`.
- **Measurement before architecture churn.** Every phase that claims speed must
  add or consume timing counters that can prove parser calls, fallback rate, JSON
  read/parse cost, or graph rebuild cost changed.
- **No hidden analyzer mode.** Existing public artifacts remain the contract.
  Faster internals must emit the same review-relevant facts unless a narrower
  spec explicitly changes a fact contract.
- **Phase gates over mega rewrites.** Start with topology/barrel scanner paths
  and resolver memoization. Full producer fusion comes only after smaller phases
  show measured value.
- **Scanner output is evidence, not trust.** Scanner artifacts should report
  accepted files, fallback files, and risk reasons so reviewers can see why a
  file avoided AST parsing.

## 4. Fast Module-Edge Scanner

### 4.1 Scope

Add a small tokenizer-state scanner for JS/TS module edges. This is not a regex
extractor. It may accept a file only after it has confidently skipped comments,
strings, regex literals, JSX text, and template literal bodies, or has fallen
back to Oxc.

The first supported subset should be deliberately narrow:

```text
import ... from "x"
import type ... from "x"
import "x"
export ... from "x"
export type ... from "x"
export * from "x"
export type * from "x"
import("literal")
```

Import/export attributes may be accepted only when the module specifier is a
string literal and the scanner can skip the trailing `with { ... }` or
`assert { ... }` syntax without state uncertainty:

```ts
import data from "./data.json" with { type: "json" };
export * from "./data.json" assert { type: "json" };
```

If the scanner cannot confidently skip the attribute or assertion syntax, the
file falls back.

The scanner may return:

```json
{
  "ok": true,
  "mode": "fast-module-edge",
  "loc": 120,
  "edges": [
    {
      "source": "./x",
      "typeOnly": false,
      "reExport": false,
      "dynamic": false,
      "line": 8
    }
  ],
  "risk": []
}
```

### 4.2 Fallback Conditions

The scanner must reject and fall back when it sees constructs that can affect
module-edge accuracy or cannot be confidently skipped:

- non-literal `import(...)`,
- dynamic `import(...)` with options that the scanner cannot skip confidently,
- template literal dynamic imports,
- `require(...)`,
- `require.context(...)`,
- `import.meta.glob(...)`,
- TypeScript `import foo = require("foo")`,
- TypeScript `export = foo`,
- TypeScript `declare module "foo"`,
- import/export attributes or assertions that cannot be skipped confidently,
- decorator or reflection syntax that a later producer relies on,
- scanner state uncertainty around comments, regex, strings, or template
  expressions,
- syntax that appears malformed or unsupported by the scanner,
- any pattern that needs symbol/member semantics rather than file-level module
  edges.

Rejecting too often is acceptable for the first slice. Accepting an ambiguous
file is not.

### 4.3 Risk Metadata

Risk strings should be stable and versioned enough for tests:

```text
non-literal-dynamic-import
template-dynamic-import
require-call
require-context
import-meta-glob
dynamic-import-options
ts-import-equals
ts-export-assignment
ts-ambient-module
import-attribute-unsupported
decorator-or-reflect
scanner-state-ambiguous
unsupported-syntax
```

The exact list may grow, but new risk strings need fixture coverage if they
change whether a file uses fast path or fallback.

### 4.4 Equivalence Contract

For every accepted file, scanner-derived module-edge facts must be equivalent
to Oxc-derived module-edge facts after applying the same normalization,
resolver, type-edge lens, and include-test policy.

The implementation should include a fixture or calibration "shadow mode":

```text
run scanner
if scanner says ok:
  run Oxc extractor too
  compare normalized module-edge facts
  fail on mismatch
```

This mode does not need to run in production. It exists so fast-path acceptance
is backed by a mechanical equivalence check.

## 5. First Consumers

### 5.1 `measure-topology.mjs`

Topology is the safest first consumer because it mostly needs file-level module
edges:

- static imports,
- static re-exports,
- literal dynamic imports,
- type-only edge labels.

The first implementation should:

1. read source,
2. run the scanner,
3. resolve scanner edges if `ok`,
4. otherwise run the existing `parseOxcOrThrow(...)` path unchanged.

The topology artifact should add coarse metadata such as:

```json
{
  "scanner": {
    "policyVersion": "module-edge-scanner-v1",
    "acceptedFiles": 120,
    "fallbackFiles": 30,
    "riskCounts": {
      "require-call": 18,
      "import-meta-glob": 3
    }
  }
}
```

This metadata is diagnostic. It must not affect ranking.

### 5.2 `check-barrel-discipline.mjs`

Barrel discipline currently parses each JS/TS file to collect import and
re-export source strings. It should use the same scanner once topology proves
the scanner contract.

The scanner path may provide:

- source specifier,
- import versus re-export,
- type-only label where available,
- line number for reporting.

Barrel discipline also preserves nearby `eslint-disable` behavior today. The
first scanner slice should not try to become a full comment-directive engine. If
a file contains relevant `eslint-disable` / `eslint-disable-next-line`
directives near import or re-export statements, either the scanner must return
directive metadata with line numbers, or the file must fall back with a stable
reason such as:

```text
barrel-eslint-directive-present
```

Fallback remains the existing Oxc path.

### 5.3 Symbol Graph Consumer Fast Path

`extract-ts.mjs` is more complex because it owns definitions, uses, CJS
surfaces, dynamic opacity, and namespace/member precision. Do not replace it
wholesale with scanner output.

A later narrow fast path may skip AST only when the scanner can prove:

- the file has no top-level exports,
- no CommonJS export surface exists,
- imports are literal and fully represented,
- no non-literal dynamic import or `require` opacity exists,
- no member/symbol facts are needed from the file.

The symbol fast path may emit scanner-derived import-use facts only for forms
whose exported-name identity is fully represented by the scanner:

- named imports with literal source,
- default imports when default identity is modeled by the consumer,
- side-effect imports as file reachability only.

It must fall back for namespace imports, `import = require`, re-export alias
ambiguity, CommonJS require, member access, or any form where exported-name
identity is not fully represented.

When the fast path emits empty arrays, it must also emit proof state:

```json
{
  "symbol": {
    "status": "complete-fast-no-defs",
    "defs": [],
    "uses": [],
    "completeness": {
      "topLevelExportsProvenAbsent": true,
      "cjsExportSurfaceProvenAbsent": true,
      "namespaceMemberUsesModeled": false
    }
  }
}
```

An empty array without a complete section status is not absence proof. If any
condition is uncertain, use the existing AST path.

## 6. Edge Hash And Topology Incremental

Topology should migrate away from legacy producer-local incremental semantics
toward the strict shared cache shape used by newer producers. The cache payload
should include:

```json
{
  "contentHash": "sha256:...",
  "edgeHash": "sha256:...",
  "edges": [],
  "loc": 120,
  "parseError": false,
  "extractionMode": "fast-module-edge"
}
```

`edgeHash` is computed from sorted graph-affecting edge facts, after resolver
normalization:

```text
specifier
resolutionStatus: resolved | external | unresolved_internal | non_source_asset
target or sentinel identity
typeOnly
dynamic
reExport
```

Raw specifier strings are not enough. A raw `./x` edge can resolve to a
different target when resolver context changes, and SCC output depends on the
resolved target or sentinel.

If a file content hash changes but its edge hash is unchanged, topology graph
and SCC outputs should not be treated as changed for graph purposes. If the
repo-level graph identity hash is unchanged, fan-in/fan-out/SCC sections may be
reused from the previous topology artifact, while LOC and parse/read summaries
still refresh.

The repo graph identity must include:

```text
graphPolicyVersion
resolverVersion or resolverContextHash
conditionProfile
scannerPolicyVersion
scan policy and include-test policy
type-edge lens
sorted included file node ids
sorted graph-affecting resolved edges
```

The file set matters. A new edge-free file may not change SCC edges, but it
does change the topology node set and summary surfaces.

`parseError` and `extractionMode` are not graph edges, but they are diagnostic
identity inputs. A transition into or out of parse error must prevent blind
reuse of previous diagnostic sections.

This optimization is safe only if the previous aggregate artifact and the
current cache identity share the same resolver, scan, include-test, and
type-edge lens context.

## 7. Resolver Memoization

Resolver calls should be memoized inside a producer run. The first safe key is:

```text
fromFile + "\0" + specifier
```

If the resolver supports multiple resolution modes or context profiles, the key
must also account for them. A complete conceptual key is:

```text
resolverVersion
repoRoot
fromFile
specifier
resolutionMode: import | require | dynamic-import | type
conditionProfile
tsconfigIdentity
packageScopeIdentity
generatedArtifactPolicyVersion
includeTestPolicyVersion
```

A memoization key may omit a context field only when the resolver instance
guarantees that field is constant for the entire run.

A directory-based key may produce better hit rates:

```text
dirname(fromFile) + "\0" + specifier
```

but it is forbidden until fixtures prove the resolver result cannot vary by
exact importer path inside that directory under tsconfig, package-scope,
condition-profile, and generated virtual rules.

Memoization must preserve sentinel results:

- `EXTERNAL`,
- `UNRESOLVED_INTERNAL`,
- `NON_SOURCE_ASSET`,
- generated virtual resolution objects,
- `null`.

Memoized unresolved or unsupported results must preserve the same reason code,
resolver family, provenance, generated-virtual payload, and blind-zone
diagnostics as an uncached result. A cache hit must not make diagnostics
disappear.

The cache belongs to one resolver instance and one run. It is not a persistent
cache in this slice.

## 8. File Walk Reduction

`triage-repo.mjs` currently collects TS, JS, Python, and Go files through
separate `collectFiles(...)` calls. It should collect the union once and split
by extension in memory.

This is intentionally small. It is useful because it reduces repeated directory
walks without touching proof semantics.

## 9. Full-Profile JS Fact Fusion

The long-term performance win is to stop parsing a file once per producer.
This phase is the riskiest part of the roadmap and should move slowly.

Add a future producer:

```text
build-js-facts.mjs
```

It should produce file-scoped fact payloads per JS/TS source. A single
monolithic JSON file is not required and may be counterproductive if every
consumer repeatedly parses sections it does not need. Prefer a sectioned format
that lets consumers read only compatible facts:

```text
js-facts/
  manifest.json
  files.ndjson
  module-edges.ndjson
  symbol-facts.ndjson
  call-facts.ndjson
  shape-facts.ndjson
  function-fingerprints.ndjson
```

If a JSON wrapper is used, each section still needs explicit completeness
metadata:

```json
{
  "schemaVersion": "js-facts.v1",
  "files": [
    {
      "relPath": "src/a.ts",
      "contentHash": "sha256:...",
      "edgeHash": "sha256:...",
      "loc": 120,
      "parse": {
        "mode": "fast-module-edge",
        "error": null
      },
      "moduleEdges": {
        "status": "complete",
        "facts": []
      },
      "importsForBarrel": {
        "status": "complete",
        "facts": []
      },
      "symbol": {
        "status": "complete",
        "mode": "full-ast",
        "defs": [],
        "uses": [],
        "reExports": [],
        "typeEscapes": [],
        "dynamicImportOpacity": [],
        "cjsExportSurface": null,
        "cjsRequireOpacity": []
      },
      "call": {
        "status": "complete",
        "importMap": [],
        "calls": [],
        "namespaceMethodCalls": [],
        "importedObjectMemberCalls": [],
        "prototypeCalls": [],
        "exportedObjectMaps": []
      },
      "shapeFacts": {
        "status": "complete",
        "facts": []
      },
      "functionCloneFingerprints": {
        "status": "complete",
        "facts": []
      }
    }
  ]
}
```

An empty fact array is proof only when the corresponding section status is
`complete`. If a section is `not-computed`, `unavailable`, `fallback-required`,
or absent, consumers must behave as if compatible facts were not provided.

Function clone data in `js-facts` must remain per-file/per-function normalized
fingerprints. Exact groups, structure groups, and near candidates remain
cross-file aggregation outputs produced by `function-clones.json`; they must
not be cached as file facts.

Existing producers should learn `--facts <path>` one at a time:

```text
measure-topology.mjs
check-barrel-discipline.mjs
build-symbol-graph.mjs
build-call-graph.mjs
build-shape-index.mjs
build-function-clone-index.mjs
```

The migration rule is simple: if a producer receives compatible facts, consume
them; otherwise keep the existing self-contained behavior. Do not make the
entire audit depend on `js-facts.json` until each consumer has regression
coverage.

## 10. Scheduler And Parallelism

Bounded producer parallelism remains useful, but it should not be the first
large rewrite. Producer fusion may remove or shrink the biggest parse-heavy
steps, changing the scheduler's ROI.

Before a scheduler lands, Lumin needs:

- producer dependency declarations,
- child-process or producer peak memory visibility,
- artifact read/parse counters,
- a memory cap,
- before/after measurements on at least one large corpus.

Parallelism without memory evidence can make public CI and local agent loops
less reliable even if wall time improves.

## 11. SCC Work

SCC replacement is not the first target. Current SCC cost should be measured
separately from extraction and resolution before changing the algorithm.

Reasonable later improvements:

- directed trim before SCC,
- iterative SCC to avoid recursion risk,
- dirty-region recompute when topology edge hashes change only in a bounded
  part of the graph.

Prediction-augmented SCC and watch-session region prediction are parked until
watch mode has real usage data. They should not be implemented from speculative
benchmark appeal alone.

## 12. Phased Plan

### Phase 0: Measurement Guardrails

- Keep `producer-performance.json`.
- Add parser-call and scanner/fallback counters only when touching a producer.
- Add artifact read/parse counters before optimizing JSON I/O.
- Record enough counters to compare cold and warm runs:

```json
{
  "filesRead": 1200,
  "bytesRead": 8300000,
  "scanner": {
    "filesAttempted": 1000,
    "acceptedFiles": 760,
    "fallbackFiles": 240,
    "scannerMs": 180,
    "riskCounts": {
      "require-call": 80,
      "scanner-state-ambiguous": 12
    }
  },
  "parser": {
    "oxcCalls": 240,
    "parseMs": 920
  },
  "resolver": {
    "calls": 4300,
    "cacheHits": 1700,
    "cacheMisses": 2600,
    "resolveMs": 310
  },
  "artifacts": {
    "jsonReadBytes": 1200000,
    "jsonParseMs": 90
  }
}
```

### Phase 1: Low-Risk Local Wins

- Add resolver run-local memoization.
- Change `triage-repo.mjs` to single `collectFiles(...)` walk.
- Inventory existing topology incremental behavior.
- Do not route topology incremental reuse through public audit flow unless
  strict identity is already satisfied, or the mode is explicit,
  experimental, and off by default.

### Phase 2: Topology Scanner

- Add `js-module-edge-scanner.mjs`.
- Use it in `measure-topology.mjs`.
- Emit scanner accepted/fallback counts.
- Preserve existing Oxc fallback output.

### Phase 3: Barrel Scanner

- Reuse the scanner in `check-barrel-discipline.mjs`.
- Preserve line-number and eslint-disable behavior.

### Phase 4: Symbol Consumer Fast Path

- Add no-export consumer fast path in `extract-ts.mjs`.
- Keep CJS, namespace, dynamic, and member-precision files on the AST path.

### Phase 5: Strict Topology Incremental

- Migrate topology to strict shared incremental identity.
- Add per-file edge hash.
- Add repo graph identity hash.
- Reuse graph/SCC aggregate only when graph identity is unchanged.

### Phase 6: JS Facts Fusion

- Add `build-js-facts.mjs`.
- Migrate parse-heavy producers to `--facts` one at a time.
- Only then reconsider bounded producer parallelism.

## 13. Acceptance Criteria

Before this performance architecture is considered beyond `SPEC`:

- topology fast path has fixtures proving scanner output matches Oxc output for
  accepted files,
- scanner accepted files are compared against Oxc-derived normalized module
  edges in fixture or shadow mode,
- scanner fixtures cover comments, strings, regex literals, JSX text, and
  template literals containing fake import/export text,
- import attributes/assertions are either accepted with fixtures or fall back
  with stable reason codes,
- TypeScript import-equals, export-assignment, and ambient-module forms either
  produce correct edges or fall back with stable reason codes,
- scanner fallback fixtures cover every risk reason,
- barrel scanner path preserves eslint-disable behavior, or files with relevant
  eslint directives fall back,
- topology artifacts expose accepted/fallback counts,
- resolver memoization tests prove sentinel and generated virtual results are
  preserved,
- resolver memoization cache hits preserve unresolved/unsupported reason codes
  and blind-zone diagnostics,
- `triage-repo.mjs` emits the same language counts after single-walk collection,
- topology incremental does not reuse graph outputs across changed resolver,
  scan, include-test, or type-edge lens context,
- repo graph identity includes included file node set, resolved edge facts,
  resolver identity, scan policy, include-test policy, and type-edge lens,
- `js-facts` consumers treat empty arrays as proof only when the corresponding
  section status is complete,
- a producer receiving incompatible or incomplete facts behaves exactly as if
  `--facts` was not provided,
- topology incremental is not routed through public audit flow unless strict
  shared cache identity is satisfied or the mode is explicitly experimental,
- any claimed speedup includes before/after `producer-performance.json` data,
- speedup claims identify cold versus warm cache and should use the same corpus,
  include policy, resolver profile, wall time, parser calls, and peak memory,
- no PR claims runtime parity with another tool from a single corpus.

## 14. Non-Goals

- Do not implement a general JS parser in the scanner.
- Do not parse CJS or dynamic glob semantics in the first scanner slice.
- Do not make scanner output positive `SAFE_FIX` evidence.
- Do not collapse parse-heavy producers into `build-js-facts.mjs` before local
  scanner and measurement phases land.
- Do not switch to Rust/rayon until repeated parse/I/O costs are reduced and
  measured CPU-bound traversal remains dominant.

## 15. Relationship To Existing Specs

- `lumin-architecture-realignment.md` owns the broad architecture direction.
  This spec narrows its P5 performance architecture into an implementation
  sequence.
- `recall-and-performance-gap-plan.md` owns the corpus feedback inventory and
  high-level performance diagnosis.
- `incremental-engine-architecture.md` owns cache correctness. Topology
  incremental migration must follow that contract.
- `agent-entry-resolver-calibration.md` owns threshold and resolver capability
  discipline. Scanner and memoization changes must not weaken unsupported-family
  reporting.

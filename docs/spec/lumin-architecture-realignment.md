# Lumin Architecture Realignment

> **Role:** maintainer-facing architecture direction for moving Lumin Repo Lens
> away from unbounded heuristic accumulation.
> **Status:** SPEC.
> **Last updated:** 2026-05-09

---

## 1. Problem

Recent corpus work exposed a recurring failure mode: each new JS/TS ecosystem
shape can be patched locally, but a growing list of special cases does not scale
to real repositories.

Observed examples include:

- workspace manifests where `package.json#workspaces` and
  `pnpm-workspace.yaml` both contribute package roots,
- nested framework apps that are not declared as root workspaces,
- framework-owned route and resource files such as Next.js app routes,
  Storybook stories, Strapi controllers, scaffold templates, and generated
  declaration surfaces,
- namespace re-exports, class methods, and dynamic import surfaces that require
  member-aware modeling,
- source-first or build-output package layouts that cannot be represented by a
  fixed list of output-to-source directory pairs,
- large monorepos where repeated subprocess orchestration, repeated JSON parse,
  repeated AST parse, and peak RSS make the skill too expensive for agent loops.

The core issue is not that any single heuristic is wrong. The issue is that
heuristics are currently too close to the main evidence path. Lumin needs an
architecture where ecosystem knowledge is explicit, versioned, testable, and
allowed to say "unsupported" without silently weakening or overclaiming
deadness.

## 2. North Star

Lumin should be a **repo evidence broker**, not a bag of dead-code heuristics.

This is not a feature reduction. The goal is to support more repositories over
time by making support units explicit:

```text
capability detected
  -> resolver/symbol/surface evidence emitted
  -> unsupported areas reported as blind zones
  -> ranking consumes only grounded proof
```

Supported areas may produce stronger claims. Unsupported or partially supported
areas must be visible in artifacts and must not be translated into silence or
repo-global panic.

## 3. Architecture Pillars

### 3.1 Resolver Core

The resolver should become a staged pipeline with explicit output states:

- `resolved`: a concrete graph edge may be created.
- `candidate`: a plausible target exists, but proof requirements for a graph
  edge are not met.
- `unresolved`: the engine tried supported rules and failed.
- `unsupported`: the import family is recognized but not implemented.
- `external`: the specifier is outside the scanned repository.

Only `resolved` creates concrete graph edges. `candidate`, `unresolved`, and
`unsupported` may block or limit absence claims only when relevant to the
candidate package, file, export, or re-export surface under review.

Resolver inputs should be collected additively from supported manifest sources.
No supported source should shadow another by default. For example, workspace
pattern discovery must not choose between `package.json#workspaces` and a
workspace manifest when both are present; it must collect both and dedupe.

### 3.2 Capability Packs

Framework and ecosystem knowledge should move toward capability packs. A pack is
not just a function with file-name checks. A pack owns:

- the detection rule,
- the supported file/resource surfaces,
- the artifact fields it emits,
- fixtures and corpus provenance,
- blind-zone behavior for unsupported shapes,
- the ranking interaction policy.

Examples:

- `framework.next`
- `framework.storybook`
- `framework.strapi`
- `runtime.cloudflare-worker`
- `surface.import-meta-glob`
- `surface.generated-prisma`
- `package.output-to-source`

Capability packs may start as in-process modules. They do not need a plugin
loader in the first slice. The important boundary is contract ownership: a pack
must say what it supports, what it refuses to infer, and how downstream ranking
should treat its evidence.

### 3.3 Symbol And Member Model

The symbol model must distinguish top-level definitions from member surfaces.
Pre-write and deadness recall cannot rely on exported/top-level definitions
alone.

Required surfaces:

- top-level exported definitions,
- local declarations,
- class methods and accessors,
- object literal methods when they participate in exported surfaces,
- namespace re-export members,
- bounded member reads such as `ns.used`,
- opaque member escapes that force confidence limits.

Member evidence is not automatically action proof. For pre-write, class methods
and object members should usually become `AGENT_REVIEW_CUE` evidence. For
deadness, exact member reads may protect only the observed member; opaque member
usage should become scoped blind-zone evidence.

### 3.4 Dynamic And Generated Surfaces

Dynamic and generated surfaces need provenance, not guesswork.

Examples:

- `import.meta.glob("./routes/*.ts")`,
- framework filesystem routes,
- generated `.d.ts` files,
- Prisma enum/client surfaces,
- bundled or compiled resources,
- scaffold/template files.

Literal, scan-policy-compatible dynamic surfaces may produce concrete edges.
Non-literal or unsupported surfaces must emit diagnostics. Generated artifacts
may support graph construction only when their source, mode, staleness, and
completeness are explicit.

Generated or dynamic evidence must not become positive `SAFE_FIX` evidence by
itself.

### 3.5 Evidence Artifacts

Every major claim should be backed by machine-readable artifacts. Prose is for
humans; JSON fields own the contract.

Required artifact directions:

- static capability matrix: what this engine version knows how to evaluate,
- per-run resolver diagnostics: what this repository exposed,
- per-run blind zones: what absence claims are limited and why,
- capability pack summaries: which packs activated, rejected, or stayed
  unavailable,
- threshold policy metadata: which numeric thresholds affected ranking,
  rendering, suppression, or pruning,
- producer performance metadata: what the run cost and where time/memory went.

`manifest.json` should summarize and point to full artifacts. Large raw evidence
should live in focused artifacts rather than bloating the manifest.

### 3.6 Performance Orchestrator

Lumin is a skill/plugin. If it becomes too slow or memory-heavy for agent loops,
the product fails even when analysis quality improves.

The current producer architecture should move toward:

- producer dependency graph declaration,
- orchestrator-level timing and artifact-size metadata,
- memory-aware bounded parallelism,
- shared artifact read cache where safe,
- shared AST or single-process producers only after measurement proves the
  memory and lifecycle contract,
- Rust/rayon exploration only after orchestration and redundant I/O costs are
  measured and reduced.

Performance work must be evidence-led. Do not replace sequential simplicity with
blind parallelism before peak RSS is visible.

## 4. Hard Rules

- A `candidate` resolver result is not a graph edge.
- Unsupported resolver or capability families must be visible in artifacts.
- A blind zone must be scoped before it blocks `SAFE_FIX`; it must not become
  repo-global by default.
- Framework/resource support must come with fixtures or corpus provenance.
- Numeric thresholds must belong to named, versioned policies.
- Ranking must not consume review-only annotations as promotion evidence.
- Generated/dynamic surfaces must carry provenance and completeness labels.
- Performance changes must include before/after measurement hooks or artifacts.
- No implementation slice should claim a capability is complete because a single
  corpus passed.

## 5. What We Stop Doing

Stop adding behavior in these forms:

- unversioned output-directory pair lists that silently affect public surface,
- framework file-name checks without capability identity and fixtures,
- broad namespace or member liveness propagation,
- pre-write wording that turns missing evidence into `NOT_OBSERVED` absence,
- score-only promotion to `SAFE_FIX`,
- repo-global taint when package or dependency relevance can be scoped,
- performance rewrites without phase timing and memory evidence.

These patterns may still exist in legacy code. New PRs should avoid extending
them unless the PR is explicitly paying down the legacy pattern.

## 6. What We Keep

Keep the parts that are already aligned with the long-term direction:

- PCEF proof-carrying edit actions,
- `safeAction` and action-safety proof,
- `actionBlockers` versus `strongerActionBlockers`,
- grouped safe actions,
- resolver blocked absence hints,
- generated artifact missing as blind-zone evidence rather than fake resolved
  files,
- pre-write evidence availability,
- install verification before marking user-visible work `DONE`,
- small PR slices with fixtures.

## 7. Phases

### P0: Contract Landing

- Land this architecture spec.
- Link it from the spec README and work tracker.
- Treat existing gap specs as supporting debt inventories, not architecture
  owners.

### P1: Resolver Capability Backbone

- Emit a static resolver capability matrix.
- Emit per-run resolver diagnostics separately from symbol artifacts.
- Define relevance-scoped blocking for unresolved and unsupported families.
- Keep existing resolver behavior where possible, but route new resolver work
  through the staged output states.

### P2: Symbol And Member Surface

- Add a member index for class methods and selected object-member surfaces.
- Add pre-write method review cues.
- Keep member evidence out of `SAFE_FIX` unless paired with explicit proof.
- Expand namespace re-export member propagation with chained and opaque escape
  fixtures.

### P3: Capability Pack Structure

- Convert framework/resource surface logic into named capability modules.
- Start with existing Next, Storybook, Strapi, Cloudflare, and config sentinel
  behavior.
- Each pack must emit support status and fixture coverage.
- Ranking consumes pack evidence only through documented evidence lanes.

### P4: Dynamic And Generated Surface Model

- Implement literal `import.meta.glob` as a supported dynamic surface or report
  it as unavailable.
- Strengthen generated artifact evidence quorum, mode precedence, staleness, and
  relevance-scoped blocking.
- Keep virtual surfaces partial and provenance-labeled.

### P5: Performance Architecture

- Add `producer-performance.json` or equivalent metadata.
- Record artifact read/parse counts and producer wall time.
- Declare producer dependencies.
- Add memory-aware parallelism only after measurement.
- Explore shared artifact/AST caches after single-run overhead is visible.

### P6: Optional Deep Analysis

Only after P1-P5 are stable, evaluate deeper analysis:

- bounded interprocedural call graph improvements,
- limited escape analysis,
- optional type-flow-assisted analysis,
- Rust/rayon implementation for measured CPU-bound phases.

This is not the default path for ordinary skill execution.

## 8. Relationship To Existing Specs

- `recall-and-performance-gap-plan.md` records concrete recall and performance
  failures. This spec defines the architecture that should absorb those
  failures.
- `agent-entry-resolver-calibration.md` owns agent entry friction, resolver
  completeness metadata, and threshold policy debt.
- `generated-artifact-support.md` owns generated artifact provenance.
- `framework-resource-surface-policy.md` owns current resource surface
  diagnostics.
- `proof-carrying-export-fix.md` remains the SAFE_FIX contract owner.
- `incremental-engine-architecture.md` remains the cache correctness owner.

If this document and a narrower spec disagree, the narrower spec owns the exact
implementation contract, but the conflict should be resolved by updating one of
the documents. Do not let implicit code behavior settle architecture conflicts.

## 9. Acceptance Criteria

This spec is satisfied only when later work establishes these durable outcomes:

- resolver capabilities are visible as stable artifacts,
- unsupported import families can be reported without creating fake graph edges,
- framework support is declared by pack identity and fixture coverage,
- class method and namespace member surfaces have first-class review or proof
  lanes,
- generated and dynamic surfaces carry provenance,
- large-run performance has measurement artifacts before scheduler or
  single-process rewrites,
- ranking can explain why a candidate is safe, review-only, muted, degraded, or
  blocked by a scoped blind zone.

Until then, this document is an active architecture guide, not a completed
feature.

# P6 TypeScript Precision And Throughput Backlog

Status: draft
Date: 2026-04-22
Context: post-P5 dogfood, after P3 canon draft -> promoted canonical -> P5 drift check round-trip.

P5 closes the stickiness loop. It does not make the TypeScript analysis
semantically perfect. P6 exists to turn the current upper-intermediate /
advanced static analyzer into a more marketplace-ready TS analyzer with
measured false-positive budgets and faster large-repo runs.

Implementation rule: do not start P6-1 precision work before P6-0 locks
the measurement harness, false-positive denominator, sampling protocol,
runtime metrics, and corpus pinning.

## 1. Current Truth Table

### Resolved

- AST local reference counting is the primary path for TS/JS files.
  Regex counting is now an explicit parse-error fallback.
- JSX references are counted. Compound component patterns such as
  `<AlertDialogTrigger />` no longer disappear from same-file reference
  evidence.
- Common lexical shadowing is covered by the local reference counter:
  inner `const`, function parameters, arrow parameters, block `let` /
  `const`, catch parameters, for-loop bindings, named function
  expressions, and destructured parameters.
- Scoped tsconfig path resolution and workspace fallback aliases cover
  the major duyet-style FP-36 / FP-38 classes.
- P3/P5 schema drift is now dogfood-detectable. A draft that cannot be
  promoted and re-read by P5 is treated as a product bug, not user error.

### Partially Resolved

- Local reference counting is scope-aware, but not TypeScript
  checker-grade binding. `var` hoisting across blocks is approximated,
  and there is no full semantic symbol table for every identifier.
- Public API protection is still partly file-level. If a file is public
  through an entrypoint or re-export, unrelated exports in that file may
  be overprotected.
- Namespace imports and dynamic imports now have v1 direct-member
  precision for `ns.foo()`, `const mod = await import('./m'); mod.foo()`,
  and `import('./m').then((m) => m.foo())`. Computed members, aliased
  members such as `const f = ns.foo; f()`, and non-literal dynamic import
  paths still degrade conservatively.
- Package entrypoint detection has workspace `exports` / wildcard /
  `#imports` / `main` coverage, but not the full policy matrix for
  `bin`, `module`, `browser`, `types`, and script-driven entrypoints.
- Generated-file and framework sentinels exist as policies for known
  classes, but there is no general `isGeneratedFile()` auto-MUTE layer
  and no complete Cloudflare Pages Functions sentinel.
- `audit-repo.mjs --check-canon` now uses one child process when all P5
  sources are requested and the required primary artifacts are present,
  but full audit still reparses across multiple producers.

### Deferred

- TypeScript checker-grade symbol binding for local references.
- Symbol-level public API expansion.
- Namespace import and dynamic import member precision beyond the v1
  direct-call cases.
- Shared AST cache across `measure-topology.mjs`,
  `build-symbol-graph.mjs`, `build-call-graph.mjs`,
  `generate-canon-draft.mjs`, and `check-canon.mjs`.
- Generated-file auto-MUTE policy.
- Full package entrypoint policy.
- Cloudflare Pages Functions sentinel.
- Benchmark harness that records false-positive rate, runtime, and
  schema round-trip health on representative TS repos.

## 2. P6-0 Measurement Harness And Gates

P6-0 is the first implementation session. It creates the measurement
foundation used by every later precision and throughput phase. Session
contract: `docs/history/phases/p6/p6-0.md`.

### Required Outputs

- Baseline candidate counts by tier and source.
- Baseline false-positive adjudication data.
- Runtime step timings in `manifest.json`.
- Corpus commit pinning for each benchmark repo.
- P3/P5 schema round-trip health.
- Tool version, Node version, platform, include-tests mode, and exclude
  patterns.

### False-Positive Target Populations

"FP rate" is meaningless unless the denominator is pinned.

Primary:

- Review-visible cleanup candidates that would be shown to a user as
  removable, demotable, or cleanup-actionable.

Separate:

- `SAFE_FIX` precision. This is the strict marketplace-adjacent metric.
  A measured zero `SAFE_FIX` population is not a false-positive failure and
  not an unknown FP rate, but it cannot support Green/autonomous-cleanup
  wording because no `SAFE_FIX` precision was measured.
- Raw Tier C candidates. This remains raw evidence, not a claim.
- Canon-drift candidates. These are schema/register drift checks, not
  dead-export cleanup candidates.
- `DEGRADED` and `MUTED` counts. These are reported but not silently
  mixed into cleanup precision.

FP rate formula:

```text
false_positives / (true_positives + false_positives)
```

`inconclusive` is reported separately and excluded from both numerator
and denominator.

`not_applicable` is also excluded from the FP numerator and denominator,
like `inconclusive`, but reported separately as a sampling-quality
signal. Examples: the sampled item is outside the cleanup domain,
generated-policy-only, canon-drift-only, or fixture-malformed.

### Corpus Registry

P6-0 must write a corpus registry with immutable benchmark identity.
Minimum shape:

```json
{
  "corpus": [
    {
      "name": "duyet-monorepo",
      "repo": "local-or-remote-id",
      "commit": "abc123",
      "snapshotId": null,
      "worktreeDirty": false,
      "contentHash": null,
      "locBucket": "50k",
      "packageManager": "pnpm",
      "includeTests": false,
      "exclude": ["node_modules", "dist"],
      "reason": "multi-app workspace with scoped tsconfig aliases"
    }
  ]
}
```

P6-0 must refuse Green readiness if any corpus entry lacks a `commit` or
equivalent immutable snapshot id. If `worktreeDirty === true`, Green
readiness is also forbidden unless `snapshotId` or `contentHash` captures
the exact dirty state.

P6-0 must also distinguish missing artifacts from measured zero. If
`fix-plan.json` is absent, candidate counts are unavailable/null, not `0`,
and Green readiness is forbidden.

### Sampling Protocol

For each benchmark repo:

- Adjudicate at least 50 candidates, or all candidates if fewer than 50.
- Stratify the sample by label: `SAFE_FIX`, `REVIEW_FIX`, `DEGRADED`,
  and `MUTED`.
- Record each adjudication as `true_dead`, `false_positive`,
  `inconclusive`, or `not_applicable`.
- Record the FP ledger match, if any.
- Keep generated/framework muted candidates visible as policy results,
  even when they are excluded from cleanup precision.

### Readiness Gates

| Gate | Requirement | Allowed claim |
|---|---|---|
| Red | FP unknown, review-visible FP > 25%, or SAFE_FIX FP >= 5% | Advisory audit only |
| Yellow | Review-visible FP 10-25%, SAFE_FIX FP < 5% when non-empty, measured-zero SAFE_FIX population, or benchmark incomplete | Review-assisted cleanup candidates |
| Green | non-empty SAFE_FIX population with measured SAFE_FIX FP < 5%, review-visible FP < 10%, at least two non-trivial TS repos, at least 50 adjudicated candidates per repo or all if fewer, immutable corpus snapshot, schema round-trip attempted, zero known P3/P5 schema-drift bugs, no unresolved HIGH findings | Marketplace cleanup claim may be considered |

Regardless of gate, Tier C alone never means "truly dead."

### Throughput Baseline Metrics

P6-0 must record:

- repo size bucket: 25k / 50k / 100k LOC;
- wall time;
- per-step time;
- parse time, when available;
- file-walk time, when available;
- resolver construction time, when available;
- child process count;
- AST parse count;
- cache hit/miss counts, once cache exists;
- candidate counts before and after any throughput change.

Throughput changes must preserve candidate counts unless the phase
explicitly changes precision behavior and documents the expected delta.

## 3. Implementation Order

### P6-1 Package/Public Surface Model

Build the public surface model before symbol-level protection.

Scope:

- `exports`;
- `main`, `module`, `browser`, `types`;
- `bin`;
- workspace package surfaces;
- script-driven entrypoint hints.

Acceptance: package entrypoint fixtures produce an explicit public
surface artifact. Public surface evidence identifies both the file and
the entrypoint reason.

### P6-2 Symbol-Level Public API Protection

Replace file-level public API exclusion with symbol-level reachability.

Acceptance: a file with one re-exported public symbol and one private
unused export protects only the public symbol.

### P6-3 Namespace And Dynamic Import Member Precision

Resolve direct member uses to concrete exported symbols where possible.

Status: v1 direct-call support landed. Remaining work is the degraded
case family below and any broader TypeScript-checker-backed member flow.

Supported v1:

```ts
import * as ns from './mod';
ns.foo();

const mod = await import('./mod');
mod.foo();

import('./mod').then((m) => m.foo());
```

Degraded v1:

```ts
ns[dynamicName]();
const f = ns.foo; f();
const mod = await import(path);
```

Acceptance: `ns.foo()` protects `foo` without blanket-protecting
unrelated exports from the same module. Degraded cases carry an explicit
confidence downgrade.

### P6-4 Generated And Framework Sentinel Policy

Generated-file auto-MUTE is allowed only with evidence.

Evidence sources:

- generated header comment;
- path convention such as `generated/`, `__generated__`, or `.gen.ts`;
- lockfile or codegen manifest reference;
- sourcemap or `sourceMappingURL` relation;
- framework-specific generated directories.

MUTE only when at least one strong generated evidence source exists and
the user has not manually overridden the policy.

Artifact shape must preserve the reason:

```json
{
  "mutedReason": "generated-file",
  "evidence": "header: @generated"
}
```

Implementation must keep three policy families separate:

- generated-file auto-MUTE;
- framework-convention sentinels;
- package/public entrypoints.

Framework scope includes known config, Nuxt/Nitro, SvelteKit,
Cloudflare Pages Functions, and future framework sentinels. Cloudflare
Pages Functions are framework-convention sentinels, not generated-file
evidence by themselves.

### P6-5 Checker-Grade Local Binding

Do not make the checker path the default on day one. It starts as a
precision upgrade path.

Subphases:

- P6-5a Binding model design: TypeScript Program loading, per-app
  tsconfig selection, project references, monorepo boundaries, memory
  budget, and time budget.
- P6-5b Local reference parity fixtures: `var` hoisting,
  function/class/enum/type namespaces, declaration merging, type/value
  space separation, and nested scopes.
- P6-5c Opt-in checker path: default remains AST; checker path upgrades
  confidence only when the caller asks for stronger precision.

Acceptance: pinned fixtures match TypeScript semantics, and the opt-in
checker path reports its runtime cost.

### P6-6 Throughput Architecture

Share parsed ASTs and file metadata across producers in one audit run.

Scope:

- shared AST cache;
- resolver cache;
- shared file walk results;
- reduced child process fan-out;
- manifest timing for parse, walk, resolve, and child process phases.

Acceptance:

- repeated parse count reduced by at least 50% on a benchmark repo;
- `manifest.json` reports `parseCount`, `cacheHits`, and `cacheMisses`;
- candidate counts stay unchanged versus the uncached run unless an
  explicit precision phase also changed them.

## 4. P6-final Marketplace Gate

P6-final reruns the P6-0 benchmark harness after P6-1 through P6-6.
It decides wording from measured evidence, not intent.

Red wording:

- "Grounded structural audit for TS/JS/Python/Go repositories."
- "Produces evidence-backed review candidates with confidence labels."
- No cleanup safety claim.

Yellow wording:

- "Review-assisted cleanup candidates with measured caveats."
- "Human review required outside strict SAFE_FIX gates."

Green wording:

- "Review-assisted cleanup with a measured false-positive budget."
- "Autonomous removal may be considered only for SAFE_FIX under
  configured gates."
- "Config-gated autonomous removal for SAFE_FIX only, disabled by
  default."

Always forbidden:

- "Zero false positives."
- "Perfect TypeScript semantic analysis."
- "Fully autonomous cleanup for all findings."
- "Safe deletion" without naming the configured gate and benchmark
  result.

## 5. Current Marketplace Wording

Allowed now:

- "Grounded structural audit for TS/JS/Python/Go repositories."
- "Produces evidence-backed cleanup candidates with confidence labels."
- "Detects canonical drift across type, helper, topology, and naming
  registers."

Not allowed yet:

- "Automatically removes dead code safely."
- "Perfect TypeScript semantic analysis."
- "Zero false positives."
- "Marketplace-ready autonomous cleanup."

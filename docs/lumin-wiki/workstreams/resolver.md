# Resolver Workstream

Resolver work determines whether Lumin may create graph edges, report candidate
targets, or admit an unsupported blind zone. Resolver mistakes are high impact:
fake resolved edges contaminate reachability, while silent misses create false
absence claims.

## Current Themes

- Only resolved targets create concrete graph edges.
- Candidate and unsupported outputs are diagnostic evidence, not proof.
- Unsupported families must be named, scoped, and surfaced through resolver
  diagnostics.
- Generated, output-to-source, and dynamic-module surfaces should not become
  one-off heuristics.

## Test Inventory

| Suite | Risk Type | Protected Invariant | Edge Case Or Negative Guard |
|---|---|---|---|
| `tests/test-resolver-diagnostics-artifacts.mjs` | artifact shape | Resolver capabilities and per-run diagnostics serialize separately and deterministically. | Static capability metadata must not be confused with repo-specific unresolved evidence. |
| `tests/test-resolver-blind-zone-relevance.mjs` | blind-zone scoping | Resolver blind zones block only candidate-relevant absence claims. | An unrelated unresolved import must not become a repo-global blocker. |
| `tests/test-resolver-paths.mjs` | resolver regression | Core path resolution handles historical resolver edge cases. | Relative or alias misses must not silently fall through to external when they are internal-looking. |
| `tests/test-tsconfig-paths-scoped.mjs` | monorepo resolver regression | Per-scope `tsconfig` paths and baseUrl aliases resolve against the importing package/app. | The same `@/*` specifier may resolve to different files in different app scopes; missing local targets stay unresolved internal. |
| `tests/test-hash-imports.mjs` | Node package imports | Node `#imports` exact, wildcard, and suffix wildcard aliases resolve only when supported. | Unsupported imports maps must not degrade to external fallback or fake edges. |
| `tests/test-node-imports-unsupported.mjs` | unsupported-family diagnostic | Unsupported Node `#imports` surfaces emit a named family and no concrete edge. | Condition-profile ambiguity and unsupported imports stay diagnostic-only. |
| `tests/test-output-source-layout-diagnostics.mjs` | unsupported-family diagnostic | Non-standard package output/source layouts emit `output-to-source-mapping` diagnostics. | Compiled output paths without supported source mapping do not become deadness evidence or fake resolved edges. |
| `tests/test-import-meta-glob-diagnostics.mjs` | dynamic-module diagnostic | Unsupported `import.meta.glob` calls are recorded as dynamic-module blind zones. | Literal globs create no concrete graph edge until scan-policy-aware expansion exists. |
| `tests/test-generated-artifact-evidence.mjs` | generated evidence policy | Generated-artifact classification requires strong package/surface evidence. | Package name, dependency, or short path token alone must not promote a miss to generated evidence. |
| `tests/test-generated-blind-zone-relevance.mjs` | blind-zone scoping | Generated artifact blind zones block only relevant SAFE_FIX promotion. | Unrelated generated misses remain confidence limitations, not global blockers. |
| `tests/test-generated-consumer-blind-zones.mjs` | artifact shape | Missing or excluded generated consumer surfaces are listed in symbols and resolver summaries. | Generated consumers can block absence claims without being treated as observed source consumers. |
| `tests/test-generated-virtual-surface.mjs` | virtual surface contract | Supported virtual generated surfaces expose conservative import/export facts. | Virtual facts must not claim runtime equivalence or body/call evidence. |
| `tests/test-workspace-no-exports.mjs` | workspace package resolver | Workspace packages without `exports` still resolve supported legacy/source-direct subpaths. | The fix is additive; truly unused siblings remain dead and missing generated typings remain unresolved. |
| `tests/test-wildcard.mjs` | package exports wildcard | Package `exports` wildcard subpaths resolve through supported source/output mappings. | Missing wildcard targets stay unresolved internal, not external. |
| `tests/test-resolved-edges.mjs` | graph artifact shape | Resolved internal file-level edges are emitted for downstream reachability. | Only concrete resolved edges enter the graph artifact. |
| `tests/test-dynamic-import.mjs` | topology resolver behavior | Literal dynamic imports contribute topology edges. | Dynamic behavior must stay distinct from unsupported dynamic-module surfaces such as `import.meta.glob`. |

## Reform Direction

Resolver tests should include both the unsupported record and the absence of a
fake edge. A good resolver regression proves:

- no concrete graph edge was created
- the right family/reason was recorded
- affected candidates are scoped, not repo-global
- summary artifacts point readers to raw diagnostics

## Reform Targets

- Split resolver tests by output level: `resolved`, `candidate`, `unsupported`,
  `external`, and `unresolved_internal`.
- Keep every unsupported-family fixture paired with a "no fake graph edge"
  assertion.
- Compare generated, output-to-source, and dynamic-module fixtures for shared
  artifact-shape helpers before moving files.
- Keep condition-profile and workspace-scope cases separate; they protect
  different resolver identities.
- Do not widen resolver heuristics from a single fixture without adding a named
  capability or unsupported-family policy.

# Deadness Workstream

Deadness is the most sensitive Lumin claim. "No consumer was found in the
constructed graph" is not the same as "no consumer exists." SAFE_FIX must stay
behind explicit proof and scoped blockers.

## Current Themes

- Namespace re-export fan-in should protect used members exactly, not blanket
  protect every sibling.
- Entry-unreachable SCCs are review evidence for dead file groups, not export
  SAFE_FIX proof.
- Public API, generated consumers, framework conventions, and resolver blind
  zones can block or degrade absence claims.

## Test Inventory

| Suite | Risk Type | Protected Invariant | Edge Case Or Negative Guard |
|---|---|---|---|
| `tests/test-export-action-safety.mjs` | SAFE_FIX action proof | Export demotion/deletion actions carry proof, blockers, and syntax preservation facts. | Demote may be safe while delete remains blocked by local refs, class semantics, or side effects. |
| `tests/test-rank-fixes.mjs` | ranking contract | Fix-plan tiers merge evidence, blockers, and action safety into stable user-facing ranks. | Review evidence and soft confidence gaps must not silently promote to `SAFE_FIX`. |
| `tests/test-module-reachability.mjs` | file reachability | Runtime/type entry BFS records reachable, unreachable, and entry-unreachable SCC evidence. | Entry-unreachable SCCs are dead-file-group review evidence, not export-level SAFE_FIX proof. |
| `tests/test-namespace-reexport-deadness.mjs` | member precision regression | Namespace re-export fan-in protects used members exactly and leaves unused siblings visible. | Chained namespace reads must not blanket-protect every sibling; opaque escapes become diagnostics. |
| `tests/test-public-deep-import-risk.mjs` | public surface blocker | Public package exports/deep-import risk can block entry-unreachable confidence support. | Files excluded by `files` can reduce one blocker, but compiled artifacts are not proven absent. |
| `tests/test-public-surface.mjs` | public entry surface | Package, declaration, script-driven, and HTML entry surfaces feed public contract evidence. | Public entry detection must not create phantom files or overclaim unreachable source. |
| `tests/test-framework-resource-surfaces.mjs` | framework/resource blocker | Framework/resource lanes identify surfaces that can hide consumers outside ordinary imports. | Stories, Strapi routes, bundles, generated declarations, templates, and codemods are evidence lanes, not positive deadness proof. |
| `tests/test-generated-blind-zone-relevance.mjs` | blind-zone blocker | Generated artifact blind zones block only relevant SAFE_FIX promotion. | Unrelated generated misses must not become repo-global SAFE_FIX blockers. |
| `tests/test-generated-consumer-blind-zones.mjs` | generated consumer blocker | Missing/excluded generated consumer surfaces are reported as possible hidden consumers. | Generated consumers block absence claims without being counted as observed source consumers. |
| `tests/test-resolver-blind-zone-relevance.mjs` | resolver blocker | Resolver blind-zone relevance scopes unresolved imports to affected candidates. | Unresolved imports outside the candidate package must not block unrelated SAFE_FIX promotion. |
| `tests/test-cjs-classification.mjs` | consumer extraction | CommonJS consumers participate in symbol graph and dead-export classification. | CJS support should add real consumers without blanket-protecting opaque surfaces. |
| `tests/test-cjs-integration.mjs` | CJS opacity regression | CJS export surface, alias destructuring, and dynamic require opacity integrate with deadness. | Dynamic or broad CJS opacity must degrade claims rather than fake precise consumer evidence. |
| `tests/test-extract-cjs-consumer.mjs` | consumer extraction unit | Direct CJS require consumers are extracted for exact, side-effect-only, and broad escape forms. | Broad escapes should stay conservative and not pretend named consumers are known. |
| `tests/test-mdx-consumers.mjs` | docs-driven consumer evidence | MDX imports can contribute symbol fan-in without file-level overprotection. | Docs-driven component consumers should protect imported symbols only, not all siblings. |
| `tests/test-p6-member-precision.mjs` | member precision calibration | Namespace and dynamic import member precision protect only directly used exports when possible. | Degraded aliases remain conservative instead of fabricating exact member evidence. |
| `tests/test-p6-safe-fix-calibration.mjs` | calibration corpus | SAFE_FIX calibration uses runtime/staleness convergence on a real mini git repo. | Static confidence alone cannot promote SAFE_FIX without proof objects and calibration evidence. |
| `tests/test-definition-id-canonical.mjs` | identity contract | Canonical definition IDs align symbols, action safety, and call-graph alias fan-in. | Identity drift must fail before consumers or blockers attach to the wrong export. |

## Reform Direction

Deadness tests should state which graph lens is being asserted:

- export-level consumer evidence
- file-level entry reachability
- runtime SCC review evidence
- public or framework surface blockers
- SAFE_FIX action proof

Do not use an unreachable-file signal as automated export-removal proof without
a separate ranking design.

## Reform Targets

- Split deadness tests by graph lens: export consumer, file reachability,
  runtime SCC, public surface, generated surface, framework surface, and action
  proof.
- Keep every review-evidence fixture paired with a "not SAFE_FIX proof"
  assertion.
- Compare CJS, MDX, namespace, and dynamic-member fixtures for shared consumer
  extraction helpers before moving files.
- Keep blocker tests scoped: public, resolver, generated, and framework
  blockers protect different absence contracts.
- Do not use unreachable-file evidence as automated export-removal proof without
  a separate ranking design.

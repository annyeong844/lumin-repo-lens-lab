# Vitest Framework/Resource Surfaces Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-framework-resource-surfaces.mjs`
> - `tests/test-build-framework-resource-surfaces.mjs`
> - `tests/test-framework-policy-facts.mjs`
> - `tests/test-framework-policy-matrix.mjs`

---

## Purpose

This review decides whether the framework/resource surface suites can move
together as one Lane D Vitest mirror batch. It does not add the Vitest suites.

The batch is acceptable because all four suites protect the same review-evidence
boundary around framework conventions and resource-like files:

- framework dispatch entries and resource surfaces must be visible in artifacts;
- path-shaped framework files without package evidence stay review-visible, not
  grounded;
- generated declarations, bundles, scaffold templates, and codemod resources
  must limit absence claims without becoming deadness proof;
- package-scoped framework policies must not leak across nested package
  boundaries;
- route-specific framework facts, such as Hono handler references, must be
  grounded in route registration evidence rather than path shape alone.

This is analyzer-adjacent. The future mirror must remain a behavior-preserving
runner mirror. It must not change framework muting policy, resolver behavior,
deadness ranking, or action-safety promotion.

## Reviewed Evidence

| Suite                                              | Preserved Node Command                                  | Proposed Focused Vitest Command                         | Surface Under Review                             |
| -------------------------------------------------- | ------------------------------------------------------- | ------------------------------------------------------- | ------------------------------------------------ |
| `tests/test-framework-resource-surfaces.mjs`       | `node tests/test-framework-resource-surfaces.mjs`       | `npm run test:vitest:framework-resource-surfaces`       | pure framework/resource surface classification   |
| `tests/test-build-framework-resource-surfaces.mjs` | `node tests/test-build-framework-resource-surfaces.mjs` | `npm run test:vitest:build-framework-resource-surfaces` | producer artifact, manifest, and summary routing |
| `tests/test-framework-policy-facts.mjs`            | `node tests/test-framework-policy-facts.mjs`            | `npm run test:vitest:framework-policy-facts`            | Hono route registration facts                    |
| `tests/test-framework-policy-matrix.mjs`           | `node tests/test-framework-policy-matrix.mjs`           | `npm run test:vitest:framework-policy-matrix`           | framework policy matrix and package boundaries   |

Current Node evidence checked for this review:

```text
node tests/test-framework-resource-surfaces.mjs       # 4 passed, 0 failed
node tests/test-build-framework-resource-surfaces.mjs # 2 passed, 0 failed
node tests/test-framework-policy-facts.mjs            # 4 passed, 0 failed
node tests/test-framework-policy-matrix.mjs           # 15 passed, 0 failed
```

Goal lane: Lane D, resolver/surface. This review covers framework/resource and
framework policy surfaces only, not all resolver/surface suites.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add all four focused mirrors together because
they share the same framework/resource evidence contract. The mirror must keep
every Node entrypoint runnable and must not turn framework/resource review
evidence into concrete consumer proof, deadness proof, or `SAFE_FIX` evidence.

## Protected Invariants

The future Vitest batch must preserve these framework/resource contracts:

- Storybook and Strapi dispatch surfaces become `grounded` only when package
  evidence is present; path-only shapes remain `path-shaped-review`.
- `framework-resource-surfaces.json` keeps `schemaVersion`,
  `policyVersion`, per-file `surfaceLanes[]`, `summary.byLane`,
  `summary.byCapabilityPack`, and `summary.byConfidence`.
- `manifest.json.frameworkResourceSurfaces` mirrors the raw summary enough for
  readers to find the full artifact.
- Audit summary and review-pack wording point readers to
  `framework-resource-surfaces.json` before treating import absence as
  deadness.
- Generated declaration files, bundled build artifacts, Emscripten bindings,
  scaffold templates, and codemod resources stay review-visible surfaces.
- Resource surface output remains deterministic and sorted by file path.
- Hono route facts are collected from concrete `app.get`, `app.post`,
  `app.use`, and `app.route` registrations with imported or local exported
  handlers.
- Dynamic Hono handler expressions are skipped rather than guessed.
- Hono route fact collection is gated by package-scoped Hono dependency
  evidence.
- Framework policy decisions are package-scoped: root framework dependencies
  do not activate nested packages with their own `package.json`.
- Non-workspace nested Next.js packages with their own Next dependency protect
  app router files; nested packages without evidence stay visible.
- `package.json` workspaces and `pnpm-workspace.yaml` patterns are merged for
  workspace discovery.
- Next, Nuxt, SvelteKit, Astro, React Router, Hono, NestJS, and Cloudflare
  Worker policy cases keep their current mute, review-hint, rejected-signal, or
  visible behavior.
- Framework counters separately track muted findings, review hints, rejected
  signals, and path-shaped candidates kept visible.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- A Storybook or Strapi-shaped path without dependency evidence must not become
  `grounded`.
- A bundled file, generated declaration, scaffold template, or codemod resource
  must not become automated deadness proof.
- Root Next.js evidence must not mute files in a nested package that lacks its
  own Next evidence.
- A non-workspace nested Next package must still be discovered through local
  `package.json` evidence.
- A repo with both `package.json` workspaces and `pnpm-workspace.yaml` must not
  ignore either source.
- Arbitrary nested Next middleware paths must stay visible unless a supported
  sentinel rule covers them.
- Nuxt rejected signals, such as `@nuxt/opencollective`, must not activate
  unrelated muting.
- NestJS dependencies and path shapes must not framework-mute arbitrary helpers.
- Hono path shape alone must not mute a handler; route registration facts are
  required.
- Cloudflare Worker default export protection must not leak to helper exports
  or unrelated nested packages.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Every preserved Node command listed above remains runnable.
- Temporary package fixtures may be shared for setup, file writes, JSON reads,
  and cleanup only.
- Shared helpers must not decide framework activation, capability-pack
  assignment, confidence level, Hono route fact meaning, package ownership,
  muting, review-hint promotion, rejected-signal accounting, or deadness
  ranking.
- The mirror must not absorb broader resolver expansion suites, public/deep
  import suites, generated blind-zone suites, deadness/ranking/action-safety
  suites, performance/incremental suites, pre-write cue policy suites, or full
  audit-repo orchestration suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/framework-resource-surfaces.test.mjs`,
2. `tests/build-framework-resource-surfaces.test.mjs`,
3. `tests/framework-policy-facts.test.mjs`,
4. `tests/framework-policy-matrix.test.mjs`,
5. focused `npm run test:vitest:*` commands for each suite,
6. candidate-board updates moving the four suites from `REVIEWED` to `DONE`.

The implementation PR should keep the current Node assertion groups represented
as named Vitest `it(...)` blocks. It may share setup-only temporary repo and
artifact-read helpers inside test files, but no shared helper should decide
framework/resource classification, package-scope ownership, Hono route
registration meaning, framework muting, review-hint semantics, or deadness
meaning.

Run the preserved Node commands and focused Vitest commands when changing this
batch. Also run `npm run test:vitest`, doc-script checks, and formatting checks
so the reviewed runner discovery boundary and wiki references stay current.

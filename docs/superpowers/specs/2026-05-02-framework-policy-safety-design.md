# Framework Policy Safety Design

Date: 2026-05-02
Status: draft for user review

## Purpose

Framework policy should make Lumin Repo Lens safer, not quieter.

Dead-export analysis is useful only when review candidates stay visible until
the engine has strong evidence that a framework runtime consumes them by
convention. A false positive in `REVIEW_FIX` costs reviewer time. A false mute
is worse: it can hide a real cleanup candidate from the normal chat surface.

This design defines the first framework safety pass for Next.js, Remix, Hono,
SvelteKit, Astro, Nuxt/Nitro, and NestJS. The initial goal is not maximum
coverage. The initial goal is to prevent broad framework policies from
activating on weak or misleading evidence.

## Guiding Rule

Mute only with framework evidence strong enough to explain who consumes the
file or export.

If framework evidence is weak, ambiguous, or contradicted, the finding must
stay review-visible. The engine may attach a framework hint, but it must not
hide the finding in `MUTED`.

This rule matters most for directory names such as `app/`, `pages/`,
`routes/`, `middleware/`, `plugins/`, and `server/`. Those names are common
outside framework routing systems.

Framework evidence is scoped to the nearest package root. A dependency or
config in the repository root must not activate framework muting for a nested
workspace package that has its own `package.json` unless that package
explicitly declares or inherits the framework evidence through a supported
workspace/config mechanism.

## Current State

Framework policy currently lives mainly in `_lib/classify-policies.mjs`.

Existing protections:

- FP-22 mutes config files consumed by tool naming conventions.
- FP-27 treats Next.js-style `pages/`, selected `app/` basenames, and
  SvelteKit `+page`/`+layout`/`+server` files as framework sentinels.
- FP-30 detects Nuxt/Nitro from package metadata and mutes Nuxt/Nitro
  filesystem-routed files.
- FP-46 and FP-47 cover VitePress and HTML module-script entrypoints.

Recent beta findings:

- `0.9.0-beta.3` fixed baseUrl-relative imports such as `app/_types`.
- `0.9.0-beta.4` maps workspace package `dist/` outputs back to
  package-root source files and prevents `@nuxt/opencollective` from enabling
  Nuxt/Nitro muting in NestJS-style projects.

Remaining design issue:

- Framework policies mix path shape and framework detection in a few places.
  Path shape alone is not sufficient evidence for broad muting.

## Scope

### In Scope

- Define a framework policy matrix for Next.js, Remix, Hono, SvelteKit, Astro,
  Nuxt/Nitro, and NestJS.
- Split framework detection from path/export policy decisions.
- Require evidence bundles before a policy can emit `MUTED`.
- Record rejected or weak framework signals in artifacts so users can see why
  a policy did not activate.
- Add fixture tests for both positive framework cases and negative
  non-framework collisions.
- Add a reproducible lab process for pinned real repositories, but keep lab
  results outside the deployable skill package.

### Out of Scope

- Full semantic framework execution.
- Perfect dependency injection analysis for NestJS.
- Full Hono route graph interpretation across arbitrary factory patterns.
- A guarantee that all framework magic exports are covered.
- Turning every framework hint into a mute.
- Replacing unresolved-import confidence gates. Resolver work remains its own
  track, though framework labs should record resolver confidence.

## Definitions

`framework evidence`: A concrete signal that a repository or workspace package
is actually using a framework runtime. Examples: runtime dependency,
framework config file, framework-specific route file convention, or framework
entrypoint.

`protected convention`: A file path, export name, or module shape that a
framework consumes without ordinary static imports.

`mute`: Move a finding to `fix-plan.json.muted` with a policy reason. This
hides it from ordinary cleanup recommendations.

`review hint`: Attach framework-related evidence while keeping the finding in
`REVIEW_FIX`, `SAFE_FIX`, or `DEGRADED`. This informs the reviewer without
hiding the item.

`rejected signal`: A signal that superficially looks framework-related but is
not enough to activate a framework policy. Example: `@nuxt/opencollective`.

`package scope`: The nearest package root that owns a candidate file. The
policy matrix evaluates activation evidence inside that scope first. Evidence
from another workspace package is not borrowed for broad muting.

## Policy Actions

Every framework policy decision should return one of three actions:

```js
{
  action: 'mute' | 'review-hint' | 'none',
  reason: string,
  evidence: Array<object>,
  rejectedSignals?: Array<object>
}
```

`mute` requires a framework evidence bundle and a matching protected
convention.

`review-hint` is for ambiguous cases. The finding stays visible.

`none` means the framework policy has nothing to say.

The default on parser errors, missing package metadata, malformed manifests,
conflicting framework signals, or unsupported conventions is `review-hint` or
`none`, never broad `mute`.

Phase 1 records `review-hint` actions as aggregate counters only. It does not
add per-finding review-hint payloads to visible findings. That keeps the first
patch focused on safer muting and avoids a broad artifact schema change.

A `mute` decision must include both:

1. package-scoped framework activation evidence; and
2. a specific protected convention match for the candidate file/export.

Path shape may contribute to evidence, but path shape alone may not satisfy
both conditions.

## Framework Matrix

### Next.js

Activation evidence:

- `next` dependency or devDependency.
- `next.config.*`.
- A package-local `app/` or `pages/` tree plus a Next dependency in the same
  root or workspace package.

Protected conventions:

- `pages/**` and `src/pages/**` route modules.
- `pages/api/**` and `src/pages/api/**` API route modules.
- `app/**/page.*`, `src/app/**/page.*`, and corresponding App Router special
  files: `layout.*`, `route.*`, `loading.*`, `error.*`,
  `not-found.*`, `template.*`, `default.*`, `global-error.*`.
- Root or `src`-level `middleware.*` and `proxy.*` only when located alongside
  `app/`, `src/app/`, `pages/`, or `src/pages/`.
- `instrumentation.*` and `src/instrumentation.*` when Next evidence is active.
- `instrumentation-client.*` and `src/instrumentation-client.*` as top-level
  client instrumentation files when Next evidence is active.
- Next top-level file export protections:
  - `proxy.*` / `middleware.*`: default export, `proxy`, `middleware`, and
    `config`.
  - `instrumentation.*`: `register` and `onRequestError`.
  - `instrumentation-client.*`: file-level protection; `onRouterTransitionStart`
    is the only phase 1 named export candidate, initially review-hint until
    fixtures prove safe muting.

Rejected evidence:

- A generic `app/` directory without Next evidence.
- A generic `pages/` directory in a non-Next package.
- Arbitrary `app/**/middleware.*` files. Next middleware/proxy is a top-level
  or `src`-level convention, not a nested route-file convention.

Initial fixtures:

- Positive: `next` dependency + `app/dashboard/page.tsx` default export muted.
- Negative: non-Next service with `app/page.ts` stays review-visible.
- Positive: `src/app/dashboard/page.tsx` and `src/pages/index.tsx` under a
  Next package.
- Positive: root or `src`-level `middleware.ts` / `proxy.ts` alongside a Next
  router tree.
- Positive: `instrumentation.ts` protects `register` and `onRequestError`.
- Review-hint: `instrumentation-client.ts` with `onRouterTransitionStart`.
- Negative: `app/foo/middleware.ts` stays review-visible unless a later
  fixture proves a supported framework convention.

### Remix / React Router Framework Mode

Activation evidence:

- `@remix-run/node`, `@remix-run/react`, or `@remix-run/dev`.
- React Router framework package/config evidence when route modules are used
  through framework mode.
- Remix config file when present.
- `app/routes/**` under a package with Remix dependencies.

Protected conventions:

- Legacy Remix filesystem route modules under `app/routes/**`.
- React Router framework route modules referenced by a resolvable `routes.ts`
  route config.
- Initial protected exports in route modules: default, `loader`, `action`,
  `meta`, `links`, `headers`, and `ErrorBoundary`.
- Review-hint only for `clientLoader`, `clientAction`, `HydrateFallback`,
  `handle`, and `shouldRevalidate` until fixtures prove safe muting.
- `app/root.*`, `app/entry.client.*`, `app/entry.server.*`.

React Router framework exports outside the initial protected set remain
review-visible until route-config fixtures and version-specific docs prove that
muting is safe.

Rejected evidence:

- A generic `routes/` folder without Remix evidence.
- A `routes.ts` file whose referenced route modules cannot be resolved.

Initial fixtures:

- Positive: Remix route module exports are muted with route evidence.
- Negative: Express-style `routes/user.ts` stays review-visible.
- Positive: React Router framework `routes.ts` references a route module and
  protects its initial export set.

### Hono

Activation evidence:

- `hono` dependency.
- Static route registration syntax such as `app.get(...)`, `app.post(...)`,
  `app.use(...)`, or `router.get(...)` in scanned source.

Matrix input contract:

```js
{
  packageEvidenceByRoot,
  frameworkFacts: {
    honoRouteRegistrations: [
      {
        file: "src/server.ts",
        callee: "app.get",
        route: "/x",
        handlerRefs: [
          { file: "src/middleware.ts", exportName: "auth" },
          { file: "src/handlers.ts", exportName: "handler" }
        ]
      }
    ]
  }
}
```

The policy matrix does not parse source to discover Hono handlers. It consumes
route-registration facts produced by a separate scanner. Without those facts,
Hono path-shaped files stay review-visible.

Protected conventions:

- Handler identifiers directly passed to Hono route registration.
- Route modules only when imported by a Hono entrypoint or referenced through
  a route registry.

Rejected evidence:

- `routes/**` by path alone.
- `hono` dependency alone muting every route-looking export.

Initial fixtures:

- Positive: `app.get('/x', handler)` protects `handler`.
- Negative: unused export in `routes/helpers.ts` remains review-visible.

### SvelteKit

Activation evidence:

- `@sveltejs/kit` dependency.
- `svelte.config.*` with Kit usage.
- `src/routes/**/+page.svelte` or related Kit route files in a package with
  SvelteKit evidence.

Protected conventions:

- `src/routes/**/+page.svelte`.
- `+page.*`, `+page.server.*`, `+layout.*`, `+layout.server.*`,
  `+server.*`, `+error.svelte`.
- `src/hooks.server.*`, `src/hooks.client.*`.
- Protected export names in SvelteKit route modules:
  - `load`.
  - `actions`.
  - HTTP methods in `+server.*`: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`,
    `OPTIONS`, `HEAD`.
  - Page/layout options: `prerender`, `ssr`, `csr`, `trailingSlash`, and
    `config` in matching `+page.*`, `+page.server.*`, `+layout.*`, and
    `+layout.server.*` modules.
  - `entries` only in dynamic `+page.*`, `+page.server.*`, or `+server.*`
    route modules where SvelteKit supports prerender entry discovery.

Rejected evidence:

- Any `+name.ts` file outside a SvelteKit route tree.
- A `routes/` directory without SvelteKit evidence.

Initial fixtures:

- Positive: SvelteKit route module exports are muted.
- Negative: ordinary `src/routes/+helper.ts` in non-SvelteKit code remains
  review-visible.
- Positive: route export names `load`, `actions`, `GET`, `prerender`,
  `entries`, `ssr`, `csr`, and `config` are protected only inside matching
  SvelteKit route modules.

### Astro

Activation evidence:

- `astro` dependency.
- `astro.config.*`.
- `src/pages/**` containing `.astro` files in an Astro package.

Protected conventions:

- `src/pages/**/*.astro`, `src/pages/**/*.md`, `src/pages/**/*.mdx`, and
  `src/pages/**/*.html` as page files when the scanner surfaces findings for
  them.
- `src/pages/**/*.(ts|js)` endpoint files.
- Endpoint exports: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `OPTIONS`,
  `HEAD`, and `ALL`.
- `getStaticPaths` in dynamic page or endpoint modules.
- `astro.config.*` default export is handled by the config-file policy or by
  an Astro-specific config match. Do not treat arbitrary default exports as
  Astro config defaults.

If `.astro` files are not parsed by the dead-export scanner, phase 1 protects
only emitted JS/TS endpoint findings under `src/pages/**` and named exports
such as HTTP methods and `getStaticPaths`.

Rejected evidence:

- Generic `pages/` directory without Astro or Next evidence.

Initial fixtures:

- Positive: `src/pages/blog/[slug].astro` and `getStaticPaths` are protected.
- Negative: non-Astro `pages/report.ts` remains review-visible.
- Positive: `src/pages/api/[id].ts` protects `GET` and `getStaticPaths`.

### Nuxt / Nitro

Activation evidence:

- `nuxt`, `nitropack`, `nitro`, `@nuxt/schema`, or other explicit Nuxt/Nitro
  runtime package.
- `nuxt.config.*`.
- Package name that is itself a Nuxt/Nitro runtime package.

Protected conventions:

- `server/api/**`, `server/middleware/**`, `server/plugins/**`,
  `server/routes/**`.
- `runtime/handlers/**`, `runtime/middleware/**`, `runtime/plugins/**`,
  `runtime/utils/**`, `runtime/server-assets/**`.
- `app/plugins/**`, `app/middleware/**`, `app/entry.*`, `app/entry-spa.*`.
- Root or `app/` `plugins/*.ts` and `middleware/*.ts` only when Nuxt evidence
  is active.
- Root `composables/*.ts` and `app/composables/*.ts` only at top level by
  default.
- Nested composables require additional evidence: re-export from `index`,
  Nuxt scan/import config, or observed generated import metadata.

Rejected evidence:

- `h3` dependency alone.
- `@nuxt/opencollective`.
- `@nuxt/*` packages explicitly listed as non-runtime helpers.
- Directory names `middleware/` or `plugins/` in a NestJS or generic Node
  package.

Initial fixtures:

- Positive: Nuxt package with `nuxt.config.ts` mutes `server/api/ping.ts`.
- Negative: NestJS package with `@nuxt/opencollective` keeps
  `middleware/utils.ts` review-visible.
- Negative: nested Nuxt composable stays review-visible without re-export,
  scan config, or generated import evidence.

### NestJS

Activation evidence:

- `@nestjs/core` or `@nestjs/common`.
- Decorator imports such as `Controller`, `Injectable`, `Module`, `Get`,
  `Post`, `MiddlewareConsumer`.

Protected conventions:

- NestJS should not use broad filesystem muting in phase 1.
- Decorated classes or methods may receive review hints when static import
  evidence is incomplete, but they should not be hidden solely because a
  decorator exists.

Rejected evidence:

- `middleware/` or `plugins/` path names as Nuxt/Nitro evidence.
- `@nuxt/opencollective` as Nuxt evidence.

Initial fixtures:

- Positive: provider/controller classes imported through a Nest module remain
  live through ordinary static graph evidence.
- Negative: file-internal middleware helpers tested only in specs remain
  review-visible, not muted as Nuxt/Nitro.

## Architecture

### New Pure Policy Layer

Add a pure framework policy module:

```text
_lib/framework-policy-matrix.mjs
```

Responsibilities:

- Detect framework evidence per root or workspace package.
- Classify a file/export against known protected conventions.
- Return `mute`, `review-hint`, or `none`.
- Emit evidence and rejected signals in a stable shape.

It should not read or write artifacts. It should accept already-loaded package
metadata, workspace dirs, and repo-relative file paths from callers.

### Integration Point

`_lib/classify-policies.mjs` remains the public policy entry used by
`classify-dead-exports.mjs`. It delegates framework-specific decisions to
`framework-policy-matrix.mjs`.

This avoids expanding `classify-dead-exports.mjs` and keeps policy change
surface local.

### Artifact Shape

Existing muted findings already carry policy evidence. Extend policy evidence
with rejected or weak framework signals when relevant:

```json
{
  "reason": "frameworkPolicyReviewHint",
  "framework": "nuxt",
  "action": "review-hint",
  "evidence": [
    { "source": "path", "value": "middleware/utils.ts" }
  ],
  "rejectedSignals": [
    {
      "source": "package",
      "value": "@nuxt/opencollective",
      "reason": "non-runtime helper"
    }
  ]
}
```

For phase 1, do not attach this payload to every visible finding. Instead,
expose aggregate rejected-signal and review-hint counters in
`dead-classify.json.summary`. Per-finding hints are a later phase after the
matrix behavior is stable.

### Summary Counters

Add or preserve counters that can answer:

- How many framework mutes fired by framework and policy id?
- How many weak or rejected framework signals were seen?
- How many path-shaped framework candidates stayed review-visible?
- Whether each counter is finding-scoped, package-scoped, or signal-occurrence
  scoped.

Suggested shape:

```json
{
  "frameworkPolicy": {
    "mutedFindings": { "next": 4, "nuxt": 2 },
    "reviewHintFindings": { "nestjs": 3 },
    "rejectedSignalOccurrences": {
      "@nuxt/opencollective": 1,
      "h3-alone": 1
    },
    "pathShapedCandidatesKeptVisible": {
      "middleware": 12,
      "routes": 8,
      "app": 3
    },
    "rejectedSignals": {
      "@nuxt/opencollective": {
        "packages": 1,
        "findingsAffected": 12
      }
    }
  }
}
```

Counting units:

- `mutedFindings`: finding count.
- `reviewHintFindings`: finding count.
- `pathShapedCandidatesKeptVisible`: finding count grouped by path family.
- `rejectedSignalOccurrences`: package-scope signal occurrence count.
- `rejectedSignals.*.packages`: package count.
- `rejectedSignals.*.findingsAffected`: candidate finding count that would
  have been eligible for a broad policy if the signal had been accepted.

## Data Flow

1. `audit-repo.mjs` runs the usual producer chain.
2. `classify-dead-exports.mjs` loads symbols, repo mode, package metadata, and
   current policy helpers.
3. Framework detection computes package-scoped evidence once.
4. Each candidate file/export asks the matrix whether a framework policy
   applies.
5. Only `mute` actions enter `MUTED`.
6. Ambiguous or weak matches stay review-visible. In phase 1, their evidence
   is summarized through aggregate counters only; per-finding review-hint
   payloads are reserved for a later schema phase.
7. `fix-plan.json` and `checklist-facts.json` continue to treat `MUTED` as
   intentionally hidden policy exclusions.

## Error Handling

- Missing `package.json`: no framework activation.
- Malformed `package.json`: no framework activation; existing strict/soft
  parse behavior remains unchanged.
- Missing workspace package: no framework activation for that package.
- Conflicting evidence: no broad mute; emit review hint if possible.
- Conflicting evidence is evaluated within a package scope. Evidence from
  separate workspace packages is not considered conflicting unless a candidate
  cannot be assigned to a single nearest package root.
- Unknown framework package under a familiar namespace: no activation until
  explicitly classified as runtime evidence.
- Resolver blind zone: keep confidence warnings separate from framework
  policy. Do not let framework policy hide unresolved-import risk.

## Testing Strategy

### Unit Tests

Add a focused test file for the pure policy module:

```text
tests/test-framework-policy-matrix.mjs
```

Test groups:

- Framework evidence detection for each framework.
- Rejected signal handling.
- Positive protected convention matches.
- Negative path-name collision cases.
- Deterministic evidence payloads.

### Integration Fixtures

Extend `tests/test-corpus.mjs` or add a separate integration suite when the
fixture size grows too much.

Required fixtures:

- Root framework dependency does not activate a nested non-framework workspace
  package with its own `package.json`.
- Next.js positive and non-Next `app/` negative.
- Next `src/app/**` and `src/pages/**` positive fixtures.
- Next root or `src`-level `middleware.*` / `proxy.*` positive fixture, and
  `app/foo/middleware.ts` negative fixture unless explicitly supported later.
- Remix positive and Express-style `routes/` negative.
- React Router framework route module referenced by resolvable `routes.ts`.
- Hono `app.get(..., handler)` positive and `routes/**` path-only negative.
- SvelteKit positive and non-Svelte `+helper.ts` negative.
- SvelteKit route export names: `load`, `actions`, HTTP methods,
  `prerender`, `entries`, `ssr`, `csr`, and `config`.
- Astro positive and generic `pages/` negative.
- Astro endpoint exports: HTTP methods, `ALL`, and `getStaticPaths`.
- Nuxt/Nitro positive and `@nuxt/opencollective`/NestJS negative.
- Nuxt nested composable remains review-visible unless re-export/config
  evidence exists.
- NestJS decorator/module fixture where normal imports keep live classes live
  and ordinary helpers remain review-visible.

### Real Repository Lab

Create a lab note under `docs/lab/` for pinned real-world runs. The lab should
not be part of the deployable skill context.

Each lab entry records:

- Repository and commit SHA.
- Command line.
- File count and scan profile.
- Parse errors.
- `unresolvedInternalRatio`.
- Framework policies activated.
- Muted count by policy.
- Review-visible candidate sample.
- Spot-check results and corrections.

Candidate repos should be pinned and small enough to reproduce locally before
being used as regression evidence. Representative coverage is more important
than popularity.

## Acceptance Criteria

- A non-framework project with `app/`, `pages/`, `routes/`, `middleware/`, or
  `plugins/` directories does not receive broad framework muting from path
  names alone.
- `@nuxt/opencollective` and `h3` alone do not activate Nuxt/Nitro muting.
- Known positive framework fixtures still mute framework-consumed sentinels.
- NestJS middleware/helper exports are not hidden by framework policy. Ordinary
  static graph evidence may classify them as live, safe-fix, review-fix, or
  degraded according to the normal dead-export pipeline.
- Framework policy evidence is visible in artifacts when a mute occurs.
- Weak or rejected framework signals are visible in aggregate counters or
  summary fields.
- Existing beta4 resolver and policy tests continue to pass.
- Public docs describe the conservative policy in user-facing language, not
  internal FP ids.

## Implementation Sequence

1. Add `tests/test-framework-policy-matrix.mjs` with pure RED cases for
   framework evidence and rejected signals.
2. Add `_lib/framework-policy-matrix.mjs` as a pure module.
3. Route FP-27 and FP-30 decisions through the matrix without changing public
   artifact shapes beyond counters.
4. Add negative collision fixtures for non-framework `app/`, `pages/`,
   `routes/`, `middleware/`, and `plugins/`.
5. Add positive fixtures for each framework where the protected convention is
   clear.
6. Add aggregate review-hint and rejected-signal counters.
7. Build skill/plugin surfaces and update generated copies.
8. Run the full local CI.
9. Run a pinned lab pass on at least one real repo before broad claims.

## Phase 1 Decisions

- False-mute prevention comes before coverage expansion.
- Aggregate counters come before per-finding review-hint payloads.
- The first matrix covers Next.js, Remix, Hono, SvelteKit, Astro,
  Nuxt/Nitro, and NestJS.
- VitePress remains on its existing FP-46 policy path for phase 1. It can move
  into the matrix later if the matrix proves stable.

## Release Strategy

Ship this as a beta patch or beta minor only after fixtures prove that existing
positive protections still work.

The release note should avoid saying "supports all framework magic." Safer
wording:

> Framework policy safety pass: framework-routed files are muted only when the
> engine has matching framework evidence; weak namespace/path signals remain
> review-visible.

## Risks

### Risk: Fewer Findings Are Muted

Some projects may see more review-visible findings because weak framework
signals no longer hide them. This is acceptable. Review-visible noise is safer
than false mute.

### Risk: Framework Evidence Is Incomplete

A real framework project may lack a dependency or config signal in the scanned
package. The safe behavior is review-visible. Lab runs should identify common
evidence gaps before expanding muting.

### Risk: Policy Matrix Becomes a Grab Bag

The matrix must stay declarative. Framework-specific quirks should be added
only with a fixture and a lab note or real report. No untested namespace or
path-prefix guesses.

### Risk: NestJS Is Overprotected

NestJS decorators are runtime metadata, but decorator presence alone does not
prove a class or helper is externally consumed. Phase 1 therefore uses NestJS
mainly as a negative guard against Nuxt/Nitro path collisions, not as a broad
mute policy.

## Non-Claims

- This design does not claim perfect dead-export detection for production
  monorepos.
- This design does not claim full support for all framework conventions.
- This design does not make `MUTED` a correctness verdict.
- This design does not eliminate the need to read `manifest.confidence` and
  unresolved-import diagnostics before making absence claims.

## Review Focus

When reviewing this design, check two decisions first:

1. Phase 1 intentionally prefers review-visible noise over false mute.
2. Phase 1 intentionally keeps artifact shape small by using aggregate
   counters instead of per-finding review hints.

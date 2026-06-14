# Framework And Resource Surface Policy

> **Role:** maintainer-facing design spec for files that exist in the checkout
> but are not ordinary application source for absence claims.
> **Status:** MVP implemented for classifier-only artifact and summary
> visibility; ranking integration deferred.
> **Last updated:** 2026-05-09

---

## 1. Problem

Some repositories contain files that look parseable but are consumed by build
tools, generators, framework loaders, test harnesses, or scaffold pipelines
rather than by ordinary static imports. If Lumin Repo Lens treats these files as
normal application modules, it can create false absence claims:

```text
not imported in the constructed graph
=> looks unused
=> proposed as dead or unreachable
```

The correct claim is often weaker:

```text
this file is part of a framework/resource/generated surface;
ordinary import graph absence is not enough to prove deadness.
```

Recent stress repos exposed the pattern clearly:

- Storybook-style repos can contain hundreds of `*.stories.*` files. Story files
  are indexed by Storybook tooling and test runners, not necessarily imported by
  application source.
- Strapi-style repos use filesystem-routed `src/api/*/{controllers,routes,services}`
  modules and generated declarations such as `types/generated/*.d.ts`.
- Scaffold and codemod repos keep source-like files under `templates/`,
  `__testfixtures__/`, `resources/codemods/`, or `*.hbs` resources.
- Build output may be checked in or generated during tests, such as
  `vendor.js`, `*.bundle.js`, minified files, or Emscripten/WASM bindings with
  generated headers like `@ts-nocheck`.

This is not one resolver bug. It is a cross-cutting surface policy gap.

## 2. Goals

- Classify framework, resource, scaffold, and bundled/generated surfaces before
  they can become false absence evidence.
- Keep the policy generic. A real Storybook or Strapi repo may motivate a
  fixture, but the rule must not hard-code one repository layout.
- Preserve `Tier != claim`: surface labels may mute, review, or confidence-limit
  findings, but they must not act as positive deadness evidence.
- Start with diagnostic inventory and review-pack visibility before ranking
  behavior changes.
- Keep expensive framework execution, build execution, and code generation out
  of default scans.

## 3. Non-goals

- Do not run Storybook, Strapi, Emscripten, bundlers, codemods, or scaffold
  generators by default.
- Do not parse generated bundles for semantic facts such as clone groups,
  dead-export ownership, or call graph edges unless a later spec explicitly
  defines generated-source provenance.
- Do not treat a path token such as `template`, `controller`, `story`, or
  `generated` as sufficient proof by itself.
- Do not hide all files under broad directories like `templates/` without
  exposing diagnostics. Quiet suppression without evidence would make reviews
  harder.
- Do not make framework-specific rules direct `SAFE_FIX` evidence.

## 4. Surface Taxonomy

The first implementation should classify files into surface lanes. A file may
have more than one lane, but each lane has its own claim and evidence.

| Surface lane | Examples | Default claim |
|---|---|---|
| `framework-dispatch-entry` | Storybook stories, Strapi controllers/routes/services, framework route modules | The framework may consume this file through convention or a manifest. Import absence alone is not a deadness proof. |
| `scaffold-template-resource` | `templates/**`, `.hbs`, starter app files copied by a generator | This file is generator input or scaffold material, not ordinary app runtime code. |
| `codemod-resource` | `resources/codemods/**`, codemod fixtures, `__testfixtures__/**` | This file may be consumed as transform input/output or test fixture data. |
| `bundled-build-artifact` | `vendor.js`, `*.bundle.js`, `*.min.js`, generated WASM JS bindings | This file is likely build output; source ownership and clone/deadness claims are confidence-limited. |
| `generated-declaration-surface` | `types/generated/*.d.ts`, generated API declaration trees | This is generated type surface. It may affect contract proof, but should not become ordinary source proof. |

These lanes are separate from `generated-artifact-support.md`, which primarily
handles missing generated targets. This spec handles files that exist but should
not be interpreted as ordinary source without provenance.

## 5. Evidence Strength

Surface classification should use a small, versioned policy:

```text
framework-resource-surface-policy-v1
```

Evidence levels:

| Level | Meaning | Example |
|---|---|---|
| `grounded` | Package/config evidence and path convention agree. | `@storybook/*` dependency plus `*.stories.tsx`; `@strapi/strapi` plus `src/api/foo/controllers/foo.ts`. |
| `path-shaped-review` | Path looks like a surface, but package/config activation is missing or weak. | `templates/foo.ts` in a package with no generator metadata. |
| `resource-only` | File is clearly a fixture/resource by extension or location, but no framework ownership is inferred. | `__testfixtures__/input.ts`, `*.hbs`. |
| `generated-output-review` | File looks like build/generated output but the producer is unknown. | `vendor.js`, `app.bundle.js`, generated `@ts-nocheck` header. |

Rules:

- Framework-specific muting requires activation evidence such as package
  dependencies, config files, or framework manifest files.
- Path-only evidence may produce a review hint or confidence limitation, but it
  must not silently mute findings.
- Generated/bundled artifact labels are not source-resolution success. They
  should reduce overclaim, not create graph edges.
- Surface policy decisions must carry structured evidence, including matched
  dependency/config/path rules and policy version.

## 6. Artifact Shape

The first behavior-changing slice should produce a deterministic artifact before
ranking consumes it:

```json
{
  "schemaVersion": "framework-resource-surfaces.v1",
  "policyVersion": "framework-resource-surface-policy-v1",
  "files": [
    {
      "file": "packages/app/src/Button.stories.tsx",
      "surfaceLanes": [
        {
          "lane": "framework-dispatch-entry",
          "capabilityPack": "framework.storybook",
          "confidence": "grounded",
          "framework": "storybook",
          "reason": "storybook-story-file",
          "defaultAction": "review-hint",
          "affectsAbsenceClaims": true,
          "evidence": [
            { "kind": "dependency", "field": "devDependencies.@storybook/react" },
            { "kind": "path-convention", "matched": "*.stories.*" }
          ]
        }
      ]
    }
  ],
  "summary": {
    "byLane": { "framework-dispatch-entry": 1 },
    "byCapabilityPack": { "framework.storybook": 1 },
    "byConfidence": { "grounded": 1 }
  }
}
```

Manifest and review surfaces should summarize the artifact, not bury it:

```text
Framework/resource surfaces: 585 story files, 204 filesystem-routed API files,
892 template/codemod resources. Review framework/resource surface diagnostics
before treating import absence as deadness.
```

The full file list belongs in a dedicated artifact. `manifest.json` should carry
counts, top examples, and the artifact path.

Each lane must also carry a stable `capabilityPack` owner. Consumers should read
that field instead of inferring ownership from path tokens, lane names, or
framework-specific reasons. The manifest summary should preserve
`byCapabilityPack` as a top-level pivot for shallow review.

## 7. Ranking Contract

This spec does not directly promote or delete anything.

Ranking integration should follow these rules when implemented:

- `framework-dispatch-entry` with grounded framework evidence may mute or
  review-hint convention exports according to the existing framework policy
  matrix.
- `path-shaped-review` must not mute by itself. It may block `SAFE_FIX` for
  candidate exports in the same file or surface scope until reviewed.
- `bundled-build-artifact` and `generated-declaration-surface` should prevent
  source-action suggestions on those files unless a later generated-source
  contract says otherwise.
- `scaffold-template-resource` and `codemod-resource` should stay visible in
  diagnostics but should not create ordinary app graph edges.
- Surface labels must never be used as positive deadness evidence.

## 8. Implementation Phases

### P0: Spec and tracker

- Capture this policy.
- Add a work-tracker row so the gap is not mistaken for completed generated
  artifact support.
- Do not change analyzer behavior.

### P1: Classifier-only artifact

- Add a pure classifier for file surface lanes.
- Emit `framework-resource-surfaces.json`.
- Add manifest/review-pack summary lines.
- Add fixtures for Storybook story files, Strapi filesystem-routed modules,
  generated `.d.ts`, bundled JS, templates, codemod resources, and Emscripten
  style generated headers.
- No ranking changes.

P1 status: implemented as an evidence-only artifact. The audit pipeline emits
`framework-resource-surfaces.json`, `manifest.json.frameworkResourceSurfaces`,
an audit-summary measured cue line, and a review-pack Lane 3 reminder. These
fields are diagnostics only; they do not alter graph edges, ranking, or
`SAFE_FIX` promotion.

### P2: Safe framework/resource gates

- Wire grounded framework entries into existing framework policy only when
  package/config evidence exists.
- Add confidence-limited blockers for source-action suggestions on generated or
  bundled artifacts.
- Keep path-only cases as review hints.

### P3: Framework-specific grounded support

- Add Storybook conventions as grounded only with Storybook dependency/config
  evidence.
- Add Strapi filesystem-routing conventions as grounded only with Strapi
  dependency/config evidence.
- Add narrow generator/template metadata where package scripts or config files
  prove scaffold ownership.

### P4: Corpus calibration

- Measure on Storybook and Strapi stress repos plus at least one unrelated
  large JS/TS workspace.
- Track false-muted findings separately from false-review hints.
- Do not mark `DONE` until public package verification confirms summary and
  review-pack wording.

## 9. Acceptance Checks

- A Storybook package with `@storybook/*` dependency and `src/Button.stories.tsx`
  produces a grounded `framework-dispatch-entry` lane.
- A `*.stories.tsx` file without Storybook activation remains
  `path-shaped-review`, not muted.
- A Strapi package with `@strapi/strapi` and
  `src/api/article/controllers/article.ts` produces a grounded
  `framework-dispatch-entry` lane.
- A Strapi-shaped path without Strapi activation remains review-visible.
- `vendor.js`, `*.bundle.js`, `*.min.js`, and generated `@ts-nocheck` WASM
  binding headers produce `bundled-build-artifact` diagnostics and do not
  contribute source clone/deadness proof by default.
- `types/generated/*.d.ts` produces `generated-declaration-surface` diagnostics.
- `templates/**`, `*.hbs`, `resources/codemods/**`, and `__testfixtures__/**`
  produce resource lanes without creating app graph edges.
- Manifest summaries expose counts and examples; full details live in
  `framework-resource-surfaces.json`.
- No surface lane creates `SAFE_FIX` by itself.
- Path-only evidence cannot become a repo-global blocker.

## 10. Priority Guidance

This work is higher priority than broad ranking relaxation because it prevents
false absence claims at the evidence boundary. It should run before adding more
framework-specific dead-export exceptions and before weakening `SAFE_FIX`
requirements.

Recommended next implementation slice:

```text
P1 classifier-only artifact + manifest/review-pack summary
```

This slice is useful even before ranking consumes it: agents and reviewers can
see when a repo is dominated by framework dispatch, templates, generated
declarations, or bundled output, and they can avoid treating ordinary import
absence as a cleanup instruction.

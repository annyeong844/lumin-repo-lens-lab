# SFC Support Policy

> **Status:** MVP support boundary.
> **Policy:** SFC support must be capability-scoped. Do not turn the
> repository-wide `sfc-scan-gap` warning off just because one SFC lane is
> implemented.

## Problem

Single-file components are not JavaScript files with a different extension.
Vue, Svelte, and Astro files combine script, template, style, framework magic,
and sometimes external script references in one container. Treating the whole
file as JavaScript creates false edges. Ignoring the file creates false
deadness.

The current correct stance is narrow: prove the lanes we actually understand,
advertise those lanes with explicit supports flags, and keep a scan-gap signal
for everything else.

## Current Contract

P0 and P1 have shipped these contracts:

- triage counts `.vue`, `.svelte`, and `.astro` files and emits one grouped
  `sfc-scan-gap` blind zone;
- `symbols.meta.supports.sfcScriptImportConsumers === true` advertises the
  implemented script-import lane;
- Vue and Svelte inline `<script>` blocks and Astro frontmatter static imports
  are parsed as SFC script consumers;
- declared JSX/TSX script dialects are honored, including Vue
  `<script lang="tsx">`;
- internal SFC script imports feed `resolvedInternalEdges` and symbol fan-in;
- external SFC script imports feed `dependencyImportConsumers` with source
  `sfc-script-import`;
- template text and `<script src>` are ignored by the P1 lane;
- the `sfc-scan-gap` blind zone remains after P1 because template, style,
  external-script, and framework-magic surfaces are not modeled yet.

The runtime verification note for beta.63 records the installed-package check:
[`wt-sfc-beta63-script-import-consumers-verification-2026-05-25.md`](../lab/wt-sfc-beta63-script-import-consumers-verification-2026-05-25.md).

P2 has shipped a second narrow contract:

- literal relative Vue/Svelte `<script src>` references create
  `resolvedInternalEdges[]` with `kind: "sfc-script-src"`;
- the referenced source file becomes reachable, but named exports in that file
  do not receive fan-in from script-source evidence;
- package, URL/data, non-literal, empty, and missing script-source forms do not
  become concrete import edges;
- missing relative script sources are diagnostic-only with reason
  `sfc-script-src-unresolved`;
- the `sfc-scan-gap` blind zone remains after P2.

The runtime verification note for beta.64 records the installed-package check:
[`wt-sfc-beta64-script-src-reachability-verification-2026-05-25.md`](../lab/wt-sfc-beta64-script-src-reachability-verification-2026-05-25.md).
The source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

P3 source implementation has started a third narrow contract:

- literal relative SFC style `url()` and style `@import` references are
  recorded in `symbols.json.sfcStyleAssetReferences[]`;
- resolved style assets carry `status: "resolved"` and a `resolvedFile`;
- missing relative style assets carry `status: "unresolved"` with reason
  `sfc-style-asset-unresolved`;
- style asset evidence does not enter `resolvedInternalEdges[]` and does not
  affect named export fan-in;
- package, URL/data, dynamic, commented, and template-attribute style forms do
  not become concrete source edges.

The beta.65 public-install verification records
`sfc-style-assets-public-verified` and `asset-evidence-not-symbol-fan-in`:
[`wt-sfc-beta65-style-assets-verification-2026-05-26.md`](../lab/wt-sfc-beta65-style-assets-verification-2026-05-26.md).
The source guards are
[`test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

## Capability Lanes

Each SFC lane needs its own capability and evidence contract.

| Lane                                | Status      | Evidence Rule                                                                                                                                                      |
| ----------------------------------- | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `sfc-script-import-consumers`       | Implemented | Static imports in Vue/Svelte inline scripts and Astro frontmatter may produce graph edges and fan-in.                                                              |
| `sfc-template-component-refs`       | MVP         | Explicit script/frontmatter component bindings may produce review-only `symbols.json.sfcTemplateComponentRefs[]` evidence; they do not feed graph edges or fan-in. |
| `sfc-script-src`                    | MVP         | Literal Vue/Svelte `<script src>` may produce a reachability edge with kind `sfc-script-src`, but not named export fan-in.                                         |
| `sfc-style-assets`                  | MVP         | Style `url()` and `@import` references may become non-source asset evidence, not module graph edges or symbol fan-in.                                              |
| `sfc-global-component-registration` | MVP         | Explicit Vue global/app/plugin registration may produce review-only availability evidence, not template consumption or fan-in.                                     |
| `sfc-framework-magic`               | MVP         | Generated manifests, selected framework conventions, and unavailable config-shape signals may produce review-only evidence; they still cannot affect deadness.       |

## P2 Source Shape

Do not jump straight to broad template parsing. That would be a fancy way to
manufacture lies.

The P2 slice chose one narrow lane and proved its semantics before touching
template parsing. `sfc-script-src` is syntactic and easier to bound than
template component resolution. Even there, the first safe claim is file
reachability, not named export fan-in.

The script-source fixture inventory is recorded at
[`wt-sfc-script-src-fixture-inventory-2026-05-25.md`](../lab/wt-sfc-script-src-fixture-inventory-2026-05-25.md).

Implemented P2 source shape:

1. fixtures cover Vue/Svelte literal `<script src="./logic.ts">`;
2. `symbols.json.resolvedInternalEdges[]` records `kind: "sfc-script-src"`;
3. named exports in that referenced script stay eligible for normal deadness
   unless they are consumed elsewhere;
4. non-literal, package, and URL sources do not become concrete edges;
5. unresolved relative sources are diagnostic records, not fake edges;
6. default Markdown and SAFE/action lanes remain unchanged.

Template/component support should wait for corpus data. It needs binding-aware
rules, not tag-name guessing.

## P3 Candidate: Style Assets

The selected lane is style asset evidence, recorded in
[`wt-sfc-style-asset-fixture-inventory-2026-05-25.md`](../lab/wt-sfc-style-asset-fixture-inventory-2026-05-25.md).

Decision: `style-assets-before-template-refs` and
`asset-reachability-not-symbol-fan-in`.

The first safe style contract is non-source asset evidence for literal relative
style `url()` and style `@import` references. The source implementation records
that evidence in `symbols.json.sfcStyleAssetReferences[]`. This evidence can
support future asset hygiene or resource-surface reporting, but it must not
enter JS/TS module reachability, named export fan-in, `SAFE_FIX`, `EXISTS`,
package edits, or dead-export ranking.

Template component references remain future work. They need binding-aware
resolution before they can become review evidence, and they need a much stronger
contract before they affect absence claims.

## P4 Candidate: Template Component Refs

The next selected design lane is template component reference evidence, recorded
in
[`wt-sfc-template-component-ref-fixture-inventory-2026-05-26.md`](../lab/wt-sfc-template-component-ref-fixture-inventory-2026-05-26.md).

Decision: `template-binding-inventory-before-evidence` and
`binding-aware-or-no-claim`.

Template component tags are not import declarations. A tag can become evidence
only when a fixture-proven binding model connects it to a script import, local
registration, or other explicit component binding. The first safe surface, if
implemented, should be review-only and must stay out of JS/TS graph edges,
named export fan-in, `SAFE_FIX`, `EXISTS`, package edits, and dead-export
ranking.

Dynamic components, global registration, auto-import plugins, namespace member
components, custom elements, and framework magic are capability gaps until a
future spec proves stable reason codes and fixture coverage.

The first source implementation emits `symbols.json.sfcTemplateComponentRefs[]`
for explicit Vue/Svelte/Astro component bindings only. Positive evidence is
review-only and carries `eligibleForFanIn: false` plus
`eligibleForSafeFix: false`. Dynamic components and namespace-member tags are
muted, missing bindings are diagnostic-only, and unbound custom/native/global
tags stay out of the evidence surface. The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

## P5 Candidate: Explicit Global Component Registration

The explicit Vue global component registration lane was selected in
[`wt-sfc-global-component-registration-fixture-inventory-2026-05-26.md`](../lab/wt-sfc-global-component-registration-fixture-inventory-2026-05-26.md).

Decision: `explicit-registration-before-framework-convention` and
`registration-evidence-is-not-template-consumption`.

This lane is deliberately narrower than auto imports or Nuxt conventions.
Literal `app.component("Name", Binding)` and plugin `install(app)` registration
syntax can prove that a component name may be made available to templates. That
is availability evidence, not a proof that any template uses the component. The
first safe implementation should use a separate surface such as
`symbols.json.sfcGlobalComponentRegistrations[]` and keep it out of graph
edges, named export fan-in, deadness, `SAFE_FIX`, `EXISTS`, package edits, and
default action lanes.

The beta.68 public-install verification records
`global-registration-public-verified` and
`registration-availability-not-template-consumption`:
[`wt-sfc-beta68-global-component-registration-verification-2026-05-27.md`](../lab/wt-sfc-beta68-global-component-registration-verification-2026-05-27.md).
The source implementation recognizes explicit `app.component(...)`,
`createSSRApp(...)`, and app-returning chains such as
`createApp(...).use(router)`, while excluding `createApp(...).mount(...)`.
SFC targets stay muted as non-source bindings while preserving `resolvedFile`
for reviewer navigation, and source targets may resolve. The contract is pinned
by [`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Nuxt file-system auto-registration, `unplugin-vue-components`, generated
component registries, and non-literal component names remain capability gaps
until their own fixtures and reason codes exist.

The next global-registration refinement is recorded in
[`wt-sfc-global-registration-p2-fixture-inventory-2026-05-30.md`](../lab/wt-sfc-global-registration-p2-fixture-inventory-2026-05-30.md).
It selects plugin `install(app) { app.component(...) }` syntax, literal async
component factories, and duplicate literal registrations as the next bounded
P2 fixture set. Plugin install syntax stays syntax-level evidence, async
factories stay muted, duplicate registrations stay ambiguous, and the lane
still does not enter graph edges, named export fan-in, deadness, `SAFE_FIX`,
`EXISTS`, fix-plan, export-action, template refs, or default action lanes.

## P6 Candidate: Framework Magic And Convention Registration

The framework magic inventory is recorded in
[`wt-sfc-framework-magic-fixture-inventory-2026-05-27.md`](../lab/wt-sfc-framework-magic-fixture-inventory-2026-05-27.md).

Decision: `framework-magic-inventory-before-implementation`,
`generated-manifest-is-availability-not-absence`,
`convention-and-compiler-magic-stay-muted-or-unavailable`, and
`scan-gap-stays`.

The inventory uses three buckets:

- `explicit-supportable`: generated declarations or literal in-source shapes
  may become future review-only availability evidence;
- `muted`: observed framework config, convention files, directives, or macro
  shapes are visible but too weak for strong claims;
- `unavailable`: compiler rewrites, build virtual modules, custom resolvers,
  runtime injection, and framework-specific hidden wiring cannot produce
  per-instance facts in this lane.

The next safe implementation candidate is generated component-manifest
evidence, especially Nuxt `.nuxt/components.d.ts` and
`unplugin-vue-components` `components.d.ts`. These manifests can describe that a
component may be available to templates when present. Their absence is not proof
that framework registration is absent.

The P1 generated-manifest contract is recorded in
[`sfc-generated-component-manifest-evidence.md`](sfc-generated-component-manifest-evidence.md).
It selects a dedicated `symbols.json.sfcGeneratedComponentManifests[]` surface,
not `sfcGlobalComponentRegistrations[]`, and keeps missing generated-manifest
targets as `unresolved` evidence instead of silently dropping them. P1 remains
manifest-only: convention/config muted evidence is a future slice.

The first source implementation emits
`symbols.json.sfcGeneratedComponentManifests[]` for allow-listed
`components.d.ts` and `.nuxt/components.d.ts` manifests. SFC targets stay muted
with `resolvedFile`, source targets may resolve, stale manifest targets are
`unresolved`, and computed/nonliteral members are kept as `status: "skipped"`
with `sfc-framework-generated-manifest-nonliteral` so reviewers can see why no
target was inferred. Package/nonrelative imports and convention shapes still
stay out. The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

The next source implementation adds a separate
`symbols.json.sfcFrameworkConventionComponents[]` surface for Nuxt
`components/` filesystem convention files. This is muted framework-convention
evidence, not generated-manifest evidence: each observed `.vue` component under
`components/` records `sourceFile` / `resolvedFile` navigation only when a Nuxt
signal is present (`nuxt.config.*`, `.nuxt/components.d.ts`, or a root `nuxt`
package dependency), with `reason: "sfc-framework-nuxt-fs-convention"` and
`confidence: "framework-convention-observed"`, but it does not create graph
edges, named export fan-in, deadness proof, or action-lane entries. Convention
component names are derived from the path segments plus filename, matching
Nuxt's default path-prefix naming for nested components. The contract is pinned
by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

The remaining Nuxt app-dir, `#components`, layer, and custom resolver gap is
tracked in
[`wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md`](../lab/wt-sfc-nuxt-app-dir-custom-resolver-inventory-2026-05-31.md).
Decision: `nuxt-resolver-inventory-before-implementation`,
`generated-manifest-before-alias-resolution`,
`nuxt-app-dir-convention-stays-muted`,
`custom-resolver-stays-unavailable`, `scan-gap-stays`, and
`no-action-surface`. Future Nuxt work must keep generated manifests,
root `components/` convention evidence, `app/components/**`, `#components`
imports, literal component-dir config, custom resolver functions, and Nuxt
layers in separate explicit-supportable, muted-observed, or unavailable
buckets. None of those records may strengthen absence claims or enter graph,
fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF, or
package-edit lanes without a new fixture-pinned spec.

The first follow-up source implementation selects only the app-dir convention
subset: `.vue` files under `app/components/**` are recorded in the same
`symbols.json.sfcFrameworkConventionComponents[]` review-only surface when a
Nuxt app-dir signal is present: an explicit Nuxt 4 dependency range or a parsed
`nuxt.config.*` literal `srcDir: "app"` / `srcDir: "app/"`. Generic Nuxt 3
signals still enable root `components/` convention evidence, but do not enable
the app-dir root. Records use
`reason: "sfc-framework-nuxt-app-dir-convention"`,
`conventionKind: "nuxt-app-components-directory"`, path-derived Nuxt component
names, and `sourceFile` / `resolvedFile` navigation. They remain muted and do
not prove template consumption, graph edges, named export fan-in, deadness,
`SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF, package edits, or full
Nuxt resolver support.

The next follow-up source implementation selects only static Nuxt
`#components` named imports in SFC script blocks. When the imported component
name is backed by an allow-listed `.nuxt/components.d.ts` generated manifest
entry, the analyzer emits muted
`symbols.json.sfcFrameworkConventionComponents[]` evidence with
`reason: "sfc-framework-nuxt-components-alias-manifest"`,
`source: "sfc-framework-nuxt-components-alias"`, the consumer file, imported
name, local binding name, manifest file/kind, manifest binding source, and
`resolvedFile` when the manifest target exists. When a Nuxt signal is present
but the import has no generated-manifest mapping, the analyzer emits an
unresolved review-only record with
`reason: "sfc-framework-nuxt-components-alias-unresolved"` and no guessed file
path. Type-only imports, non-Nuxt projects, and known virtual helper exports
such as `componentNames` are ignored. This lane records
availability/navigation only: it does not create graph edges, dependency
consumers, named export fan-in, deadness proof, `SAFE_FIX`, `EXISTS`, fix-plan,
export-action, SARIF, package edits, or full Nuxt resolver support. The
contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

The next follow-up source implementation records only literal Nuxt component
directory config signals. Literal `components` / `components.dirs` values in
`nuxt.config.*` produce muted
`symbols.json.sfcFrameworkConventionComponents[]` evidence with
`reason: "sfc-framework-nuxt-components-dir-config"`, the config file,
literal configured directory, optional resolved directory navigation, and
literal prefix/path-prefix/global metadata when present. This is directory
availability evidence only: it does not scan those directories into component
records, infer Nuxt names, create graph edges, named export fan-in, deadness
proof, `SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF, package edits, or
full Nuxt resolver support. The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).
Resolution follows Nuxt source-directory semantics for literal aliases:
`~/...` and `@/...` resolve against literal `srcDir` when present, against the
Nuxt 4 default `app/` source directory when a Nuxt 4 dependency is detected,
and otherwise against the repository root.

The next follow-up source implementation records only high-level Nuxt component
hook presence for custom resolver paths. Literal `hooks` entries for
`components:dirs` and `components:extend` in `nuxt.config.*` produce
`symbols.json.sfcFrameworkConventionComponents[]` records with
`reason: "sfc-framework-nuxt-custom-resolver-unavailable"`,
`conventionKind: "nuxt-custom-resolver-unavailable"`, the config file, hook
name, and `status: "unavailable"`. This is a capability-gap disclosure only:
it does not execute the hook, inspect returned component lists, infer
component names, infer targets, create graph edges, named export fan-in,
deadness proof, `SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF, package
edits, or full Nuxt resolver support. The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

The next follow-up source implementation records only high-level Nuxt layer
`extends` config presence. Literal and nonliteral `extends` entries in
`nuxt.config.*` produce
`symbols.json.sfcFrameworkConventionComponents[]` records with
`reason: "sfc-framework-nuxt-layer-extends-unavailable"`,
`conventionKind: "nuxt-layer-extends-unavailable"`, the config file,
`configProperty: "extends"`, and `status: "unavailable"`. Literal entries may
preserve the configured `extendsSource`; nonliteral entries preserve only the
unavailable signal. This is a layer-merge capability-gap disclosure only: it
does not evaluate layers, infer component names, infer targets, create graph
edges, named export fan-in, deadness proof, `SAFE_FIX`, `EXISTS`, fix-plan,
export-action, SARIF, package edits, or full Nuxt resolver support. The
contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

The next follow-up source implementation records only high-level Nuxt
`modules` package config presence. Literal module package entries, tuple
entries such as `["@nuxtjs/tailwindcss", options]`, and nonliteral entries in
`nuxt.config.*` produce
`symbols.json.sfcFrameworkConventionComponents[]` records with
`reason: "sfc-framework-nuxt-module-package-unavailable"`,
`conventionKind: "nuxt-module-package-unavailable"`, the config file,
`configProperty: "modules"`, and `status: "unavailable"`. Literal and tuple
literal entries may preserve the configured `moduleSource`; nonliteral entries
preserve only the unavailable signal. This is a module-execution capability-gap
disclosure only: it does not execute modules, infer module-provided components,
infer targets, create graph edges, named export fan-in, deadness proof,
`SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF, package edits, or full
Nuxt resolver support. The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

Together, the current Nuxt framework-convention surface has three distinct
claim strengths:

- generated manifest and `#components` manifest-backed records are
  availability/navigation evidence when a generated declaration names a target;
- root/app component directory and literal component-dir config records are
  muted observed evidence for convention or config shapes;
- custom resolver hooks, layer `extends`, and `modules` config records are
  unavailable evidence that tells reviewers a resolver, layer merge, or module
  execution path exists but is not modeled.

That is the MVP boundary. It is useful, and it is deliberately not full Nuxt
resolver support. These records still do not create graph edges, named export
fan-in, deadness proof, `SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF,
package edits, or broad absence claims. If a future slice wants stronger Nuxt
semantics, it needs a new fixture-pinned spec.

The source implementation records `unplugin-vue-components`
Vite/Webpack config usage in the same
`symbols.json.sfcFrameworkConventionComponents[]` review-only surface. Config
files that import `unplugin-vue-components`, or require it from CommonJS
Webpack config, and call the plugin function emit muted evidence with
`reason: "sfc-framework-auto-import-plugin-config"` and
`confidence: "framework-convention-observed"`. This records the config file,
plugin import specifier, and call site only; it does not infer component names
or targets from custom resolvers, plugin transforms, or runtime framework
behavior. The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).
The beta.72 public-install verification records
`unplugin-config-public-verified` and
`inline-require-plugin-config-public-verified`:
[`wt-sfc-beta72-unplugin-config-verification-2026-05-28.md`](../lab/wt-sfc-beta72-unplugin-config-verification-2026-05-28.md).

The next source implementation records Astro `client:*` directives on
explicitly imported components in the same
`symbols.json.sfcFrameworkConventionComponents[]` review-only surface. A tag
such as `<UsedByAstro client:load />` emits muted evidence with
`reason: "sfc-framework-astro-client-directive"` only when the tag resolves
through the existing frontmatter/import binding model. The lane records the
consumer file, tag name, directive name, binding name, binding source, and line;
it does not infer Astro integration-injected components or any target not
grounded in an explicit binding. The fixture inventory is recorded in
[`wt-sfc-astro-client-directive-fixture-inventory-2026-05-28.md`](../lab/wt-sfc-astro-client-directive-fixture-inventory-2026-05-28.md).
The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).
The beta.73 public-install verification records
`astro-client-directive-public-verified` and
`explicit-binding-required-for-astro-client-evidence`:
[`wt-sfc-beta73-astro-client-directive-verification-2026-05-28.md`](../lab/wt-sfc-beta73-astro-client-directive-verification-2026-05-28.md).

The next source implementation records Svelte `use:action` directives when the
action name resolves to an explicit imported action or local function binding in
Svelte script. The evidence uses the same
`symbols.json.sfcFrameworkConventionComponents[]` review-only surface with
`reason: "sfc-framework-svelte-action-directive"` and
`confidence: "framework-convention-observed"`. It records the consumer file,
tag name, directive name, action name, binding source, and line, but it does not
infer arbitrary non-function local values, unbound actions, comment-only markup,
Svelte compiler reactivity beyond the fixture-pinned store lane, graph edges,
named export fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, or
export-action entries. The fixture inventory is recorded in
[`wt-sfc-svelte-action-directive-fixture-inventory-2026-05-29.md`](../lab/wt-sfc-svelte-action-directive-fixture-inventory-2026-05-29.md).
The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs); public
install verification records `svelte-action-directive-public-verified`,
`explicit-binding-required-for-svelte-action-evidence`, and
`svelte-action-evidence-stays-review-only`:
[`wt-sfc-beta74-svelte-action-directive-verification-2026-05-28.md`](../lab/wt-sfc-beta74-svelte-action-directive-verification-2026-05-28.md).

The next source implementation records Svelte `$store` auto-subscription
syntax when the store name resolves to an explicit import or a local
`writable` / `readable` / `derived` store factory binding from `svelte/store`.
The evidence uses the same
`symbols.json.sfcFrameworkConventionComponents[]` review-only surface with
`reason: "sfc-framework-svelte-store-subscription"` and
`confidence: "framework-convention-observed"`. It records the consumer file,
subscription name, store name, binding source, binding kind, and line, but it
does not infer unbound stores, comment-only markup, plain text `$name`
mentions, non-store local values, graph edges, named export fan-in, deadness,
`SAFE_FIX`, `EXISTS`, fix-plan, export-action, SARIF findings, or package-edit
entries. The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).

The next source implementation records Vue `<script setup>` macro component
registrations from literal `defineOptions({ components: { ... } })` objects in
the same `symbols.json.sfcFrameworkConventionComponents[]` review-only surface.
Only components backed by explicit non-type imports are recorded, with
`reason: "sfc-framework-vue-macro-registration"` and
`confidence: "framework-convention-observed"`. Dynamic/computed component
names, unbound identifiers, and comment-only/template text stay out. This lane
does not enter graph edges, named export fan-in, deadness, `SAFE_FIX`, `EXISTS`,
fix-plan, or export-action lanes. The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).
The beta.75 public-install verification records
`vue-macro-registration-public-verified`,
`explicit-binding-required-for-vue-macro-evidence`, and
`vue-macro-evidence-stays-review-only`:
[`wt-sfc-beta75-vue-macro-registration-verification-2026-05-28.md`](../lab/wt-sfc-beta75-vue-macro-registration-verification-2026-05-28.md).

The next source implementation records Vue Options API local component
registrations from literal `export default { components: { ... } }` objects in
ordinary Vue `<script>` blocks. Only components backed by explicit non-type
imports are recorded, with
`reason: "sfc-framework-vue-options-registration"` and
`confidence: "framework-convention-observed"`. Dynamic/computed component
names, unbound identifiers, and comment-only/template text stay out. This lane
does not enter graph edges, named export fan-in, deadness, `SAFE_FIX`, `EXISTS`,
fix-plan, or export-action lanes. The contract is pinned by
[`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs) and
[`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs).
The beta.76 public-install verification records
`vue-options-registration-public-verified`,
`explicit-value-binding-required-for-vue-options-evidence`, and
`vue-options-evidence-stays-review-only`:
[`wt-sfc-beta76-vue-options-registration-verification-2026-05-29.md`](../lab/wt-sfc-beta76-vue-options-registration-verification-2026-05-29.md).

Framework convention evidence must remain review-only: no graph edges, named
export fan-in, deadness, `SAFE_FIX`, `EXISTS`, package edits, fix-plan,
export-action-safety, or default action-lane changes. The `sfc-scan-gap` blind
zone remains visible until framework semantics are proven lane by lane.

## Audit Brief Surface

`manifest.json.sfcEvidence` mirrors only shallow SFC counts from `symbols.json`:
script import consumers, script-src reachability, style asset references,
template component refs, global registrations, generated component manifests,
and framework convention records. It must not mirror component names, tag names,
file spans, or raw per-record payloads.

`audit-summary.latest.md` and `audit-review-pack.latest.md` may surface those
counts as orientation text that points back to `manifest.json.sfcEvidence` and
the SFC arrays in `symbols.json`. The wording must preserve the review-only
boundary: review-only SFC lanes are not fan-in or action-tier proof, and
`sfc-scan-gap` still applies.

This contract is pinned by
[`tests/audit-manifest-export-surface.test.mjs`](../../tests/audit-manifest-export-surface.test.mjs)
and
[`tests/audit-repo-artifact-brief.test.mjs`](../../tests/audit-repo-artifact-brief.test.mjs).
The beta.78 public-install verification records
`sfc-evidence-brief-public-verified`,
`sfc-evidence-stays-count-only`, and
`sfc-raw-names-stay-out-of-markdown`:
[`wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md`](../lab/wt-sfc-beta78-sfc-evidence-brief-verification-2026-05-31.md).

## Script Src Contract

`<script src>` is not an `import` declaration. The first safe contract is
reachability of the referenced script file, not consumption of that file's
exports.

Implementation guidance:

- use a distinct source label such as `sfc-script-src`;
- preserve the SFC container file, raw specifier, resolved file, language, and
  confidence in the payload;
- feed reachability only through an edge kind or SFC-specific surface that
  dead-export fan-in can distinguish from named imports;
- do not mirror script-source evidence into
  `sfc-script-import-consumers`;
- keep missing, package, URL/data, generated, and non-literal sources out of
  concrete graph edges.

If an implementation chooses to route script-source evidence through
`resolvedInternalEdges`, it must use a non-import kind and the symbol fan-in lens
must ignore that kind. Using `import` or `import-named` here would be wrong.

## Invariants

1. Implementing one lane must not remove the broader `sfc-scan-gap`.
2. Unsupported SFC lanes must not create concrete graph edges.
3. Template text must never be scanned with regex-style import matching.
4. SFC script imports may protect exports only after the same resolver and
   import-kind normalization used by ordinary JS/TS imports.
5. `<script src>` may establish file reachability before it establishes symbol
   fan-in.
6. SFC evidence must stay out of `SAFE_FIX`, `EXISTS`, and package-edit lanes
   unless a future spec explicitly proves a stronger contract.
7. Every new SFC lane must add both Node and Vitest coverage before public
   verification.

## Acceptance For Current MVP

WT-SFC remains `MVP`, not `DONE`, until later lanes are addressed. The current
accepted surface is intentionally narrow:

- static script imports in supported SFC containers are visible in
  `symbols.json`;
- TSX/JSX dialect handling keeps real imports alive;
- literal relative Vue/Svelte `<script src>` creates reachability evidence with
  `kind: "sfc-script-src"`;
- script-source reachability does not create named export fan-in;
- package, URL/data, dynamic, empty, and missing script-source forms do not
  produce false concrete edges;
- literal relative style `url()` and style `@import` references are surfaced as
  style asset evidence, not graph edges;
- CSS escapes in style asset URLs are decoded before path resolution;
- missing relative style assets are diagnostic-only with reason
  `sfc-style-asset-unresolved`;
- template component references remain out of graph and fan-in until a
  binding-aware review-only surface is fixture-pinned;
- explicit template component binding evidence is isolated in
  `symbols.json.sfcTemplateComponentRefs[]` and keeps
  `eligibleForFanIn: false` plus `eligibleForSafeFix: false`;
- explicit global component registration is isolated in
  `symbols.json.sfcGlobalComponentRegistrations[]` as review-only availability
  evidence, not template consumption;
- generated component manifest entries are isolated in
  `symbols.json.sfcGeneratedComponentManifests[]` as availability evidence;
- Nuxt filesystem conventions, Nuxt `#components` aliases, Nuxt literal
  component-dir config, and unplugin config signals are isolated in
  `symbols.json.sfcFrameworkConventionComponents[]` as review-only framework
  convention evidence;
- Nuxt custom resolver hook, layer `extends`, and `modules` config shapes are
  isolated in `symbols.json.sfcFrameworkConventionComponents[]` as
  `status: "unavailable"` capability-gap evidence;
- Astro `client:*` directives on explicitly imported components are isolated in
  `symbols.json.sfcFrameworkConventionComponents[]` as muted framework
  convention evidence, not hydration reachability or named export fan-in;
- Svelte `use:action`, Svelte `$store` auto-subscriptions, Vue
  `defineOptions({ components })`, and Vue Options API `components: { ... }`
  records require explicit bindings and stay isolated in
  `symbols.json.sfcFrameworkConventionComponents[]` as muted framework
  convention evidence;
- `manifest.json.sfcEvidence`, `audit-summary.latest.md`, and
  `audit-review-pack.latest.md` surface shallow SFC counts and pointers only;
- template fake imports do not produce false consumers;
- external SFC imports feed dependency hygiene evidence;
- `sfc-scan-gap` remains visible as the absence-claim guard.

## Follow-Up Checklist

- Read the
  [`WT-SFC MVP status and remaining gaps`](../lab/wt-sfc-mvp-status-and-remaining-gaps-2026-05-31.md)
  before choosing the next SFC implementation lane. The older
  [`WT-SFC remaining gaps inventory`](../lab/wt-sfc-remaining-gaps-inventory-2026-05-26.md)
  is historical beta.67 context, not the current status record.
- Gather at least one Vue, one Svelte, and one Astro corpus before template
  component references affect deadness.
- Use the template component ref fixture inventory before implementing any
  template evidence surface.
- Treat explicit global component registration as public-verified review-only
  evidence; do not extend it to convention-driven registration without a new
  fixture inventory.
- Use the framework magic fixture inventory before implementing generated
  manifest or convention-registration evidence. Start with generated manifests
  before broad framework inference.
- Keep style-asset evidence isolated from graph edges, symbol fan-in, and
  action lanes unless a future spec proves a stronger contract.
- Keep SFC public-install verification runtime-based, not source-only.
- Use the
  [`WT-SFC corpus calibration plan`](../lab/wt-sfc-corpus-calibration-plan-2026-05-31.md)
  before changing absence claims, default Markdown wording, action lanes, or
  broad framework semantics.

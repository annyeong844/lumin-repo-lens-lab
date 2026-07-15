# Changelog

## Unreleased

### Rust review checklist recovery

- Restore the Rust structural-review companion to the source and packaged skill,
  and route Rust reviews through it without handing source-level judgment to a
  non-expert user.
- Cite macro/cfg opacity only from
  `rust-analyzer-health.latest.json.summary.syntaxReviewOpaqueSurfaces`; no
  emitted artifact field is named `compilerOracleOpaqueSurfaces`.

### Workspace dependency ownership

- Make native JS/TS pre-write dependency lookup resolve the nearest workspace
  `package.json` from dependency owner hints or a unique planned-file owner.
  Mixed workspace scopes now report `DEPENDENCY_OWNER_AMBIGUOUS` instead of a
  false `NEW_PACKAGE`; lockfiles remain insufficient as direct-declaration
  evidence.

### Tool-neutral lint enforcement evidence

- Discover `.oxlintrc.json` and named `.oxlintrc.*.json` files alongside
  ESLint configs, parsing active root and override rules into normalized C5
  boundary evidence.
- Record lint adapter/config/command status in
  `triage.json.lintEnforcement` instead of treating a missing ESLint config as
  proof that no lint enforcement exists.
- Fail C5 closed as `unknown` when a declared lint command is unsupported or a
  lint config is invalid and no independent boundary rule was recovered.
- Keep Rust formatting (`rustfmt`), Rust linting (`clippy`), and repository
  evidence (`--rust-analyzer`) as separate contracts.

### Export-identity public surface protection

- Stop expanding a named public re-export into file-wide `publicApi_FP23`
  protection for its source module.
- Keep direct package entry files protected while relying on symbol identity
  fan-in and namespace evidence for re-exported symbols, so unused sibling
  exports remain visible to dead-export classification.
- Route non-relative dotted aliases such as `@/app/layout.config` through the
  configured module resolver before applying non-source-asset classification,
  preventing Next.js route-group symbol scans from hard-stopping.
- Include every rejected source-use record ID and reason in symbol-graph
  hard-stop diagnostics instead of reporting only an opaque skipped count.

## 0.0.0-lab.0 - 2026-06-15

### Lab plugin fork

- Initialize the isolated `lumin-repo-lens-lab` source fork from stable Lumin
  Repo Lens `origin/main` commit `28dafdf69a4e69ecd5d6e0d04e524f947f157f34`.
- Separate lab plugin, package, command, and skill identities from the stable
  `/lumin-repo-lens:*` namespace.
- Point lab source work at `annyeong844/lumin_lab` and lab generated package
  work at `annyeong844/lumin-repo-lens-lab`.
- Add a lab publish guard that refuses to publish to the stable public package
  repo `annyeong844/lumin-repo-lens`.

## 0.9.0-beta.90 - 2026-06-14

### Staleness shared cache

- Cache `measure-staleness.mjs` results under the shared incremental cache
  root so full-profile audits can reuse staleness evidence across output
  directories when the git head, thresholds, pickaxe mode, and dead-candidate
  set are unchanged.
- Reduce cold-run blame overhead by using full-file blame only when multiple
  dead candidates share a file, while preserving the cheaper single-line blame
  path for one-off candidates.
- Forward incremental cache flags from `audit-repo.mjs` to staleness, record
  staleness cache/performance metadata, remove the dead topology
  `--incremental` flag, and update generated skill and plugin package versions
  including SARIF tool version metadata.

## 0.9.0-beta.89 - 2026-06-13

### Topology scanner fallback reduction

- Add capped `scannerFallbackExamples` to topology performance metadata so
  fallback reasons point at concrete files without leaking absolute paths.
- Keep unrelated interpolated template literals on the fast JS module-edge
  scanner path while preserving fallback for template dynamic imports,
  non-literal dynamic imports, and CommonJS `require` calls.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.88 - 2026-06-06

### Pre-write compact intent output

- Suppress benign missing-array intent default notes from rendered pre-write
  Markdown and CLI output while preserving the normalized `intentWarnings`
  evidence in `pre-write-advisory.latest.json`.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.87 - 2026-06-03

### Pre-write file-local function signature cues

- Index top-level file-local helper function signatures in `function-clones.json`
  so pre-write can warn before duplicating existing helper shapes.
- Preserve identifier-backed `export default` functions as exported `default`
  facts instead of routing public helpers through the file-local review-only
  lane.
- Ensure the standalone function-clone producer creates its output directory
  before writing `function-clones.json`.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.86 - 2026-06-01

### SFC Nuxt components alias helper filter

- Filter known Nuxt `#components` virtual helper exports such as
  `componentNames` out of component alias evidence instead of recording them
  as unresolved component diagnostics.
- Keep manifest-backed and unresolved component-like `#components` imports as
  review-only evidence while preserving the graph, fan-in, deadness, and
  action-surface isolation boundary.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.85 - 2026-06-01

### SFC Nuxt module package unavailable evidence

- Record literal, tuple-literal, and nonliteral Nuxt `modules` config entries
  as high-level unavailable framework convention evidence in
  `symbols.json.sfcFrameworkConventionComponents[]`.
- Preserve literal module package strings while keeping nonliteral module
  entries target-free, without executing modules, inferring module-provided
  component names, or entering graph, fan-in, deadness, or action-surface
  output.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.84 - 2026-06-01

### SFC Nuxt layer extends unavailable evidence

- Record literal and nonliteral Nuxt `extends` config entries as high-level
  unavailable framework convention evidence in
  `symbols.json.sfcFrameworkConventionComponents[]`.
- Preserve literal layer source strings while keeping nonliteral layer entries
  target-free, without evaluating layers, inferring component names, or
  entering graph, fan-in, deadness, or action-surface output.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.83 - 2026-06-01

### SFC Nuxt custom resolver unavailable evidence

- Record literal Nuxt `components:dirs` and `components:extend` hook presence
  as high-level unavailable framework convention evidence in
  `symbols.json.sfcFrameworkConventionComponents[]`.
- Preserve the custom-resolver boundary by exposing only config-file and
  hook-name metadata, without inferring component names, target files, graph
  edges, fan-in, deadness, or action-surface output.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.82 - 2026-06-01

### SFC Nuxt component directory config evidence

- Record literal Nuxt `components` / `components.dirs` directory config as
  muted review-only framework convention evidence in
  `symbols.json.sfcFrameworkConventionComponents[]`.
- Resolve `~/...` and `@/...` component directory paths through Nuxt `srcDir`
  semantics, including explicit `srcDir: "app"` and the Nuxt 4 default
  `app/` source directory, without scanning configured directories into
  component target records.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.81 - 2026-05-31

### SFC Nuxt components alias evidence

- Record manifest-backed Nuxt `#components` named imports as muted
  review-only framework convention evidence in
  `symbols.json.sfcFrameworkConventionComponents[]`.
- Keep unmapped Nuxt `#components` imports as unresolved review-only evidence
  without guessing targets, and suppress the alias from dependency,
  unresolved-internal, graph, fan-in, deadness, and action lanes.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.80 - 2026-05-31

### SFC Nuxt app-dir convention evidence

- Record Nuxt `app/components/**` convention files as muted review-only
  framework convention evidence only when a Nuxt app-dir signal is present:
  an explicit Nuxt 4 dependency range or parsed `srcDir: "app"` config.
- Keep Nuxt 3 dependency-only projects from emitting app-dir convention
  records while preserving existing root `components/` convention evidence.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.79 - 2026-05-30

### SFC Svelte local action evidence

- Record Svelte `use:action` directives backed by local function declarations
  or `const` function bindings in
  `symbols.json.sfcFrameworkConventionComponents[]` as muted review-only
  framework convention evidence.
- Keep non-function locals, unbound actions, and comment-only markup unclaimed;
  local action evidence stays out of graph edges, named export fan-in,
  deadness, and action lanes.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.78 - 2026-05-30

### SFC evidence audit brief surface

- Add a shallow `manifest.json.sfcEvidence` mirror for SFC evidence counts from
  `symbols.json`.
- Surface SFC evidence counts in `audit-summary.latest.md` and
  `audit-review-pack.latest.md` without component names or action wording.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.77 - 2026-05-30

### SFC advanced Vue global registration evidence

- Record Vue plugin `install(app)` global component registrations in
  `symbols.json.sfcGlobalComponentRegistrations[]` as review-only evidence.
- Add muted evidence for literal `defineAsyncComponent(() => import(...))`
  registrations while preserving navigation-only `resolvedFile` details.
- Mark duplicate literal global registrations as ambiguous only within the same
  receiver/API, so client and SSR apps can register the same component name
  without losing resolved evidence.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.76 - 2026-05-28

### SFC Vue Options API registration evidence

- Record literal Vue Options API `export default { components: { ... } }`
  entries in `symbols.json.sfcFrameworkConventionComponents[]` as muted
  review-only framework convention evidence.
- Keep Vue Options API evidence out of graph edges, named export fan-in,
  deadness, and action lanes; dynamic/computed names, unbound identifiers, and
  comment-only text stay unclaimed.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.75 - 2026-05-28

### SFC Vue macro registration evidence

- Record literal Vue `<script setup>` `defineOptions({ components })` entries in
  `symbols.json.sfcFrameworkConventionComponents[]` as muted review-only
  framework convention evidence.
- Keep Vue macro evidence out of graph edges, named export fan-in, deadness,
  and action lanes; dynamic/computed names, unbound identifiers, and
  comment-only text stay unclaimed.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.74 - 2026-05-28

### SFC Svelte action directive evidence

- Record explicitly imported Svelte actions used by `use:action` directives in
  `symbols.json.sfcFrameworkConventionComponents[]` as muted review-only
  framework convention evidence.
- Keep Svelte action evidence out of graph edges, named export fan-in, deadness,
  and action lanes; unbound actions and comment-only markup stay unclaimed.
- Update the generated skill and plugin package versions for a fresh
  installable beta, including SARIF tool version metadata.

## 0.9.0-beta.73 - 2026-05-28

### SFC Astro client directive evidence

- Record explicitly imported Astro components with `client:*` directives in
  `symbols.json.sfcFrameworkConventionComponents[]` as muted review-only
  framework convention evidence.
- Keep Astro directive evidence out of graph edges, named export fan-in,
  deadness, and action lanes; unresolved or intrinsic tags stay unclaimed.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.72 - 2026-05-28

### SFC unplugin config evidence

- Detect `unplugin-vue-components` config evidence for ESM imports, assigned
  CommonJS `require(...)`, and inline `require(...)()` plugin calls.
- Keep auto-import plugin config evidence muted and out of graph edges, named
  export fan-in, deadness, and action lanes.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.71 - 2026-05-27

### SFC Nuxt convention evidence

- Record Nuxt root `components/` `.vue` files in
  `symbols.json.sfcFrameworkConventionComponents[]` as muted review-only
  framework convention evidence.
- Keep filesystem convention evidence out of graph edges, named export fan-in,
  deadness, and action lanes; generated-manifest evidence stays separate.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.70 - 2026-05-27

### SFC generated manifest skipped evidence

- Surface computed/nonliteral generated component manifest members as
  `status: "skipped"` with
  `sfc-framework-generated-manifest-nonliteral`, instead of silently dropping
  them.
- Keep package/nonrelative manifest imports excluded and keep skipped manifest
  evidence out of graph edges, fan-in, deadness, and action lanes.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.69 - 2026-05-27

### SFC generated component manifest evidence

- Record Nuxt `.nuxt/components.d.ts` and `unplugin-vue-components`
  `components.d.ts` mappings in
  `symbols.json.sfcGeneratedComponentManifests[]` as review-only availability
  evidence.
- Keep generated-manifest evidence out of graph edges, named export fan-in,
  deadness, and action lanes; SFC targets stay muted with `resolvedFile` while
  source targets may resolve.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.67 - 2026-05-26

### SFC template component target evidence

- Preserve review-only SFC template component refs when a binding points at
  another SFC file: keep the ref muted as non-source evidence, but include the
  existing `resolvedFile` so reviewers can inspect the target.
- Keep SFC template target evidence out of graph edges, named export fan-in,
  deadness, and action lanes.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.66 - 2026-05-26

### SFC template component reference evidence

- Record explicit Vue/Svelte/Astro template component refs in
  `symbols.json.sfcTemplateComponentRefs[]` as review-only evidence.
- Keep template refs out of graph edges, named export fan-in, deadness, and
  action lanes; dynamic, namespace, and missing bindings remain weak evidence.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.65 - 2026-05-26

### SFC style asset evidence

- Record literal relative SFC style `url()` and `@import` references in
  `symbols.json.sfcStyleAssetReferences[]`.
- Keep style assets out of JS/TS graph edges and named export fan-in; missing
  relative style assets stay diagnostic-only.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.64 - 2026-05-25

### SFC script source reachability

- Record literal relative Vue/Svelte `<script src>` references as
  `sfc-script-src` reachability edges without treating the referenced file's
  named exports as consumed.
- Keep package, URL/data, dynamic, empty, and missing script-source forms out of
  concrete import edges; missing relative sources stay diagnostic-only.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.63 - 2026-05-25

### SFC script import consumers

- Extract static imports from Vue and Svelte inline `<script>` blocks and Astro
  frontmatter without parsing SFC templates as JavaScript.
- Feed SFC script imports into symbol fan-in, resolved internal edges, and
  dependency import consumers while keeping the broader SFC scan-gap warning.
- Honor declared SFC script dialects such as `lang="tsx"` so JSX/TSX imports do
  not silently disappear from graph evidence.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.62 - 2026-05-25

### SFC scan-gap surfacing

- Count Vue, Svelte, and Astro single-file components in `triage.json` without
  treating raw SFC containers as JavaScript parser inputs.
- Emit a grouped `sfc-scan-gap` blind zone so SFC-owned imports, exports, and
  template reachability are not silently folded into repo-wide absence claims.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.61 - 2026-05-25

### Block clone cap/noise v2 allocation

- Move block clone threshold metadata to `block-clone-threshold-policy-v2`
  with an internal candidate cap plus independent review/muted output caps.
- Preserve deprecated `maxGroups` as a final total compatibility cap and emit
  it in `block-clones.json.thresholds` when callers supply it.
- Mirror the v2 cap and saturation fields in `manifest.blockClones`.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.60 - 2026-05-25

### Block clone noise policy

- Add `block-clone-noise-policy-v1` so `block-clones.json` classifies repeated
  region groups as `review` or `muted` without deleting raw evidence.
- Mirror shallow review/muted group counts and mute reasons in
  `manifest.blockClones`, while keeping raw groups and source spans in
  `block-clones.json`.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.59 - 2026-05-24

### Block clone manifest mirror

- Mirror `block-clones.json` into shallow `manifest.blockClones` metadata for
  review-only status, normalization policy, threshold defaults, and summary
  counts.
- Keep raw clone groups, instances, and source spans in `block-clones.json`
  instead of expanding them into `manifest.json`.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.58 - 2026-05-24

### Review-only unused dependency Markdown surface

- Surface `unused-deps.json` in `audit-summary.latest.md` and
  `audit-review-pack.latest.md` with review-only counts and raw artifact paths.
- Keep dependency names in JSON evidence and avoid package-edit action wording
  in default Markdown.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.57 - 2026-05-18

### Review-only unused dependency artifact

- Add `unused-deps.json` as a review-only dependency hygiene artifact derived
  from declared package dependencies, observed import consumers, package script
  tool evidence, and workspace package ownership.
- Keep the first dependency hygiene slice artifact-only: no summary/review-pack
  Markdown surfacing, fix-plan/SARIF/package edits, or SAFE_FIX behavior.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.56 - 2026-05-18

### Runtime package script entry surfaces

- Treat package scripts that invoke `node`, `tsx`, `ts-node`, `ts-node-esm`,
  or `bun` with a JavaScript/TypeScript file as runtime entry evidence for
  `entry-surface.json`.
- Stop runtime script target parsing after the first executable input file so
  later positional arguments are not misreported as reachable entry files.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.55 - 2026-05-17

### Pre-write service-operation type-name filtering

- Preserve `definitionKind` on suppressed service-operation policy candidates
  and mute TypeScript type/interface/module declarations with
  `service-sibling-non-callable-definition` instead of promoting them as service
  operation siblings.
- Pin the VNplayer-style `ListLibraryDocsOptions` false-positive shape so
  `listLibraryDocs` can remain review evidence while the type-like options name
  stays muted.

## 0.9.0-beta.54 - 2026-05-17

### Pre-write local operation support reasons

- Add `local-operation-same-file-domain-overlap` to promoted
  `localOperationSiblingPolicy` entries so local-operation review Markdown no
  longer falls back to `unknown`.
- Preserve the review-only local-operation cue lane and keep
  `serviceOperationSiblingPolicy` isolated.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.53 - 2026-05-17

### Pre-write local operation review cues

- Render promoted `localOperationSiblingPolicy` entries as review-only
  `AGENT_REVIEW_CUE` cards with `local-operation-sibling` evidence.
- Show local-operation Markdown as `Review related local service operation`
  with policy evidence, container context, shared domain tokens, locality, and
  supporting local-operation reasons.
- Keep muted local-operation policy details hidden from default Markdown while
  preserving them in JSON `suppressedCues[]` for readers.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.52 - 2026-05-17

### Pre-write local operation policy

- Add `lookupName().localOperationSiblingPolicy` as the WT-23 P2a
  review-evidence surface for nested read/query operations observed in
  `symbols.json.preWriteLocalOperationIndex`.
- Keep nested local operations out of `serviceOperationSiblingPolicy`,
  `cueCards[]`, Markdown rendering, formal lookup lanes, and safe-action paths
  until a separate cue-integration slice is verified.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.51 - 2026-05-16

### Pre-write local operation index

- Add `symbols.json.preWriteLocalOperationIndex` as an artifact-only WT-23
  surface for nested read/query operations inside exported repository/service
  factories.
- Keep the first local-operation slice review-only: nested operations stay out
  of `defIndex`, `classMethodIndex`, formal lookup result lanes, and export
  safety paths.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.50 - 2026-05-13

### Pre-write service-operation sibling Markdown cues

- Render `service-operation-sibling` review cues in Markdown as explicit
  `Review related service operation` rows.
- Include the policy evidence path, policy version, operation family, shared
  domain tokens, locality, and supporting suppressed reasons while keeping
  muted service-operation policy details hidden by default.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.49 - 2026-05-13

### Pre-write service-operation sibling cue cards

- Promote `serviceOperationSiblingPolicy.promoted[]` entries into review-only
  JSON cue cards so likely read/query siblings are visible to agents without
  relaxing near-name or semantic thresholds.
- Mirror muted service-operation sibling policy entries into `suppressedCues[]`
  and keep class-method/generated-policy exclusions out of rendered cue cards.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.48 - 2026-05-13

### Pre-write service-operation sibling policy

- Emit a versioned `serviceOperationSiblingPolicy` evidence object from
  `lookupName()` so suppressed read/query service siblings such as
  `searchUser` → `fetchUser` are visible without relaxing near-name or semantic
  thresholds.
- Keep the first slice review-only: promoted policy entries do not become
  formal `nearNames`, `semanticHints`, cue cards, `EXISTS`, or `SAFE_FIX`
  evidence.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.47 - 2026-05-12

### Pre-write suppressed sibling diagnostics verification

- Publish the WT-23 suppressed candidate diagnostics for public install and
  corpus verification.
- Keep suppressed semantic and near-name candidates as muted evidence, not
  promoted `AGENT_REVIEW_CUE` or `SAFE_FIX` proof.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.46 - 2026-05-12

### Entry-unreachable SCC review evidence

- Record entry-unreachable runtime SCCs in `module-reachability.json` so
  mutually importing files that are unreachable from every entry surface do not
  look alive only because they import each other.
- Surface unreachable SCC counts in `audit-summary.latest.md` and review-pack
  Lane 3 as dead-file-group review evidence, not export-level `SAFE_FIX` proof.
- Update the generated skill package version for a fresh installable beta.

## 0.9.0-beta.45 - 2026-05-11

### Class method prototype-key hardening

- Fix class method indexing for prototype-named methods such as `constructor`,
  `toString`, `hasOwnProperty`, `valueOf`, and `__proto__` by using
  null-prototype dictionaries for method buckets.
- Sweep nearby artifact summary accumulators to null-prototype dictionaries so
  package names, artifact names, or reason keys cannot collide with inherited
  object properties.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.44 - 2026-05-11

### Output-to-source layout diagnostics

- Publish the WT-20 `output-to-source-mapping` unsupported resolver family for
  public install verification.
- Preserve candidate-level output/source layout evidence so unresolved
  compiled output paths do not silently become deadness evidence.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.43 - 2026-05-11

### Framework/resource capability pack ownership

- Surface stable `capabilityPack` owners on framework/resource surface lanes,
  so downstream readers do not infer support ownership from path tokens or
  reason strings.
- Mirror `summary.byCapabilityPack` through the raw
  `framework-resource-surfaces.json` artifact and the manifest summary.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.42 - 2026-05-11

### Class method pre-write lookup

- Surface TS/JS class methods in `symbols.json.classMethodIndex` separately
  from `defIndex`, so pre-write can find method-shaped reuse candidates
  without treating class members as dead-export identities.
- Render class-method near-name and intent-token matches as
  `AGENT_REVIEW_CUE` evidence through the `class-method-name` lane.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.41 - 2026-05-10

### Wildcard alias resolver probes

- Cache wildcard alias resolver probes by specifier within each resolver run,
  covering resolved paths, no-match fallthroughs, unresolved internal sentinels,
  and generated virtual surfaces.
- Freeze cached generated virtual surface results so repeated wildcard cache
  hits cannot be corrupted by accidental caller mutation.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.40 - 2026-05-10

### Scoped tsconfig resolver probes

- Cache package-scoped tsconfig resolver probes so repeated pass-through and
  resolved outcomes avoid probing the same package scope repeatedly.
- Preserve scoped tsconfig probe telemetry, including cache-hit and cache-miss
  counters, so public verification can measure the cache behavior directly.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.39 - 2026-05-10

### Scoped baseUrl cache telemetry

- Add explicit scoped `baseUrl` probe cache-miss counters so resolver telemetry
  can report hit ratios without inference.
- Surface `sourceUseResolverStage*CacheMisses` counters from symbol graph
  source-use resolution phases.
- Record the beta.38 public cal.diy WT-18 verification note and update the
  work tracker with the next measured resolver bottleneck.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.38 - 2026-05-10

### Scoped baseUrl resolver probes

- Cache package-scoped `baseUrl` resolver probes so repeated misses do not
  re-probe the same package scope while preserving scoped resolution behavior.
- Keep unresolved resolver outcomes and diagnostics stable when cached probe
  results are reused.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.37 - 2026-05-10

### Namespace re-export precision

- Resolve namespace objects that pass through named re-exports, so
  `export * as ns from "./source"` followed by `export { ns } from "./barrel"`
  preserves exact member fan-in for observed namespace member reads.
- Keep unused source exports dead when only sibling namespace members are read,
  while reporting opaque namespace object escapes through
  `symbols.json.namespaceReExportDiagnostics`.
- Update WT-16 tracking to require public install verification before closing
  the namespace recall bug.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.36 - 2026-05-09

### Producer performance instrumentation

- Add `producer-performance.json` with run metadata, scan/cache settings,
  per-producer wall time, skipped steps, and a `manifest.performance` summary
  link.
- Add artifact byte summaries, largest artifact samples, and honest
  orchestrator-process memory snapshots without claiming child producer peak
  RSS.
- Update WT-18 tracking to keep child peak memory, JSON read/parse counters,
  and scheduler dependency graph work explicit follow-ups.
- Update the generated skill and plugin package versions for a fresh
  installable beta.

## 0.9.0-beta.35 - 2026-05-09

### Pre-write evidence availability and method-surface planning

- Add pre-write evidence availability metadata and Markdown warnings so missing baseline artifacts do not make `NOT_OBSERVED` look like grounded absence.
- Mirror pre-write evidence availability into `manifest.preWrite` for orchestrated `audit-repo --pre-write` runs.
- Add the maintainer spec and tracker entry for TS/JS class-method pre-write search, keeping method cues separate from `symbols.json.defIndex`.
- Update the generated skill package version for a fresh installable beta.

## 0.9.0-beta.34 - 2026-05-09

### Resolver blocker distribution

- Add deterministic reason/family distributions for resolver blocked absence
  hints in manifest resolver diagnostics.
- Surface the compact resolver blocker distribution in `audit-summary.latest.md`
  and `audit-review-pack.latest.md` Lane 3 before the raw hint examples.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.33 - 2026-05-09

### Review-pack resolver blockers

- Surface resolver blocked absence hints in `audit-review-pack.latest.md`
  Lane 3 so reviewer-facing dead-export/public-surface review sees the same
  scoped blind-zone warning as the audit summary.
- Add `docs/spec/lumin-work-tracker.md` so MVP slices keep explicit follow-up
  and verification states instead of being mistaken for complete work.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.32 - 2026-05-09

### Resolver diagnostic sample metadata

- Add `blockedCandidateHintSampleLimit` to manifest resolver diagnostics so
  count/sample differences are explicit.
- Include the manifest sample limit in the audit summary resolver blocked
  absence hint line.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.31 - 2026-05-09

### Resolver diagnostic summary surface

- Carry resolver `blockedCandidateHints` into manifest resolver diagnostics
  summary metadata.
- Surface blocked absence hint examples in `audit-summary.latest.md` so agents
  see which candidate surfaces resolver blind zones can block.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.30 - 2026-05-09

### Resolver diagnostic blocking hints

- Add compact `blockedCandidateHints` to `resolver-diagnostics.json` so
  candidate-relevant blind zones can explain which absence claims they block.
- Include `blockedCandidateHintCount` in resolver diagnostics summary metadata.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.29 - 2026-05-09

### Resolver and threshold evidence contracts

- Add a named calibration corpus registry and attach compact corpus summaries
  to threshold policy metadata.
- Add threshold-only hashes plus a drift snapshot guard so numeric threshold
  edits require explicit policy/calibration review.
- Mark resolver candidate targets as diagnostic-only, non-edge evidence in
  `resolver-diagnostics.json`.
- Add a structured `capabilityReference` from resolver diagnostics to the
  matching static resolver capability matrix.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.28 - 2026-05-09

### HTML entry resolution

- Prevent resolved HTML module script entries from emitting non-existent
  extension-probe variants such as phantom `.jsx` siblings.
- Keep nested Vite-style HTML entry resolution grounded to the concrete
  resolved source file in entry-surface and reachability artifacts.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.27 - 2026-05-09

### HTML entry resolution

- Resolve absolute HTML module script targets against the HTML file directory
  before falling back to the package root.
- Keep unresolved HTML entry diagnostics when neither candidate exists, so
  unknown static-server mappings remain visible as confidence gaps.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.26 - 2026-05-08

### Bounded call-graph evidence

- Add bounded depth-1 member-call resolution for imported exported-object APIs.
- Emit bounded member-call support metadata and per-file bounded/member call
  stats in `call-graph.json`.
- Require bounded stats before `rank-fixes` can attach
  `call-graph-no-observed-callers`, preventing optimistic evidence from older
  call-graph artifacts.

## 0.9.0-beta.25 - 2026-05-08

### Generated consumer blind-zone reporting

- Add manifest summary fields for generated consumer blind zones so large
  generated-surface gaps are visible without opening `symbols.json` first.
- Surface top generated consumer blind-zone scopes in `audit-summary.latest.md`
  before agents trust generated code absence claims.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.24 - 2026-05-08

### Generated consumer blind zones

- Add explicit `symbols.json.generatedConsumerBlindZones` inventory for missing
  or out-of-scope generated target surfaces.
- Scope generated consumer blind-zone `SAFE_FIX` blocking to the generated
  package or target submodule instead of every importing consumer submodule.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.23 - 2026-05-08

### Generated blind-zone relevance

- Narrow generated artifact blind-zone relevance so provider misses no longer
  taint every candidate in the importing consumer submodule.
- Keep generated provider package roots and target-candidate surfaces as
  relevant `SAFE_FIX` blocking evidence, preserving conservative behavior where
  the candidate actually intersects the missing generated surface.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.22 - 2026-05-08

### Generated asset diagnostics

- Classify missing relative generated asset imports as
  `workspace-generated-artifact-missing` when the nearest package script
  explicitly emits the requested target path.
- Preserve the missing generated asset as unresolved blind-zone evidence rather
  than pretending a generated file exists in source checkout.
- Update the generated skill mirror and public package version for a fresh
  installable beta.

## 0.9.0-beta.21 - 2026-05-07

### Resolver diagnostics and workspace fallback

- Resolve source-checkout tsconfig path misses through concrete workspace
  package subpath fallbacks when the matched tsconfig target points at absent
  generated output.
- Add `manifest.resolverDiagnostics` with top unresolved reasons and specifier
  roots so large monorepos can explain remaining resolver blind zones.
- Surface top unresolved roots in `audit-summary.latest.md`, keeping the brief
  actionable without changing resolver ranking or `SAFE_FIX` policy.

## 0.9.0-beta.20 - 2026-05-06

### Public deep-import review clarity

- Add explicit public deep-import risk reasons to review evidence so agents can
  distinguish publishable packages without `exports` from wildcard or explicit
  package surfaces.
- Summarize public deep-import review blockers in `fix-plan.summary` by reason,
  making large review batches actionable without changing tier policy.

## 0.9.0-beta.19 - 2026-05-06

### CommonJS precision

- Track exact CommonJS export surfaces, alias destructuring consumers, and
  dynamic `require(...)` opacity in the generated symbol graph.
- Report genuinely dynamic CommonJS requires as structured blind-zone evidence
  instead of silently making dead-export absence claims nearby.
- Treat static JSON metadata reads such as
  `require(path.resolve(..., "package.json"))` as package metadata rather than
  CommonJS dynamic require opacity.

## 0.9.0-beta.18 - 2026-05-05

### Unresolved blind-zone explanations

- Add `symbols.json.unresolvedInternalSummaryByReason`, grouping unresolved
  internal imports by resolver reason with deterministic counts, hints,
  resolver stages, and compact examples.
- Keep resolver and `SAFE_FIX` ranking behavior unchanged; the new summary is
  explanatory evidence for large repos, not a promotion signal.

## 0.9.0-beta.17 - 2026-05-05

### Safe-fix declaration dependency handling

- Keep declaration-dependent dead exports eligible for `SAFE_FIX` when the
  selected action preserves the local binding, such as
  `demote_export_declaration` or `remove_export_specifier`.
- Continue to block delete-style actions for declaration dependencies with the
  `declaration-dependency-not-preserved` review reason.
- Extend export-action safety so resolvable type/interface declaration
  dependency candidates can receive proof-carrying demotion actions.

## 0.9.0-beta.16 - 2026-05-05

### Pre-write cue tiers

- Add pre-write cue classification that separates grounded facts, agent-review
  cues, muted token noise, and unavailable evidence in both JSON artifacts and
  Markdown output.
- Suppress weak common-token matches such as `create`-only candidates from the
  default chat surface while preserving diagnostic details in `suppressedCues`.
- Update the generated skill mirror and public package version so Claude Code
  installs a fresh marketplace cache containing the cue-tier engine changes.

## 0.9.0-beta.14 - 2026-05-04

### Write-gate feedback fixes

- Load absolute `preWrite.anyInventoryPath` values as exact paths during
  post-write, instead of joining them under the advisory or output directory.
- Prefer strong two-token basename-prefix siblings such as `merge-with-*`
  before falling back to broad single-token domain clusters.
- Clarify README expectations: pre-write is the fast name/path/topology
  transaction gate, full profile carries broader duplicate evidence, semantic
  equivalence still requires code reading or embedding, and post-write
  currently pays for a fresh after-snapshot.
- Move the first-run README path from the quick profile to `:full`, so shape
  index, function-clone, public-surface policy, and post-write workflow value
  are visible before users judge the tool as a dead-export sorter.

### Safe-fix contract hardening

- Gate `SAFE_FIX` promotion for publishable packages whose package surface
  still allows external deep imports, keeping those candidates review-visible
  under `public-deep-import-risk`.
- Keep private workspace packages eligible for safe export demotion when
  deadness proof and action proof are both clean.

## 0.9.0-beta.13 - 2026-05-04

### Self-audit export surface cleanup

- Trim internal helper exports surfaced by the engine's own SAFE_FIX audit
  candidates while preserving each helper's internal runtime use.
- Add export-surface regression tests for the classify policy, manifest,
  function clone, definition id, and post-write file-delta helper modules.
- Regenerate the shipping skill mirror so Claude Code installs the cleanup
  under a fresh public beta cache key.
- Self-audit result after cleanup: `SAFE_FIX = 0`, `safeFixGroups = 0`,
  `REVIEW_FIX = 0`, `DEGRADED = 0`, with no blind zones.

## 0.9.0-beta.12 - 2026-05-03

### Public plugin cache refresh

- Bump the public beta package version so Claude Code installs a fresh plugin
  cache containing the `0.9.0-beta.11` engine changes.
- This release carries the merged Cloudflare Worker entry protection and
  `safeFixGroups` artifact output under a new installable version key.

## 0.9.0-beta.11 - 2026-05-03

### Large-repo classify scaling

- Add a safe text-zero reference shortcut so candidates whose identifier
  appears exactly once on the declaration line skip AST walking without
  dropping large files or lowering evidence precision.
- Cache per-file provenance and importer-scoped tsconfig alias filtering so
  unresolved-specifier taint no longer scales as candidates x unresolved specs
  x alias entries.
- Keep file-size degradation opt-in (`--classify-max-file-bytes`, default `0`)
  and record classify performance metadata including text-zero candidates,
  provenance cache entries, unprocessed candidates, and file-size cap status.
- Stress-test result: `next.js-canary --production --profile quick` completed
  4,873 files in 221.5s with `classify.incomplete=false`,
  `unprocessedCandidates=0`, `maxFileBytes=0`, and 92 SAFE_FIX candidates;
  Hono, Kit, Astro, Nuxt, and Nest stress runs also completed without
  incomplete classify artifacts.

## 0.9.0-beta.10 - 2026-05-02

### Static SAFE_FIX calibration

- Recalibrate `SAFE_FIX` to mean static-graph-clean mechanical cleanup under
  the recorded scan range, instead of requiring optional runtime coverage and
  git staleness evidence.
- Allow bucket A export-demotion candidates to rank `SAFE_FIX` when local
  provenance is clean; declaration dependencies, policy exclusions, blocking
  taint, and runtime-executed contradictions still block SAFE_FIX.
- Update ranking tests and public wording so cleanup value does not collapse to
  hundreds of review-only candidates in repos without coverage or git history.

## 0.9.0-beta.9 - 2026-05-02

### Invalid tsconfig fixture tolerance

- Skip unusable `tsconfig*.json` files when TypeScript throws while reading or
  parsing config fixtures, instead of aborting alias discovery.
- Add regression coverage for malformed tsconfig fixtures living beside valid
  sibling configs.
- Stress-test result: `astro-main --production` now completes the required
  symbol graph step and reports `parseErrors: 0`, `blindZones: 0`, and
  `unresolvedInternalRatio: 0.0045`.

## 0.9.0-beta.8 — 2026-05-02

### Production-scope test-path calibration

- Treat `runtime-tests/` and `test-utils/` directories as test-like paths for
  `--production` scans, so runtime harness and test helper exports do not leak
  into production dead-export proposals.
- Add regression coverage in the shared path classifier and file collector.
- Stress-test results: `hono-main --production` exposed
  `runtime-tests/workerd/index.ts`, and `kit-main --production` exposed a
  root `test-utils/` directory. The shared classifier now filters both
  conventions consistently before downstream dead-export bucketing.

## 0.9.0-beta.7 — 2026-05-02

### Declaration-file parser blind-zone reduction

- Parse `.d.ts`, `.d.mts`, and `.d.cts` files with `oxc-parser`'s
  declaration-file mode (`lang: "dts"`) instead of ordinary TypeScript mode.
- Add regression coverage for declaration-only value exports such as
  `export const runtimeDependencies: string[];`.
- Stress-test result: `nuxt-main --production --exclude nuxt-main` now reports
  `parseErrors: 0`, `blindZones: 0`, and records
  `packages/nuxt/meta.d.ts::runtimeDependencies` as a definition.

## 0.9.0-beta.6 — 2026-05-02

### JSX-in-JS parser blind-zone reduction

- Retry OXC parsing in JSX mode when `.js`/`.mjs`/`.cjs` files fail in
  plain JS mode, covering Next.js and React route/component files that keep
  JSX syntax in `.js` sources.
- Align `manifest.confidence.parseErrors` with symbol-graph parse-error
  warnings that use `code: "parse-errors"`, with a fallback to
  `filesWithParseErrors.length`, so confidence and blind-zone reporting stay
  consistent.
- Add regression coverage for JSX-in-JS parse handling and manifest parse
  error confidence.
- Stress-test result: `next.js-canary --production` parser gaps dropped from
  511 files to 3 remaining non-JSX syntax gaps (Babel `do` expression, Flow,
  and one `.d.ts` parser edge).

## 0.9.0-beta.5 — 2026-05-02

### Package-scoped framework policy safety

- Replace broad framework path muting in dead-export classification with
  package-scoped framework evidence plus specific protected convention
  matching, so root framework dependencies do not hide nested workspace
  exports.
- Add a pure framework policy matrix for Next.js, Remix/React Router,
  Hono, SvelteKit, Astro, Nuxt/Nitro, and NestJS safety boundaries.
- Add Hono route registration facts from OXC-parsed source so Hono handlers
  are protected only when passed to route APIs, not by `routes/` path shape.
- Keep weak framework matches review-visible and emit phase-1 aggregate
  `summary.frameworkPolicy` counters for muted findings, review hints,
  rejected signals, and path-shaped candidates kept visible.
- Add release-blocking corpus fixtures for Next package scope, Next proxy,
  Hono handlers, SvelteKit route exports, Astro endpoint exports, React
  Router review hints, and Nest/Nuxt false-mute prevention.

## 0.9.0-beta.4 — 2026-05-02

### Stress-test resolver and policy follow-up

- Map workspace package outputs like `./dist/index.js` and `./dist/*.js`
  back to package-root source files (`index.ts`, `api.ts`, ...), covering
  Cal.com-style `@calcom/platform-*` packages whose authored source is not
  under `src/`.
- Keep missing matched workspace package targets classified as resolver
  blindness, but stop reporting the resolvable dist-root package imports as
  unresolved internal edges.
- Narrow Nuxt/Nitro FP-30 detection so `@nuxt/opencollective` does not
  activate broad filesystem-route muting in non-Nuxt projects such as NestJS.

## 0.9.0-beta.3 — 2026-05-02

### Public naming cleanup

- Remove legacy `lumin-audit` and `grounded-audit` CLI aliases from the
  generated package so the public beta presents one current CLI name:
  `lumin-repo-lens-lab`.
- Remove deprecated `GROUNDED_AUDIT_*` auto-install opt-out environment
  variables; use `LUMIN_REPO_LENS_NO_AUTO_INSTALL=1`.
- Rename the generated sibling skill surfaces to
  `lumin-repo-lens-lab-write-gate` and `lumin-repo-lens-lab-canon` so all shipped
  surfaces share the public plugin family name.
- Align public English skill/reference/template docs on the `unknown`
  evidence label and add surface tests so Korean epistemic tokens do not
  reappear in the marketplace-facing docs.
- Resolve app-scoped `compilerOptions.baseUrl` imports such as `app/_types`
  without requiring a `paths` entry, and surface missing baseUrl-local imports
  as resolver blindness instead of silently treating them as external packages.
- Narrow Nuxt/Nitro FP-30 muting so a bare `h3` dependency does not hide
  ordinary `middleware/` or `plugins/` exports in non-Nuxt server projects.

## 0.9.0-beta.2 — 2026-05-02

### Public beta packaging polish

- Clean stale `dist/lumin-audit-plugin/` output when building the current
  `dist/lumin-repo-lens-lab-plugin/` package.
- Align `/lumin-repo-lens-lab:full` command metadata with its `full` routing mode.
- Keep this package change on a new beta version so Claude Code update
  caching can see the change.

## 0.9.0-beta.1 — 2026-05-02

### Public marketplace beta

- Re-label the first public Claude Code marketplace package as a beta
  release instead of a stable `1.x` line.
- Keep the generated plugin metadata, deployable skill package, and SARIF
  tool version aligned on `0.9.0-beta.1`.
- Mark the prior `1.11.11` public release as superseded so official
  marketplace submission can point at the beta tag.

## 1.11.11 — 2026-04-30

### Thin Codex install

- Replace the duplicate Codex engine package with Codex metadata on the
  shared generated `skills/` surfaces, matching the superpowers-style
  clone-and-link install model.
- Put Codex install instructions in the shipped README, including
  macOS/Linux symlink and Windows junction instructions.
- Keep one runtime engine copy so Codex and Claude Code surfaces cannot
  drift apart.

## 1.11.10 — 2026-04-30

### Codex skill surface

- Add a Codex-native generated skill at `codex/skills/lumin-repo-lens-lab/`
  with a concise `SKILL.md`, `agents/openai.yaml`, and the same runtime
  engine/scripts as the Claude Code package.
- Keep the Codex surface separate from Claude Code plugin discovery so
  slash-command routing and Codex skill routing do not drift into each
  other.
- Teach `scripts/build-skill.mjs` to generate both surfaces from the
  maintainer repo.

## 1.11.9 — 2026-04-30

### Subagent review boundary

- Clarify that `audit-review-pack.latest.md` is a main-controller
  artifact brief, not a subagent prompt.
- When Claude Code uses reviewer subagents for full/deep review, the
  main assistant must translate lane cues into focused codebase-reading
  assignments with concrete files, symbols, or hypotheses.
- Subagents should inspect repository files directly and report
  file:line evidence instead of trusting checklist or artifact summaries.

## 1.11.8 — 2026-04-30

### Literal union shape lens

- Extend `shape-index.json` to hash exported literal-union type aliases
  such as `"open" | "closed"` so full-profile B1/B2 review can catch
  duplicate status/event unions, not only object/interface shapes.
- Keep the lens conservative: unions mixed with broad members such as
  `string` remain diagnostic-only rather than fuzzy matches.

## 1.11.7 — 2026-04-29

### Full audit slash command

- Add `/lumin-repo-lens-lab:full` as a one-click full profile audit command so
  the Claude Code slash menu exposes deep review directly instead of
  requiring users to remember `/lumin-repo-lens-lab:audit --profile full`.
- Keep `/lumin-repo-lens-lab:audit --profile full` supported for explicit CLI
  argument use.

## 1.11.6 — 2026-04-29

### First-run setup UX

- Make automatic dependency setup speak in first-run user terms: the
  wrapper now says it is installing parser dependencies locally and names
  `LUMIN_REPO_LENS_NO_AUTO_INSTALL=1` as the opt-out.
- Keep `GROUNDED_AUDIT_SKIP_AUTO_INSTALL=1` as a deprecated
  compatibility alias so older tests and scripts do not break abruptly.
- Allow the welcome flow to mention the one-time dependency setup, and
  add a smoke/contract/regression map to the generated test README.
- Dogfood the self-audit `REVIEW_FIX` lane by demoting internal-only
  helper exports in the maintainer engine, reducing current self-audit
  review-fix dead-export candidates from 13 to 0.
- Rebrand public-facing metadata toward `lumin-repo-lens-lab` under
  `annyeong844/lumin-repo-lens-lab`, including the Claude Code skill name and
  slash-command namespace. Keep `grounded-audit` only as a CLI
  compatibility alias.

## 1.11.5 — 2026-04-29

### First-run dependency setup

- Add a runtime dependency guard to the public audit orchestrator. Help
  output still works without dependencies, while real runs verify the
  supported Node range and required parser packages up front.
- In generated skill packages, automatically attempt one local
  `npm ci --omit=dev --ignore-scripts` setup when runtime dependencies
  are missing, then continue if the install succeeds.
- If automatic setup cannot run, print the exact setup command instead
  of failing later with a low-level parser import error.

## 1.11.4 — 2026-04-29

### Checklist pass before short answers

- Clarify that short chat-facing reviews still require an internal
  checklist triage pass across the main review lenses before selecting
  the few user-visible items.
- Keep `REVIEW_CHECKLIST_SHORT.md` as an output-density template rather
  than a shortcut around checklist evidence.
- Add audit cadence guidance: first baselines, stale artifacts, explicit
  review, due diligence, large refactor planning, and post-refactor
  review use full evidence; small follow-ups can stay quick.

## 1.11.3 — 2026-04-29

### Truth before warmth

- Add an escape hatch for chat-facing reviews and refactor plans: when
  grounded strengths are thin, use `Current State` and the next safe
  action instead of manufacturing praise.
- Pin the behavior in skill-surface and generated-package tests so the
  vibe-coder voice stays kind without weakening evidence.

## 1.11.2 — 2026-04-29

### Post-write confidence gate

- Add opt-in `--strict-post-write-confidence` so CI can fail when a
  post-write delta ran but the before baseline, scan-range parity, or
  after inventory makes the zero-new-escape claim confidence-limited.
- Clarify the write-gate skill description around plain-language change
  requests so the natural-language pre-write path is easier to route.

## 1.11.1 — 2026-04-29

### Release safety belt

- Reject unknown `audit-repo.mjs` options instead of silently falling
  back to defaults, and accept `--source` as a canon source alias for
  `--sources`.
- Caveat post-write summaries when baseline, scan-range parity, or
  after-inventory completeness limits the delta confidence.
- Clarify that the three sibling skill surfaces must be shipped as a
  Claude Code plugin-root package, not as a bare `skills/` archive root.

## 1.11.0 — 2026-04-29

### Three-surface skill split

- Split model-facing instructions into three surfaces:
  `auditing-repo-structure` for read-only audit and refactor-plan
  coaching, `grounded-write-gate` for the pre-write/post-write
  transaction, and `grounded-canon` for canon-draft/check-canon.
- Kept one shared engine under `skills/auditing-repo-structure/` so the
  sibling surfaces change routing and voice without duplicating runtime
  code.
- Updated plugin command delegators so write-gate and canon commands
  load their own slim `SKILL.md` first, then use the shared
  `command-routing.md`.

## 1.10.44 — 2026-04-29

### Drift hygiene

- Unified the Iron Law wording on "machine evidence" across SKILL and
  canonical invariants so non-AST evidence such as runtime, staleness,
  and coverage signals stays inside the public contract.
- Added a mode-dispatch mirror check that compares canonical trigger
  vocabulary against the runtime dispatcher, including the Korean
  `리팩토링해줘` spelling and guard terms.
- Clarified that soft hedging in chat is allowed only when backed by an
  internal `degraded` or `[확인 불가]` label.

## 1.10.43 — 2026-04-29

### LLM-facing identity

- Reframed the public contract from "vibe-coder-facing skill" to
  "LLM-facing repo evidence engine with vibe-coder-friendly answers" so
  the tool's operator and the human reader are no longer collapsed into
  one audience.
- Updated README, SKILL, command routing, package metadata, and plugin
  marketplace metadata to keep raw artifact vocabulary behind the
  assistant layer while preserving machine-evidence proof paths.

## 1.10.42 — 2026-04-29

### anyContamination owner facts

- Wired `build-symbol-graph.mjs` to emit per-identity
  `anyContamination` annotations for parsed TS/JS helper and type owners,
  backed by occurrence-level `type-escape` facts.
- Added `helperOwnersByIdentity` and `typeOwnersByIdentity` to
  `symbols.json`, including clean owner facts with `anyContamination:
  null` and measured dirty owners with `{label, labels, measurements}`.
- Covered exported type declarations and JSDoc `{any}` on exported JS
  constants so `.mjs` + JSDoc code no longer sits outside the default
  contamination surface.
- Routed the new identity-level contamination signal into
  `audit-summary.latest.md` and `audit-review-pack.latest.md`, so
  reviewers are pointed at `symbols.json` owner maps instead of seeing
  only occurrence-level discipline totals.

## 1.10.41 — 2026-04-29

### LLM-authored summary surface

- Reframed `audit-summary.latest.md` as an artifact map instead of a
  deterministic recommendation/ranking engine. The engine now points to
  measured lanes while Claude/LLM reads raw artifacts and authors the
  user-facing summary from context.
- Updated routing, README, SKILL guidance, console preview tests, and
  audit tests so the short chat answer no longer inherits summary
  ordering or engine-generated coding prompts.
- Slimmed the deployable skill package by excluding maintainer
  self-audit canonical fact snapshots (`type-ownership.md`,
  `helper-registry.md`, `topology.md`, `naming.md`). The shipped
  package now keeps only the runtime canon spine under `canonical/`.

## 1.10.40 — 2026-04-28

### Full-review reminder pack

- Added `audit-review-pack.latest.md` for full/CI profiles so richer
  artifact lanes can be injected into Claude Code's working context before
  a deep review, instead of relying on the short summary alone.
- Kept the pack strictly local: the audit engine writes Markdown only and
  never calls external models or APIs. Claude Code decides inside the same
  session whether to read lanes locally or paste whole lanes into built-in
  reviewer subagents.
- Added product-surface tests so the no-external-API contract and
  lane-pasting reminder cannot silently drift.

## 1.10.39 — 2026-04-28

### Richer audit summary bridge

- Widened `audit-summary.latest.md` beyond checklist/fix-plan gates so
  runtime cycles from `topology.json`, type-check escape clusters from
  `discipline.json`, and semi-dead imports from `call-graph.json` can
  reach the first-read user surface.
- Added priority scoring for summary candidates so cross-file/high-impact
  evidence can outrank lower-value watch signals such as a single catch
  handling cue.
- Grouped multiple review-visible exports as one surface question before
  falling back to one isolated export candidate, keeping cleanup prompts
  focused on owner/public-surface screening rather than raw counts.

## 1.10.38 — 2026-04-28

### Lifecycle command results

- Added a first-read `Command Result` section to `audit-summary.latest.md`
  so pre-write, post-write, canon-draft, and check-canon runs report what
  happened before asking users to inspect JSON artifacts.
- Updated console previews and slash-command routing guidance so
  check-canon cannot be summarized as merely "done" without naming clean,
  drift, missing-canon, or unreadable states.
- Preserved the vibe-coder default voice while keeping maintainer commands
  honest about draft promotion and post-write lane limits.

## 1.10.37 — 2026-04-28

### Vibe-coder public voice

- Recentered the skill's public surface around vibe-coding sessions:
  kind, short, action-first summaries by default, with raw JSON, FP ids,
  tier labels, and canonical jargon reserved for explicit proof/debug asks.
- Tightened slash-command routing, short review output, and refactor-plan
  policy so users get next actions and copy/paste coding-agent prompts
  before evidence trails.

## 1.10.36 — 2026-04-28

### B1/B2 near-shape cues

- Added `nearShapeCandidates[]` and `nearShapeCandidateCount` to
  `checklist-facts.json.B1B2_shape_drift` so full-profile audits can
  surface exported type shapes with strong field/name overlap even when
  their exact shape hashes differ.
- Kept near-shape matches explicitly as review cues, not semantic verdicts
  or automatic refactor instructions. The summary and checklist now point
  users to inspect the owner chain before merging or extracting shared
  types.

## 1.10.35 — 2026-04-28

### E2 catch evidence

- Strengthened E2 catch-handling evidence so `checklist-facts.json`
  explicitly records the AST-backed analysis method, empty undocumented
  catches, non-empty anonymous catches, and non-empty catches whose bound
  error parameter is never referenced.
- Updated the human audit summary and checklist guidance to surface
  `unusedParamCount` alongside empty and anonymous catch counts, reducing
  the chance that `count = 0` is misread as "no catch-handling watch sites
  exist."

## 1.10.34 — 2026-04-28

### Template surface split

- Moved the dogfood-only self-audit handbook from `templates/` to
  `docs/maintainer/`, keeping generated skill packages free of
  maintainer-only review overlays.
- Split `refactor-plan` behavior from output shape:
  `references/refactor-plan-policy.md` now owns tone, slice selection,
  lifecycle integration, and evidence discipline, while
  `templates/refactor-plan-template.md` stays focused on the rendered
  SHORT/FULL answer shape.
- Added `templates/README.md` as a local selection guide for short
  reviews, formal reports, refactor plans, and full checklist walks.
- Made `report-template.md` language-neutral by defaulting section
  headings and placeholder text to English while instructing the final
  report to translate headings into the user's language.

## 1.10.33 — 2026-04-28

### Skill entrypoint clarity

- Slimmed `SKILL.md` by moving lifecycle command examples behind
  `references/lifecycle-modes.md`, leaving the body focused on routing
  rules, claim discipline, and template selection.
- Promoted the pre-write-first rule near the core contract so write,
  refactor, edit, move, and rename requests route through pre-write
  before coding whenever a compact intent can be inferred.
- Added `references/glossary.md` and linked it from `SKILL.md` so terms
  such as FP families, Tier C, SAFE_FIX, HCA, P4, and canonical drift
  have a lightweight first-stop explanation.
- Replaced in-group "blessed" wording with recommended/canonical
  public-entrypoint language across user-facing docs and surface tests.

## 1.10.32 — 2026-04-28

### Rule 1 citation verifier

- Added `test-harness/lib/verify-citations.mjs`, a maintainer harness
  that mechanically checks saved model output for falsifiable
  `[grounded, artifact.json.path = value]` citations.
- The verifier rejects unfalsifiable grounded labels, missing artifact
  paths, placeholder values, and value mismatches; it supports bracket
  paths, `.length`, object literals, root `package.json` fallback, and
  stdin input.
- Added focused regression coverage and documented the verifier beside
  the existing triggering and refactor-plan harnesses.

## 1.10.31 — 2026-04-28

### Repo-neutral review checklist

- Moved `auditing-repo-structure` dogfood notes out of the long
  structural review checklist and into
  `templates/SELF_AUDIT_HANDBOOK.md`, so ordinary user repos no longer
  inherit maintainer-only checks for `vocab.mjs`, `ranking.mjs`, or the
  tool's fixture style.
- Marked the long checklist as repo-neutral and kept self-audit
  expectations as an explicit overlay for dogfooding only.
- Added surface and generated-package tests to ensure the shipping skill
  includes the self-audit handbook while keeping
  `templates/REVIEW_CHECKLIST.md` free of self-reference noise.

## 1.10.30 — 2026-04-28

### JSDoc any escape coverage

- Added `jsdoc-any` as the eleventh canonical `type-escape.escapeKind`
  so `.mjs` / JSDoc-typed codebases no longer under-count
  `/** @type {any} */`, `@param {any}`, or `@returns {any}` sites.
- Updated `any-inventory`, pre-write intent validation/rendering,
  post-write delta capability parity, and canonical docs to treat
  `jsdoc-any` like every other planned-vs-observed type escape.
- Added a post-write diagnostic,
  `ambiguous-duplicate-occurrence-key`, for duplicate-key silent-new
  entries whose count is reliable but exact line localization may be
  ambiguous after an insertion.
- Added focused regression coverage for JSDoc extraction, inventory
  capability metadata, canonical enum drift, pre-write planned escapes,
  and duplicate occurrence-key diagnostics.

## 1.10.29 — 2026-04-28

### Pre-write semantic search hints

- Added intent-token search hints for name lookups so morphologically
  distant helper requests such as `loadArtifactJson` can surface existing
  `loadIfExists` / `readJsonFile` style candidates as degraded search
  hints, not grounded reuse claims.
- Expanded file domain-cluster detection beyond strict basename prefixes
  to repeated basename tokens, so `artifact-loader.mjs` notices existing
  `*-artifact.mjs` and `artifacts.mjs` siblings.
- Kept both signals advisory-only and cited from existing artifacts so
  pre-write still distinguishes "look here first" from "this is already
  the same thing".
- Added focused regression coverage for suffix/token domain clusters,
  intent-token name hints, and user-facing advisory rendering.

## 1.10.28 — 2026-04-28

### B1/B2 exact shape-drift cue

- Added `checklist-facts.json.B1B2_shape_drift`, an artifact-backed
  observation cue that reads `shape-index.json` and surfaces exact
  exported type-shape clusters as `watch`, never as an automatic refactor
  verdict.
- Updated the gentle audit summary to include a humane "exact exported
  type-shape cluster" smoothing candidate when full-profile shape evidence
  is present.
- Kept broader duplicate implementation and domain-shape drift in
  `_not_computed` so LLM/human judgment remains explicit beyond exact
  shape-hash evidence.
- Added regression coverage for missing-shape-index unknown behavior,
  exact duplicate type-shape grouping, and full-profile summary surfacing.

## 1.10.27 — 2026-04-28

### Declaration-surface dead-export guard

- Tightened dead-export declaration-surface evidence for exported classes
  and exported variable type annotations. Types used only through public
  class fields, method signatures, or exported const annotations now carry
  `declarationExportDependency` and rank as `DEGRADED` instead of
  review-visible cleanup.
- Kept implementation bodies out of that public-declaration count so local
  helper annotations inside method bodies and initializers do not inflate
  public surface evidence.
- Added regression coverage in `tests/test-classify-facts-ast.mjs` plus a
  precision-corpus fixture proving class/const signature dependencies stay
  out of `REVIEW_FIX` while unrelated private exports remain review-visible.

## 1.10.26 — 2026-04-28

### A2 file-role context

- Added production/test/script role evidence to A2 function-size facts so
  oversized smoke harnesses and tests do not look identical to production
  refactor targets.
- Updated the gentle audit summary to prefer production A2 candidates and
  keep test/script large functions as context unless they are the requested
  slice.
- Updated the review checklist citation guidance to cite `roleBuckets`
  and screen A2 findings by role before recommending changes.
- Pinned the role split with checklist-facts and audit-summary regression
  tests.

## 1.10.25 — 2026-04-28

### Anonymous catch evidence

- Added E2 tracking for parameterless non-empty `catch { ... }` blocks,
  so `count = 0` no longer implies every catch site preserved error
  identity.
- Kept empty silent catches, documented empty catches, and non-empty
  anonymous catches as separate fields in `checklist-facts.json`.
- Updated the audit summary and review checklist citation text so
  anonymous catch evidence reaches the user-facing surface.
- Pinned the behavior with focused checklist-facts regression tests.

## 1.10.24 — 2026-04-28

### Last-mile coding prompt

- Added a short "Ask the coding agent" handoff to generated audit
  summaries, so each smoothing candidate can be turned into a narrow
  implementation request without translating raw artifacts by hand.
- Updated the short review and refactor-plan templates to require the
  same copy/paste prompt for code-changing next slices.
- Kept the prompt scoped to one slice, pre-write first when useful,
  unrelated cleanup out, and explicit verification.
- Pinned the contract in audit, skill-surface, and generated-package
  tests.

## 1.10.23 — 2026-04-28

### Natural-language pre-write entry

- Updated the pre-write slash-command surface so ordinary chat users can
  describe the intended change in natural language instead of writing
  intent JSON.
- Taught command routing to infer a compact intent and stream it through
  `--intent -` when no explicit `--intent` is provided.
- Clarified in README that explicit intent JSON is for automation,
  debugging, and reproducible tests.
- Pinned the user-facing contract in the skill surface test.

## 1.10.22 — 2026-04-28

### Compact pre-write intent defaults

- Allowed compact pre-write intents by defaulting missing top-level
  arrays to `[]` with `intentWarnings`, while preserving hard schema
  errors for malformed present fields.
- Rendered intent schema notes in pre-write Markdown and JSON so the
  assistant can see which lanes were omitted by the caller.
- Clarified that human users should not hand-author intent JSON in
  ordinary chat; assistants should infer or stream compact intents from
  the user's request.
- Pinned the behavior in intent, render, and CLI smoke tests.

## 1.10.21 — 2026-04-28

### Dependency pre-write consumer evidence

- Added `symbols.json.dependencyImportConsumers` plus
  `meta.supports.dependencyImportConsumers`, so fresh dependency
  pre-write can ground observed static package-import consumer counts
  from the artifact shape the producer actually emits.
- Updated dependency lookup to prefer that consumer stream while keeping
  the older `symbols.uses[]` fixture shape as a legacy fallback.
- Narrowed `intent.dependencies` documentation to package dependency
  specifiers; internal modules and API surfaces now route through
  `files` or `names`.
- Pinned the shipped path with lookup, render, CLI, integration, and
  orchestrator tests.

## 1.10.20 — 2026-04-28

### First-run console summary preview

- Kept `audit-summary.latest.md` as the durable first-read artifact, and
  added a short console preview after audit runs so users immediately
  see the stable areas, the first smoothing candidate, and the main
  keep-as-is caveat without opening raw JSON.
- Fixed post-write type-escape delta comparison to treat `occurrenceKey`
  as a multiset bucket, so duplicate `any` shapes inside the same
  exported identity can still produce `silent-new` acknowledgements.
- Pinned the console preview in `tests/test-audit-repo.mjs`.
- Pinned duplicate-key `any` delta behavior in
  `tests/test-post-write-delta.mjs`.

## 1.10.19 — 2026-04-28

### Repo-relative excludes and dependency import-count honesty

- Changed `collectFiles()` exclude matching to use repo-relative paths, so
  `--exclude vendor` no longer prunes an entire repo just because its
  absolute parent path contains `/vendor/`.
- Kept the `1.10.18` behavior for explicit file excludes such as
  `--exclude src/a.ts` and `--exclude skip-me.js`.
- Made dependency pre-write lookups report `import graph unavailable`
  when `symbols.json` is absent instead of rendering a grounded
  `0 observed consumers` count.
- Pinned both behaviors in collect-files, dependency lookup, render, and
  pre-write CLI tests.

## 1.10.18 — 2026-04-28

### Exclude pattern contract alignment

- Kept bare `--exclude build` as a directory-segment prune so
  `build-index.ts` stays in scope.
- Added explicit file-path and basename excludes such as
  `--exclude src/a.ts` and `--exclude skip-me.js`.
- Updated help/reference text to describe directory-segment and
  file-path behavior precisely.
- Pinned the behavior in `tests/test-collect.mjs`.

## 1.10.17 — 2026-04-28

### Wildcard public-surface and alias resolver precision

- Protected `package.exports` wildcard targets that resolve to authored
  JS sources instead of only TS/TSX source guesses.
- Resolved `package.imports` wildcard targets through the same candidate
  pattern set, preserving authored JS files and TS fallbacks.
- Stopped treating matched alias directories as concrete files; directory
  targets now resolve through `index.*` probing or surface
  `UNRESOLVED_INTERNAL`.
- Pinned the behavior with public-surface and hash-import resolver tests.

## 1.10.16 — 2026-04-28

### Pre-write stdin contract alignment

- Preserved `--intent -` through the blessed `audit-repo.mjs --pre-write`
  path instead of resolving it to a literal `-` file path.
- Propagated dispatched `pre-write.mjs` failures to the orchestrator exit
  code while still writing `manifest.preWrite.ran=false` and the failure
  reason.
- Documented stdin intent usage and pinned the behavior with orchestrator
  tests for both successful and malformed stdin intents.

## 1.10.15 — 2026-04-28

### Collect-files walker helper split

- Split `collectFiles` into focused helpers for option normalization,
  root search-dir selection, root entry collection, recursive walking,
  exclude matching, and sorted de-dupe.
- Preserved language filters, test exclusion, root entrypoint detection,
  and directory-segment `--exclude` behavior while removing `collectFiles`
  from the self-audit A2 function-size watch list.

## 1.10.14 — 2026-04-28

### AST reference counter helper split

- Moved the parser-backed `countFileReferencesAst` implementation into
  `_lib/classify-facts-ast-counter.mjs` while preserving the public
  `classify-facts.mjs` export.
- Split the AST walker into smaller scope, skip-position, child traversal,
  and result-building helpers, removing both `classify-facts` watch entries
  from the self-audit A2 function-size list.

## 1.10.13 — 2026-04-28

### Check-canon lifecycle helper refactor

- Split the `runCheckCanonLifecycle` internals into smaller helpers for
  child process execution, all-source fallback selection, summary assembly,
  and strict exit-code calculation.
- Preserved the manifest `checkCanon` schema and advisory/strict behavior
  while removing the lifecycle function from the self-audit watch list.

## 1.10.12 — 2026-04-28

### Symbol graph artifact helper split

- Extracted `symbols.json` artifact assembly from `build-symbol-graph.mjs`
  into `_lib/symbol-graph-artifact.mjs`.
- Preserved the emitted schema, warning enrichment, identity fan-in map,
  re-export records, unresolved specifier hints, and dynamic import opacity
  records while keeping the producer focused on scanning and graph building.

## 1.10.11 — 2026-04-27

### Check-canon source parser split

- Extracted topology and naming canon Markdown parsers from
  `_lib/check-canon-utils.mjs` into source-specific parser modules.
- Preserved the existing `check-canon-utils` public exports as re-exports,
  so downstream imports keep working while the shared utility module becomes
  smaller and more focused on type/helper parsers plus drift JSON helpers.

## 1.10.10 — 2026-04-27

### Check-canon lifecycle helper split

- Extracted the `audit-repo.mjs --check-canon` source validation, child
  process orchestration, per-source manifest aggregation, and strict-mode
  exit calculation into `_lib/audit-check-canon.mjs`.
- Preserved advisory default behavior, `--strict-check-canon` escalation,
  `--sources all` expansion/dedupe, single-child all-source execution when
  primary artifacts are ready, and per-source fallback behavior.

## 1.10.9 — 2026-04-27

### Canon-draft lifecycle helper split

- Extracted the `audit-repo.mjs --canon-draft` source validation and child
  process orchestration into `_lib/audit-canon-draft.mjs`. The public
  orchestrator now delegates the canon-draft lifecycle the same way it
  delegates manifest evidence, keeping the entrypoint easier to read.
- Preserved per-source manifest shape, `--sources all` expansion/dedupe,
  versioned draft path capture, shell-safe spawning, and advisory exit-code
  behavior.

## 1.10.8 — 2026-04-27

### Audit manifest helper split

- Extracted audit manifest evidence collection and produced-artifact
  enumeration from `audit-repo.mjs` into `_lib/audit-manifest.mjs`.
  The public orchestrator now keeps more of its surface focused on lifecycle
  dispatch and child producer ordering.
- Preserved scan-range, blind-zone, confidence, and dynamic artifact listing
  behavior while making those helpers reusable for future summary/report
  work.

## 1.10.7 — 2026-04-27

### Check-canon parser split

- Extracted the shared Markdown table parser from
  `_lib/check-canon-utils.mjs` into `_lib/check-canon-markdown.mjs`.
  The check-canon source parsers now keep source-specific canon logic while
  the GFM table splitting, strict header diagnostics, section slicing, and
  numeric cell parsing live in one reusable helper module.
- Dogfood follow-up: `_lib/check-canon-utils.mjs` dropped from roughly 948
  LOC to roughly 767 LOC without changing drift behavior.

## 1.10.6 — 2026-04-27

### Self-audit scope cleanup

- `audit-repo.mjs` now detects the grounded-audit maintainer checkout and
  automatically excludes lab/corpus/generated mirror directories during
  self-audit runs. This keeps dogfood summaries focused on the tool code
  instead of surfacing `p6-corpus/`, `output/corpus/`, or generated
  `skills/auditing-repo-structure/_engine/` files as top findings.
- `manifest.scanRange` now records both the effective `excludes` and the
  `autoExcludes` applied for maintainer self-audits. Use
  `--no-self-audit-excludes` when an intentional whole-repo scan is needed.
- Added regression coverage for the auto-exclude detector and for the
  orchestrator path that forwards those exclusions into producer steps.

## 1.10.5 — 2026-04-27

### Human-readable audit summary

- Audit profiles now emit `audit-summary.latest.md` beside the JSON
  artifacts. The summary gives a short first-read surface with scan
  range, confidence, already-stable signals, at most three smoothing
  candidates, keep-as-is caveats, and the next pass.
- `audit-repo.mjs` prints the summary path after a run and includes the
  Markdown file in `manifest.artifactsProduced`. The raw JSON artifacts
  remain authoritative; the summary is a readable guide, not a second
  analysis engine.
- README, SKILL.md, and command routing now point chat-facing users to
  the summary first while preserving the requirement to verify concrete
  claims against `manifest.json`, `checklist-facts.json`,
  `fix-plan.json`, and other source artifacts.
- Clarified that the skill-triggering evaluation harness is
  maintainer-only and intentionally absent from skill-only deployment
  archives.

## 1.10.4 — 2026-04-27

### Intent-shaped pre-write path

- `audit-repo.mjs --pre-write --intent ...` now skips the base
  quick-audit producer chain when pre-write is the only requested
  lifecycle mode. The blessed public path now matches
  `canonical/pre-write-gate.md`: pre-write uses the intent-shaped
  cold-cache path instead of running triage, topology, discipline,
  classify, rank, and checklist producers first.
- Cold-cache preflight now selects producers from the validated intent:
  names/dependencies need `symbols.json`, files need
  `symbols.json` + `topology.json` + `triage.json`, and shapes need
  `shape-index.json`. Shape-only intents no longer create unrelated
  graph artifacts.
- `audit-repo.mjs` refreshes manifest scan-range, confidence,
  blind-zone, and artifact evidence after lifecycle blocks run, so
  pre-write-only manifests honestly reflect the artifacts created by the
  pre-write child.
- Regression pins:
  `tests/test-pre-write-cli.mjs` and
  `tests/test-audit-repo-pre-write.mjs` now assert that names-only
  pre-write creates `symbols.json` but not `triage.json`,
  `topology.json`, or full-audit fix-plan artifacts.

## 1.10.3 — 2026-04-27

### Public surface and resolver precision

- Expanded `package.exports` wildcard public-surface protection so
  subpath patterns such as `./features/*` protect matching authored
  source files instead of leaving them review-visible as dead exports.
- Stopped treating Node `package.imports` (`#internal`) exact aliases as
  external public API surface. They still help resolve internal
  consumers, but no longer mute findings as `publicApi_FP23`.
- Resolver wildcard aliases now return `UNRESOLVED_INTERNAL` when the
  alias pattern matches but no target file can be found, preserving the
  blind-spot signal instead of falling through as external.
- Barrel detection now normalizes root `exports` string and conditional
  forms, not only `exports["."]`, so valid package entry barrels stay
  out of dead-export candidates.

## 1.10.2 — 2026-04-27

### Softer chat-facing audit language

- Updated the short structural review template to lead with
  `Gentle Structural Review`, `Already Stable`, `Worth Smoothing Next`,
  and `Keep As-Is For Now` instead of a colder findings-first report.
- Default command routing now asks the model to use those section names
  for one-click audits, keep raw evidence behind the scenes unless
  requested, and avoid "findings" language in the chat-facing summary.
- Bumped the plugin/package version so Claude Code can refresh from the
  `1.10.1` cache to the softer default-output surface.

## 1.10.1 — 2026-04-27

### Plugin cache refresh and first-touch polish

- Bumped the skill/plugin/package version from `1.10.0` to `1.10.1`
  so Claude Code plugin caches can install the current thin-delegator
  command surface instead of reusing stale `1.10.0` content.
- The default `/auditing-repo-structure` command now explicitly treats
  an empty invocation as "check this repo now": it should run a quick
  current-workspace audit, avoid a mode menu, keep chat output short,
  and give a setup command plus one next action if the scan cannot run.
- Added maintainer validation for refactor-plan chat output so SHORT
  plans preserve readable sections, evidence anchors, tone guards, and
  pre-write handoff for code-changing slices.

## 1.10.0 — 2026-04-26

### Skill packaging surface

- Generated deployable package under `skills/auditing-repo-structure/`
  now exposes a smaller skill surface: `scripts/`, `_engine/`,
  `canonical/`, `templates/`, and `references/`.
- Public documentation now points generated-package users to
  `scripts/audit-repo.mjs` / `grounded-audit`; maintainer-only root
  scripts remain documented as internal engine entrypoints.
- Default audit profile is now `quick`. `full` remains available for
  deeper audits, comparison baselines, CI evidence passes, and explicit
  user requests.
- The direct platform-specific `@oxc-parser/binding-linux-x64-gnu`
  dependency was removed from the top-level dependency list; portability
  is delegated to `oxc-parser`'s optional bindings.
- External skill review on 2026-04-27 flagged a pre-write intent
  doc-vs-engine mismatch: the new
  `references/pre-write-intent-shape.md` showed structured
  `{ name, why }` / `{ specifier, why }` entries while the engine still
  accepted only strings. `_lib/pre-write-intent.mjs` now accepts both
  terse and structured entries, normalizes lookup arrays to strings, and
  preserves self-declaration metadata in advisory JSON. Regression pins:
  `tests/test-pre-write-intent.mjs` and `tests/test-pre-write-cli.mjs`.
- External skill review on 2026-04-27 flagged a generated-package Go
  regression: `_engine/lib/tree-sitter-langs.mjs` looked for WASM deps
  under `_engine/node_modules` instead of the package root. The
  tree-sitter resolver now uses `createRequire(import.meta.url)`, and
  `tests/test-skill-package.mjs` pins both generated-package availability
  and Go symbol extraction. Generated package fact-canon headers are also
  promoted from "draft" wording to packaged canonical truth, and
  `CHANGELOG.md` is excluded from the generated skill surface.
- External skill review on 2026-04-27 flagged a first-touch plugin UX
  gap: invoking the default command asked the user to pick a mode instead
  of doing the obvious quick audit. The plugin now ships
  `commands/auditing-repo-structure.md`, which defaults to a quick
  current-workspace pass and applies focused maintainer self-audit
  exclusions when appropriate.
- The shared CLI parser now uses the same default artifact directory as
  the public orchestrator (`<root>/.audit`). Tests also pin
  `audit-repo.mjs --pre-write --production` so pre-write scan-scope
  forwarding cannot regress silently.
- `audit-repo.mjs` now consumes the shared scan-scope normalization
  helper instead of carrying a separate boolean interpretation path.
- `refactor-plan` now defaults to a short four-section coaching plan.
  The formal evidence trail and machine-readable scope JSON are opt-in
  so the command stays readable in chat while preserving raw validation
  modes for users who need cold counts.
- External skill review on 2026-04-27 flagged remaining deployment
  polish: the generated package allowed Node versions below
  `oxc-parser`'s own engine floor, shipped a lockfile carrying
  maintainer-only ESLint packages, and did not explicitly tell users
  when Python/Go extractors were unavailable. `engines.node` now matches
  `^20.19.0 || >=22.12.0`, `scripts/build-skill.mjs` prunes generated
  `package-lock.json` to runtime-reachable packages, and
  `symbols.json.meta.languageSupport` feeds `python-scan-gap` /
  `go-scan-gap` blind zones when files are counted but extractor
  support is missing. README/SKILL wording now frames the surface as
  vibe-coder friendly while keeping the senior-grade evidence ledger
  behind the scenes.
- External UX review on 2026-04-27 compared the entry surface with
  Superpowers and Ouroboros. Slash commands are now thin delegators:
  each command loads the packaged `SKILL.md` plus
  `references/command-routing.md`, and the routing reference owns the
  default audit, validation modes, and `refactor-plan` coaching behavior.
  This keeps command files small while preserving the single
  `scripts/audit-repo.mjs` execution path.
- External dogfood on 2026-04-27 found that FP-18 was documented but not
  always operationalized: non-literal lazy loaders such as
  ``import(`./commands/${name}.js`)`` could leave command-module exports
  in `REVIEW_FIX`. `build-symbol-graph.mjs` now emits
  `dynamicImportOpacity` directory hints for template imports with static
  prefixes, and `classify-dead-exports.mjs` materializes matching exports
  as `MUTED` with `dynamicImportOpacity_FP18` evidence. The chat routing
  reference and short checklist now require a quick FP screen before
  turning review-visible dead-export entries into user-facing findings.

### FP-41 fix (JSX identifier blindness)

### Summary

`_lib/classify-facts.mjs::countFileReferencesAst` in v1.10.0 P0 matched
`node.type === 'Identifier'` only. `JSXIdentifier` nodes fell through
the condition uncounted. Any same-file JSX usage of an exported symbol
was invisible to the file-internal reference counter. Compound-component
patterns (shadcn/ui style: `AlertDialog` + `AlertDialogTrigger`,
`CodeBlock` + `CodeBlockContainer`) over-escalated from Tier A
("export 제거 가능, file-internal use only") to Tier C ("completely
dead") on any TSX/JSX repo.

External reviewer sampled 70 Tier A → Tier C migrations on duyet
between v1.9.11 and the v1.10.x in-progress state; 35 of 70 (50%) were
this over-escalation. v1.9.11's regex counter had incidentally caught
JSX usage because `<Foo` hits the `\b...\b` word-boundary match. The
AST rewrite fixed four FP classes (comments, string literals, property
keys, export-specifier self-references) but lost the accidental JSX
coverage.

### Fix

**`_lib/classify-facts.mjs::countFileReferencesAst`:**

- Walker now accepts both `Identifier` and `JSXIdentifier` when matching
  the symbol name. JSX identifier references always increment
  `valueRefs` (JSX compiles to `React.createElement(Foo, ...)`; type
  position doesn't apply).
- `JSXClosingElement` subtrees are skipped entirely. `<Foo>...</Foo>`
  has `Foo` in both opening and closing tags — same semantic reference,
  one count.

**`_lib/classify-facts.mjs::isSkipPosition`** — three new JSX skip
rules parallel the existing non-JSX ones:

- `JSXAttribute.name` — attribute prop name, not a JS binding reference.
  Parallels `Property.key` on non-computed access.
- `JSXMemberExpression.property` — sub-component name in `<Foo.Bar />`.
  Parallels `MemberExpression.property` on non-computed access.
- `JSXNamespacedName.namespace` — XML-style prefix slot.

### Tests

- `tests/test-classify-facts-ast.mjs` — 17 JSX cases added (T24–T36b)
  on `.tsx` fixtures. Covers single/nested/self-closing elements,
  JSXMemberExpression head vs property slot, attribute name vs attribute
  value expression, JSX text content, fragments, conditional JSX, self-
  render, spread attributes, and the AlertDialog / AlertDialogTrigger
  reproduction. Total: 40 assertions, all pass.
- `tests/test-corpus.mjs::CASE-FP41` — release-blocking corpus case.
  Two-file TSX fixture (live `AlertDialog` + internal-use
  `AlertDialogTrigger`). Asserts Trigger lands in Tier A, not Tier C,
  with `fileInternalUses === 1` and `fileInternalRefs.valueRefs === 1`.
  `FP_BUDGET = 0` gate unchanged.

### Evidence label

`ast-ident-ref-count` label preserved. Description stays accurate —
the counter still counts identifier references via AST; JSX
identifiers ARE identifier references in the source-form sense.

### Scope

This entry originally closed only the JSX gap. A follow-up landed the
common lexical shadowing pass in `_lib/classify-facts.mjs` on
2026-04-20: inner `const` / function parameters / arrow parameters /
block `let`-`const` / catch parameters / for-loop bindings /
destructured parameters are covered by `tests/test-classify-facts-ast.mjs`
T37-T46.

The remaining boundary is narrower: the counter is still not a
TypeScript checker-grade symbol binder. `var` hoisting across blocks is
approximated, and public API protection remains partly file-level rather
than symbol-level.

A later P6-3 follow-up narrowed the namespace / dynamic import boundary:
direct member calls now protect only the concrete exported member
(`ns.foo()`, `const mod = await import('./m'); mod.foo()`, and
`import('./m').then((m) => m.foo())`). Computed members, aliased members
such as `const f = ns.foo; f()`, and non-literal dynamic import paths
still degrade conservatively.

### Files changed

- `_lib/classify-facts.mjs` (walker + skip rules + file-header note)
- `_lib/extract-ts.mjs` (P6-3 namespace / dynamic direct-member uses)
- `classify-dead-exports.mjs` (comment at line 193 — counter-completeness
  note referencing FP-41)
- `tests/test-classify-facts-ast.mjs` (JSX test block)
- `tests/test-corpus.mjs` (CASE-FP41)
- `tests/test-p6-member-precision.mjs` (direct member precision and
  conservative degraded cases)
- `docs/history/FP-41-regression.md` (root-cause report, referenced from the test
  headers and from this changelog entry)
- `CHANGELOG.md` (this entry)

## 1.9.11 — 2026-04-19

**FP-38: workspace packages without `exports` field.** User's
empirical v1.9.10 confirmation on duyet/monorepo brought the FP rate
from 73.2% down to 10.9%. The report identified that the remaining
13 of 229 Tier C findings were all workspace imports of packages
that use `main` + direct subpath resolution instead of the modern
`exports` map — a common pattern in Bun, older pnpm, and Turborepo
monorepos.

### The bug, from the v1.9.10 empirical report

```
packages/libs/getPost.ts → getPostBySlug, getAllCategories, ...
  (consumed from apps/blog via `@duyet/libs/getPost`)
packages/components/Menu.tsx → HOME, ABOUT, INSIGHTS, PHOTOS, BLOG
  (consumed from apps/cv via `@duyet/components/Menu`)
apps/insights/components/tabs.tsx → Tabs
  (cross-app from apps/agents)
```

All 13 remaining FPs shared one shape: workspace package's
`package.json` has `main` (or nothing) but no `exports` field.
Before v1.9.11, `buildAliasMap` only registered entries from
`exports`. Packages without `exports` contributed NOTHING to the
alias map, so every `@scope/pkg/subpath` import fell through to
`EXTERNAL`, hiding the real consumer relationship.

### Fix

Two coupled changes:

**1. `_lib/alias-map.mjs` (v1.9.11 FP-38)** — when a workspace
package has no `exports` entry covering subpaths, register two
fallback entries:

- Bare-name entry: `@scope/pkg` → resolved `main` (if present)
- Wildcard entry: `@scope/pkg/*` → `<pkgDir>/*` with a
  `legacySubpath: true` marker

Both additions respect `exports` priority — if `exports` already
covers a subpath, the explicit entry wins. Packages that deliberately
restrict subpath access via `exports` are unaffected.

**2. `_lib/resolver-core.mjs`** — the wildcard matcher previously
swapped `.mjs`/`.cjs`/`.js` extensions for `.ts`/`.tsx`, but it did
NOT probe source extensions on extensionless literals. For
`@scope/libs/getPost` → wildcard substitute → `<pkgDir>/getPost`
(no extension), the matcher would fail to find `getPost.ts`.

Added an extensionless probe step: if the substituted literal has
no extension, probe each of `.ts, .tsx, .mts, .cts, .mjs, .cjs,
.js, .jsx` in order. This also improves other wildcard scenarios
where the target pattern is extensionless (legacy source-direct
workspaces).

### Self-dogfood fixture

Before (v1.9.10):

```
deadInProd: 4
uses: resolvedInternal=0, external=2, unresolvedInternal=0
deadProdList: [Page, getPostBySlug, getAllCategories, getPostsByCategory]
```

After (v1.9.11):

```
deadInProd: 2
uses: resolvedInternal=2, external=0, unresolvedInternal=0
deadProdList: [Page, getPostsByCategory]
```

`getPostBySlug` and `getAllCategories` correctly recognized as live
(they ARE consumed by apps/blog/app/page.tsx). `getPostsByCategory`
correctly remains dead (not imported in the fixture).

### Defensive tests (F1–F8)

New suite `tests/test-workspace-no-exports.mjs`:

- **F1** main-only workspace — imported symbol NOT classified dead
- **F2** second named import from same line also live
- **F3** deep subpath resolution (different .ts file in same package)
- **F4** `resolvedInternal` count reflects the new resolutions
- **F5** `external` count is 0 — workspace specs no longer leak
- **F6** truly unconsumed exports STILL flagged dead — the fix is
  additive, not blanket-live. This is the critical guardrail: it
  would be a trivial overreach to just mark every `@scope/pkg/*`
  import as resolved, making the skill miss real dead code.
- **F7** explicit `exports` still wins — regression against the
  wildcard masking a narrower exports entry
- **F8** legacy wildcards are per-package additive — adding one
  for pkg A doesn't prevent one for pkg B

### Expected impact on duyet/monorepo

Based on the user's v1.9.10 empirical report, the remaining 13 FPs
all fall into the FP-38 class. If the analysis is correct, v1.9.11
should recover most or all of them, bringing duyet's Tier C strict
FP rate from 10.9% toward something lower still.

**Claim discipline**: per v1.9.8 Evidence Honesty, this release does
NOT claim "FP rate drops to X%" until user confirms via re-run on
duyet. The CHANGELOG states only what is empirically verified in
the skill's own fixture (F1–F8). The duyet claim follows after the
user runs v1.9.11 and reports.

### Files changed

- `_lib/alias-map.mjs` — legacy-subpath registration (31 LOC added)
- `_lib/resolver-core.mjs` — extensionless wildcard probe (7 LOC added)
- `tests/test-workspace-no-exports.mjs` — new, 8 assertions

### Status

| Release | Asserts | Suites | Capability |
|---|---|---|---|
| 1.9.9 | 234 | 19 | Orchestrator + blindZones |
| 1.9.10 | 247 | 20 | TS compiler API (true AST) for tsconfig |
| **1.9.11** | **255** | **21** | **Workspace packages without `exports` (FP-38)** |

### Three-release arc, closed

v1.9.7 claimed FP-36 fixed (tests green, not working on real repo).
v1.9.10 empirically fixed it (TS compiler API; tests green, WORKING
on real repo). v1.9.11 addresses the remaining 10.9% the empirical
run surfaced — which is exactly the pattern this project is
designed around:

1. Skill emits evidence (Tier C findings)
2. User's LLM review identifies a class of false positives
3. A focused patch with regression tests closes that class
4. The next empirical run surfaces the next class, if any

The alternative — shipping v1.9.11 with a bigger but speculative
set of changes — would break the feedback loop. Narrow, testable
iterations are what let v1.9.10 confirm and v1.9.11 build on that
confirmation.

## 1.9.10 — 2026-04-19

**True AST Config Pass.** User found that v1.9.7-v1.9.9 produced
byte-identical results to v1.9.3 on duyet/monorepo despite three
releases claiming FP-36 was fixed. Investigation revealed the
residual was in our hand-rolled `extends` resolution, not the JSONC
parser as the user's hypothesis suggested — but the honest answer
to "is full AST transition the right call?" is yes, and further
than jsonc-parser. This release replaces the entire tsconfig
loading path with TypeScript's own compiler API.

### The investigation

User's hypothesis: the regex JSONC parser was stripping `//` out of
`"$schema": "https://..."` URLs, corrupting the JSON, causing silent
parse failure on 11 of 12 app tsconfigs in duyet/monorepo.

What I found when I reproduced it: the `^\s*//` regex is
line-anchored, so it does NOT match URLs mid-string. Both the regex
parser and jsonc-parser successfully parse the realistic tsconfig
with `$schema` URL. So the specific hypothesis isn't reproduced.

But testing a different configuration — `apps/*/tsconfig.json` that
relies on `extends` to inherit `paths`, with Bun/pnpm-style
root-hoisted `node_modules` — revealed the real residual. Our
hand-rolled `loadTsconfigMerged` looked for
`<configDir>/node_modules/<extended>`. In a hoisted-install workspace,
that file doesn't exist per-app — only at the repo root. So extends
silently failed, inherited paths were dropped, and `@/*` imports
fell through to `EXTERNAL`.

This is one concrete failure mode. Whether it matches what the user
observed on duyet specifically I could not confirm — my fixture
doesn't match the duyet config shape the user shared (which had
local paths + local baseUrl, and under test on my fixture of that
shape BOTH parsers work). What's clear: there WAS unresolved FP-36
residue in the hand-rolled extends resolver; the shape that
exercises it is common in real monorepos; and a better parser alone
wouldn't have fixed it.

### The answer: TypeScript compiler API

The right move isn't a better hand-rolled extends resolver. It's
`ts.parseJsonConfigFileContent` — the exact function `tsc` uses.
This does:

- JSONC tokenization (handling `$schema` URLs, comments,
  trailing commas, all the edge cases that tripped the regex parser)
- `extends` chain resolution (walks up from the config's directory
  looking for `node_modules`, finding hoisted installs correctly)
- `baseUrl` resolution relative to the config that DECLARED it
  (which may be an ancestor in the extends chain, not the leaf)
- `paths` replacement semantics (extending config's paths REPLACE
  extended config's — they do not merge; this is tsc's actual rule)
- All the defaulting logic of the TypeScript compiler

By construction we cannot drift from tsc. If tsc resolves a path, so
do we. If tsc doesn't, neither do we.

### Zero new dependencies

`typescript@^5.7.0` was already in the skill's dependencies for other
reasons (type-only re-export detection, workspace symbol tracking).
The new `_lib/tsconfig-paths.mjs` imports `ts` directly. Net
dependency change: zero.

### Files changed

- `_lib/tsconfig-paths.mjs` — full rewrite. `parseTsconfigJson` and
  `loadTsconfigMerged` deleted. New `loadTsconfig` calls
  `ts.readConfigFile` + `ts.parseJsonConfigFileContent`. 224 LOC
  shrunk to 172.
- `tests/test-tsconfig-paths-scoped.mjs` — added T9-T12 covering
  extends-inherited paths + hoisted node_modules + tsc replacement
  semantics.

### Regression safety

All of T1-T8 still pass unchanged. The shape of `discoverScopedTsconfigPaths`
return values is byte-identical. Callers in `_lib/alias-map.mjs` and
`_lib/resolver-core.mjs` need no changes.

### New assertions (T9-T12)

- **T9** app tsconfig extending a hoisted-node_modules config
  produces scoped entries (pre-v1.9.10 silently dropped this)
- **T10** local `@/*` alias preserved when extends resolution runs
- **T11** tsc semantics: extending config's paths REPLACE extended
  config's (verifying we don't accidentally merge, which would
  leak aliases into scopes that shouldn't have them)
- **T12** extends-only config (no local paths) inherits paths from
  hoisted shared config — this is the silent-drop failure mode on
  duyet-like workspaces. The critical empirical guarantee.

### Honest limitations

I could not run this release against duyet/monorepo directly from
the sandbox. The user's observation that "v1.9.9 ≡ v1.9.3 byte-
identical" on duyet is a real observation that my fixtures don't
fully explain — my reproduction of duyet-like-structure works on
v1.9.9. What this release DOES prove empirically:

- The TS compiler API correctly handles extends + hoisted node_modules
  (T9-T12)
- Existing behavior is preserved on all previous fixtures (T1-T8)
- Zero regressions across the full 247-assertion / 20-suite CI

What it does NOT prove:

- That duyet/monorepo specifically moves from 73.2% FP rate to a
  lower number. That claim requires re-running the skill on duyet
  and comparing `fix-plan.json` against v1.9.3 baseline. Pending
  user confirmation.

Following v1.9.8 "Evidence Honesty Patch" principles, the release
note will NOT claim empirical duyet improvement until a
post-release re-run confirms it.

### Status

| Release | Assertions | Suites | Capability |
|---|---|---|---|
| 1.9.8 | 218 | 18 | Evidence honesty |
| 1.9.9 | 234 | 19 | Orchestrator + blindZones |
| **1.9.10** | **247** | **20** | **TS compiler API for tsconfig (true AST)** |

### User's question, answered in code

> 문자열로 판단하는거말고 ast로 완전 전환이 맞을까요?

Yes. But the correct "AST" for tsconfig is the TypeScript compiler
API itself, not a JSONC tokenizer. A tokenizer only fixes syntax
parsing. The resolution semantics — extends chains, baseUrl
inheritance, paths replacement, hoisted module lookup — are the
hard part, and tsc has the canonical implementation. We use it.

### Lesson

The reviewer pattern throughout 1.9.5 → 1.9.10 keeps teaching the
same thing: **a fix is not a fix until the target repo confirms it.**
v1.9.7 claimed FP-36 fixed with green tests; the shipped behavior
was incomplete on real workspaces. v1.9.8 addressed documentation
honesty. v1.9.10 addresses the resolution logic itself using the
canonical upstream implementation. Each iteration narrowed what
the skill trusts about itself. The next iteration should be
"v1.9.10 empirically verified on duyet" — not assumed.

## 1.9.9 — 2026-04-19

**Product UX Pass.** Reviewer's P1-c item (`audit-repo.mjs` one-shot
orchestrator) closed, plus the "blindZones should be in artifacts,
not just prose" recommendation that accompanied it. Two new
surfaces:

1. `audit-repo.mjs` — quick/full/ci profile orchestrator
2. `_lib/blind-zones.mjs` + `manifest.json.blindZones` — standardized
   blind-zone surfacing

### 1. `audit-repo.mjs` — one-shot orchestrator

```bash
node audit-repo.mjs --root <repo> --output <dir> [--profile quick|full|ci] [--sarif]
```

Runs the full pipeline in the order SKILL.md documents and writes
`manifest.json`. Profiles:

- **quick**: triage + topology + discipline + symbols + classify + rank
- **full**: quick + runtime (if coverage present) + staleness
  (if git working tree) + (optional SARIF). As of 1.10.0, `quick` is
  the default profile.
- **ci**: full + SARIF always

Preconditions gracefully skip rather than abort — `merge-runtime-evidence`
skipped if no `coverage-final.json`, `measure-staleness` skipped if no
`.git/`. Each step's status recorded in `manifest.commandsRun[]` with
timing. Required steps (triage, symbols) abort the run on failure;
optional steps continue with a warning.

Self-dogfood (TS/JS-only audit of the skill itself):

```
[audit-repo] profile=full  root=<skill>  output=<out>
[audit-repo] ok    triage-repo.mjs  (238ms)
[audit-repo] ok    measure-topology.mjs  (500ms)
[audit-repo] ok    measure-discipline.mjs  (213ms)
[audit-repo] ok    build-symbol-graph.mjs  (544ms)
[audit-repo] ok    classify-dead-exports.mjs  (153ms)
[audit-repo] skip  merge-runtime-evidence.mjs  (no coverage-final.json...)
[audit-repo] skip  measure-staleness.mjs  (not a git working tree)
[audit-repo] ok    rank-fixes.mjs  (181ms)
[audit-repo] wrote .../manifest.json
[audit-repo] artifacts: 6 / 9
[audit-repo] blindZones: none detected
```

Full pipeline completes in ~2s on this repo.

### 2. `_lib/blind-zones.mjs` — standardized detection

Centralizes the scattered "we know we're blind here" prose from
SKILL.md and tests/README.md into one predicate. Reads available
artifacts (triage + symbols) and returns a structured list:

```json
[
  {
    "area": "python-method-resolution",
    "severity": "precision-gap",
    "effect": "Method-level dead-code claims are degraded. __getattr__ / lazy export maps not detected.",
    "details": { "files": 244 }
  },
  {
    "area": "resolver",
    "severity": "confidence-gap",
    "effect": "Tier C dead-export claims must be reviewed... See FP-36 in references/false-positive-patterns.md.",
    "details": { "unresolvedInternalRatio": 0.22, "topUnresolvedSpecifiers": ["@/"] }
  }
]
```

Three severity levels:

- **scan-gap**: files in an unsupported language (Rust, Kotlin,
  Swift, etc.). Effect: don't make repo-wide absence claims.
- **precision-gap**: we looked but the answer is weaker (Python
  method resolution, Go interface dispatch, parse errors).
- **confidence-gap**: resolver unresolvedInternalRatio ≥ 15%.
  Effect: downgrade Tier C claims; add tsconfig path entries.

The predicate is pure — takes artifacts in, returns zones out.
`audit-repo.mjs` calls it as the last step before writing manifest.

### 3. `manifest.json` — the thing Claude reads before claiming

Every orchestrator run writes a structured summary:

```json
{
  "profile": "full",
  "commandsRun": [ { "step": "triage-repo.mjs", "status": "ok", "ms": 238 }, ... ],
  "skipped":     [ { "step": "measure-staleness.mjs", "reason": "not a git working tree" } ],
  "scanRange":   { "root": "...", "includeTests": false, "languages": [...], "files": 44 },
  "confidence":  { "parseErrors": 0, "unresolvedInternalRatio": 0, "externalImports": 202,
                   "resolvedInternal": 121, "unresolvedInternal": 0 },
  "blindZones":  [...],
  "artifactsProduced": ["triage.json", "symbols.json", ...]
}
```

This closes the longstanding gap: SKILL.md prose said "Rust not
supported, Python method resolution blind" but these notes lived
only in docs. Now they surface in every single run's output.
Claude's review layer reads `manifest.blindZones` and scopes its
absence/removal claims accordingly — the exact pattern FP-36
taught us is necessary.

Reviewer's principle, now implemented:

> blindZones는 문서가 아니라 artifact에 있어야 해.
> 실행 결과에도 이렇게 떠야 해.
> 이게 바로 "NO ABSENCE CLAIM WITHOUT STATED SCAN RANGE"를
> 실제 artifact에 녹이는 방법이야.

### 4. SKILL.md workflow updated

Workflow section now leads with the orchestrator:

```
Recommended (v1.9.9+): node audit-repo.mjs --profile full

Manual, if you need step-by-step control: [9-step list]
```

Quick Reference table gets a top row:

| **One-shot audit (recommended)** | `audit-repo.mjs` | `manifest.json` + all artifacts |

### Tests (234 total, up from 218)

New suite `tests/test-audit-repo.mjs` — 16 assertions:

- **B1–B8** (blind-zones predicate): Rust → scan-gap, Python →
  precision-gap, high resolver blindness → confidence-gap pointing
  at FP-36, low blindness → no zone, parse errors → precision-gap,
  clean repo → zero zones, summary formatter shape
- **O1–O7** (orchestrator): manifest has five required sections,
  quick profile runs right subset, skipped steps recorded,
  artifactsProduced enumerates disk state, clean TS fixture → zero
  blind zones, confidence section has FP-36 fields, console output
  directs to blindZones
- **O8** (integration): a fixture with a `.py` file in it produces
  a python-method-resolution precision-gap in the actual manifest
  (not just the predicate)

### Deferred to v1.10.0 (AST Precision Pass, per reviewer)

Still not in this release:
- **AST local reference counter** (replacing the regex path labeled
  in v1.9.8). Biggest remaining residue.
- **Symbol-level public API expansion** (replacing file-level
  `reExportsByFile`)
- **Namespace / dynamic import symbol precision**
- **Shared AST cache** across scripts (currently each re-parses)
- **Generated-file policy** as an `isGeneratedFile()` auto-MUTE
- **Package entrypoint detection** (main/bin/module/browser/types/scripts)
- **Cloudflare Pages Functions sentinel**

### Status

| Release | Assertions | Suites | Capability |
|---|---|---|---|
| 1.9.7 | 207 | 17 | Scope-aware tsconfig paths (FP-36) |
| 1.9.8 | 218 | 18 | Evidence honesty (compare, doc-ref guard) |
| **1.9.9** | **234** | **19** | **One-shot orchestrator + blindZones manifest** |

### The principle converging

Across 1.9.5 → 1.9.9 the same idea keeps becoming more concrete:

- 1.9.5: fix-plan ranking — evidence → tiers
- 1.9.6: MUTED materialized so excluded candidates are visible
- 1.9.7: FP-36 resolver scope — don't silently miss consumers
- 1.9.8: honest evidence labels — regex vs AST, compare-repos exists
- 1.9.9: blind zones in every run's manifest — Claude reads them

The skill was always designed as "evidence producer, claim consumer
separate." Until 1.9.9 the blind-spot half of that contract lived
only in prose. Now every orchestrated run ships with a machine-
readable confidence surface. Claude can quote it directly:

> "This repo contains 18 Rust files, and resolver blindness is 22%.
> I am limiting absence claims to the TS graph only, and I am
> downgrading all Tier C findings to REVIEW_FIX until the tsconfig
> paths for `@/*` are verified."

## 1.9.8 — 2026-04-19

**Evidence Honesty Patch.** Reviewer caught a doc-vs-reality drift
that's especially pointed for a tool whose core claim is "evidence
before claims": `SKILL.md:57` listed `compare-repos.mjs` as step 8 of
the Workflow, but the file didn't exist on disk. Closed that gap
plus three related "label your evidence honestly" items the reviewer
grouped with it.

### 1. `compare-repos.mjs` now exists

Thin artifact-diffing script (~170 LOC). Deliberately does NOT walk
source — reads two audit-output directories and diffs their
artifacts. Keeps the skill philosophy intact: evidence comes from
the per-repo pipeline; this just shows what changed.

```
node compare-repos.mjs --left <dirA> --right <dirB> --output <out>
```

Output `compare.json` carries per-side summaries (triage, topology,
symbols, fix-plan), delta counters (files, loc, SAFE_FIX,
REVIEW_FIX, DEGRADED, MUTED, runtimeSccs, unresolvedInternalRatio),
and a `missingArtifacts` map. Deltas for dimensions missing from
either side are `null`, not invented numbers — intentional, so the
consumer can't mistake "one side didn't measure" for "one side was
zero."

### 2. `scripts/check-doc-script-refs.mjs` — CI guard

Scans `SKILL.md`, `templates/report-template.md`, and `tests/README.md` for
`*.mjs` filename references and asserts each one exists on disk.
Wired into `npm run ci` as `check:doc-script-refs`. Excludes
`CHANGELOG.md` and `references/false-positive-patterns.md` (historical records
may reference scripts from earlier versions).

Empirically verified it catches the original drift: removing
`compare-repos.mjs` and running the guard produces:

```
SKILL.md references compare-repos.mjs but file is not present on disk
```

With concrete remediation hints (create / remove reference / rename
to avoid matching).

### 3. Regex-based occurrence counting now labeled in artifacts

`_lib/classify-facts.mjs`'s `countOccurrencesExceptDefLine` and
`countExcludingDeclAndExport` are text-level. The current
implementation excludes the declaration line but can still count
mentions inside comments or string literals. Previously the
`fileInternalUses` field in `dead-classify.json` looked like AST
evidence; now it carries a sibling:

```json
{
  "fileInternalUses": 2,
  "fileInternalUsesEvidence": "regex-text-with-decl-exclusion"
}
```

Downstream consumers (fix-plan, Claude review layer) can now weigh
this counter appropriately. Full AST local reference count is
deferred to v1.10.0 (reviewer's recommended staging). The label
makes it explicit that this counter's confidence is lower than
AST-derived consumer lookups.

### 4. SKILL.md Overview framed honestly

Before:

> Repository structural audit using AST evidence, not LLM intuition.

After:

> Repository structural audit using machine-collected evidence:
> AST graphs where available, explicit text heuristics where not.
> The skill emits evidence; the model makes scoped claims from it.

Adds "A script tier is not a claim" sentence next to the Iron Law.
Same principle the v1.9.7 FP-36 release formalized for Tier C,
now surfaced at the very top of the document.

### Self-dogfood: compare-repos against self

Two runs of the same audit, diffed:

```
══════ audit-artifact compare ══════
  left:  self-a  (3 artifacts)
  right: self-b  (3 artifacts)

  totalDefs      : +0
  deadInProd     : +0
  SAFE_FIX       : +0
  REVIEW_FIX     : +0
  DEGRADED       : +0
  MUTED          : +0
```

Self vs self shows zero deltas across all dimensions. Asymmetry
case (one side missing runtime-evidence) correctly nullifies the
affected deltas. Verified by the regression test.

### Tests (218 total, up from 207)

New suite `tests/test-evidence-honesty.mjs` — 11 assertions split
across two sub-tools:

- **C1–C7** (compare-repos): valid inputs → exit 0; correct delta
  arithmetic (files, safeFixes, degraded); artifactsFound
  enumeration; missingArtifacts correctly populated; **C7**
  asymmetric case produces `null` delta (not invented number)
- **D1–D4** (check-doc-script-refs): exit 0 when references
  resolve; exit non-zero when they don't; error message includes
  remediation hints; files under `_lib/` count as present

### CI pipeline — 6 gates now

```
check               → syntax check
check:drift         → version drift (5 sources)
check:test-doc      → tests/README.md generator drift
check:doc-script-refs  → SKILL.md referenced .mjs exist on disk (NEW)
lint                → ESLint flat config
test                → 17 suites, 218 assertions
```

### Explicitly NOT in this release (deferred)

Reviewer's full "Evidence Honesty Patch" list included two items
beyond the four closed here. Left for v1.10.0 per reviewer's own
staging:

- **`audit-repo.mjs` one-shot orchestrator**: would run
  triage → topology → discipline → symbols → classify → runtime →
  staleness → rank → sarif in order, emit a `manifest.json` with
  `commandsRun`, `scanRange`, `unsupportedLanguages`,
  `unresolvedInternalRatio`. Substantial design decisions (partial
  failure handling, parallelism, overwrite policy) — worth one
  focused release rather than folding in here.
- **AST local reference counter** (replacing the regex path labeled
  above). Requires real AST walking of every symbol scope, which is
  more work than the label-and-flag approach used here.
- **Unsupported language manifest field**: triage already reports
  per-language file counts; a formal `absenceClaimAllowed: false`
  gate is a separate artifact-shape change.

Cloudflare Pages Functions sentinel (`onRequestGet/Post/etc` in
`functions/**`) also still deferred from v1.9.7 — framework policy
release, separate concern from evidence-honesty.

### Status

| Release | Assertions | Suites | Capability |
|---|---|---|---|
| 1.9.6 | 200 | 16 | MUTED materialized + SARIF verified |
| 1.9.7 | 207 | 17 | Scope-aware tsconfig paths (FP-36) |
| **1.9.8** | **218** | **18** | **Evidence honesty: compare, doc-ref guard, regex labels** |

### Review principle made explicit

The v1.9.7 release said "Tier C is raw evidence, not a claim." This
release puts the same principle on four additional surfaces:

- If the skill says to run `X.mjs`, `X.mjs` must exist (check:doc-script-refs)
- If the skill emits a count, the count's evidence source is labeled (regex-text)
- If two repos are compared, missing evidence produces `null` not `0`
- If the Overview describes the approach, it distinguishes AST from heuristic

One rule: **the skill must not quietly overstate the evidence it has.**

## 1.9.7 — 2026-04-19

**FP-36 emergency patch.** Critical resolver bug discovered on
duyet/monorepo (2026-04): 218 of 397 Tier C dead-export findings were
actually consumed via per-app `tsconfig.json` `paths` aliases that the
resolver did not read. 73.2% FP rate from one blind spot.

Reviewer's framing of the discovery is what this release formalizes:

> Tier C = no consumer found in the constructed graph.
> Tier C ≠ truly dead.
> When the resolver is scope-blind, Tier C is not comparable to any
> other tool's "orphan" count.

### The bug

In multi-app monorepos each app often defines its own tsconfig:

```json
// apps/agents/tsconfig.json
{ "compilerOptions": { "baseUrl": ".", "paths": { "@/*": ["./*"] } } }
```

Inside `apps/agents`, `import { AuthControl } from '@/components/auth-control'`
must resolve to `apps/agents/components/auth-control.tsx`. The same
specifier in `apps/admin` must resolve to `apps/admin/components/auth-control.tsx`.

Before this release the resolver had **no tsconfig paths support at
all** — only `package.json` exports + root-prefix fallbacks. Every
`@/*` specifier fell through to `EXTERNAL`, which got collapsed to
`null`, which inflated `unresolvedUses`. `AuthControl` appeared
consumer-less and landed in Tier C. Observed on duyet/monorepo: 218
Tier C FPs from this single cause.

### The fix — five coupled changes

**1. New module `_lib/tsconfig-paths.mjs`** — walks the repo finding
every `tsconfig.json`, parses `compilerOptions.paths` with its
`scopeDir` and `baseUrlDir`. Handles `extends` chains, comments,
trailing commas. Returns a flat array; consumers apply it
nearest-scope-first per importer.

**2. `_lib/alias-map.mjs`** — attaches `scopedTsconfigPaths` as a
property on the returned Map. Backward compatible: callers that
iterate `for (const [k, v] of aliasMap)` are unaffected.

**3. `_lib/resolver-core.mjs`** — new probe block BEFORE alias
lookup. Filters scopes to ones containing `fromFile`, sorts by
`scopeDir.length` desc (more specific wins) then `matchPrefix.length`
desc. New sentinel `UNRESOLVED_INTERNAL` distinguishes "local alias
matched but target file missing" from `EXTERNAL` (genuine npm package):

```
spec = '@/components/missing'
scoped match = yes, but probeTarget returned null
→ UNRESOLVED_INTERNAL (scanner blind spot, not external)
```

**4. `build-symbol-graph.mjs`** — counters split into three:
`uses.resolvedInternal` / `uses.external` / `uses.unresolvedInternal`.
Plus `uses.unresolvedInternalRatio` and `topUnresolvedSpecifiers[]`
with a `likelyCause` heuristic for `@/`, `~/`, `#/`, `@scope/*`
prefixes pointing users at FP-36. Legacy fields
(`totalUsesResolved`, `unresolvedUses`) still emitted for
backward compat.

**5. `rank-fixes.mjs` + `emit-sarif.mjs`** — gate now uses
`unresolvedInternalRatio`, not conflated total. External packages
(react, oxc-parser, eslint) no longer trip the 15% resolver-blindness
gate. Self-dogfood went from 61.4% (false) to 0% (real) blind spot.

### Self-dogfood — the bug fixed itself

Before (v1.9.6):
```
uses: resolvedInternal not tracked; unresolvedUses = external + internal
unresolvedRatio = 0.614 → gate tripped → everything DEGRADED
fix-plan: SAFE_FIX 0, REVIEW_FIX 0, DEGRADED 2, MUTED 1
```

After (v1.9.7):
```
uses: resolvedInternal 121, external 202, unresolvedInternal 0
unresolvedInternalRatio = 0 → gate ok
fix-plan: SAFE_FIX 0, REVIEW_FIX 2, DEGRADED 0, MUTED 1
```

Every one of the 202 "unresolved" uses in 1.9.6 was a legitimate
external package. Splitting the counter recovered the findings that
the conflated gate was burying.

### `tests/test-tsconfig-paths-scoped.mjs` — 7 assertions

Two-app fixture (agents + admin), both using `@/*` mapped to their
own directories. Critical invariants:

- **T1** agents import → agents component
- **T2** admin import → admin component
- **T3** (key) same specifier, different importers, DIFFERENT target
  files. A flat alias map cannot pass T3. Only a scope-aware
  resolver can.
- **T4** `AuthControl` is NOT in deadProdList (consumer found)
- **T5** `uses.unresolvedInternal === 0` for these imports
- **T6** matched local alias + missing target → `UNRESOLVED_INTERNAL`
- **T7** genuine external (react) → `EXTERNAL`

Empirically verified the test catches FP-36: reverting the
scoped-paths block in resolver-core makes T1/T2/T3/T4/T6 all fail
(5 of 7 red). Restored: 7/7 green.

### Tier C framing in SKILL.md

Added "Tier C is raw evidence, not a claim" section to Grounding
Levels. Spells out when Claude must downgrade Tier C to REVIEW_FIX
or DEGRADED even when the ranking layer hasn't:

- `resolverBlindness.gate === 'tripped'`
- `topUnresolvedSpecifiers` contains local alias prefix patterns
- parse errors in `symbols.json.meta.warnings`
- framework/config/generated files the classifier missed

Reviewer's principle made explicit:

> Tier C는 claim이 아니라 raw evidence다.
> resolver confidence가 낮으면 Claude가 claim을 낮춰야 한다.

### FP-36 ledger entry

Full entry in `references/false-positive-patterns.md` including symptom,
design caveat (why a flat alias map cannot fix this), mitigation,
evidence, and Iron Law implication. Added to the FP lookup list
at the end of the file.

### Backward compatibility

- Existing `symbols.json` fields unchanged — `totalUsesResolved`,
  `unresolvedUses`, `deadProdList`, etc. all still emitted.
- New fields are additive: `uses.*`, `topUnresolvedSpecifiers`.
- `rank-fixes.mjs` falls back to legacy total-ratio when a pre-1.9.7
  `symbols.json` is present (tagged with `source: 'legacy (may
  include externals)'` in `fix-plan.meta.resolverBlindness`).
- `aliasMap` still iterates cleanly as a Map; new
  `scopedTsconfigPaths` is a property.
- Existing 10 resolver-path tests still pass — scoped probe only
  activates when a tsconfig with `paths` is found, which none of the
  existing fixtures have.

### Tests (207 total, up from 200)

- `test-tsconfig-paths-scoped.mjs`: 7 new assertions (T1–T7)
- All existing suites pass unchanged

### Explicitly deferred to v1.9.8

- Cloudflare Pages Functions sentinel (`onRequestGet/Post/etc.` in
  `functions/**`) — reviewer flagged as lower priority (16 findings
  vs 218 for FP-36; worth doing but not the same urgency).
- Generated-file policy (`isGeneratedFile()` heuristics) — v1.9.8
- Package entrypoint detection (`main/module/bin/browser/types/scripts`) — v1.9.8
- Resolver "candidate evidence" shape for Claude review layer
  (per-finding `unresolvedInternalNearby`, `resolverConfidence`) —
  the current implementation surfaces this globally via
  `topUnresolvedSpecifiers`; per-finding attribution is a bigger
  artifact-shape change.

### Status

| Release | Assertions | Suites | Capability |
|---|---|---|---|
| 1.9.5 | 189 | 15 | 4-tier ranking (MUTED empty) |
| 1.9.6 | 200 | 16 | MUTED materialized + SARIF verified |
| **1.9.7** | **207** | **17** | **Scope-aware tsconfig paths (FP-36)** |

### The lesson

The user's LLM review layer caught this by symbol-grepping the
corpus after the mechanical analysis ran. The skill said
`AuthControl` is dead; `grep "AuthControl" apps/` returned
`chat-top-bar.tsx` importing it via `@/components/auth-control`.
That's the pattern this project was designed around — skill
produces evidence, Claude produces claims. Before 1.9.7 the evidence
was silently wrong on a common repo shape, and the claim layer had
nothing to flag. Now:

- The evidence tells you it's unsure (`unresolvedInternalRatio`,
  `topUnresolvedSpecifiers`)
- The ranking layer gates warnings on that signal
- SKILL.md tells Claude to downgrade Tier C when the signal fires

## 1.9.6 — 2026-04-19

Review-driven completion of the v1.9.5 ranking layer. Reviewer
confirmed 1.9.5's direction but caught five integration gaps between
the claim and the actual pipeline. All five closed.

### 1. MUTED tier materialized end-to-end

v1.9.5 claim:

> MUTED — classifier-excluded by an FP policy. Not emitted.

Implemented: `_lib/ranking.mjs` had the MUTED predicate, but
`classify-dead-exports.mjs` dropped excluded candidates via `continue`
— they never reached `fix-plan.json`. Result: `summary.MUTED` was
always 0 in practice, even on repos where the classifier excluded
dozens of config/public-API files.

**Fix (classifier)**: `classify-dead-exports.mjs` now records each
excluded candidate in `dead-classify.excludedCandidates[]` with
`{file, line, symbol, kind, reason}`, where `reason` is one of
`config_FP22` / `publicApi_FP23` / `frameworkSentinel_FP27` /
`nuxtNitro_FP30`. Exclusion counts are unchanged — the materialized
list is additive.

**Fix (ranker)**: `rank-fixes.mjs` flattens `excludedCandidates` into
a separate list, passes `policy.excluded=true` + `policy.reason` into
`tierForFinding()`, and pushes results into the MUTED bucket of
`fix-plan.json`. Users can now audit what policy hid.

**Verified**: self-dogfood now produces `MUTED: 1` (the
`eslint.config.mjs default` symbol that FP-22 excluded), matching
reviewer's expected `DEGRADED: 2 / MUTED: 1 / total: 3`.

### 2. Self-dogfood numbers in CHANGELOG matched reality

v1.9.5 printed `DEGRADED: 3, total: 3` but the actual pipeline
produces `DEGRADED: 2` because one candidate was FP-22 excluded
before reaching the classifier proposals. With v1.9.6's MUTED
materialization the line is now `DEGRADED: 2, MUTED: 1, total: 3`,
which the reviewer explicitly predicted and verified empirically.

### 3. Tier descriptions in ranking.mjs match the code

The v1.9.5 header listed "no coverage" and "namespace/dynamic shadow"
under DEGRADED, but the actual code routes those to REVIEW_FIX (no
coverage is a common, benign state — demoting every finding without
coverage to DEGRADED would be too aggressive). SAFE_FIX was also
described as "strong evidence for removal/demotion" but the code
deliberately caps A-bucket (demote-to-internal) at REVIEW_FIX — only
mechanical C-bucket + specifier reach SAFE_FIX.

Rewrote the header to match behavior:
- SAFE_FIX = mechanical removal / specifier cleanup only (C +
  specifier with strong multi-source evidence; A never reaches here)
- REVIEW_FIX = concrete action exists but evidence is incomplete
  (no runtime, no staleness, recent edits, A-bucket, B-bucket)
- DEGRADED = runtime-executed override, resolver ≥15% global gate,
  or unclassified bucket

### 4. SKILL.md + templates/report-template.md surface the ranking layer

The v1.9.5 Workflow block listed 7 steps without `rank-fixes.mjs`.
Quick Reference had no fix-plan row. SARIF section described the old
ad-hoc grounded/degraded → warning/note mapping.

Updated:
- Workflow now has step 6 `rank-fixes.mjs → fix-plan.json` between
  the measurement scripts and `emit-sarif.mjs`, with a paragraph
  explaining why skipping this step causes SARIF severity to fall
  back to ad-hoc logic.
- Quick Reference row `rank-fixes.mjs` added.
- SARIF section rewritten to explain the tier → level mapping and
  the `properties.tier` / `properties.reason` fields available for
  programmatic filtering.
- `templates/report-template.md` HCA-2 now maps fix-plan tiers to report
  decision categories (SAFE_FIX → section 6 auto-progressable,
  REVIEW_FIX → HCA-2 human decision, DEGRADED → evidence-improvement
  first, MUTED → not a finding).

### 5. emit-sarif fix-plan branch has a regression test

New suite `tests/test-sarif-fix-plan.mjs` — 10 integration
assertions. Synthesizes a `fix-plan.json` with one entry per tier,
runs `emit-sarif.mjs` against it, asserts the resulting SARIF:

- carries `properties.tier` on all GA001 results (S1)
- emits 3 results, not 4 — MUTED is filtered out (S2)
- no leaked `tier=MUTED` (S3)
- SAFE_FIX → warning, REVIEW_FIX → note, DEGRADED → note (S4-S6)
- `properties.proposalBucket` preserves classifier bucket (S7)
- `properties.reason` carries ranking reason for filtering (S8)
- overall distribution 1w/2n/0e (S9)
- `hitsInSymbol` surfaces on runtime-executed DEGRADED for audit (S10)

This closes the v1.9.5 claim "SARIF severity comes from the tier →
level map" end-to-end. Previously only the predicate and the merge
layer were tested; the emission wiring was unverified.

### Small fix

v1.9.4 CHANGELOG subject said "Four small text fixes" but the
numbered list had 5 items. Corrected to "Five."

### Tests (200 total, up from 189)

- `test-sarif-fix-plan.mjs` — 10 new assertions (S1-S10)
- `test-rank-fixes.mjs` — 1 new assertion (I1b) verifying
  excludedCandidates materialize as MUTED through the pipeline
- Total: +11 across 2 suites (one new: `test-sarif-fix-plan.mjs`)

### Pipeline contract (unchanged since v1.9.5)

```
build-symbol-graph      → symbols.json
classify-dead-exports   → dead-classify.json (now includes excludedCandidates)
merge-runtime-evidence  → runtime-evidence.json [optional]
measure-staleness       → staleness.json [optional]
rank-fixes              → fix-plan.json (now populates MUTED)
emit-sarif              → grounded-audit.sarif (consumes fix-plan)
```

### Status

| Release | Assertions | Suites | Capability |
|---|---|---|---|
| 1.9.4 | 172 | 14 | ranked detector |
| 1.9.5 | 189 | 15 | 4-tier fix proposal engine (MUTED empty in practice) |
| **1.9.6** | **200** | **16** | **MUTED materialized; SARIF integration verified** |

### Next: v1.9.7 (precision layer, per reviewer's v1.9.6 plan)

Reviewer's top-priority follow-up: split external dependencies from
unresolved internal imports so `resolver.unresolvedRatio` isn't
dominated by legitimate `oxc-parser` / `eslint` imports. Currently
self-dogfood trips the 15% gate at 61% purely because of external
package imports counted as unresolved. Proposed breakdown:

```
uses: {
  resolvedInternal: number,   // local imports that resolved
  external:         number,   // package imports (node_modules)
  unresolvedInternal: number  // the real blind spot
}
unresolvedRatio = unresolvedInternal / (resolvedInternal + unresolvedInternal)
```

Plus `topUnresolvedSpecifiers[]` so users see which tsconfig path or
alias would have the biggest impact.

Also for v1.9.7: generated-file policy (`isGeneratedFile()` →
auto-MUTED), package entrypoint detection (main/bin/module/browser/
types/scripts), definition start/end line range for coverage fusion
precision.

## 1.9.5 — 2026-04-19

**Ranking layer.** First release of the architectural track proposed
by external review: shift from "candidate detector" toward "fix
proposal engine." Sixth of the reviewer's 13-point plan is the first
to land.

### What's new

**`_lib/ranking.mjs`** — pure 4-tier predicate used by both
`rank-fixes.mjs` and `emit-sarif.mjs`. No AST parsing, no scanning;
consumes existing artifacts and classifies each finding:

- **SAFE_FIX** — strong multi-source convergence: AST-dead +
  runtime-dead-confirmed + staleness fossil/stale + mechanical
  bucket (C or specifier). Candidate for automated fix. Emitted as
  SARIF **warning**.
- **REVIEW_FIX** — AST-dead, classifier proposes concrete action,
  but supporting evidence short of the SAFE bar (missing runtime
  coverage, recent edits, or A-bucket "demote-to-internal" which
  needs human judgment on internal consumers). Emitted as SARIF
  **note**.
- **DEGRADED** — evidence contradicts or is structurally missing:
  runtime-executed (overrides everything), resolver unresolvedRatio
  ≥ 15% (global gate), B-bucket without predicate match, etc.
  Emitted as SARIF **note** but never as warning.
- **MUTED** — classifier-excluded by FP policy (config FP-22, public
  API FP-23, framework sentinel FP-27, Nuxt/Nitro FP-30).
  Not emitted.

**`rank-fixes.mjs`** — merges `dead-classify.json` +
`runtime-evidence.json` + `staleness.json` + `symbols.json` into
`fix-plan.json`. Missing optional inputs degrade findings rather
than dropping them (never promote without evidence).

Input → tier table (reviewer's design):

| AST-dead | runtime | staleness | bucket | tier |
|---|---|---|---|---|
| yes | dead-confirmed | fossil/stale | C or specifier | **SAFE_FIX** |
| yes | dead-confirmed | recent | C/specifier | REVIEW_FIX |
| yes | dead-confirmed | fossil/stale | A | REVIEW_FIX |
| yes | executed (n>0) | any | any | **DEGRADED** |
| yes | any | any | B | REVIEW_FIX |
| — | — | — | resolver unresolved ≥15% | **DEGRADED** |
| — | — | — | policy-excluded | **MUTED** |

### emit-sarif.mjs

Added a fix-plan branch at the top of the cascade. When present,
SARIF severity comes from the tier → level map, not the ad-hoc
logic. Old branches (runtime-evidence, dead-classify, symbols.json)
remain as fallbacks for pipelines that don't run `rank-fixes`.

Each GA001 result now carries `properties.tier`, `properties.reason`,
`properties.proposalBucket` — consumers can filter by tier without
regex-parsing the message text.

### Pipeline contract (new)

```
build-symbol-graph      → symbols.json
classify-dead-exports   → dead-classify.json
merge-runtime-evidence  → runtime-evidence.json        [optional]
measure-staleness       → staleness.json               [optional]
rank-fixes              → fix-plan.json                [new]
emit-sarif              → grounded-audit.sarif        [consumes fix-plan]
```

Backward compatible: if `rank-fixes` doesn't run, `emit-sarif` falls
through to the previous runtime-evidence branch.

### Self-dogfood finding

Running the full pipeline against this skill's own source surfaces
something honest:

```
══════ fix-plan ranking ══════
  SAFE_FIX    : 0
  REVIEW_FIX  : 0
  DEGRADED    : 3
  MUTED       : 0

  ⚠ resolver unresolvedRatio = 61.1%
    All findings DEGRADED — add a tsconfig path or alias to reduce.
```

61% is driven by external dependencies (`oxc-parser`, `eslint`, etc.)
being counted alongside `_lib/*` local imports in the resolver stats.
Treating external packages as "external" rather than "unresolved" is
part of the v1.9.6 precision layer (reviewer item 9: surface top
unresolved specifiers). For now, the gate correctly protects users
from overconfident warnings on a scanner in this state.

### Tests (189 total, up from 172)

New suite `tests/test-rank-fixes.mjs` — 10 unit + 7 integration
assertions (17 total, bringing grand total to 189 across 15 suites).

**Unit** (hand-built finding + evidence objects, no I/O):
- R1–R10: every tier boundary — SAFE_FIX convergence, DEGRADED
  override on runtime-executed, MUTED on policy exclusion, REVIEW_FIX
  fallback, resolver-blindness gate, A-bucket ceiling, specifier
  SAFE_FIX path, B-bucket always-review, TIER_TO_SARIF_LEVEL shape.

**Integration** (synthesize artifacts, invoke `rank-fixes.mjs`):
- I1: fix-plan.json has the expected summary+bucket shape.
- I2–I4: "Fossil" symbol (all signals align) reaches SAFE_FIX;
  "Active" symbol (recent edits) does NOT, ends up in REVIEW_FIX.
- I5: meta.inputs records which artifacts were consumed.
- I6: resolverBlindness gate stays `ok` on a healthy fixture.
- I7: **runtime-executed finding never reaches SAFE_FIX** — the
  reviewer's explicit CI-noise prevention requirement, guarded.

### Status

| Release | Assertions | Suites | Capability |
|---|---|---|---|
| 1.9.4 | 172 | 14 | ranked detector |
| **1.9.5** | **189** | **15** | **4-tier fix proposal engine** |

### Next: v1.9.6 (precision layer)

Per reviewer's recommended sequence, items 7, 8, 9, 11:
- generated-file policy (`isGeneratedFile()` → MUTED)
- package entrypoint detection (main/bin/module/browser/types + scripts)
- unresolved-ratio gate with topUnresolvedSpecifiers artifact output
- definition start/end ranges for precise coverage fusion

v2.0.0 (solve expansion) lands items 3, 4, 5, 6, 12:
- symbol-level public API (replace file-level reExportsByFile)
- namespace-import symbol-level tracking
- dynamic-import symbol-level tracking
- AST local reference count (replacing regex)
- `apply-dead-fixes.mjs --dry-run`

## 1.9.4 — 2026-04-19

Five small text fixes. No code changes.

1. CHANGELOG v1.9.3's v1.9.1-proof description said "Correct when
   written" and also "would have been misleading even the day it
   shipped" — self-contradiction. The second was true: v1.9.1's own
   subject prose already contained "999", so the broad `grep -c "999"`
   proof returned 1 the day it shipped too. Kept the "too broad even
   when it shipped" framing; removed the "correct when written"
   phrasing.

2. "v1.9.2's own narrative quotes '999'" in the same block was
   attributing the string to the wrong release. Corrected to
   "v1.9.1's own subject prose."

3. The v1.9.3 meta section was titled "sixth release in a row" but
   1.8.2 → 1.9.3 is eight releases. Heading now reads "another
   release in the review loop" — dropping the count entirely, which
   is safer than getting the count right once and drifting on the
   next release.

4. `scripts/check-drift.mjs` inline section 5 still said README drift
   is "not a build break." Since 1.9.0, `check:test-doc` in CI makes
   it a build break. Comment updated to point at the sibling script.

5. `tests/test-update-test-doc.mjs` header said "v1.9.2 (this
   revision)" but this revision is v1.9.4. Removed the "(this
   revision)" parenthetical rather than chasing the version on
   every release.

### Tests (172 total, unchanged)

No test changes. This is prose cleanup only.

### Status

| Release | Assertions | Suites |
|---|---|---|
| 1.9.3 | 172 | 14 |
| **1.9.4** | **172** | **14** |

## 1.9.3 — 2026-04-19

Self-consistency patch. Reviewer accepted v1.9.2's correctness and
claim-scoping fixes but found two CHANGELOG-internal contradictions
that themselves undermined the "honesty patch" framing. Plus one
cleanup that removes dead code the previous comment already flagged.

### 1. v1.9.1 status table said "accurate"

The v1.9.2 release text said:

> 1.9.1 overclaimed "cannot contain any count"

But the status table a few lines above, inside the same file, still had:

> \| **1.9.1** \| ... \| "README cannot carry current/historical
>   counts" — **accurate** \|

File-internal contradiction. Fixed: the 1.9.1 row now reads
"count-suffix leak fixed; broader 'cannot carry any count' claim was
overclaim, corrected in 1.9.2" — matching the v1.9.2 prose.

### 2. v1.9.1's empirical proof was a `grep -c "999"` that now fails

The v1.9.1 CHANGELOG entry shipped this proof block:

```
$ sed -i 's/### Tests (171 total/### Tests (999 total/' CHANGELOG.md
$ node scripts/update-test-doc.mjs
$ grep -c "999" tests/README.md
0
```

The proof was too broad even when it shipped: v1.9.1's own subject
prose contains the string "999" while describing reviewer's attack,
and that prose gets copied into the README retrospective. Running
`grep -c "999" tests/README.md` in v1.9.1 already returned 1, not 0
— the fix didn't regress, the proof form was wrong. Rewritten to
match 1.9.1's actual guarantee (per-release count suffix is absent):

```
$ grep -cE '^- \*\*v[0-9.]+\*\* \(999\):' tests/README.md
0
```

This stays true regardless of how CHANGELOG narrative prose evolves.

### 3. `testCount` parse removed from generator

`scripts/update-test-doc.mjs` stopped rendering `### Tests (N total)`
counts in v1.9.1, but the parse itself stuck around:

```js
const testsM = body.match(/### Tests \((\d+) total/);
const testCount = testsM ? parseInt(testsM[1], 10) : null;
entries.push({ version, subject, testCount });
```

Reviewer flagged as polish, not a blocker. Removed in this patch
because keeping dead fields around is exactly the kind of thing the
generator should not model — "what the README gets" is now exactly
"what's parsed from CHANGELOG." Entries shape simplified to
`{ version, subject }`. Header comment updated to match.

### Tests (172 total, unchanged)

No test changes. This release is prose/dead-code cleanup only.

### Status

| Release | Assertions | Suites | Claim honesty |
|---|---|---|---|
| 1.9.0 | 171 | 14 | overclaimed "all drift impossible" |
| 1.9.1 | 172 | 14 | count-suffix leak fixed; broader claim was overclaim |
| 1.9.2 | 172 | 14 | claim scoped correctly but CHANGELOG self-contradicted |
| **1.9.3** | **172** | **14** | CHANGELOG consistent with itself; dead parse removed |

### Meta observation — another release in the review loop

Each of 1.8.2 → 1.9.3 has been a reviewer-driven patch where the
specific code/doc issue got fixed but something adjacent — a header
comment, an assertion-count hardcode, a subject-prose count leak, a
CHANGELOG table row — remained inconsistent. Each release, one more
layer of prose drift surfaced. The pattern feels recursive.

The substantive convergence: each fix narrows the claim. The code's
actual guarantees haven't changed much since 1.9.0; the text around
them has tightened release after release. If 1.9.4 shows up with
another stale line somewhere, the honest response is probably to
accept that no CHANGELOG edit can be fully self-consistent while
adding new self-referential commentary, and ship it.

## 1.9.2 — 2026-04-19

Small honesty patch on top of 1.9.1. Reviewer accepted the count-leak
fix but caught two correctness issues and three small cleanups. All
five closed.

### 1. Release claim was still slightly overclaimed

v1.9.1 said:

> README structurally cannot contain a current or historical count,
> even if CHANGELOG lies.

That was true only for INJECTED counts (the `**v1.9.0** (171)` suffix
the generator used to produce). It wasn't true for counts inside
CHANGELOG subject prose — e.g. "All 104 test assertions pass" — which
the generator extracts and copies as-is. Reviewer demonstrated by
injecting "All 999 test assertions pass." into a CHANGELOG subject
and confirming the 999 appeared in README, while T7 still passed.

Corrected claim in this release:

> the README no longer injects a count suffix like `**v1.9.0** (171)`
> for any release. Historical prose copied from CHANGELOG subjects
> can still mention counts — that's factual record from the release
> description, not a drift vector we inject. `npm test` is the only
> source of truth for the CURRENT count.

This is what the code actually guarantees. Scoping the claim to the
concrete mechanism is the pattern v1.9.0 already failed at; getting
it right here.

### 2. Test was mutating the real working tree

`tests/test-update-test-doc.mjs` backed up and restored the real
`tests/README.md` and created a temporary suite file in the real
`tests/` directory. Normal runs restored via `finally`, but a SIGKILL
or parallel-test-runner disruption would leave the working tree dirty.

Rewritten to run entirely in an `mkdtempSync` temp repo:
- copies CHANGELOG.md, scripts/, tests/ into a fresh `/tmp/fx-test-doc-*`
- runs `node <fixture>/scripts/update-test-doc.mjs` pointing at the
  fixture's files
- never touches the real repo

Worst-case leftover on abort is now a `/tmp/fx-test-doc-*` directory —
zero impact on the working tree. Matches the README's own promise:
"No shared state between runs."

### 3. scripts/update-test-doc.mjs header was stale

Header said the retrospective extracts counts from
`### Tests (N total)`. Since 1.9.1, it parses but does not render
them. Updated to reflect v1.9.2 reality — another instance of
header-vs-implementation drift (same pattern the tool was created to
help catch).

### 4. scripts/check-drift.mjs header was stale

Header said tests/README.md drift is "intentionally not checked"
because "a hard check would block releases for cosmetic reasons."
Since 1.9.0, `check:test-doc` actually does hard-check the README,
just via a different mechanism (regeneration equivalence). Comment
now reflects the sibling-script reality.

### 5. Lint warning cleared

`let startIdx` → `const startIdx` in `scripts/update-test-doc.mjs`.
Zero lint output on `npm run lint` now.

### Tests (172 total, unchanged from 1.9.1)

T1–T8 unchanged in shape, same regression guarantees. The suite is
fully hermetic now — pass/fail semantics identical, failure modes on
abort radically smaller.

### Status

| Release | Assertions | Suites | Claim honesty |
|---|---|---|---|
| 1.9.0 | 171 | 14 | overclaimed "all drift impossible" |
| 1.9.1 | 172 | 14 | overclaimed "cannot contain any count" |
| **1.9.2** | **172** | **14** | "injected counts are impossible; CHANGELOG subject prose is not filtered" |

### Meta observation (five releases running)

1.8.2 → 1.8.5 caught doc drift four times. 1.9.0 apologized and
patched the concrete case. 1.9.1 apologized for overclaim-while-patching.
1.9.2 (this) apologizes for overclaim-while-patching-the-overclaim.

The pattern finally converges when the claim matches the code:
"injected counts are gone; extracted prose is not sanitized." That
statement is boring but correct. Probably the right kind of ending.

## 1.9.1 — 2026-04-19

Honest patch. v1.9.0 claimed "drift is mechanically impossible to
ship." That was over-strong. Reviewer proved it by setting
`### Tests (999 total)` in CHANGELOG and watching the false count
propagate into the README through `update-test-doc.mjs`, with
`check:test-doc` and `test-update-test-doc.mjs` T7 both passing.

### The remaining hole

v1.9.0 removed the hardcoded total from the README header but left
per-release bullets as `**v1.9.0** (171): …`. The `(171)` came from
the generator reading `### Tests (N total` out of CHANGELOG. Since
nothing verifies that CHANGELOG count against actual test output,
a wrong CHANGELOG number quietly became a wrong README number.

T7 had a similar gap: it only matched `**N assertions**` and
`N assertions across`, never `**v1.9.0** (N)`. So the exact vector
the generator actually produced was outside the guard.

### Fix

**Generator** (`scripts/update-test-doc.mjs`): release bullets no
longer carry counts. The line

```js
const count = e.testCount !== null ? ` (${e.testCount})` : '';
return `- **v${e.version}**${count}: ${e.subject}`;
```

is now

```js
return `- **v${e.version}**: ${e.subject}`;
```

`parseChangelog` still extracts `testCount` for internal use but it's
no longer rendered. Result: the README no longer injects a count
suffix like `**v1.9.0** (171)` for any release. Historical prose
copied from CHANGELOG subjects ("all 104 test assertions pass") can
still mention counts — that's factual record from the release
description, not a drift vector we inject. `npm test` is the only
source of truth for the CURRENT count.

**Test** (`tests/test-update-test-doc.mjs`): T7 rewritten with four
narrower patterns targeting the drift-prone forms:
- `**v1.9.0** (171)` — bullet count suffix
- `**N assertions**` — bold current total
- `**total: N**` — bold total
- `N assertions across M suites` — grand-total prose

Historical prose from extracted CHANGELOG subjects ("all 104 test
assertions pass") is correctly NOT flagged — that's a factual record,
not drift. Previous T7 was too broad and would have matched such
subjects, which is why I had to tune it before shipping.

**Test** (new T8): verifies the "Maintainer note" path actually
works. Previous test file's header comment claimed this was covered
but no assertion existed. T8 now:
1. writes a fresh `tests/test-z-zz-temp.mjs` with no description map entry
2. regenerates the README
3. asserts the README contains `## Maintainer note` and the new
   suite's filename
4. cleans up in `finally`

### Empirical proof

Reviewer's attack repeated after this fix. The scoped version of the
check — looking for the specific injected form `**vX.Y.Z** (N)` — is
what 1.9.1 actually closes. (A naive `grep -c "999"` would have been
misleading even in 1.9.1: the string "999" appears in this very
CHANGELOG's v1.9.1 narrative, and that prose is copied verbatim into
the README retrospective; 1.9.2 corrects the claim to acknowledge
this.)

```
$ sed -i 's/### Tests (171 total/### Tests (999 total/' CHANGELOG.md
$ node scripts/update-test-doc.mjs
$ grep -cE '^- \*\*v[0-9.]+\*\* \(999\):' tests/README.md
0
```

The false header count no longer reaches the README as an injected
per-release count suffix. Counts inside CHANGELOG subject prose are
out of scope — see v1.9.2 for the corrected claim.

### Tests (172 total, up from 171)

- T7 tightened (same assertion, stricter predicate)
- T8 added (+1)

### Status

| Release | Assertions | Suites | Honest claim |
|---|---|---|---|
| 1.9.0 | 171 | 14 | "hand-edit drift prevented" — correct |
| 1.9.0 | 171 | 14 | "drift mechanically impossible" — overclaim |
| **1.9.1** | **172** | **14** | count-suffix leak fixed; broader "cannot carry any count" claim was overclaim, corrected in 1.9.2 |

### Meta observation

v1.9.0 meta-claimed to break the write-then-apologize cycle. v1.9.1
breaks a SPECIFIC subset of it: count-drift through the
README-generation pipeline. Other forms (suite-description map
forgetting an entry; CHANGELOG count being wrong relative to actual
test output) are still possible. The second one — CHANGELOG vs
`npm test` reality — could be closed with a small checker but the
payoff is lower now that the count doesn't leak into user-facing
docs.

When I said "mechanically impossible" last release, I meant "the
specific drift pattern I was fixing is closed." That was true. I
overstated it to "all drift." The lesson: scope the claim to what
the fix actually guarantees. Noted and shipped.

## 1.9.0 — 2026-04-19

Structural release. Stops the write-then-apologize cycle that ran
through 1.8.2 → 1.8.5 by making `tests/README.md` a generated artifact
with a CI gate against drift.

### Why a minor bump

Four consecutive patch releases (1.8.2 → 1.8.5) each shipped with the
same meta-bug: code fix correct, adjacent doc/test claim weaker than
advertised. Each release's CHANGELOG acknowledged the pattern; none
actually fixed it. This release does.

Minor rather than patch because:
- New script `scripts/update-test-doc.mjs`
- New npm scripts: `update-test-doc`, `check:test-doc`
- CI pipeline now has 5 gates instead of 4
- `tests/README.md` changes from hand-maintained to generated —
  external tooling that reads this path still works (same location,
  same markdown) but anyone editing by hand will see their changes
  reverted by the next `npm run update-test-doc`.

### Generator: `scripts/update-test-doc.mjs`

Reads:
- `CHANGELOG.md` → extracts `## X.Y.Z` headings + subject lines +
  `### Tests (N total` counts where present
- `tests/test-*.mjs` → enumerates current suite list on disk
- Small local dictionary mapping suite filename → short description

Writes `tests/README.md` with:
- A `<!-- GENERATED FILE — do not edit by hand -->` banner
- An explicit note that the assertion count was REMOVED from this
  file because it drifted in four consecutive releases; `npm test`
  output is now the authoritative count
- Suite list generated from the filesystem (new suites appear
  automatically)
- Per-release retrospective generated from CHANGELOG (no hand-edit
  required)
- A "maintainer note" section when a new suite is added without a
  description — prompts the next editor to add one

Modes:
- `npm run update-test-doc`   → regenerate README
- `npm run check:test-doc`    → exit 1 if README differs from
  generated output (CI gate)

### Tests (171 total, up from 164)

New suite `tests/test-update-test-doc.mjs` (7 assertions):

- T1: `--check` passes when README is in sync
- T2: `--check` exits non-zero when README has drifted
- T3: drift report points the reader at the fix command
- T4: regenerate writes without error
- T5: `--check` passes after regeneration
- T6: generated README carries the do-not-edit banner
- T7: generated README does NOT hardcode an assertion count
  (the specific anti-pattern that caused four drifts)

First self-dogfood finding by the new test: adding
`test-update-test-doc.mjs` to the tests dir without an entry in the
suite-descriptions map produced drift. Generator caught its own
addition. Added the description and re-ran. Clean.

### Empirical verification the mechanism works

Before 1.9.0 tried to push a drifted README, `npm test` would exit 0
but the README would be wrong — reviewer catches it on the next
read. After 1.9.0, `npm run ci` now exits 1 if anyone forgets to run
`npm run update-test-doc` after a CHANGELOG change. The check is
authoritative — drift is mechanically impossible to ship.

### Meta observation, final form

Across 1.8.2 → 1.8.5 I apologized four times for the same class of
drift and each time said "filed for later." Four releases of deferral
on a pattern the reviewer flagged each release. That's not a drift
problem — that's a response-to-feedback problem. Lesson recorded:
when a reviewer catches the same shape of bug in successive reviews,
the correct response is to do the structural fix THAT release, not to
defer it while patching the specific instance. The structural fix
usually isn't much more work than the specific fix plus apology.

### Status

| Release | Assertions | Suites | CI gates |
|---|---|---|---|
| 1.8.5 | 164 | 13 | 4 |
| **1.9.0** | **171** | **14** | **5** |

### Known debt still carried forward

Unchanged from 1.8.5:
- Cross-process parse cache sharing (measure-topology / build-call-graph
  / check-barrel-discipline each re-parse independently)
- Rust tree-sitter extractor
- Python `__getattr__`-based lazy-export map detection
- `reExportsByFile` per-entry `typeOnly` for symbol-level public API
  precision
- Migration path from JS-style "dead export" messaging to Python's
  module-private-by-convention semantics in proposal text

## 1.8.5 — 2026-04-19

Review-driven precision patch, third in the review cycle. Reviewer
confirmed the v1.8.4 changes landed correctly, then filed four
precision issues — two release-blockers, two UX improvements. All four
closed here.

### 1. tests/README.md drift returned (third consecutive release)

v1.8.4 updated the top-of-file count (162 → 164) but left the per-release
"What the tests cover" retrospective section with `v1.8.3 (current)`
and no v1.8.4 entry. Exactly the same shape of drift the v1.8.4
CHANGELOG apologized for.

Fixed: v1.8.3 loses `(current)`, v1.8.4 gets its own bullet, v1.8.5
gets `(current)`. Pattern now obvious enough that it's documented
inline in the file as the release process.

Meta: **three consecutive releases caught the same doc-drift pattern
in one section or another**. Moving the retrospective to a generated
file (`npm run update-test-doc`) is the proper fix and is flagged for
a future patch. For now, release-process discipline.

### 2. test-type-only-reexport T6 loosened an invariant

The assertion was `files.length >= 5` but the adjacent comment listed 6
expected files. Pre-fix that meant: if `a.ts` silently disappeared from
`reExportsByFile` and a stray unrelated file leaked in, the test would
still pass. Defeats the point of an invariant check.

Tightened to an exact expected-set check:

```js
const expected = ['src/a.ts', 'src/b.ts', 'src/c.ts', 'src/d.ts',
                  'src/kind2.ts', 'src/kind3.ts'];
const missing = expected.filter((f) => !files.includes(f));
const extra = files.filter((f) => !expected.includes(f));
```

Failure localizes to the specific file that went missing or leaked.
Assertion count unchanged (still 6 total in this suite, 164 overall).

### 3. check-drift.mjs header contradicted its implementation

The header said it checks `tests/README.md assertion count` but the
implementation had an explicit `// Intentionally NOT checked here`
block. The script that guards against drift contained its own
header-vs-impl drift. Ironic.

Header rewritten to match what the script actually does: checks
`package.json`, `emit-sarif.mjs`, `CHANGELOG.md`, `package-lock.json`.
Deliberate non-check of `tests/README.md` is now explained inline with
the rationale (prose docs drift; `npm test` output is authoritative).

### 4. Test UX when parser dependency is missing

Running `npm test` on a bare checkout without `npm install` first
produced a confusing cascade: `test-alias.mjs` tried to run
`build-symbol-graph`, which threw `ERR_MODULE_NOT_FOUND` for
`oxc-parser`, but the test caught and discarded the error — then
failed on the subsequent `readFileSync(symbols.json)` with an opaque
`ENOENT`. Reviewer took more than a minute to diagnose.

Added `runChecked()` helpers in `test-alias.mjs` and
`test-type-only-reexport.mjs`. They surface the pipeline step's
stderr verbatim and suggest `npm install` when the error signature
matches. First-time maintainers now see:

```
[test-alias.mjs] pipeline step failed:
  cmd: node build-symbol-graph.mjs --root ... --output ...
  stderr: Cannot find package 'oxc-parser' imported from ...

Hint: if this is "Cannot find package 'oxc-parser'", run `npm install` first.
```

Verified by temporarily hiding `node_modules/oxc-parser` and observing
the improved output.

### Tests (164 total, unchanged)

No new assertions — T6 was tightened in place. The regression
guarantee is what strengthened, not the count.

### Status

| Release | Assertions | Suites |
|---|---|---|
| 1.8.3 | 162 | 13 |
| 1.8.4 | 164 | 13 |
| **1.8.5** | **164** | **13** |

### Meta observation (updated)

Across 1.8.2 → 1.8.3 → 1.8.4 → 1.8.5, reviewer has caught the same
meta pattern four times: **code fix correct, accompanying doc/test
claim looser than advertised**. The pattern says more about my release
discipline than about any individual fix. Actions taken in this patch:

1. Release process now explicitly includes "update tests/README.md
   retrospective section" — previously only the top count was updated.
2. Assertion comments and assertion bodies must match — prefer exact
   checks over threshold checks unless threshold is explicitly
   documented.
3. Header comments in guard scripts must match what the script
   actually does.

The right structural fix is to generate the doc retrospective from
`CHANGELOG.md` + the actual suite outputs, so the drift becomes
mechanically impossible. Filed for v1.9.0.

## 1.8.4 — 2026-04-19

Review-driven patch. Reviewer dogfooded v1.8.3 and confirmed the six
substantive items from the prior review were closed — then filed three
precision issues on the work itself. All three closed here.

### 1. tests/README.md drift returned

v1.8.3 fix for "README says 86/8" overshot: I wrote 155/12, but the
release itself shipped with 162/13. Reviewer caught it on the very
next read. **This is the second time in two releases I've drifted the
same document**, which is meta-feedback about review discipline, not
just a number fix.

Fixed to 164/13 (current). Updated the list of individual suites to
include `test-type-only-reexport.mjs`. Added a line saying `npm test`
is authoritative so future drifts are self-correcting.

### 2. test-type-only-reexport fixture was a star graph, not a cycle

The v1.8.3 fixture had five files all re-exporting from `types.ts`.
No cycles anywhere in the graph. The assertion
"runtime lens reports 0 SCCs" passed because the graph was acyclic
regardless of whether type-only filtering worked. In other words: the
test did not catch the regression it was supposed to guard.

Rewrote the fixture with two deliberate cycles:

- **`a.ts ⇄ b.ts`** via `export type { } from` both directions — a
  real type-only cycle. Pre-fix runtime lens would report SCC of
  size 2 including both files; post-fix the cycle is invisible at
  runtime (all edges `typeOnly: true`).
- **`c.ts ⇄ d.ts`** via mixed and pure runtime re-exports — a real
  runtime cycle. Both pre- and post-fix must keep the SCC; fix must
  not over-filter.

Verified empirically: reverting the fix in `measure-topology.mjs` makes
T2/T3/T4/T5 fail (4 of 6 red), confirming the test now catches the
actual regression. Restoring the fix returns all 6 to green.

### 3. T4 assertion vs comment mismatch

Previous T4 had the comment `// symbols.json should also mark these
uses as typeOnly` but the actual assertion only counted
`reExportsByFile` keys. The artifact does NOT currently propagate
`typeOnly` into `reExportsByFile` — that's a separate deeper fix with
mixed-form caveats.

Resolved by option (a) from reviewer's suggestion: the assertion and
comment now both say "tracks all re-exporting files". The deeper fix
(per-entry `typeOnly` in `reExportsByFile`) is explicitly flagged as
follow-up debt.

### Also cleaned up

Unused `langForFile` imports removed from `build-symbol-graph`,
`measure-topology`, `check-barrel-discipline` after the `parseOxcOrThrow`
migration internalized the call. `build-call-graph` keeps
`canContainJsx` (still used inline).

### Explicitly deferred follow-ups

From reviewer's "Issue 4" — public API barrel expansion:
`reExportsByFile` is file-level, so if `index.ts` has
`export type { X } from './internal'`, then all of `internal.ts` is
marked public API. This shields unrelated runtime exports in the same
file from the dead-export analysis. Design trade-off, not a bug —
protecting false positives at the cost of some recall. Moving to
symbol-level public API expansion is the proper fix and is a bigger
epic. Filed for later.

### Tests (164 total, up from 162)

Strengthened fixture means more assertions in `test-type-only-reexport`:

| Release | Assertions | Suites |
|---|---|---|
| 1.8.2 | 155 | 12 |
| 1.8.3 | 162 | 13 |
| **1.8.4** | **164** | **13** |

### Meta observation

Both reviews in the 1.8.3 → 1.8.4 cycle exposed the same pattern: the
code fix landed correctly, but the accompanying documentation, test
fixture, or assertion text was weaker than the claim in the CHANGELOG.
Pattern says: release discipline needs the same care as the fix
itself — the docs and the regression guard are part of the deliverable,
not tail polish. Adding that note for the next maintainer.

## 1.8.3 — 2026-04-19

Review-driven consolidation. An external reviewer dogfooded v1.8.2 and
filed six issues, each specific and reproducible. All six closed here.

### 1. Release drift: package-lock.json fell four versions behind

`package.json` read 1.8.2; `package-lock.json` still said 1.4.0 —
because `sed`-based bumps never touched the lockfile and we didn't run
`npm install` between releases. The drift guard missed it because it
only checked `package.json`, `emit-sarif.mjs`, and `CHANGELOG.md`.

Fixed the lockfile (manual JSON edit; authoritative fix is still
`npm install --package-lock-only`). Extended `scripts/check-drift.mjs`
to check both `lock.version` and `lock.packages[""].version`.

### 2. `measure-discipline.mjs` ignored common CLI flags

The SKILL.md documents `--include-tests`, `--no-include-tests`, and
`--exclude` as shared across every scanner. `measure-discipline` was
doing `collectFiles(root, { languages: langList })` — neither flag
propagated. So users running `--production` saw test-file smells
mixed in with production counts.

Fixed. Also dropped the `isPythonAvailable` / `isTreeSitterAvailable`
gates from this scanner: it's regex-only, parsers aren't needed to
count `panic(` or `# type: ignore`.

### 3. Type-only re-exports leaked into runtime topology

`measure-topology.mjs` correctly filtered `import type` edges in
runtime lens but didn't filter type-only re-exports:
`export type { X } from`, `export type * from`, and
`export { type X, type Y } from`. Same oversight in
`build-symbol-graph.mjs`'s re-export use emission (hard-coded
`typeOnly: false`).

Fixed. Re-export edges now carry `typeOnly` derived from
`node.exportKind === 'type'` (forms 1-2) or the all-specs-type-only
predicate (form 3). Mixed forms like `export { X, type Y } from` keep
the edge because X is still a runtime re-export.

Runtime lens at the Tarjan SCC boundary was already filtering
`typeOnly` edges, so the fix takes effect transparently.

### 4. `parseSync(...).errors[]` handling landed in only one of four callers

v1.8.2 fixed this for `build-symbol-graph` but left
`build-call-graph`, `measure-topology`, and `check-barrel-discipline`
still ignoring the errors array. Result: malformed files surfaced as
parse errors in one artifact and as silent empty-AST data in three
others.

Extracted to `_lib/parse-oxc.mjs` exporting `parseOxcOrThrow()`. All
four callers migrated to the helper. Error handling is now identical
across the pipeline.

### 5. Extension matrix only partially unified

`_lib/lang.mjs` knew all eight JS-family extensions since 1.8.0, but
`_lib/collect-files.mjs` default was still `['ts', 'tsx', 'js', 'mjs']`.
Scanners that took that default (`build-call-graph`,
`check-barrel-discipline`, `resolve-method-calls`) silently dropped
`.jsx`, `.cjs`, `.mts`, `.cts`. `triage-repo.mjs` counted TS as
`['ts', 'tsx']` only, missing `.mts` / `.cts`.

Added `JS_FAMILY_LANGS` and `TS_FAMILY_LANGS` constants to `_lib/lang.mjs`.
Widened the `collect-files` default to `JS_FAMILY_LANGS`. `triage-repo`
TS count now includes `.mts` / `.cts`. One source of truth; callers
that want a narrower subset still can by passing explicit `languages`.

### 6. tests/README.md stale (86/8 vs actual 155/12)

Updated to 155/12 as of 1.8.2 + per-release history, and documented
what the test suite does NOT cover so future maintainers know where
the guard rails stop.

### Tests (162 total, up from 155)

- `tests/test-type-only-reexport.mjs` (4 assertions): fixture with all
  4 syntactic forms; topology's `typeOnlyEdges` counts them; runtime
  lens reports 0 SCCs when the only cycles are type-only.
- `tests/test-smoke-uncovered.mjs` F section (3 assertions): drift
  guard exits 0 on a clean synthetic repo, non-zero when lockfile is
  out of sync, and names `package-lock.json` in the report.

Total test count progression:

| Release | Assertions | Suites |
|---|---|---|
| 1.8.2 | 155 | 12 |
| **1.8.3** | **162** | **13** |

### What wasn't reviewed but remains known

- `__getattr__` lazy-export Python idiom in `__init__.py` — still
  deferred.
- Rust tree-sitter extractor — still deferred.
- Cross-script parse cache sharing — still deferred; separate epic.

## 1.8.2 — 2026-04-19

Performance + hardening release. Closes feedback item #5 (silent
catches + perf). Three deliberate improvements plus one incidental bug
found along the way.

### Performance: line-offset table

`getNodeLine` in build-symbol-graph was O(N × avg_offset) — each AST
node triggered a scan from byte 0 counting `\n`. On a 3k-LOC file with
hundreds of nodes this produced tens of millions of inner-loop
iterations. Replaced with a precomputed `lineStarts[]` array + binary
search: O(L) once + O(log L) per query.

A near-identical helper already existed (uncommented) in
`check-barrel-discipline.mjs`. Both callers now import from a shared
`_lib/line-offset.mjs` module.

Measured on ouroboros (244 .py files, ~7K LOC equivalent, cold cache):

| | Baseline | 1.8.2 |
|---|---|---|
| Cold run | 6073 ms | 4387 ms |

Net ~28% reduction. Warm/incremental runs see less improvement
because the parse cache short-circuits `getNodeLine` calls entirely.

### Hardening: silent catches 25 → 0

Every `catch {}` in the codebase audited. Outcomes:

- **14 sites in `_lib/resolver-core.mjs`** — all the same
  `try { statSync(p).isFile() } catch {}` probe pattern. Replaced with
  a small helper trio in `_lib/paths.mjs`:
  `fileExists()`, `dirExists()`, `pathExists()`.
- **1 site in `_lib/alias-map.mjs`** — same pattern, same helper.
- **10 sites elsewhere** (`triage-repo`, `_lib/python`, `_lib/collect-files`,
  `build-symbol-graph`, `_lib/tree-sitter-langs`, `_lib/repo-mode`,
  `_lib/classify-policies`) — genuinely intentional swallows (missing
  package.json, unreadable yaml, race on readdir). Each annotated
  with a comment explaining what's being swallowed and why.

ESLint: `no-empty` upgraded from `allowEmptyCatch: true` to
`allowEmptyCatch: false`. The rule now enforces the discipline —
any new empty catch will fail `npm run lint`.

### Hardening: explicit failure recording

Artifacts now carry structured warnings under `meta.warnings[]`
instead of dumping non-fatal failures to stderr only:

```json
{
  "meta": {
    "generated": "2026-04-19T08:45:00Z",
    "root": "...",
    "tool": "build-symbol-graph.mjs",
    "warnings": [
      { "code": "parse-errors", "count": 3,
        "message": "3 file(s) failed to parse; ..." }
    ]
  }
}
```

Populated for:
- `python-ndjson-parse-failure` — stray non-JSON in extractor stdout
- `python-batch-crashed` — subprocess fault
- `tree-sitter-batch-crashed` — parser initialization failure
- `parse-errors` — files oxc-parser rejected (now escalated; see below)

`emit-sarif.mjs` reads each upstream artifact's `meta.warnings[]` and
surfaces them under the SARIF `runs[0].properties.upstreamWarnings`
field. CI dashboards and human reviewers now see at a glance whether
findings came from a clean scan or a partial one.

### Incidental bug fixed along the way

`extractDefinitionsAndUses` in build-symbol-graph was ignoring
`parseSync.result.errors[]`. oxc-parser does NOT throw on syntactic
errors — it returns them in an array and hands back whatever AST it
could salvage. So malformed files silently contributed empty def/use
lists to the graph, making every symbol in them look dead and
depriving `parseErrors` of their count. Now we throw on any reported
error, which the outer try/catch records into `warnings[]`.

### Tests (155 total, up from 151)

Added 4 assertions to `tests/test-smoke-uncovered.mjs`:
- D"1: `symbols.json.meta.warnings` is always an array
- D"2: a parse error surfaces as a structured warning (not stderr only)
- D"3: clean scan produces `warnings: []` (no spurious entries)
- D"4: SARIF propagates upstream warnings under
  `properties.upstreamWarnings`

### Status vs feedback items

| Item | Status |
|---|---|
| #1 resolver 과밀 | ✅ 1.5.0 |
| #2 uncovered + CI/lint | ✅ 1.4.0 + 1.6.0 |
| #3 classifier 휴리스틱 | ✅ 1.7.0 |
| symlink aliasing | ✅ 1.7.1 |
| Python convention FP | ✅ 1.7.2 |
| #4 언어 매트릭스 | ✅ 1.8.0 |
| SARIF 정책 누수 | ✅ 1.8.1 |
| **#5 성능/하드닝** | ✅ **1.8.2** |

All five original feedback items plus three additional dogfood
findings now fully closed.

### Known remaining debt

- Parse cache sharing across scripts — each of measure-topology,
  build-call-graph, check-barrel-discipline re-parses from scratch.
  A shared parse-manager module is a bigger refactor; flagged for
  future work.
- `__getattr__`-based lazy-export detection in Python `__init__.py`
  files — still misses ~5 FPs in the ouroboros C bucket.
- Rust tree-sitter extractor for repos like `crates/*`.
- Sync-to-async I/O conversion — evaluated and deferred. Parse
  (ast walk + resolver loop) dominates; I/O is not the bottleneck.

## 1.8.1 — 2026-04-19

Self-dogfood patch. Ran the full pipeline against the tool's own source
and uncovered two things: one real dead export in our own code, and one
legitimate tool bug that the existing tests didn't catch.

### Dead code in our own source

`_lib/tree-sitter-langs.mjs:77 treeSitterLanguages()` is exported but
has zero cross-file consumers — the only mention outside the definition
is a comment in the module's own docstring. Classifier correctly buckets
it as A (demote-to-internal). This is the first self-identified debt.
Left in place for now with a comment; removal or `// @internal` marking
is a separate decision.

### Tool bug fixed: SARIF bypassed the classifier policy layer

`emit-sarif.mjs` read `symbols.json.deadProdList` directly and ignored
`dead-classify.json` entirely. Consequence: all the framework-exclusion
policies that `classify-dead-exports.mjs` applies (FP-22 config files,
FP-23 public API, FP-25 transitive barrels, FP-27 framework sentinels,
FP-30 Nuxt/Nitro) were **invisible in the SARIF output**. A Next.js
`app/page.tsx` correctly excluded by the classifier would still appear
in SARIF. `eslint.config.mjs` would still appear in SARIF. Python
`__all__`-gated names were fine because that filter runs upstream at
build-symbol-graph, but everything else leaked.

### Fix

Priority chain in `emit-sarif.mjs` is now:

1. `runtime-evidence.json` + optional `staleness.json` — richest signal
2. **`dead-classify.json`** — policy-filtered, classifier-bucketed
3. `symbols.json.deadProdList` — last-resort pre-policy fallback

When using the classifier output, SARIF `properties.proposalBucket`
carries which bucket (C / A / B / specifier) the symbol came from, the
severity maps appropriately (C / A → `warning`; B / specifier → `note`),
and the message text carries the human-readable action from the
classifier.

Self-dogfood output:

| Source | Dead findings in SARIF |
|---|---|
| v1.8.0 raw symbols.json | 2 (`treeSitterLanguages`, `eslint.config default` — the second is a FP-22 false positive) |
| v1.8.1 dead-classify.json | 1 (`treeSitterLanguages` — real dead code) |

### Tests (151 total, up from 147)

Added 4 assertions to `tests/test-smoke-uncovered.mjs`:
- D'1. full pipeline (build-symbol-graph → classify → emit-sarif) exits 0
- D'2. SARIF emits the real dead export (`genuinelyUnused`)
- D'3. SARIF does NOT emit the `eslint.config.mjs` default — proves FP-22
  propagation end-to-end
- D'4. `proposalBucket` field present in SARIF properties — evidence
  the classifier output is the source

Pre-1.8.1 D'3 would have failed.

## 1.8.0 — 2026-04-19

Language matrix release. Closes feedback item #4. Surfaced via dogfood on
a pure-JSX fixture which pre-1.8.0 scanned as 0 files / 0 defs / 0 dead.

### Bugs fixed

1. **JS-family parser mode was wrong for 6 of 7 extensions.** Four scripts
   (`build-call-graph`, `build-symbol-graph`, `check-barrel-discipline`,
   `measure-topology`) did `filePath.endsWith('.tsx') ? 'tsx' : 'ts'` at
   each `parseSync` call. Everything non-.tsx was forced into TS mode.
   Pure `.jsx` files produced parse errors ("Unterminated regular
   expression") and returned empty def/use lists — so every symbol in
   them looked dead to cross-file analysis. `.js` / `.mjs` / `.cjs` /
   `.mts` / `.cts` were silently tolerated by TS's syntactic generosity
   but edge cases leaked.

2. **Default language collection list was narrow.** Two scripts had
   `const langList = ['ts', 'tsx', 'js', 'mjs']`. Pure-JSX repos and
   `.cjs`-heavy projects weren't even walked — `build-symbol-graph` on
   a JSX fixture reported `files: 0`. Widened to the full
   `['ts', 'tsx', 'mts', 'cts', 'js', 'jsx', 'mjs', 'cjs']`.

### Fix

New module `_lib/lang.mjs` exporting three small helpers:

```js
langForFile(path)       → 'ts' | 'tsx' | 'js' | 'jsx' | null
canContainJsx(path)     → boolean  (.tsx / .jsx only)
nonJsLangForFile(path)  → 'python' | 'go' | null
```

The four parse sites now dispatch per-file via `langForFile(path) ?? 'ts'`.
`build-call-graph`'s JSX-aware element-detection branch uses
`canContainJsx()` instead of inlined `endsWith` checks.

Before / after on minimal JSX fixture:

| | v1.7.2 | v1.8.0 |
|---|---|---|
| files walked | 0 | 7 |
| totalDefs | 0 | ≥7 |
| parse errors on .jsx | 2 per file | 0 |

### Tests (147 total, up from 123)

New suite `tests/test-lang-matrix.mjs` with 24 assertions:

- **Unit (L1-L17)**: each extension maps to the right lang value, dunders
  and edge cases covered, `canContainJsx` and `nonJsLangForFile` branches.
- **Integration (I1-I7)**: mixed-extension fixture (`.tsx` / `.jsx` /
  `.ts` / `.mts` / `.cts` / `.mjs` / `.cjs`) walked end-to-end through
  `build-symbol-graph`. All 7 files seen, parses clean, dead-list has
  correct members (cross-file uses suppress false-dead; truly unused
  symbols appear).

### Status vs feedback items

| Item | Status |
|---|---|
| #1 resolver 과밀 | ✅ 1.5.0 |
| #2 uncovered + CI/lint | ✅ 1.4.0 + 1.6.0 |
| #3 classifier 휴리스틱 | ✅ 1.7.0 |
| symlink aliasing | ✅ 1.7.1 |
| Python convention FP | ✅ 1.7.2 |
| **#4 언어 매트릭스** | ✅ **1.8.0** |
| #5 성능/하드닝 | ⏳ partial (47 silent catches + perf cold spots) |

4 of 5 review items fully closed; the last one (`#5`) is ongoing and
partial work is already in (ESLint `no-empty` is on, just with
`allowEmptyCatch: true` as tracked debt).

## 1.7.2 — 2026-04-19

Python convention release. Surfaced by dogfooding v1.7.1 against a real
Python monorepo (ouroboros) and comparing the output against a manual
audit. Three separate bugs inflated the dead-export list from 35 real
candidates to 166. Each is fixed in a targeted way with no effect on
JS/TS behavior.

### Bugs fixed

1. **Python self-reference import resolution.** `resolvePythonImport`
   probed `/pkg/pkg/x.py` when given `--root /pkg` and spec `pkg.x`,
   treating the leading package segment as a subdirectory. This
   falsely marked every symbol imported by fully-qualified name as
   unreachable — e.g. `load_agent_prompt` in ouroboros flagged as dead
   despite being imported from five files. Mirror of the TypeScript
   FP-16 self-reference branch. Fix: when `level === 0` and
   `parts[0] === path.basename(root)`, probe once with the leading
   segment stripped.

2. **`__all__` now respected as explicit export declaration.** Python's
   own import semantics: when a module declares `__all__ = [...]`,
   `from m import *` only pulls in listed names. Our dead-export
   analysis ignored this and reported every top-level name as a
   candidate. Fix: extractor parses the `__all__` literal and emits a
   `pyDunderAll` field on the file record; dead-list gate skips any
   symbol not in the list. Also: `__all__` itself no longer appears as
   a def (it's Python syntax, not a user symbol).

3. **Framework-registered decorated functions skipped.** `@app.command()`
   (Typer / Click), `@app.route()` (Flask / FastAPI), `@task` (Celery),
   `@fixture` (pytest), and related decorators mean the framework
   invokes the function by dispatch, not by JS-style import + call.
   Extractor detects these patterns and marks defs with
   `frameworkRegistered: true`; dead-list gate skips them.
   Architecturally parallel to the FP-27 framework-sentinel policy for
   Next.js / SvelteKit on the JS side.

4. **Python dunder methods excluded at extraction.** `__getattr__`,
   `__dir__`, `__init__`, `__enter__`, `__call__`, `__iter__`, and
   friends are runtime protocol methods — Python invokes them by
   convention, never by user-facing name lookup. These no longer
   enter the def list at all (not just the dead list), since they're
   not user-exportable symbols.

### Dogfood impact (ouroboros-main, 244 .py files)

| Measurement | v1.7.1 | v1.7.2 |
|---|---|---|
| Dead production candidates (build-symbol-graph) | 1,964 | **693** |
| Category C (full removal recommended) | 166 | **33** |
| Category A (drop export) | 720 | 436 |
| Category B (design review) | 329 | 224 |

C-bucket dropped 80%. The remaining 33 candidates were spot-checked and
are either genuinely dead (`clear_cache`, `parse_jsonc`) or documented
as residual Python patterns we don't yet handle (`__getattr__`-based
lazy-export maps in `__init__.py` — requires string-literal scanning,
flagged as future work).

### New tests (123 total, up from 110)

New suite `tests/test-python-conventions.mjs` with 13 assertions:

- **A. Self-reference** (2): absolute import via package name resolves;
  undecorated consumer still appears when it has no cross-file use.
- **B. `__all__`** (4): `__all__` itself not in dead list; listed names
  are candidates; unlisted top-level names are module-private.
- **C. Decorator registration** (4): `@app.command()`,
  `@app.command(name=...)`, `@app.callback()` skip the dead list;
  undecorated function still enters it.
- **D. Dunders** (3): `__getattr__`, `__dir__` never enter defs;
  regular functions do.

Suite skips cleanly if `python3` isn't on `$PATH` (required by the
extractor subprocess).

### Known debt still carried

- `__getattr__`-based lazy-export maps in `__init__.py` files — common
  pattern in Python packages that want to expose a shallow API without
  eager-loading submodules. Text scan for string literals matching
  symbol names would catch these. Flagged for future work.
- Rust crate extractor still not registered (ouroboros ships one;
  blind on `crates/ouroboros-tui/`). Separate epic.
- `langForFile()` helper for parser-mode dispatch — feedback #4, not
  yet addressed.

### Status vs feedback items

| Item | Status |
|---|---|
| #1 resolver 과밀 | ✅ 1.5.0 |
| #2 uncovered + CI/lint | ✅ 1.4.0 + 1.6.0 |
| #3 classifier 휴리스틱 | ✅ 1.7.0 |
| symlink aliasing | ✅ 1.7.1 |
| **Python conventions** (dogfood finding) | ✅ **1.7.2** |
| #4 언어 매트릭스 | ⏳ |
| #5 성능/하드닝 | ⏳ partial |

## 1.7.1 — 2026-04-19

Patch release. Fixes a symlink-aliasing bug that caused falsely-dead
symbol reports in repos using vendored symlinks or dir-symlink workspace
layouts.

### Bug

`collectFiles` skips symlinks and walks canonical paths only, so the
symbol graph was keyed by realpath. But `makeResolver` returned the
symlink path it found first (`src/lib.ts` for a symlink target
`../vendored/lib.ts`). Downstream consumers (classify-dead-exports,
resolve-method-calls, build-call-graph) then failed to match the
resolved path against any symbol-graph key and reported used symbols
as dead.

Minimal repro:

```
repo/
├── package.json
├── src/
│   ├── lib.ts -> ../vendored/lib.ts     # symlink
│   └── app.ts                           # imports './lib.js'
└── vendored/
    └── lib.ts                           # exports vendoredValue
```

Before 1.7.1: `vendoredValue` classified as dead even though `app.ts`
uses it. After: resolver returns `vendored/lib.ts` for the `./lib.js`
import, matching the symbol graph's canonical key.

### Fix

`_lib/resolver-core.mjs` wraps its inner resolver with a canonicalizing
outer closure that runs every returned file path through `realpathSync`.
`null` and the `'EXTERNAL'` sentinel pass through unchanged. Results
are memoized in a per-resolver `realpathCache` so the same path isn't
stat'd repeatedly across the many call sites in a typical audit run.

### Tests (110 total, up from 104)

New suite `tests/test-symlink-aliasing.mjs` with 6 assertions covering:

- file-symlink → realpath (T1, T2)
- directory-symlink with `/index.ts` lookup → realpath (T3)
- `null` and `'EXTERNAL'` pass through unchanged (T4, T5)
- non-symlinked relative imports unchanged (T6)

## 1.7.0 — 2026-04-19

Structural release. `classify-dead-exports.mjs` (517 LOC — the second
feedback hotspot: accumulating framework exceptions, text-regex occurrence
counting, aliased-export special case all in one file) splits into three
layers: **fact extraction**, **policy rules**, **orchestration**. Public
behavior is unchanged — `dead-classify.json` keeps the same schema, all
104 test assertions pass.

### New modules

| Module | LOC | Purity | Role |
|---|---|---|---|
| `_lib/classify-facts.mjs` | 96 | Pure | `countOccurrencesExceptDefLine`, `countExcludingDeclAndExport`, `hasPredicatePartner`, `isAliasedSpec` |
| `_lib/classify-policies.mjs` | 118 | Pure | `isConfigFile`, `isCoreSentinel`, `isNuxtNitroSentinel`, `detectNuxtNitro` — plus the `CONFIG_PATTERNS`, `FRAMEWORK_SENTINEL_BASENAMES` tables |
| `classify-dead-exports.mjs` | 304 | Stateful | Orchestration: read `symbols.json`, build the public-API file set (FP-23/25), apply policy filters, extract facts, categorize, emit the proposal artifact |

The two `_lib/classify-*` modules are side-effect free — no I/O, no
global state, no CLI parsing. They take plain strings and return plain
booleans / numbers. This is the fact/policy split the review asked for.

### Why this matters

The old 517-LOC file conflated three concerns:

1. **"Is this symbol dead in the source?"** (facts — counting occurrences,
   finding partners, detecting aliasing).
2. **"Does a framework convention exempt this file from the count?"**
   (policies — Next.js app router, SvelteKit, Nuxt/Nitro, config files).
3. **"Given the facts and the policies, what do we tell the user?"**
   (orchestration — building the public-API set, reading symbols.json,
   looping and categorizing, writing the report).

When feedback hits like "add support for Remix's filesystem routing" or
"the occurrence counter is miscounting inside template literals", the
change used to ripple through the whole 517-LOC decision tree. Now:

- New framework support → edit `classify-policies.mjs` only.
- Better occurrence counting (e.g., AST-based replacement) → edit
  `classify-facts.mjs` only.
- New category bucket or artifact shape → edit the orchestrator only.

### Known debt still carried (unchanged from 1.6.0)

- Occurrence counting in `classify-facts.mjs` is still regex-based over
  source text. An AST-identifier-reference replacement would eliminate
  false matches in strings / comments / JSDoc. The split makes this
  swap local — the fact module is 96 lines of pure functions with a
  clean public interface, so the replacement is a targeted rewrite.
- 47 silent catches. ESLint `no-empty` still runs with
  `allowEmptyCatch: true`.
- Language parser mode is still `tsx` or `ts` hard-coded in several
  scripts; `.jsx` / `.mjs` / `.cjs` get parsed in TS mode.

### Status vs feedback items

| Item | Status |
|---|---|
| #1 resolver 과밀 | ✅ 1.5.0 (737 → 23 LOC facade) |
| #2 uncovered 5 + no CI/lint | ✅ 1.4.0 + 1.6.0 |
| #3 classifier 휴리스틱 | ✅ **1.7.0** (517 → 304 + two pure modules) |
| #4 언어 매트릭스 | ⏳ Next |
| #5 성능/하드닝 | ⏳ Partial |

## 1.6.0 — 2026-04-19

Coverage release. Closes feedback item #2 — five scripts that previously
had zero automated coverage now have smoke tests. Total assertions:
86 → 104 across 9 suites.

### Added

New suite `tests/test-smoke-uncovered.mjs` covering the five scripts
that had no tests in 1.5.0:

| Script | Assertions | What's checked |
|---|---|---|
| `build-call-graph.mjs` | 3 | exits 0, produces `call-graph.json`, parseable with recognized top-level shape |
| `check-barrel-discipline.mjs` | 3 | exits 0, produces `barrels.json`, parseable with recognized shape |
| `measure-discipline.mjs` | 3 | exits 0, produces `discipline.json`, parseable with recognized shape |
| `emit-sarif.mjs` | 6 | exits 0 on zero upstream artifacts, produces valid SARIF 2.1.0 with populated `tool.driver` — closes the loop with the drift guard |
| `merge-runtime-evidence.mjs` | 3 | exits 0 with minimal symbols + Istanbul-shape coverage input, produces `runtime-evidence.json` |

The SARIF block (D4-D6) verifies the artifact carries the `TOOL_VERSION`
that `scripts/check-drift.mjs` enforces consistency on. If someone bumps
`package.json` without updating `emit-sarif.mjs`, now **two** CI gates
fail (drift check + smoke test), making the miss harder.

### Intentionally shallow

These are smoke tests by design. Each script's deeper semantics deserve
a dedicated suite, but they're not here — the goal was to establish a
coverage floor so a totally broken script doesn't ship silently.
Subsequent PRs can extend assertions per-script.

### Status vs feedback items

| Item | Status |
|---|---|
| #1 resolver 과밀 | ✅ Resolved in 1.5.0 (737 → 23 LOC facade, 7 modules) |
| #2 uncovered 5 + no CI/lint | ✅ Resolved in 1.4.0 + 1.6.0 |
| #3 classifier 휴리스틱 | ⏳ Carried over — next epic |
| #4 언어 매트릭스 | ⏳ Carried over |
| #5 성능/하드닝 | ⏳ Carried over (partial — 47 silent catches still flagged as debt) |

## 1.5.0 — 2026-04-19

Structural release. `_lib/resolver.mjs` (737 LOC — the regression hotspot
that caused the 1.3.0 → 1.3.1 fix cycle) is split into seven focused
submodules. Public API is unchanged: every existing
`import { … } from './_lib/resolver.mjs'` continues to work via a
re-export facade. All 86 test assertions pass unchanged.

### Module layout

| Module | LOC | Responsibility |
|---|---|---|
| `_lib/cli.mjs` | 71 | `parseCliArgs`, negation flag set, bool coercion |
| `_lib/repo-mode.mjs` | 149 | `detectRepoMode`, pnpm YAML parser, workspace enumeration |
| `_lib/alias-map.mjs` | 181 | `buildAliasMap`, `mapOutputToSource`, exports resolution |
| `_lib/resolver-core.mjs` | 160 | `makeResolver`, `RESOLVE_FILE_EXTS`/`RESOLVE_INDEX_EXTS` |
| `_lib/collect-files.mjs` | 118 | `collectFiles` walker + pruning + exclude patterns |
| `_lib/test-paths.mjs` | 29 | `isTestLikePath` classifier |
| `_lib/paths.mjs` | 13 | `relPath` utility |
| `_lib/resolver.mjs` | 23 | Re-export facade (backward compat) |

### Why

The old 737-LOC `resolver.mjs` did nine distinct things:

- CLI parsing
- pnpm workspace YAML parsing
- Repo mode detection
- Alias map construction
- Package exports resolution
- Output → source path remapping
- Specifier resolution
- File collection
- Test-path classification

Every one of the seven correctness bugs fixed in 1.2.0 lived in this file.
The 1.3.0 → 1.3.1 regression (narrow extension probe table) was a direct
consequence of the density — it was hard to see that the relative-path
branch and the wildcard branch shared extension tables that needed to
stay in sync. Splitting them out makes the next bug cheaper to catch
and cheaper to fix: changes to `alias-map.mjs` can't accidentally break
`collect-files.mjs` the way two adjacent functions in one file could.

### Migration

None required for consumers — the facade re-exports every public symbol
that used to live directly in `resolver.mjs`. New code may optionally
import from the specific submodule for a tighter dependency declaration:

```js
// Both work, both resolve to the same implementation:
import { buildAliasMap } from './_lib/resolver.mjs';     // backward-compat facade
import { buildAliasMap } from './_lib/alias-map.mjs';    // direct import (new)
```

### Known debt (carried over from 1.4.0)

- 47 silent catches remain (`_lib/resolver-core.mjs` has the largest
  concentration now that they've been divided up). ESLint `no-empty`
  still runs with `allowEmptyCatch: true`. Next target: audit each and
  either add a `// intentional` comment, log at verbose level, or
  re-throw typed.
- 5 scripts still lack smoke tests (`build-call-graph`,
  `check-barrel-discipline`, `emit-sarif`, `measure-discipline`,
  `merge-runtime-evidence`).

## 1.4.0 — 2026-04-19

Infrastructure release. No behavior changes to audit logic; adds the
tooling layer that prevents the class of drift we hit between 1.2.0 and
1.3.1. Version/doc drift, silent catches, unused imports, and missing CI
gates were each called out in review.

### Added

- **ESLint flat config** (`eslint.config.mjs`). Rules: `no-undef`,
  `no-unused-vars` (warn, allows `_`-prefixed), `no-empty` (error,
  `allowEmptyCatch: true` as tracked debt), `no-const-assign`,
  `no-var`, `prefer-const` (warn), plus the other standard correctness
  checks from `eslint:recommended`. Tests get a looser regime
  (no `no-unused-vars` — fixture construction is intentionally noisy).
- **Drift guard** (`scripts/check-drift.mjs`). Enforces that
  `package.json` `version`, `emit-sarif.mjs` `TOOL_VERSION`, and the
  top entry in `CHANGELOG.md` all agree. Prints a table of mismatches
  and exits 1 on drift. Runs in CI. This catches exactly the class of
  miss that landed the `TOOL_VERSION = '0.6.8'` staleness in 1.3.1.
- **`npm run ci`** aggregate: `check` (node --check) → `check:drift` →
  `lint` → `test`. Fails fast on the first gate that breaks. This is
  what GitHub Actions invokes.
- **GitHub Actions workflow** (`.github/workflows/ci.yml`). Matrix on
  Node 20.x / 22.x. Runs `npm ci && npm run ci` on push / PR.
- **`devDependencies.eslint ^9.14.0`**.

### Fixed (drift piggyback)

- `emit-sarif.mjs` `TOOL_VERSION` was stuck at `'0.6.8'` through several
  releases. Now tracked by the drift guard.
- `tests/README.md` no longer claims an outdated assertion count; reworded
  to span the 1.2.0 → 1.3.1 range (86 assertions across 8 suites).
- Handful of unused imports / destructures removed so the baseline lint
  run is `0 problems` clean: future PRs see new violations against a
  green floor.

### Known technical debt (flagged for future releases)

- `no-empty` is configured with `allowEmptyCatch: true`. The codebase has
  47 silent catches, concentrated in `_lib/resolver.mjs` (21). Each
  should eventually either (a) log at verbose level, (b) re-throw a
  typed error, or (c) carry an explicit `// intentional` comment. This
  is the first target once `_lib/resolver.mjs` is split up.
- `_lib/resolver.mjs` remains at ~737 LOC and is the regression hotspot
  that caused the 1.3.0 merge → 1.3.1 fix cycle. Planned split:
  `cli.mjs` / `repo-mode.mjs` / `alias-map.mjs` / `resolver-core.mjs` /
  `collect-files.mjs` / `test-paths.mjs`.
- Scripts without test coverage: `build-call-graph.mjs`,
  `check-barrel-discipline.mjs`, `emit-sarif.mjs`,
  `measure-discipline.mjs`, `merge-runtime-evidence.mjs`. Each needs at
  minimum a smoke test asserting the artifact is produced and parses
  as JSON.
- Parser mode is still `tsx` or `ts` in several scripts; `.jsx` / `.mjs`
  / `.cjs` files get parsed in TS mode. A `langForFile()` helper in
  `_lib/` would unify this.

## 1.3.1 — 2026-04-19

Patch release. Fixes a regression introduced in the 1.3.0 merge where the
resolver's relative-path extension probe was narrower than the parallel
patch it was merged with.

### Regression fixed

- **Extension-less relative imports now resolve across the full extension
  set.** `./mod` → `mod.cjs`, `./view` → `view.jsx`, `./util` → `util.mts`,
  `./conf` → `conf.cts`, `./dir` → `dir/index.js`, `./cjs-dir` →
  `cjs-dir/index.cjs` — all of these returned `null` in 1.3.0, leaving
  unresolved edges in the audit and inflated blind counts for JS-flavored
  repos. The fix introduces module-scope `RESOLVE_FILE_EXTS` and
  `RESOLVE_INDEX_EXTS` tables covering `.ts`/`.tsx`/`.js`/`.jsx`/`.mjs`/
  `.cjs`/`.mts`/`.cts` + symmetric `/index.*` variants, and applies them
  in three places: the relative-path branch of `makeResolver`, the
  `tryResolveFromRoot` helper used by the root-prefix interpretation, and
  the wildcard-lookup directory fallback.

### Unaffected in 1.3.0 (already correct despite reviewer concern)

- `--no-include-tests` scans on fixtures with `tests/helper.ts` correctly
  dropped the file — the `isTestLikePath` helper merged in 1.3.0 was
  already doing directory-segment filtering.
- JS-only method-call analysis (`A.foo()` against `.js` sources) tracked
  the internal edge correctly — the `allowJs: true` / `checkJs: false`
  merge in 1.3.0 was already in effect.

### Tests (86 total, up from 77)

New suite `tests/test-resolver-paths.mjs` with 9 assertions covering
extension-less resolution, explicit-extension preservation, and the
null return for truly missing relative specs. Previous 77 assertions
still pass unchanged.

## 1.3.0 — 2026-04-19

Merge release combining the v1.2.0 fixes with improvements from a parallel
patch submitted against the same 1.1.0 baseline. Net gain over either lone
version: 9 additional test assertions and five strict-improvement items.

### Pulled in from the parallel patch (strict improvements)

- **`resolve-method-calls.mjs` is now JS-aware.** `allowJs: true` +
  `checkJs: false` in both the fallback options and the tsconfig-merge
  override. Previously the scanner collected `.js` / `.mjs` / `.cjs`
  files but tsc refused to parse them, producing empty `propSymbol`
  declarations and skewed resolution rates.
- **Root prefix collision fixed.** `declFileNorm.startsWith(rootNorm)` now
  guards against `/repo` spuriously matching `/repo-other/...` by requiring
  a trailing slash (equality with `rootNorm` is still accepted).
- **`focusClassReport` is now in the JSON artifact.** `level2-methods.json`
  exposes `{className, methods, totalCalls}` when `--focus-class` is set,
  `null` otherwise — machine-readable, no need to parse console output.
- **`--exclude-tests` accepted as another negation alias.** Union of the
  two patch sets; all five forms now work: `--no-include-tests`,
  `--no-tests`, `--exclude-tests`, `--production`, `--include-tests=false`.
- **`isTestLikePath` promoted to an exported helper.** `_lib/resolver.mjs`
  now exports a shared test-path classifier covering JS/TS (`*.test.*` /
  `*.spec.*`), Python pytest (`test_*.py`, `*_test.py`), Go (`*_test.go`),
  and directory-segment conventions (`test`, `tests`, `__tests__`, `e2e`,
  `integration`, `fixtures`, `mocks`). `triage-repo.mjs` delegates to it
  so `shape.testFiles` and `--no-include-tests` agree on classification.
- **`measure-staleness.mjs` blame line coercion.**
  `Number.isFinite(Number(line)) ? Number(line) : 1` — defensive against
  non-numeric line values in the input symbols.json.
- **`exclude: cli.exclude` plumbed through triage.** All four `collectFiles`
  calls and the per-subdir shape walk now respect user-supplied `--exclude`
  patterns. Previously triage ignored them.

### Behavior change worth flagging

`--no-include-tests` now also excludes any file under a `tests/` /
`__tests__/` / `e2e/` / `integration/` / `fixtures/` / `mocks/` directory
segment — even without a test-suffixed filename. Repos with non-test
helper code inside `tests/` (e.g. `tests/mock-server.ts`) will see those
files disappear from production-only scans. This matches the original
user feedback that `/tests/` paths were leaking into production counts.

### New tests (77 total, up from 68)

- `test-cli.mjs` gains the `--exclude-tests` alias assertion and six
  `isTestLikePath` assertions (including a false-positive guard for
  substrings like `contest`).
- `test-hardcoding.mjs` gains T7 and T8 verifying `focusClassReport` in
  the JSON artifact (present with className when flag set, null otherwise).

## 1.2.0 — 2026-04-19

Correctness fixes across seven areas. All changes come with regression tests
(see `tests/` folder). 68/68 assertions pass after these fixes.

### File collection (`_lib/resolver.mjs`)

- **Language filter bug fixed.** `collectFiles({ languages: ['py'] })` on a
  repo with root-level `.mjs` / `.ts` files previously leaked those into the
  result. Symmetrically, root-level `main.py` / `main.go` were silently
  dropped from single-language scans because the root-entry collector
  hardcoded JS/TS extensions. Now honors `languages` uniformly.
- **Test-file filter is language-aware.** Previously the filter regex was
  JS/TS-only (`*.test.ts` / `*.spec.js`). Added Python (`test_*.py`,
  `*_test.py`) and Go (`*_test.go`) conventions.

### CLI parsing (`_lib/resolver.mjs`)

- **`--no-include-tests` / `--no-tests` / `--production` now work.** The
  previous `--include-tests` boolean option could not be negated: `--no-foo`
  syntax requires Node ≥22.4, and `--include-tests=false` with `strict:false`
  stored the string `"false"` — which is truthy in JS, so the flag had the
  *opposite* of the intended effect. Now pre-scans argv for negation forms
  and coerces string booleans.

### Topology (`measure-topology.mjs`)

- **Dynamic imports (`import('./x')`, `await import(...)`, inline arrows,
  conditionals) are now tracked** as runtime edges. Previously only
  top-level `ImportDeclaration` and re-export nodes were read, so lazy
  routes / plugins / dynamic cycles were invisible to SCC analysis. Walker
  now recurses the full AST subtree, matching the existing behavior in
  `build-symbol-graph.mjs`.

### Package exports resolution (`_lib/resolver.mjs`)

- **Subpath wildcards now resolve.** `"./features/*": "./src/features/*.ts"`
  and similar patterns (including nested `"./ui/components/*"` and
  suffix-bearing `"./sub/*.js"`) now register and match correctly. Only
  `"./*"` was handled before — and even that was broken by a `startsWith`
  check against a literal `*`, so in practice internal imports were
  classified as EXTERNAL, producing dead-export false positives.
- Most-specific match wins when multiple patterns apply.
- `dist/` → `src/` remapping (via `mapOutputToSource`) and `.js` → `.ts`
  extension swap now apply in the wildcard branch.

### Parameterization (`classify-dead-exports.mjs`, `resolve-method-calls.mjs`)

- **Workspace-derived package labels.** The per-package × category
  breakdown used to hardcode four specific paths (`packages/protocol/`,
  `packages/shared-utils/`, `apps/daemon/`, `apps/web-shell/`), collapsing
  every other repo's distribution into `other`. Now uses
  `repoMode.workspaceDirs`; single-package repos fall back to top-level
  directory segments.
- **`--focus-class <name>` flag.** `resolve-method-calls.mjs` previously
  always emitted a `RunChannelClient` drilldown regardless of the target.
  The block is now opt-in and accepts any class name.

### Aliased export classification (`build-symbol-graph.mjs`,
`classify-dead-exports.mjs`)

- **`export { local as publicName }` no longer triggers unsafe "definition
  removal" proposals.** The classifier now records the local name on
  `ExportSpecifier` defs and routes aliased dead exports to a dedicated
  `proposal_remove_export_specifier` bucket. Action text distinguishes
  "remove the specifier only" (local still in use) from "specifier + local
  are both unused" — never conflating the two.
- Occurrence counting for aliased specs uses the local name (not the
  exported alias, which appears only on the export line itself).
- Declaration lines (`function X`, `const X`, `class X`, etc.) are
  excluded from the count alongside the export line.

### Shell safety (`measure-staleness.mjs`, `triage-repo.mjs`)

- **`execSync` with string interpolation replaced by `execFileSync` /
  `collectFiles()`.** Previous `execSync(\`git log ... -- "${relFile}"\`)`
  and `execSync(\`find "${root}" ...\`)` broke on filenames containing
  shell metacharacters (`$`, backticks, `;`). `$name` in a filename got
  expanded as an empty shell variable, making git return null timestamps.
  Now uses argv arrays throughout.
- **Python detection no longer gated on `src/` or `tests/` existence.**
  Root-only Python repos were silently reported as 0 Python files. Now
  uses `collectFiles(root, { languages: ['py'] })`.
- **Go files counted in triage.** New `shape.goFiles` field; Go counts
  into totals and `topDirs`.
- Two subprocess calls eliminated — minor speedup.

### Versioning

- `package.json` bumped to `1.2.0`. Internal commit markers in code use
  `v0.6.8` to match the existing FP-XX / v0.6.x comment convention.

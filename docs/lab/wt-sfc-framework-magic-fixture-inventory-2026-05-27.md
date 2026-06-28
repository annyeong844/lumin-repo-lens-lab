# WT-SFC Framework Magic Fixture Inventory

This note records the fixture inventory for the next WT-SFC design lane:
framework magic and convention-driven registration. It follows the SFC support
boundary in [`sfc-support-policy.md`](../spec/sfc-support-policy.md) and the
remaining gaps recorded in
[`wt-sfc-remaining-gaps-inventory-2026-05-26.md`](wt-sfc-remaining-gaps-inventory-2026-05-26.md).

No implementation is selected here. This is a `SPEC` inventory: future slices
must add fixtures, stable reason codes, and public-install verification before
any framework convention affects absence claims.

## Decision

Decision: `framework-magic-inventory-before-implementation`,
`generated-manifest-is-availability-not-absence`,
`convention-and-compiler-magic-stay-muted-or-unavailable`, and
`scan-gap-stays`.

The next safe implementation candidate is generated component-manifest
evidence, not broad framework inference. Generated declarations such as
`.nuxt/components.d.ts` or `components.d.ts` may provide availability evidence
when present, but their absence is not proof that framework registration is
absent.

## Bucket Semantics

Framework magic must be classified before implementation. Use three buckets:

- `explicit-supportable`: future-eligible, review-only evidence from a concrete
  source or generated manifest shape. Presence may create availability evidence.
  Absence must not create an absence claim.
- `muted`: observed but weak evidence. The scanner saw a file, config,
  directive, or convention signal, but the record must stay review-only with a
  stable reason code.
- `unavailable`: not observable from static source in this lane. Compiler
  rewrites, build-time virtual modules, custom runtime resolvers, and framework
  injection stay capability gaps. Do not emit per-instance records without a
  concrete observed target.

Short version: `muted` means "observed but weak"; `unavailable` means "not
observable here."

## Family Matrix

| Family                    | Explicit-supportable                                                                                  | Muted                                                                                                                     | Unavailable                                                                                      |
| ------------------------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| Nuxt auto registration    | Generated `.nuxt/components.d.ts` or equivalent manifest with literal component name to file mapping. | `components/` filesystem convention files observed without a generated manifest: `sfc-framework-nuxt-fs-convention`.      | Build virtual registration and Nuxt auto-registration when no generated manifest is present.     |
| `unplugin-vue-components` | Generated `components.d.ts` with literal component name to file mapping.                              | Vite/Webpack plugin config observed: `sfc-framework-auto-import-plugin-config`.                                           | Custom resolver functions, plugin transforms, and build-time virtual registries.                 |
| Vue macros                | None in this lane.                                                                                    | Literal macro-like component option syntax observed, if future fixtures prove a bounded shape: `sfc-framework-vue-macro`. | Compiler macro expansion and framework-specific runtime behavior.                                |
| Svelte stores/actions     | None beyond existing SFC script import lanes.                                                         | `use:action` directive pointing at an explicit binding: `sfc-framework-svelte-action-directive`.                          | `$store` auto-subscription, compiler reactivity rewrites, and nonlocal action semantics.         |
| Astro conventions         | None beyond existing Astro frontmatter import lanes.                                                  | `client:*` directive on an explicitly imported component: `sfc-framework-astro-client-directive`.                         | Integration-injected components, content collections, and Astro build/runtime convention wiring. |

## Review-Only Contract

Any future evidence surface from this inventory must stay review-only unless a
separate spec proves a stronger contract.

Required constraints:

1. Use a dedicated surface or a clearly isolated lane; do not reuse
   `resolvedInternalEdges[]` or symbol fan-in for convention evidence.
2. Emit `eligibleForFanIn: false` and `eligibleForSafeFix: false` for
   review-only records.
3. Keep the evidence out of deadness, `SAFE_FIX`, `EXISTS`, fix-plan,
   export-action-safety, package edits, and default action lanes.
4. Keep the grouped `sfc-scan-gap` visible after any single lane lands.
5. Treat generated manifests as availability evidence only. Missing generated
   manifests are not absence proof.

## Fixture Inventory

Future slices should start with failing fixtures for these edge cases.

### Nuxt Auto Registration

- Acceptable generated manifest:
  - `.nuxt/components.d.ts` declares a literal component name and a literal file
    path.
  - Expected future status: review-only availability evidence.
- Muted convention:
  - `components/Foo.vue` exists but no generated manifest exists.
  - Expected reason: `sfc-framework-nuxt-fs-convention`.
- Unavailable:
  - auto-registration exists only through Nuxt build virtual modules.
  - Expected behavior: no per-instance record; retain `sfc-scan-gap`.

### `unplugin-vue-components`

- Acceptable generated manifest:
  - `components.d.ts` or plugin-generated declaration maps component names to
    literal source files.
  - Expected future status: review-only availability evidence.
- Muted config:
  - Vite/Webpack config includes `Components(...)`, but no generated manifest
    exists.
  - Expected reason: `sfc-framework-auto-import-plugin-config`.
- Unavailable:
  - custom resolver function or build transform hides the mapping.
  - Expected behavior: no guessed component target.

### Vue Macros

- Muted literal shape:
  - future fixture may cover a narrow literal macro/component option shape only
    if the target binding is explicit.
  - Expected reason: `sfc-framework-vue-macro`.
- Unavailable:
  - compiler-only macro expansion or nonliteral option composition.
  - Expected behavior: no concrete target.

### Svelte Stores And Actions

- Muted explicit directive:
  - `use:actionName` points at an imported or locally declared binding.
  - Expected reason: `sfc-framework-svelte-action-directive`.
- Unavailable:
  - `$store` auto-subscription or compiler reactivity rewrite.
  - Expected behavior: no fan-in or deadness effect.

### Astro Conventions

- Muted explicit directive:
  - `client:*` appears on an explicitly imported component.
  - Expected reason: `sfc-framework-astro-client-directive`.
- Unavailable:
  - integration-injected components, content collections, or runtime wiring.
  - Expected behavior: no concrete target.

## Required Gates Before Implementation

Each future slice must provide:

1. exact accepted syntax or manifest schema;
2. stable muted/unavailable reason codes for rejected shapes;
3. Node and Vitest fixtures;
4. runtime public-install verification;
5. explicit proof that graph edges, fan-in, deadness, `SAFE_FIX`, `EXISTS`,
   fix-plan, export-action-safety, and package edits remain unaffected;
6. confirmation that `sfc-scan-gap` remains visible.

## Next Candidate

If this lane proceeds, start with generated component manifests:

- Nuxt `.nuxt/components.d.ts`;
- `unplugin-vue-components` `components.d.ts`;
- review-only availability records only;
- no absence claim when the generated manifest is missing.

The selected P1 contract is recorded in
[`sfc-generated-component-manifest-evidence.md`](../spec/sfc-generated-component-manifest-evidence.md):
use a dedicated `symbols.json.sfcGeneratedComponentManifests[]` surface, preserve
`.nuxt/components.d.ts` as an allow-list read exception, keep SFC targets muted
with `resolvedFile`, and record missing manifest targets as `unresolved`
manifest freshness evidence.

Do not implement broad framework convention inference first. That would be a
fast path to believable nonsense.

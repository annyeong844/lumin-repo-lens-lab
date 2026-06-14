# SFC Generated Component Manifest Evidence

> **Status:** P1 design contract.
> **Boundary:** Generated component manifests only. Do not infer broad
> framework conventions in this slice.

## Problem

Frameworks and auto-import plugins can make SFC components available to
templates without explicit imports in the consuming file. Guessing those
conventions from directory layout would create believable false claims.

Generated declaration manifests are different. When present, they can provide a
literal component-name to file mapping. That is availability evidence, not
template consumption and not deadness proof.

## Scope

P1 accepts only this explicit allow-list:

- `<root>/components.d.ts` for `unplugin-vue-components`;
- `<root>/.nuxt/components.d.ts` for Nuxt generated component declarations.

The `.nuxt/components.d.ts` path is an explicit read exception even if `.nuxt/`
is otherwise excluded from broad scanning. This exception is narrow: it does
not admit other `.nuxt/` files.

Out of scope:

- Nuxt filesystem convention inference from `components/`;
- `auto-imports.d.ts` composable manifests;
- Vite/Webpack plugin config inference;
- custom resolver functions;
- runtime or compiler-generated virtual modules;
- package edits, fix plans, SARIF, or default Markdown action wording.

Nuxt filesystem convention evidence is handled by a separate review-only
surface. This manifest contract must stay limited to generated declaration
files.

## Surface

Use a dedicated review-only surface:

- `symbols.json.sfcGeneratedComponentManifests[]`;
- `symbols.meta.supports.sfcGeneratedComponentManifests === true`;
- `symbols.uses.sfcGeneratedComponentManifests`.

Do not merge this into `sfcGlobalComponentRegistrations[]`. The provenance is
different: global registration is in-source availability evidence, while this
lane is generated-manifest availability evidence.

## Record Shape

Example record:

```json
{
  "manifestFile": ".nuxt/components.d.ts",
  "manifestKind": "nuxt-components-dts",
  "componentName": "BaseButton",
  "normalizedTagNames": ["BaseButton", "base-button"],
  "bindingSource": "../components/BaseButton.vue",
  "fromSpec": "../components/BaseButton.vue",
  "status": "muted",
  "reason": "sfc-framework-generated-manifest-non-source-binding",
  "resolvedFile": "components/BaseButton.vue",
  "source": "sfc-framework-generated-manifest",
  "confidence": "generated-manifest-availability",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false,
  "line": 7
}
```

`manifestKind` is one of:

- `nuxt-components-dts`;
- `unplugin-vue-components-dts`.

## Parsing Contract

Parse declaration files as TypeScript declaration input. P1 recognizes literal
members under `declare module "vue"` / `interface GlobalComponents` with this
shape:

```ts
Name: typeof import("LITERAL")["default"];
```

Rules:

- the component name comes from the interface member key;
- the binding source comes from the literal `import("...")` string;
- only literal import strings are accepted;
- nonliteral or computed mappings do not produce concrete component records;
- path resolution is relative to the manifest file.

## Status And Reasons

| Case                                                      | Status                         | Reason                                                                                 |
| --------------------------------------------------------- | ------------------------------ | -------------------------------------------------------------------------------------- |
| Target resolves to `.vue`, `.svelte`, or `.astro`         | `muted` with `resolvedFile`    | `sfc-framework-generated-manifest-non-source-binding`                                  |
| Target resolves to source `.js`, `.jsx`, `.ts`, or `.tsx` | `resolved` with `resolvedFile` | none                                                                                   |
| Target file is missing                                    | `unresolved`                   | `sfc-framework-generated-manifest-unresolved`                                          |
| Mapping is nonliteral or computed                         | skipped                        | `sfc-framework-generated-manifest-nonliteral` if a skipped diagnostic/count is emitted |
| Manifest file is absent                                   | zero records                   | no absence proof                                                                       |

Missing targets should be recorded as `unresolved`, not silently skipped. A
stale generated manifest is useful evidence for a reviewer.

## Review-Only Contract

This lane must not enter:

- `resolvedInternalEdges[]`;
- symbol fan-in;
- deadness ranking;
- `SAFE_FIX`;
- `EXISTS`;
- fix-plan;
- export-action-safety;
- package edits;
- default action lanes.

Every record must carry:

- `source: "sfc-framework-generated-manifest"`;
- `confidence: "generated-manifest-availability"`;
- `eligibleForFanIn: false`;
- `eligibleForSafeFix: false`.

The grouped `sfc-scan-gap` remains visible after this lane lands. A generated
manifest can prove availability when present; its absence does not prove that
framework registration is absent.

## Fixture Matrix

P1 should start with failing edge-case fixtures.

| ID  | Fixture                                                                                                        | Expected                                                                                   |
| --- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| F1  | `components.d.ts` maps a component to an existing `.vue` file.                                                 | `muted`, non-source-binding reason, and `resolvedFile`.                                    |
| F2  | `.nuxt/components.d.ts` maps a component to an existing `.vue` file under an otherwise excluded `.nuxt/` tree. | record exists, proving the allow-list read exception.                                      |
| F3  | manifest maps a component to an existing `.ts` or `.tsx` source module.                                        | `resolved` and `resolvedFile`.                                                             |
| F4  | manifest maps a component to a missing file.                                                                   | `unresolved`, generated-manifest-unresolved reason, no `resolvedFile`.                     |
| F5  | manifest contains a nonliteral/computed mapping.                                                               | no concrete target record; optional skipped diagnostic uses generated-manifest-nonliteral. |
| F6  | no manifest exists but `components/Foo.vue` exists.                                                            | zero records; `sfc-scan-gap` remains.                                                      |
| F7  | manifest maps to a module with exports.                                                                        | graph edges, fan-in, deadness, SAFE/action lanes, and fix-plan remain unchanged.           |
| F8  | component name normalization.                                                                                  | `normalizedTagNames[]` contains PascalCase and kebab-case forms.                           |

## Decision Tokens

- `generated-manifest-evidence-before-convention-inference`;
- `manifest-allow-list-read-exception`;
- `manifest-availability-not-absence`;
- `sfc-target-stays-muted-with-resolvedFile`;
- `missing-manifest-target-is-unresolved`;
- `scan-gap-stays`.

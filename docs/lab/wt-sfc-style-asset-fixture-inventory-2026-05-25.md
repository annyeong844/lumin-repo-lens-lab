# WT-SFC Style Asset Fixture Inventory

This note selects the next SFC lane after beta.64 script-source reachability.
It is a design fixture inventory, not an implementation result.

## Decision

Decision: `style-assets-before-template-refs` and
`asset-reachability-not-symbol-fan-in`.

The next small SFC lane should be style asset references, not template component
references. Style assets are still not JavaScript, but they are syntactic enough
to bound: a literal `url("./logo.svg")` or style `@import "./theme.css"` can be
recorded as non-source asset evidence without claiming module graph reachability
or named export fan-in.

Template component refs stay parked. A tag such as `<UserCard />` needs
binding-aware resolution across local imports, global registration, framework
auto-registration, custom elements, and compiler conventions. Guessing from tag
names would be fast, cute, and wrong.

## Proposed Evidence Surface

The first implementation should expose a narrow style-asset surface with stable
source labels such as `sfc-style-url` and `sfc-style-import`.

Recommended payload shape:

```json
{
  "consumerFile": "components/App.vue",
  "fromSpec": "./logo.svg",
  "resolvedFile": "components/logo.svg",
  "source": "sfc-style-url",
  "language": "vue",
  "styleKind": "url",
  "confidence": "grounded-asset-reference"
}
```

The evidence may help future asset hygiene or resource-surface reporting. It
must not feed JS/TS module reachability, symbol fan-in, `SAFE_FIX`, `EXISTS`,
package edits, or dead-export ranking.

## Fixture Matrix

| #     | Fixture                           | Expected Evidence                          | Must Not Do                                                       |
| ----- | --------------------------------- | ------------------------------------------ | ----------------------------------------------------------------- |
| S3-1  | Vue `<style>` relative `url()`    | `App.vue` records `./logo.svg` as asset    | Do not create a JS/TS module graph edge.                          |
| S3-2  | Svelte `<style>` relative `url()` | `Widget.svelte` records `./icon.svg`       | Do not mark any source export consumed.                           |
| S3-3  | Astro `<style>` relative `url()`  | `Page.astro` records `./hero.png`          | Do not disable `sfc-scan-gap`.                                    |
| S3-4  | Style `@import "./theme.css"`     | Stylesheet reference evidence              | Do not parse imported CSS as JavaScript.                          |
| S3-5  | Quoted and unquoted `url()`       | Same normalized asset reference shape      | Do not treat CSS syntax trivia as distinct resources.             |
| S3-6  | `data:` / `http:` / absolute URL  | External or ignored asset diagnostic       | Do not resolve it as a source file.                               |
| S3-7  | Package-style asset specifier     | Unsupported or package-asset diagnostic    | Do not feed package asset specs into dependency usage as imports. |
| S3-8  | Missing relative asset            | Diagnostic-only unresolved asset evidence  | Do not invent a resolved file.                                    |
| S3-9  | `url()` inside CSS comments       | No evidence                                | Do not regex-match commented text.                                |
| S3-10 | CSS variables / dynamic URL       | No concrete edge or diagnostic-only record | Do not guess runtime-computed asset paths.                        |
| S3-11 | `<template style="...url()">`     | Out of scope for this lane                 | Do not scan template attribute CSS before a template lane exists. |
| S3-12 | `<style lang="scss">` literal URL | Either grounded evidence or explicit skip  | Do not claim full SCSS parsing unless that dialect is tested.     |

## Acceptance Criteria

Positive checks:

1. Literal relative style `url()` and style `@import` references are recorded
   with a style-asset source label.
2. The payload records the SFC file, raw specifier, normalized/resolved target,
   language, style evidence kind, and confidence.
3. Missing relative assets are diagnostic-only and keep their raw specifier.
4. The broader `sfc-scan-gap` blind zone remains visible.

Negative checks:

1. Style asset evidence does not enter `resolvedInternalEdges[]` as an import.
2. Style asset evidence does not affect named export fan-in.
3. Package, URL/data, absolute, dynamic, commented, and template-attribute
   forms do not become concrete source edges.
4. `SAFE_FIX`, `EXISTS`, package-edit, fix-plan, and default Markdown action
   lanes remain unchanged.

## Why Not Template Components Yet

Template component support is the tempting shiny thing. It is also the easiest
place to lie.

Before component tags affect deadness, the implementation needs a binding model
that can distinguish imported components, local declarations, auto-registered
framework components, custom elements, slots, and plain markup. The current
SFC contract has not proven that model. Style assets are a smaller lane with a
cleaner evidence boundary.

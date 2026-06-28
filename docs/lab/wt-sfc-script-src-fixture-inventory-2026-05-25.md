# WT-SFC Script Src Fixture Inventory

This note defines the fixture inventory for the next SFC lane after beta.63
script import consumers. It is intentionally a design fixture, not an
implementation result.

## Decision

Decision: `script-src-reachability-first` and `no-symbol-fan-in`.

Literal SFC `<script src>` references should be modeled as file reachability or
script-source evidence before they are allowed to affect named export fan-in.
That distinction matters: a referenced script file is included by the SFC
container, but the SFC is not importing every named export from that file.

## Proposed Evidence Surface

The first implementation should expose a narrow script-source surface with a
stable source label such as `sfc-script-src`.

The surface may feed module reachability, but it must not claim that exported
symbols inside the referenced file are consumed unless a later fixture proves a
separate import/binding relationship.

Recommended payload shape:

```json
{
  "consumerFile": "components/App.vue",
  "fromSpec": "./app-logic.ts",
  "resolvedFile": "components/app-logic.ts",
  "source": "sfc-script-src",
  "language": "vue",
  "scriptKind": "external-script",
  "confidence": "grounded-reachability"
}
```

The exact artifact can live in `symbols.json`, `module-reachability.json`, or a
small SFC-specific surface. The implementation PR should choose one and pin it
with tests before public verification.

## Fixture Matrix

| #     | Fixture                    | Expected Evidence                                      | Must Not Do                                                                   |
| ----- | -------------------------- | ------------------------------------------------------ | ----------------------------------------------------------------------------- |
| S2-1  | Vue literal relative src   | `App.vue` records `./app-logic.ts` as `sfc-script-src` | Do not mark all exports in `app-logic.ts` as symbol fan-in.                   |
| S2-2  | Svelte literal src         | `Widget.svelte` records `./widget-logic.ts`            | Do not scan Svelte markup text for imports.                                   |
| S2-3  | Astro no script-src        | No script-source evidence                              | Do not invent Astro script-source semantics from frontmatter.                 |
| S2-4  | Non-literal src            | Unsupported or ignored diagnostic                      | Do not create a concrete graph edge.                                          |
| S2-5  | Package src                | Unsupported or ignored diagnostic                      | Do not resolve package names as source files.                                 |
| S2-6  | URL/data src               | Unsupported or ignored diagnostic                      | Do not feed URL/data references into source reachability.                     |
| S2-7  | Generated path             | Unsupported, non-source, or policy-excluded evidence   | Do not create SAFE/action claims.                                             |
| S2-8  | Missing file               | Unresolved SFC script-source diagnostic                | Do not silently drop it as if absence were proven.                            |
| S2-9  | Inline script plus src     | Inline imports still work; src evidence is separate    | Do not merge src evidence into `sfc-script-import-consumers`.                 |
| S2-10 | TSX inline script plus src | TSX import consumer still parses                       | Do not regress beta.63 TSX dialect handling while adding src support.         |
| S2-11 | Re-export-only logic file  | File may become reachable                              | Do not protect re-exported symbols solely because the file is script-sourced. |
| S2-12 | Runtime side effect file   | File reachability may be recorded                      | Do not claim dead-export safety for the file's exports.                       |

## Acceptance Criteria

Positive checks:

1. Literal relative Vue/Svelte `<script src>` references are recorded with
   source `sfc-script-src`.
2. The referenced source file participates in reachability according to the
   chosen implementation surface.
3. Inline script import consumers from beta.63 still feed graph and fan-in.
4. The broader `sfc-scan-gap` blind zone remains visible.

Negative checks:

1. No named export is marked consumed only because its file appeared in
   `<script src>`.
2. Non-literal, package, URL/data, generated, and missing script sources do not
   become concrete import edges.
3. Template text still cannot create fake imports.
4. `SAFE_FIX`, `EXISTS`, package-edit, fix-plan, and default Markdown action
   lanes remain unchanged.

## Why Not Template Components Yet

Template component references require binding-aware resolution. A tag like
`<UserCard />` might refer to an imported binding, a local component, an auto
registered global, a framework convention, or plain custom element markup. Tag
name guessing would make the graph look smarter while quietly lying.

Script-source reachability is narrower and easier to prove. Do that first.

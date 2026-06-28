# WT-SFC Global Component Registration Fixture Inventory

This note selects the next WT-SFC lane after the beta.67 template component
target evidence and the remaining-gaps inventory:
[`wt-sfc-remaining-gaps-inventory-2026-05-26.md`](wt-sfc-remaining-gaps-inventory-2026-05-26.md).

It is a design fixture inventory, not an implementation result.

## Decision

Decision: `explicit-registration-before-framework-convention` and
`registration-evidence-is-not-template-consumption`.

The next small SFC lane should record explicit Vue component registration, not
Nuxt auto-imports or framework convention guesses. `app.component("UserCard",
UserCard)` is concrete syntax with a bounded API shape. It can tell reviewers
that a component name may be made available globally, but it still does not
prove that any template actually renders that component.

This lane should produce review-only registration evidence. It must not feed
JS/TS graph edges, named export fan-in, deadness, `SAFE_FIX`, `EXISTS`, package
edits, or default action lanes.

## Proposed Evidence Surface

If implemented, the first safe surface should be a new isolated surface such as
`symbols.json.sfcGlobalComponentRegistrations[]`.

Recommended payload shape:

```json
{
  "registrationFile": "src/main.ts",
  "framework": "vue",
  "api": "app.component",
  "componentName": "UserCard",
  "normalizedTagNames": ["UserCard", "user-card"],
  "bindingName": "UserCard",
  "bindingSource": "./components/UserCard.vue",
  "resolvedFile": "src/components/UserCard.vue",
  "source": "sfc-global-component-registration",
  "status": "registration-syntax",
  "confidence": "registration-review",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

The evidence is about availability, not use. A globally registered component
can help a reviewer understand why `<UserCard />` may be valid without a local
binding, but the registration alone is not a consumption proof.

## Fixture Matrix

| #     | Fixture                                                                           | Expected Evidence                                      | Must Not Do                                                                     |
| ----- | --------------------------------------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------- |
| S5-1  | Vue `app.component("UserCard", UserCard)` with static import                      | Registration evidence with `resolvedFile`              | Do not mark `UserCard.vue` exports consumed.                                    |
| S5-2  | Vue `app.component("user-card", UserCard)`                                        | Registration evidence with explicit name normalization | Do not guess unrelated component names from token similarity.                   |
| S5-3  | Vue plugin `install(app) { app.component("UserCard", UserCard) }`                 | Plugin registration evidence                           | Do not require the plugin to be installed before recording registration syntax. |
| S5-4  | `app.component(componentName, UserCard)`                                          | Muted or diagnostic-only computed-name evidence        | Do not invent a concrete global tag name.                                       |
| S5-5  | `app.component("UserCard", resolveComponent(...))`                                | Muted unsupported value evidence                       | Do not create a fake `resolvedFile`.                                            |
| S5-6  | `app.component("UserCard", defineAsyncComponent(() => import("./UserCard.vue")))` | Future or muted async-registration evidence            | Do not treat async component factories as ordinary static imports in P1.        |
| S5-7  | Duplicate literal registrations for the same name                                 | Deterministic duplicate/ambiguous evidence             | Do not choose one target silently.                                              |
| S5-8  | Local Options API `components: { UserCard }`                                      | Out of scope for this global-registration lane         | Do not mix local registration into global evidence.                             |
| S5-9  | `unplugin-vue-components`, Nuxt components dir, framework config                  | Capability gap                                         | Do not resolve convention-based components from file names alone.               |
| S5-10 | Svelte/Astro component imports                                                    | No global-registration evidence                        | Do not generalize Vue registration APIs to other frameworks.                    |
| S5-11 | Registration target is another SFC file                                           | `resolvedFile` for navigation, still review-only       | Do not convert the SFC target into graph/fan-in evidence.                       |
| S5-12 | Missing imported component target                                                 | Resolver diagnostic only                               | Do not create a concrete registration target.                                   |

## Acceptance Criteria

Positive checks for a future implementation:

1. Explicit Vue literal `app.component(name, binding)` registrations are
   recorded as isolated registration evidence when the component binding is
   statically known.
2. The payload records the registration file, API shape, literal component
   name, normalized tag names, binding name, binding source, resolved file when
   available, framework, evidence source, confidence, and safety flags.
3. Plugin `install(app)` registrations may be recorded as registration syntax,
   but the evidence remains review-only unless a separate app-install contract
   proves runtime installation.
4. Computed names, dynamic values, async factories, duplicate registrations,
   convention-based auto-imports, and missing targets are muted or
   diagnostic-only with stable reason codes.
5. The broader `sfc-scan-gap` blind zone remains visible.

Negative checks:

1. Global component registration evidence does not enter
   `resolvedInternalEdges[]`.
2. Global component registration evidence does not affect named export fan-in,
   deadness, or reachable file sets beyond ordinary imports already present in
   source code.
3. Registration evidence does not appear in `sfcTemplateComponentRefs[]` unless
   a later integration slice explicitly connects template tags to the registry.
4. `SAFE_FIX`, `EXISTS`, package-edit, fix-plan, SARIF, and default Markdown
   action lanes remain unchanged.
5. Nuxt, `unplugin-vue-components`, route/layout conventions, and generated
   component registries stay out of P1.

## Open Questions

1. Should async component factories with literal dynamic imports be recorded as
   muted evidence with a `resolvedFile`, or deferred entirely?
2. Should plugin registration evidence require proving that the plugin is
   passed to `app.use(...)`, or is syntax-level registration evidence enough
   for the first review-only slice?
3. Should duplicate global registrations be grouped under one ambiguous record
   or emitted as separate records with a shared ambiguity reason?
4. Should a later template integration slice connect unbound template tags to
   `sfcGlobalComponentRegistrations[]`, and if so should the result remain
   `muted` rather than `resolved`?

## Why This Comes Before Auto Imports

Auto imports are convention-heavy. They depend on framework config, generated
registries, route/layout rules, and plugin behavior. Explicit
`app.component(...)` registration is smaller: the syntax is local, the literal
name is visible, and the binding can be resolved with the existing import
machinery.

The rule for this lane is blunt: registration proves availability, not use.

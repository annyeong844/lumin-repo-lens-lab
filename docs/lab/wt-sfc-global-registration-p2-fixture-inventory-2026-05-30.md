# WT-SFC Global Registration P2 Fixture Inventory

Date: 2026-05-30

This note selects the next narrow follow-up for the WT-SFC Vue global component
registration lane after the beta.68 public-install verification and the
framework-magic slices through beta.76.

It is a fixture inventory, not an implementation result. P2 must keep the
existing review-only `symbols.json.sfcGlobalComponentRegistrations[]` surface
and must not turn global registration evidence into template consumption or
named export fan-in.

## Decision

Decision tokens:
`global-registration-p2-before-template-consumption`,
`plugin-install-syntax-is-not-runtime-install-proof`,
`async-registration-stays-muted`,
`duplicate-registration-stays-ambiguous`, and `scan-gap-stays`.

P2 should refine the already selected global-registration lane, not expand into
Nuxt, `unplugin-vue-components`, generated manifests, Options API local
registration, or template-tag matching. The useful next slice is:

- pin plugin `install(app) { app.component(...) }` as syntax-level
  registration evidence;
- record literal async component factories as muted evidence, not ordinary
  resolved registrations;
- record duplicate literal registrations as ambiguity evidence instead of
  silently choosing one target.

## Surface

P2 continues to use `symbols.json.sfcGlobalComponentRegistrations[]`.

All records must keep:

- `framework: "vue"`;
- `source: "sfc-global-component-registration"`;
- `eligibleForFanIn: false`;
- `eligibleForSafeFix: false`.

`registration-syntax` remains available only for direct, literal
`app.component("Name", ImportedBinding)` style registrations with a value import
binding. New P2 evidence that is incomplete, ambiguous, or factory-shaped must
use `status: "muted"` and a stable reason code.

## Accepted And Muted Shapes

### Plugin Install Syntax

Fixture:

```js
import PluginCard from "./components/PluginCard.vue";

export default {
  install(app) {
    app.component("PluginCard", PluginCard);
  },
};
```

Expected:

- one `sfcGlobalComponentRegistrations[]` record;
- `api` reflects the receiver call, for example `app.component`;
- `componentName: "PluginCard"`;
- `bindingName: "PluginCard"`;
- the record is review-only;
- no requirement to prove `app.use(...)` in the app entrypoint.

The syntax says the plugin registers a component when installed. It does not
prove that the plugin is actually installed in the current app.

### Async Component Factory

Fixture:

```js
import { defineAsyncComponent } from "vue";

const app = createApp({});
app.component(
  "AsyncCard",
  defineAsyncComponent(() => import("./components/AsyncCard.vue")),
);
```

Expected:

- one muted record with `componentName: "AsyncCard"`;
- reason `sfc-global-component-async-factory`;
- `fromSpec: "./components/AsyncCard.vue"`;
- `resolvedFile` when the literal relative target resolves;
- no named export fan-in and no ordinary `registration-syntax` claim.

Async factories describe a lazy availability path. They must not be treated as
the same evidence as an imported value binding.

### Duplicate Literal Registrations

Fixture:

```js
import FirstCard from "./components/FirstCard.vue";
import SecondCard from "./components/SecondCard.vue";

const app = createApp({});
app.component("UserCard", FirstCard);
app.component("UserCard", SecondCard);
```

Expected:

- deterministic records for both registrations;
- each ambiguous record uses reason
  `sfc-global-component-duplicate-registration`;
- the records share an ambiguity key such as `UserCard`;
- the analyzer does not choose `FirstCard` or `SecondCard` as the canonical
  target.

Duplicate registration order may matter at runtime. P2 should expose the
ambiguity and leave interpretation to reviewers.

## Rejected Or Deferred Shapes

| Fixture shape                                             | Expected behavior                                                                                    |
| --------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `app.component(dynamicName, ImportedCard)`                | Keep the existing muted dynamic-name evidence; do not invent a concrete tag.                         |
| `app.component("FactoryCard", resolveComponent())`        | Keep unsupported-value evidence; do not create `resolvedFile`.                                       |
| `defineAsyncComponent(loader)` with nonliteral loader     | Muted with `sfc-global-component-async-factory-nonliteral` or skipped with a stable reason.          |
| `defineAsyncComponent(() => import(packageName))`         | Muted or skipped as non-relative; do not create a concrete file target.                              |
| Missing async literal target                              | Muted/unresolved with `sfc-global-component-async-target-unresolved`; do not create a fake target.   |
| `app.use(MyPlugin)` without visible `install(app)` syntax | No registration target; plugin installation is not component registration evidence in this P2 slice. |
| Mixins or `extends` providing components                  | Deferred. They need their own fixture inventory and must not be folded into global registration P2.  |

## Safety Contract

P2 remains review-only:

1. no `resolvedInternalEdges[]` entries from these records;
2. no named export fan-in from these records;
3. no deadness, `SAFE_FIX`, `EXISTS`, fix-plan, export-action-safety, package
   edit, SARIF, or default Markdown action-lane entries;
4. no `sfcTemplateComponentRefs[]` integration in this slice;
5. `sfc-scan-gap` remains visible.

## Required Test Anchors

Before implementation, add failing fixtures for:

1. plugin `install(app)` syntax remains syntax-level registration evidence and
   does not require app installation proof;
2. literal async factory target is muted with `resolvedFile`;
3. nonliteral async factory target does not fake `resolvedFile`;
4. duplicate literal names produce deterministic ambiguity evidence;
5. graph, fan-in, deadness, `SAFE_FIX`, `EXISTS`, fix-plan, export-action, and
   template refs remain clean.

Both Node and Vitest mirrors must be updated:

- [`tests/test-sfc-consumers.mjs`](../../tests/test-sfc-consumers.mjs)
- [`tests/sfc-consumers.test.mjs`](../../tests/sfc-consumers.test.mjs)

## Non-Goals

Do not connect global registrations to template tags in P2. A later integration
slice may decide whether an unbound `<UserCard />` can be matched against
`sfcGlobalComponentRegistrations[]`, but that must remain separate and probably
muted until corpus data proves the false-positive budget.

Do not add mixin/extends component aggregation here. Those APIs can compose
objects across files and runtime factories. Treating them as a small extension
would be a tidy-looking trap.

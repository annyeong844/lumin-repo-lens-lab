# WT-SFC Template Component Ref Fixture Inventory

This note selects the next SFC lane after beta.65 style asset evidence. It is
a design fixture inventory, not an implementation result.

## Decision

Decision: `template-binding-inventory-before-evidence` and
`binding-aware-or-no-claim`.

Template component references should not affect graph reachability, named
export fan-in, deadness, `SAFE_FIX`, `EXISTS`, or package edits until the
implementation can prove which script binding a tag refers to. A tag name alone
is not evidence. `<UserCard />` might be an imported component, a locally
registered component, a globally registered component, a framework auto-import,
a custom element, or plain markup with an unfortunate name.

The next slice should inventory fixtures and choose a narrow review-only
surface. It should not promote template references into ordinary import
consumers or symbol fan-in.

## Proposed Evidence Surface

If implemented, the first safe surface should be review-only template binding
evidence with a stable source label such as `sfc-template-component-ref`.

Recommended payload shape:

```json
{
  "consumerFile": "components/App.vue",
  "tagName": "UserCard",
  "normalizedTagName": "UserCard",
  "bindingName": "UserCard",
  "bindingSource": "./UserCard.vue",
  "resolvedFile": "components/UserCard.vue",
  "source": "sfc-template-component-ref",
  "language": "vue",
  "templateKind": "component-tag",
  "confidence": "binding-review",
  "eligibleForFanIn": false,
  "eligibleForSafeFix": false
}
```

The first implementation should keep this surface out of
`resolvedInternalEdges[]` and out of formal symbol fan-in. Review evidence can
help an agent inspect a possibly related component before creating a duplicate,
but it must not become proof that an export is consumed.

## Fixture Matrix

| #     | Fixture                                       | Expected Evidence                                          | Must Not Do                                                                    |
| ----- | --------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------ |
| S4-1  | Vue `<UserCard />` with `import UserCard`     | Review-only binding evidence                               | Do not mark `UserCard.vue` exports consumed.                                   |
| S4-2  | Vue `<user-card />` with `import UserCard`    | Review-only evidence only if casing normalization is explicit | Do not guess across arbitrary token similarity.                             |
| S4-3  | Vue Options API `components: { UserCard }`    | Local registration evidence if the registration binding resolves | Do not treat every registered component as used without a matching tag.  |
| S4-4  | Vue global or auto-import component           | Capability gap or muted diagnostic                         | Do not invent a resolved file from convention alone.                           |
| S4-5  | Vue dynamic `<component :is="UserCard" />`    | Muted or diagnostic-only dynamic evidence                  | Do not convert runtime component selection into concrete fan-in.               |
| S4-6  | Vue native/custom element `<my-widget>`       | No concrete component evidence                             | Do not classify unknown kebab tags as imports.                                 |
| S4-7  | Svelte `<UserCard />` with import             | Review-only binding evidence                               | Do not scan string/template text as tags.                                      |
| S4-8  | Svelte `<svelte:component this={UserCard} />` | Muted or dynamic component evidence                        | Do not claim a static component use without a direct binding proof.            |
| S4-9  | Svelte action/store syntax                    | Out of scope                                               | Do not mix framework magic into component refs.                                |
| S4-10 | Astro `<UserCard />` with import              | Review-only binding evidence                               | Do not treat Astro frontmatter imports as template use unless the tag matches. |
| S4-11 | Astro hydrated component directives           | Preserve directive metadata if surfaced                    | Do not let `client:*` directives strengthen the claim beyond review-only.      |
| S4-12 | Markdown/MDX-like or lowercase HTML tags      | No component evidence                                      | Do not let broad tag parsing pollute SFC component evidence.                   |
| S4-13 | Namespace/member component `<UI.Card />`      | Future or muted diagnostic                                 | Do not flatten namespace member access into a plain binding name.              |
| S4-14 | Duplicate local/global names                  | Ambiguous diagnostic                                       | Do not choose one silently.                                                    |
| S4-15 | Missing imported component file               | Resolver diagnostic only                                   | Do not create a fake resolved component.                                       |

## Acceptance Criteria

Positive checks for a future implementation:

1. Imported Vue/Svelte/Astro component bindings can be surfaced as review-only
   template evidence when a matching template tag is proven.
2. The payload records the SFC file, tag name, normalized tag name, matched
   binding, binding source, resolved file when available, language, evidence
   kind, confidence, and safety flags.
3. Ambiguous, dynamic, global, auto-import, namespace, and missing cases are
   muted or diagnostic-only with stable reason codes.
4. The broader `sfc-scan-gap` blind zone remains visible.

Negative checks:

1. Template component evidence does not enter `resolvedInternalEdges[]`.
2. Template component evidence does not affect named export fan-in.
3. Kebab/Pascal casing normalization is explicit and fixture-pinned.
4. Native HTML tags, custom elements, comments, strings, slots, dynamic tags,
   and framework magic do not become concrete source edges.
5. `SAFE_FIX`, `EXISTS`, package-edit, fix-plan, SARIF, and default Markdown
   action lanes remain unchanged.

## Open Questions

1. Should the first implementation support Vue Options API registration, or
   start with script import bindings only?
2. Should kebab-case to PascalCase normalization be enabled in P1, or recorded
   as muted evidence until corpus data proves it is safe?
3. Should global component conventions be represented as a capability gap or as
   framework/resource surface evidence?
4. Should template refs ever feed fan-in, or should they stay permanently
   review-only unless a compiler-grade SFC binding model exists?

## Why This Comes After Style Assets

Style asset references are syntactic and isolated. Template component tags are
semantic. The tool needs a binding model before it can say anything meaningful
about them.

The rule is simple: no binding proof, no graph claim.

# WT-21 beta.43 Public Framework/Resource Capability Pack Verification

Maintainer checklist for `0.9.0-beta.43` public install verification. This note
is lab evidence scaffolding for the WT-21 capability artifact surface; it is not
a user-facing contract.

## Scope

- Feature: WT-21 framework/resource capability-pack ownership.
- Package under test: public installed `lumin-repo-lens-lab` beta.43.
- Expected artifact: `framework-resource-surfaces.json`.
- Expected manifest mirror: `manifest.json.frameworkResourceSurfaces`.
- Recommended corpus: a small fixture or repo that emits at least one
  framework/resource surface, such as Storybook, Strapi, bundled/generated JS,
  generated `.d.ts`, templates, or codemod resources.

## Checks

| Check | Expected Result | Evidence To Record |
|---|---:|---|
| Installed version is `0.9.0-beta.43` | PASS | Plugin cache path, `plugin.json`, `marketplace.json`, and skill `package.json` agree on `0.9.0-beta.43`. |
| Raw surface lanes carry `capabilityPack` | PASS | `framework-resource-surfaces.json.files[].surfaceLanes[].capabilityPack` includes owners such as `framework.storybook`, `framework.strapi`, `surface.bundled-build-artifact`, `surface.generated-declaration`, `surface.scaffold-template`, or `surface.codemod-resource`. |
| Raw summary carries pack pivot | PASS | `framework-resource-surfaces.json.summary.byCapabilityPack` exists and counts the emitted pack owners. |
| Manifest mirrors pack pivot | PASS | `manifest.json.frameworkResourceSurfaces.byCapabilityPack` exists and matches the raw artifact summary for the same run. |
| Human review reminders remain visible | PASS | `audit-summary.latest.md` and `audit-review-pack.latest.md` still tell readers to inspect `manifest.json.frameworkResourceSurfaces` and `framework-resource-surfaces.json` before treating import absence as deadness. |

## Interpretation

This verification closes only the public-install visibility slice for
framework/resource capability-pack ownership. It does not mark WT-21 `DONE` by
itself. Future WT-17, WT-20, and WT-04 work should still route new dynamic,
output-mapping, and generated-artifact gaps through named capability diagnostics
instead of one-off heuristics.

## Result

```text
Run path:
node audit-repo.mjs --root <fixture> --output <fixture>/.audit --profile full
in C:/Users/endof/Downloads/auditing-repo-structure. The engine bytes matched
the public installed beta.43 package for the checked WT-21 libraries.

Corpus:
Inline BFRS2-shaped fixture containing:
- Storybook story: src/Button.stories.tsx with @storybook/react dependency.
- Strapi controller: src/api/article/controllers/article.ts with
  @strapi/strapi dependency.
- Generated declaration: types/generated/contentTypes.d.ts.
- Bundled/build artifacts: public/vendor.js, src/app.bundle.js, and
  src/emscripten-bindings.js.
- Scaffold template: templates/controller.ts.hbs.
- Codemod resource: resources/codemods/rename/input.ts.
- Plain TS control: src/plain.ts.

Generated:
framework-resource-surfaces.json, manifest.json, audit-summary.latest.md, and
audit-review-pack.latest.md.

Installed version:
0.9.0-beta.43 across installed_plugins.json, plugin.json, marketplace.json, and
skills/lumin-repo-lens-lab/package.json.

Raw capability packs:
framework-resource-surfaces.json recorded
files["src/Button.stories.tsx"].surfaceLanes[].capabilityPack as
["framework.storybook"].

summary.byCapabilityPack:
{
  "framework.storybook": 1,
  "framework.strapi": 1,
  "surface.bundled-build-artifact": 3,
  "surface.codemod-resource": 1,
  "surface.generated-declaration": 1,
  "surface.scaffold-template": 1
}

Manifest byCapabilityPack:
manifest.frameworkResourceSurfaces.byCapabilityPack matched the raw
summary.byCapabilityPack exactly.

Summary/review-pack lines:
audit-summary.latest.md and audit-review-pack.latest.md both told readers to
read manifest.json.frameworkResourceSurfaces and
framework-resource-surfaces.json before treating import absence as deadness.

Verdict:
WT-21 capability-pack public install visibility: 5/5 PASS.
```

The same code path also passed the dev tests:

- `tests/test-framework-resource-surfaces.mjs`
- `tests/test-build-framework-resource-surfaces.mjs`

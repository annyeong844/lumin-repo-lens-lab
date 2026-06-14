# Unused Dependencies Review Surface

> **Role:** maintainer-facing wording and leakage spec for surfacing
> `unused-deps.json` in manifest, audit summary, and review-pack artifacts.
> **Status:** DONE. P2c summary/review-pack wording and P2d public install
> verification are complete.
> **Last updated:** 2026-05-24.

## Context

[`unused-deps-producer.md`](unused-deps-producer.md) defines the P1
artifact-only dependency hygiene producer. Beta.57 installed-package
verification confirmed that `unused-deps.json` is review-only evidence and does
not leak into package edits, fix plans, SARIF, action safety, or Markdown
removal wording.

Beta.58 installed-package verification confirmed the P2c review surface:
`manifest.json.unusedDependencies` mirrors review counts plus capped
package-name examples for navigation, while `audit-summary.latest.md` and
`audit-review-pack.latest.md` surface only counts and artifact paths without
package names in Markdown, deletion language, fix-plan/action-safety entries,
SARIF warnings, or safe cue leakage.

The public skill package does not ship `node_modules`. That is intentional:
the runtime dependency guard performs first-run lazy install in
`skills/lumin-repo-lens-lab` with
`npm ci --omit=dev --ignore-scripts --no-audit --fund=false`. With
`LUMIN_REPO_LENS_NO_AUTO_INSTALL=1`, the installed wrapper exits with setup
guidance instead of guessing.

The next risk is not classification. The next risk is language.

Dependency hygiene evidence is easy to overstate. A declaration with no
observed consumer is not proven dead, because package scripts, peer contracts,
optional runtime loading, framework conventions, generated code, incomplete
scan ranges, and package-manager behavior can all explain a declaration without
an import edge.

This spec defines the narrow P2/P3 surface for making dependency hygiene visible
without changing the action contract.

## Current Implementation State

P2b mirrors shallow `unused-deps.json` metadata into
`manifest.json.unusedDependencies`. The mirror includes status, counts,
`byReason`, and capped `topReviewUnused[]` examples for navigation only.

P2c renders weak dependency hygiene lines in `audit-summary.latest.md` and
`audit-review-pack.latest.md`. Markdown shows counts and JSON paths, not
package-name examples, and direct tests guard the wording against package-edit
or strong action language.

P2d public verification passed on beta.58. The verification checked installed
version surfaces, manifest mirror fields, summary/review-pack wording,
Markdown package-name absence, strong-wording absence, and no leakage into
fix-plan, action-safety, dead-classify, SARIF, `SAFE_FIX`, `EXISTS`, or
`SAFE_CUE` surfaces.

## Problem

`unused-deps.json` exists, but the default reviewer surfaces are still mostly
silent. That means a reader has to discover the artifact manually.

The naive fix is to add a line that says "unused dependencies found." That is
too strong. It turns a review cue into an action hint and invites package
manifest edits before the evidence is checked.

The correct fix is a review-only surface:

- enough signal that the artifact exists,
- enough counts to make the audit navigable,
- enough caveat language to prevent action claims,
- enough JSON links that a reviewer can inspect the raw evidence.

## Goals

1. Mirror bounded dependency hygiene metadata into
   `manifest.json.unusedDependencies`.
2. Add weak `audit-summary.latest.md` and `audit-review-pack.latest.md` reader
   guidance when `unused-deps.json` exists or is unavailable.
3. Keep `review-unused` as an inspect-only status.
4. Preserve status semantics for `complete`, `unavailable`, and
   `confidence-limited` artifacts.
5. Keep detailed dependency names in JSON first; Markdown may show only capped
   examples when the wording contract is pinned by tests.
6. Prove that no dependency hygiene fact creates package edits, fix-plan
   entries, SARIF warnings, `SAFE_FIX`, `EXISTS`, or `SAFE_CUE` entries.

## Non-Goals

- Do not edit `package.json` or lockfiles.
- Do not run package-manager uninstall commands.
- Do not add SARIF warnings.
- Do not create `fix-plan.json` entries.
- Do not create `export-action-safety.json` entries.
- Do not create `SAFE_FIX`, `EXISTS`, or `SAFE_CUE` cards.
- Do not add package allowlists or dependency hygiene config in this slice.
- Do not claim that `review-unused` is package-removal proof.

## Manifest Mirror

`manifest.json` may mirror shallow summary fields:

```json
{
  "unusedDependencies": {
    "artifact": "unused-deps.json",
    "schemaVersion": "unused-deps.v1",
    "policyVersion": "unused-deps-review-policy-v1",
    "status": "complete",
    "packageCount": 2,
    "declaredDependencyCount": 12,
    "usedCount": 7,
    "reviewUnusedCount": 2,
    "mutedCount": 3,
    "confidenceLimitedCount": 0,
    "unavailableCount": 0,
    "byReason": {
      "external-import-consumer": 7,
      "no-observed-consumer": 2,
      "package-script-tool": 1,
      "peer-contract": 1,
      "ambient-types": 1
    },
    "topReviewUnused": [
      {
        "packageDir": ".",
        "manifestPath": "package.json",
        "name": "left-pad",
        "field": "dependencies",
        "reason": "no-observed-consumer",
        "confidence": "review"
      }
    ]
  }
}
```

`topReviewUnused` is optional, capped, and shallow. It must not include action
language. Its purpose is navigation, not ranking.

If `unused-deps.json` is missing, malformed, or not supported by the current
run, the manifest mirror must prefer explicit incomplete status over false zero
counts:

```json
{
  "unusedDependencies": {
    "artifact": "unused-deps.json",
    "status": "unavailable",
    "reason": "input-artifact-missing"
  }
}
```

## Markdown Wording

Default Markdown must use weak review language. The preferred complete-state
line is:

```text
Dependency hygiene: 2 review-only dependency declarations need inspection; 3 muted explanations.
Read manifest.json.unusedDependencies and unused-deps.json before changing package manifests.
```

If the artifact is complete and has no review candidates, either stay silent or
use this caveated line:

```text
Dependency hygiene: no review-only dependency declarations in the scanned package scope. This is scan evidence, not package-manager proof.
```

If the artifact is unavailable or confidence-limited:

```text
Dependency hygiene: evidence incomplete; do not infer unused dependency declarations. Read manifest.json.unusedDependencies and unused-deps.json.
```

The default Markdown body must avoid these words as action claims:

- `safe`
- `remove`
- `delete`
- `uninstall`
- `drop`
- `fix`

The words may appear only inside explicit policy explanations, test names, or
artifact identifiers that cannot be read as advice. Tests should check the
rendered dependency hygiene section, not unrelated legacy headings.

## Review-Pack Lane

The review pack should place dependency hygiene in a reviewer evidence lane, not
in a fix lane. The line should be close to other evidence that limits absence
claims:

```text
Dependency hygiene review: inspect unused-deps.json before changing package manifests. review-only=2; muted=3; confidence-limited=0.
```

The review pack may include capped examples only after tests prove the wording
does not become an action cue:

```text
Examples: left-pad in package.json dependencies (reason: no-observed-consumer, confidence: review).
```

Examples must not use stronger language than the artifact status.

## Status Semantics

`status: "complete"` means the producer evaluated the scanned package scope
with supported input lanes. It does not mean dependency removal is proven.

`status: "unavailable"` means the surface must explain why it cannot make
candidate claims. It must not render zero-count success.

`status: "confidence-limited"` means at least one package or declaration could
not be evaluated safely. The surface may show counts, but the Markdown must
say evidence is incomplete.

`review-unused` means no accepted consumer or explanation was observed in the
recorded scan range.

`muted` means the declaration has an explanation or contract that prevents it
from being a review-unused candidate in this policy.

## Leakage Invariants

1. `unused-deps.json` never creates package manifest edits.
2. `unused-deps.json` never creates lockfile edits.
3. `unused-deps.json` never creates `fix-plan.json` entries.
4. `unused-deps.json` never creates `export-action-safety.json` entries.
5. `unused-deps.json` never creates SARIF warnings.
6. `unused-deps.json` never creates `SAFE_FIX`, `EXISTS`, or `SAFE_CUE` cards.
7. `review-unused` never means safe to change.
8. Markdown must link readers to both `manifest.json.unusedDependencies` and
   `unused-deps.json`.
9. Missing or incomplete input must render incomplete evidence, not a zero
   candidate claim.
10. Muted declarations may be counted, but they must not render as action
    candidates.
11. Package names may appear only in review-only examples or JSON evidence, not
    in strong action surfaces.
12. Public install verification is required before this surface can be marked
    `DONE`.

## Acceptance Fixtures

### Complete With Review Candidates

Fixture:

- `dependencies.lodash` imported by source.
- `dependencies.left-pad` has no observed consumer.
- `devDependencies.tsx` appears in a package script.
- `devDependencies["@types/node"]` is ambient type evidence.

Expected:

- `manifest.json.unusedDependencies.reviewUnusedCount === 1`.
- `manifest.json.unusedDependencies.mutedCount === 2`.
- Summary and review pack include review-only dependency hygiene wording.
- `left-pad` does not appear in fix-plan, action-safety, SARIF, or safe cue
  surfaces.

### Unavailable Artifact

Fixture:

- `symbols.meta.supports.dependencyImportConsumers !== true`, or
  `unused-deps.json` is missing/malformed.

Expected:

- Manifest mirror status is `unavailable` or `confidence-limited`.
- Markdown says evidence is incomplete.
- Markdown does not claim zero review candidates.

### Zero Review Candidates

Fixture:

- Every declaration is either `used` or `muted`.

Expected:

- Either no default summary line, or a caveated scanned-scope line.
- No "all clean" or action-style language.

### Muted-Only Evidence

Fixture:

- Only `peerDependencies`, `optionalDependencies`, `@types/*`, and
  package-script tools appear without static imports.

Expected:

- Muted counts surface.
- Muted package names remain in JSON or capped review-only examples.
- No action candidate is created.

### Workspace Ownership

Fixture:

- Root package declares `react`.
- `packages/app` imports `react`.
- Root is not the nearest package for `packages/app/src/App.tsx`.

Expected:

- Root `react` is not marked used from the child package import.
- Any summary wording remains scoped to the evaluated package boundary.

## Implementation Slices

### P2a: Spec And Wiki Links

- Add this spec.
- Link it from the dependency hygiene workstream.
- Record that the P2 surface is still design-only.
- No behavior change.

### P2b: Manifest Mirror

- Add `manifest.json.unusedDependencies`.
- Keep names capped and shallow.
- Add leakage guards for fix-plan, action-safety, SARIF, and cue tiers.
- Implemented by `audit-manifest.mjs` and covered by
  `tests/test-audit-manifest-export-surface.mjs` plus
  `tests/audit-manifest-export-surface.test.mjs`.

### P2c: Summary And Review-Pack Wording

- Add default Markdown wording.
- Test rendered text directly.
- Check that banned action words do not appear in the dependency hygiene
  section.
- Implemented in beta.58 and guarded by `tests/test-audit-repo.mjs` plus
  `tests/audit-repo-artifact-brief.test.mjs`.

### P2d: Public Install Verification

- Run a beta install against a fixture that contains used, muted, review-only,
  and unavailable/confidence-limited dependency cases.
- Verify generated artifacts, summary Markdown, review-pack Markdown,
  fix-plan, action-safety, and SARIF outputs.
- Completed for beta.58. Package names remained in JSON evidence only, Markdown
  stayed count/path-only, and no dependency hygiene evidence leaked into strong
  action surfaces.

## Open Questions

- Should the zero-review-candidate state render a line, or should it stay
  silent by default?
- Should capped package examples appear in `audit-summary.latest.md`, or only
  in the review pack?
- Should `devDependencies` get a separate `dev-review-unused` reason before
  Markdown surfacing?
- Should dependency hygiene receive explicit config before any example package
  names render in Markdown?
- Should framework/resource capability packs mute declarations in P2, or only
  confidence-limit them until corpus calibration grows?

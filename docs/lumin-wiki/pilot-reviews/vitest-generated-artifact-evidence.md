# Vitest Generated Artifact Evidence Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-14.
> **Pilot candidate:** `tests/test-generated-artifact-evidence.mjs`.

---

## Purpose

This review decides whether `tests/test-generated-artifact-evidence.mjs` is
ready for a narrow Vitest mirror. It does not add the Vitest suite. The goal is
to name the generated-artifact evidence policy contracts that a runner
migration must preserve before generated resolver and blind-zone suites move.

This suite is analyzer-sensitive even though it mostly tests a policy helper.
The helper decides when missing build output, static assets, local generated
assets, workspace generated subpaths, or path-segment hints may be treated as
generated artifact evidence. If the migration weakens those distinctions,
resolver diagnostics and `SAFE_FIX` blockers can overclaim.

## Reviewed Evidence

- Preserved Node command:
  `node tests/test-generated-artifact-evidence.mjs`.
- Proposed focused Vitest command:
  `npm run test:vitest:generated-artifact-evidence`.
- Policy module under review:
  `skills/lumin-repo-lens-lab/_engine/lib/generated-artifact-evidence.mjs`.
- Identity consumers checked by the suite:
  `skills/lumin-repo-lens-lab/_engine/lib/audit-manifest.mjs`,
  `skills/lumin-repo-lens-lab/_engine/lib/resolver-core.mjs`,
  `skills/lumin-repo-lens-lab/_engine/lib/finding-provenance.mjs`,
  `skills/lumin-repo-lens-lab/_engine/lib/generated-blind-zone-relevance.mjs`, and
  `skills/lumin-repo-lens-lab/_engine/lib/ranking.mjs`.
- Companion generated suites:
  `node tests/test-generated-blind-zone-relevance.mjs`,
  `node tests/test-generated-consumer-blind-zones.mjs`, and
  `node tests/test-generated-virtual-surface.mjs`.
- Resolver workstream inventory:
  `docs/lumin-wiki/workstreams/resolver.md`.

## Result

The suite is acceptable as a Vitest pilot candidate, but only as a
behavior-preserving mirror.

The current Node suite protects policy boundaries, not resolver expansion. It
must stay focused on the generated evidence classifier and identity constants.
Generated blind-zone relevance, generated consumer inventories, and virtual
surface behavior remain separate suites.

## Protected Invariants

The future Vitest pilot must preserve these generated-artifact evidence
contracts:

- build output evidence requires both package `files` coverage and build-like
  script evidence before it becomes strong generated evidence;
- `files` evidence alone remains weak and returns `null`;
- static artifact evidence requires an explicit package script output path;
- relative generated asset evidence requires an exact package script output
  path and does not match ordinary neighboring assets;
- path-segment generated evidence is supporting only, exposes the
  `generated-artifact-missing` hint, and is not strong evidence;
- workspace subpath evidence matches normalized target subpaths and rejects
  unrelated subpaths;
- generated artifact identity constants come from the policy module:
  `generated-artifact-policy-v1`, `generated-artifact-missing`, and
  `workspace-generated-artifact-missing`;
- downstream modules must not hardcode those generated artifact identity
  strings directly.

## Edge-Case Failures To Preserve

The migration must keep the following failure modes visible:

- A helper must not turn `files: ["dist"]` into strong generated evidence
  without a build-like script.
- A helper must not treat broad path tokens such as `generated` as strong
  proof.
- A helper must not match a relative generated asset unless a package script
  names that exact output path.
- A helper must not treat a workspace generated subpath packet for `enums` as
  evidence for `client`.
- A helper must not duplicate generated-artifact identity string literals into
  downstream modules.
- The mirror must not combine this policy fixture with generated consumer
  blind-zone relevance, generated virtual surfaces, output-to-source layouts,
  dynamic modules, deadness/ranking, or performance fixtures.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- `node tests/test-generated-artifact-evidence.mjs` remains runnable.
- The pilot may use temporary directories for relative generated asset
  evidence, but the helper boundary is setup only. Generated evidence meaning,
  identity constant ownership, and source-file hardcode checks stay local to
  this suite.
- The pilot must not add new generated artifact heuristics.
- The pilot must not change generated blind-zone relevance.
- The pilot must not change virtual generated surfaces.
- The pilot must not promote supporting evidence to strong evidence.
- The pilot must not migrate generated consumer, generated relevance,
  generated virtual surface, resolver, deadness/ranking, or performance suites.

## Recommendation

Proceed to a narrow implementation PR that adds:

1. `tests/generated-artifact-evidence.test.mjs`,
2. `npm run test:vitest:generated-artifact-evidence`,
3. a candidate-board update moving this suite from reviewed candidate to
   implemented pilot evidence.

The implementation PR should keep the Node suite and represent the current
contract as named Vitest `describe(...)` / `it(...)` blocks grouped by
generated evidence policy lane:

- strong build-output evidence;
- weak files-only evidence;
- static and relative exact script output path evidence;
- supporting path-segment evidence;
- workspace subpath matching;
- policy constant ownership and downstream hardcode guard.

Run both commands when changing this suite:

- `node tests/test-generated-artifact-evidence.mjs`
- `npm run test:vitest:generated-artifact-evidence`

Do not migrate any other generated resolver suite as part of the generated
artifact evidence pilot.

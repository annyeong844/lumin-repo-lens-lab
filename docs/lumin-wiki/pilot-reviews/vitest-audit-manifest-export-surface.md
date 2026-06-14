# Vitest Audit Manifest Export Surface Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidate:** `tests/test-audit-manifest-export-surface.mjs`.

---

## Purpose

This review decides whether `tests/test-audit-manifest-export-surface.mjs` can
move as a narrow Lane A Vitest mirror. It does not add a Vitest suite. The goal
is to preserve the public boundary and summary-shape contracts around
`_lib/audit-manifest.mjs` without turning the mirror into a broad audit
pipeline test.

The candidate is acceptable as a single-suite mirror because it imports one
module directly, creates temporary artifact directories only as setup, and
checks manifest evidence summaries from controlled JSON inputs. It does not run
the full audit orchestrator, resolver, classifier, generated-artifact producer,
or public package install flow.

The future mirror should keep those contracts local. It must not expand into
living-audit discovery behavior, resolver correctness, generated artifact
execution, framework/resource producer behavior, blind-zone ranking, or
Markdown rendering.

## Reviewed Evidence

| Suite                                          | Preserved Node Command                              | Proposed Focused Vitest Command                     | Surface Under Review                                      |
| ---------------------------------------------- | --------------------------------------------------- | --------------------------------------------------- | --------------------------------------------------------- |
| `tests/test-audit-manifest-export-surface.mjs` | `node tests/test-audit-manifest-export-surface.mjs` | `npm run test:vitest:audit-manifest-export-surface` | audit manifest public exports and evidence summary shapes |

Current suite description is in `tests/README.md`.

Goal lane: Lane A, low-risk core/helper manifest boundary guard.

## Result

This suite is acceptable as one narrow Vitest mirror.

The future implementation PR should preserve the same direct-module and
temporary-artifact behavior without changing `_lib/audit-manifest.mjs`, manifest
evidence semantics, or producer artifact schemas. The Node entrypoint must
remain runnable.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `_lib/audit-manifest.mjs` exports `buildManifestEvidence`,
  `refreshManifestEvidence`, and `collectProducedArtifacts`;
- `_lib/audit-manifest.mjs` does not expose living-audit internals such as
  `LIVING_AUDIT_DOC_CANDIDATES` or `detectLivingAuditDocs`;
- generated artifact misses summarize reason counts, policy version,
  execution mode, supported generators, and top generated misses;
- framework/resource surface artifacts summarize artifact path, policy version,
  lane counts, capability-pack counts, and top examples;
- generated consumer blind zones summarize count and top scope buckets;
- generated present mode reports existing generated targets that are excluded
  by scan policy as `present-but-out-of-scope`;
- generated prepared mode marks those excluded targets with
  `staleStatus: "unknown"` and
  `staleReason: "generator-input-hash-not-recorded"`;
- resolver diagnostics summarize unresolved counts, ratios, top unresolved
  reasons, specifier roots, and legacy top-unresolved specifier records;
- when `resolver-diagnostics.json` is present, manifest resolver blind-zone
  summary prefers that artifact summary over symbols-only fallback.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- exporting living-audit internals from the public manifest helper must fail;
- hiding any public manifest builder must fail;
- losing generated-artifact policy metadata must fail;
- losing framework/resource `byCapabilityPack` mirroring must fail;
- collapsing generated consumer blind zones without scope/status detail must
  fail;
- treating excluded generated files as cleanly present must fail;
- dropping prepared-mode stale-unknown labeling must fail;
- replacing resolver-diagnostics artifact evidence with older symbols-only
  fallback when the diagnostics artifact exists must fail.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- The preserved Node command remains runnable.
- The fixture boundary is temporary directories and JSON artifacts only.
- A future mirror may use the setup-only temp repo fixture helper if useful,
  but the helper must not decide manifest, resolver, generated-artifact, or
  framework/resource meaning.
- The mirror must not run the full audit pipeline.
- The mirror must not change public exports or manifest evidence shape.
- The mirror must not absorb resolver, generated, framework/resource,
  deadness/ranking, performance, public package, or Markdown renderer suites.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/audit-manifest-export-surface.test.mjs`,
2. `npm run test:vitest:audit-manifest-export-surface`,
3. candidate-board updates moving the suite from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest command fail
because the script or file is missing, then add a mirror that preserves every
current Node assertion as named Vitest cases. It should run the preserved Node
command, the focused Vitest command, and `npm run test:vitest`.

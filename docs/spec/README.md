# Spec Staging Area

This directory holds maintainer-facing design references that still
matter to the engine, but are not part of the small public capability surface.

For the broader docs map, start at [`docs/README.md`](../README.md).

Typical contents:

- longer-lived design specs such as `docs/spec/SPEC-canon-generator.md`
- supporting sub-specs such as `docs/spec/FP-41-sentinel-spec.md`
- proof and ranking specs such as
  `docs/spec/proof-carrying-export-fix.md`
- implementation plans such as
  `docs/spec/proof-carrying-export-fix-implementation-plan.md`
- architecture specs such as
  `docs/spec/incremental-engine-architecture.md`
- architecture realignment specs such as
  `docs/spec/lumin-architecture-realignment.md`
- resolver/blind-zone specs such as
  `docs/spec/generated-artifact-support.md`
- framework/resource surface specs such as
  `docs/spec/framework-resource-surface-policy.md`
- SFC support specs such as `docs/spec/sfc-support-policy.md` and
  `docs/spec/sfc-generated-component-manifest-evidence.md`
- package script runtime entry specs such as
  `docs/spec/package-script-runtime-entry-surface.md`
- dependency hygiene specs such as `docs/spec/unused-deps-producer.md` and
  `docs/spec/unused-deps-review-surface.md`
- dynamic module expansion specs such as
  `docs/spec/import-meta-glob-scan-policy-expansion.md`
- block-level clone detection specs such as
  `docs/spec/block-clone-detection.md`
- pre-write review-lane specs such as
  `docs/spec/pre-write-inline-extraction-cues.md`
- pre-write method-surface specs such as
  `docs/spec/pre-write-class-method-surface.md`
- pre-write sibling-cue specs such as
  `docs/spec/pre-write-service-operation-sibling-cues.md`
- nested pre-write local operation specs such as
  `docs/spec/pre-write-nested-service-operation-surface.md`
- agent-entry, resolver completeness, and threshold calibration debt such as
  `docs/spec/agent-entry-resolver-calibration.md`
- recall/performance debt such as
  `docs/spec/recall-and-performance-gap-plan.md`
- performance architecture slices such as
  `docs/spec/lumin-fused-safer-graph.md`
- wiki/test-reform design slices such as
  `docs/superpowers/specs/2026-05-12-lumin-wiki-test-reform-design.md`
- shared test fixture helper specs such as
  `docs/spec/shared-test-fixture-helper.md`
- living work trackers such as
  `docs/spec/lumin-work-tracker.md`
- calibration notes such as
  `docs/spec/pcef-calibration-eslint-2026-05-04.md` and
  `docs/spec/pcef-calibration-calcom-2026-05-04.md`

User-facing operating material has been moved to the shipping skill
surface:

- `templates/report-template.md`
- `templates/REVIEW_CHECKLIST.md`
- `references/false-positive-index.md`
- `references/lifecycle-modes.md`
- `references/cli-options.md`
- `references/operational-gates.md`
- `references/language-support.md`

This area supports maintainers and reviewers. It is not the public
entrypoint. The public contract remains anchored at:

- `SKILL.md`
- repo-root `README.md`
- `audit-repo.mjs`
- `.claude-plugin/plugin.json`
- `commands/`
- `canonical/` runtime spine

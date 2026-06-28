# Vitest Unused Deps Producer Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-18.
> **Pilot candidate:** `tests/test-unused-deps-producer.mjs`

---

## Purpose

This review records the Node/Vitest mirror boundary for the WT-25 dependency
hygiene producer after implementation. The suite protects `unused-deps.json` as
a review-only artifact. It does not permit summary wording, package edits,
SARIF output, fix-plan entries, or `SAFE_FIX` promotion.

The mirror is acceptable because the behavior under test is a narrow artifact
contract: package identity normalization, package-script tool evidence,
workspace package ownership, unsupported symbols-lane handling, and audit
artifact visibility.

## Reviewed Evidence

| Suite                                 | Preserved Node Command                     | Focused Vitest Command                     | Surface Under Review                          |
| ------------------------------------- | ------------------------------------------ | ------------------------------------------ | --------------------------------------------- |
| `tests/test-unused-deps-producer.mjs` | `node tests/test-unused-deps-producer.mjs` | `npm run test:vitest:unused-deps-producer` | `unused-deps.json` producer and policy helper |

Current evidence checked for this review:

```text
node tests/test-unused-deps-producer.mjs        # 6 passed, 0 failed
npm run test:vitest:unused-deps-producer       # 1 file / 6 tests passed
```

The beta.57 installed-package verification also executed the public package
against a temporary fixture and confirmed the artifact stayed review-only.

## Result

The mirror is accepted as complete for the P1 producer slice.

Future work may extend the dependency hygiene surface only through a new spec.
In particular, summary/review-pack wording is P2 work and must keep
`review-unused` distinct from removal proof.

## Protected Invariants

- `unused-deps.json.schemaVersion` is `unused-deps.v1`.
- `policyVersion` is `unused-deps-review-policy-v1`.
- missing or unsupported `symbols.json.dependencyImportConsumers[]` support
  produces an unavailable artifact rather than false `review-unused` claims.
- observed external import consumers classify declarations as `used`.
- package script tool evidence can mute CLI/runtime dependencies such as `tsx`.
- `@types/*` declarations stay muted as ambient type packages.
- internal workspace package declarations stay muted.
- consumer files are attributed to the nearest package root.
- `review-unused` means "inspect this declaration", not "remove it".
- audit pipeline runs the producer and records `unused-deps.json` as an
  artifact.
- P1 does not surface dependency deletion claims in fix-plan, SARIF, summary
  Markdown, review-pack Markdown, or action-safety outputs.

## Edge-Case Failures To Preserve

- `node:fs`, `#internal`, URLs, data URLs, absolute paths, relative paths, and
  virtual specifiers must not become package names.
- `npm run start`, `npm start`, and `npm test` must not create package binary
  evidence.
- `bunx vite`, `npx eslint`, and `npm exec eslint` must create direct tool
  evidence for the named package.
- a root package must not mark a dependency used only because a child workspace
  package imports it.
- an unsupported symbols lane must not produce complete review-unused claims.

## Boundaries

- The Node suite remains runnable.
- Vitest remains a mirror for the reviewed P1 artifact contract.
- Shared helpers may write fixture files and run the audit pipeline, but must
  not decide package removal, package-manager operations, or action-safety
  proof.
- This mirror must not absorb package publishing tests, public install tests,
  resolver unsupported-family tests, dead-export ranking tests, or performance
  suites.

## Follow-Up Gates

Before P2:

- write a summary/review-pack wording spec;
- prove muted/review-unused evidence does not leak into `SAFE_FIX`,
  `EXISTS`, SARIF, or fix-plan entries;
- run at least one installed-package fixture after the wording change.

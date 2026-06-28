# Vitest Entry Surface Artifact Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-16.
> **Pilot candidate:** `tests/test-entry-surface-artifact.mjs`

---

## Purpose

This review decides whether `tests/test-entry-surface-artifact.mjs` may move to
a focused Lane D Vitest mirror. It does not add the Vitest suite.

The suite protects the `entry-surface.json` artifact and the quick audit hook
that feeds entry reachability and HTML-entry blind-zone evidence. It is safe to
review as a single-suite batch because its fixture boundary is narrower than
general resolver behavior: package exports, public re-export traversal, script
entries, HTML module scripts, framework/config sentinels, and HTML-entry
confidence labels.

This review must stay separate from broader resolver expansion, module
reachability ranking, dead-export action safety, generated/framework resource
surface packs, public package publishing, and performance/incremental cache
identity.

## Reviewed Evidence

| Suite                                   | Preserved Node Command                       | Proposed Focused Vitest Command              | Surface Under Review                |
| --------------------------------------- | -------------------------------------------- | -------------------------------------------- | ----------------------------------- |
| `tests/test-entry-surface-artifact.mjs` | `node tests/test-entry-surface-artifact.mjs` | `npm run test:vitest:entry-surface-artifact` | `entry-surface.json` and audit hook |

Current Node evidence checked for this review:

```text
node tests/test-entry-surface-artifact.mjs # 25 passed, 0 failed
```

Goal lane: Lane D, resolver/surface. This review covers only the entry-surface
artifact and the quick audit pipeline hook that produces it.

## Result

This suite is acceptable as one focused Vitest mirror.

The future implementation PR may add one mirror file and one focused script,
provided it keeps the Node entrypoint runnable and keeps every assertion tied to
entry-surface artifact evidence. The mirror must not turn entry-surface
uncertainty into deadness proof or SAFE_FIX evidence.

## Protected Invariants

The future Vitest mirror must preserve these contracts:

- `entry-surface.json.meta.tool` remains `build-entry-surface.mjs`;
- `publicApiFiles` includes package export roots;
- public re-export traversal adds transitive public re-export targets;
- `scriptEntrypointFiles` includes package-script source entries;
- `htmlEntrypointFiles` includes resolved HTML module script targets;
- `frameworkEntrypointFiles` includes Next app route files;
- `frameworkEntrypointFiles` includes Cloudflare Worker default exports from a
  `wrangler.toml` scope;
- `configEntrypointFiles` includes tool config files such as `vite.config.ts`;
- `entryFiles` is the union of public API, script, HTML, framework, and config
  entries;
- ordinary internals do not enter `entryFiles` only because they are present in
  the source tree;
- `evidenceByFile` preserves public re-export evidence for transitive targets;
- clean fixtures report `globalCompleteness: "high"`;
- clean fixtures report high `completenessBySubmodule` labels for local
  submodules;
- quick `audit-repo.mjs` runs `build-entry-surface.mjs`;
- quick audit artifacts list `entry-surface.json`;
- the pipeline-produced artifact preserves public API evidence;
- absolute HTML module paths that do not exist under the analyzed root are not
  promoted to phantom entry files;
- unresolved HTML module targets are recorded under
  `unresolvedHtmlEntrypoints[]`;
- unresolved HTML module targets lower entry-surface completeness to `medium`;
- unresolved HTML module targets create an `html-entry-surface` blind zone in
  `manifest.json`;
- HTML entry-surface blind zones prevent SAFE_FIX promotion for matching static
  asset exports;
- matching static asset exports stay visible as review fixes with
  `html-entry-surface-blind-zone` blocked-promotion evidence;
- nested HTML app roots resolve absolute `/src/...` module scripts relative to
  the HTML directory when that produces an existing file;
- nested HTML app roots do not prefer an unrelated repo-root `src/main.tsx`;
- extension probing does not leak missing candidates such as
  `apps/web/src/main.jsx` into `htmlEntrypointFiles`, `entryFiles`, or
  `evidenceByFile`;
- resolved nested HTML app roots keep `unresolvedHtmlEntrypoints[]` empty and
  `globalCompleteness: "high"`;
- excluded HTML files do not create unresolved entry-surface blind zones.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- dropping public re-export traversal must fail;
- failing to include script, HTML, framework, or config entries in the union
  must fail;
- adding ordinary internals to `entryFiles` without entry evidence must fail;
- losing the audit pipeline hook must fail;
- treating `/assets/app.js` as a root-relative source entry when the file is
  actually served from a static root outside the analyzer model must fail;
- silently claiming high completeness for unresolved HTML entrypoints must
  fail;
- turning HTML-entry uncertainty into SAFE_FIX promotion must fail;
- hiding blocked static-asset candidates from review evidence must fail;
- resolving nested Vite-style `/src/main.tsx` against the wrong root must fail;
- leaking a missing extension-probe candidate into entry files must fail;
- treating excluded corpus/output HTML as entry-surface evidence must fail.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The Node suite remains runnable and authoritative until a later cleanup spec
  retires it.
- The fixture boundary is temporary repo creation, package.json/script/html
  files, config files, framework sentinel files, direct producer invocation,
  quick audit invocation, and JSON artifact reads.
- Shared setup may write fixture files, run `build-symbol-graph.mjs`, run
  `build-entry-surface.mjs`, run `audit-repo.mjs`, read `manifest.json`,
  `entry-surface.json`, `module-reachability.json`, and `fix-plan.json`, and
  clean up temporary directories.
- Shared helpers must not decide resolver semantics, entry completeness,
  HTML-entry blind-zone relevance, SAFE_FIX promotion, framework sentinel
  policy, generated-surface policy, or reachability ranking.
- The mirror must not change `build-entry-surface.mjs`, resolver behavior,
  module reachability, rank/fix classification, audit summary wording, or
  public package behavior.
- The mirror must not absorb `tests/test-module-reachability.mjs`,
  `tests/test-rank-fixes.mjs`, `tests/test-export-action-safety.mjs`,
  generated/framework resource surface suites, Python convention suites,
  symlink resolver suites, or performance/incremental suites.

## Implementation Notes

- Prefer one Vitest file: `tests/entry-surface-artifact.test.mjs`.
- Add one focused script:
  `test:vitest:entry-surface-artifact`.
- Keep assertion labels close to the Node labels `E1` through `E24`.
- Keep the static-root mismatch fixture explicit; it is the proof that missing
  HTML static-server mapping becomes a confidence limit rather than a phantom
  entry.
- Keep the nested HTML fixture explicit; it is the proof that `/src/main.tsx`
  can be resolved relative to the HTML directory when the file exists there.
- Keep excluded HTML coverage explicit; it prevents scan-policy exclusions from
  being reported as unresolved entrypoints.

## Validation Commands

The implementation PR must run:

```text
node tests/test-entry-surface-artifact.mjs
npm run test:vitest:entry-surface-artifact
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-entry-surface-artifact.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/test-migration-candidate-board.md
git diff --check
```

Before merge, the implementation should also keep the broader runner lane
green:

```text
npm run check
npm run lint
npm run test:vitest
npm test
```

## Non-Goals

- Do not add resolver expansion.
- Do not change HTML path resolution behavior.
- Do not broaden framework sentinel policy.
- Do not add generated/resource capability packs.
- Do not change dead-export ranking or action-safety proof.
- Do not treat entry-surface evidence as automatic deletion proof.

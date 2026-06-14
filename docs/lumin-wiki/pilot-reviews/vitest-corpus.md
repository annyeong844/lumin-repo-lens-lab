# Vitest Precision Corpus Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-22.
> **Pilot candidate:** `tests/test-corpus.mjs`.

---

## Purpose

This review records the focused Vitest mirror for `tests/test-corpus.mjs`. The
suite is the release-blocking deadness precision corpus: it builds many small
repositories, runs the real analyzer pipeline, and enforces a zero
false-positive budget across dynamic imports, AST reference counting, resolver
taint, package/public surface rules, framework conventions, declaration
dependencies, and ranking action proof.

The key risk is that a broad mirror could preserve only "the corpus ran" while
dropping which precision family failed. That would make false-positive budget
regressions harder to localize and could let review-only evidence become
automated cleanup proof.

## Reviewed Evidence

| Suite                   | Preserved Node Command       | Proposed Focused Vitest Command | Surface Under Review                                                  |
| ----------------------- | ---------------------------- | ------------------------------- | --------------------------------------------------------------------- |
| `tests/test-corpus.mjs` | `node tests/test-corpus.mjs` | `npm run test:vitest:corpus`    | release-blocking precision corpus and zero false-positive budget gate |

Goal lane: deadness/ranking precision corpus. This is a suite-specific review
for the accepted synthetic corpus, not permission to migrate P6 calibration,
rank-fixes, export-action-safety, namespace re-export, cue-tier, or audit-repo
umbrella behavior.

Fresh preserved-command evidence on 2026-05-22:

```text
node tests/test-corpus.mjs
78 passed, 0 failed

FP budget gate
precision failures: 0
budget:             0
gate:               PASS
```

## Result

This suite now has a narrow Vitest mirror at `tests/corpus.test.mjs`, and the
preserved Node command remains runnable. The mirror shares setup-only helpers
for temporary repo creation, nested file writes,
producer execution, JSON artifact reads, cleanup, and simple assertion grouping.
It runs the real `build-symbol-graph.mjs`, `classify-dead-exports.mjs`,
`export-action-safety.mjs`, and `rank-fixes.mjs` producers for the corpus cases.

The mirror must not extract helper logic that decides dynamic import opacity,
AST reference counts, resolver taint locality, public API mutes, package import
semantics, script/framework entrypoint policy, declaration dependency safety,
framework route conventions, or the false-positive budget gate.

## Protected Invariants

The Vitest mirror preserves these 78 contracts plus the zero-FP budget gate:

### Dynamic Import Opacity

- CASE-FP18B.1: symbols emit dynamic import opacity target directory evidence.
- CASE-FP18B.2: a dynamic command export is not review-visible cleanup.
- CASE-FP18B.3: the dynamic command export is `MUTED` with FP-18 evidence.
- CASE-FP18B.4: classifier summary counts `dynamicImportOpacity_FP18`.
- CASE-FP18B.5: an unrelated private export remains review-visible.

### AST Reference Counting

- CASE-AST.1: `deadOnly` is in the classified list.
- CASE-AST.2: `deadOnly` is Class C with zero refs, not inflated by comment or
  string text.
- CASE-AST.3: `deadOnly` evidence is `ast-ident-ref-count`, not regex.

### Resolver Taint Locality

- CASE-P1.1: the fixture has high resolver blindness and forces the scenario.
- CASE-P1.2: `AuthControl` outside the known alias target is not `DEGRADED`.
- CASE-P1.3: `AuthControl` reason does not cite unresolved spec taint.
- CASE-P1.4: clean `logger.log` is not `DEGRADED`.

### Test-Pinned Contracts

- CASE-FP44.1: test-pinned `contractHelper` is not review-visible cleanup.
- CASE-FP44.2: `contractHelper` materializes as `MUTED` `testConsumer_FP44`.
- CASE-FP44.3: export-manifest pinned symbols also materialize as `MUTED`.
- CASE-FP44.4: `trulyDead` remains review-visible cleanup.

### Package Export Barrels

- CASE-FP40.1: `barrelExport` from `./dist/index.mjs` entry is not dead-listed.
- CASE-FP40.2: `barrelFn` from `./dist/index.mjs` entry is not dead-listed.
- CASE-FP40.3: `barrelExport` is not proposed for removal.
- CASE-FP40.4: `trulyDead` is dead-listed, proving the fixture actually runs the
  analysis.
- CASE-FP40B.string-root: root export string form marks `src/index.ts` as a
  barrel.
- CASE-FP40B.conditional-root: root export conditional form marks
  `src/index.ts` as a barrel.

### JSX Value References

- CASE-FP41.1: live `AlertDialog` is not in any dead bucket.
- CASE-FP41.2: `AlertDialogTrigger` appears in classifier output.
- CASE-FP41.3: `AlertDialogTrigger` has exactly one file-internal use from JSX.
- CASE-FP41.4: the trigger JSX use is tracked as a value reference.
- CASE-FP41.5: the trigger is Class A remove-export, not Class C completely
  dead.
- CASE-FP41.6: evidence label remains `ast-ident-ref-count`.

### Public Package Surface

- CASE-P6-1.1: root export `publicEntry` is not review-visible cleanup.
- CASE-P6-1.2: type-only public subpath is `MUTED` as `publicApi_FP23`.
- CASE-P6-1.3: unrelated private export remains review-visible.
- CASE-P6-1a.1: `package.imports` exact target is not counted as
  `publicApi_FP23`.
- CASE-P6-1a.2: `#imports`-only dead export is not `MUTED` as public API.
- CASE-P6-1a.3: `#imports`-only dead export remains review-visible.
- CASE-P6-1b.1: script entrypoint function is not review-visible cleanup.
- CASE-P6-1b.2: script entrypoint function is `MUTED` with FP45 evidence.
- CASE-P6-1b.3: all exports in the script entry file are muted.
- CASE-P6-1b.4: unrelated private export remains review-visible.
- CASE-P6-1d.1: sibling `.d.ts` type import contributes fan-in.
- CASE-P6-1d.2: source declaration type is not dead-listed.
- CASE-P6-1d.3: source declaration type is not review-visible cleanup.
- CASE-P6-1d.4: public script entry still gets muted by entrypoint policy.
- CASE-P6-1d.5: unrelated private export remains review-visible.
- CASE-P6-1e.1: package import fan-in lands on source function.
- CASE-P6-1e.2: package type import fan-in lands on source interface.
- CASE-P6-1e.3: imported public source exports are not review-visible cleanup.
- CASE-P6-1e.4: unreferenced export in package public source file is not
  review-visible cleanup.
- CASE-P6-1e.5: unrelated private export remains review-visible.
- CASE-P6-1f.1: declaration sidecar is not review-visible cleanup.
- CASE-P6-1f.2: declaration sidecar is `MUTED` with FP48 evidence.
- CASE-P6-1f.3: unrelated private export remains review-visible.

### Framework And HTML Entrypoints

- CASE-P6-1c.1: VitePress config is `MUTED` by convention.
- CASE-P6-1c.2: VitePress theme index is `MUTED` by convention.
- CASE-P6-1c.3: HTML module main entrypoint is `MUTED` with evidence.
- CASE-P6-1c.4: unrelated `.vitepress` helper is still review-visible.
- CASE-P6-1c.5: unrelated app private export remains review-visible.

### Framework Policy Scope

- CASE-FRAMEWORK-POLICY-1.1: root Next app route is `MUTED` by framework
  policy.
- CASE-FRAMEWORK-POLICY-1.2: root Next proxy export is `MUTED` by framework
  policy.
- CASE-FRAMEWORK-POLICY-1.3: nested non-Next app route stays visible.
- CASE-FRAMEWORK-POLICY-1.4: nested app middleware path stays visible.
- CASE-FRAMEWORK-POLICY-1.5: phase-1 framework counters are emitted.
- CASE-FRAMEWORK-POLICY-2.1: Hono local route handler is `MUTED` by route fact.
- CASE-FRAMEWORK-POLICY-2.2: unrelated Hono helper remains visible.
- CASE-FRAMEWORK-POLICY-2.3: SvelteKit load and dynamic entries are `MUTED`.
- CASE-FRAMEWORK-POLICY-2.4: ordinary SvelteKit route helper remains visible.
- CASE-FRAMEWORK-POLICY-2.5: Astro endpoint `GET` is `MUTED` but helper remains
  visible.
- CASE-FRAMEWORK-POLICY-2.6: React Router loader is `MUTED` while
  `clientLoader` is review-visible.
- CASE-FRAMEWORK-POLICY-2b.1: non-workspace nested SvelteKit load is `MUTED` by
  nearest package.
- CASE-FRAMEWORK-POLICY-2b.2: non-workspace nested SvelteKit dynamic entries are
  `MUTED`.
- CASE-FRAMEWORK-POLICY-2b.3: ordinary nested SvelteKit route helper remains
  visible.
- CASE-FRAMEWORK-POLICY-2b.4: non-workspace nested Astro endpoint `GET` is
  `MUTED` by nearest package.
- CASE-FRAMEWORK-POLICY-2b.5: ordinary nested Astro helper remains visible.
- CASE-FP30-SCOPE.1: `h3` alone does not activate Nuxt/Nitro mute policy.
- CASE-FP30-SCOPE.2: Nest-style middleware helper is not `MUTED` as
  `nuxtNitro_FP30`.
- CASE-FP30-SCOPE.3: ordinary middleware helper remains review-visible.

### Declaration Surface Safety

- CASE-DECL-1.1: exported class/const signature type is demote-only `SAFE_FIX`.
- CASE-DECL-1.2: exported class/const signature type is not `DEGRADED` when
  demote preserves binding.
- CASE-DECL-1.3: unrelated private export remains review-visible.

## Edge-Case Failures To Preserve

The mirror must fail if:

- a dynamic import template with a static directory prefix leaves command
  exports review-visible;
- comments or strings inflate identifier references;
- repo-global resolver blindness degrades unaffected candidates;
- test-pinned or export-manifest pinned symbols become cleanup candidates;
- package `exports` barrels, string roots, or conditional roots are dead-listed;
- JSX identifiers are ignored or counted as type-only;
- package `imports` are treated as public API;
- package script entries, HTML module entries, declaration sidecars, or public
  package source mappings become cleanup candidates;
- framework policy applies outside the package that owns the framework;
- framework route facts stop muting framework-owned route handlers;
- `h3` or Nest-style paths activate Nuxt/Nitro policy without Nuxt/Nitro;
- declaration dependency safe actions become delete actions or degraded
  findings;
- the final false-positive budget is raised above zero.

## Fixture Boundary

Allowed shared helpers:

- create and remove temporary repository directories;
- write nested files;
- run the real producer pipeline with normal or production flags;
- read `symbols.json`, `dead-classify.json`, and `fix-plan.json`;
- collect cleanup candidates, cleanup symbols, identities, and entries;
- group assertions by corpus case.

Forbidden helper behavior:

- deciding whether an export is live, muted, review-visible, or safe;
- deciding dynamic import opacity or target directory families;
- deciding resolver taint locality;
- deciding public API, package export, package import, or source mapping meaning;
- deciding framework package ownership or route convention eligibility;
- deciding JSX value-space references;
- deciding declaration dependency action safety;
- deciding or mutating the FP budget.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The preserved Node command remains runnable and authoritative until a later
  cleanup spec retires it.
- The mirror must not change analyzer producers, classifier policy, framework
  policy, public-surface detection, resolver behavior, ranking, or fix
  application.
- The mirror must not absorb P6 measurement/member/safe-fix calibration,
  rank-fixes, export-action-safety, namespace re-export, cue-tier policy, or
  audit-repo umbrella behavior.
- The mirror must not split away the final FP budget gate; the gate and all
  corpus cases must stay visible in the focused command.

## Recommendation

The narrow implementation PR added:

1. `tests/corpus.test.mjs`;
2. `npm run test:vitest:corpus`;
3. candidate-board updates moving this suite from `REVIEWED` to `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the 78 current Node
assertions as named Vitest cases plus the zero-FP budget gate. It remains
covered by the preserved Node command, the focused Vitest command, and the doc
guards.

## Validation Commands

The implementation PR must run:

```text
node tests/test-corpus.mjs
npm run test:vitest:corpus
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check docs/lumin-wiki/pilot-reviews/vitest-corpus.md docs/lumin-wiki/index.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md
git diff --check
```

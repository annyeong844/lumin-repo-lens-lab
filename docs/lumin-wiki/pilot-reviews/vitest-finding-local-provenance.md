# Vitest Finding-Local Provenance Pilot Review

> **Status:** DONE.
> **Date:** 2026-05-21.
> **Pilot candidate:** `tests/test-finding-local-provenance.mjs`.

---

## Purpose

This review decides whether `tests/test-finding-local-provenance.mjs` may move
to a focused Vitest mirror. The suite protects the v1.10.0 finding-local
provenance fix: resolver blindness, parse errors, and generated-artifact blind
zones must taint only the affected finding instead of demoting every finding in
the repository.

The key risk is that a broad mirror could preserve only "ranking still runs"
while dropping the local taint evidence that keeps clean findings promotable to
`SAFE_FIX` even when another submodule is resolver-blind.

## Reviewed Evidence

| Suite                                     | Preserved Node Command                         | Proposed Focused Vitest Command                | Surface Under Review                                               |
| ----------------------------------------- | ---------------------------------------------- | ---------------------------------------------- | ------------------------------------------------------------------ |
| `tests/test-finding-local-provenance.mjs` | `node tests/test-finding-local-provenance.mjs` | `npm run test:vitest:finding-local-provenance` | finding-local taint, resolver confidence, and ranking tier effects |

Goal lane: deadness/ranking provenance lens. This is a suite-specific review
for finding-local taint and ranking-tier behavior, not permission to migrate
corpus, rank-fixes, export-action-safety, P6 calibration, or generic ranking
policy.

Fresh preserved-command evidence on 2026-05-21:

```text
node tests/test-finding-local-provenance.mjs
48 passed, 0 failed
```

## Result

This suite now has a narrow Vitest mirror at
`tests/finding-local-provenance.test.mjs`. The Node command remains
authoritative until a later cleanup spec retires it. The mirror shares
setup-only fixtures for alias maps, `submoduleOf()`, and synthetic
findings/evidence objects, and calls the real
`specifierCouldMatchFile()`, `computeFindingProvenance()`, and
`tierForFinding()` implementations.

It must not extract helper logic that decides whether a specifier could match a
file, whether a taint is blocking or soft, whether resolver confidence is high,
medium, or low, or whether a ranking tier should degrade.

## Protected Invariants

The Vitest mirror preserves these 48 contracts:

### Specifier Matching

- S1: a known alias matches only inside its alias scope.
- S2: a known alias does not taint a different scope.
- S3: a known alias does not match an unrelated target in scope.
- S4: a bare specifier without a slash does not match anything.
- S5: an unknown alias-like specifier is unknown only in the same submodule.
- S6: an unknown alias-like specifier does not taint another submodule.
- S7: a baseUrl-like specifier is unknown in the matching baseUrl scope.
- S8: a baseUrl-like specifier does not taint outside the matching scope.
- S9: a relative specifier matches the importer-normalized path.
- S10: a Windows-style backslash target path is normalized before matching.

### Finding Provenance

- P1: a clean finding has no taint.
- P1b: a clean finding has `resolverConfidence: "high"`.
- P1c: a clean finding has `parseStatus: "ok"` when its file is not in the
  parse-error list.
- P1d: clean AST evidence records `ast-ident-ref-count` in `supportedBy`.
- P2: parse errors elsewhere emit a `parse-errors-present` taint.
- P2b: only soft parse-error taint lowers resolver confidence to medium.
- P2c: a parse error in an unrelated submodule does not taint the finding.
- P2d: a parse error in the same submodule remains a relevant soft taint.
- P3: a parse error in the defining file emits
  `defining-file-parse-error`.
- P3b: a defining-file parse error sets `parseStatus: "error"`.
- P3c: a defining-file parse error lowers resolver confidence to low.
- P4: a scoped alias unresolved specifier emits
  `unresolved-specifier-could-match`.
- P4b: the matching unresolved specifier is listed.
- P4c: non-matching unresolved specifiers are not listed.
- P4d: a blocking unresolved specifier match lowers resolver confidence to
  low.
- P4e: an unknown alias in the same submodule emits weak unresolved taint.
- P4f: weak unresolved taint records the consumer file.
- P4g: weak unresolved taint is medium confidence, not low.
- P4h: an unknown alias from another submodule does not taint the finding.
- P5: the affected file is tainted by the unresolved specifier.
- P5b: the unaffected file stays clean in the same repo.
- P6: a generated artifact miss in the candidate package emits relevant soft
  taint.
- P6b: generated artifact relevant taint lowers resolver confidence to medium.
- P7: an unrelated generated artifact miss does not taint another package.
- P8: a generated provider miss in the consumer submodule alone stays clean.
- P9: a generated consumer blind zone emits consumer-surface taint.

### Ranking Tier Effects

- T1: empty `taintedBy` plus strong evidence promotes to `SAFE_FIX`.
- T2: `unresolved-specifier-could-match` degrades strong evidence to
  `DEGRADED`.
- T2b: the degradation reason surfaces the matching specifier.
- T2c: `unresolved-specifier-could-match-unknown` demotes `SAFE_FIX` to
  `REVIEW_FIX`.
- T3: `defining-file-parse-error` degrades to `DEGRADED`.
- T4: `parse-errors-present` demotes `SAFE_FIX` to `REVIEW_FIX`.
- T4b: the soft parse-error reason mentions parse errors elsewhere.
- T4c: a relevant generated artifact miss demotes `SAFE_FIX` to `REVIEW_FIX`.
- T4d: the generated-artifact reason mentions `generated-artifact-missing`.
- T5: a clean finding in a high-global-ratio repo still promotes to
  `SAFE_FIX`.
- T6: a legacy finding without `taintedBy` falls back to the global resolver
  ratio gate.
- T6b: the legacy fallback reason mentions `resolver-blind`.

## Edge-Case Failures To Preserve

The mirror must fail if:

- alias matching becomes repo-global instead of scope-local;
- unknown aliases taint unrelated submodules;
- Windows paths stop normalizing before alias comparison;
- parse errors elsewhere become blocking taint;
- parse errors in unrelated submodules taint clean findings;
- unresolved specifiers demote unrelated files through a repo-global ratio;
- generated artifact misses taint unrelated packages;
- generated consumer blind zones lose their consumer-surface impact;
- weak unresolved taint becomes low confidence instead of medium confidence;
- a clean finding with explicit `taintedBy: []` falls back to the old global
  unresolved-ratio gate;
- legacy findings without `taintedBy` stop preserving backward-compatible
  global resolver-blind behavior.

## Fixture Boundary

Allowed shared helpers:

- build the scoped alias/baseUrl map used by the suite;
- provide a path-normalizing `submoduleOf()` helper;
- construct small finding objects;
- construct small evidence objects for parse errors, unresolved specifiers,
  generated artifact misses, generated consumer blind zones, and strong ranking
  evidence;
- call `specifierCouldMatchFile()`, `computeFindingProvenance()`, and
  `tierForFinding()`;
- assert `taintedBy`, `supportedBy`, `resolverConfidence`, `parseStatus`,
  ranking tier, and ranking reason fields.

Forbidden helper behavior:

- deciding whether a specifier matches a file;
- deciding whether a taint is relevant to the finding;
- deciding whether a taint is blocking or soft;
- deciding resolver confidence;
- deciding ranking tier or reason text;
- hiding the P1 win where clean findings stay `SAFE_FIX` in a high global
  unresolved-ratio repo;
- sharing semantic helper logic with corpus, rank-fixes, export-action-safety,
  P6 calibration, resolver blind-zone, or module reachability suites.

## Boundaries

- Vitest remains a dev-only mirror runner.
- The preserved Node command remains runnable and authoritative until a later
  cleanup spec retires it.
- The mirror must not change `_lib/finding-provenance.mjs`,
  `_lib/ranking.mjs`, dead-export classification, action safety, rank-fixes,
  corpus budgets, P6 calibration, resolver diagnostics, or module reachability.
- The mirror must not absorb `tests/test-corpus.mjs`,
  `tests/test-rank-fixes.mjs`, `tests/test-export-action-safety.mjs`, P6
  suites, or `tests/test-audit-repo.mjs`.
- The mirror must not convert local taint evidence into broader ranking policy
  changes.

## Recommendation

The narrow implementation PR added:

1. `tests/finding-local-provenance.test.mjs`;
2. `npm run test:vitest:finding-local-provenance`;
3. candidate-board updates moving this suite from `REVIEWED` to `DONE`.

The implementation first watched the focused Vitest command fail because the
script was missing, then added a mirror that preserves the 48 current Node
assertions as named Vitest cases. It remains covered by the preserved Node
command, the focused Vitest command, and the doc guards.

## Validation Commands

The implementation PR must run:

```text
node tests/test-finding-local-provenance.mjs
npm run test:vitest:finding-local-provenance
npm run check:test-doc
npm run check:doc-script-refs
npx prettier --check tests/finding-local-provenance.test.mjs package.json docs/lumin-wiki/pilot-reviews/vitest-finding-local-provenance.md docs/lumin-wiki/log.md docs/lumin-wiki/vitest-mirror-goal.md docs/lumin-wiki/vitest-mirror-closure-audit.md tests/README.md
git diff --check
```

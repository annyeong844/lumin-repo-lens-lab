# Grounded Audit Self-Audit Handbook

Use this only when reviewing `lumin-repo-lens-lab` itself. Do not
load it for ordinary user repositories. It layers maintainer-specific
dogfood checks on top of `templates/REVIEW_CHECKLIST.md`.

## B2 Vocabulary Drift

- `_lib/vocab.mjs` is the source of truth for evidence, taint, and delta
  vocabulary.
- `_lib/ranking.mjs::TIERS` owns the public ranking tier literals:
  `SAFE_FIX`, `REVIEW_FIX`, `DEGRADED`, `MUTED`.
- Check that new inline string literals do not duplicate these owners.

Suggested scan:

```bash
rg -n "'(SAFE_FIX|REVIEW_FIX|DEGRADED|MUTED)'" -- *.mjs _lib/*.mjs
```

Hits outside `_lib/ranking.mjs` and `_lib/vocab.mjs` are review
candidates, not automatic defects. Confirm whether the literal is a
public output fixture, a test pin, or a real duplicated owner.

## B4 Pipeline Ownership

- `audit-repo.mjs` is the recommended user-facing orchestrator.
- Public wrappers and lifecycle modes should delegate to it or to the
  narrow lifecycle owner; they should not independently rebuild the same
  file-walk -> parse -> analyze -> emit pipeline.

## D1 JS Contract Owners

Because the engine is `.mjs`, D1 focuses on documented runtime shapes
and vocabulary owners rather than TypeScript declarations:

- `_lib/vocab.mjs` for shared labels.
- `_lib/ranking.mjs::TIERS` and `tierForFinding` for fix-plan tiers.
- Artifact parser modules for JSON shape contracts.

## D5 Variant Shape Watch

`_lib/ranking.mjs::tierForFinding` evidence may carry optional groups
such as `runtime`, `staleness`, `resolver`, and `policy`. When editing
this area, check whether a discriminated union-style shape would make
the active evidence branch clearer.

## F3 Contract-Level Tests

- `test-corpus.mjs` is contract-level.
- `test-classify-facts-ast.mjs` exercises AST counting internals, but
  assertions should remain tied to externally visible classification
  output.

## F4 Mock Boundary

The preferred testing style is real temporary fixtures plus public
scripts:

- Build fixture repos under `tmpdir()`.
- Run CLI scripts with `execFileSync` / shell-safe argv arrays.
- Avoid deep mocks unless the dependency is external, slow, or
  intentionally unavailable in CI.

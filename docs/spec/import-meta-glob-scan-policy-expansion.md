# Import Meta Glob Scan-Policy Expansion

> **Role:** maintainer-facing design spec for moving WT-17 from
> `import.meta.glob(...)` unsupported diagnostics toward bounded concrete
> dynamic-module edges.
> **Status:** SPEC. No implementation is authorized until the P1 fixture matrix
> below is added.
> **Last updated:** 2026-05-24.

## Problem

WT-17 currently handles literal `import.meta.glob("./routes/*.ts")` honestly:
`build-symbol-graph.mjs` records an unsupported `dynamic-modules` diagnostic,
`resolver-diagnostics.json` exposes a candidate-scoped blind zone, and no
concrete graph edge is created. That is safer than pretending a broad dynamic
surface was resolved.

The next useful step is not "expand every glob." That would be unsafe. A Vite
glob can represent routing, lazy chunks, eager imports, generated pages, or
framework-private conventions. Expansion is only safe when the pattern, scan
range, and target files all stay inside the same audited source policy.

This spec defines the narrow conditions where Lumin may turn a literal glob
into concrete file-level dynamic edges, and the conditions where it must keep
the existing unsupported diagnostic.

## Current Contract

The current diagnostic-only behavior is pinned by:

- [`tests/test-import-meta-glob-diagnostics.mjs`](../../tests/test-import-meta-glob-diagnostics.mjs)
- [`tests/import-meta-glob-diagnostics.test.mjs`](../../tests/import-meta-glob-diagnostics.test.mjs)
- [`docs/lumin-wiki/pilot-reviews/vitest-import-meta-glob-diagnostics.md`](../lumin-wiki/pilot-reviews/vitest-import-meta-glob-diagnostics.md)
- WT-17 in [`lumin-work-tracker.md`](lumin-work-tracker.md)

Those tests are still correct for unsupported shapes. The implementation PR
must not weaken them by making every literal glob resolved. It must split the
contract into "supported literal glob" and "unsupported diagnostic" cases.

## Goals

1. Expand a tiny supported subset of literal `import.meta.glob(...)` patterns
   into concrete dynamic edges.
2. Preserve candidate-scoped unsupported diagnostics for every unsupported
   shape.
3. Respect scan policy: excluded files, files outside the audit root, and
   non-source targets must not become graph edges.
4. Keep expansion deterministic and capped.
5. Record enough evidence to explain whether a glob was expanded or left
   unsupported.

## Non-Goals

- Do not support non-literal glob expressions.
- Do not evaluate template expressions, variables, arrays of patterns, negative
  patterns, or runtime options in the first slice.
- Do not infer framework semantics such as route priority or eager/lazy
  execution.
- Do not promote dynamic edges into `SAFE_FIX`, `EXISTS`, or deadness proof.
- Do not make expansion repo-global. The surface remains tied to the consumer
  file and the matched source files.

## P1 Supported Shape

P1 may expand only this shape:

```ts
const routes = import.meta.glob("./routes/*.ts");
```

Required constraints:

- the callee is exactly `import.meta.glob`;
- the first argument is a single string literal;
- the pattern is relative and starts with `./` or `../`;
- the pattern contains exactly one `*`;
- `*` appears in a single path segment;
- the suffix is a supported JS/TS source extension;
- every matched file is inside the audited root;
- every matched file is inside the same scan range as ordinary source files;
- match count is greater than zero and less than or equal to the cap;
- matches are sorted deterministically by repo-relative path.

If any constraint fails, keep the existing unsupported diagnostic.

## Scan Policy

Glob expansion must use the same source policy as the producer's normal file
collection. A file excluded from the audit must not become a dynamic edge just
because a glob pattern names it.

The implementation may satisfy this by passing a `scannedSourceFileSet` into
the expander. It must not run a separate broad filesystem glob that ignores
`includeTests`, `exclude`, package boundaries, or generated/resource filters.

The graph identity for an expanded glob includes:

- consumer file;
- raw glob specifier;
- sorted matched target files;
- edge kind `dynamic-import-meta-glob`;
- scan policy version;
- expansion cap.

Changing scan policy can therefore change edges without changing file content.

## Output Contract

For a supported expanded glob, `symbols.json` records:

- `resolvedInternalEdges[]` entries with:
  - `kind: "dynamic-import-meta-glob"`;
  - `source` equal to the raw glob specifier;
  - `from` equal to the consumer file;
  - `to` equal to each matched source file.
- an optional diagnostic/evidence record that names:
  - `resolverStage: "import-meta-glob"`;
  - `outputLevel: "resolved"`;
  - `unsupportedFamily` absent;
  - `matchCount`;
  - `scanPolicy: "source-file-set"`;
  - `cap`.

For an unsupported glob, the current diagnostic contract stays:

- `symbols.json.unresolvedInternalSpecifierRecords[]`;
- `resolver-diagnostics.json.unsupportedImports[]`;
- `resolver-diagnostics.json.blindZones[]`;
- `unsupportedFamily: "dynamic-modules"`;
- no concrete graph edge.

## Cap And Fallback

P1 cap: `64` matched files.

If a literal glob matches more than the cap, do not emit partial concrete edges.
Emit an unsupported diagnostic with a reason such as
`import-meta-glob-match-cap-exceeded`, preserving `matchCount` when cheap to
compute. Partial edges would be worse than no edges because they imply false
precision.

Zero matches also stay diagnostic-only. A zero-match glob can be a typo, an
environment-specific route surface, or a generated path; it is not proof that
no dynamic module exists.

## Test Matrix

P1 needs Node and Vitest coverage before implementation is accepted:

1. Supported literal glob expands `./routes/*.ts` into two concrete dynamic
   edges when both route files are in the scan set.
2. Existing unsupported diagnostic fixture remains unsupported when the test is
   intentionally configured as a diagnostic-only shape.
3. A glob matching an excluded file emits no concrete edge for that file.
4. A zero-match glob remains unsupported and candidate-scoped.
5. A non-literal glob remains unsupported.
6. A broad glob above the cap remains unsupported and emits no partial edges.
7. Resolver diagnostics continue to surface unsupported shapes while excluding
   successfully expanded shapes from `unsupportedImports[]`.

## Implementation Notes

Recommended split:

1. Add a pure helper such as `_lib/import-meta-glob-expansion.mjs`.
2. Feed it `consumerFile`, `pattern`, `root`, `scannedSourceFileSet`, and a cap.
3. Keep `extract-ts.mjs` responsible only for detecting `import.meta.glob`
   syntax and carrying the literal pattern.
4. Let `build-symbol-graph.mjs` decide whether the detected glob becomes
   concrete edges or the existing unsupported diagnostic.
5. Keep `resolver-capabilities.mjs` and `resolver-blind-zone-relevance.mjs`
   unchanged for unsupported shapes.

The helper must be boring: no framework route rules, no minimatch option zoo,
and no guessing. If the pattern is outside P1, return an explicit unsupported
reason.

## Acceptance Gate

The implementation PR must run:

```text
node tests/test-import-meta-glob-diagnostics.mjs
npm run test:vitest:import-meta-glob-diagnostics
node tests/test-resolver-diagnostics-artifacts.mjs
npm run test:vitest:resolver-diagnostics-artifacts
```

It must also add the new supported-expansion fixtures before changing the
current diagnostic expectations. Expansion must not land without fixture
coverage.

# Structure Review Charter

This charter is the shared review lens for Lumin code, tests, and wiki
refactors. It keeps shape, function, and helper decisions explicit so a cleanup
does not become another hidden contract.

## Review Order

Use this order before judging line count or style:

1. Structure boundaries
2. Types and contracts
3. Failure handling
4. Size and simplicity
5. Duplication and shape management
6. Abstraction and tests

Boundaries come first because a small function with the wrong dependency
direction can cost more than a long function with one clear responsibility.

## Finding Format

Every structural finding should name three things:

- Symptom: what is visible in the code.
- Cause: which boundary, contract, or workflow made it happen.
- First fix: the smallest change that reduces future cost without hiding the
  original risk.

Example:

```text
Symptom: the function is long.
Cause: validation, normalization, and state mutation are mixed in one path.
First fix: extract the normalization contract and keep state mutation at the
single existing entry point.
```

## Shape Charter

A shape is a repeated data, artifact, fixture, or workflow structure. A shared
shape may be extracted only when the protected invariant remains visible.

Record these facts when introducing or changing a shared shape:

- Owner: which module or suite owns the meaning.
- Consumers: which files read or write it.
- Completeness state: whether an empty array means proven empty, unavailable,
  or not computed.
- Failure mode: which edge case should fail if the shape drifts.
- Boundary: which claims the shape must not make.

Do not merge shapes only because they look similar. Resolver blind zones,
deadness blockers, public package surfaces, and performance counters may share
temporary repo setup while still protecting different claims.

## Function Charter

A function should have one reason to change. Long functions are acceptable only
when they are a dense pipeline with one contract and local state that is hard to
split safely.

Review functions for:

- mixed validation, normalization, graph mutation, rendering, and I/O
- temporal coupling, where callers must know a hidden call order
- feature envy, where the function knows another module's internals
- fallback paths that rewrite the real failure into a misleading reason
- hidden shared state or mutation outside a single predictable entry point

When splitting, preserve the current evidence contract first. Do not extract a
helper that turns a specific diagnostic into a generic success or failure.

## Helper Charter

A helper is allowed when it removes repeated setup without owning analyzer
semantics.

Good helpers:

- create temporary roots, output directories, and files
- normalize paths at the boundary
- write and read JSON with stable formatting
- guarantee cleanup through a narrow, auditable path
- make edge-case fixtures shorter without hiding the edge case

Bad helpers:

- decide whether evidence is SAFE_FIX, review-only, unsupported, or resolved
- collapse different resolver families into one anonymous setup path
- swallow errors or replace them with unrelated reasons
- require a hidden call order that tests do not enforce
- become a boundary-less utility bucket

Every new helper needs at least one contract test that would fail on an edge
case, not only because the helper file is missing.

## Anti-Patterns To Name

Use these names in reviews when they fit:

- God object or mega interface
- helper zoo
- hidden shared state
- catch and ignore
- fallback masks real failure
- shared shape drift
- over-split files without split responsibility
- boundary-less util
- temporal coupling
- feature envy
- stringly typed contract
- barrel bomb or import amplification

## Barrel Bomb Rule

An index or barrel file is risky when importing one symbol causes unrelated
modules to load transitively. This matters especially in Node.js ESM and other
runtime paths without tree-shaking.

When reviewing a barrel, ask:

- Does one import pull in the whole re-export graph?
- Are framework or generated surfaces being loaded as a side effect?
- Is this a public API convenience or an accidental import amplifier?

If the barrel is intentional, document the reason near the public surface. If
not, prefer direct imports or a narrower public entry.

## Test Charter

Tests should protect behavior and contracts, not implementation shape.

Prefer tests that:

- include prototype-name keys such as `constructor`, `toString`, `valueOf`, and
  `hasOwnProperty` when dictionary behavior matters
- distinguish empty, absent, unsupported, and not-computed states
- prove review evidence does not become SAFE_FIX proof
- preserve original negative assertions after helper extraction
- exercise cleanup and error boundaries explicitly

Avoid tests that pass only because a helper exists or because a happy path still
renders output.

# Vitest Parser And AST Guards Pilot Review

> **Status:** REVIEWED.
> **Date:** 2026-05-15.
> **Pilot candidates:**
>
> - `tests/test-classify-facts-ast.mjs`
> - `tests/test-lang-matrix.mjs`

---

## Purpose

This review decides whether two parser-adjacent guard suites can move together
as one Lane A Vitest mirror batch. It does not add Vitest suites. The goal is
to preserve parser dispatch and AST reference-counting contracts that keep
deadness facts grounded in parsed syntax rather than raw text or extension
guesswork.

The batch is acceptable because both candidates protect parser/helper behavior
before downstream deadness, resolver, ranking, or action-safety decisions use
the resulting facts:

- `test-classify-facts-ast.mjs` directly exercises
  `_lib/classify-facts.mjs` identifier reference counting from source text;
- `test-lang-matrix.mjs` directly exercises `_lib/lang.mjs` dispatch helpers
  and one small `build-symbol-graph.mjs` extension-matrix fixture;
- neither suite asserts `SAFE_FIX`, ranking, public API, generated, resolver,
  or pre/post-write behavior;
- both suites are regression-heavy and must keep their edge cases as named
  cases rather than broad happy-path smoke checks.

The future mirror should keep these suites focused on parser/AST facts. It
must not absorb broader dead-export classification, symbol action safety,
resolver expansion, topology, or producer performance semantics.

## Reviewed Evidence

| Suite                               | Preserved Node Command                   | Proposed Focused Vitest Command          | Module Or Surface Under Review                    |
| ----------------------------------- | ---------------------------------------- | ---------------------------------------- | ------------------------------------------------- |
| `tests/test-classify-facts-ast.mjs` | `node tests/test-classify-facts-ast.mjs` | `npm run test:vitest:classify-facts-ast` | `_lib/classify-facts.mjs` AST reference counter   |
| `tests/test-lang-matrix.mjs`        | `node tests/test-lang-matrix.mjs`        | `npm run test:vitest:lang-matrix`        | `_lib/lang.mjs` and mixed-extension symbol ingest |

Current suite descriptions are in `tests/README.md`.

Goal lane: Lane A, low-risk core/parser/helper parser and AST guards.

## Result

These suites are acceptable as one narrow Vitest mirror batch.

The future implementation PR may add both focused mirrors together because they
share a parser/AST fact boundary and protect syntax dispatch before analyzer
proof layers consume those facts. Each Node entrypoint must remain runnable,
and each mirror must keep edge-case assertions visible as named `it(...)`
blocks.

## Protected Invariants

The future Vitest batch must preserve these contracts:

- comments, block comments, string literals, template literal text, member
  properties, object keys, import specifier slots, and export specifier
  self-references do not count as local symbol references;
- real identifier uses, shorthand object values, calls, type references,
  `extends`, `new`, JSX tags, JSX member heads, JSX attribute expressions, and
  spread expressions count in the correct type/value lane;
- JSX attribute names, JSX member tails, JSX text, and JSX property slots do
  not count as local binding uses;
- scope-aware shadowing suppresses references from inner bindings including
  block bindings, parameters, catch parameters, loop bindings, named function
  expressions, and destructured parameters;
- exported declaration-surface references stay distinct from implementation
  body references;
- batch reference counting matches single-symbol semantics;
- `langForFile(...)`, `canContainJsx(...)`, and `nonJsLangForFile(...)`
  dispatch every supported JS/TS extension deterministically;
- `.jsx`, `.js` containing JSX, `.cjs`, `.mjs`, `.mts`, `.cts`, and `.d.ts`
  files are walked and parsed by the symbol graph fixture;
- mixed-extension imports preserve used/dead outcomes for `.cjs`, `.jsx`,
  `.js` with JSX, `.mts`, and declaration-only exports.

## Edge-Case Failures To Preserve

The migration must keep these failure modes visible:

- raw-text reference counting must fail if comments, strings, property names,
  or export specifiers inflate local references;
- JSX compound component usage must fail if the AST walker ignores
  `JSXIdentifier` or `JSXMemberExpression` heads;
- scope shadowing must fail if an inner binding is counted against the
  top-level export;
- exported declaration dependency evidence must fail if function/class/variable
  public type surfaces are confused with implementation bodies;
- batch counting must fail if parsing once for multiple candidates changes
  single-symbol reference counts;
- `.jsx` or JSX-in-`.js` parsing must fail loudly if extension dispatch
  regresses to TypeScript-only mode;
- mixed CJS/MJS/MTS/CTS resolution must fail if the symbol graph stops walking
  or resolving supported extension families;
- declaration-only `.d.ts` exports must fail if they stop appearing as
  definitions.

## Boundaries

- Vitest remains a dev-only dependency.
- Bun remains parked and is not required.
- The public package runtime remains Node-based.
- `scripts/run-tests.mjs` remains unchanged.
- `npm test` remains unchanged.
- Both preserved Node commands listed above remain runnable.
- The `classify-facts` mirror may share only assertion and fixture-string
  setup; it must not move AST-counting semantics into a test helper.
- The `lang-matrix` mirror may share temporary directory setup only; it must
  keep extension expectations and used/dead assertions local to the suite.
- The mirror must not add dead-export classification, action-safety,
  resolver-family, topology, performance, or public package assertions.
- The mirror must not widen `npm run test:vitest` discovery beyond reviewed
  first-party `tests/*.test.mjs` files.

## Recommendation

Proceed to one narrow implementation PR that adds:

1. `tests/classify-facts-ast.test.mjs`,
2. `tests/lang-matrix.test.mjs`,
3. focused `npm run test:vitest:*` commands for each suite,
4. candidate-board updates moving both suites from `REVIEWED` to `DONE`.

The implementation PR should first watch the focused Vitest commands fail
because the scripts or files are missing, then add mirrors that preserve every
current Node assertion as named Vitest cases. It should run the preserved Node
commands, focused Vitest commands, and `npm run test:vitest`.

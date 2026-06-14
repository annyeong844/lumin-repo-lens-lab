# CJS Consumer Hardening Design

> **Status:** design draft  
> **Last updated:** 2026-05-07  
> **Owner:** maintainer-facing accuracy work for `lumin-repo-lens-lab`

## 1. Problem

`lumin-repo-lens-lab` already extracts several CommonJS consumer patterns:

- `const { foo } = require("./x")`
- `const mod = require("./x"); mod.foo`
- `require("./x").foo()`
- side-effect-only `require("./x")`
- broad namespace escapes and re-export escapes
- dynamic `require(expr)` opacity

That is the right spine. The remaining risk is not "no CJS support"; it is that
mixed JS/CJS repositories can still create false dead-export confidence when a
consumer pattern is just outside the current exact model.

This design hardens the current model without turning Lumin into a full
CommonJS interpreter.

## 2. Goals

1. Keep exact CJS consumers exact only when binding identity is mechanically
   clear.
2. Degrade uncertain CJS flows to broad opacity or namespace escape rather than
   pretending the named export is unused.
3. Preserve the difference between symbol-level fan-in and file-level
   reachability.
4. Add regression fixtures based on real JS/CJS failure modes, especially pure
   JS repos and mixed module repos.
5. Avoid repo-specific rules such as package-name allowlists or one-off
   special cases for a single checkout.

## 3. Non-Goals

- No full CommonJS execution model.
- No constant propagation beyond literal `require` specifiers in this slice.
- No semantic equivalence or embedding-based duplicate detection.
- No new parser dependency unless the current OXC AST path cannot represent a
  required pattern.
- No broad "all CJS files are unsafe" downgrade. Exact supported CJS evidence
  should remain useful.

## 4. Existing Contract

The current contract stays:

| Pattern | Classification | Meaning |
|---|---|---|
| `require("./x")` statement | `cjs-side-effect-only` | file evaluation edge, not named fan-in |
| `const { foo } = require("./x")` | `cjs-require-exact` | exact consumer of `foo` |
| `const mod = require("./x"); mod.foo` | `cjs-namespace-member` | exact only while `mod` identity is clean |
| `const mod = require("./x"); use(mod)` | `cjs-namespace-escape` | broad consumer, not exact |
| `module.exports = require("./x")` | `cjs-reexport-broad` | broad re-export consumer |
| `require(expr)` | `cjsRequireOpacity` | dynamic CJS blind zone |

`SAFE_FIX` must not be promoted through relevant `cjs-namespace-escape`,
`cjs-reexport-broad`, or dynamic require opacity.

## 5. Hardening Scope

### 5.1 Exact Patterns To Preserve

These patterns should keep or gain exact named fan-in when the required
specifier is a string literal and the namespace binding is identity-clean:

```js
const { foo } = require("./x");
const { foo: localFoo } = require("./x");
const mod = require("./x");
mod.foo;
mod.foo();
mod["foo"];
const localFoo = mod.foo;
const { foo } = mod;
require("./x").foo;
require("./x")["foo"];
exports.foo = require("./x").foo;
module.exports.foo = require("./x").foo;
```

The last two are exact consumers of `./x`'s `foo`; they may also be CJS export
surface facts for the current file, but the consumer fact must not be lost.

### 5.2 Broad Patterns To Preserve

These patterns must remain broad, not exact:

```js
let mod = require("./x");
var mod = require("./x");
const mod = require("./x");
mod = other;
use(mod);
Object.keys(mod);
{ ...mod };
const { foo, ...rest } = mod;
"foo" in mod;
module.exports = require("./x");
exports.all = require("./x");
module.exports = makeExports();
exports[dynamicName] = require("./x").foo;
```

The core rule is conservative: if the analyzer cannot prove the member read
uses the original `const` require binding, it emits a broad CJS escape instead
of exact fan-in.

### 5.3 Scope And Shadowing

Exact namespace member reads require binding identity, not name equality.

```js
const mod = require("./x");
function f(mod) {
  mod.foo(); // not the outer require binding
}
```

This must not exact-protect `foo`. It should either be ignored as unrelated or
degrade the relevant CJS evidence only when the outer require binding actually
escapes.

The implementation should keep using lexical scope records. If a case cannot be
represented by the current lightweight scope model, it should degrade rather
than add ad hoc name matching.

### 5.4 Guard Reads

Truthiness or existence guards should not by themselves turn an otherwise exact
CJS member read into a broad escape:

```js
const mod = require("./x");
if (mod) mod.foo();
mod && mod.foo();
typeof mod !== "undefined" && mod.foo();
```

These guards inspect whether the namespace object exists; they do not enumerate
or pass the namespace elsewhere. They should be neutral when paired with exact
member reads. By contrast, key introspection such as `"foo" in mod` or
`Object.keys(mod)` remains broad because it observes the namespace surface.

## 6. Artifact Requirements

`symbols.json` must continue to expose:

- `meta.supports.cjsExportSurface === true`
- `meta.supports.cjsRequireOpacity === true`
- `cjsExportSurfaceByFile`
- `cjsRequireOpacity`
- resolved internal edges for exact, side-effect, and broad CJS uses

When new CJS per-file facts are added, symbol graph incremental cache identity
must be bumped or otherwise invalidated. Stale cache entries without the new
field must not be silently treated as "no CJS evidence."

## 7. Ranking Contract

CJS evidence affects ranking in two different ways:

1. Exact CJS consumer facts protect the referenced export from being classified
   as dead.
2. Broad CJS evidence blocks `SAFE_FIX` only when relevant to the candidate
   source file or unresolved consumer surface.

Side-effect-only CJS requires should contribute to file/module reachability but
must not keep every named export in the target file alive.

## 8. Tests

Add or extend focused tests before implementation:

- `_lib/extract-ts.mjs` direct extraction tests:
  - `module.exports.foo = require("./x").foo`
  - `exports.foo = require("./x").foo`
  - static computed member `mod["foo"]` and `require("./x")["foo"]`
  - truthiness guard `if (mod) mod.foo()` and `mod && mod.foo()`
  - `const localFoo = mod.foo`
  - shadowed parameter does not exact-protect
  - assignment/reassignment degrades
  - object spread / `Object.keys(mod)` degrades
- symbol graph tests:
  - exact CJS consumer increases only the referenced export's fan-in
  - side-effect-only require creates no named fan-in
  - broad CJS escape prevents `trulyDead` confidence for that target file
- ranking tests:
  - relevant CJS broad evidence blocks `SAFE_FIX`
  - unrelated CJS broad evidence does not globally block all `SAFE_FIX`
- incremental tests:
  - any new CJS payload field invalidates legacy symbol graph cache entries

## 9. Calibration

After unit and integration tests pass, run targeted quick/full audits on:

- a pure JS CommonJS-heavy repo fixture or checkout,
- `memento-mcp-main`,
- the geulbat2 CLI scratch repo previously reported as CJS-heavy.

The expected result is not "zero findings." The expected result is:

- exact CJS consumers reduce false dead exports,
- remaining unsupported CJS forms are visible as blind zones,
- `SAFE_FIX` does not appear when broad CJS could hide a consumer.

## 10. Recommended Implementation Slice

Start with test-only red cases. Then patch the existing extractor in place:

1. Add missing exact consumer cases that the current AST walk can already see.
2. Add missing broad-degrade cases for namespace escape patterns.
3. Wire any new fact fields through `symbol-graph-artifact`.
4. Bump symbol graph producer identity if persisted cache payload shape changes.
5. Run focused CJS tests, then local `npm test` only if the change touches shared
   extraction or symbol graph assembly.

This keeps the work YAGNI-compliant: it improves the exact/broad contract where
we have evidence, and refuses to infer more than the analyzer can prove.

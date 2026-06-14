// Tests for AST-based file-internal reference counting (v1.10.0 P0).
//
// Replaces `countOccurrencesExceptDefLine` / `countExcludingDeclAndExport`
// which used word-boundary regex over raw text. The regex approach
// inflated counts in four predictable ways:
//
//   1. Comments — `// foo is our export` → regex counts "foo"
//   2. String literals — `const msg = "foo bar"` → regex counts "foo"
//   3. Property keys — `{ foo: 1 }` or `obj.foo` → regex counts "foo"
//   4. Export specifier self-references — `export { foo }` → regex counts
//
// Each of these over-counts pushes a symbol from C (completely dead) to
// A (export-removable) or B (file-internal hub), altering the proposed
// action in `dead-classify.json`. The AST counter fixes all four.
//
// Scope-aware shadowing (landed 2026-04-20): the walker tracks a scope
// stack; inner bindings of `symbolName` (let/const/var/function/class/
// params, including destructuring) suppress references to the top-level.
// See T37–T46 for coverage. Evidence label stays `ast-ident-ref-count`
// since scope-awareness is a precision fix on top of AST identifier
// counting, not a distinct evidence category.
//
// FP-41 (filed 2026-04-20, fixed same day): the original v1.10.0 walker
// matched `node.type === 'Identifier'` only, which misses JSXIdentifier
// and JSXMemberExpression. Any same-file JSX usage of an exported symbol
// (`<Foo />`, `<Foo.Bar />`) was invisible, pushing intra-file compound
// components (shadcn/ui-class patterns: AlertDialog + AlertDialogTrigger,
// CodeBlock + CodeBlockContainer) from Tier A ("export unnecessary, file-
// internal use") into Tier C ("completely dead"). The JSX test block
// below (T24–T36) pins the fix — uses `.tsx` fixtures so the parser
// accepts JSX syntax.

import { countFileReferencesAst, countFileReferencesAstMany } from '../_lib/classify-facts.mjs';

let passed = 0, failed = 0;
function assert(label, expected, actual) {
  if (expected === actual) { passed++; console.log(`  PASS  ${label}`); }
  else {
    failed++;
    console.log(`  FAIL  ${label}`);
    console.log(`        expected: ${expected}`);
    console.log(`        actual:   ${actual}`);
  }
}

function c(src, name, line = 1) {
  // File path only used for language detection (extension dispatch in
  // `parseOxcOrThrow`). No I/O — src is passed directly.
  const r = countFileReferencesAst(src, '/fake/test.ts', name, line);
  return r;
}

function tsx(src, name, line = 1) {
  // Same as c(), but `.tsx` extension so the oxc-parser accepts JSX.
  const r = countFileReferencesAst(src, '/fake/test.tsx', name, line);
  return r;
}

// ── Baseline: simple value reference ──
assert('T1. single value reference counts', 1,
  c(`export const foo = 1;\nconst bar = foo;\n`, 'foo').count);

// ── FP class 1: comments ──
assert('T2. comment mentioning symbol does NOT count', 0,
  c(`export const foo = 1;\n// foo is our export\n`, 'foo').count);

assert('T3. block comment mentioning symbol does NOT count', 0,
  c(`export const foo = 1;\n/* see foo above for details */\n`, 'foo').count);

// ── FP class 2: string literals ──
assert('T4. symbol inside string literal does NOT count', 0,
  c(`export const foo = 1;\nconst msg = "foo bar";\n`, 'foo').count);

assert('T5. symbol inside template literal text does NOT count', 0,
  c(`export const foo = 1;\nconst msg = \`foo text\`;\n`, 'foo').count);

// ── FP class 3: property keys and member access ──
assert('T6. member access property does NOT count (obj.foo)', 0,
  c(`export const foo = 1;\nconst x = obj.foo;\n`, 'foo').count);

assert('T7. object literal key does NOT count ({ foo: 1 })', 0,
  c(`export const foo = 1;\nconst obj = { foo: 2 };\n`, 'foo').count);

assert('T8. object destructuring key does NOT count ({ foo: renamed } = ...)', 0,
  c(`export const foo = 1;\nconst { foo: r } = some;\n`, 'foo').count);

assert('T9. class method key does NOT count', 0,
  c(`export const foo = 1;\nclass X { foo() {} }\n`, 'foo').count);

// ── FP class 4: export-specifier self-reference ──
assert('T10. `export { foo }` does NOT count as internal use', 0,
  c(`const foo = 1;\nexport { foo };\n`, 'foo', 1).count);

assert('T11. aliased `export { foo as bar }` local slot does NOT count', 0,
  c(`const foo = 1;\nexport { foo as bar };\n`, 'foo', 1).count);

// ── Import specifiers are declarations, not uses ──
assert('T12. `import { foo } from "x"` does NOT count against our foo', 0,
  c(`import { foo } from 'lib';\n`, 'foo').count);

// ── Shorthand object literal IS a reference (value short-form) ──
assert('T13. shorthand `{ foo }` counts as use (it IS a local ref)', 1,
  c(`const foo = 1;\nconst obj = { foo };\n`, 'foo', 1).count);

// ── Real uses that should count ──
assert('T14. function call counts', 1,
  c(`export function foo() {}\nfoo();\n`, 'foo').count);

assert('T15. type reference counts', 1,
  c(`export type Foo = string;\nconst x: Foo = 'a';\n`, 'Foo').count);

assert('T16. array element counts', 2,
  c(`const foo = 1;\nconst arr = [foo, foo];\n`, 'foo', 1).count);

assert('T17. extends clause counts', 1,
  c(`class Foo {}\nclass Bar extends Foo {}\n`, 'Foo', 1).count);

assert('T18. new expression counts', 1,
  c(`class Foo {}\nconst x = new Foo();\n`, 'Foo', 1).count);

// ── Mixed: comment + string + real use = count only the real one ──
assert('T19. mixed comment+string+real use = 1', 1,
  c(
    `export const foo = 1;\n` +
    `// foo is ours\n` +
    `const msg = "foo";\n` +
    `const ref = foo;\n`,
    'foo'
  ).count);

// ── Type+value position split in provenance ──
{
  const r = c(
    `export type Foo = string;\n` +
    `const x: Foo = 'a';\n` +
    `export const Foo2 = (v: Foo) => v;\n`,
    'Foo'
  );
  assert('T20. type reference increments typeRefs', true, r.typeRefs >= 1);
  assert('T21. mixed refs carry typeRefs + valueRefs', true,
    r.count === (r.typeRefs + r.valueRefs));
}

// ── Parse errors: return structured marker, don't throw ──
{
  const r = c('const foo = ;;; bad syntax', 'foo');
  assert('T22. parse error returns count=null with evidence marker',
    true, r.count === null && r.evidence === 'parse-error');
}

// ── Evidence label is explicit ──
assert('T23. evidence label is ast-ident-ref-count for successful parse',
  'ast-ident-ref-count',
  c('const foo = 1; const x = foo;', 'foo', 1).evidence);

// ═════════════════════════════════════════════════════════════
// JSX block — FP-41 regression guards (2026-04-20)
// ═════════════════════════════════════════════════════════════
// Walker must treat JSXIdentifier as a counted identifier reference.
// Skip rules for JSXAttribute.name (attribute naming slot), for the
// `property` slot of JSXMemberExpression (sub-component name, not our
// symbol), and for JSXNamespacedName.namespace (prefix slot).

// ── T24. Single self-closing JSX element ──
{
  const r = tsx(`export const Foo = () => null;\n<Foo />;\n`, 'Foo', 1);
  assert('T24. single `<Foo />` counts 1', 1, r.count);
  assert('T24b. JSX usage is a value reference (not type)', 1, r.valueRefs);
  assert('T24c. JSX usage does NOT register as type ref', 0, r.typeRefs);
}

// ── T25. Nested JSX: outer tag counts ──
assert('T25. `<Foo><Bar /></Foo>` counts 1 for Foo',
  1,
  tsx(`export const Foo = (p) => p.children;\n<Foo><Bar /></Foo>;\n`, 'Foo', 1).count);

// ── T26. Self-closing with JS + string props ──
assert('T26. `<Foo a={1} b="x" />` counts 1 for Foo',
  1,
  tsx(`export const Foo = () => null;\n<Foo a={1} b="x" />;\n`, 'Foo', 1).count);

// ── T27. JSXMemberExpression: head counts for Foo ──
assert('T27. `<Foo.Bar />` counts 1 for Foo (head/object slot)',
  1,
  tsx(`export const Foo = { Bar: () => null };\n<Foo.Bar />;\n`, 'Foo', 1).count);

// ── T28. JSXMemberExpression: tail does NOT count for Bar ──
//
// Rationale: `<Foo.Bar />` is a property-style access. `Bar` names a
// sub-component on Foo, not a JS binding called `Bar` — parallels
// `MemberExpression.property` on non-computed access.
assert('T28. `<Foo.Bar />` does NOT count for Bar (property slot)',
  0,
  tsx(`export const Bar = 1;\nconst Foo = { Bar: () => null };\n<Foo.Bar />;\n`, 'Bar', 1).count);

// ── T29. Attribute name slot does NOT count ──
//
// `<div foo={...} />` — `foo` in attribute-name position is a prop name,
// not a reference to our symbol.
assert('T29. JSXAttribute name slot does NOT count',
  0,
  tsx(`export const foo = 1;\n<div foo={1} />;\n`, 'foo', 1).count);

// ── T30. Attribute value expression DOES count ──
assert('T30. JSXAttribute value expression DOES count',
  1,
  tsx(`export const foo = 1;\n<div a={foo} />;\n`, 'foo', 1).count);

// ── T31. JSX children text does NOT count ──
//
// `<div>foo bar</div>` — `foo` appears as JSXText, not as an identifier.
assert('T31. JSXText content does NOT count',
  0,
  tsx(`export const foo = 1;\n<div>foo bar</div>;\n`, 'foo', 1).count);

// ── T32. Fragment wrapper: inner counts ──
assert('T32. `<><Foo /></>` counts 1',
  1,
  tsx(`export const Foo = () => null;\n<><Foo /></>;\n`, 'Foo', 1).count);

// ── T33. JSX in conditional expression ──
assert('T33. `cond ? <Foo /> : null` counts 1',
  1,
  tsx(`export const Foo = () => null;\nconst cond = true;\nconst x = cond ? <Foo /> : null;\n`, 'Foo', 1).count);

// ── T34. Self-render (inside its own body) counts ──
//
// `export const Foo = () => ...\n<Foo />` — the JSX usage is on a
// later line than declLine, so the line-based skip does NOT fire and
// the structural skip rules correctly see this as a JSX reference.
assert('T34. self-render on a later line counts 1',
  1,
  tsx(`export const Foo = () =>\n  <Foo />\n;\n`, 'Foo', 1).count);

// ── T35. Spread attributes: symbol appears as the spread argument ──
//
// `<Foo {...props} />` — spread attribute value is an expression; the
// head identifier `Foo` counts as the JSX tag. A separate spread of a
// plain identifier counts that identifier too.
{
  const r = tsx(`export const Foo = () => null;\nconst props = {};\n<Foo {...props} />;\n`, 'Foo', 1);
  assert('T35. `<Foo {...props} />` counts 1 for Foo tag', 1, r.count);
  const rp = tsx(`export const props = {};\nconst Foo = () => null;\n<Foo {...props} />;\n`, 'props', 1);
  assert('T35b. `{...props}` spread expression counts 1 for props', 1, rp.count);
}

// ── T36. FP-41 reproduction — compound component pattern ──
//
// This is the exact AlertDialog / CodeBlock shape sampled by the
// external reviewer on duyet. AlertDialogTrigger has no external
// consumer but is used via JSX inside the live sibling's render.
// v1.9.11 regex counter: 1 (correct, Tier A).
// v1.10.0 pre-fix AST counter: 0 (Tier C — the FP-41 bug).
// Post-fix: 1 (Tier A — correct).
{
  const src =
    `export const AlertDialog = (props) => (\n` +
    `  <div className="root">\n` +
    `    <AlertDialogTrigger>{props.children}</AlertDialogTrigger>\n` +
    `  </div>\n` +
    `);\n` +
    `export const AlertDialogTrigger = (props) => (\n` +
    `  <button>{props.children}</button>\n` +
    `);\n`;
  // Declaration of AlertDialogTrigger is on line 6 of the source above.
  const r = tsx(src, 'AlertDialogTrigger', 6);
  assert('T36. compound component: AlertDialogTrigger JSX usage counts 1',
    1, r.count);
  assert('T36b. compound component: counted as value ref',
    1, r.valueRefs);
}

// ═══ T37–T46. Scope-aware shadowing (v1.10.0 gap — landed 2026-04-20) ═══
//
// The counter now walks with a scope stack. References to `symbolName`
// inside an inner scope that has its own binding of the same name do NOT
// count against the top-level export. Covers the common cases:
//   - inner const/let bindings (block-scoped)
//   - function parameters (function-scoped)
//   - arrow function parameters
//   - catch clause parameter
//   - for-loop binding (init of for/for-in/for-of)
//
// Known limit: `var` is function-scoped not block-scoped; v1 handles the
// declaration-position skip but does not hoist `var` across block boundaries.
// Rare for modern TS codebases; documented in the walker header.

// T37. Inner const shadows top-level — no refs count.
{
  const src =
    `export const foo = 1;\n` +
    `function bar() {\n` +
    `  const foo = 2;\n` +
    `  return foo;\n` +
    `}\n`;
  const r = c(src, 'foo', 1);
  assert('T37. inner const shadows top-level: count 0', 0, r.count);
}

// T38. Function parameter shadows top-level.
{
  const src =
    `export const foo = 1;\n` +
    `export function bar(foo) {\n` +
    `  return foo;\n` +
    `}\n`;
  assert('T38. function param shadows: count 0', 0, c(src, 'foo', 1).count);
}

// T39. Arrow function parameter shadows.
{
  const src =
    `export const foo = 1;\n` +
    `export const bar = (foo) => foo;\n`;
  assert('T39. arrow param shadows: count 0', 0, c(src, 'foo', 1).count);
}

// T40. Block-scoped const inside if shadows within that block.
{
  const src =
    `export const foo = 1;\n` +
    `if (Math.random() > 0) {\n` +
    `  const foo = 2;\n` +
    `  console.log(foo);\n` +
    `}\n`;
  assert('T40. block-scoped shadow: count 0', 0, c(src, 'foo', 1).count);
}

// T41. Different block — sibling scope does NOT shadow.
{
  const src =
    `export const foo = 1;\n` +
    `if (Math.random() > 0) {\n` +
    `  const foo = 2;\n` +
    `}\n` +
    `console.log(foo);\n`;  // outside the if — refers to top-level foo
  assert('T41. sibling block does NOT shadow: count 1', 1, c(src, 'foo', 1).count);
}

// T42. Catch clause parameter shadows inside the catch body.
{
  const src =
    `export const foo = 1;\n` +
    `try { f(); } catch (foo) {\n` +
    `  console.log(foo);\n` +
    `}\n`;
  assert('T42. catch param shadows: count 0', 0, c(src, 'foo', 1).count);
}

// T43. For-of loop variable shadows inside the loop body.
{
  const src =
    `export const foo = 1;\n` +
    `for (const foo of [1, 2, 3]) {\n` +
    `  console.log(foo);\n` +
    `}\n`;
  assert('T43. for-of loop binding shadows: count 0', 0, c(src, 'foo', 1).count);
}

// T44. Nested function does NOT shadow unless the inner actually rebinds.
{
  const src =
    `export const foo = 1;\n` +
    `function outer() {\n` +
    `  return foo;\n` +  // references top-level foo
    `}\n`;
  assert('T44. nested function without rebind: count 1', 1, c(src, 'foo', 1).count);
}

// T45. Same-name function expression in a different scope.
// `const bar = function foo() { return foo; }` — the inner `foo` is the
// function-expression-name binding (also called NFE name), bound in the
// function body. It shadows the top-level `foo`.
{
  const src =
    `export const foo = 1;\n` +
    `const bar = function foo() { return foo; };\n`;
  assert('T45. named function expression shadow: count 0', 0, c(src, 'foo', 1).count);
}

// T46. Destructured parameter shadows.
{
  const src =
    `export const foo = 1;\n` +
    `function bar({ foo }) { return foo; }\n`;
  assert('T46. destructured param shadows: count 0', 0, c(src, 'foo', 1).count);
}

// T47. Exported TS declaration references are surfaced separately.
{
  const src =
    `export interface PublicBox {\n` +
    `  value: HiddenType;\n` +
    `}\n` +
    `export interface HiddenType {\n` +
    `  id: string;\n` +
    `}\n` +
    `export function useHidden() {\n` +
    `  return HiddenType;\n` +
    `}\n`;
  const r = c(src, 'HiddenType', 4);
  assert('T47. exported declaration dependency is counted',
    1, r.exportedDeclarationRefs);
  assert('T47b. exported declaration dependency line is retained',
    2, r.exportedDeclarationRefLines[0]);
}

// T48. Runtime references inside exported function bodies are not
// declaration-surface refs. They remain normal value refs only.
{
  const src =
    `export const HiddenValue = 1;\n` +
    `export function useHidden() {\n` +
    `  return HiddenValue;\n` +
    `}\n`;
  const r = c(src, 'HiddenValue', 1);
  assert('T48. exported function body does NOT count as declaration dependency',
    0, r.exportedDeclarationRefs);
  assert('T48b. exported function body still counts as value ref',
    1, r.valueRefs);
}

// T49. Exported function signatures are declaration surface; bodies are not.
{
  const src =
    `export interface LoadOptions {\n` +
    `  strict?: boolean;\n` +
    `}\n` +
    `export function loadData(options?: LoadOptions): LoadOptions[] {\n` +
    `  const LoadOptions = 1;\n` +
    `  return [];\n` +
    `}\n`;
  const r = c(src, 'LoadOptions', 1);
  assert('T49. exported function signature refs count as declaration dependency',
    2, r.exportedDeclarationRefs);
  assert('T49b. function body shadow does not count as declaration dependency',
    2, r.count);
}

// T50. Exported class type surface is declaration surface. Method bodies
// remain implementation details and must not count as public declaration
// dependencies.
{
  const src =
    `export interface HiddenType {\n` +
    `  id: string;\n` +
    `}\n` +
    `export class PublicClass {\n` +
    `  field!: HiddenType;\n` +
    `  get(): HiddenType {\n` +
    `    const value: HiddenType = { id: 'x' };\n` +
    `    return value;\n` +
    `  }\n` +
    `}\n`;
  const r = c(src, 'HiddenType', 1);
  assert('T50. exported class field + method return type count as declaration dependencies',
    2, r.exportedDeclarationRefs);
  assert('T50b. exported class method body type annotation is not declaration dependency',
    3, r.count);
}

// T51. Exported variable type annotations are declaration surface. The
// initializer body is runtime implementation and should not add public
// declaration refs.
{
  const src =
    `export interface HiddenType {\n` +
    `  id: string;\n` +
    `}\n` +
    `export const makeHidden: () => HiddenType = () => {\n` +
    `  const value: HiddenType = { id: 'x' };\n` +
    `  return value;\n` +
    `};\n`;
  const r = c(src, 'HiddenType', 1);
  assert('T51. exported variable type annotation counts as declaration dependency',
    1, r.exportedDeclarationRefs);
  assert('T51b. exported variable initializer body type annotation is not declaration dependency',
    2, r.count);
}

// T52. Batch reference counting preserves single-symbol semantics for
// multiple dead candidates in the same file. This pins the performance
// contract used by classify-dead-exports: parse once per file, count many.
{
  const src =
    `export const Alpha = 1;\n` +
    `export type Beta = { value: Alpha };\n` +
    `const a = Alpha;\n` +
    `const b: Beta = { value: a };\n`;
  const many = countFileReferencesAstMany(src, '/fake/batch.ts', [
    { symbolName: 'Alpha', declLine: 1 },
    { symbolName: 'Beta', declLine: 2 },
  ]);
  const alpha = many.get('Alpha');
  const beta = many.get('Beta');
  assert('T52. batch Alpha count matches single-symbol counter',
    c(src, 'Alpha', 1).count, alpha?.count);
  assert('T52b. batch Beta count matches single-symbol counter',
    c(src, 'Beta', 2).count, beta?.count);
  assert('T52c. batch Alpha declaration dependency count is preserved',
    c(src, 'Alpha', 1).exportedDeclarationRefs, alpha?.exportedDeclarationRefs);
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

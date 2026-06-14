import { describe, expect, it } from "vitest";

import {
  countFileReferencesAst,
  countFileReferencesAstMany,
} from "../_lib/classify-facts.mjs";

function c(src, name, line = 1) {
  return countFileReferencesAst(src, "/fake/test.ts", name, line);
}

function tsx(src, name, line = 1) {
  return countFileReferencesAst(src, "/fake/test.tsx", name, line);
}

describe("AST reference counting skips non-reference text", () => {
  it("T1. single value reference counts", () => {
    expect(c(`export const foo = 1;\nconst bar = foo;\n`, "foo").count).toBe(1);
  });

  it("T2. comment mentioning symbol does NOT count", () => {
    expect(
      c(`export const foo = 1;\n// foo is our export\n`, "foo").count,
    ).toBe(0);
  });

  it("T3. block comment mentioning symbol does NOT count", () => {
    expect(
      c(`export const foo = 1;\n/* see foo above for details */\n`, "foo")
        .count,
    ).toBe(0);
  });

  it("T4. symbol inside string literal does NOT count", () => {
    expect(
      c(`export const foo = 1;\nconst msg = "foo bar";\n`, "foo").count,
    ).toBe(0);
  });

  it("T5. symbol inside template literal text does NOT count", () => {
    expect(
      c("export const foo = 1;\nconst msg = `foo text`;\n", "foo").count,
    ).toBe(0);
  });

  it("T6. member access property does NOT count (obj.foo)", () => {
    expect(c(`export const foo = 1;\nconst x = obj.foo;\n`, "foo").count).toBe(
      0,
    );
  });

  it("T7. object literal key does NOT count ({ foo: 1 })", () => {
    expect(
      c(`export const foo = 1;\nconst obj = { foo: 2 };\n`, "foo").count,
    ).toBe(0);
  });

  it("T8. object destructuring key does NOT count ({ foo: renamed } = ...)", () => {
    expect(
      c(`export const foo = 1;\nconst { foo: r } = some;\n`, "foo").count,
    ).toBe(0);
  });

  it("T9. class method key does NOT count", () => {
    expect(
      c(`export const foo = 1;\nclass X { foo() {} }\n`, "foo").count,
    ).toBe(0);
  });

  it("T10. `export { foo }` does NOT count as internal use", () => {
    expect(c(`const foo = 1;\nexport { foo };\n`, "foo", 1).count).toBe(0);
  });

  it("T11. aliased `export { foo as bar }` local slot does NOT count", () => {
    expect(c(`const foo = 1;\nexport { foo as bar };\n`, "foo", 1).count).toBe(
      0,
    );
  });

  it('T12. `import { foo } from "x"` does NOT count against our foo', () => {
    expect(c(`import { foo } from 'lib';\n`, "foo").count).toBe(0);
  });

  it("T13. shorthand `{ foo }` counts as use (it IS a local ref)", () => {
    expect(c(`const foo = 1;\nconst obj = { foo };\n`, "foo", 1).count).toBe(1);
  });
});

describe("AST reference counting records real references and evidence", () => {
  it("T14. function call counts", () => {
    expect(c(`export function foo() {}\nfoo();\n`, "foo").count).toBe(1);
  });

  it("T15. type reference counts", () => {
    expect(
      c(`export type Foo = string;\nconst x: Foo = 'a';\n`, "Foo").count,
    ).toBe(1);
  });

  it("T16. array element counts", () => {
    expect(c(`const foo = 1;\nconst arr = [foo, foo];\n`, "foo", 1).count).toBe(
      2,
    );
  });

  it("T17. extends clause counts", () => {
    expect(c(`class Foo {}\nclass Bar extends Foo {}\n`, "Foo", 1).count).toBe(
      1,
    );
  });

  it("T18. new expression counts", () => {
    expect(c(`class Foo {}\nconst x = new Foo();\n`, "Foo", 1).count).toBe(1);
  });

  it("T19. mixed comment+string+real use = 1", () => {
    const src =
      `export const foo = 1;\n` +
      `// foo is ours\n` +
      `const msg = "foo";\n` +
      `const ref = foo;\n`;

    expect(c(src, "foo").count).toBe(1);
  });

  it("T20. type reference increments typeRefs", () => {
    const r = c(
      `export type Foo = string;\n` +
        `const x: Foo = 'a';\n` +
        `export const Foo2 = (v: Foo) => v;\n`,
      "Foo",
    );

    expect(r.typeRefs).toBeGreaterThanOrEqual(1);
  });

  it("T21. mixed refs carry typeRefs + valueRefs", () => {
    const r = c(
      `export type Foo = string;\n` +
        `const x: Foo = 'a';\n` +
        `export const Foo2 = (v: Foo) => v;\n`,
      "Foo",
    );

    expect(r.count).toBe(r.typeRefs + r.valueRefs);
  });

  it("T22. parse error returns count=null with evidence marker", () => {
    const r = c("const foo = ;;; bad syntax", "foo");

    expect(r).toMatchObject({ count: null, evidence: "parse-error" });
  });

  it("T23. evidence label is ast-ident-ref-count for successful parse", () => {
    expect(c("const foo = 1; const x = foo;", "foo", 1).evidence).toBe(
      "ast-ident-ref-count",
    );
  });
});

describe("JSX reference counting", () => {
  it("T24. single `<Foo />` counts 1", () => {
    expect(
      tsx(`export const Foo = () => null;\n<Foo />;\n`, "Foo", 1).count,
    ).toBe(1);
  });

  it("T24b. JSX usage is a value reference (not type)", () => {
    expect(
      tsx(`export const Foo = () => null;\n<Foo />;\n`, "Foo", 1).valueRefs,
    ).toBe(1);
  });

  it("T24c. JSX usage does NOT register as type ref", () => {
    expect(
      tsx(`export const Foo = () => null;\n<Foo />;\n`, "Foo", 1).typeRefs,
    ).toBe(0);
  });

  it("T25. `<Foo><Bar /></Foo>` counts 1 for Foo", () => {
    expect(
      tsx(
        `export const Foo = (p) => p.children;\n<Foo><Bar /></Foo>;\n`,
        "Foo",
        1,
      ).count,
    ).toBe(1);
  });

  it('T26. `<Foo a={1} b="x" />` counts 1 for Foo', () => {
    expect(
      tsx(`export const Foo = () => null;\n<Foo a={1} b="x" />;\n`, "Foo", 1)
        .count,
    ).toBe(1);
  });

  it("T27. `<Foo.Bar />` counts 1 for Foo (head/object slot)", () => {
    expect(
      tsx(`export const Foo = { Bar: () => null };\n<Foo.Bar />;\n`, "Foo", 1)
        .count,
    ).toBe(1);
  });

  it("T28. `<Foo.Bar />` does NOT count for Bar (property slot)", () => {
    expect(
      tsx(
        `export const Bar = 1;\nconst Foo = { Bar: () => null };\n<Foo.Bar />;\n`,
        "Bar",
        1,
      ).count,
    ).toBe(0);
  });

  it("T29. JSXAttribute name slot does NOT count", () => {
    expect(
      tsx(`export const foo = 1;\n<div foo={1} />;\n`, "foo", 1).count,
    ).toBe(0);
  });

  it("T30. JSXAttribute value expression DOES count", () => {
    expect(
      tsx(`export const foo = 1;\n<div a={foo} />;\n`, "foo", 1).count,
    ).toBe(1);
  });

  it("T31. JSXText content does NOT count", () => {
    expect(
      tsx(`export const foo = 1;\n<div>foo bar</div>;\n`, "foo", 1).count,
    ).toBe(0);
  });

  it("T32. `<><Foo /></>` counts 1", () => {
    expect(
      tsx(`export const Foo = () => null;\n<><Foo /></>;\n`, "Foo", 1).count,
    ).toBe(1);
  });

  it("T33. `cond ? <Foo /> : null` counts 1", () => {
    expect(
      tsx(
        `export const Foo = () => null;\nconst cond = true;\nconst x = cond ? <Foo /> : null;\n`,
        "Foo",
        1,
      ).count,
    ).toBe(1);
  });

  it("T34. self-render on a later line counts 1", () => {
    expect(
      tsx(`export const Foo = () =>\n  <Foo />\n;\n`, "Foo", 1).count,
    ).toBe(1);
  });

  it("T35. `<Foo {...props} />` counts 1 for Foo tag", () => {
    const r = tsx(
      `export const Foo = () => null;\nconst props = {};\n<Foo {...props} />;\n`,
      "Foo",
      1,
    );

    expect(r.count).toBe(1);
  });

  it("T35b. `{...props}` spread expression counts 1 for props", () => {
    const r = tsx(
      `export const props = {};\nconst Foo = () => null;\n<Foo {...props} />;\n`,
      "props",
      1,
    );

    expect(r.count).toBe(1);
  });

  it("T36. compound component: AlertDialogTrigger JSX usage counts 1", () => {
    const src =
      `export const AlertDialog = (props) => (\n` +
      `  <div className="root">\n` +
      `    <AlertDialogTrigger>{props.children}</AlertDialogTrigger>\n` +
      `  </div>\n` +
      `);\n` +
      `export const AlertDialogTrigger = (props) => (\n` +
      `  <button>{props.children}</button>\n` +
      `);\n`;

    expect(tsx(src, "AlertDialogTrigger", 6).count).toBe(1);
  });

  it("T36b. compound component: counted as value ref", () => {
    const src =
      `export const AlertDialog = (props) => (\n` +
      `  <div className="root">\n` +
      `    <AlertDialogTrigger>{props.children}</AlertDialogTrigger>\n` +
      `  </div>\n` +
      `);\n` +
      `export const AlertDialogTrigger = (props) => (\n` +
      `  <button>{props.children}</button>\n` +
      `);\n`;

    expect(tsx(src, "AlertDialogTrigger", 6).valueRefs).toBe(1);
  });
});

describe("scope-aware shadowing", () => {
  it("T37. inner const shadows top-level: count 0", () => {
    const src =
      `export const foo = 1;\n` +
      `function bar() {\n` +
      `  const foo = 2;\n` +
      `  return foo;\n` +
      `}\n`;

    expect(c(src, "foo", 1).count).toBe(0);
  });

  it("T38. function param shadows: count 0", () => {
    const src =
      `export const foo = 1;\n` +
      `export function bar(foo) {\n` +
      `  return foo;\n` +
      `}\n`;

    expect(c(src, "foo", 1).count).toBe(0);
  });

  it("T39. arrow param shadows: count 0", () => {
    const src = `export const foo = 1;\nexport const bar = (foo) => foo;\n`;

    expect(c(src, "foo", 1).count).toBe(0);
  });

  it("T40. block-scoped shadow: count 0", () => {
    const src =
      `export const foo = 1;\n` +
      `if (Math.random() > 0) {\n` +
      `  const foo = 2;\n` +
      `  console.log(foo);\n` +
      `}\n`;

    expect(c(src, "foo", 1).count).toBe(0);
  });

  it("T41. sibling block does NOT shadow: count 1", () => {
    const src =
      `export const foo = 1;\n` +
      `if (Math.random() > 0) {\n` +
      `  const foo = 2;\n` +
      `}\n` +
      `console.log(foo);\n`;

    expect(c(src, "foo", 1).count).toBe(1);
  });

  it("T42. catch param shadows: count 0", () => {
    const src =
      `export const foo = 1;\n` +
      `try { f(); } catch (foo) {\n` +
      `  console.log(foo);\n` +
      `}\n`;

    expect(c(src, "foo", 1).count).toBe(0);
  });

  it("T43. for-of loop binding shadows: count 0", () => {
    const src =
      `export const foo = 1;\n` +
      `for (const foo of [1, 2, 3]) {\n` +
      `  console.log(foo);\n` +
      `}\n`;

    expect(c(src, "foo", 1).count).toBe(0);
  });

  it("T44. nested function without rebind: count 1", () => {
    const src =
      `export const foo = 1;\n` +
      `function outer() {\n` +
      `  return foo;\n` +
      `}\n`;

    expect(c(src, "foo", 1).count).toBe(1);
  });

  it("T45. named function expression shadow: count 0", () => {
    const src =
      `export const foo = 1;\n` +
      `const bar = function foo() { return foo; };\n`;

    expect(c(src, "foo", 1).count).toBe(0);
  });

  it("T46. destructured param shadows: count 0", () => {
    const src =
      `export const foo = 1;\n` + `function bar({ foo }) { return foo; }\n`;

    expect(c(src, "foo", 1).count).toBe(0);
  });
});

describe("exported declaration surface references", () => {
  it("T47. exported declaration dependency is counted", () => {
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

    expect(c(src, "HiddenType", 4).exportedDeclarationRefs).toBe(1);
  });

  it("T47b. exported declaration dependency line is retained", () => {
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

    expect(c(src, "HiddenType", 4).exportedDeclarationRefLines[0]).toBe(2);
  });

  it("T48. exported function body does NOT count as declaration dependency", () => {
    const src =
      `export const HiddenValue = 1;\n` +
      `export function useHidden() {\n` +
      `  return HiddenValue;\n` +
      `}\n`;

    expect(c(src, "HiddenValue", 1).exportedDeclarationRefs).toBe(0);
  });

  it("T48b. exported function body still counts as value ref", () => {
    const src =
      `export const HiddenValue = 1;\n` +
      `export function useHidden() {\n` +
      `  return HiddenValue;\n` +
      `}\n`;

    expect(c(src, "HiddenValue", 1).valueRefs).toBe(1);
  });

  it("T49. exported function signature refs count as declaration dependency", () => {
    const src =
      `export interface LoadOptions {\n` +
      `  strict?: boolean;\n` +
      `}\n` +
      `export function loadData(options?: LoadOptions): LoadOptions[] {\n` +
      `  const LoadOptions = 1;\n` +
      `  return [];\n` +
      `}\n`;

    expect(c(src, "LoadOptions", 1).exportedDeclarationRefs).toBe(2);
  });

  it("T49b. function body shadow does not count as declaration dependency", () => {
    const src =
      `export interface LoadOptions {\n` +
      `  strict?: boolean;\n` +
      `}\n` +
      `export function loadData(options?: LoadOptions): LoadOptions[] {\n` +
      `  const LoadOptions = 1;\n` +
      `  return [];\n` +
      `}\n`;

    expect(c(src, "LoadOptions", 1).count).toBe(2);
  });

  it("T50. exported class field + method return type count as declaration dependencies", () => {
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

    expect(c(src, "HiddenType", 1).exportedDeclarationRefs).toBe(2);
  });

  it("T50b. exported class method body type annotation is not declaration dependency", () => {
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

    expect(c(src, "HiddenType", 1).count).toBe(3);
  });

  it("T51. exported variable type annotation counts as declaration dependency", () => {
    const src =
      `export interface HiddenType {\n` +
      `  id: string;\n` +
      `}\n` +
      `export const makeHidden: () => HiddenType = () => {\n` +
      `  const value: HiddenType = { id: 'x' };\n` +
      `  return value;\n` +
      `};\n`;

    expect(c(src, "HiddenType", 1).exportedDeclarationRefs).toBe(1);
  });

  it("T51b. exported variable initializer body type annotation is not declaration dependency", () => {
    const src =
      `export interface HiddenType {\n` +
      `  id: string;\n` +
      `}\n` +
      `export const makeHidden: () => HiddenType = () => {\n` +
      `  const value: HiddenType = { id: 'x' };\n` +
      `  return value;\n` +
      `};\n`;

    expect(c(src, "HiddenType", 1).count).toBe(2);
  });
});

describe("batch reference counting", () => {
  it("T52. batch Alpha count matches single-symbol counter", () => {
    const src =
      `export const Alpha = 1;\n` +
      `export type Beta = { value: Alpha };\n` +
      `const a = Alpha;\n` +
      `const b: Beta = { value: a };\n`;
    const many = countFileReferencesAstMany(src, "/fake/batch.ts", [
      { symbolName: "Alpha", declLine: 1 },
      { symbolName: "Beta", declLine: 2 },
    ]);

    expect(many.get("Alpha")?.count).toBe(c(src, "Alpha", 1).count);
  });

  it("T52b. batch Beta count matches single-symbol counter", () => {
    const src =
      `export const Alpha = 1;\n` +
      `export type Beta = { value: Alpha };\n` +
      `const a = Alpha;\n` +
      `const b: Beta = { value: a };\n`;
    const many = countFileReferencesAstMany(src, "/fake/batch.ts", [
      { symbolName: "Alpha", declLine: 1 },
      { symbolName: "Beta", declLine: 2 },
    ]);

    expect(many.get("Beta")?.count).toBe(c(src, "Beta", 2).count);
  });

  it("T52c. batch Alpha declaration dependency count is preserved", () => {
    const src =
      `export const Alpha = 1;\n` +
      `export type Beta = { value: Alpha };\n` +
      `const a = Alpha;\n` +
      `const b: Beta = { value: a };\n`;
    const many = countFileReferencesAstMany(src, "/fake/batch.ts", [
      { symbolName: "Alpha", declLine: 1 },
      { symbolName: "Beta", declLine: 2 },
    ]);

    expect(many.get("Alpha")?.exportedDeclarationRefs).toBe(
      c(src, "Alpha", 1).exportedDeclarationRefs,
    );
  });
});

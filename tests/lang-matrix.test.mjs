import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { canContainJsx, langForFile, nonJsLangForFile } from "../_lib/lang.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

function buildLangMatrixFixture() {
  const fixture = createTempRepoFixture({ prefix: "fx-vitest-lang-matrix-" });

  fixture.write(
    "src/Button.jsx",
    "export function Button({ label }) { return <button>{label}</button>; }\n",
  );
  fixture.write(
    "src/App.jsx",
    "import { Button } from './Button.jsx';\n" +
      'export function App() { return <div><Button label="hi" /></div>; }\n',
  );
  fixture.write(
    "src/JsxInJs.js",
    "export function JsxInJs() { return <section>JSX in JS</section>; }\n",
  );
  fixture.write(
    "src/JsxInJsConsumer.js",
    "import { JsxInJs } from './JsxInJs.js';\n" +
      "export const rendered = JsxInJs();\n",
  );
  fixture.write(
    "src/legacyHelper.cjs",
    "export const cjsHelper = (x) => x + 1;\n" +
      "export const cjsUnused = () => 0;\n",
  );
  fixture.write(
    "src/legacyConsumer.mjs",
    "import { cjsHelper } from './legacyHelper.cjs';\n" +
      "export const result = cjsHelper(1);\n",
  );
  fixture.write("src/typed.mts", "export const mtsValue: number = 1;\n");
  fixture.write("src/legacy.cts", "export const ctsValue = 2;\n");
  fixture.write(
    "src/classic.ts",
    "import { cjsHelper } from './legacyHelper.cjs';\n" +
      "import { mtsValue } from './typed.mjs';\n" +
      "export const classic = cjsHelper(mtsValue);\n",
  );
  fixture.write(
    "src/declarations.d.ts",
    "export const runtimeDependencies: string[];\n" +
      "export interface PublicDeclaration { enabled: boolean }\n",
  );

  return fixture;
}

describe("language helper dispatch", () => {
  it.each([
    ["L1. langForFile(.tsx) = tsx", "a.tsx", "tsx"],
    ["L2. langForFile(.jsx) = jsx", "a.jsx", "jsx"],
    ["L3. langForFile(.ts) = ts", "a.ts", "ts"],
    ["L4. langForFile(.js) = js", "a.js", "js"],
    ["L5. langForFile(.mjs) = js", "a.mjs", "js"],
    ["L6. langForFile(.cjs) = js", "a.cjs", "js"],
    ["L7. langForFile(.mts) = ts", "a.mts", "ts"],
    ["L8. langForFile(.cts) = ts", "a.cts", "ts"],
    ["L9. langForFile(.d.ts) = dts", "a.d.ts", "dts"],
    ["L10. langForFile(.d.mts) = dts", "a.d.mts", "dts"],
    ["L11. langForFile(.d.cts) = dts", "a.d.cts", "dts"],
    ["L12. langForFile(.py) = null", "a.py", null],
    ["L13. langForFile(.go) = null", "a.go", null],
  ])("%s", (_label, file, expected) => {
    expect(langForFile(file)).toBe(expected);
  });

  it.each([
    ["L14. canContainJsx(.tsx) = true", "a.tsx", true],
    ["L15. canContainJsx(.jsx) = true", "a.jsx", true],
    ["L16. canContainJsx(.ts) = false", "a.ts", false],
    ["L17. canContainJsx(.js) = false", "a.js", false],
  ])("%s", (_label, file, expected) => {
    expect(canContainJsx(file)).toBe(expected);
  });

  it.each([
    ["L18. nonJsLangForFile(.py) = python", "a.py", "python"],
    ["L19. nonJsLangForFile(.go) = go", "a.go", "go"],
    ["L20. nonJsLangForFile(.ts) = null", "a.ts", null],
  ])("%s", (_label, file, expected) => {
    expect(nonJsLangForFile(file)).toBe(expected);
  });
});

describe("mixed-extension symbol ingest", () => {
  let cachedSymbols;

  function buildSymbols() {
    const fixture = buildLangMatrixFixture();
    try {
      execFileSync(
        process.execPath,
        [
          "build-symbol-graph.mjs",
          "--root",
          fixture.root,
          "--output",
          fixture.output,
        ],
        {
          cwd: ROOT,
          stdio: ["ignore", "pipe", "pipe"],
        },
      );
      return JSON.parse(
        readFileSync(fixture.outputPath("symbols.json"), "utf8"),
      );
    } finally {
      fixture.cleanup();
    }
  }

  function symbols() {
    cachedSymbols ??= buildSymbols();
    return cachedSymbols;
  }

  it("I1. build-symbol-graph exits 0 on mixed-extension fixture", () => {
    expect(() => symbols()).not.toThrow();
  });

  it("I2. all 10 files walked (was 0 for pure-JSX repo pre-1.8.0)", () => {
    expect(symbols().files).toBe(10);
  });

  it("I2b. JSX syntax inside .js files parses without blind-zone entries", () => {
    expect(symbols().filesWithParseErrors ?? []).toHaveLength(0);
  });

  it("I3. totalDefs > 0 (JSX files parse cleanly, was 0 pre-1.8.0)", () => {
    expect(symbols().totalDefs).toBeGreaterThanOrEqual(9);
  });

  it("I4. cjsHelper NOT dead (used across .cjs -> .mjs + .ts)", () => {
    const deadSymbols = new Set(symbols().deadProdList.map((x) => x.symbol));

    expect(deadSymbols.has("cjsHelper")).toBe(false);
  });

  it("I5. cjsUnused IS dead (no cross-file consumer)", () => {
    const deadSymbols = new Set(symbols().deadProdList.map((x) => x.symbol));

    expect(deadSymbols.has("cjsUnused")).toBe(true);
  });

  it("I6. Button NOT dead (used by App.jsx via import)", () => {
    const deadSymbols = new Set(symbols().deadProdList.map((x) => x.symbol));

    expect(deadSymbols.has("Button")).toBe(false);
  });

  it("I7. mtsValue NOT dead (used by classic.ts via .mjs spec)", () => {
    const deadSymbols = new Set(symbols().deadProdList.map((x) => x.symbol));

    expect(deadSymbols.has("mtsValue")).toBe(false);
  });

  it("I8. JsxInJs NOT dead (used across .js files containing JSX)", () => {
    const deadSymbols = new Set(symbols().deadProdList.map((x) => x.symbol));

    expect(deadSymbols.has("JsxInJs")).toBe(false);
  });

  it("I9. .d.ts declaration-only value export parsed as a definition", () => {
    expect(
      symbols().defIndex?.["src/declarations.d.ts"]?.runtimeDependencies,
    ).toBeTruthy();
  });
});

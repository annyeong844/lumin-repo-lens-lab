import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

let fixture;

function runScript(scriptName, extraArgs = []) {
  return execFileSync(
    process.execPath,
    [
      scriptName,
      "--root",
      fixture.root,
      "--output",
      fixture.output,
      ...extraArgs,
    ],
    {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function loadLevel2Methods() {
  return JSON.parse(
    readFileSync(fixture.outputPath("level2-methods.json"), "utf8"),
  );
}

beforeAll(() => {
  fixture = createTempRepoFixture({
    prefix: "fx-vitest-hardcoding-",
    packageJson: {
      name: "monorepo-root",
      type: "module",
      workspaces: ["packages/*", "apps/*"],
    },
  });
  fixture.writeJson("packages/alpha/package.json", {
    name: "@mono/alpha",
    type: "module",
    exports: {
      ".": "./src/index.ts",
    },
  });
  fixture.write(
    "packages/alpha/src/helpers.ts",
    'export const alphaHelper = 1;\nexport const alphaDeadSymbol = "never-used";\n',
  );
  fixture.write(
    "packages/alpha/src/index.ts",
    "import { alphaHelper } from './helpers';\nexport const used = alphaHelper;\n",
  );
  fixture.writeJson("apps/beta/package.json", {
    name: "@mono/beta",
    type: "module",
    exports: {
      ".": "./src/index.ts",
    },
  });
  fixture.write(
    "apps/beta/src/utils.ts",
    'export const betaUtil = 1;\nexport const betaUnused1 = "x";\nexport const betaUnused2 = "y";\n',
  );
  fixture.write(
    "apps/beta/src/index.ts",
    "import { used } from '@mono/alpha';\n" +
      "import { betaUtil } from './utils';\n" +
      "export const beta = used + betaUtil;\n",
  );

  runScript("build-symbol-graph.mjs");
});

afterAll(() => {
  fixture.cleanup();
});

describe("workspace-derived classify labels", () => {
  let output;

  function classifyOutput() {
    output ??= runScript("classify-dead-exports.mjs");
    return output;
  }

  function packageCategoryBlock() {
    return (
      classifyOutput()
        .split("package별 × category")[1]
        ?.split("C (완전 dead)")[0] ?? ""
    );
  }

  it('T1. classify labels include workspace "alpha"', () => {
    expect(packageCategoryBlock()).toMatch(/packages\/alpha|alpha/);
  });

  it('T2. classify labels include workspace "beta"', () => {
    expect(packageCategoryBlock()).toMatch(/apps\/beta|beta/);
  });

  it.each(["protocol", "daemon", "web-shell", "shared-utils"])(
    'T3.%s. classify does NOT fabricate "%s" label',
    (badLabel) => {
      expect(packageCategoryBlock()).not.toMatch(
        new RegExp(`\\b${badLabel}\\b`),
      );
    },
  );
});

describe("method-call focus-class output", () => {
  it("T4. resolve-method-calls has no RunChannelClient block without --focus-class", () => {
    const output = runScript("resolve-method-calls.mjs");

    expect(output).not.toContain("RunChannelClient");
  });

  it("T5. --focus-class MyClass prints a MyClass-specific block", () => {
    const output = runScript("resolve-method-calls.mjs", [
      "--focus-class",
      "MyClass",
    ]);

    expect(output).toMatch(/MyClass method 사용 실태|MyClass\s+method/);
  });

  it("T6. --focus-class MyClass does NOT print RunChannelClient block", () => {
    const output = runScript("resolve-method-calls.mjs", [
      "--focus-class",
      "MyClass",
    ]);

    expect(output).not.toContain("RunChannelClient");
  });

  it("T7. level2-methods.json.focusClassReport carries className when flag set", () => {
    runScript("resolve-method-calls.mjs", ["--focus-class", "MyClass"]);

    expect(loadLevel2Methods().focusClassReport).toMatchObject({
      className: "MyClass",
    });
  });

  it("T8. focusClassReport is null when --focus-class omitted", () => {
    runScript("resolve-method-calls.mjs", ["--focus-class", "MyClass"]);
    runScript("resolve-method-calls.mjs");

    expect(loadLevel2Methods().focusClassReport).toBeNull();
  });
});

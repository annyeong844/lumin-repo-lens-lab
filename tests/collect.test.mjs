import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { collectFiles } from "../_lib/collect-files.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

let fixture;
let vendorParent;
let vendorRoot;

function writeFile(root, relPath, text) {
  const target = path.join(root, ...relPath.split("/"));
  mkdirSync(path.dirname(target), { recursive: true });
  writeFileSync(target, text);
  return target;
}

function slashRelative(root, files) {
  return files
    .map((file) => path.relative(root, file).split(path.sep).join("/"))
    .sort();
}

function collectRel(options) {
  return slashRelative(fixture.root, collectFiles(fixture.root, options));
}

function collectVendorRel(options) {
  return slashRelative(vendorRoot, collectFiles(vendorRoot, options));
}

beforeAll(() => {
  fixture = createTempRepoFixture({ prefix: "fx-vitest-collect-" });
  fixture.write("main.py", 'def hello(): return "py"\n');
  fixture.write("main.go", "package main\nfunc main() {}\n");
  fixture.write("some_test.go", "package main\nfunc TestFoo() {}\n");
  fixture.write("build-tool.mjs", "export const ver = 1;\n");
  fixture.write("root-entry.ts", "export const entry = true;\n");
  fixture.write(
    "src/a.ts",
    'export const x = 1;\nexport async function lazy() { return import("./b"); }\n',
  );
  fixture.write(
    "src/b.ts",
    "export const y = 2;\nconst internal = 3;\nexport { internal as publicName };\n",
  );
  fixture.write("src/build-index.ts", "export const buildIndex = true;\n");
  fixture.write(
    "src/socket-test-support.ts",
    "export const socketTestSupport = true;\n",
  );
  fixture.write("src/skip-me.js", "export const skipMe = true;\n");
  fixture.write("src/nested/exact-file.js", "export const exactFile = true;\n");
  fixture.write(
    "tests/a.test.ts",
    "import { x } from '../src/a';\nconsole.log(x);\n",
  );
  fixture.write(
    "runtime-tests/workerd/index.ts",
    'export default { fetch() { return new Response("ok"); } };\n',
  );
  fixture.write("test-utils/helper.ts", "export const testHelper = true;\n");
  fixture.write("tests/thing_test.py", "def test_x(): pass\n");
  fixture.write("pkg/worker.go", "package pkg\nfunc Worker() {}\n");
  fixture.write("pkg/worker_test.go", "package pkg\nfunc TestWorker() {}\n");
  fixture.write("output/generated.ts", "export const generated = true;\n");
  fixture.write(
    "output/nested/generated2.ts",
    "export const generated2 = true;\n",
  );

  vendorParent = path.join(tmpdir(), "vendor");
  mkdirSync(vendorParent, { recursive: true });
  vendorRoot = mkdtempSync(path.join(vendorParent, "fixture-collect-root-"));
  writeFile(
    vendorRoot,
    "package.json",
    '{"name":"vendor-root","type":"module"}',
  );
  writeFile(vendorRoot, "src/keep.ts", "export const keep = true;\n");
  writeFile(vendorRoot, "src/vendor/skip.ts", "export const skip = true;\n");
});

afterAll(() => {
  fixture.cleanup();
  if (vendorRoot) rmSync(vendorRoot, { recursive: true, force: true });
});

describe("collectFiles language filters", () => {
  it("keeps Python scans to Python files without leaking root JS/TS entries", () => {
    const files = collectRel({ languages: ["py"], includeTests: true });

    expect(files.some((file) => file.endsWith(".mjs"))).toBe(false);
    expect(files.some((file) => file.endsWith(".ts"))).toBe(false);
    expect(files.every((file) => file.endsWith(".py"))).toBe(true);
  });

  it("keeps Go scans to Go files without leaking root JavaScript entries", () => {
    const files = collectRel({ languages: ["go"], includeTests: true });

    expect(files.some((file) => file.endsWith(".mjs"))).toBe(false);
    expect(files.every((file) => file.endsWith(".go"))).toBe(true);
  });

  it("discovers root-level Python and Go files when allowed", () => {
    expect(collectRel({ languages: ["py"], includeTests: true })).toContain(
      "main.py",
    );

    const goFiles = collectRel({ languages: ["go"], includeTests: true });

    expect(goFiles).toContain("main.go");
    expect(goFiles).toContain("some_test.go");
  });
});

describe("collectFiles includeTests filtering", () => {
  it("drops pytest and Go test conventions when includeTests is false", () => {
    const pyProd = collectRel({ languages: ["py"], includeTests: false });
    const goProd = collectRel({ languages: ["go"], includeTests: false });

    expect(pyProd.some((file) => /(^|\/)[^/]*_test\.py$/.test(file))).toBe(
      false,
    );
    expect(goProd.some((file) => /(^|\/)[^/]*_test\.go$/.test(file))).toBe(
      false,
    );
  });

  it("preserves JS/TS root entries and test files when tests are included", () => {
    const files = collectRel({
      languages: ["ts", "tsx", "js", "mjs", "cjs", "jsx"],
      includeTests: true,
    });

    expect(files).toContain("root-entry.ts");
    expect(files).toContain("build-tool.mjs");
    expect(files.some((file) => file.endsWith(".test.ts"))).toBe(true);
  });

  it("drops JS/TS test conventions when includeTests is false", () => {
    const files = collectRel({
      languages: ["ts", "tsx", "js", "mjs", "cjs", "jsx"],
      includeTests: false,
    });

    expect(files.some((file) => /\.test\.tsx?$/.test(file))).toBe(false);
    expect(files).not.toContain("src/socket-test-support.ts");
    expect(files.some((file) => file.startsWith("runtime-tests/"))).toBe(false);
    expect(files.some((file) => file.startsWith("test-utils/"))).toBe(false);
  });

  it("does not leak Python or Go files into JS/TS scans", () => {
    const files = collectRel({
      languages: ["ts", "tsx", "js", "mjs", "cjs", "jsx"],
      includeTests: true,
    });

    expect(files.some((file) => file.endsWith(".py"))).toBe(false);
    expect(files.some((file) => file.endsWith(".go"))).toBe(false);
  });
});

describe("collectFiles user exclude rules", () => {
  it("prunes root-level output directories for absolute and relative roots", () => {
    const absoluteRoot = collectRel({
      languages: ["ts", "tsx", "js", "mjs", "cjs", "jsx"],
      includeTests: true,
      exclude: ["output"],
    });
    const relRoot = path.relative(process.cwd(), fixture.root) || ".";
    const relativeRoot = slashRelative(
      fixture.root,
      collectFiles(relRoot, {
        languages: ["ts", "tsx", "js", "mjs", "cjs", "jsx"],
        includeTests: true,
        exclude: ["output"],
      }),
    );

    expect(absoluteRoot.some((file) => file.startsWith("output/"))).toBe(false);
    expect(relativeRoot.some((file) => file.startsWith("output/"))).toBe(false);
  });

  it("treats directory excludes as path segments rather than filename substrings", () => {
    const files = collectRel({
      languages: ["ts", "tsx", "js", "mjs", "cjs", "jsx"],
      includeTests: true,
      exclude: ["build"],
    });

    expect(files).toContain("src/build-index.ts");
  });

  it("supports basename and exact file-path excludes without pruning siblings", () => {
    const byBasename = collectRel({
      languages: ["ts", "tsx", "js", "mjs", "cjs", "jsx"],
      includeTests: true,
      exclude: ["skip-me.js"],
    });
    const byExactPath = collectRel({
      languages: ["ts", "tsx", "js", "mjs", "cjs", "jsx"],
      includeTests: true,
      exclude: ["src/nested/exact-file.js"],
    });
    const siblingCheck = collectRel({
      languages: ["ts", "tsx", "js", "mjs", "cjs", "jsx"],
      includeTests: true,
      exclude: ["src/a.ts"],
    });

    expect(byBasename).not.toContain("src/skip-me.js");
    expect(byExactPath).not.toContain("src/nested/exact-file.js");
    expect(byExactPath).toContain("src/skip-me.js");
    expect(siblingCheck).not.toContain("src/a.ts");
    expect(siblingCheck).toContain("src/b.ts");
  });

  it("matches repo-relative vendor paths, not absolute parent directories", () => {
    const files = collectVendorRel({
      languages: ["ts", "tsx", "js", "mjs", "cjs", "jsx"],
      includeTests: true,
      exclude: ["vendor"],
    });

    expect(files).toContain("src/keep.ts");
    expect(files).not.toContain("src/vendor/skip.ts");
  });
});

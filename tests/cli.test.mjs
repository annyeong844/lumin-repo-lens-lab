import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const cliUrl = pathToFileURL(path.join(ROOT, "_lib/cli.mjs")).href;
const testPathsUrl = pathToFileURL(path.join(ROOT, "_lib/test-paths.mjs")).href;

let fixture;

beforeAll(() => {
  fixture = createTempRepoFixture({ prefix: "fx-vitest-cli-" });
});

afterAll(() => {
  fixture.cleanup();
});

async function parseWithArgs(argv) {
  const savedArgv = process.argv.slice();
  process.argv = ["node", "script.mjs", "--root", fixture.root, ...argv];
  const bust = `?t=${Date.now()}-${Math.random()}`;
  try {
    const mod = await import(`${cliUrl}${bust}`);
    return mod.parseCliArgs();
  } finally {
    process.argv = savedArgv;
  }
}

async function loadIsTestLikePath() {
  const mod = await import(
    `${testPathsUrl}?isTestLikePath=${Date.now()}-${Math.random()}`
  );
  return mod.isTestLikePath;
}

describe("parseCliArgs includeTests handling", () => {
  it("defaults to includeTests true and <root>/.audit output", async () => {
    const cli = await parseWithArgs([]);

    expect(cli.includeTests).toBe(true);
    expect(cli.output).toBe(path.join(path.resolve(fixture.root), ".audit"));
  });

  it("keeps includeTests true for --include-tests", async () => {
    await expect(parseWithArgs(["--include-tests"])).resolves.toMatchObject({
      includeTests: true,
    });
  });

  it("sets includeTests false for negation aliases", async () => {
    for (const flag of [
      "--no-include-tests",
      "--no-tests",
      "--exclude-tests",
    ]) {
      await expect(parseWithArgs([flag])).resolves.toMatchObject({
        includeTests: false,
      });
    }
  });

  it("sets includeTests false for --production", async () => {
    await expect(parseWithArgs(["--production"])).resolves.toMatchObject({
      includeTests: false,
    });
  });

  it("parses string-valued include-tests flags as booleans", async () => {
    await expect(
      parseWithArgs(["--include-tests=false"]),
    ).resolves.toMatchObject({
      includeTests: false,
    });
    await expect(
      parseWithArgs(["--include-tests=true"]),
    ).resolves.toMatchObject({
      includeTests: true,
    });
  });

  it("lets --production override --include-tests", async () => {
    await expect(
      parseWithArgs(["--include-tests", "--production"]),
    ).resolves.toMatchObject({
      includeTests: false,
    });
  });

  it("does not let unrelated flags perturb includeTests", async () => {
    await expect(parseWithArgs(["--verbose"])).resolves.toMatchObject({
      includeTests: true,
      verbose: true,
    });
  });
});

describe("isTestLikePath convention coverage", () => {
  it("recognizes JS/TS, pytest, Go, path-segment, and test-support conventions", async () => {
    const isTestLikePath = await loadIsTestLikePath();

    expect(isTestLikePath("src/foo.test.ts")).toBe(true);
    expect(isTestLikePath("src/bar.spec.js")).toBe(true);
    expect(isTestLikePath("src/test_foo.py")).toBe(true);
    expect(isTestLikePath("src/bar_test.go")).toBe(true);
    expect(isTestLikePath("/abs/tests/helper.ts")).toBe(true);
    expect(isTestLikePath("/abs/runtime-tests/workerd/index.ts")).toBe(true);
    expect(isTestLikePath("/abs/test-utils/helper.ts")).toBe(true);
    expect(isTestLikePath("src/foo-test-support.ts")).toBe(true);
  });

  it("does not match substring-only path names", async () => {
    const isTestLikePath = await loadIsTestLikePath();

    expect(isTestLikePath("src/contest.ts")).toBe(false);
  });
});

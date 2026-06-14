import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterEach, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const NODE = process.execPath;
const MEASURE_STALENESS = path.join(ROOT, "measure-staleness.mjs");
const AUDIT_REPO = path.join(ROOT, "audit-repo.mjs");

const fixtures = [];

function createFixture(prefix) {
  const fixture = createTempRepoFixture({ prefix });
  fixtures.push(fixture);
  return fixture;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: options.cwd ?? ROOT,
    encoding: "utf8",
  });

  expect(
    result.status,
    [
      `${command} ${args.join(" ")}`,
      `stdout:\n${result.stdout}`,
      `stderr:\n${result.stderr}`,
    ].join("\n\n"),
  ).toBe(0);

  return result;
}

function git(fixture, args) {
  return execFileSync("git", args, {
    cwd: fixture.root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function gitInitAndCommit(fixture) {
  git(fixture, ["init", "-q"]);
  git(fixture, ["config", "user.email", "test@example.com"]);
  git(fixture, ["config", "user.name", "Test User"]);
  git(fixture, ["add", "-A"]);
  git(fixture, ["commit", "-q", "-m", "fixture"]);
}

function writeManualSymbols(output, deadProdList) {
  mkdirSync(output, { recursive: true });
  writeFileSync(
    path.join(output, "symbols.json"),
    `${JSON.stringify({ deadProdList }, null, 2)}\n`,
  );
}

function readJson(file) {
  return JSON.parse(readFileSync(file, "utf8"));
}

function runMeasureStaleness(fixture, output, args = []) {
  run(
    NODE,
    [
      MEASURE_STALENESS,
      "--root",
      fixture.root,
      "--output",
      output,
      ...args,
    ],
    { cwd: ROOT },
  );
  return readJson(path.join(output, "staleness.json"));
}

afterEach(() => {
  for (const fixture of fixtures.splice(0).reverse()) {
    fixture.cleanup();
  }
});

describe("measure-staleness incremental cache", () => {
  it("reuses one full-file blame result for multiple dead candidates in the same file", () => {
    const fixture = createFixture("fx-vitest-staleness-blame-cache-");
    fixture.write(
      "src/helpers.ts",
      [
        "export const unusedOne = 1;",
        "export const unusedTwo = 2;",
        "export const used = unusedOne + unusedTwo;",
        "",
      ].join("\n"),
    );
    gitInitAndCommit(fixture);

    const out = fixture.mkdir("out-blame");
    writeManualSymbols(out, [
      { file: "src/helpers.ts", line: 1, symbol: "unusedOne" },
      { file: "src/helpers.ts", line: 2, symbol: "unusedTwo" },
    ]);

    const artifact = runMeasureStaleness(fixture, out, ["--skip-pickaxe"]);

    expect(artifact.enriched).toHaveLength(2);
    expect(artifact.summary?.performance).toMatchObject({
      deadCandidatesProcessed: 2,
      lineBlameGitCalls: 1,
      lineBlameCacheMisses: 1,
      lineBlameCacheHits: 1,
    });
  });

  it("reuses staleness results across output directories through cache-root", () => {
    const fixture = createFixture("fx-vitest-staleness-shared-cache-");
    fixture.write(
      "src/helpers.ts",
      [
        "export const unusedOne = 1;",
        "export const unusedTwo = 2;",
        "export const used = unusedOne + unusedTwo;",
        "",
      ].join("\n"),
    );
    gitInitAndCommit(fixture);

    const cacheRoot = fixture.mkdir(".lumin-cache");
    const coldOutput = fixture.mkdir("out-cold");
    const warmOutput = fixture.mkdir("out-warm");
    const deadProdList = [
      { file: "src/helpers.ts", line: 1, symbol: "unusedOne" },
      { file: "src/helpers.ts", line: 2, symbol: "unusedTwo" },
    ];
    writeManualSymbols(coldOutput, deadProdList);
    writeManualSymbols(warmOutput, deadProdList);

    const cold = runMeasureStaleness(fixture, coldOutput, [
      "--skip-pickaxe",
      "--cache-root",
      cacheRoot,
    ]);
    const warm = runMeasureStaleness(fixture, warmOutput, [
      "--skip-pickaxe",
      "--cache-root",
      cacheRoot,
    ]);

    expect(cold.meta?.incremental).toMatchObject({
      enabled: true,
      reusedResult: false,
    });
    expect(cold.summary?.performance?.deadCandidatesProcessed).toBe(2);

    expect(warm.meta?.incremental).toMatchObject({
      enabled: true,
      reusedResult: true,
    });
    expect(warm.meta?.symbolsSource).toBe(path.join(warmOutput, "symbols.json"));
    expect(warm.summary?.performance).toMatchObject({
      deadCandidatesProcessed: 0,
      fileTouchGitCalls: 0,
      lineBlameGitCalls: 0,
      symbolPickaxeGitCalls: 0,
    });
  });

  it("forwards audit-repo cache-root to measure-staleness in full profile", () => {
    const fixture = createFixture("fx-vitest-staleness-audit-forward-");
    fixture.write(
      "src/a.ts",
      [
        "export const live = 1;",
        "export const unusedOne = 2;",
        "export const unusedTwo = 3;",
        "",
      ].join("\n"),
    );
    gitInitAndCommit(fixture);

    const cacheRoot = fixture.mkdir(".lumin-cache");
    const coldOutput = fixture.mkdir("audit-cold");
    const warmOutput = fixture.mkdir("audit-warm");

    run(
      NODE,
      [
        AUDIT_REPO,
        "--root",
        fixture.root,
        "--output",
        coldOutput,
        "--profile",
        "full",
        "--production",
        "--cache-root",
        cacheRoot,
      ],
      { cwd: ROOT },
    );
    run(
      NODE,
      [
        AUDIT_REPO,
        "--root",
        fixture.root,
        "--output",
        warmOutput,
        "--profile",
        "full",
        "--production",
        "--cache-root",
        cacheRoot,
      ],
      { cwd: ROOT },
    );

    const cold = readJson(path.join(coldOutput, "staleness.json"));
    const warm = readJson(path.join(warmOutput, "staleness.json"));

    expect(cold.meta?.incremental).toMatchObject({
      enabled: true,
      reusedResult: false,
    });
    expect(cold.summary?.performance?.deadCandidatesProcessed).toBeGreaterThan(
      0,
    );
    expect(warm.meta?.incremental).toMatchObject({
      enabled: true,
      reusedResult: true,
    });
    expect(warm.summary?.performance).toMatchObject({
      deadCandidatesProcessed: 0,
      fileTouchGitCalls: 0,
      lineBlameGitCalls: 0,
      symbolPickaxeGitCalls: 0,
    });
  }, 120000);
});

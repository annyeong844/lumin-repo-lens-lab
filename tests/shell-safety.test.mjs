import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

const fixtures = [];
let injectionTriage;
let rootPythonTriage;
let goTriage;
let sfcTriage;
let normalStaleness;
let dollarStaleness;

function createFixture(prefix) {
  const fixture = createTempRepoFixture({ prefix });
  fixtures.push(fixture);
  return fixture;
}

function runScript(fixture, scriptName) {
  return execFileSync(
    process.execPath,
    [scriptName, "--root", fixture.root, "--output", fixture.output],
    {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function git(fixture, args) {
  execFileSync("git", args, {
    cwd: fixture.root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function gitInitAndCommit(fixture) {
  git(fixture, ["init", "-q"]);
  git(fixture, ["config", "user.email", "t@t"]);
  git(fixture, ["config", "user.name", "t"]);
  git(fixture, ["add", "-A"]);
  git(fixture, ["commit", "-q", "-m", "init"]);
}

beforeAll(() => {
  const injectionFixture = createFixture("fx-vitest-shell-injection-");
  injectionFixture.write("src/normal.ts", "export const x = 1;\n");
  injectionFixture.write("src/dollar$file.ts", "export const y = 2;\n");
  injectionFixture.write("src/weird$name.py", "def foo(): pass\n");
  runScript(injectionFixture, "triage-repo.mjs");
  injectionTriage = injectionFixture.readJson("triage.json", {
    from: "output",
  });

  const rootPythonFixture = createFixture("fx-vitest-shell-root-py-");
  rootPythonFixture.write("main.py", "def foo(): pass\n");
  runScript(rootPythonFixture, "triage-repo.mjs");
  rootPythonTriage = rootPythonFixture.readJson("triage.json", {
    from: "output",
  });

  const goFixture = createFixture("fx-vitest-shell-go-");
  goFixture.write("src/main.go", "package main\nfunc main() {}\n");
  runScript(goFixture, "triage-repo.mjs");
  goTriage = goFixture.readJson("triage.json", { from: "output" });

  const sfcFixture = createFixture("fx-vitest-shell-sfc-");
  sfcFixture.write(
    "src/App.vue",
    '<script setup lang="ts">const x = 1</script>\n',
  );
  sfcFixture.write(
    "src/Page.svelte",
    '<script lang="ts">export let y;</script>\n',
  );
  sfcFixture.write("src/Home.astro", "---\nconst z = 1;\n---\n");
  runScript(sfcFixture, "triage-repo.mjs");
  sfcTriage = sfcFixture.readJson("triage.json", { from: "output" });

  const normalStalenessFixture = createFixture("fx-vitest-shell-staleness-");
  normalStalenessFixture.write(
    "src/good.ts",
    "export const ok = 1;\nexport const dead = 2;\n",
  );
  gitInitAndCommit(normalStalenessFixture);
  runScript(normalStalenessFixture, "build-symbol-graph.mjs");
  runScript(normalStalenessFixture, "measure-staleness.mjs");
  normalStaleness = normalStalenessFixture.readJson("staleness.json", {
    from: "output",
  });

  const dollarStalenessFixture = createFixture("fx-vitest-shell-dollar-");
  dollarStalenessFixture.write("src/weird$name.ts", "export const dead = 1;\n");
  gitInitAndCommit(dollarStalenessFixture);
  runScript(dollarStalenessFixture, "build-symbol-graph.mjs");
  runScript(dollarStalenessFixture, "measure-staleness.mjs");
  dollarStaleness = dollarStalenessFixture.readJson("staleness.json", {
    from: "output",
  });
});

afterAll(() => {
  for (const fixture of fixtures.reverse()) {
    fixture.cleanup();
  }
});

describe("shell-metacharacter safe triage fixtures", () => {
  it("A1. triage completes without error on $-containing filenames", () => {
    expect(injectionTriage).toBeTruthy();
  });

  it("A2. triage counts TS files correctly including $-named files", () => {
    expect(injectionTriage.shape?.tsFiles).toBe(2);
  });

  it("A3. triage counts Python files correctly including $-named files", () => {
    expect(injectionTriage.shape?.pyFiles).toBe(1);
  });

  it("A4. triage records single-pass file collection telemetry", () => {
    expect(injectionTriage.performance?.fileCollection).toMatchObject({
      strategy: "single-pass-language-split",
      collectFilesCalls: 1,
      totalFilesCollected: 3,
      languageFiles: {
        ts: 2,
        py: 1,
      },
    });
  });
});

describe("root-level language discovery", () => {
  it("B1. root-only Python repo triage completes", () => {
    expect(rootPythonTriage).toBeTruthy();
  });

  it("B2. root-only Python repo detects main.py", () => {
    expect(rootPythonTriage.shape?.pyFiles).toBeGreaterThanOrEqual(1);
  });

  it("C1. Go-containing repo triage completes", () => {
    expect(goTriage).toBeTruthy();
  });

  it("C2. triage artifact exposes goFiles count", () => {
    expect(goTriage.shape?.goFiles).toBeGreaterThanOrEqual(1);
  });

  it("C3. triage artifact exposes SFC counts without parser support claims", () => {
    expect(sfcTriage.shape?.sfcFiles).toBe(3);
    expect(sfcTriage.byLanguage).toMatchObject({
      vue: 1,
      svelte: 1,
      astro: 1,
    });
    expect(sfcTriage.performance?.fileCollection?.languageFiles?.sfc).toBe(3);
  });
});

describe("shell-metacharacter safe triage summaries", () => {
  it('D1. topDirs["src"] reports all 3 files', () => {
    expect(injectionTriage.topDirs?.src?.files).toBe(3);
  });
});

describe("shell-metacharacter safe staleness fixtures", () => {
  it("E1. staleness runs on normal git repo", () => {
    expect(normalStaleness).toBeTruthy();
  });

  it("E2. staleness emits per-symbol records with stalenessTier", () => {
    expect(normalStaleness.enriched?.some((entry) => entry.stalenessTier)).toBe(
      true,
    );
  });

  it("F1. staleness handles $-containing filename without crash", () => {
    expect(dollarStaleness).toBeTruthy();
  });

  it("F2. staleness emits entry for $-named file with non-null fileLastTouchedAt", () => {
    const entry = dollarStaleness.enriched?.find((candidate) =>
      candidate.file?.includes("weird$name"),
    );

    expect(entry?.fileLastTouchedAt).not.toBeNull();
  });
});

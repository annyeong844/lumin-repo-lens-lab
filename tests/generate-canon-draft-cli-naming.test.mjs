import { execFileSync, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterEach, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const CLI = path.join(ROOT, "generate-canon-draft.mjs");
const fixtures = [];

function createFixture(prefix, options = {}) {
  const fixture = createTempRepoFixture({ prefix, ...options });
  fixtures.push(fixture);
  return fixture;
}

function writeNamingFixture(fixture) {
  fixture.write("_lib/canon-util.mjs", "export function doWork() {}\n");
  fixture.write("_lib/alias-helper.mjs", "export function loadAliases() {}\n");
  fixture.write(
    "_lib/resolver-core.mjs",
    "export function makeResolver() {}\n",
  );
  fixture.write(
    "src/app.mjs",
    "import { doWork } from '../_lib/canon-util.mjs';\n" +
      "export const x = doWork();\n",
  );
}

function writePlain(root, relPath, text) {
  const full = path.join(root, ...relPath.split("/"));
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, text);
  return full;
}

function runCli(fixture, args = [], options = {}) {
  return spawnSync(process.execPath, [CLI, "--root", fixture.root, ...args], {
    cwd: ROOT,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    ...options,
  });
}

function execCli(fixture, args = []) {
  execFileSync(process.execPath, [CLI, "--root", fixture.root, ...args], {
    cwd: ROOT,
    stdio: "ignore",
  });
}

afterEach(() => {
  while (fixtures.length > 0) {
    fixtures.pop().cleanup();
  }
});

describe("generate-canon-draft naming CLI", () => {
  it("T1/T8. --source naming emits the default draft and stderr summary", () => {
    const fixture = createFixture("fx-vitest-canon-naming-");
    writeNamingFixture(fixture);

    const result = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "naming",
    ]);
    const md = fixture.read("canonical-draft/naming.md");

    expect(result.status).toBe(0);
    expect(md).toContain("# Naming conventions draft");
    expect(md).toContain("## 1. File-naming cohorts");
    expect(md).toContain("## 2. Symbol-naming cohorts");
    expect(md).toContain("CohortIdentityShape: submodule | submodule::kind");
    expect(result.stderr).toMatch(/\d+ file cohorts/);
    expect(result.stderr).toMatch(/\d+ symbol cohorts/);
  }, 30_000);

  it("T2/T7/T9. source validation lists all sources, keeps regressions green, and rejects missing root", () => {
    const fixture = createFixture("fx-vitest-canon-naming-src-");
    writeNamingFixture(fixture);
    fixture.write("src/types.ts", "export type User = { id: string };\n");
    fixture.writeJson(
      "symbols.json",
      {
        meta: {
          tool: "build-symbol-graph.mjs",
          generated: "2026-04-21T00:00:00Z",
          root: fixture.root,
          supports: { identityFanIn: true },
        },
        defIndex: {
          "src/types.ts": {
            User: { name: "User", kind: "TSTypeAliasDeclaration", line: 1 },
          },
        },
        fanInByIdentity: {},
        reExportsByFile: {},
      },
      { to: "output" },
    );

    const badSource = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "foobar",
    ]);
    const typeOwnership = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "type-ownership",
    ]);
    const helperRegistry = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "helper-registry",
    ]);
    const missingRoot = spawnSync(
      process.execPath,
      [CLI, "--output", fixture.output, "--source", "naming"],
      {
        cwd: ROOT,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      },
    );

    expect(badSource.status).toBe(1);
    expect(badSource.stderr).toMatch(/type-ownership/);
    expect(badSource.stderr).toMatch(/helper-registry/);
    expect(badSource.stderr).toMatch(/topology/);
    expect(badSource.stderr).toMatch(/naming/);
    expect(typeOwnership.status).toBe(0);
    expect(helperRegistry.status).toBe(0);
    expect(missingRoot.status).toBe(1);
  }, 30_000);

  it("T3/T4/T5. keeps draft versioning, existing-canon header, and canon-output override", () => {
    const fixture = createFixture("fx-vitest-canon-naming-io-");
    const customOutput = createFixture("fx-vitest-canon-naming-custom-", {
      outputDirName: "audit",
    });
    writeNamingFixture(fixture);

    execCli(fixture, ["--output", fixture.output, "--source", "naming"]);
    const first = fixture.read("canonical-draft/naming.md");
    fixture.write("_lib/extra-helper.mjs", "export function extra() {}\n");
    execCli(fixture, ["--output", fixture.output, "--source", "naming"]);

    expect(readdirSync(fixture.path("canonical-draft"))).toContain(
      "naming.v2.md",
    );
    expect(fixture.read("canonical-draft/naming.md")).toBe(first);

    const existing = createFixture("fx-vitest-canon-naming-existing-");
    writeNamingFixture(existing);
    existing.write("canonical/naming.md", "# Existing canon\n");
    execCli(existing, ["--output", existing.output, "--source", "naming"]);
    expect(existing.read("canonical-draft/naming.md")).toContain(
      "⚠ Existing canon detected",
    );

    const override = createFixture("fx-vitest-canon-naming-override-");
    writeNamingFixture(override);
    execCli(override, [
      "--output",
      override.output,
      "--canon-output",
      customOutput.root,
      "--source",
      "naming",
    ]);
    expect(existsSync(path.join(customOutput.root, "naming.md"))).toBe(true);
    expect(existsSync(override.path("canonical-draft/naming.md"))).toBe(false);
  }, 30_000);

  it("T6/T10. preserves shell-sensitive paths and production scope text", () => {
    const parent = mkdtempSync(path.join(tmpdir(), "fx-vitest-canon-shell-"));
    fixtures.push({
      cleanup() {
        rmSync(parent, { recursive: true, force: true });
      },
    });
    const specialRoot = path.join(parent, "my $root");
    const specialOut = path.join(parent, "my $out");
    mkdirSync(specialRoot, { recursive: true });
    mkdirSync(specialOut, { recursive: true });
    writePlain(
      specialRoot,
      "package.json",
      '{"name":"nm-fx","type":"module"}\n',
    );
    const shellFixture = {
      root: specialRoot,
      output: specialOut,
      path(relPath) {
        return path.join(specialRoot, ...relPath.split("/"));
      },
      write(relPath, text) {
        return writePlain(specialRoot, relPath, text);
      },
    };
    writeNamingFixture(shellFixture);
    const shellResult = runCli(shellFixture, [
      "--output",
      shellFixture.output,
      "--source",
      "naming",
    ]);

    expect(shellResult.status).toBe(0);
    expect(existsSync(shellFixture.path("canonical-draft/naming.md"))).toBe(
      true,
    );

    const prod = createFixture("fx-vitest-canon-naming-prod-");
    writeNamingFixture(prod);
    execCli(prod, [
      "--output",
      prod.output,
      "--source",
      "naming",
      "--production",
    ]);
    expect(prod.read("canonical-draft/naming.md")).toContain(
      "Scope: TS/JS production files",
    );
  }, 30_000);
});

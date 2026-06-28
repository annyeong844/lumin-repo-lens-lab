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

function writePlain(root, relPath, text) {
  const full = path.join(root, ...relPath.split("/"));
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, text, "utf8");
}

function writeTypeFixture(fixture) {
  fixture.write(
    "package.json",
    JSON.stringify({ name: "cd-fx", type: "module" }),
  );
  fixture.write(
    "src/types.ts",
    "export type User = { id: string; name: string };\n",
  );
}

function writeSymbols(fixture, override = {}) {
  const base = {
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
    fanInByIdentity: { "src/types.ts::User": 2 },
    reExportsByFile: {},
  };
  fixture.writeJson("symbols.json", { ...base, ...override }, { to: "output" });
}

function writeShapeIndex(fixture, facts, { complete = true } = {}) {
  const groupsByHash = {};
  for (const fact of facts) {
    groupsByHash[fact.hash] ??= [];
    groupsByHash[fact.hash].push(fact.identity);
  }
  for (const ids of Object.values(groupsByHash)) ids.sort();
  fixture.writeJson(
    "shape-index.json",
    {
      schemaVersion: "shape-index.v1",
      meta: { complete },
      facts,
      groupsByHash,
      diagnostics: [],
    },
    { to: "output" },
  );
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

describe("generate-canon-draft type-ownership CLI", () => {
  it("T1/T8. emits a type-ownership draft with default output and production scope text", () => {
    const fixture = createFixture("fx-vitest-canon-type-");
    writeTypeFixture(fixture);
    writeSymbols(fixture);

    const result = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "type-ownership",
    ]);

    expect(result.status).toBe(0);
    const md = fixture.read("canonical-draft/type-ownership.md");
    expect(md).toContain("# Type ownership draft");
    expect(md).toContain("src/types.ts::User");
    expect(md).toMatch(/single-owner-(weak|strong)|zero-internal-fan-in/);

    const production = createFixture("fx-vitest-canon-type-prod-");
    writeTypeFixture(production);
    writeSymbols(production);
    execCli(production, [
      "--output",
      production.output,
      "--source",
      "type-ownership",
      "--production",
    ]);
    expect(production.read("canonical-draft/type-ownership.md")).toContain(
      "TS/JS production files",
    );
  }, 30_000);

  it("T2/T3. rejects unknown sources with the full source list and missing root", () => {
    const fixture = createFixture("fx-vitest-canon-type-source-");
    writeTypeFixture(fixture);
    writeSymbols(fixture);

    const badSource = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "foobar",
    ]);
    const missingRoot = spawnSync(
      process.execPath,
      [CLI, "--output", fixture.output, "--source", "type-ownership"],
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
    expect(missingRoot.status).not.toBe(0);
  }, 30_000);

  it("T4/T5/T6. preserves non-overwrite versioning, existing-canon header, and canon-output override", () => {
    const fixture = createFixture("fx-vitest-canon-type-io-");
    writeTypeFixture(fixture);
    writeSymbols(fixture);

    execCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "type-ownership",
    ]);
    const first = fixture.read("canonical-draft/type-ownership.md");
    fixture.write("src/more.ts", "export type Extra = { x: number };\n");
    writeSymbols(fixture, {
      defIndex: {
        "src/types.ts": {
          User: { name: "User", kind: "TSTypeAliasDeclaration", line: 1 },
        },
        "src/more.ts": {
          Extra: { name: "Extra", kind: "TSTypeAliasDeclaration", line: 1 },
        },
      },
      fanInByIdentity: {
        "src/types.ts::User": 2,
        "src/more.ts::Extra": 1,
      },
    });
    execCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "type-ownership",
    ]);

    expect(readdirSync(fixture.path("canonical-draft"))).toContain(
      "type-ownership.v2.md",
    );
    expect(fixture.read("canonical-draft/type-ownership.md")).toBe(first);

    const existing = createFixture("fx-vitest-canon-type-existing-");
    writeTypeFixture(existing);
    existing.write("canonical/type-ownership.md", "# Existing canon\n");
    writeSymbols(existing);
    execCli(existing, [
      "--output",
      existing.output,
      "--source",
      "type-ownership",
    ]);
    expect(existing.read("canonical-draft/type-ownership.md")).toContain(
      "⚠ Existing canon detected",
    );

    const override = createFixture("fx-vitest-canon-type-override-");
    const customOutput = createFixture("fx-vitest-canon-type-custom-", {
      outputDirName: "audit",
    });
    writeTypeFixture(override);
    writeSymbols(override);
    execCli(override, [
      "--output",
      override.output,
      "--canon-output",
      customOutput.root,
      "--source",
      "type-ownership",
    ]);
    expect(existsSync(path.join(customOutput.root, "type-ownership.md"))).toBe(
      true,
    );
    expect(existsSync(override.path("canonical-draft/type-ownership.md"))).toBe(
      false,
    );
  }, 30_000);

  it("T7/T9. handles shell-sensitive paths and missing symbols without failing", () => {
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
      JSON.stringify({ name: "cd-fx", type: "module" }),
    );
    writePlain(
      specialRoot,
      "src/types.ts",
      "export type User = { id: string; name: string };\n",
    );
    const shellFixture = {
      root: specialRoot,
      output: specialOut,
      path(relPath) {
        return path.join(specialRoot, ...relPath.split("/"));
      },
    };
    const result = runCli(shellFixture, [
      "--output",
      shellFixture.output,
      "--source",
      "type-ownership",
    ]);

    expect(result.status).toBe(0);
    expect(
      existsSync(shellFixture.path("canonical-draft/type-ownership.md")),
    ).toBe(true);
    expect(
      readFileSync(
        shellFixture.path("canonical-draft/type-ownership.md"),
        "utf8",
      ),
    ).toMatch(/fresh-ast-pass|barrels-opaque|opaque/);
  }, 30_000);

  it("T10. enriches the draft with optional shape-index evidence", () => {
    const fixture = createFixture("fx-vitest-canon-type-shape-");
    writeTypeFixture(fixture);
    const hash = `sha256:${"d".repeat(64)}`;
    writeSymbols(fixture, {
      defIndex: {
        "src/a.ts": {
          Result: { name: "Result", kind: "TSTypeAliasDeclaration", line: 1 },
        },
        "src/b.ts": {
          Result: { name: "Result", kind: "TSTypeAliasDeclaration", line: 1 },
        },
      },
      fanInByIdentity: {
        "src/a.ts::Result": 18,
        "src/b.ts::Result": 3,
      },
    });
    writeShapeIndex(fixture, [
      { identity: "src/a.ts::Result", hash },
      { identity: "src/b.ts::Result", hash },
    ]);

    const result = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "type-ownership",
    ]);
    const md = fixture.read("canonical-draft/type-ownership.md");

    expect(result.status).toBe(0);
    expect(md).toContain("DUPLICATE_STRONG");
    expect(md).toContain("## Shape evidence");
    expect(md).toContain("same-shape evidence");
    expect(md).toContain(hash);
  }, 30_000);
});

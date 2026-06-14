import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterEach, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const CLI = path.join(ROOT, "generate-canon-draft.mjs");
const fixtures = [];

function createFixture(prefix) {
  const fixture = createTempRepoFixture({ prefix });
  fixtures.push(fixture);
  return fixture;
}

function writeHelperFixture(fixture) {
  fixture.write(
    "src/util.ts",
    "export function renderHelperThing(x: number): number { return x + 1 }\n",
  );
  fixture.write(
    "src/consumer.ts",
    "import { renderHelperThing } from './util';\n" +
      "export const y = renderHelperThing(1);\n",
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

describe("generate-canon-draft helper-registry CLI", () => {
  it("T1. --source helper-registry emits the default helper draft with mode and fan-in metadata", () => {
    const fixture = createFixture("fx-vitest-canon-cli-");
    writeHelperFixture(fixture);

    const result = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "helper-registry",
    ]);
    const draftPath = fixture.path("canonical-draft/helper-registry.md");
    const md = readFileSync(draftPath, "utf8");

    expect(result.status).toBe(0);
    expect(md).toContain("# Helper registry draft");
    expect(md).toContain("src/util.ts::renderHelperThing");
    expect(md).toMatch(
      /shared-helper|central-helper|zero-internal-fan-in-helper|HELPER_LOCAL_COMMON/,
    );
    expect(md).toContain("FanInKind: consumer-file-count");
    expect(md).toContain("Mode: fresh-ast");
  });

  it("T2/T3/T4. source validation accepts helper-registry, preserves type-ownership, and rejects missing root", () => {
    const fixture = createFixture("fx-vitest-canon-src-");
    writeHelperFixture(fixture);
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
    const missingRoot = spawnSync(
      process.execPath,
      [CLI, "--output", fixture.output, "--source", "helper-registry"],
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
    expect(existsSync(fixture.path("canonical-draft/type-ownership.md"))).toBe(
      true,
    );
    expect(missingRoot.status).toBe(1);
  });

  it("T5/T6/T7. draft output preserves versioning, existing-canon headers, and --canon-output", () => {
    const fixture = createFixture("fx-vitest-canon-output-");
    const custom = createFixture("fx-vitest-canon-custom-");
    writeHelperFixture(fixture);
    fixture.write("canonical/helper-registry.md", "# Existing canon\n");

    execCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "helper-registry",
    ]);
    const firstContent = readFileSync(
      fixture.path("canonical-draft/helper-registry.md"),
      "utf8",
    );
    fixture.write(
      "src/extra.ts",
      "export function anotherHelper() { return 42 }\n",
    );
    execCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "helper-registry",
    ]);
    execCli(fixture, [
      "--output",
      fixture.output,
      "--canon-output",
      custom.root,
      "--source",
      "helper-registry",
    ]);

    expect(
      readdirSync(fixture.path("canonical-draft")).includes(
        "helper-registry.v2.md",
      ),
    ).toBe(true);
    expect(
      readFileSync(fixture.path("canonical-draft/helper-registry.md"), "utf8"),
    ).toBe(firstContent);
    expect(firstContent).toContain("Existing canon detected");
    expect(existsSync(path.join(custom.root, "helper-registry.md"))).toBe(true);
  });

  it("T8/T9. shell-sensitive paths and --production scope survive end-to-end", () => {
    const fixture = createFixture("fx-vitest-canon-shell-");
    const parent = fixture.mkdir("my $parent");
    const rootWithSymbols = path.join(parent, "my $root");
    const outWithSymbols = path.join(parent, "my $out");
    fixture.mkdir("my $parent/my $root");
    fixture.mkdir("my $parent/my $out");
    const shellFixture = {
      root: rootWithSymbols,
      output: outWithSymbols,
    };
    writeHelperFixture({
      write(relPath, text) {
        const target = path.join(rootWithSymbols, relPath);
        return fixture.write(path.relative(fixture.root, target), text);
      },
    });

    const shellRun = runCli(shellFixture, [
      "--output",
      outWithSymbols,
      "--source",
      "helper-registry",
    ]);
    const prodRun = runCli(shellFixture, [
      "--output",
      outWithSymbols,
      "--source",
      "helper-registry",
      "--production",
    ]);
    const md = readFileSync(
      path.join(rootWithSymbols, "canonical-draft", "helper-registry.v2.md"),
      "utf8",
    );

    expect(shellRun.status).toBe(0);
    expect(prodRun.status).toBe(0);
    expect(md).toContain("Scope: TS/JS production files");
  });

  it("T10/T11/T12. call-graph absence, enrichment mode, and stale call-graph warnings stay visible", () => {
    const fixture = createFixture("fx-vitest-canon-meta-");
    writeHelperFixture(fixture);

    const absent = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "helper-registry",
    ]);
    fixture.writeJson(
      "symbols.json",
      {
        helperOwnersByIdentity: {
          "src/util.ts::renderHelperThing": {
            signature: "(x: number) => number",
          },
        },
      },
      { to: "output" },
    );
    const enriched = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "helper-registry",
    ]);
    fixture.writeJson(
      "call-graph.json",
      {
        meta: {
          generated: new Date(Date.now() - 30 * 60 * 60 * 1000).toISOString(),
          root: fixture.root,
          tool: "build-call-graph.mjs",
        },
        summary: {},
        topCallees: [],
      },
      { to: "output" },
    );
    const stale = runCli(fixture, [
      "--output",
      fixture.output,
      "--source",
      "helper-registry",
    ]);
    const latest = readFileSync(
      fixture.path("canonical-draft/helper-registry.v3.md"),
      "utf8",
    );

    expect(absent.status).toBe(0);
    expect(enriched.status).toBe(0);
    expect(stale.status).toBe(0);
    expect(latest).toContain("fresh-ast + helper-owner enrichment");
    expect(latest).toContain("stale");
    expect(stale.stderr).toContain("callGraph=stale");
  });
});

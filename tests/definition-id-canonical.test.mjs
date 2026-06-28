import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

let fixture;
let symbols;
let calls;
let actionSafety;
let symbolDefId;

function runScript(scriptName) {
  execFileSync(
    process.execPath,
    [scriptName, "--root", fixture.root, "--output", fixture.output],
    {
      cwd: ROOT,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

beforeAll(() => {
  fixture = createTempRepoFixture({
    prefix: "pcef-defid-canonical-",
  });
  fixture.write(
    "src/lib.ts",
    [
      "function impl() {",
      "  return 1;",
      "}",
      "export { impl as publicApi };",
      "",
    ].join("\n"),
  );
  fixture.write(
    "src/consumer.ts",
    ['import { publicApi } from "./lib";', "publicApi();", ""].join("\n"),
  );

  runScript("build-symbol-graph.mjs");
  runScript("build-call-graph.mjs");
  fixture.writeJson(
    "dead-classify.json",
    {
      proposal_C_remove_symbol: [
        {
          file: "src/lib.ts",
          symbol: "publicApi",
          localName: "impl",
          line: 4,
          kind: "ExportSpecifier",
        },
      ],
      proposal_A_demote_to_internal: [],
      proposal_B_review: [],
      proposal_remove_export_specifier: [],
    },
    { to: "output" },
  );
  runScript("export-action-safety.mjs");

  symbols = fixture.readJson("symbols.json", { from: "output" });
  calls = fixture.readJson("call-graph.json", { from: "output" });
  actionSafety = fixture.readJson("export-action-safety.json", {
    from: "output",
  });
  symbolDefId = symbols.defIndex?.["src/lib.ts"]?.publicApi?.definitionId;
});

afterAll(() => {
  fixture.cleanup();
});

describe("canonical definitionId identity across producers", () => {
  it("D1. symbols.json emits canonical definitionId for export alias", () => {
    expect(symbolDefId).toEqual(
      expect.stringMatching(/^src\/lib\.ts#FunctionDeclaration:\d+-\d+$/),
    );
  });

  it("D2. call graph exportAliasMap uses the same definitionId", () => {
    expect(calls.exportAliasMap?.["src/lib.ts::publicApi"]).toBe(symbolDefId);
  });

  it("D3. callFanInByDefinitionId counts calls through aliased export", () => {
    expect(calls.callFanInByDefinitionId?.[symbolDefId]).toBe(1);
  });

  it("D4. callFanInByIdentity also counts the exported identity", () => {
    expect(calls.callFanInByIdentity?.["src/lib.ts::publicApi"]).toBe(1);
  });

  it("D5. export-action-safety target uses the same definitionId", () => {
    expect(actionSafety.findings?.[0]?.safeAction?.target?.definitionId).toBe(
      symbolDefId,
    );
  });
});

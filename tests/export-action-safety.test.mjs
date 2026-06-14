import { execFileSync } from "node:child_process";
import path from "node:path";

import { beforeAll, describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const ROOT = path.resolve(import.meta.dirname, "..");

function runFixture(files, proposals, buckets = {}) {
  const fixture = createTempRepoFixture({
    prefix: "vitest-export-action-safety-",
    packageJson: {
      name: "export-action-safety-fixture",
      private: true,
      type: "module",
    },
  });

  try {
    for (const [relPath, content] of Object.entries(files)) {
      fixture.write(relPath, content);
    }

    fixture.writeJson(
      "dead-classify.json",
      {
        proposal_C_remove_symbol: buckets.C ?? proposals,
        proposal_A_demote_to_internal: buckets.A ?? [],
        proposal_B_review: buckets.B ?? [],
        proposal_remove_export_specifier: [],
      },
      { to: "output" },
    );
    fixture.writeJson(
      "symbols.json",
      {
        defIndex: {},
        fanInByIdentity: {},
      },
      { to: "output" },
    );

    execFileSync(
      process.execPath,
      [
        path.join(ROOT, "export-action-safety.mjs"),
        "--root",
        fixture.root,
        "--output",
        fixture.output,
      ],
      { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] },
    );

    return fixture.readJson("export-action-safety.json", { from: "output" });
  } finally {
    fixture.cleanup();
  }
}

const common = {
  line: 1,
  kind: "VariableDeclaration",
  bucket: "C",
};

describe("export-action-safety concrete safe actions", () => {
  describe("side-effect initializer", () => {
    let action;

    beforeAll(() => {
      const artifact = runFixture(
        {
          "src/token.ts": "export const token = registerTelemetry();\n",
        },
        [{ ...common, file: "src/token.ts", symbol: "token" }],
      );
      action = artifact.findings[0].safeAction;
    });

    it("A1. selects demote action", () => {
      expect(action?.kind).toBe("demote_export_declaration");
    });

    it("A1b. has no selected-action blockers", () => {
      expect(action?.actionBlockers).toEqual([]);
    });

    it("A1c. blocks stronger delete action only", () => {
      expect(action?.strongerActionBlockers).toContain(
        "side-effect-initializer",
      );
    });
  });

  describe("local value references", () => {
    let action;

    beforeAll(() => {
      const artifact = runFixture(
        {
          "src/size.ts":
            "export const SIZE = 12;\nexport const buttonSize = SIZE + 4;\n",
        },
        [{ ...common, file: "src/size.ts", symbol: "SIZE" }],
      );
      action = artifact.findings[0].safeAction;
    });

    it("A2. preserve binding via demote", () => {
      expect(action?.kind).toBe("demote_export_declaration");
    });

    it("A2b. block stronger delete only", () => {
      expect(action?.strongerActionBlockers).toContain("local-refs-present");
    });
  });

  describe("local type references", () => {
    let action;

    beforeAll(() => {
      const artifact = runFixture(
        {
          "src/types.ts":
            "export type Options = { debug: boolean };\nconst defaults: Options = { debug: false };\n",
        },
        [
          {
            ...common,
            file: "src/types.ts",
            symbol: "Options",
            kind: "TSTypeAliasDeclaration",
          },
        ],
      );
      action = artifact.findings[0].safeAction;
    });

    it("A3. preserve type binding via demote", () => {
      expect(action?.kind).toBe("demote_export_declaration");
    });

    it("A3b. block type deletion only", () => {
      expect(action?.strongerActionBlockers).toContain("local-refs-present");
    });
  });

  it("A4. unreferenced interface can delete type declaration", () => {
    const artifact = runFixture(
      {
        "src/dead-type.ts":
          "export interface InternalOptions { debug: boolean }\n",
      },
      [
        {
          ...common,
          file: "src/dead-type.ts",
          symbol: "InternalOptions",
          kind: "TSInterfaceDeclaration",
        },
      ],
    );

    expect(artifact.findings[0].safeAction?.kind).toBe(
      "delete_type_declaration",
    );
  });

  describe("B bucket local type declaration dependency", () => {
    let action;

    beforeAll(() => {
      const proposal = {
        ...common,
        bucket: "B",
        file: "src/public-types.ts",
        symbol: "Internal",
        kind: "TSTypeAliasDeclaration",
        declarationExportDependency: true,
        declarationExportRefs: { count: 1, lines: [2] },
        fileInternalRefs: { typeRefs: 1, valueRefs: 0 },
      };
      const artifact = runFixture(
        {
          "src/public-types.ts": [
            "export type Internal = string;",
            "export interface PublicThing { value: Internal }",
            "",
          ].join("\n"),
        },
        [],
        { C: [], B: [proposal] },
      );
      action = artifact.findings[0].safeAction;
    });

    it("A4b. gets demote action", () => {
      expect(action?.kind).toBe("demote_export_declaration");
    });

    it("A4c. blocks stronger delete only", () => {
      expect(action?.strongerActionBlockers).toContain("local-refs-present");
    });
  });

  describe("partial multi-declarator", () => {
    let finding;

    beforeAll(() => {
      const artifact = runFixture(
        {
          "src/multi.ts": "export const a = 1, b = 2;\n",
        },
        [{ ...common, file: "src/multi.ts", symbol: "a" }],
      );
      finding = artifact.findings[0];
    });

    it("A5. has no safe action in v1", () => {
      expect(finding.safeAction).toBeNull();
    });

    it("A5b. records action blocker", () => {
      expect(finding.actionBlockers).toContain("partial-multi-declarator");
    });
  });

  it("A6. re-export-from-source remains review in v1", () => {
    const artifact = runFixture(
      {
        "src/reexport.ts": 'export { value } from "./source";\n',
      },
      [
        {
          ...common,
          file: "src/reexport.ts",
          symbol: "value",
          kind: "ExportSpecifier",
        },
      ],
    );
    const finding = artifact.findings[0];

    expect(finding.safeAction).toBeNull();
    expect(finding.actionBlockers).toContain("re-export-from-source");
  });

  it("A7. last export safe action includes module marker patch", () => {
    const artifact = runFixture(
      {
        "src/only.ts": "export const only = 1;\n",
      },
      [{ ...common, file: "src/only.ts", symbol: "only" }],
    );
    const action = artifact.findings[0].safeAction;

    expect(action?.requiresModuleMarker).toBe(true);
    expect(action?.edits).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "insert",
          text: expect.stringContaining("export {};"),
        }),
      ]),
    );
  });
});

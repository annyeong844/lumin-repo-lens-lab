import { describe, expect, it } from "vitest";

import {
  HELPER_OWNER_KINDS,
  UNCERTAIN_REASONS,
} from "../_lib/canon-draft-utils.mjs";
import {
  collectHelperIdentities,
  renderHelperRegistry,
} from "../_lib/canon-draft-helpers.mjs";

const ROOT = "/fx";

function makeExtractFn(perFile) {
  return (absFile) =>
    perFile.get(absFile) ?? { defs: [], uses: [], reExports: [] };
}

function makeResolver(resolves) {
  return (fromFile, spec) => resolves.get(`${fromFile}|${spec}`) ?? null;
}

function collect({ files, perFile, resolves = new Map(), options = {} }) {
  return collectHelperIdentities({
    files,
    root: ROOT,
    extractFn: makeExtractFn(perFile),
    resolveSpecifier: makeResolver(resolves),
    ...options,
  });
}

function render(result, meta = {}) {
  return renderHelperRegistry({
    helperDefsByIdentity: result.helperDefsByIdentity,
    helpersByName: result.helpersByName,
    distinctConsumerFiles: result.distinctConsumerFiles,
    diagnostics: result.diagnostics,
    meta: {
      scope: "TS/JS including tests",
      helperContamination: "unavailable",
      ...meta,
    },
  });
}

describe("helper registry aggregation", () => {
  it("I1/I3. inventories helpers and keeps exported-never-called helpers with fan-in 0", () => {
    const owner = "/fx/src/util.ts";
    const consumer = "/fx/src/consumer.ts";
    const perFile = new Map([
      [
        owner,
        {
          defs: [
            { name: "parseJson", kind: "FunctionDeclaration", line: 3 },
            { name: "unusedButPublic", kind: "FunctionDeclaration", line: 5 },
          ],
          uses: [],
          reExports: [],
        },
      ],
      [
        consumer,
        {
          defs: [],
          uses: [
            {
              fromSpec: "./util",
              name: "parseJson",
              kind: "import",
              typeOnly: false,
            },
          ],
          reExports: [],
        },
      ],
    ]);
    const result = collect({
      files: [owner, consumer],
      perFile,
      resolves: new Map([[`${consumer}|./util`, owner]]),
    });

    expect(
      result.helperDefsByIdentity.get("src/util.ts::parseJson"),
    ).toMatchObject({ fanIn: 1 });
    expect(
      result.helperDefsByIdentity.get("src/util.ts::unusedButPublic"),
    ).toMatchObject({ fanIn: 0 });
    expect(result.meta.helperContamination).toBe("unavailable");
  });

  it("I2/I5/I6/I20/I21. fan-in counts consumer files and rejects self/type-only/namespace/default noise", () => {
    const owner = "/fx/src/util.ts";
    const consumer = "/fx/src/consumer.ts";
    const perFile = new Map([
      [
        owner,
        {
          defs: [{ name: "helper", kind: "FunctionDeclaration", line: 2 }],
          uses: [
            {
              fromSpec: "./util",
              name: "helper",
              kind: "import",
              typeOnly: false,
            },
          ],
          reExports: [],
        },
      ],
      [
        consumer,
        {
          defs: [],
          uses: [
            {
              fromSpec: "./util",
              name: "helper",
              kind: "import",
              typeOnly: false,
            },
            {
              fromSpec: "./util",
              name: "helper",
              kind: "import",
              typeOnly: false,
            },
            {
              fromSpec: "./util",
              name: "helper",
              kind: "import",
              typeOnly: true,
            },
            { fromSpec: "./util", name: "*", kind: "import", typeOnly: false },
            {
              fromSpec: "./util",
              name: "default",
              kind: "import",
              typeOnly: false,
            },
          ],
          reExports: [],
        },
      ],
    ]);
    const result = collect({
      files: [owner, consumer],
      perFile,
      resolves: new Map([
        [`${owner}|./util`, owner],
        [`${consumer}|./util`, owner],
      ]),
    });

    expect(
      result.helperDefsByIdentity.get("src/util.ts::helper"),
    ).toMatchObject({ fanIn: 1 });
  });

  it("I4/I7. duplicate helper groups and central-helper rows render from fan-in", () => {
    const ownerA = "/fx/src/a.ts";
    const ownerB = "/fx/src/b.ts";
    const consumers = ["/fx/src/c1.ts", "/fx/src/c2.ts", "/fx/src/c3.ts"];
    const perFile = new Map([
      [
        ownerA,
        {
          defs: [{ name: "doWork", kind: "FunctionDeclaration", line: 1 }],
          uses: [],
          reExports: [],
        },
      ],
      [
        ownerB,
        {
          defs: [{ name: "doWork", kind: "FunctionDeclaration", line: 1 }],
          uses: [],
          reExports: [],
        },
      ],
      ...consumers.map((consumer, index) => [
        consumer,
        {
          defs: [],
          uses: [
            {
              fromSpec: index === 0 ? "./b" : "./a",
              name: "doWork",
              kind: "import",
              typeOnly: false,
            },
          ],
          reExports: [],
        },
      ]),
    ]);
    const resolves = new Map([
      [`${consumers[0]}|./b`, ownerB],
      [`${consumers[1]}|./a`, ownerA],
      [`${consumers[2]}|./a`, ownerA],
    ]);
    const result = collect({
      files: [ownerA, ownerB, ...consumers],
      perFile,
      resolves,
    });
    const md = render(result);

    expect(result.helpersByName.get("doWork")).toHaveLength(2);
    expect(result.helperDefsByIdentity.get("src/a.ts::doWork")).toMatchObject({
      fanIn: 2,
    });
    expect(md).toContain("HELPER_DUPLICATE_REVIEW");
  });

  it("I8/I9. filters unsupported definition kinds but keeps exported const-var helpers", () => {
    const owner = "/fx/src/k.ts";
    const perFile = new Map([
      [
        owner,
        {
          defs: [
            { name: "topLevelFn", kind: "FunctionDeclaration", line: 1 },
            { name: "aClass", kind: "ClassDeclaration", line: 5 },
            { name: "neverMethod", kind: "MethodDefinition", line: 10 },
            { name: "constHelper", kind: "const-var", line: 15 },
          ],
          uses: [],
          reExports: [],
        },
      ],
    ]);
    const result = collect({ files: [owner], perFile });

    expect([...result.helperDefsByIdentity.keys()]).toEqual([
      "src/k.ts::topLevelFn",
      "src/k.ts::constHelper",
    ]);
  });
});

describe("helper registry diagnostics and rendering", () => {
  it("I10. extractor failures become parse-error diagnostics without stopping other files", () => {
    const bad = "/fx/src/bad.ts";
    const ok = "/fx/src/ok.ts";
    const result = collectHelperIdentities({
      files: [bad, ok],
      root: ROOT,
      extractFn(absFile) {
        if (absFile === bad) throw new Error("forced parse failure");
        return {
          defs: [{ name: "okHelper", kind: "FunctionDeclaration", line: 1 }],
          uses: [],
          reExports: [],
        };
      },
      resolveSpecifier: makeResolver(new Map()),
    });

    expect(result.diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          kind: "parse-error",
          target: "src/bad.ts",
        }),
      ]),
    );
    expect(result.helperDefsByIdentity.has("src/ok.ts::okHelper")).toBe(true);
  });

  it("I11/I12/I13. call-graph cross-checks record fresh, stale, and absent states", () => {
    const owner = "/fx/src/util.ts";
    const perFile = new Map([
      [
        owner,
        {
          defs: [{ name: "parseJson", kind: "FunctionDeclaration", line: 1 }],
          uses: [],
          reExports: [],
        },
      ],
    ]);
    const fresh = collect({
      files: [owner],
      perFile,
      options: {
        callGraph: {
          meta: { generated: new Date().toISOString() },
          topCallees: [{ file: "src/util.ts", name: "parseJson", count: 3 }],
        },
      },
    });
    const stale = collect({
      files: [owner],
      perFile,
      options: {
        callGraph: {
          meta: {
            generated: new Date(Date.now() - 30 * 60 * 60 * 1000).toISOString(),
          },
          topCallees: [],
        },
      },
    });
    const absent = collect({ files: [owner], perFile });

    expect(fresh.meta.callGraphStaleness).toBe("fresh");
    expect(fresh.diagnostics).toEqual(
      expect.arrayContaining([
        expect.objectContaining({ kind: "call-graph-cross-check" }),
      ]),
    );
    expect(stale.meta.callGraphStaleness).toBe("stale");
    expect(render(stale, { ...stale.meta, callGraphAgeHours: 30 })).toContain(
      "stale",
    );
    expect(absent.meta.callGraphStaleness).toBe("absent");
  });

  it("I14/I15/I18/I19. render keeps contamination mode, existing-canon, and empty inventory semantics visible", () => {
    const owner = "/fx/src/util.ts";
    const perFile = new Map([
      [
        owner,
        {
          defs: [
            { name: "unsafeHelper", kind: "FunctionDeclaration", line: 1 },
          ],
          uses: [],
          reExports: [],
        },
      ],
    ]);
    const enriched = collect({
      files: [owner],
      perFile,
      options: {
        symbols: {
          helperOwnersByIdentity: {
            "src/util.ts::unsafeHelper": {
              signature: "(x: any) => void",
              anyContamination: { label: "severely-any-contaminated" },
            },
          },
        },
      },
    });
    const empty = collect({ files: [], perFile: new Map() });

    expect(render(enriched, enriched.meta)).toContain(
      "fresh-ast + helper-owner enrichment",
    );
    expect(render(enriched, enriched.meta)).toContain(
      "severely-any-contaminated-helper",
    );
    expect(render(empty, { existingCanon: true })).toContain(
      "Existing canon detected",
    );
    expect(render(empty)).not.toContain("Notes");
  });

  it("I16/I17. helper owner constants stay frozen", () => {
    expect(Object.isFrozen(HELPER_OWNER_KINDS)).toBe(true);
    expect(Object.isFrozen(UNCERTAIN_REASONS)).toBe(true);
    expect(UNCERTAIN_REASONS).toHaveLength(4);
  });

  it("I22. re-export and alias hops attribute fan-in to terminal owner", () => {
    const owner = "/fx/src/util.ts";
    const barrel = "/fx/src/index.ts";
    const consumer = "/fx/src/consumer.ts";
    const perFile = new Map([
      [
        owner,
        {
          defs: [{ name: "tryParse", kind: "FunctionDeclaration", line: 1 }],
          uses: [],
          reExports: [],
        },
      ],
      [
        barrel,
        {
          defs: [],
          uses: [],
          reExports: [
            {
              source: "./util",
              name: "tryParseJson",
              importedName: "tryParse",
            },
          ],
        },
      ],
      [
        consumer,
        {
          defs: [],
          uses: [
            {
              fromSpec: "./util",
              name: "tryParse",
              kind: "import",
              typeOnly: false,
            },
          ],
          reExports: [],
        },
      ],
    ]);
    const result = collect({
      files: [owner, barrel, consumer],
      perFile,
      resolves: new Map([
        [`${barrel}|./util`, owner],
        [`${consumer}|./util`, owner],
      ]),
    });

    expect(
      result.helperDefsByIdentity.get("src/util.ts::tryParse"),
    ).toMatchObject({ fanIn: 1 });
    expect(result.helperDefsByIdentity.has("src/index.ts::parse")).toBe(false);
  });
});

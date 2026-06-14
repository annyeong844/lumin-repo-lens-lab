import { describe, expect, it } from "vitest";

import { detectTypeOwnershipDrift } from "../_lib/check-canon-types.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const TYPE_LABEL_SET = new Set([
  "zero-internal-fan-in",
  "low-signal-type-name",
  "DUPLICATE_STRONG",
  "DUPLICATE_REVIEW",
  "LOCAL_COMMON_NAME",
  "single-owner-strong",
  "single-owner-weak",
  "severely-any-contaminated",
  "ANY_COLLISION",
]);

function withFixture(testFn) {
  const fixture = createTempRepoFixture({ prefix: "fx-vitest-check-types-" });
  try {
    return testFn(fixture);
  } finally {
    fixture.cleanup();
  }
}

function writeCanon(fixture, relPath, rows, { fanInSpace = false } = {}) {
  const lines = [];
  if (fanInSpace) {
    lines.push(
      "| Name | Identity | Owner | Fan-in | Fan-in space | Status | Tags |",
    );
    lines.push(
      "|------|----------|-------|-------:|--------------|--------|------|",
    );
  } else {
    lines.push("| Name | Identity | Owner | Fan-in | Status | Tags |");
    lines.push("|------|----------|-------|-------:|--------|------|");
  }
  for (const row of rows) {
    if (fanInSpace) {
      lines.push(
        `| \`${row.name}\` | \`${row.identity}\` | \`${row.owner}\` | ${row.fanIn} | ${row.fanInSpace ?? "value 0, type 0, broad 0"} | ${row.label} ✅ | |`,
      );
    } else {
      lines.push(
        `| \`${row.name}\` | \`${row.identity}\` | \`${row.owner}\` | ${row.fanIn} | ${row.label} ✅ | |`,
      );
    }
  }
  fixture.write(relPath, `${lines.join("\n")}\n`);
  return fixture.path(relPath);
}

function makeSymbols(typeDefs) {
  const defIndex = {};
  const fanInByIdentity = {};
  for (const def of typeDefs) {
    defIndex[def.ownerFile] ??= {};
    defIndex[def.ownerFile][def.name] = {
      kind: def.kind ?? "TSInterfaceDeclaration",
      line: def.line,
      anyContamination: def.anyContamination ?? null,
    };
    fanInByIdentity[`${def.ownerFile}::${def.name}`] = def.fanIn;
  }
  return {
    meta: { scope: "fixture", supports: { identityFanIn: true } },
    defIndex,
    fanInByIdentity,
    reExportsByFile: {},
  };
}

function makeHash(ch) {
  return `sha256:${ch.repeat(64)}`;
}

function makeShapeIndex(facts, { complete = true } = {}) {
  const groupsByHash = {};
  for (const fact of facts) {
    groupsByHash[fact.hash] ??= [];
    groupsByHash[fact.hash].push(fact.identity);
  }
  for (const ids of Object.values(groupsByHash)) ids.sort();
  return {
    schemaVersion: "shape-index.v1",
    meta: { complete },
    facts: facts.map((fact) => ({ ...fact })),
    groupsByHash,
  };
}

function detect({ canonPath, symbols, shapeIndex }) {
  return detectTypeOwnershipDrift({
    canonPath,
    symbols,
    shapeIndex,
    canonLabelSet: TYPE_LABEL_SET,
  });
}

describe("type ownership drift engine", () => {
  it("T-1/T-2. missing canon skips without producing drift or Markdown", () =>
    withFixture((fixture) => {
      const result = detect({
        canonPath: fixture.path("missing.md"),
        symbols: makeSymbols([]),
      });

      expect(result).toMatchObject({
        status: "skipped-missing-canon",
        drifts: [],
        reportMarkdown: null,
      });
    }));

  it("T-3..T-10. detects added, removed, and label-changed identities", () =>
    withFixture((fixture) => {
      const addedCanon = writeCanon(fixture, "added.md", [
        {
          name: "Foo",
          identity: "src/foo.ts::Foo",
          owner: "src/foo.ts:10",
          fanIn: 3,
          label: "single-owner-strong",
        },
      ]);
      const added = detect({
        canonPath: addedCanon,
        symbols: makeSymbols([
          { name: "Foo", ownerFile: "src/foo.ts", line: 10, fanIn: 3 },
          { name: "Bar", ownerFile: "src/bar.ts", line: 5, fanIn: 1 },
        ]),
      }).drifts.filter((drift) => drift.category === "identity-added");
      expect(added).toEqual([
        expect.objectContaining({
          identity: "src/bar.ts::Bar",
          family: "added",
        }),
      ]);

      const removedCanon = writeCanon(fixture, "removed.md", [
        {
          name: "Foo",
          identity: "src/foo.ts::Foo",
          owner: "src/foo.ts:10",
          fanIn: 3,
          label: "single-owner-strong",
        },
        {
          name: "Gone",
          identity: "src/gone.ts::Gone",
          owner: "src/gone.ts:2",
          fanIn: 1,
          label: "single-owner-weak",
        },
      ]);
      const removed = detect({
        canonPath: removedCanon,
        symbols: makeSymbols([
          { name: "Foo", ownerFile: "src/foo.ts", line: 10, fanIn: 3 },
        ]),
      }).drifts.filter((drift) => drift.category === "identity-removed");
      expect(removed).toEqual([
        expect.objectContaining({ identity: "src/gone.ts::Gone" }),
      ]);

      const labelCanon = writeCanon(fixture, "label.md", [
        {
          name: "Foo",
          identity: "src/foo.ts::Foo",
          owner: "src/foo.ts:10",
          fanIn: 0,
          label: "zero-internal-fan-in",
        },
      ]);
      const labelChanged = detect({
        canonPath: labelCanon,
        symbols: makeSymbols([
          { name: "Foo", ownerFile: "src/foo.ts", line: 10, fanIn: 3 },
        ]),
      }).drifts.filter((drift) => drift.category === "label-changed");
      expect(labelChanged).toEqual([
        expect.objectContaining({
          family: "label-changed",
          canon: expect.objectContaining({ label: "zero-internal-fan-in" }),
          fresh: expect.objectContaining({ label: "single-owner-strong" }),
        }),
      ]);
    }));

  it("T-11..T-15c. upgrades a 1:1 same-name move to owner-changed with canonical identity shape", () =>
    withFixture((fixture) => {
      const canonPath = writeCanon(fixture, "owner.md", [
        {
          name: "Foo",
          identity: "src/a.ts::Foo",
          owner: "src/a.ts:10",
          fanIn: 3,
          label: "single-owner-strong",
        },
      ]);
      const result = detect({
        canonPath,
        symbols: makeSymbols([
          { name: "Foo", ownerFile: "src/b.ts", line: 4, fanIn: 3 },
        ]),
      });

      expect(
        result.drifts.filter((drift) => drift.category === "identity-added"),
      ).toHaveLength(0);
      expect(
        result.drifts.filter((drift) => drift.category === "identity-removed"),
      ).toHaveLength(0);
      expect(
        result.drifts.filter((drift) => drift.category === "owner-changed"),
      ).toEqual([
        expect.objectContaining({
          identity: "src/a.ts::Foo",
          family: "structural-status-changed",
          canon: expect.objectContaining({
            identity: "src/a.ts::Foo",
            owner: expect.stringMatching(/^src\/a\.ts/),
            label: expect.any(String),
          }),
          fresh: expect.objectContaining({
            identity: "src/b.ts::Foo",
            owner: expect.stringMatching(/^src\/b\.ts/),
            label: expect.any(String),
          }),
        }),
      ]);
      expect(result.drifts[0].identity).not.toMatch(/→|->/);
    }));

  it("T-16..T-18. keeps ambiguous 2:1 same-name moves as low-confidence add/remove drift", () =>
    withFixture((fixture) => {
      const canonPath = writeCanon(fixture, "ambig.md", [
        {
          name: "X",
          identity: "src/a.ts::X",
          owner: "src/a.ts:1",
          fanIn: 1,
          label: "single-owner-weak",
        },
      ]);
      const result = detect({
        canonPath,
        symbols: makeSymbols([
          { name: "X", ownerFile: "src/b.ts", line: 1, fanIn: 1 },
          { name: "X", ownerFile: "src/c.ts", line: 1, fanIn: 1 },
        ]),
      });
      const added = result.drifts.filter(
        (drift) => drift.category === "identity-added",
      );
      const removed = result.drifts.filter(
        (drift) => drift.category === "identity-removed",
      );

      expect(
        result.drifts.filter((drift) => drift.category === "owner-changed"),
      ).toHaveLength(0);
      expect(added).toHaveLength(2);
      expect(removed).toHaveLength(1);
      expect(
        [...added, ...removed].every((drift) => drift.confidence === "low"),
      ).toBe(true);
    }));

  it("T-18b..T-18g. uses unique shape pairs to ground only the resolvable owner-change subset", () =>
    withFixture((fixture) => {
      const upgradeCanon = writeCanon(fixture, "shape-upgrade.md", [
        {
          name: "X",
          identity: "src/a.ts::X",
          owner: "src/a.ts:1",
          fanIn: 1,
          label: "single-owner-weak",
        },
      ]);
      const upgraded = detect({
        canonPath: upgradeCanon,
        symbols: makeSymbols([
          { name: "X", ownerFile: "src/b.ts", line: 1, fanIn: 1 },
          { name: "X", ownerFile: "src/c.ts", line: 1, fanIn: 1 },
        ]),
        shapeIndex: makeShapeIndex([
          { identity: "src/a.ts::X", hash: makeHash("a") },
          { identity: "src/b.ts::X", hash: makeHash("a") },
          { identity: "src/c.ts::X", hash: makeHash("b") },
        ]),
      });
      expect(
        upgraded.drifts.filter((drift) => drift.category === "owner-changed"),
      ).toEqual([
        expect.objectContaining({
          canon: expect.objectContaining({ identity: "src/a.ts::X" }),
          fresh: expect.objectContaining({ identity: "src/b.ts::X" }),
        }),
      ]);
      expect(
        upgraded.drifts.filter((drift) => drift.category === "identity-added"),
      ).toEqual([
        expect.objectContaining({
          identity: "src/c.ts::X",
          confidence: "high",
        }),
      ]);

      const partialCanon = writeCanon(fixture, "shape-partial.md", [
        {
          name: "X",
          identity: "src/a.ts::X",
          owner: "src/a.ts:1",
          fanIn: 1,
          label: "single-owner-weak",
        },
        {
          name: "X",
          identity: "src/b.ts::X",
          owner: "src/b.ts:1",
          fanIn: 1,
          label: "single-owner-weak",
        },
      ]);
      const partial = detect({
        canonPath: partialCanon,
        symbols: makeSymbols([
          { name: "X", ownerFile: "src/c.ts", line: 1, fanIn: 1 },
          { name: "X", ownerFile: "src/d.ts", line: 1, fanIn: 1 },
        ]),
        shapeIndex: makeShapeIndex([
          { identity: "src/a.ts::X", hash: makeHash("a") },
          { identity: "src/c.ts::X", hash: makeHash("a") },
        ]),
      });
      expect(
        partial.drifts.filter((drift) => drift.category === "owner-changed"),
      ).toEqual([
        expect.objectContaining({
          canon: expect.objectContaining({ identity: "src/a.ts::X" }),
          fresh: expect.objectContaining({ identity: "src/c.ts::X" }),
        }),
      ]);
      expect(
        partial.drifts.filter((drift) => drift.category === "identity-added"),
      ).toEqual([
        expect.objectContaining({ identity: "src/d.ts::X", confidence: "low" }),
      ]);
      expect(
        partial.drifts.filter((drift) => drift.category === "identity-removed"),
      ).toEqual([
        expect.objectContaining({ identity: "src/b.ts::X", confidence: "low" }),
      ]);
    }));

  it("T-18h/T-18i. malformed shape-index falls closed to low-confidence add/remove ambiguity", () =>
    withFixture((fixture) => {
      const canonPath = writeCanon(fixture, "shape-invalid.md", [
        {
          name: "X",
          identity: "src/a.ts::X",
          owner: "src/a.ts:1",
          fanIn: 1,
          label: "single-owner-weak",
        },
      ]);
      const result = detect({
        canonPath,
        symbols: makeSymbols([
          { name: "X", ownerFile: "src/b.ts", line: 1, fanIn: 1 },
          { name: "X", ownerFile: "src/c.ts", line: 1, fanIn: 1 },
        ]),
        shapeIndex: { schemaVersion: "wrong" },
      });

      expect(
        result.drifts.filter((drift) => drift.category === "owner-changed"),
      ).toHaveLength(0);
      expect(
        result.drifts.filter((drift) => drift.category === "identity-added"),
      ).toHaveLength(2);
      expect(
        result.drifts.filter((drift) => drift.category === "identity-removed"),
      ).toHaveLength(1);
      expect(result.drifts.every((drift) => drift.confidence === "low")).toBe(
        true,
      );
    }));

  it("T-19..T-25. keeps clean render, fan-in-space parsing, owner labels, zero-section omission, and drift kind", () =>
    withFixture((fixture) => {
      const cleanCanon = writeCanon(fixture, "clean.md", [
        {
          name: "Foo",
          identity: "src/foo.ts::Foo",
          owner: "src/foo.ts:10",
          fanIn: 3,
          label: "single-owner-strong",
        },
      ]);
      const clean = detect({
        canonPath: cleanCanon,
        symbols: makeSymbols([
          { name: "Foo", ownerFile: "src/foo.ts", line: 10, fanIn: 3 },
        ]),
      });
      expect(clean.status).toBe("clean");
      expect(clean.drifts).toEqual([]);
      expect(clean.reportMarkdown).toContain("## 1. Summary");
      expect(clean.reportMarkdown).not.toContain("## 2. identity-added");

      const fanInCanon = writeCanon(
        fixture,
        "clean-fanin-space.md",
        [
          {
            name: "Foo",
            identity: "src/foo.ts::Foo",
            owner: "src/foo.ts:10",
            fanIn: 3,
            fanInSpace: "value 2, type 1, broad 0",
            label: "single-owner-strong",
          },
        ],
        { fanInSpace: true },
      );
      expect(
        detect({
          canonPath: fanInCanon,
          symbols: makeSymbols([
            { name: "Foo", ownerFile: "src/foo.ts", line: 10, fanIn: 3 },
          ]),
        }).status,
      ).toBe("clean");

      const renderCanon = writeCanon(fixture, "render.md", [
        {
          name: "Foo",
          identity: "src/a.ts::Foo",
          owner: "src/a.ts:10",
          fanIn: 3,
          label: "single-owner-strong",
        },
      ]);
      const ownerChanged = detect({
        canonPath: renderCanon,
        symbols: makeSymbols([
          { name: "Foo", ownerFile: "src/b.ts", line: 4, fanIn: 0 },
        ]),
      });
      expect(ownerChanged.reportMarkdown).toContain("Canon label");
      expect(ownerChanged.reportMarkdown).toContain("Fresh label");
      expect(ownerChanged.reportMarkdown).toContain("owner-changed");

      const kindCanon = writeCanon(fixture, "kind.md", [
        {
          name: "Foo",
          identity: "src/foo.ts::Foo",
          owner: "src/foo.ts:10",
          fanIn: 3,
          label: "single-owner-strong",
        },
      ]);
      const removed = detect({
        canonPath: kindCanon,
        symbols: makeSymbols([]),
      });
      expect(removed.drifts.length).toBeGreaterThan(0);
      expect(removed.drifts.every((drift) => drift.kind === "type-drift")).toBe(
        true,
      );
    }));
});

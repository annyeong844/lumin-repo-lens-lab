import path from "node:path";

import { describe, expect, it } from "vitest";

import { detectNamingDrift } from "../_lib/check-canon-naming.mjs";
import { NAMING_LABEL_SET } from "../_lib/check-canon-utils.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

function withFixture(testFn) {
  const fixture = createTempRepoFixture({ prefix: "fx-vitest-check-naming-" });
  try {
    return testFn(fixture);
  } finally {
    fixture.cleanup();
  }
}

function buildCanonNamingMd({
  fileCohorts = [],
  symbolCohorts = [],
  outliers = null,
}) {
  const lines = ["# Naming canon (fixture)", ""];
  lines.push("## 1. File-naming cohorts", "");
  lines.push(
    "| Cohort (submodule) | Files | DominantConvention | ConsistencyRate | OutliersCount | Status |",
  );
  lines.push(
    "|--------------------|------:|--------------------|----------------:|--------------:|--------|",
  );
  for (const cohort of fileCohorts) {
    lines.push(
      `| \`${cohort.cohort}\` | ${cohort.files} | \`${cohort.convention}\` | ${cohort.rate}% | ${cohort.outliers ?? 0} | ${cohort.label} ✅ |`,
    );
  }
  lines.push("", "## 2. Symbol-naming cohorts", "");
  lines.push(
    "| Cohort (submodule::kind) | Items | DominantConvention | ConsistencyRate | OutliersCount | Status |",
  );
  lines.push(
    "|--------------------------|------:|--------------------|----------------:|--------------:|--------|",
  );
  for (const cohort of symbolCohorts) {
    lines.push(
      `| \`${cohort.cohort}\` | ${cohort.items} | \`${cohort.convention}\` | ${cohort.rate}% | ${cohort.outliers ?? 0} | ${cohort.label} ✅ |`,
    );
  }
  lines.push("");

  if (outliers !== null) {
    lines.push("## 3. Outliers", "");
    lines.push(
      "| Identity | Cohort | Name | ObservedConvention | DominantConvention | Status |",
    );
    lines.push(
      "|----------|--------|------|--------------------|--------------------|--------|",
    );
    for (const outlier of outliers) {
      lines.push(
        `| \`${outlier.identity}\` | \`${outlier.cohort}\` | \`${outlier.name}\` | \`${outlier.observed}\` | \`${outlier.dominant}\` | ${outlier.label} ⚠ |`,
      );
    }
    lines.push("");
  }

  return lines.join("\n");
}

function writeCanon(fixture, relPath, spec) {
  fixture.write(relPath, buildCanonNamingMd(spec));
  return fixture.path(relPath);
}

function makeExtractStub(byFile, failing = new Set()) {
  return (absFile) => {
    if (failing.has(absFile)) {
      throw new Error(`forced-throw on ${absFile}`);
    }
    return byFile.get(absFile) ?? { defs: [], uses: [], reExports: [] };
  };
}

function submoduleOfTopDir(root) {
  return (absFile) => {
    const rel = path.relative(root, absFile).replace(/\\/g, "/");
    const firstSlash = rel.indexOf("/");
    return firstSlash < 0 ? rel : rel.slice(0, firstSlash);
  };
}

function buildScan({
  root,
  files,
  defsByFile = new Map(),
  failing = new Set(),
  lowInfoNames = new Set(),
  lowInfoHelperNames = new Set(),
}) {
  return {
    files,
    root,
    extractFn: makeExtractStub(defsByFile, failing),
    submoduleOf: submoduleOfTopDir(root),
    lowInfoNames,
    lowInfoHelperNames,
  };
}

function detect({ canonPath, scanContext }) {
  return detectNamingDrift({
    canonPath,
    scanContext,
    canonLabelSet: NAMING_LABEL_SET,
  });
}

describe("naming drift engine", () => {
  it("N-1/N-2. missing canon skips and the label set is canonical", () =>
    withFixture((fixture) => {
      const result = detect({
        canonPath: fixture.path("missing.md"),
        scanContext: buildScan({ root: fixture.root, files: [] }),
      });

      expect(result).toMatchObject({
        status: "skipped-missing-canon",
        drifts: [],
        reportMarkdown: null,
      });
      expect(NAMING_LABEL_SET).toEqual(
        new Set([
          "camelCase-dominant",
          "PascalCase-dominant",
          "kebab-case-dominant",
          "snake_case-dominant",
          "UPPER_SNAKE-dominant",
          "mixed-convention",
          "insufficient-evidence",
          "convention-match",
          "convention-outlier",
          "low-info-excluded",
        ]),
      );
    }));

  it("N-3/N-4/N-6. added and removed cohorts preserve file and symbol identity shapes", () =>
    withFixture((fixture) => {
      const addedCanon = writeCanon(fixture, "added.md", {
        fileCohorts: [
          {
            cohort: "src",
            files: 3,
            convention: "kebab-case",
            rate: 100,
            label: "kebab-case-dominant",
          },
        ],
      });
      const added = detect({
        canonPath: addedCanon,
        scanContext: buildScan({
          root: fixture.root,
          files: [
            fixture.path("src/a.ts"),
            fixture.path("src/b.ts"),
            fixture.path("src/c.ts"),
            fixture.path("lib/d.ts"),
            fixture.path("lib/e.ts"),
            fixture.path("lib/f.ts"),
          ],
        }),
      }).drifts.filter((drift) => drift.category === "cohort-added");

      expect(added).toEqual([
        expect.objectContaining({
          identity: "lib",
          family: "added",
        }),
      ]);
      expect(added[0].identity).not.toContain("::");
      expect(added[0].identity).not.toContain("→");

      const removedCanon = writeCanon(fixture, "removed.md", {
        fileCohorts: [
          {
            cohort: "src",
            files: 3,
            convention: "kebab-case",
            rate: 100,
            label: "kebab-case-dominant",
          },
          {
            cohort: "gone",
            files: 2,
            convention: "kebab-case",
            rate: 100,
            label: "kebab-case-dominant",
          },
        ],
        symbolCohorts: [
          {
            cohort: "src::helper-export",
            items: 3,
            convention: "camelCase",
            rate: 100,
            label: "camelCase-dominant",
          },
        ],
      });
      const removed = detect({
        canonPath: removedCanon,
        scanContext: buildScan({
          root: fixture.root,
          files: [
            fixture.path("src/a.ts"),
            fixture.path("src/b.ts"),
            fixture.path("src/c.ts"),
          ],
        }),
      }).drifts.filter((drift) => drift.category === "cohort-removed");

      expect(removed).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ identity: "gone", family: "removed" }),
          expect.objectContaining({
            identity: "src::helper-export",
            family: "removed",
          }),
        ]),
      );
      expect(
        removed
          .filter((drift) => drift.identity.includes("::"))
          .every((drift) =>
            ["type-export", "helper-export", "constant-export"].includes(
              drift.identity.split("::")[1],
            ),
          ),
      ).toBe(true);
    }));

  it("N-5/N-7/N-8. convention shifts, clean runs, and drift kind remain explicit", () =>
    withFixture((fixture) => {
      const shiftedCanon = writeCanon(fixture, "shifted.md", {
        fileCohorts: [
          {
            cohort: "src",
            files: 3,
            convention: "kebab-case",
            rate: 100,
            label: "kebab-case-dominant",
          },
        ],
      });
      const shifted = detect({
        canonPath: shiftedCanon,
        scanContext: buildScan({
          root: fixture.root,
          files: [
            fixture.path("src/alphaBeta.ts"),
            fixture.path("src/gammaDelta.ts"),
            fixture.path("src/epsilonZeta.ts"),
          ],
        }),
      }).drifts;

      expect(shifted).toEqual([
        expect.objectContaining({
          kind: "naming-drift",
          category: "cohort-convention-shifted",
          identity: "src",
          family: "label-changed",
          canon: expect.objectContaining({ dominantConvention: "kebab-case" }),
          fresh: expect.objectContaining({ dominantConvention: "camelCase" }),
        }),
      ]);

      const cleanCanon = writeCanon(fixture, "clean.md", {
        fileCohorts: [],
        symbolCohorts: [],
      });
      const clean = detect({
        canonPath: cleanCanon,
        scanContext: buildScan({ root: fixture.root, files: [] }),
      });

      expect(clean.status).toBe("clean");
      expect(clean.drifts).toEqual([]);
      expect(clean.reportMarkdown).toContain("## 1. Summary");
      expect(clean.reportMarkdown).not.toContain("## 2. cohort-added");
    }));

  it("N-9. outlier records distinguish introduced and resolved file identities", () =>
    withFixture((fixture) => {
      const canonPath = writeCanon(fixture, "outliers.md", {
        fileCohorts: [
          {
            cohort: "src",
            files: 4,
            convention: "kebab-case",
            rate: 75,
            outliers: 1,
            label: "kebab-case-dominant",
          },
        ],
        outliers: [
          {
            identity: "src/OLD.ts",
            cohort: "src",
            name: "OLD.ts",
            observed: "UPPER_SNAKE",
            dominant: "kebab-case",
            label: "convention-outlier",
          },
        ],
      });
      const drifts = detect({
        canonPath,
        scanContext: buildScan({
          root: fixture.root,
          files: [
            fixture.path("src/a.ts"),
            fixture.path("src/b.ts"),
            fixture.path("src/c.ts"),
            fixture.path("src/FOO.ts"),
          ],
        }),
      }).drifts;
      const introduced = drifts.filter(
        (drift) => drift.category === "new-outlier-introduced",
      );
      const resolved = drifts.filter(
        (drift) => drift.category === "outlier-resolved",
      );

      expect(introduced).toEqual([
        expect.objectContaining({
          identity: expect.stringMatching(/FOO\.ts/),
          family: "content-shifted",
        }),
      ]);
      expect(resolved).toEqual([
        expect.objectContaining({
          identity: expect.stringMatching(/OLD\.ts/),
          family: "content-shifted",
        }),
      ]);
      expect(
        [...introduced, ...resolved].every(
          (drift) => !drift.identity.includes("::"),
        ),
      ).toBe(true);
    }));

  it("N-10. extractor throws promote to parse-error instead of partial drift", () =>
    withFixture((fixture) => {
      const canonPath = writeCanon(fixture, "extractor-throw.md", {
        fileCohorts: [
          {
            cohort: "src",
            files: 3,
            convention: "kebab-case",
            rate: 100,
            label: "kebab-case-dominant",
          },
        ],
      });
      const badFile = fixture.path("src/broken.ts");
      const result = detect({
        canonPath,
        scanContext: buildScan({
          root: fixture.root,
          files: [
            fixture.path("src/foo.ts"),
            fixture.path("src/bar.ts"),
            badFile,
          ],
          failing: new Set([badFile]),
        }),
      });

      expect(result).toMatchObject({
        status: "parse-error",
        drifts: [],
        reportMarkdown: null,
      });
      expect(result.diagnostics).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            kind: "parse-error",
            target: expect.stringMatching(/broken\.ts/),
          }),
        ]),
      );
    }));

  it("N-11. P3 display dash and P5 null dominant convention remain equivalent", () =>
    withFixture((fixture) => {
      const canonPath = writeCanon(fixture, "dash-null.md", {
        fileCohorts: [
          {
            cohort: "src",
            files: 5,
            convention: "—",
            rate: 40,
            label: "mixed-convention",
          },
        ],
      });
      const result = detect({
        canonPath,
        scanContext: buildScan({
          root: fixture.root,
          files: [
            fixture.path("src/fooBar.ts"),
            fixture.path("src/bazQux.ts"),
            fixture.path("src/FooBar.ts"),
            fixture.path("src/BazQux.ts"),
            fixture.path("src/foo-bar.ts"),
          ],
        }),
      });

      expect(result.status).toBe("clean");
      expect(result.drifts).toEqual([]);
    }));
});

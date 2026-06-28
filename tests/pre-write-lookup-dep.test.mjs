import { describe, expect, it } from "vitest";

import {
  isWatchForEligible,
  lookupDependency,
  packageRoot,
} from "../_lib/pre-write-lookup-dep.mjs";

function buildPkg({
  dependencies = {},
  devDependencies = {},
  peerDependencies = {},
} = {}) {
  return { dependencies, devDependencies, peerDependencies };
}

function buildSymbols({ dependencyImportConsumers = [] } = {}) {
  return {
    meta: { supports: { dependencyImportConsumers: true } },
    uses: {
      resolvedInternal: 0,
      external: dependencyImportConsumers.length,
      unresolvedInternal: 0,
      mdxConsumers: 0,
      unresolvedInternalRatio: 0,
    },
    dependencyImportConsumers,
  };
}

function buildLegacySymbols({ uses = [] } = {}) {
  return { uses };
}

describe("pre-write dependency lookup package roots", () => {
  it("normalizes package roots while rejecting relative, absolute, and malformed specifiers", () => {
    expect(packageRoot("dayjs")).toBe("dayjs");
    expect(packageRoot("dayjs/plugin/utc")).toBe("dayjs");
    expect(packageRoot("@scope/pkg")).toBe("@scope/pkg");
    expect(packageRoot("@scope/pkg/sub/path")).toBe("@scope/pkg");
    expect(packageRoot("./relative")).toBeNull();
    expect(packageRoot("../up/mod")).toBeNull();
    expect(packageRoot("/abs/path")).toBeNull();
    expect(packageRoot("@malformed")).toBeNull();
    expect(packageRoot("")).toBeNull();
    expect(packageRoot(null)).toBeNull();
  });
});

describe("pre-write dependency lookup availability labels", () => {
  it("reports declared dependencies with grounded observed import consumers", () => {
    const pkg = buildPkg({ dependencies: { dayjs: "1.0.0" } });
    const symbols = buildSymbols({
      dependencyImportConsumers: [
        { file: "src/a.ts", fromSpec: "dayjs", kind: "import" },
        { file: "src/b.ts", fromSpec: "dayjs/plugin/utc", kind: "import" },
      ],
    });

    const result = lookupDependency("dayjs", { packageJson: pkg, symbols });

    expect(result).toMatchObject({
      kind: "dependency",
      depName: "dayjs",
      result: "DEPENDENCY_AVAILABLE",
      declaredIn: "dependencies",
    });
    expect(result.existingImports.examples).toHaveLength(2);
    expect(result.existingImports).toMatchObject({
      observedImportCount: 2,
      countConfidence: "grounded",
    });
    expect(result.citations.join(" ")).toContain(
      "symbols.json.dependencyImportConsumers",
    );
    expect(result.citations.join(" ")).toContain("package.json");
  });

  it("keeps declared-with-no-observed-imports distinct from unavailable import graph", () => {
    const pkg = buildPkg({ devDependencies: { eslint: "9.0.0" } });
    const noConsumers = lookupDependency("eslint", {
      packageJson: pkg,
      symbols: buildSymbols(),
    });

    expect(noConsumers.result).toBe("DEPENDENCY_AVAILABLE_NO_OBSERVED_IMPORTS");
    expect(noConsumers.declaredIn).toBe("devDependencies");
    expect(noConsumers.existingImports.observedImportCount).toBe(0);
    expect(noConsumers.citations.join(" ")).toContain("확인 불가");
    expect(noConsumers.citations.join(" ")).toMatch(/import graph/i);
    expect(noConsumers.citations.join(" ")).not.toMatch(/\bunused\b/i);
    expect(noConsumers.citations.join(" ")).not.toMatch(/\bcleanup\b/i);

    const missingSymbols = lookupDependency("eslint", {
      packageJson: buildPkg({ dependencies: { eslint: "9.0.0" } }),
      symbols: null,
    });
    expect(missingSymbols.result).toBe(
      "DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE",
    );
    expect(missingSymbols.existingImports).toMatchObject({
      observedImportCount: null,
      countConfidence: "unavailable",
    });
    expect(missingSymbols.citations.join(" ")).toContain("확인 불가");
    expect(missingSymbols.citations.join(" ")).toContain("symbols.json absent");

    const malformedSymbols = lookupDependency("eslint", {
      packageJson: buildPkg({ dependencies: { eslint: "9.0.0" } }),
      symbols: {},
    });
    expect(malformedSymbols.result).toBe(
      "DEPENDENCY_AVAILABLE_IMPORT_GRAPH_UNAVAILABLE",
    );
    expect(malformedSymbols.existingImports.countConfidence).toBe(
      "unavailable",
    );
  });

  it("handles peer, absent, scoped, subpath, and relative consumer cases", () => {
    const peer = lookupDependency("react", {
      packageJson: buildPkg({ peerDependencies: { react: ">=18" } }),
      symbols: buildSymbols({
        dependencyImportConsumers: [
          { file: "src/app.tsx", fromSpec: "react", kind: "import" },
        ],
      }),
    });
    expect(peer).toMatchObject({
      result: "DEPENDENCY_AVAILABLE",
      declaredIn: "peerDependencies",
    });

    const absent = lookupDependency("axios", {
      packageJson: buildPkg({ dependencies: { dayjs: "1.0.0" } }),
      symbols: buildSymbols(),
    });
    expect(absent).toMatchObject({
      result: "NEW_PACKAGE",
      declaredIn: null,
    });
    expect(absent.citations.join(" ")).toContain("package.json");

    const scoped = lookupDependency("@anthropic/sdk", {
      packageJson: buildPkg({ dependencies: { "@anthropic/sdk": "0.1.0" } }),
      symbols: buildSymbols({
        dependencyImportConsumers: [
          {
            file: "src/ai.ts",
            fromSpec: "@anthropic/sdk/client",
            kind: "import",
          },
        ],
      }),
    });
    expect(scoped.result).toBe("DEPENDENCY_AVAILABLE");
    expect(scoped.existingImports.examples[0].fromSpec).toBe(
      "@anthropic/sdk/client",
    );

    const subpath = lookupDependency("dayjs/plugin/utc", {
      packageJson: buildPkg({ dependencies: { dayjs: "1.0.0" } }),
      symbols: buildSymbols({
        dependencyImportConsumers: [
          { file: "src/a.ts", fromSpec: "dayjs/plugin/utc", kind: "import" },
        ],
      }),
    });
    expect(subpath.result).toBe("DEPENDENCY_AVAILABLE");

    const relativeExcluded = lookupDependency("dayjs", {
      packageJson: buildPkg({ dependencies: { dayjs: "1.0.0" } }),
      symbols: buildSymbols({
        dependencyImportConsumers: [
          { file: "src/a.ts", fromSpec: "./dayjs", kind: "import" },
          { file: "src/b.ts", fromSpec: "dayjs", kind: "import" },
        ],
      }),
    });
    expect(relativeExcluded.existingImports.observedImportCount).toBe(1);
  });

  it("caps examples while preserving true grounded counts and watch-for eligibility", () => {
    const uses = Array.from({ length: 12 }, (_, index) => ({
      file: `src/${index}.ts`,
      fromSpec: "lodash",
      kind: "import",
    }));
    const result = lookupDependency("lodash", {
      packageJson: buildPkg({ dependencies: { lodash: "4" } }),
      symbols: buildSymbols({ dependencyImportConsumers: uses }),
    });

    expect(result.existingImports.examples).toHaveLength(5);
    expect(result.existingImports.observedImportCount).toBe(12);
    expect(result.existingImports.countConfidence).toBe("grounded");
    expect(isWatchForEligible(result.existingImports)).toBe(true);
    expect(
      isWatchForEligible({
        observedImportCount: 20,
        countConfidence: "sample-only",
        examples: uses.slice(0, 5),
      }),
    ).toBe(false);
    expect(
      isWatchForEligible({
        observedImportCount: 1,
        countConfidence: "grounded",
        examples: uses.slice(0, 1),
      }),
    ).toBe(false);
  });

  it("keeps legacy symbols.uses[] consumer evidence supported", () => {
    const result = lookupDependency("dayjs", {
      packageJson: buildPkg({ dependencies: { dayjs: "1.0.0" } }),
      symbols: buildLegacySymbols({
        uses: [{ file: "src/legacy.ts", fromSpec: "dayjs", kind: "import" }],
      }),
    });

    expect(result.result).toBe("DEPENDENCY_AVAILABLE");
    expect(result.existingImports.observedImportCount).toBe(1);
    expect(result.citations.join(" ")).toContain("symbols.json.uses");
  });
});

import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import {
  parsePrismaEnums,
  schemaUsesPrismaEnumGenerator,
} from "../_lib/generated-virtual-surface.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

function runSymbolGraph(fixture) {
  execFileSync(
    process.execPath,
    [
      path.join(ROOT, "build-symbol-graph.mjs"),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
    ],
    { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] },
  );
}

function createPrismaVirtualSurfaceFixture({ schema, importedName = "BookingStatus" }) {
  const fixture = createTempRepoFixture({
    prefix: "fx-vitest-generated-virtual-prisma-",
    packageJson: {
      name: "root",
      type: "module",
      workspaces: ["apps/*", "packages/*"],
    },
  });

  fixture.writeJson("apps/web/package.json", {
    name: "web",
    type: "module",
  });
  fixture.writeJson("packages/prisma/package.json", {
    name: "@scope/prisma",
    type: "module",
    main: "index.ts",
    bin: { "prisma-enum-generator": "./run-enum-generator.js" },
    scripts: { generate: "prisma generate" },
    dependencies: { "@prisma/client": "1.0.0" },
  });
  fixture.write("packages/prisma/index.ts", "export const prismaRoot = 1;\n");
  fixture.write("packages/prisma/schema.prisma", schema);
  fixture.write(
    "apps/web/src/consumer.ts",
    `import { ${importedName} } from '@scope/prisma/enums';\n` +
      `export const status = ${importedName}.ACCEPTED;\n`,
  );

  return fixture;
}

describe("Prisma enum virtual surface parser", () => {
  it("extracts enum names and values without value attributes", () => {
    const schema =
      'generator enums {\n' +
      '  provider = "prisma-enum-generator"\n' +
      '}\n\n' +
      'enum BookingStatus {\n' +
      '  /// accepted booking\n' +
      '  ACCEPTED @map("accepted")\n' +
      '  CANCELLED\n' +
      '}\n';

    const enums = parsePrismaEnums(schema);

    expect(schemaUsesPrismaEnumGenerator(schema)).toBe(true);
    expect(enums).toHaveLength(1);
    expect(enums[0]).toMatchObject({
      name: "BookingStatus",
      values: ["ACCEPTED", "CANCELLED"],
    });
  });
});

describe("generated virtual Prisma enum surfaces", () => {
  it("resolves supported enum imports as partial generated virtual surfaces", () => {
    const fixture = createPrismaVirtualSurfaceFixture({
      schema:
        'generator enums {\n' +
        '  provider = "prisma-enum-generator"\n' +
        '}\n\n' +
        'enum BookingStatus {\n' +
        '  ACCEPTED @map("accepted")\n' +
        '  CANCELLED\n' +
        '}\n',
    });

    try {
      runSymbolGraph(fixture);

      const symbols = fixture.readJson("symbols.json", { from: "output" });
      const unresolved = symbols.unresolvedInternalSpecifierRecords ?? [];
      const surface = (symbols.generatedVirtualSurfaces ?? []).find(
        (item) =>
          item.matchedPackage === "@scope/prisma" &&
          item.targetSubpath === "enums",
      );
      const consumer = (symbols.generatedVirtualImportConsumers ?? []).find(
        (item) =>
          item.specifier === "@scope/prisma/enums" &&
          item.name === "BookingStatus",
      );

      expect(symbols.uses?.unresolvedInternal).toBe(0);
      expect(unresolved.some((item) => item.specifier === "@scope/prisma/enums")).toBe(
        false,
      );
      expect(symbols.uses?.resolvedGeneratedVirtual).toBe(1);
      expect(symbols.meta?.supports?.generatedVirtualSurfaces).toBe(true);
      expect(surface).toMatchObject({
        source: "generated-virtual",
        virtual: true,
        runtimeEquivalence: false,
        surfaceCompleteness: "partial",
      });
      expect(surface?.exports).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            name: "BookingStatus",
            spaces: expect.arrayContaining(["value", "type"]),
          }),
        ]),
      );
      expect(consumer).toMatchObject({
        surfaceId: surface?.id,
        consumerFile: "apps/web/src/consumer.ts",
      });
    } finally {
      fixture.cleanup();
    }
  });

  it("does not create a virtual surface without schema generator evidence", () => {
    const fixture = createPrismaVirtualSurfaceFixture({
      schema:
        'generator client {\n' +
        '  provider = "prisma-client-js"\n' +
        '}\n\n' +
        'enum BookingStatus {\n' +
        '  ACCEPTED\n' +
        '}\n',
    });

    try {
      runSymbolGraph(fixture);

      const symbols = fixture.readJson("symbols.json", { from: "output" });

      expect(symbols.uses?.unresolvedInternal).toBe(1);
      expect(symbols.generatedVirtualSurfaces ?? []).toEqual([]);
      expect(symbols.generatedVirtualImportConsumers ?? []).toEqual([]);
    } finally {
      fixture.cleanup();
    }
  });

  it("does not resolve enum imports absent from the schema surface", () => {
    const fixture = createPrismaVirtualSurfaceFixture({
      importedName: "MissingEnum",
      schema:
        'generator enums {\n' +
        '  provider = "prisma-enum-generator"\n' +
        '}\n\n' +
        'enum BookingStatus {\n' +
        '  ACCEPTED\n' +
        '}\n',
    });

    try {
      runSymbolGraph(fixture);

      const symbols = fixture.readJson("symbols.json", { from: "output" });

      expect(symbols.uses?.unresolvedInternal).toBe(1);
      expect(symbols.generatedVirtualSurfaces ?? []).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            matchedPackage: "@scope/prisma",
            exports: expect.arrayContaining([
              expect.objectContaining({ name: "BookingStatus" }),
            ]),
          }),
        ]),
      );
      expect(
        (symbols.generatedVirtualImportConsumers ?? []).some(
          (item) => item.name === "MissingEnum",
        ),
      ).toBe(false);
    } finally {
      fixture.cleanup();
    }
  });
});

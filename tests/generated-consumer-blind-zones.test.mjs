import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");

function runSymbolGraph(fixture, args = []) {
  execFileSync(
    process.execPath,
    [
      path.join(ROOT, "build-symbol-graph.mjs"),
      "--root",
      fixture.root,
      "--output",
      fixture.output,
      ...args,
    ],
    { cwd: ROOT, stdio: ["ignore", "pipe", "pipe"] },
  );
}

function createGeneratedConsumerFixture() {
  const fixture = createTempRepoFixture({
    prefix: "fx-vitest-generated-consumer-zone-",
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
  fixture.write(
    "apps/web/src/consumer.ts",
    "import { BookingStatus } from '@scope/prisma/enums';\n" +
      "export const status = BookingStatus.ACCEPTED;\n",
  );

  return fixture;
}

describe("generated consumer blind-zone symbol artifact", () => {
  it("records missing generated workspace subpath consumers as blind-zone inventory", () => {
    const fixture = createGeneratedConsumerFixture();

    try {
      runSymbolGraph(fixture);

      const symbols = fixture.readJson("symbols.json", { from: "output" });
      const zone = symbols.generatedConsumerBlindZones?.[0];

      expect(symbols.meta?.supports?.generatedConsumerBlindZones).toBe(true);
      expect(zone).toMatchObject({
        reason: "generated-consumer-blind-zone",
        sourceReason: "workspace-generated-artifact-missing",
        specifier: "@scope/prisma/enums",
        consumerFile: "apps/web/src/consumer.ts",
        matchedPackage: "@scope/prisma",
        targetSubpath: "enums",
        status: "missing",
        scopePackageRoot: "packages/prisma",
      });
    } finally {
      fixture.cleanup();
    }
  });

  it("forwards prepared generated artifact mode with unknown stale provenance", () => {
    const fixture = createGeneratedConsumerFixture();

    try {
      runSymbolGraph(fixture, ["--generated-artifacts", "prepared"]);

      const symbols = fixture.readJson("symbols.json", { from: "output" });
      const zone = symbols.generatedConsumerBlindZones?.[0];

      expect(zone).toMatchObject({
        status: "missing",
        mode: "prepared",
        staleStatus: "unknown",
        staleReason: "generator-input-hash-not-recorded",
      });
    } finally {
      fixture.cleanup();
    }
  });
});

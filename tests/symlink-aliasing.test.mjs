import { mkdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { afterAll, beforeAll, describe, expect, it } from "vitest";

import { buildAliasMap } from "../_lib/alias-map.mjs";
import { detectRepoMode } from "../_lib/repo-mode.mjs";
import { makeResolver } from "../_lib/resolver-core.mjs";

function probeSymlinkAvailability() {
  const probeDir = path.join(
    tmpdir(),
    `fx-symlink-probe-${process.pid}-${Date.now()}`,
  );
  mkdirSync(probeDir, { recursive: true });
  try {
    writeFileSync(path.join(probeDir, "target"), "");
    symlinkSync(path.join(probeDir, "target"), path.join(probeDir, "link"));
    return true;
  } catch (error) {
    if (error.code === "EPERM" || error.code === "EACCES") {
      return false;
    }
    throw error;
  } finally {
    rmSync(probeDir, { recursive: true, force: true });
  }
}

const symlinksAvailable = probeSymlinkAvailability();
const maybeIt = symlinksAvailable ? it : it.skip;

describe("symlink aliasing resolver realpath canonicalization", () => {
  let root;
  let resolve;
  let appTs;
  let consumerTs;

  beforeAll(() => {
    if (!symlinksAvailable) return;

    root = path.join(
      tmpdir(),
      `fx-symlink-aliasing-${process.pid}-${Date.now()}`,
    );
    mkdirSync(path.join(root, "src"), { recursive: true });
    mkdirSync(path.join(root, "vendored"), { recursive: true });
    writeFileSync(
      path.join(root, "package.json"),
      '{"name":"fx-symlink","type":"module"}',
    );

    writeFileSync(
      path.join(root, "vendored/lib.ts"),
      "export const vendoredValue = 42;\n",
    );
    symlinkSync("../vendored/lib.ts", path.join(root, "src/lib.ts"));
    writeFileSync(
      path.join(root, "src/app.ts"),
      "import { vendoredValue } from './lib.js';\n" +
        "export const used = vendoredValue;\n",
    );

    mkdirSync(path.join(root, "shared/core"), { recursive: true });
    writeFileSync(
      path.join(root, "shared/core/index.ts"),
      "export const sharedCore = 1;\n",
    );
    symlinkSync("../shared/core", path.join(root, "src/core-link"));
    writeFileSync(
      path.join(root, "src/consumer.ts"),
      "import { sharedCore } from './core-link';\n" +
        "export const c = sharedCore;\n",
    );

    const mode = detectRepoMode(root);
    resolve = makeResolver(root, buildAliasMap(root, mode));
    appTs = path.join(root, "src/app.ts");
    consumerTs = path.join(root, "src/consumer.ts");
  });

  afterAll(() => {
    if (root) rmSync(root, { recursive: true, force: true });
  });

  maybeIt("T1. file-symlink resolved to realpath (not src/lib.ts)", () => {
    expect(resolve(appTs, "./lib.js")).toBe(path.join(root, "vendored/lib.ts"));
  });

  maybeIt("T2. extensionless symlink spec resolves to realpath", () => {
    expect(resolve(appTs, "./lib")).toBe(path.join(root, "vendored/lib.ts"));
  });

  maybeIt("T3. dir-symlink + /index.ts lookup returns realpath", () => {
    expect(resolve(consumerTs, "./core-link")).toBe(
      path.join(root, "shared/core/index.ts"),
    );
  });

  maybeIt("T4. null passes through unchanged", () => {
    expect(resolve(appTs, "")).toBeNull();
  });

  maybeIt("T5. EXTERNAL passes through unchanged", () => {
    expect(resolve(appTs, "some-npm-package")).toBe("EXTERNAL");
  });

  maybeIt("T6. non-symlinked relative import unchanged", () => {
    writeFileSync(path.join(root, "src/normal.ts"), "export const n = 1;\n");
    expect(resolve(appTs, "./normal")).toBe(path.join(root, "src/normal.ts"));
  });
});

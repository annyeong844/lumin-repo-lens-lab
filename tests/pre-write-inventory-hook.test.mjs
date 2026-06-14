import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

const ROOT = path.resolve(import.meta.dirname, "..");
const NODE = process.execPath;
const PREWRITE = path.join(ROOT, "pre-write.mjs");

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function buildFixture(root) {
  write(root, "package.json", JSON.stringify({ name: "fx", type: "module" }));
  write(root, "src/a.ts", "export const foo = (x as any).y;\n");
}

function writeIntent(output) {
  const intent = {
    names: ["foo"],
    shapes: [],
    files: [],
    dependencies: [],
    plannedTypeEscapes: [],
  };
  const intentPath = path.join(output, "intent.json");
  writeFileSync(intentPath, JSON.stringify(intent));
  return intentPath;
}

function runPreWrite(root, output, intentPath, extraArgs = []) {
  execFileSync(
    NODE,
    [
      PREWRITE,
      "--root",
      root,
      "--output",
      output,
      "--intent",
      intentPath,
      ...extraArgs,
    ],
    { stdio: ["ignore", "pipe", "pipe"] },
  );
}

function invocationFiles(output) {
  return readdirSync(output).filter(
    (name) =>
      name.startsWith("pre-write-advisory.") && !name.endsWith(".latest.json"),
  );
}

function inventorySnapshotFiles(output) {
  return readdirSync(output).filter((name) =>
    name.startsWith("any-inventory.pre."),
  );
}

function readJson(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

function withPreWriteFixture(fn) {
  const root = mkdtempSync(path.join(tmpdir(), "pw-hook-"));
  const output = mkdtempSync(path.join(tmpdir(), "pw-hook-out-"));

  try {
    buildFixture(root);
    const intentPath = writeIntent(output);
    fn({ root, output, intentPath });
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(output, { recursive: true, force: true });
  }
}

describe("pre-write inventory hook artifact availability", () => {
  it("writes an invocation-specific snapshot and stamps both advisory artifacts", () => {
    withPreWriteFixture(({ root, output, intentPath }) => {
      runPreWrite(root, output, intentPath);

      const snapshotFiles = inventorySnapshotFiles(output);
      expect(snapshotFiles).toHaveLength(1);

      const snapshot = readJson(path.join(output, snapshotFiles[0]));
      expect(snapshot.typeEscapes).toEqual(expect.any(Array));

      const latest = readJson(
        path.join(output, "pre-write-advisory.latest.json"),
      );
      const invocationAdvisories = invocationFiles(output);
      expect(invocationAdvisories).toHaveLength(1);

      const invocation = readJson(path.join(output, invocationAdvisories[0]));
      expect(latest.preWrite?.anyInventoryPath).toMatch(
        /^any-inventory\.pre\./,
      );
      expect(invocation.preWrite?.anyInventoryPath).toMatch(
        /^any-inventory\.pre\./,
      );
      expect(invocation.preWrite.anyInventoryPath).toBe(
        latest.preWrite.anyInventoryPath,
      );
      expect(
        existsSync(path.join(output, latest.preWrite.anyInventoryPath)),
      ).toBe(true);
    });
  });

  it("leaves anyInventoryPath absent when fresh audit is disabled", () => {
    withPreWriteFixture(({ root, output, intentPath }) => {
      runPreWrite(root, output, intentPath, ["--no-fresh-audit"]);

      expect(inventorySnapshotFiles(output)).toHaveLength(0);

      const latest = readJson(
        path.join(output, "pre-write-advisory.latest.json"),
      );
      const invocationAdvisories = invocationFiles(output);
      expect(invocationAdvisories).toHaveLength(1);
      const invocation = readJson(path.join(output, invocationAdvisories[0]));

      expect(latest.preWrite ?? {}).not.toHaveProperty("anyInventoryPath");
      expect(invocation.preWrite ?? {}).not.toHaveProperty("anyInventoryPath");
    });
  });

  it("preserves existing P1 advisory fields while adding hook metadata", () => {
    withPreWriteFixture(({ root, output, intentPath }) => {
      runPreWrite(root, output, intentPath);

      const advisory = readJson(
        path.join(output, "pre-write-advisory.latest.json"),
      );

      expect(advisory.invocationId).toEqual(expect.any(String));
      expect(advisory.invocationId.length).toBeGreaterThan(0);
      expect(advisory.intentHash).toMatch(/^[a-f0-9]{64}$/);
      expect(advisory.lookups).toEqual(expect.any(Array));
      expect(advisory.drift).toEqual(expect.any(Array));
      expect(advisory.capabilities).toBeDefined();
      expect(advisory.failures).toEqual(expect.any(Array));
    });
  });

  it("records type-escape capability metadata in the snapshot", () => {
    withPreWriteFixture(({ root, output, intentPath }) => {
      runPreWrite(root, output, intentPath);

      const [snapshotName] = inventorySnapshotFiles(output);
      const snapshot = readJson(path.join(output, snapshotName));

      expect(snapshot.meta?.supports?.typeEscapes).toBe(true);
      expect(snapshot.meta?.complete).toBe(true);
      expect(snapshot.meta?.supports?.escapeKinds).toHaveLength(11);
    });
  });

  it("does not clobber an existing shared any-inventory.json", () => {
    withPreWriteFixture(({ root, output, intentPath }) => {
      const sentinelPath = path.join(output, "any-inventory.json");
      const sentinel = `${JSON.stringify({ sentinel: true })}\n`;
      writeFileSync(sentinelPath, sentinel);

      runPreWrite(root, output, intentPath);

      const latest = readJson(
        path.join(output, "pre-write-advisory.latest.json"),
      );
      const snapshotPath = path.join(output, latest.preWrite.anyInventoryPath);

      expect(readFileSync(sentinelPath, "utf8")).toBe(sentinel);
      expect(existsSync(snapshotPath)).toBe(true);
      expect(latest.preWrite.anyInventoryPath).toMatch(/^any-inventory\.pre\./);
    });
  });
});

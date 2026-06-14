import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { extractDefinitionsAndUses } from "../_lib/extract-ts.mjs";

function extractSource(source) {
  const dir = mkdtempSync(
    path.join(os.tmpdir(), "lrl-vitest-cjs-export-surface-"),
  );
  const file = path.join(dir, "exporter.cjs");
  writeFileSync(file, source);
  try {
    return extractDefinitionsAndUses(file);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

function hasExact(surface, name, kind) {
  return (surface?.exact ?? []).some(
    (entry) =>
      entry.name === name && (kind === undefined || entry.kind === kind),
  );
}

function hasOpaque(surface, kind) {
  return (surface?.opaque ?? []).some((entry) => entry.kind === kind);
}

describe("CJS export surface extraction", () => {
  function surface() {
    const info = extractSource(
      [
        "exports.foo = 1;",
        "module.exports.bar = 2;",
        'exports["quoted"] = 3;',
        "module.exports = { baz: 4, renamed: localValue };",
        "exports[dynamicName] = 5;",
        "module.exports = makeExports();",
        "",
      ].join("\n"),
    );
    return info.cjsExportSurface;
  }

  it("CJSX1. exact exports.foo assignment is recorded", () => {
    expect(hasExact(surface(), "foo", "exports-member")).toBe(true);
  });

  it("CJSX1b. exact module.exports.bar assignment is recorded", () => {
    expect(hasExact(surface(), "bar", "module-exports-member")).toBe(true);
  });

  it("CJSX1c. exact quoted exports member assignment is recorded", () => {
    expect(hasExact(surface(), "quoted", "exports-member")).toBe(true);
  });

  it("CJSX1d. exact module.exports object properties are recorded", () => {
    const cjsSurface = surface();

    expect(hasExact(cjsSurface, "baz", "module-exports-object")).toBe(true);
    expect(hasExact(cjsSurface, "renamed", "module-exports-object")).toBe(true);
  });

  it("CJSX1e. computed export name is recorded as opaque", () => {
    expect(hasOpaque(surface(), "computed-export-name")).toBe(true);
  });

  it("CJSX1f. non-object module.exports assignment is recorded as opaque", () => {
    expect(hasOpaque(surface(), "module-exports-assignment")).toBe(true);
  });
});

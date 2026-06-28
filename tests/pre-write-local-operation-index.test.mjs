import { execFileSync } from "node:child_process";
import {
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { lookupName } from "../_lib/pre-write-lookup-name.mjs";

const REPO = process.cwd();

function runNode(args, cwd = REPO) {
  return execFileSync(process.execPath, args, {
    cwd,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
}

function writeRepositoryFixture(root) {
  mkdirSync(path.join(root, "src"), { recursive: true });
  writeFileSync(
    path.join(root, "package.json"),
    JSON.stringify({ private: true, type: "module" }, null, 2),
  );
  writeFileSync(
    path.join(root, "src", "repository.ts"),
    [
      "export function createRepository(db: any) {",
      "  function getWorld(id: string) {",
      "    return db.world.find(id);",
      "  }",
      "",
      "  const listLibraryDocs = async (worldId: string) => {",
      "    return db.docs.list(worldId);",
      "  };",
      "",
      "  function deleteWorld(id: string) {",
      "    return db.world.delete(id);",
      "  }",
      "",
      "  function normalizeInput(value: string) {",
      "    return value.trim();",
      "  }",
      "",
      "  return { getWorld, listLibraryDocs, deleteWorld, normalizeInput };",
      "}",
      "",
    ].join("\n"),
  );
}

function readSymbolsAfterBuild(root) {
  const out = path.join(root, ".audit");
  runNode([
    "build-symbol-graph.mjs",
    "--root",
    root,
    "--output",
    out,
    "--no-incremental",
  ]);
  return JSON.parse(readFileSync(path.join(out, "symbols.json"), "utf8"));
}

function withRepositoryFixture(callback) {
  const root = mkdtempSync(path.join(os.tmpdir(), "lrl-local-operation-index-"));
  try {
    writeRepositoryFixture(root);
    const symbols = readSymbolsAfterBuild(root);
    return callback({ root, symbols });
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function entriesByName(symbols) {
  const entries =
    symbols.preWriteLocalOperationIndex?.byOwnerFile?.["src/repository.ts"] ??
    [];
  return {
    entries,
    byName: Object.fromEntries(entries.map((entry) => [entry.name, entry])),
  };
}

function searchWorldLookup(symbols) {
  return lookupName("searchWorld", {
    symbols,
    canonicalClaims: [],
    intentDeclaration: {
      name: "searchWorld",
      kind: "function",
      why: "search world data from the repository",
      ownerFile: "src/repository.ts",
    },
  });
}

describe("pre-write local operation index", () => {
  it("advertises nested local operation index support", () => {
    withRepositoryFixture(({ symbols }) => {
      expect(symbols.meta?.supports?.nestedLocalOperationIndex).toBe(true);
      expect(symbols.preWriteLocalOperationIndex).toMatchObject({
        schemaVersion: "pre-write-local-operations.v1",
        status: "complete",
      });
    });
  });

  it("indexes read/query nested operations with container identity", () => {
    withRepositoryFixture(({ symbols }) => {
      const { byName } = entriesByName(symbols);

      expect(byName.getWorld).toMatchObject({
        identity: "src/repository.ts::createRepository#getWorld",
        containerName: "createRepository",
        containerKind: "function-declaration",
        matchedField: "preWriteLocalOperationIndex",
        operationFamily: "read-query",
        eligibleForDeadExportRanking: false,
        eligibleForSafeFix: false,
      });
      expect(byName.getWorld.domainTokens).toContain("world");
    });
  });

  it("indexes const arrow read/query operations with a closed container kind", () => {
    withRepositoryFixture(({ symbols }) => {
      const { byName } = entriesByName(symbols);

      expect(byName.listLibraryDocs).toMatchObject({
        identity: "src/repository.ts::createRepository#listLibraryDocs",
        containerKind: "function-declaration",
      });
      expect(byName.listLibraryDocs.domainTokens).toEqual(
        expect.arrayContaining(["library", "docs"]),
      );
    });
  });

  it("excludes mutation and generic helpers from the v1 local operation surface", () => {
    withRepositoryFixture(({ symbols }) => {
      const { byName } = entriesByName(symbols);

      expect(byName.deleteWorld).toBeUndefined();
      expect(byName.normalizeInput).toBeUndefined();
    });
  });

  it("does not contaminate export defIndex or classMethodIndex", () => {
    withRepositoryFixture(({ symbols }) => {
      expect(
        symbols.defIndex?.["src/repository.ts"]?.createRepository,
      ).toBeTruthy();
      expect(symbols.defIndex?.["src/repository.ts"]?.getWorld).toBeUndefined();
      expect(
        symbols.defIndex?.["src/repository.ts"]?.listLibraryDocs,
      ).toBeUndefined();
      expect(
        symbols.classMethodIndex?.["src/repository.ts"]?.getWorld,
      ).toBeUndefined();
    });
  });

  it("does not leak local operations into formal lookup lanes", () => {
    withRepositoryFixture(({ symbols }) => {
      const lookup = searchWorldLookup(symbols);

      expect(lookup.nearNames).not.toContainEqual(
        expect.objectContaining({ name: "getWorld" }),
      );
      expect(lookup.semanticHints).not.toContainEqual(
        expect.objectContaining({ name: "getWorld" }),
      );
    });
  });

  it("surfaces local operations as a separate review-evidence policy", () => {
    withRepositoryFixture(({ symbols }) => {
      const localPolicy = searchWorldLookup(symbols).localOperationSiblingPolicy;
      const localPromotion = localPolicy?.promoted?.find(
        (entry) => entry.name === "getWorld",
      );

      expect(localPolicy).toMatchObject({
        policyId: "prewrite-local-operation-sibling",
        policyVersion: "prewrite-local-operation-sibling-v1",
      });
      expect(localPolicy.evaluatedCandidateCount).toBeGreaterThanOrEqual(1);
      expect(localPromotion).toMatchObject({
        matchedField: "preWriteLocalOperationIndex",
        surfaceKind: "nested-local-operation",
        containerName: "createRepository",
        operationFamily: "read-query",
        eligibleForDeadExportRanking: false,
        eligibleForSafeFix: false,
      });
      expect(localPromotion.sharedDomainTokens).toContain("world");
      expect(localPromotion.supportingReasons).toContain(
        "local-operation-same-file-domain-overlap",
      );
      expect(localPromotion.locality?.sameFile).toBe(true);
    });
  });

  it("does not feed the service-operation cue policy", () => {
    withRepositoryFixture(({ symbols }) => {
      const servicePolicy = searchWorldLookup(symbols)
        .serviceOperationSiblingPolicy;

      expect(servicePolicy.promoted).not.toContainEqual(
        expect.objectContaining({ name: "getWorld" }),
      );
      expect(servicePolicy.muted).not.toContainEqual(
        expect.objectContaining({ name: "getWorld" }),
      );
    });
  });
});

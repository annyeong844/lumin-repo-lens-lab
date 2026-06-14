import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

import { afterAll, describe, expect, it } from "vitest";

const DIR = path.resolve(import.meta.dirname, "..");
const CLI = path.join(DIR, "check-canon.mjs");
const FIXTURE_ROOT = path.join(DIR, "tests", "fixtures");
const cleanup = [];
const CLI_TEST_TIMEOUT_MS = 30000;

afterAll(() => {
  for (const directory of cleanup)
    rmSync(directory, { recursive: true, force: true });
});

function sha256(filePath) {
  return createHash("sha256").update(readFileSync(filePath)).digest("hex");
}

function runCli(args, { cwd } = {}) {
  const result = spawnSync(process.execPath, [CLI, ...args], {
    cwd: cwd ?? DIR,
    encoding: "utf8",
  });
  return {
    exit: result.status ?? -1,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

function runFixture({ prefix, srcName, source, canonFile, mdFile }) {
  const srcDir = path.join(FIXTURE_ROOT, `${prefix}-${srcName}`);
  expect(existsSync(srcDir), `${srcDir} should exist`).toBe(true);
  const workDir = mkdtempSync(
    path.join(tmpdir(), `vitest-${prefix}-${srcName}-`),
  );
  cleanup.push(workDir);
  cpSync(srcDir, workDir, { recursive: true });
  const canonPath = path.join(workDir, "canonical", canonFile);
  const canonShaBefore = existsSync(canonPath) ? sha256(canonPath) : null;
  const output = path.join(workDir, "audit-output");
  const result = runCli([
    "--source",
    source,
    "--root",
    workDir,
    "--output",
    output,
  ]);
  const canonShaAfter = existsSync(canonPath) ? sha256(canonPath) : null;
  const jsonPath = path.join(output, "canon-drift.json");
  const mdPath = path.join(output, mdFile);
  const json = existsSync(jsonPath)
    ? JSON.parse(readFileSync(jsonPath, "utf8"))
    : null;
  return {
    ...result,
    workDir,
    canonShaBefore,
    canonShaAfter,
    jsonPath,
    mdPath,
    json,
  };
}

function runTypeFixture(srcName) {
  return runFixture({
    prefix: "canon-drift-types",
    srcName,
    source: "type-ownership",
    canonFile: "type-ownership.md",
    mdFile: "canon-drift.type-ownership.md",
  });
}

function runHelperFixture(srcName) {
  return runFixture({
    prefix: "canon-drift-helpers",
    srcName,
    source: "helper-registry",
    canonFile: "helper-registry.md",
    mdFile: "canon-drift.helper-registry.md",
  });
}

function runTopologyFixture(srcName) {
  return runFixture({
    prefix: "canon-drift-topology",
    srcName,
    source: "topology",
    canonFile: "topology.md",
    mdFile: "canon-drift.topology.md",
  });
}

describe("check-canon end-to-end drift fixtures", () => {
  it(
    "preserves clean type-ownership output and never rewrites canonical files",
    () => {
      const result = runTypeFixture("clean");
      expect(result.exit).toBe(0);
      expect(result.canonShaBefore).toBeTruthy();
      expect(result.canonShaBefore).toBe(result.canonShaAfter);
      expect(result.json.perSource["type-ownership"]).toMatchObject({
        status: "clean",
        driftCount: 0,
      });
    },
    CLI_TEST_TIMEOUT_MS,
  );

  it(
    "detects type-ownership added, removed, label, and owner drift",
    () => {
      const added = runTypeFixture("added");
      expect(added.exit).toBe(1);
      expect(added.json.drifts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ category: "identity-added" }),
        ]),
      );
      expect(added.canonShaBefore).toBe(added.canonShaAfter);

      const removed = runTypeFixture("removed");
      expect(removed.exit).toBe(1);
      expect(removed.json.drifts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ category: "identity-removed" }),
        ]),
      );
      expect(removed.canonShaBefore).toBe(removed.canonShaAfter);

      const labelChanged = runTypeFixture("label-changed");
      expect(labelChanged.exit).toBe(1);
      expect(labelChanged.json.drifts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            category: "label-changed",
            canon: expect.objectContaining({ label: expect.any(String) }),
            fresh: expect.objectContaining({ label: expect.any(String) }),
          }),
        ]),
      );

      const ownerChanged = runTypeFixture("owner-changed");
      expect(ownerChanged.exit).toBe(1);
      const ownerRecord = ownerChanged.json.drifts.find(
        (drift) => drift.category === "owner-changed",
      );
      expect(ownerRecord).toMatchObject({
        identity: expect.stringMatching(/^[^:]+::[^:]+$/),
        canon: expect.objectContaining({
          identity: expect.stringMatching(/^[^:]+::[^:]+$/),
          label: expect.any(String),
        }),
        fresh: expect.objectContaining({
          identity: expect.stringMatching(/^[^:]+::[^:]+$/),
          label: expect.any(String),
        }),
      });
      expect(ownerRecord.identity).not.toContain("→");
      expect(ownerRecord.canon.identity).not.toBe(ownerRecord.fresh.identity);
      expect(readFileSync(ownerChanged.mdPath, "utf8")).toContain(
        "Canon label",
      );
      expect(readFileSync(ownerChanged.mdPath, "utf8")).toContain(
        "Fresh label",
      );
    },
    CLI_TEST_TIMEOUT_MS,
  );

  it(
    "ignores stale type-ownership canonical-draft files",
    () => {
      const srcDir = path.join(FIXTURE_ROOT, "canon-drift-types-clean");
      const workDir = mkdtempSync(
        path.join(tmpdir(), "vitest-type-stale-draft-"),
      );
      cleanup.push(workDir);
      cpSync(srcDir, workDir, { recursive: true });
      mkdirSync(path.join(workDir, "canonical-draft"), { recursive: true });
      writeFileSync(
        path.join(workDir, "canonical-draft", "type-ownership.md"),
        "| Name | Identity | Owner | Fan-in | Status | Tags |\n" +
          "|--|--|--|--:|--|--|\n" +
          "| `BOGUS` | `src/bogus.ts::BOGUS` | `src/bogus.ts:1` | 99 | severely-any-contaminated | |\n",
        "utf8",
      );
      const result = runCli([
        "--source",
        "type-ownership",
        "--root",
        workDir,
        "--output",
        path.join(workDir, "audit-output"),
      ]);
      const json = JSON.parse(
        readFileSync(
          path.join(workDir, "audit-output", "canon-drift.json"),
          "utf8",
        ),
      );
      expect(result.exit).toBe(0);
      expect(json.perSource["type-ownership"].status).toBe("clean");
      expect(json.drifts).toEqual([]);
    },
    CLI_TEST_TIMEOUT_MS,
  );

  it(
    "detects helper-registry drift categories and preserves identity shape",
    () => {
      const clean = runHelperFixture("clean");
      expect(clean.exit).toBe(0);
      expect(clean.canonShaBefore).toBe(clean.canonShaAfter);
      expect(clean.json.perSource["helper-registry"].status).toBe("clean");

      for (const [fixture, category] of [
        ["added", "helper-added"],
        ["removed", "helper-removed"],
        ["label-changed", "label-changed"],
        ["contamination-changed", "contamination-changed"],
        ["fan-in-tier-changed", "fan-in-tier-changed"],
      ]) {
        const result = runHelperFixture(fixture);
        expect(result.exit, fixture).toBe(1);
        expect(result.json.drifts, fixture).toEqual(
          expect.arrayContaining([expect.objectContaining({ category })]),
        );
        expect(
          result.json.drifts.every(
            (drift) =>
              /^[^:]+::[^:]+$/.test(drift.identity) &&
              !drift.identity.includes("→"),
          ),
          fixture,
        ).toBe(true);
        if (fixture === "contamination-changed") {
          const md = readFileSync(result.mdPath, "utf8");
          expect(md).toContain("Canon signal");
          expect(md).toContain("Fresh signal");
          expect(result.json.drifts).toEqual(
            expect.arrayContaining([
              expect.objectContaining({
                category: "contamination-changed",
                canon: expect.objectContaining({
                  anyUnknownSignal: expect.any(String),
                }),
              }),
            ]),
          );
        }
      }
    },
    CLI_TEST_TIMEOUT_MS,
  );

  it(
    "detects topology drift categories and preserves display-scope evidence",
    () => {
      const clean = runTopologyFixture("clean");
      expect(clean.exit).toBe(0);
      expect(clean.canonShaBefore).toBe(clean.canonShaAfter);
      expect(clean.json.perSource.topology.status).toBe("clean");

      const submoduleAdded = runTopologyFixture("submodule-added");
      expect(submoduleAdded.exit).toBe(1);
      expect(submoduleAdded.json.drifts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            category: "submodule-added",
            identity: expect.not.stringMatching(/::|→/),
          }),
        ]),
      );

      const submoduleRemoved = runTopologyFixture("submodule-removed");
      expect(submoduleRemoved.exit).toBe(1);
      expect(submoduleRemoved.json.drifts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({ category: "submodule-removed" }),
        ]),
      );

      const scc = runTopologyFixture("scc-status-changed");
      expect(scc.exit).toBe(1);
      expect(scc.json.drifts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            category: "scc-status-changed",
            canon: expect.objectContaining({ sccMember: expect.any(Boolean) }),
            fresh: expect.objectContaining({ sccMember: expect.any(Boolean) }),
          }),
        ]),
      );

      const oversize = runTopologyFixture("oversize-changed");
      expect(oversize.exit).toBe(1);
      expect(oversize.json.drifts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            category: "oversize-changed",
            identity: expect.stringMatching(/\.ts|[/\\]/),
          }),
        ]),
      );

      const crossAdded = runTopologyFixture("cross-edge-added");
      expect(crossAdded.exit).toBe(1);
      expect(crossAdded.json.drifts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            category: "cross-edge-added",
            identity: expect.stringMatching(/ → /),
            fresh: expect.objectContaining({ count: expect.any(Number) }),
          }),
        ]),
      );
      expect(readFileSync(crossAdded.mdPath, "utf8")).toContain("top-30");

      const crossRemoved = runTopologyFixture("cross-edge-removed");
      expect(crossRemoved.exit).toBe(1);
      expect(crossRemoved.json.drifts).toEqual(
        expect.arrayContaining([
          expect.objectContaining({
            category: "cross-edge-removed",
            identity: expect.stringMatching(/ → /),
            canon: expect.objectContaining({
              count: expect.any(Number),
              line: expect.any(Number),
            }),
          }),
        ]),
      );
      expect(readFileSync(crossRemoved.mdPath, "utf8")).toContain("top-30");
    },
    CLI_TEST_TIMEOUT_MS,
  );

  it(
    "ignores stale topology canonical-draft files",
    () => {
      const srcDir = path.join(FIXTURE_ROOT, "canon-drift-topology-clean");
      const workDir = mkdtempSync(
        path.join(tmpdir(), "vitest-topology-stale-draft-"),
      );
      cleanup.push(workDir);
      cpSync(srcDir, workDir, { recursive: true });
      mkdirSync(path.join(workDir, "canonical-draft"), { recursive: true });
      writeFileSync(
        path.join(workDir, "canonical-draft", "topology.md"),
        "## 1. Submodule inventory\n\n" +
          "| Submodule | Files | LOC | In-edges | Out-edges | SCC | Status | Tags |\n" +
          "|-----------|------:|----:|---------:|----------:|-----|--------|------|\n" +
          "| `bogus` | 99 | 9999 | 0 | 0 | — | isolated-submodule ⚠ | |\n",
        "utf8",
      );
      const result = runCli([
        "--source",
        "topology",
        "--root",
        workDir,
        "--output",
        path.join(workDir, "audit-output"),
      ]);
      const json = JSON.parse(
        readFileSync(
          path.join(workDir, "audit-output", "canon-drift.json"),
          "utf8",
        ),
      );
      expect(result.exit).toBe(0);
      expect(json.perSource.topology.status).toBe("clean");
      expect(json.drifts).toEqual([]);
    },
    CLI_TEST_TIMEOUT_MS,
  );
});

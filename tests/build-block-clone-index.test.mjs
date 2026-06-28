import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import {
  BLOCK_CLONE_NOISE_POLICY_ID,
  assembleBlockCloneArtifact,
  applyBlockCloneNoisePolicy,
  pruneContainedBlockCloneGroups,
  tokenizeBlockCloneSource,
} from "../_lib/block-clone-artifact.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const BLOCK_CLI = path.join(REPO_ROOT, "build-block-clone-index.mjs");
const FUNCTION_CLI = path.join(REPO_ROOT, "build-function-clone-index.mjs");
const AUDIT_CLI = path.join(REPO_ROOT, "audit-repo.mjs");

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function createFixture(prefix = "vitest-block-clone-") {
  const root = mkdtempSync(path.join(tmpdir(), prefix));
  const output = mkdtempSync(path.join(tmpdir(), `${prefix}out-`));
  return {
    root,
    output,
    cleanup() {
      rmSync(root, { recursive: true, force: true });
      rmSync(output, { recursive: true, force: true });
    },
  };
}

function run(script, root, output, extraArgs = []) {
  return execFileSync(
    process.execPath,
    [script, "--root", root, "--output", output, ...extraArgs],
    {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function runAudit(root, output, extraArgs = []) {
  return execFileSync(
    process.execPath,
    [AUDIT_CLI, "--root", root, "--output", output, ...extraArgs],
    {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function readJson(file) {
  return JSON.parse(readFileSync(file, "utf8"));
}

function stableBlockCloneArtifact(artifact) {
  const {
    generated: _generated,
    meta: _meta,
    ...stable
  } = artifact;
  return stable;
}

function repeatedBlock({ sourceName, listName, outputName }) {
  return [
    `  const ${outputName}: string[] = [];`,
    `  for (const entry of ${listName}) {`,
    `    const rawValue = String(entry ?? '').trim();`,
    `    if (!rawValue) continue;`,
    `    const pieces = rawValue.split(':');`,
    `    const label = (pieces[0] ?? '').toLowerCase();`,
    `    const score = Number.parseInt(pieces[1] ?? '0', 10);`,
    `    if (Number.isNaN(score)) continue;`,
    `    const ranked = score > 10 ? 'high' : 'low';`,
    `    ${outputName}.push([${sourceName}, label, ranked].join('/'));`,
    `  }`,
    `  return ${outputName}.join('|');`,
  ].join("\n");
}

function destructuredRepeatedBlock({
  listName,
  idName,
  labelName,
  firstName,
  outputName,
}) {
  return [
    `  const ${outputName}: string[] = [];`,
    `  for (const row of ${listName}) {`,
    `    const { id: ${idName}, label: ${labelName} } = row;`,
    `    const [${firstName}] = row.parts ?? [];`,
    `    const rawValue = String(${idName} ?? '').trim();`,
    `    if (!rawValue) continue;`,
    `    const pieces = rawValue.split(':');`,
    `    const suffix = String(${firstName} ?? '').toLowerCase();`,
    `    const score = Number.parseInt(pieces[1] ?? '0', 10);`,
    `    if (Number.isNaN(score)) continue;`,
    `    const ranked = score > 10 ? 'high' : 'low';`,
    `    ${outputName}.push([${labelName}, suffix, ranked].join('/'));`,
    `  }`,
    `  return ${outputName}.join('|');`,
  ].join("\n");
}

function writeNestedCloneFixture(root) {
  write(
    root,
    "package.json",
    JSON.stringify({ name: "block-clone-fixture", type: "module" }),
  );
  write(
    root,
    "src/a.ts",
    [
      "export function loadUserRows(rows: unknown[]) {",
      "  const prefix = 'users';",
      repeatedBlock({
        sourceName: "prefix",
        listName: "rows",
        outputName: "normalizedUsers",
      }),
      "}",
      "",
    ].join("\n"),
  );
  write(
    root,
    "src/b.ts",
    [
      "export function scanOrderRows(records: unknown[]) {",
      "  const scope = 'orders';",
      "  if (records.length === 0) return 'empty';",
      repeatedBlock({
        sourceName: "scope",
        listName: "records",
        outputName: "normalizedOrders",
      }),
      "}",
      "",
    ].join("\n"),
  );
}

function groupTouchingFiles(artifact, files) {
  return artifact.groups.find((group) => {
    const actual = new Set(group.instances.map((instance) => instance.file));
    return files.every((file) => actual.has(file));
  });
}

function spansOverlap(a, b) {
  return (
    a.file === b.file && a.startToken < b.endToken && b.startToken < a.endToken
  );
}

function spanContains(outer, inner) {
  return (
    outer.file === inner.file &&
    outer.startToken <= inner.startToken &&
    outer.endToken >= inner.endToken
  );
}

function hasOverlappingInstances(group) {
  for (let i = 0; i < group.instances.length; i++) {
    for (let j = i + 1; j < group.instances.length; j++) {
      if (spansOverlap(group.instances[i], group.instances[j])) return true;
    }
  }
  return false;
}

function hasContainedGroup(groups) {
  return groups.some((candidate) =>
    groups.some(
      (other) =>
        candidate !== other &&
        other.tokenCount >= candidate.tokenCount &&
        candidate.instances.every((instance) =>
          other.instances.some((otherInstance) =>
            spanContains(otherInstance, instance),
          ),
        ),
    ),
  );
}

describe("build-block-clone-index producer artifact", () => {
  it("BC12. classifies mirror/test/same-file noise without deleting raw groups", () => {
    const { groups, noisePolicy } = applyBlockCloneNoisePolicy(
      [
        {
          id: "mirror",
          instances: [
            { file: "tests/hook-event-store.test.mjs" },
            { file: "tests/test-hook-event-store.mjs" },
          ],
        },
        {
          id: "scaffold",
          instances: [
            { file: "tests/test-a.mjs" },
            { file: "tests/test-b.mjs" },
          ],
        },
        {
          id: "same-file",
          instances: [{ file: "_lib/a.mjs" }, { file: "_lib/a.mjs" }],
        },
        {
          id: "directory-collision",
          instances: [
            { file: "tests/auth/test-index.mjs" },
            { file: "tests/payments/index.test.mjs" },
          ],
        },
        {
          id: "engine",
          instances: [{ file: "_lib/a.mjs" }, { file: "_lib/b.mjs" }],
        },
      ],
      { thresholds: { maxGroups: 5 } },
    );
    const byId = Object.fromEntries(groups.map((group) => [group.id, group]));

    expect(noisePolicy).toEqual({
      policyId: BLOCK_CLONE_NOISE_POLICY_ID,
      reviewGroupCount: 1,
      mutedGroupCount: 4,
      mutedByReason: {
        "node-vitest-mirror-pair": 1,
        "same-file-repeat": 1,
        "test-scaffold-repeat": 2,
      },
      candidateCapSaturated: false,
      reviewCapSaturated: false,
      mutedCapSaturated: false,
    });
    expect(groups).toHaveLength(5);
    expect(byId.mirror.muteReason).toBe("node-vitest-mirror-pair");
    expect(byId.scaffold.muteReason).toBe("test-scaffold-repeat");
    expect(byId["same-file"].muteReason).toBe("same-file-repeat");
    expect(byId["directory-collision"].muteReason).toBe(
      "test-scaffold-repeat",
    );
    expect(byId.engine.visibility).toBe("review");
  });

  it("BC12c/BC12d. preserves review groups under noise pressure and keeps maxGroups total", () => {
    const pressureInput = [
      {
        id: "same-file-large",
        tokenCount: 300,
        occurrenceCount: 2,
        instances: [
          { file: "src/noisy-fixture.ts" },
          { file: "src/noisy-fixture.ts" },
        ],
      },
      {
        id: "test-scaffold-large",
        tokenCount: 250,
        occurrenceCount: 2,
        instances: [
          { file: "tests/test-a.mjs" },
          { file: "tests/test-b.mjs" },
        ],
      },
      {
        id: "review-small",
        tokenCount: 50,
        occurrenceCount: 2,
        instances: [{ file: "src/a.ts" }, { file: "src/b.ts" }],
      },
    ];

    const { groups, noisePolicy } = applyBlockCloneNoisePolicy(pressureInput, {
      thresholds: {
        maxCandidateGroups: 10,
        maxReviewGroups: 1,
        maxMutedGroups: 1,
      },
    });

    expect(groups).toHaveLength(2);
    expect(groups[0]).toMatchObject({
      id: "review-small",
      visibility: "review",
    });
    expect(groups[1]?.visibility).toBe("muted");
    expect(noisePolicy).toMatchObject({
      reviewGroupCount: 1,
      mutedGroupCount: 1,
      candidateCapSaturated: false,
      reviewCapSaturated: false,
      mutedCapSaturated: true,
    });

    const legacy = applyBlockCloneNoisePolicy(pressureInput, {
      thresholds: {
        maxGroups: 2,
        maxReviewGroups: 100,
        maxMutedGroups: 100,
      },
    });

    expect(legacy.groups).toHaveLength(2);
    expect(legacy.groups[0]).toMatchObject({
      id: "review-small",
      visibility: "review",
    });
    expect(legacy.noisePolicy).toMatchObject({
      reviewGroupCount: 1,
      mutedGroupCount: 1,
    });

    const legacyArtifact = assembleBlockCloneArtifact({
      root: "legacy-max-groups-fixture",
      files: [],
      thresholds: { maxGroups: 2 },
    });

    expect(legacyArtifact.thresholds.maxGroups).toBe(2);
  });

  it("BC12e. prunes contained block clone groups in ranked order with a cap sentinel", () => {
    const group = (id, tokenCount, startA, endA, startB, endB) => ({
      id,
      tokenCount,
      occurrenceCount: 2,
      instances: [
        { file: "src/a.ts", startToken: startA, endToken: endA },
        { file: "src/b.ts", startToken: startB, endToken: endB },
      ],
    });
    const parent = group("parent", 100, 0, 100, 200, 300);
    const child = group("child", 50, 10, 60, 210, 260);
    const independentOne = group("independent-one", 90, 400, 490, 600, 690);
    const independentTwo = group("independent-two", 80, 800, 880, 900, 980);
    const independentThree = group(
      "independent-three",
      70,
      1000,
      1070,
      1100,
      1170,
    );

    const pruned = pruneContainedBlockCloneGroups(
      [
        child,
        independentThree,
        independentTwo,
        parent,
        independentOne,
      ],
      { maxGroups: 3 },
    );

    expect(pruned.map((item) => item.id)).toEqual([
      "parent",
      "independent-one",
      "independent-two",
    ]);
  });

  it("BC1/BC2. surfaces renamed nested block clones without widening function clone lanes", () => {
    const fixture = createFixture();
    const functionOutput = mkdtempSync(
      path.join(tmpdir(), "vitest-block-clone-fn-out-"),
    );
    try {
      writeNestedCloneFixture(fixture.root);

      const stdout = run(BLOCK_CLI, fixture.root, fixture.output);
      run(FUNCTION_CLI, fixture.root, functionOutput);
      const block = readJson(path.join(fixture.output, "block-clones.json"));
      const phase = readJson(
        path.join(
          fixture.output,
          ".producer-phases",
          "build-block-clone-index.mjs.json",
        ),
      );
      const fn = readJson(path.join(functionOutput, "function-clones.json"));
      const group = groupTouchingFiles(block, ["src/a.ts", "src/b.ts"]);

      expect(existsSync(path.join(fixture.output, "block-clones.json"))).toBe(
        true,
      );
      expect(stdout).toContain("[block-clones]");
      expect(stdout).toContain("review groups");
      expect(block.schemaVersion).toBe("block-clones.v1");
      expect(block.policyVersion).toBe("block-clone-review-policy-v1");
      expect(block.status).toBe("complete");
      expect(block.noisePolicy?.policyId).toBe(BLOCK_CLONE_NOISE_POLICY_ID);
      expect(phase).toMatchObject({
        schemaVersion: "producer-phase-timing.v1",
        producer: "build-block-clone-index.mjs",
      });
      expect(phase.phases.map((item) => item.name)).toEqual([
        "collect-files",
        "tokenize-files",
        "assemble-artifact",
        "write-artifact",
      ]);
      expect(phase.counters).toMatchObject({
        filesCollected: 2,
        tokenizedFiles: 2,
      });
      expect(group).toBeTruthy();
      expect(group.reviewOnly).toBe(true);
      expect(group.visibility).toBe("review");
      expect(group.eligibleForSafeFix).toBe(false);
      expect(group.normalizationMode).toBe("alpha-identifier");
      expect(group.reasons).toContain("suffix-array-lcp-repeat");

      const functionLaneText = JSON.stringify([
        fn.exactBodyGroups,
        fn.structureGroups,
        fn.signatureGroups,
        fn.nearFunctionCandidates,
      ]);
      expect(functionLaneText).not.toContain("#");
    } finally {
      fixture.cleanup();
      rmSync(functionOutput, { recursive: true, force: true });
    }
  });

  it("BC13. reuses block clone artifact across output directories through cache-root", () => {
    const fixture = createFixture("vitest-block-clone-inc-");
    const coldOutput = mkdtempSync(
      path.join(tmpdir(), "vitest-block-clone-inc-cold-"),
    );
    const warmOutput = mkdtempSync(
      path.join(tmpdir(), "vitest-block-clone-inc-warm-"),
    );
    const cacheRoot = mkdtempSync(
      path.join(tmpdir(), "vitest-block-clone-inc-cache-"),
    );
    try {
      writeNestedCloneFixture(fixture.root);

      run(BLOCK_CLI, fixture.root, coldOutput, ["--cache-root", cacheRoot]);
      run(BLOCK_CLI, fixture.root, warmOutput, ["--cache-root", cacheRoot]);

      const cold = readJson(path.join(coldOutput, "block-clones.json"));
      const warm = readJson(path.join(warmOutput, "block-clones.json"));
      const warmPhase = readJson(
        path.join(
          warmOutput,
          ".producer-phases",
          "build-block-clone-index.mjs.json",
        ),
      );

      expect(stableBlockCloneArtifact(warm)).toEqual(
        stableBlockCloneArtifact(cold),
      );
      expect(cold.meta.incremental).toMatchObject({
        enabled: true,
        reusedResult: false,
        reason: "cache-miss",
      });
      expect(warm.meta.incremental).toMatchObject({
        enabled: true,
        reusedResult: true,
        reason: "cache-hit",
      });
      expect(path.resolve(warm.meta.incremental.cacheRoot)).toBe(
        path.resolve(cacheRoot),
      );
      expect(warmPhase.counters).toMatchObject({
        cacheReusedResult: 1,
        tokenizedFiles: 0,
        tokenCount: 0,
      });
    } finally {
      fixture.cleanup();
      rmSync(coldOutput, { recursive: true, force: true });
      rmSync(warmOutput, { recursive: true, force: true });
      rmSync(cacheRoot, { recursive: true, force: true });
    }
  });

  it("BC14. audit-repo forwards cache-root to block clone producer", () => {
    const fixture = createFixture("vitest-block-clone-audit-inc-");
    const cacheRoot = mkdtempSync(
      path.join(tmpdir(), "vitest-block-clone-audit-inc-cache-"),
    );
    try {
      writeNestedCloneFixture(fixture.root);

      runAudit(fixture.root, fixture.output, [
        "--profile",
        "full",
        "--cache-root",
        cacheRoot,
      ]);
      runAudit(fixture.root, fixture.output, [
        "--profile",
        "full",
        "--cache-root",
        cacheRoot,
      ]);

      const warm = readJson(path.join(fixture.output, "block-clones.json"));
      expect(warm.meta.incremental).toMatchObject({
        enabled: true,
        reusedResult: true,
        reason: "cache-hit",
      });
      expect(path.resolve(warm.meta.incremental.cacheRoot)).toBe(
        path.resolve(cacheRoot),
      );
    } finally {
      fixture.cleanup();
      rmSync(cacheRoot, { recursive: true, force: true });
    }
  }, 60_000);

  it("BC3/BC4/BC6. skips import boilerplate and refuses overlapping same-file fake occurrences", () => {
    const fixture = createFixture("vitest-block-clone-imports-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "block-clone-imports", type: "module" }),
      );
      const imports = [
        "import a from 'a';",
        "import b from 'b';",
        "import c from 'c';",
        "import d from 'd';",
        "import e from 'e';",
        "import f from 'f';",
      ].join("\n");
      write(fixture.root, "src/a.ts", `${imports}\nexport const aValue = 1;\n`);
      write(fixture.root, "src/b.ts", `${imports}\nexport const bValue = 2;\n`);
      write(
        fixture.root,
        "src/overlap.ts",
        [
          "export function overlap(items: string[]) {",
          repeatedBlock({
            sourceName: "'one'",
            listName: "items",
            outputName: "first",
          }),
          repeatedBlock({
            sourceName: "'two'",
            listName: "items",
            outputName: "second",
          }),
          "}",
          "",
        ].join("\n"),
      );

      run(BLOCK_CLI, fixture.root, fixture.output);
      const block = readJson(path.join(fixture.output, "block-clones.json"));
      const importOnlyGroup = groupTouchingFiles(block, [
        "src/a.ts",
        "src/b.ts",
      ]);

      expect(importOnlyGroup).toBeUndefined();
      expect(block.groups.some(hasOverlappingInstances)).toBe(false);
      expect(hasContainedGroup(block.groups)).toBe(false);
    } finally {
      fixture.cleanup();
    }
  });

  it("BC5/BC7/BC8. keeps spans file-local and records limited-confidence inputs", () => {
    const fixture = createFixture("vitest-block-clone-limited-");
    try {
      writeNestedCloneFixture(fixture.root);
      write(
        fixture.root,
        "src/generated/repeated.generated.ts",
        [
          "// @generated",
          "export function generatedOne(items: string[]) {",
          repeatedBlock({
            sourceName: "'generated'",
            listName: "items",
            outputName: "out",
          }),
          "}",
          "",
        ].join("\n"),
      );
      write(fixture.root, "src/bad.ts", "export function broken( {");

      run(BLOCK_CLI, fixture.root, fixture.output);
      const block = readJson(path.join(fixture.output, "block-clones.json"));

      expect(
        block.groups.every((group) =>
          group.instances.every(
            (instance) =>
              typeof instance.file === "string" &&
              instance.startLine <= instance.endLine,
          ),
        ),
      ).toBe(true);
      expect(block.status).toBe("confidence-limited");
      expect(
        block.skipped.some(
          (entry) => entry.file === "src/generated/repeated.generated.ts",
        ),
      ).toBe(true);
      expect(
        block.diagnostics.some(
          (entry) =>
            entry.file === "src/bad.ts" && entry.kind === "parse-error",
        ),
      ).toBe(true);
    } finally {
      fixture.cleanup();
    }
  });

  it("BC10/BC11. normalizes destructured bindings without treating object keys as locals", () => {
    const fixture = createFixture("vitest-block-clone-patterns-");
    try {
      write(
        fixture.root,
        "package.json",
        JSON.stringify({ name: "block-clone-patterns", type: "module" }),
      );
      write(
        fixture.root,
        "src/a.ts",
        [
          "export function loadUserRows(rows: any[]) {",
          destructuredRepeatedBlock({
            listName: "rows",
            idName: "userId",
            labelName: "userLabel",
            firstName: "firstUserPart",
            outputName: "normalizedUsers",
          }),
          "}",
          "",
        ].join("\n"),
      );
      write(
        fixture.root,
        "src/b.ts",
        [
          "export function scanOrderRows(records: any[]) {",
          destructuredRepeatedBlock({
            listName: "records",
            idName: "orderId",
            labelName: "orderLabel",
            firstName: "firstOrderPart",
            outputName: "normalizedOrders",
          }),
          "}",
          "",
        ].join("\n"),
      );

      run(BLOCK_CLI, fixture.root, fixture.output);
      const block = readJson(path.join(fixture.output, "block-clones.json"));
      const group = groupTouchingFiles(block, ["src/a.ts", "src/b.ts"]);
      expect(group).toBeTruthy();
      expect(group.instances.length).toBeGreaterThanOrEqual(2);

      const tokenized = tokenizeBlockCloneSource({
        root: fixture.root,
        filePath: path.join(fixture.root, "src/param.ts"),
        src: "export function probe({ id: userId }: any) { return id + userId; }",
      });
      const values = tokenized.tokens.map((token) => token.value);
      expect(values).toContain("GLOBAL:id");
      expect(values).toContain("REF:$0");
    } finally {
      fixture.cleanup();
    }
  });

  it("BC9. full profile emits block-clones.json without Markdown or action-lane leakage", () => {
    const fixture = createFixture("vitest-block-clone-audit-");
    try {
      writeNestedCloneFixture(fixture.root);
      run(AUDIT_CLI, fixture.root, fixture.output, [
        "--profile",
        "full",
        "--no-incremental",
      ]);
      const manifest = readJson(path.join(fixture.output, "manifest.json"));
      const summary = existsSync(
        path.join(fixture.output, "audit-summary.latest.md"),
      )
        ? readFileSync(
            path.join(fixture.output, "audit-summary.latest.md"),
            "utf8",
          )
        : "";
      const reviewPack = existsSync(
        path.join(fixture.output, "audit-review-pack.latest.md"),
      )
        ? readFileSync(
            path.join(fixture.output, "audit-review-pack.latest.md"),
            "utf8",
          )
        : "";
      const fixPlan = existsSync(path.join(fixture.output, "fix-plan.json"))
        ? readFileSync(path.join(fixture.output, "fix-plan.json"), "utf8")
        : "";
      const actionSafety = existsSync(
        path.join(fixture.output, "export-action-safety.json"),
      )
        ? readFileSync(
            path.join(fixture.output, "export-action-safety.json"),
            "utf8",
          )
        : "";

      expect(existsSync(path.join(fixture.output, "block-clones.json"))).toBe(
        true,
      );
      expect(manifest.artifactsProduced).toContain("block-clones.json");
      expect(manifest.blockClones).toMatchObject({
        artifact: "block-clones.json",
        reviewOnly: true,
        thresholdPolicyId: "block-clone-threshold-policy-v2",
      });
      expect(summary + reviewPack).not.toMatch(
        /block-clones\.json|Review repeated code region|repeated normalized token region/i,
      );
      expect(fixPlan + actionSafety).not.toMatch(
        /block-clones\.json|block-clone:sha256|repeated normalized token region/i,
      );
    } finally {
      fixture.cleanup();
    }
  }, 60000);
});

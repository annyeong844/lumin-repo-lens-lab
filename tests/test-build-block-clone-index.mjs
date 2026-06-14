// Tests for build-block-clone-index.mjs — review-only repeated block evidence.

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

import {
  BLOCK_CLONE_NOISE_POLICY_ID,
  assembleBlockCloneArtifact,
  applyBlockCloneNoisePolicy,
  pruneContainedBlockCloneGroups,
  tokenizeBlockCloneSource,
} from "../_lib/block-clone-artifact.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const DIR = path.resolve(__dirname, "..");
const NODE = process.execPath;
const BLOCK_CLI = path.join(DIR, "build-block-clone-index.mjs");
const FUNCTION_CLI = path.join(DIR, "build-function-clone-index.mjs");
const AUDIT_CLI = path.join(DIR, "audit-repo.mjs");

let passed = 0,
  failed = 0;
function assert(label, ok, detail = "") {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}\n        ${detail}`);
  }
}

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function run(script, root, output, extraArgs = []) {
  return execFileSync(
    NODE,
    [script, "--root", root, "--output", output, ...extraArgs],
    {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function runAudit(root, output, extraArgs = []) {
  return execFileSync(
    NODE,
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

function readBlock(output) {
  return readJson(path.join(output, "block-clones.json"));
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

function createNestedCloneFixture() {
  const fx = mkdtempSync(path.join(tmpdir(), "block-clone-"));
  write(
    fx,
    "package.json",
    JSON.stringify({ name: "block-clone-fixture", type: "module" }),
  );
  write(
    fx,
    "src/a.ts",
    [
      `export function loadUserRows(rows: unknown[]) {`,
      `  const prefix = 'users';`,
      repeatedBlock({
        sourceName: "prefix",
        listName: "rows",
        outputName: "normalizedUsers",
      }),
      `}`,
      ``,
    ].join("\n"),
  );
  write(
    fx,
    "src/b.ts",
    [
      `export function scanOrderRows(records: unknown[]) {`,
      `  const scope = 'orders';`,
      `  if (records.length === 0) return 'empty';`,
      repeatedBlock({
        sourceName: "scope",
        listName: "records",
        outputName: "normalizedOrders",
      }),
      `}`,
      ``,
    ].join("\n"),
  );
  return fx;
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

// BC12. Noise policy classifies mirror/test/same-file groups without deleting
// raw groups.
{
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
        instances: [
          { file: "_lib/a.mjs" },
          { file: "_lib/a.mjs" },
        ],
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
        instances: [
          { file: "_lib/a.mjs" },
          { file: "_lib/b.mjs" },
        ],
      },
    ],
    { thresholds: { maxGroups: 5 } },
  );
  const byId = Object.fromEntries(groups.map((group) => [group.id, group]));
  assert(
    "BC12a. artifact noise policy keeps raw groups while classifying muted lanes",
    noisePolicy.policyId === BLOCK_CLONE_NOISE_POLICY_ID &&
      noisePolicy.reviewGroupCount === 1 &&
      noisePolicy.mutedGroupCount === 4 &&
      noisePolicy.candidateCapSaturated === false &&
      noisePolicy.reviewCapSaturated === false &&
      noisePolicy.mutedCapSaturated === false &&
      groups.length === 5,
    JSON.stringify({ groups, noisePolicy }, null, 2),
  );
  assert(
    "BC12b. mirror, test scaffold, and same-file groups get stable mute reasons",
    byId.mirror?.muteReason === "node-vitest-mirror-pair" &&
      byId.scaffold?.muteReason === "test-scaffold-repeat" &&
      byId["same-file"]?.muteReason === "same-file-repeat" &&
      byId["directory-collision"]?.muteReason === "test-scaffold-repeat" &&
      byId.engine?.visibility === "review",
    JSON.stringify(groups, null, 2),
  );
}

// BC12c. Review groups survive high-ranked muted noise, and legacy maxGroups
// still acts as a total output cap.
{
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
      instances: [{ file: "tests/test-a.mjs" }, { file: "tests/test-b.mjs" }],
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

  assert(
    "BC12c. muted noise cannot displace a lower-ranked review group",
    groups.length === 2 &&
      groups[0]?.id === "review-small" &&
      groups[0]?.visibility === "review" &&
      groups[1]?.visibility === "muted" &&
      noisePolicy.reviewGroupCount === 1 &&
      noisePolicy.mutedGroupCount === 1 &&
      noisePolicy.candidateCapSaturated === false &&
      noisePolicy.reviewCapSaturated === false &&
      noisePolicy.mutedCapSaturated === true,
    JSON.stringify({ groups, noisePolicy }, null, 2),
  );

  const legacy = applyBlockCloneNoisePolicy(pressureInput, {
    thresholds: {
      maxGroups: 2,
      maxReviewGroups: 100,
      maxMutedGroups: 100,
    },
  });
  assert(
    "BC12d. deprecated maxGroups remains a total output cap",
    legacy.groups.length === 2 &&
      legacy.groups[0]?.id === "review-small" &&
      legacy.noisePolicy.reviewGroupCount === 1 &&
      legacy.noisePolicy.mutedGroupCount === 1,
    JSON.stringify(legacy, null, 2),
  );

  const legacyArtifact = assembleBlockCloneArtifact({
    root: "legacy-max-groups-fixture",
    files: [],
    thresholds: { maxGroups: 2 },
  });
  assert(
    "BC12e. deprecated maxGroups is preserved in artifact thresholds when supplied",
    legacyArtifact.thresholds.maxGroups === 2,
    JSON.stringify(legacyArtifact.thresholds, null, 2),
  );
}

// BC12f. Contained block clone pruning works in rank order and can stop at a
// candidate sentinel.
{
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
    [child, independentThree, independentTwo, parent, independentOne],
    { maxGroups: 3 },
  );

  assert(
    "BC12f. subset pruning keeps ranked non-contained groups up to the cap",
    pruned.map((item) => item.id).join(",") ===
      "parent,independent-one,independent-two",
    JSON.stringify(pruned, null, 2),
  );
}

// BC1/BC2. Nested renamed blocks are block-clone evidence only; top-level
// function clone lanes remain separate.
{
  const fx = createNestedCloneFixture();
  const out = mkdtempSync(path.join(tmpdir(), "block-clone-out-"));
  const fnOut = mkdtempSync(path.join(tmpdir(), "block-clone-fn-out-"));
  try {
    const stdout = run(BLOCK_CLI, fx, out);
    run(FUNCTION_CLI, fx, fnOut);
    const block = readBlock(out);
    const phase = readJson(
      path.join(out, ".producer-phases", "build-block-clone-index.mjs.json"),
    );
    const fn = readJson(path.join(fnOut, "function-clones.json"));
    const group = groupTouchingFiles(block, ["src/a.ts", "src/b.ts"]);
    const blockIds = new Set(
      group?.instances
        .map((instance) => instance.container?.name)
        .filter(Boolean),
    );

    assert(
      "BC1a. CLI writes block-clones.json",
      existsSync(path.join(out, "block-clones.json")),
    );
    assert(
      "BC1b. stdout summarizes block clone run",
      stdout.includes("[block-clones]") && stdout.includes("review groups"),
      stdout,
    );
    assert(
      "BC1c. artifact declares review-only block clone schema",
      block.schemaVersion === "block-clones.v1" &&
        block.policyVersion === "block-clone-review-policy-v1" &&
        block.status === "complete" &&
        block.noisePolicy?.policyId === BLOCK_CLONE_NOISE_POLICY_ID,
      JSON.stringify(block, null, 2),
    );
    assert(
      "BC1d. producer phase timing sidecar records block-clone phases",
      phase.schemaVersion === "producer-phase-timing.v1" &&
        phase.producer === "build-block-clone-index.mjs" &&
        phase.phases?.map((item) => item.name).join(",") ===
          "collect-files,tokenize-files,assemble-artifact,write-artifact" &&
        phase.counters?.filesCollected === 2 &&
        phase.counters?.tokenizedFiles === 2,
      JSON.stringify(phase, null, 2),
    );
    assert(
      "BC1e. renamed nested regions are grouped across files",
      !!group &&
        group.reviewOnly === true &&
        group.visibility === "review" &&
        group.eligibleForSafeFix === false &&
        group.normalizationMode === "alpha-identifier" &&
        group.reasons.includes("suffix-array-lcp-repeat") &&
        blockIds.has("loadUserRows") &&
        blockIds.has("scanOrderRows"),
      JSON.stringify(block.groups, null, 2),
    );
    assert(
      "BC2. block clone evidence does not widen function clone lanes",
      !fn.exactBodyGroups.some((candidate) =>
        candidate.identities?.some((id) => id.includes("#")),
      ) &&
        !fn.structureGroups.some((candidate) =>
          candidate.identities?.some((id) => id.includes("#")),
        ) &&
        !fn.signatureGroups.some((candidate) =>
          candidate.identities?.some((id) => id.includes("#")),
        ) &&
        !fn.nearFunctionCandidates?.some((candidate) =>
          candidate.identities?.some((id) => id.includes("#")),
        ),
      JSON.stringify(fn, null, 2),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
    rmSync(fnOut, { recursive: true, force: true });
  }
}

// BC13. Shared incremental cache reuses the full block-clone artifact across
// output directories.
{
  const fx = createNestedCloneFixture();
  const coldOut = mkdtempSync(path.join(tmpdir(), "block-clone-inc-cold-"));
  const warmOut = mkdtempSync(path.join(tmpdir(), "block-clone-inc-warm-"));
  const cacheRoot = mkdtempSync(path.join(tmpdir(), "block-clone-inc-cache-"));
  try {
    run(BLOCK_CLI, fx, coldOut, ["--cache-root", cacheRoot]);
    run(BLOCK_CLI, fx, warmOut, ["--cache-root", cacheRoot]);
    const cold = readBlock(coldOut);
    const warm = readBlock(warmOut);
    const warmPhase = readJson(
      path.join(warmOut, ".producer-phases", "build-block-clone-index.mjs.json"),
    );

    assert(
      "BC13a. warm block clone artifact matches cold artifact apart from run-local meta",
      JSON.stringify(stableBlockCloneArtifact(warm)) ===
        JSON.stringify(stableBlockCloneArtifact(cold)),
      JSON.stringify({ cold, warm }, null, 2),
    );
    assert(
      "BC13b. warm block clone run reports a shared cache hit",
      cold.meta?.incremental?.reusedResult === false &&
        cold.meta?.incremental?.reason === "cache-miss" &&
        warm.meta?.incremental?.reusedResult === true &&
        warm.meta?.incremental?.reason === "cache-hit" &&
        path.resolve(warm.meta.incremental.cacheRoot) === path.resolve(cacheRoot),
      JSON.stringify({ cold: cold.meta, warm: warm.meta }, null, 2),
    );
    assert(
      "BC13c. cache-hit run skips tokenization and assembly work",
      warmPhase.counters?.cacheReusedResult === 1 &&
        warmPhase.counters?.tokenizedFiles === 0 &&
        warmPhase.counters?.tokenCount === 0,
      JSON.stringify(warmPhase, null, 2),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(coldOut, { recursive: true, force: true });
    rmSync(warmOut, { recursive: true, force: true });
    rmSync(cacheRoot, { recursive: true, force: true });
  }
}

// BC14. audit-repo forwards --cache-root to the block clone producer.
{
  const fx = createNestedCloneFixture();
  const out = mkdtempSync(path.join(tmpdir(), "block-clone-audit-inc-out-"));
  const cacheRoot = mkdtempSync(path.join(tmpdir(), "block-clone-audit-inc-cache-"));
  try {
    runAudit(fx, out, ["--profile", "full", "--cache-root", cacheRoot]);
    runAudit(fx, out, ["--profile", "full", "--cache-root", cacheRoot]);
    const warm = readBlock(out);
    assert(
      "BC14. full audit forwards cache-root to block clone producer",
      warm.meta?.incremental?.reusedResult === true &&
        warm.meta?.incremental?.reason === "cache-hit" &&
        path.resolve(warm.meta.incremental.cacheRoot) === path.resolve(cacheRoot),
      JSON.stringify(warm.meta, null, 2),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
    rmSync(cacheRoot, { recursive: true, force: true });
  }
}

// BC3/BC4/BC6. Import boilerplate is skipped, overlap does not fake counts, and
// contained subset groups are removed.
{
  const fx = mkdtempSync(path.join(tmpdir(), "block-clone-imports-"));
  const out = mkdtempSync(path.join(tmpdir(), "block-clone-imports-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "block-clone-imports", type: "module" }),
    );
    const imports = [
      `import a from 'a';`,
      `import b from 'b';`,
      `import c from 'c';`,
      `import d from 'd';`,
      `import e from 'e';`,
      `import f from 'f';`,
    ].join("\n");
    write(fx, "src/a.ts", `${imports}\nexport const aValue = 1;\n`);
    write(fx, "src/b.ts", `${imports}\nexport const bValue = 2;\n`);
    write(
      fx,
      "src/overlap.ts",
      [
        `export function overlap(items: string[]) {`,
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
        `}`,
        ``,
      ].join("\n"),
    );

    run(BLOCK_CLI, fx, out);
    const block = readBlock(out);
    const importOnlyGroup = groupTouchingFiles(block, ["src/a.ts", "src/b.ts"]);

    assert(
      "BC3. repeated import blocks do not dominate block groups",
      !importOnlyGroup,
      JSON.stringify(block.groups, null, 2),
    );
    assert(
      "BC4/BC6. same-file overlapping or contained repeats do not fake occurrence counts",
      !block.groups.some(hasOverlappingInstances) &&
        !hasContainedGroup(block.groups),
      JSON.stringify(block.groups, null, 2),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// BC5/BC7/BC8. Sentinels keep spans file-local; generated/parse-error inputs
// become explicit limited evidence.
{
  const fx = createNestedCloneFixture();
  const out = mkdtempSync(path.join(tmpdir(), "block-clone-limited-out-"));
  try {
    write(
      fx,
      "src/generated/repeated.generated.ts",
      [
        `// @generated`,
        `export function generatedOne(items: string[]) {`,
        repeatedBlock({
          sourceName: "'generated'",
          listName: "items",
          outputName: "out",
        }),
        `}`,
        ``,
      ].join("\n"),
    );
    write(fx, "src/bad.ts", `export function broken( {`);

    run(BLOCK_CLI, fx, out);
    const block = readBlock(out);
    const crossFileSpan = block.groups.some((group) =>
      group.instances.some(
        (instance) => !instance.file || instance.startLine > instance.endLine,
      ),
    );

    assert(
      "BC5. emitted instances are file-local spans",
      !crossFileSpan &&
        block.groups.every((group) =>
          group.instances.every(
            (instance) => typeof instance.file === "string",
          ),
        ),
      JSON.stringify(block.groups, null, 2),
    );
    assert(
      "BC7. generated files are explicit skipped evidence",
      block.status === "confidence-limited" &&
        block.skipped.some(
          (entry) => entry.file === "src/generated/repeated.generated.ts",
        ),
      JSON.stringify(block, null, 2),
    );
    assert(
      "BC8. parser failures do not make empty groups look complete",
      block.status === "confidence-limited" &&
        block.diagnostics.some(
          (entry) =>
            entry.file === "src/bad.ts" && entry.kind === "parse-error",
        ),
      JSON.stringify(block, null, 2),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// BC10/BC11. Destructured locals are alpha-normalized, while object-pattern
// keys do not become fake local bindings.
{
  const fx = mkdtempSync(path.join(tmpdir(), "block-clone-patterns-"));
  const out = mkdtempSync(path.join(tmpdir(), "block-clone-patterns-out-"));
  try {
    write(
      fx,
      "package.json",
      JSON.stringify({ name: "block-clone-patterns", type: "module" }),
    );
    write(
      fx,
      "src/a.ts",
      [
        `export function loadUserRows(rows: any[]) {`,
        destructuredRepeatedBlock({
          listName: "rows",
          idName: "userId",
          labelName: "userLabel",
          firstName: "firstUserPart",
          outputName: "normalizedUsers",
        }),
        `}`,
        ``,
      ].join("\n"),
    );
    write(
      fx,
      "src/b.ts",
      [
        `export function scanOrderRows(records: any[]) {`,
        destructuredRepeatedBlock({
          listName: "records",
          idName: "orderId",
          labelName: "orderLabel",
          firstName: "firstOrderPart",
          outputName: "normalizedOrders",
        }),
        `}`,
        ``,
      ].join("\n"),
    );

    run(BLOCK_CLI, fx, out);
    const block = readBlock(out);
    const group = groupTouchingFiles(block, ["src/a.ts", "src/b.ts"]);
    const tokenized = tokenizeBlockCloneSource({
      root: fx,
      filePath: path.join(fx, "src/param.ts"),
      src: `export function probe({ id: userId }: any) { return id + userId; }`,
    });
    const values = tokenized.tokens.map((token) => token.value);

    assert(
      "BC10. destructured renamed locals are normalized into clone groups",
      !!group && group.instances.length >= 2,
      JSON.stringify(block.groups, null, 2),
    );
    assert(
      "BC11. object-pattern keys do not declare fake local refs",
      values.includes("GLOBAL:id") && values.includes("REF:$0"),
      values.join("\n"),
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

// BC9. Full profile produces the artifact without rendering action language.
{
  const fx = createNestedCloneFixture();
  const out = mkdtempSync(path.join(tmpdir(), "block-clone-audit-out-"));
  try {
    run(AUDIT_CLI, fx, out, ["--profile", "full", "--no-incremental"]);
    const manifest = readJson(path.join(out, "manifest.json"));
    const summary = existsSync(path.join(out, "audit-summary.latest.md"))
      ? readFileSync(path.join(out, "audit-summary.latest.md"), "utf8")
      : "";
    const reviewPack = existsSync(path.join(out, "audit-review-pack.latest.md"))
      ? readFileSync(path.join(out, "audit-review-pack.latest.md"), "utf8")
      : "";
    const fixPlan = existsSync(path.join(out, "fix-plan.json"))
      ? readFileSync(path.join(out, "fix-plan.json"), "utf8")
      : "";
    const actionSafety = existsSync(path.join(out, "export-action-safety.json"))
      ? readFileSync(path.join(out, "export-action-safety.json"), "utf8")
      : "";

    assert(
      "BC9a. full profile produces block-clones.json",
      existsSync(path.join(out, "block-clones.json")) &&
        manifest.artifactsProduced?.includes("block-clones.json") &&
        manifest.blockClones?.artifact === "block-clones.json" &&
        manifest.blockClones?.reviewOnly === true &&
        manifest.blockClones?.thresholdPolicyId ===
          "block-clone-threshold-policy-v2",
      JSON.stringify(manifest.artifactsProduced),
    );
    assert(
      "BC9b. block clone P1 does not render Markdown or action lanes",
      !/block-clones\.json|Review repeated code region|repeated normalized token region/i.test(
        summary + reviewPack,
      ) &&
        !/block-clones\.json|block-clone:sha256|repeated normalized token region/i.test(
          fixPlan + actionSafety,
        ),
      `summary=${summary}\nreview=${reviewPack}\nfix=${fixPlan}\naction=${actionSafety}`,
    );
  } finally {
    rmSync(fx, { recursive: true, force: true });
    rmSync(out, { recursive: true, force: true });
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);

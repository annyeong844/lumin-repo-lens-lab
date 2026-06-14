import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  assignSuitesToGroups,
  defaultJobs,
  formatReplayLines,
  groupForSuite,
  normalizeNodeExecutable,
  parseArgs,
} from "../scripts/run-tests-grouped.mjs";
import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const RUNNER = path.join(ROOT, "scripts", "run-tests-grouped.mjs");

let passed = 0;
let failed = 0;

function assert(label, ok, detail = "") {
  if (ok) {
    passed++;
    console.log(`  PASS  ${label}`);
  } else {
    failed++;
    console.log(`  FAIL  ${label}\n        ${detail}`);
  }
}

function writeSuite(fx, name, body = 'console.log("suite passed");\n') {
  fx.write(`tests/${name}`, body);
}

function runCli(args) {
  try {
    const stdout = execFileSync(process.execPath, [RUNNER, ...args], {
      cwd: ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
    return { ok: true, out: stdout };
  } catch (error) {
    return {
      ok: false,
      out: `${error.stdout?.toString() ?? ""}${error.stderr?.toString() ?? ""}`,
      status: error.status,
    };
  }
}

{
  assert(
    "RTG1. groupForSuite maps known risk families deterministically",
    groupForSuite("test-pre-write-render.mjs") === "pre-write" &&
      groupForSuite("test-resolver-paths.mjs") === "resolver" &&
      groupForSuite("test-symbol-graph-incremental.mjs") === "symbol-graph" &&
      groupForSuite("test-build-block-clone-index.mjs") === "function-clone" &&
      groupForSuite("test-unknown-new-suite.mjs") === "misc",
  );
}

{
  const suites = [
    "test-pre-write-render.mjs",
    "test-resolver-paths.mjs",
    "test-unknown-new-suite.mjs",
  ];
  const groups = assignSuitesToGroups(suites);
  const assigned = groups.flatMap((group) => group.suites).sort();
  assert(
    "RTG2. assignSuitesToGroups assigns every suite exactly once",
    JSON.stringify(assigned) === JSON.stringify([...suites].sort()) &&
      groups.some(
        (group) =>
          group.name === "misc" &&
          group.suites.includes("test-unknown-new-suite.mjs"),
      ),
    JSON.stringify(groups),
  );
}

{
  const parsed = parseArgs(
    ["--jobs", "2", "--group", "pre-write", "--serial"],
    {},
  );
  assert(
    "RTG3. parseArgs honors --group and makes --serial force one job",
    parsed.group === "pre-write" && parsed.jobs === 1 && parsed.serial === true,
    JSON.stringify(parsed),
  );
  assert(
    "RTG4. defaultJobs is bounded to three workers",
    defaultJobs(16) === 3 && defaultJobs(2) === 1 && defaultJobs(1) === 1,
  );
  assert(
    "RTG4a. --node command names keep PATH lookup while path-like overrides resolve",
    parseArgs(["--node", "node20"], {}).nodePath === "node20" &&
      normalizeNodeExecutable("./tools/node") ===
        path.resolve("./tools/node") &&
      normalizeNodeExecutable("node.exe") === "node.exe",
  );
}

{
  const replay = formatReplayLines(
    "pre-write",
    "test-pre-write-render.mjs",
  ).join("\n");
  assert(
    "RTG5. replay output includes group and exact suite commands",
    replay.includes("npm run test:node:groups -- --group pre-write --serial") &&
      replay.includes("node tests/test-pre-write-render.mjs"),
    replay,
  );
}

{
  const fx = createTempRepoFixture({ prefix: "lrl-test-runner-groups-list-" });
  try {
    writeSuite(fx, "test-pre-write-render.mjs");
    writeSuite(fx, "test-resolver-paths.mjs");
    writeSuite(fx, "test-z-new-suite.mjs");
    const result = runCli(["--tests-dir", fx.path("tests"), "--list-groups"]);
    assert(
      "RTG6. --list-groups prints deterministic discovered groups",
      result.ok &&
        result.out.includes("group pre-write 1 suites") &&
        result.out.includes("group resolver 1 suites") &&
        result.out.includes("group misc 1 suites"),
      result.out,
    );
  } finally {
    fx.cleanup();
  }
}

{
  const fx = createTempRepoFixture({ prefix: "lrl-test-runner-groups-pass-" });
  try {
    writeSuite(fx, "test-pre-write-render.mjs");
    writeSuite(fx, "test-resolver-paths.mjs");
    const result = runCli([
      "--tests-dir",
      fx.path("tests"),
      "--group",
      "pre-write",
      "--serial",
    ]);
    assert(
      "RTG7. --group runs only the selected group and keeps passing logs compact",
      result.ok &&
        result.out.includes("running 1 suites across 1 groups") &&
        result.out.includes("PASS pre-write 1 suites") &&
        !result.out.includes("resolver"),
      result.out,
    );
  } finally {
    fx.cleanup();
  }
}

{
  const fx = createTempRepoFixture({ prefix: "lrl-test-runner-groups-fail-" });
  try {
    writeSuite(
      fx,
      "test-pre-write-fail.mjs",
      'console.error("intentional failure");\nprocess.exit(7);\n',
    );
    const result = runCli([
      "--tests-dir",
      fx.path("tests"),
      "--group",
      "pre-write",
      "--serial",
    ]);
    assert(
      "RTG8. failed groups print buffered logs and replay commands",
      !result.ok &&
        result.out.includes("suite test-pre-write-fail.mjs exited 7") &&
        result.out.includes("intentional failure") &&
        result.out.includes(
          "replay group: npm run test:node:groups -- --group pre-write --serial",
        ) &&
        result.out.includes("replay suite: node tests/test-pre-write-fail.mjs"),
      result.out,
    );
  } finally {
    fx.cleanup();
  }
}

{
  const fx = createTempRepoFixture({ prefix: "lrl-test-runner-groups-spawn-" });
  try {
    writeSuite(fx, "test-pre-write-spawn.mjs");
    const missingNode = fx.path("missing-node-executable");
    const result = runCli([
      "--tests-dir",
      fx.path("tests"),
      "--group",
      "pre-write",
      "--node",
      missingNode,
    ]);
    assert(
      "RTG9. child process spawn errors name the affected suite",
      !result.ok &&
        result.out.includes(
          "failed to start test suite test-pre-write-spawn.mjs",
        ) &&
        result.out.includes(
          "replay suite: node tests/test-pre-write-spawn.mjs",
        ),
      result.out,
    );
  } finally {
    fx.cleanup();
  }
}

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);

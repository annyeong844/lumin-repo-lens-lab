#!/usr/bin/env node
// Opt-in grouped Node test runner.
//
// `npm test` remains the authoritative serial lane for default Node suites.
// This runner is a maintainer shortcut over that same default set: groups run
// in bounded parallel, while suites inside each group still run serially in
// fresh Node subprocesses.

import { spawn } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(__dirname, "..");
const TESTS_DIR = path.join(REPO, "tests");
const GROUP_RUNNER_LABEL = "[run-tests:groups]";
const LEGACY_NODE_SUITES = new Set(["test-audit-repo.mjs"]);

export const GROUP_RULES = [
  {
    name: "audit-repo",
    patterns: [/^test-audit-repo/, /^test-audit-manifest/],
  },
  { name: "pre-write", patterns: [/^test-pre-write/] },
  {
    name: "post-write",
    patterns: [/^test-post-write/, /^test-file-delta-export/],
  },
  {
    name: "resolver",
    patterns: [
      /^test-resolver/,
      /^test-wildcard/,
      /^test-tsconfig/,
      /^test-hash-imports/,
      /^test-node-imports/,
      /^test-output-source-layout/,
      /^test-import-meta-glob/,
    ],
  },
  {
    name: "symbol-graph",
    patterns: [
      /^test-symbol-graph/,
      /^test-namespace-reexport/,
      /^test-class-method/,
      /^test-extract-ts/,
      /^test-cjs/,
    ],
  },
  {
    name: "module-reachability",
    patterns: [
      /^test-module-reachability/,
      /^test-topology/,
      /^test-dynamic-import/,
      /^test-type-only/,
    ],
  },
  {
    name: "function-clone",
    patterns: [
      /^test-function-clone/,
      /^test-build-function-clone/,
      /^test-build-block-clone/,
    ],
  },
  { name: "shape-index", patterns: [/^test-shape/, /^test-build-shape-index/] },
  {
    name: "canon",
    patterns: [
      /^test-canon/,
      /^test-check-canon/,
      /^test-generate-canon/,
      /^test-classification/,
      /^test-definition-id/,
    ],
  },
  {
    name: "public-surface",
    patterns: [
      /^test-public/,
      /^test-skill/,
      /^test-plugin/,
      /^test-publish/,
      /^test-workspace/,
      /^test-mdx/,
      /^test-framework/,
    ],
  },
  {
    name: "wiki-docs",
    patterns: [
      /^test-update-test-doc/,
      /^test-maintainer-scripts/,
      /^test-evidence-honesty/,
    ],
  },
];

export const GROUP_NAMES = [...GROUP_RULES.map((rule) => rule.name), "misc"];

export function listSuites(testsDir = TESTS_DIR) {
  return readdirSync(testsDir)
    .filter(
      (file) =>
        file.startsWith("test-") &&
        file.endsWith(".mjs") &&
        !LEGACY_NODE_SUITES.has(file)
    )
    .sort();
}

export function groupForSuite(suite) {
  for (const rule of GROUP_RULES) {
    if (rule.patterns.some((pattern) => pattern.test(suite))) {
      return rule.name;
    }
  }
  return "misc";
}

export function assignSuitesToGroups(suites) {
  const grouped = new Map(GROUP_NAMES.map((name) => [name, []]));
  for (const suite of suites) {
    grouped.get(groupForSuite(suite)).push(suite);
  }
  return GROUP_NAMES.map((name) => ({
    name,
    suites: grouped.get(name),
  })).filter((group) => group.suites.length > 0);
}

export function defaultJobs(cpuCount = getCpuCount()) {
  const usableCpuCount = Number.isFinite(cpuCount)
    ? Math.max(1, Math.floor(cpuCount))
    : 1;
  return Math.max(1, Math.min(3, usableCpuCount - 1 || 1));
}

function getCpuCount() {
  if (typeof os.availableParallelism === "function") {
    return os.availableParallelism();
  }
  return os.cpus().length;
}

function requireValue(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

export function normalizeNodeExecutable(value) {
  if (
    path.isAbsolute(value) ||
    path.win32.isAbsolute(value) ||
    value.includes("/") ||
    value.includes("\\") ||
    value.startsWith(".") ||
    /^[A-Za-z]:/.test(value)
  ) {
    return path.resolve(value);
  }
  return value;
}

export function parseArgs(argv = process.argv.slice(2), env = process.env) {
  const options = {
    group: null,
    help: false,
    jobs: null,
    listGroups: false,
    nodePath: process.execPath,
    serial: false,
    testsDir: TESTS_DIR,
  };

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--help" || arg === "-h") {
      options.help = true;
    } else if (arg === "--jobs") {
      const raw = requireValue(argv, i, "--jobs");
      const parsed = Number.parseInt(raw, 10);
      if (!Number.isFinite(parsed) || parsed < 1) {
        throw new Error(`--jobs must be a positive integer: ${raw}`);
      }
      options.jobs = parsed;
      i++;
    } else if (arg === "--group") {
      options.group = requireValue(argv, i, "--group");
      i++;
    } else if (arg === "--serial") {
      options.serial = true;
    } else if (arg === "--list-groups") {
      options.listGroups = true;
    } else if (arg === "--tests-dir") {
      options.testsDir = path.resolve(requireValue(argv, i, "--tests-dir"));
      i++;
    } else if (arg === "--node") {
      options.nodePath = normalizeNodeExecutable(
        requireValue(argv, i, "--node"),
      );
      i++;
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  const envJobs = env.RUN_TESTS_GROUP_JOBS
    ? Number.parseInt(env.RUN_TESTS_GROUP_JOBS, 10)
    : null;
  if (options.serial) {
    options.jobs = 1;
  } else if (options.jobs == null && Number.isFinite(envJobs) && envJobs > 0) {
    options.jobs = envJobs;
  } else if (options.jobs == null) {
    options.jobs = defaultJobs();
  }

  return options;
}

export function formatReplayLines(groupName, suite) {
  return [
    `${GROUP_RUNNER_LABEL} replay group: npm run test:node:groups -- --group ${groupName} --serial`,
    `${GROUP_RUNNER_LABEL} replay suite: node tests/${suite}`,
  ];
}

function formatSeconds(ms) {
  return `${(ms / 1000).toFixed(1)}s`;
}

function appendOutput(parts, label, text) {
  if (!text) return;
  parts.push(`--- ${label} ---\n${text.endsWith("\n") ? text : `${text}\n`}`);
}

export async function runSuite(suite, options) {
  const suitePath = path.join(options.testsDir, suite);
  const started = Date.now();

  return new Promise((resolve) => {
    const child = spawn(options.nodePath, [suitePath], {
      cwd: options.repoRoot ?? REPO,
      env: process.env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const stdout = [];
    const stderr = [];
    let settled = false;

    function finish(result) {
      if (settled) return;
      settled = true;
      resolve({
        suite,
        durationMs: Date.now() - started,
        stdout: Buffer.concat(stdout).toString("utf8"),
        stderr: Buffer.concat(stderr).toString("utf8"),
        ...result,
      });
    }

    child.stdout.on("data", (chunk) => stdout.push(Buffer.from(chunk)));
    child.stderr.on("data", (chunk) => stderr.push(Buffer.from(chunk)));
    child.on("error", (error) => finish({ error, status: null, signal: null }));
    child.on("close", (status, signal) =>
      finish({ error: null, status, signal }),
    );
  });
}

export async function runGroup(group, options) {
  const started = Date.now();
  const logParts = [];
  const suiteResults = [];

  for (const suite of group.suites) {
    const result = await runSuite(suite, options);
    suiteResults.push(result);
    logParts.push(
      `${GROUP_RUNNER_LABEL} suite ${suite} ${formatSeconds(result.durationMs)}\n`,
    );
    appendOutput(logParts, `${suite} stdout`, result.stdout);
    appendOutput(logParts, `${suite} stderr`, result.stderr);

    if (result.error) {
      return {
        groupName: group.name,
        durationMs: Date.now() - started,
        failedSuite: suite,
        logs: logParts.join(""),
        ok: false,
        reason: `failed to start test suite ${suite}: ${result.error.message}`,
        suiteResults,
      };
    }
    if (result.status !== 0) {
      return {
        groupName: group.name,
        durationMs: Date.now() - started,
        failedSuite: suite,
        logs: logParts.join(""),
        ok: false,
        reason: `suite ${suite} exited ${result.status}${result.signal ? ` (${result.signal})` : ""}`,
        suiteResults,
      };
    }
  }

  return {
    groupName: group.name,
    durationMs: Date.now() - started,
    failedSuite: null,
    logs: logParts.join(""),
    ok: true,
    reason: null,
    suiteResults,
  };
}

export async function runGroups(groups, options) {
  const jobs = Math.max(1, Math.min(options.jobs, groups.length || 1));
  const results = [];
  let nextIndex = 0;
  let stopScheduling = false;

  async function worker() {
    while (!stopScheduling) {
      const index = nextIndex;
      nextIndex++;
      if (index >= groups.length) return;

      const group = groups[index];
      const result = await runGroup(group, options);
      results.push({ index, ...result });
      if (!result.ok) {
        stopScheduling = true;
      }
    }
  }

  await Promise.all(Array.from({ length: jobs }, () => worker()));
  return results.sort((a, b) => a.index - b.index);
}

export function renderGroupList(groups) {
  return groups.map(
    (group) =>
      `${GROUP_RUNNER_LABEL} group ${group.name} ${group.suites.length} suites`,
  );
}

function usage() {
  return [
    "Usage: npm run test:node:groups -- [options]",
    "",
    "Options:",
    "  --jobs <n>        Run up to n groups in parallel (default: min(3, cpu-1))",
    "  --group <name>    Run one group only",
    "  --serial          Alias for --jobs 1",
    "  --list-groups     Print discovered groups without running tests",
    "  --tests-dir <dir> Override test directory (used by runner tests)",
    "  --node <command-or-path>",
    "                   Override Node executable; command names use PATH lookup",
    "  --help            Show this message",
  ].join("\n");
}

function selectGroups(groups, groupName) {
  if (!groupName) return groups;
  if (!GROUP_NAMES.includes(groupName)) {
    throw new Error(
      `unknown group ${groupName}. Known groups: ${GROUP_NAMES.join(", ")}`,
    );
  }
  return groups.filter((group) => group.name === groupName);
}

export async function main(argv = process.argv.slice(2), io = process) {
  const options = parseArgs(argv);

  if (options.help) {
    io.stdout.write(`${usage()}\n`);
    return 0;
  }

  if (!existsSync(options.testsDir)) {
    throw new Error(`tests directory does not exist: ${options.testsDir}`);
  }

  const suites = listSuites(options.testsDir);
  const groups = selectGroups(assignSuitesToGroups(suites), options.group);

  if (options.listGroups) {
    io.stdout.write(`${renderGroupList(groups).join("\n")}\n`);
    return 0;
  }

  if (groups.length === 0) {
    io.stdout.write(`${GROUP_RUNNER_LABEL} 0 suites matched\n`);
    return 0;
  }

  const selectedSuiteCount = groups.reduce(
    (sum, group) => sum + group.suites.length,
    0,
  );
  io.stdout.write(
    `${GROUP_RUNNER_LABEL} running ${selectedSuiteCount} suites across ${groups.length} groups (jobs=${options.jobs})\n`,
  );
  const results = await runGroups(groups, options);
  const failed = results.find((result) => !result.ok);

  for (const result of results) {
    const suiteCount =
      groups.find((group) => group.name === result.groupName)?.suites.length ??
      0;
    const state = result.ok ? "PASS" : "FAIL";
    io.stdout.write(
      `${GROUP_RUNNER_LABEL} ${state} ${result.groupName} ${suiteCount} suites ${formatSeconds(result.durationMs)}\n`,
    );
  }

  if (failed) {
    io.stderr.write(`${GROUP_RUNNER_LABEL} ${failed.reason}\n`);
    io.stderr.write(`${failed.logs}`);
    for (const line of formatReplayLines(
      failed.groupName,
      failed.failedSuite,
    )) {
      io.stderr.write(`${line}\n`);
    }
    return 1;
  }

  const passedSuites = results.reduce(
    (sum, result) => sum + result.suiteResults.length,
    0,
  );
  io.stdout.write(
    `${GROUP_RUNNER_LABEL} ${passedSuites} suites passed across ${results.length} groups\n`,
  );
  return 0;
}

const isDirectRun =
  process.argv[1] &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;

if (isDirectRun) {
  try {
    const exitCode = await main();
    process.exit(exitCode);
  } catch (error) {
    process.stderr.write(`${GROUP_RUNNER_LABEL} ERROR: ${error.message}\n`);
    process.exit(1);
  }
}

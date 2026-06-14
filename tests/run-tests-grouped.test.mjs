import path from "node:path";

import { describe, expect, it } from "vitest";

import {
  assignSuitesToGroups,
  defaultJobs,
  formatReplayLines,
  groupForSuite,
  normalizeNodeExecutable,
  parseArgs,
  renderGroupList,
} from "../scripts/run-tests-grouped.mjs";

describe("grouped Node test runner helpers", () => {
  it("maps known suite families and defaults unknown suites to misc", () => {
    expect(groupForSuite("test-pre-write-render.mjs")).toBe("pre-write");
    expect(groupForSuite("test-resolver-paths.mjs")).toBe("resolver");
    expect(groupForSuite("test-symbol-graph-incremental.mjs")).toBe(
      "symbol-graph",
    );
    expect(groupForSuite("test-build-block-clone-index.mjs")).toBe(
      "function-clone",
    );
    expect(groupForSuite("test-unknown-new-suite.mjs")).toBe("misc");
  });

  it("assigns every suite exactly once", () => {
    const suites = [
      "test-pre-write-render.mjs",
      "test-resolver-paths.mjs",
      "test-unknown-new-suite.mjs",
    ];

    const groups = assignSuitesToGroups(suites);
    const assigned = groups.flatMap((group) => group.suites).sort();

    expect(assigned).toEqual([...suites].sort());
    expect(groups.find((group) => group.name === "misc")?.suites).toContain(
      "test-unknown-new-suite.mjs",
    );
  });

  it("parses group, serial, and bounded job options", () => {
    const parsed = parseArgs(
      ["--jobs", "2", "--group", "pre-write", "--serial"],
      {},
    );

    expect(parsed.group).toBe("pre-write");
    expect(parsed.jobs).toBe(1);
    expect(parsed.serial).toBe(true);
    expect(defaultJobs(16)).toBe(3);
    expect(defaultJobs(2)).toBe(1);
    expect(defaultJobs(1)).toBe(1);
    expect(parseArgs(["--node", "node20"], {}).nodePath).toBe("node20");
    expect(normalizeNodeExecutable("./tools/node")).toBe(
      path.resolve("./tools/node"),
    );
    expect(normalizeNodeExecutable("node.exe")).toBe("node.exe");
  });

  it("renders group lists and replay commands", () => {
    const groups = assignSuitesToGroups([
      "test-pre-write-render.mjs",
      "test-resolver-paths.mjs",
    ]);
    const groupList = renderGroupList(groups).join("\n");
    const replay = formatReplayLines(
      "pre-write",
      "test-pre-write-render.mjs",
    ).join("\n");

    expect(groupList).toContain("group pre-write 1 suites");
    expect(groupList).toContain("group resolver 1 suites");
    expect(replay).toContain(
      "npm run test:node:groups -- --group pre-write --serial",
    );
    expect(replay).toContain("node tests/test-pre-write-render.mjs");
  });
});

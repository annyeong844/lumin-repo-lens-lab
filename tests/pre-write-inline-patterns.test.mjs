// Mirrors tests/test-pre-write-inline-patterns.mjs.
//
// This suite protects pre-write inline extraction cues from explicit
// refactorSources plus inline-patterns.json. Repeated inline patterns remain
// review-only evidence and missing artifacts remain unavailable evidence.

import { execFileSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const PREWRITE = path.join(REPO_ROOT, "pre-write.mjs");

function makeFixture() {
  const dir = mkdtempSync(path.join(os.tmpdir(), "lumin-prewrite-inline-"));
  mkdirSync(path.join(dir, "src"), { recursive: true });
  writeFileSync(
    path.join(dir, "package.json"),
    JSON.stringify(
      {
        name: "inline-fixture",
        type: "module",
      },
      null,
      2,
    ),
  );
  writeFileSync(
    path.join(dir, "src", "server.ts"),
    `export function server(connection, payload) {
  try {
    writeWebSocketTextMessage(connection.socket, payload);
  } catch {
    connection.socket.destroy();
  }
  try {
    writeWebSocketTextMessage(connection.socket, payload);
  } catch {
    connection.socket.destroy();
  }
  try {
    writeWebSocketTextMessage(connection.socket, payload);
  } catch {
    connection.socket.destroy();
  }
  try {
    writeWebSocketTextMessage(connection.socket, payload);
  } catch {
    connection.socket.destroy();
  }
}
`,
  );
  return dir;
}

function writeIntent(dir) {
  const intentPath = path.join(dir, "intent.json");
  writeFileSync(
    intentPath,
    JSON.stringify(
      {
        names: ["writeOrDestroyConnection", "WriteOrDestroyResult"],
        shapes: [],
        files: ["src/connection-write.ts"],
        dependencies: [],
        plannedTypeEscapes: [],
        refactorSources: [
          {
            file: "src/server.ts",
            lines: [4, 9, 14, 19],
            why: "extract repeated catch-destroy handling",
          },
        ],
      },
      null,
      2,
    ),
  );
  return intentPath;
}

function runPreWrite({ root, out, intentPath, extraArgs = [] }) {
  return execFileSync(
    process.execPath,
    [
      PREWRITE,
      "--root",
      root,
      "--output",
      out,
      "--intent",
      intentPath,
      ...extraArgs,
    ],
    {
      cwd: REPO_ROOT,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
}

function readLatest(out) {
  return JSON.parse(
    readFileSync(path.join(out, "pre-write-advisory.latest.json"), "utf8"),
  );
}

it("surfaces repeated inline statement patterns as review-only pre-write cues", () => {
  const fx = makeFixture();
  const out = path.join(fx, ".audit");
  const intentPath = writeIntent(fx);
  try {
    const stdout = runPreWrite({ root: fx, out, intentPath });
    const latest = readLatest(out);
    const inlineCue = latest.cueCards
      .flatMap((card) => card.cues ?? [])
      .find((cue) => cue.evidenceLane === "inline-extraction");

    expect(existsSync(path.join(out, "inline-patterns.json"))).toBe(true);
    expect(inlineCue).toMatchObject({
      cueTier: "AGENT_REVIEW_CUE",
      claim: "repeated inline statement pattern",
    });
    expect(stdout).toContain("Agent review cues");
    expect(stdout).toContain("repeated inline statement pattern");
    expect(stdout).not.toContain("Safe to extract");
    expect(stdout).not.toContain("Duplicate behavior found");
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

it("keeps inline extraction unavailable when no-fresh pre-write lacks inline patterns", () => {
  const fx = makeFixture();
  const out = path.join(fx, ".audit");
  const intentPath = writeIntent(fx);
  try {
    const stdout = runPreWrite({
      root: fx,
      out,
      intentPath,
      extraArgs: ["--no-fresh-audit"],
    });
    const latest = readLatest(out);

    expect(latest.unavailableEvidence).toContainEqual(
      expect.objectContaining({
        evidenceLane: "inline-extraction",
        status: "UNAVAILABLE",
        artifact: "inline-patterns.json",
      }),
    );
    expect(
      latest.cueCards
        .flatMap((card) => card.cues ?? [])
        .some((cue) => cue.evidenceLane === "inline-extraction"),
    ).toBe(false);
    expect(stdout).not.toContain("repeated inline statement pattern");
  } finally {
    rmSync(fx, { recursive: true, force: true });
  }
});

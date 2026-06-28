import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { createTempRepoFixture } from "./_helpers/temp-repo-fixture.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");

function writeCorpus(fixture) {
  fixture.write("src/dup-strong-a.ts", "export type Result = { ok: true };\n");
  fixture.write(
    "src/dup-strong-b.ts",
    "export type Result = { ok: false; reason: string };\n",
  );
  for (let i = 1; i <= 3; i++) {
    fixture.write(
      `src/use-result-a${i}.ts`,
      `import { Result } from './dup-strong-a';\nexport const resultA${i}: Result = { ok: true };\n`,
    );
    fixture.write(
      `src/use-result-b${i}.ts`,
      `import { Result } from './dup-strong-b';\nexport const resultB${i}: Result = { ok: false, reason: '${i}' };\n`,
    );
  }

  fixture.write(
    "src/card-props.ts",
    "export interface Props { cardId: string }\n",
  );
  fixture.write(
    "src/dialog-props.ts",
    "export interface Props { open: boolean }\n",
  );
  fixture.write(
    "src/use-card-props.ts",
    "import { Props } from './card-props';\nexport const cardProps: Props = { cardId: 'c' };\n",
  );
  fixture.write(
    "src/use-dialog-props.ts",
    "import { Props } from './dialog-props';\nexport const dialogProps: Props = { open: true };\n",
  );

  fixture.write(
    "src/api-envelope.ts",
    "export type Envelope = { id: string };\n",
  );
  fixture.write(
    "src/ui-envelope.ts",
    "export type Envelope = { id: string; title: string };\n",
  );
  fixture.write(
    "src/use-api-envelope.ts",
    "import { Envelope } from './api-envelope';\nexport const apiEnvelope: Envelope = { id: 'a' };\n",
  );
  fixture.write(
    "src/use-ui-envelope.ts",
    "import { Envelope } from './ui-envelope';\nexport const uiEnvelope: Envelope = { id: 'u', title: 't' };\n",
  );

  fixture.write(
    "src/opaque-a.ts",
    "export type Opaque = { a: any; b: any; c: any };\n",
  );
  fixture.write(
    "src/opaque-b.ts",
    "export type Opaque = { left: any; right: any; tag: any };\n",
  );

  fixture.write("src/session.ts", "export interface Session { id: string }\n");
  for (let i = 1; i <= 3; i++) {
    fixture.write(
      `src/use-session-${i}.ts`,
      `import { Session } from './session';\nexport const session${i}: Session = { id: '${i}' };\n`,
    );
  }
}

function parseTypeOwnershipRows(md) {
  const lines = md.split("\n");
  const start = lines.findIndex((line) => line.startsWith("| Name | Identity"));
  if (start < 0) return [];
  const headerCells = lines[start]
    .split("|")
    .slice(1, -1)
    .map((cell) => cell.trim());
  const index = Object.fromEntries(headerCells.map((cell, i) => [cell, i]));
  const rows = [];
  for (let i = start + 2; i < lines.length; i++) {
    const line = lines[i];
    if (!line.startsWith("|")) break;
    const cells = line
      .split("|")
      .slice(1, -1)
      .map((cell) => cell.trim());
    if (cells.length < 6) continue;
    rows.push({
      name: cells[0].replace(/^`|`$/g, ""),
      identity: cells[1].replace(/^`|`$/g, ""),
      owner: cells[2].replace(/^`|`$/g, ""),
      fanIn: Number(cells[index["Fan-in"]]),
      status: cells[index.Status],
      tags: cells[index.Tags],
    });
  }
  return rows;
}

function rowsFor(rows, name) {
  return rows.filter((row) => row.name === name);
}

function firstStatus(row) {
  return row.status.split(/\s+/)[0];
}

function expectUniformLabel(rows, name, expected, expectedCount) {
  const hits = rowsFor(rows, name);
  const statuses = new Set(hits.map(firstStatus));
  expect(hits).toHaveLength(expectedCount);
  expect([...statuses]).toEqual([expected]);
  return hits;
}

function runCorpus() {
  const fixture = createTempRepoFixture({
    prefix: "vitest-label-emission-corpus-",
    packageJson: {
      name: "label-emission-corpus",
      type: "module",
      private: true,
    },
  });
  try {
    writeCorpus(fixture);
    execFileSync(
      process.execPath,
      [
        "build-symbol-graph.mjs",
        "--root",
        fixture.root,
        "--output",
        fixture.output,
      ],
      { cwd: REPO_ROOT, stdio: ["ignore", "pipe", "pipe"] },
    );
    execFileSync(
      process.execPath,
      [
        "generate-canon-draft.mjs",
        "--root",
        fixture.root,
        "--output",
        fixture.output,
        "--source",
        "type-ownership",
      ],
      { cwd: REPO_ROOT, stdio: ["ignore", "pipe", "pipe"] },
    );

    const symbols = fixture.readJson("symbols.json", { from: "output" });
    const md = readFileSync(
      path.join(fixture.root, "canonical-draft/type-ownership.md"),
      "utf8",
    );
    return { rows: parseTypeOwnershipRows(md), symbols };
  } finally {
    fixture.cleanup();
  }
}

describe("classification label emission corpus", () => {
  it("C1-C2. emits any-contamination support and a parseable type ownership table", () => {
    const { rows, symbols } = runCorpus();

    expect(symbols.meta?.supports?.anyContamination).toBe(true);
    expect(rows.length).toBeGreaterThan(0);
  });

  it("C3. emits DUPLICATE_STRONG for duplicate Result owners with real fan-in", () => {
    const { rows } = runCorpus();
    const resultRows = expectUniformLabel(
      rows,
      "Result",
      "DUPLICATE_STRONG",
      2,
    );

    expect(resultRows.every((row) => row.fanIn >= 3)).toBe(true);
  });

  it("emits LOCAL_COMMON_NAME for low-info local Props owners", () => {
    const { rows } = runCorpus();

    expectUniformLabel(rows, "Props", "LOCAL_COMMON_NAME", 2);
  });

  it("emits DUPLICATE_REVIEW for duplicate non-low-info Envelope owners", () => {
    const { rows } = runCorpus();

    expectUniformLabel(rows, "Envelope", "DUPLICATE_REVIEW", 2);
  });

  it("C4. emits ANY_COLLISION with contamination tags for Opaque owners", () => {
    const { rows } = runCorpus();
    const opaqueRows = expectUniformLabel(rows, "Opaque", "ANY_COLLISION", 2);

    expect(
      opaqueRows.every((row) =>
        /contamination:severely-any-contaminated/.test(row.tags),
      ),
    ).toBe(true);
  });

  it("C5. emits single-owner-strong for Session with three real consumers", () => {
    const { rows } = runCorpus();
    const sessionRows = expectUniformLabel(
      rows,
      "Session",
      "single-owner-strong",
      1,
    );

    expect(sessionRows[0]?.fanIn).toBe(3);
  });
});

import { execFileSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

const ROOT = path.resolve(import.meta.dirname, "..");
const NODE = process.execPath;
const PREWRITE = path.join(ROOT, "pre-write.mjs");

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

function runIntegrationFixture() {
  const fixtureRoot = mkdtempSync(path.join(tmpdir(), "pw-integration-"));
  const output = mkdtempSync(path.join(tmpdir(), "pw-integration-out-"));

  write(
    fixtureRoot,
    "package.json",
    JSON.stringify({ name: "pw-integration", type: "module" }),
  );
  write(
    fixtureRoot,
    "src/utils/date.ts",
    "export const formatDate = (d) => d.toString();\n",
  );
  write(
    fixtureRoot,
    "src/app.tsx",
    "import { formatDate } from './utils/date';\n" +
      "export const App = () => formatDate(new Date());\n",
  );
  write(
    fixtureRoot,
    "canonical/type-ownership.md",
    "# canonical/type-ownership.md - DRAFT\n" +
      "\n" +
      "> **Status:** draft, v1\n" +
      "> **Generated:** 2026-04-20T00:00:00Z\n" +
      "\n" +
      "### 2.1 Single owner (strong)\n" +
      "\n" +
      "| Type | Owner | Kind | Line | Fan-in | Status |\n" +
      "|---|---|---|---|---:|---|\n" +
      "| `GoneType` | `src/types/gone.ts` | TSTypeAliasDeclaration | 7 | 0 | ok |\n",
  );

  const intentPath = path.join(output, "intent.json");
  writeFileSync(
    intentPath,
    JSON.stringify({
      names: ["GoneType", "formatDate", "formatTimestamp"],
      shapes: [{ fields: ["year", "month"] }],
      files: ["src/utils/new-helper.ts"],
      dependencies: ["dayjs"],
      plannedTypeEscapes: [
        {
          escapeKind: "as-any",
          locationHint: "src/x.ts::fn",
          reason: "integration test",
        },
      ],
    }),
  );

  const stdout = execFileSync(
    NODE,
    [
      PREWRITE,
      "--root",
      fixtureRoot,
      "--output",
      output,
      "--intent",
      intentPath,
    ],
    { stdio: ["ignore", "pipe", "pipe"], encoding: "utf8" },
  );
  const advisory = JSON.parse(
    readFileSync(path.join(output, "pre-write-advisory.latest.json"), "utf8"),
  );

  return {
    fixtureRoot,
    output,
    stdout,
    advisory,
    cleanup() {
      rmSync(fixtureRoot, { recursive: true, force: true });
      rmSync(output, { recursive: true, force: true });
    },
  };
}

describe("direct pre-write integration lifecycle", () => {
  it("renders the expected advisory sections and grounded/new evidence", () => {
    const run = runIntegrationFixture();
    try {
      expect(run.stdout.length).toBeGreaterThan(0);
      for (const section of [
        "### Grounded facts",
        "### Agent review cues",
        "### Unavailable evidence",
        "### Already exists (reuse candidates)",
        "### New code candidates",
        "### Canonical drift",
        "### Planned type escapes (from Step 2 intent)",
      ]) {
        expect(run.stdout).toContain(section);
      }

      expect(run.stdout).toContain("EXISTS");
      expect(run.stdout).toContain("src/utils/date.ts::formatDate");

      const reviewCueSection = run.stdout.slice(
        run.stdout.indexOf("### Agent review cues"),
      );
      expect(reviewCueSection).toContain("near exported name");
      expect(reviewCueSection).toContain("formatDate");

      const unavailableSection = run.stdout.slice(
        run.stdout.indexOf("### Unavailable evidence"),
      );
      expect(unavailableSection).toContain("shape-hash");
      expect(unavailableSection).toContain("P4");

      const newCodeSection = run.stdout.slice(
        run.stdout.indexOf("### New code candidates"),
      );
      expect(newCodeSection).toContain("NEW_FILE");
      expect(newCodeSection).toContain("src/utils/new-helper.ts");
      expect(newCodeSection).toContain("NEW_PACKAGE");
      expect(newCodeSection).toContain("dayjs");

      const plannedSection = run.stdout.slice(
        run.stdout.indexOf("### Planned type escapes"),
      );
      expect(plannedSection).toContain("as-any");
      expect(plannedSection).toContain("integration test");
    } finally {
      run.cleanup();
    }
  });

  it("keeps canonical absent evidence and drift in their separate sections", () => {
    const run = runIntegrationFixture();
    try {
      const alreadyExistsSection = run.stdout.slice(
        run.stdout.indexOf("### Already exists (reuse candidates)"),
        run.stdout.indexOf(
          "###",
          run.stdout.indexOf("### Already exists (reuse candidates)") + 5,
        ),
      );
      expect(alreadyExistsSection).toContain("GoneType");
      expect(alreadyExistsSection).toContain("CANONICAL_EXISTS_AST_ABSENT");
      expect(alreadyExistsSection).not.toContain("CANONICAL DRIFT:");

      const driftSection = run.stdout.slice(
        run.stdout.indexOf("### Canonical drift"),
      );
      expect(driftSection.match(/CANONICAL DRIFT:/g) ?? []).toHaveLength(1);
      expect(driftSection).toContain("GoneType");
    } finally {
      run.cleanup();
    }
  });

  it("round-trips lookup ordering, drift, and capability evidence in advisory JSON", () => {
    const run = runIntegrationFixture();
    try {
      const kinds = run.advisory.lookups.map((lookup) => lookup.kind);
      const nameIndex = kinds.indexOf("name");
      const fileIndex = kinds.indexOf("file");
      const depIndex = kinds.indexOf("dependency");
      const shapeIndex = kinds.indexOf("shape");
      expect(nameIndex).toBeLessThan(fileIndex);
      expect(fileIndex).toBeLessThan(depIndex);
      expect(depIndex).toBeLessThan(shapeIndex);

      expect(run.advisory.drift).toHaveLength(1);
      expect(run.advisory.drift[0].intentName).toBe("GoneType");
      expect(run.advisory.capabilities.identityFanIn).toBe(true);
    } finally {
      run.cleanup();
    }
  });
});

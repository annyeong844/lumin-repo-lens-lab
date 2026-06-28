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
import { describe, expect, it } from "vitest";

const ROOT = path.resolve(import.meta.dirname, "..");
const NODE = process.execPath;
const BUILD_SHAPE_INDEX = path.join(ROOT, "build-shape-index.mjs");
const PRE_WRITE = path.join(ROOT, "pre-write.mjs");

function write(root, rel, content) {
  const full = path.join(root, rel);
  mkdirSync(path.dirname(full), { recursive: true });
  writeFileSync(full, content);
}

describe("pre-write shape-index integration", () => {
  it("renders and records grounded exact shape matches from shape-index.json", () => {
    const fixture = mkdtempSync(path.join(tmpdir(), "pw-shape-index-"));
    const output = mkdtempSync(path.join(tmpdir(), "pw-shape-index-out-"));

    try {
      write(
        fixture,
        "package.json",
        JSON.stringify({ name: "shape-fixture", type: "module" }),
      );
      write(
        fixture,
        "src/a.ts",
        "export interface CalendarA { year: number; month: number }\n",
      );
      write(
        fixture,
        "src/b.ts",
        "export type CalendarB = { month: number; year: number };\n",
      );

      execFileSync(
        NODE,
        [BUILD_SHAPE_INDEX, "--root", fixture, "--output", output],
        {
          stdio: ["ignore", "pipe", "pipe"],
        },
      );
      const shapeIndex = JSON.parse(
        readFileSync(path.join(output, "shape-index.json"), "utf8"),
      );
      const hash = shapeIndex.facts.find(
        (fact) => fact.exportedName === "CalendarA",
      )?.hash;

      const intentPath = path.join(output, "intent.json");
      writeFileSync(
        intentPath,
        JSON.stringify({
          names: [],
          shapes: [
            { fields: [], typeLiteral: "{ month: number; year: number }" },
          ],
          files: [],
          dependencies: [],
          plannedTypeEscapes: [],
        }),
      );

      const stdout = execFileSync(
        NODE,
        [
          PRE_WRITE,
          "--root",
          fixture,
          "--output",
          output,
          "--intent",
          intentPath,
          "--no-fresh-audit",
        ],
        { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] },
      );

      expect(stdout).toContain("### Grounded facts");
      expect(stdout).toContain("same normalized type shape");
      expect(stdout).toContain("shape-index.json");
      expect(stdout).toContain("src/a.ts::CalendarA");
      expect(stdout).toContain("src/b.ts::CalendarB");

      const latestPath = path.join(output, "pre-write-advisory.latest.json");
      expect(existsSync(latestPath)).toBe(true);
      const latest = JSON.parse(readFileSync(latestPath, "utf8"));
      const shapeLookup = latest.lookups.find(
        (lookup) => lookup.kind === "shape",
      );

      expect(shapeLookup.result).toBe("SHAPE_MATCH");
      expect(shapeLookup.matches).toHaveLength(2);
      expect(shapeLookup.matches.every((match) => match.hash === hash)).toBe(
        true,
      );
      expect(shapeLookup.shapeHash).toBe(hash);
      expect(shapeLookup.shapeHashSource).toBe("typeLiteral");
    } finally {
      rmSync(fixture, { recursive: true, force: true });
      rmSync(output, { recursive: true, force: true });
    }
  }, 20_000);
});

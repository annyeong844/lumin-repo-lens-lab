import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "..");
const factModelText = readFileSync(
  path.join(ROOT, "canonical", "fact-model.md"),
  "utf8",
);

const EXPECTED_ESCAPE_KINDS = [
  "explicit-any",
  "as-any",
  "angle-any",
  "as-unknown-as-T",
  "rest-any-args",
  "index-sig-any",
  "generic-default-any",
  "ts-ignore",
  "ts-expect-error",
  "no-explicit-any-disable",
  "jsdoc-any",
];

function extractEscapeKindsFromMarkdown(text) {
  const marker = "`escapeKind` is one of:";
  const idx = text.indexOf(marker);
  if (idx < 0) return null;
  const rest = text.slice(idx);
  const lines = rest.split("\n");
  const kinds = [];
  for (let i = 1; i < lines.length; i += 1) {
    const line = lines[i];
    const match = line.match(/^-\s+`([^`]+)`\s*—/);
    if (match) {
      kinds.push(match[1]);
    } else if (kinds.length > 0 && line.trim().length === 0) {
      break;
    }
  }
  return kinds;
}

describe("canonical fact-model type escape drift", () => {
  it("T1-T3. documents the canonical escapeKind list and order", () => {
    const parsed = extractEscapeKindsFromMarkdown(factModelText);

    expect(parsed).not.toBeNull();
    expect(parsed).toHaveLength(11);
    expect(parsed).toEqual(EXPECTED_ESCAPE_KINDS);
  });

  it("T4-T7. documents occurrence and normalized-code-shape fields", () => {
    expect(factModelText).toMatch(/"occurrenceKey"\s*:/);
    expect(factModelText).toMatch(/"normalizedCodeShape"\s*:/);
    expect(factModelText).toSatisfy(
      (text) =>
        /normalizedCodeShape.*token-aware|token-aware.*normalizedCodeShape/is.test(
          text,
        ) || /normalizedCodeShape[\s\S]{0,400}string.*literal/i.test(text),
      "normalizedCodeShape normalization rule should be documented",
    );
    expect(factModelText).toMatch(/occurrenceKey[\s\S]{0,400}sha256/i);
    expect(factModelText).toMatch(
      /occurrenceKey[\s\S]{0,600}file.*escapeKind.*normalizedCodeShape.*insideExportedIdentity/is,
    );
  });

  it("T8-T12. documents escape-kind precedence rules", () => {
    const precedencePairs = [
      ["rest-any-args", "explicit-any"],
      ["index-sig-any", "explicit-any"],
      ["generic-default-any", "explicit-any"],
      ["angle-any", "explicit-any"],
      ["as-unknown-as-T", "as-any"],
    ];

    for (const [winner, over] of precedencePairs) {
      expect(factModelText).toMatch(
        new RegExp(`\`${winner}\`\\s+wins over\\s+\`${over}\``, "i"),
      );
    }
  });

  it("T13. documents the P2-0 amendment date", () => {
    expect(factModelText).toMatch(/P2-0 amendment.*2026-04-20/i);
  });

  it("T14. keeps PLANNED_ESCAPE_KEYS and validator normalization synchronized", async () => {
    const { PLANNED_ESCAPE_KEYS, PLANNED_ESCAPE_ALL_KEYS, validateIntent } =
      await import("../_lib/pre-write-intent.mjs");

    expect([...PLANNED_ESCAPE_KEYS.required]).toEqual([
      "escapeKind",
      "locationHint",
      "reason",
    ]);
    expect([...PLANNED_ESCAPE_KEYS.optional]).toEqual([
      "codeShape",
      "alternativeConsidered",
    ]);
    expect(PLANNED_ESCAPE_ALL_KEYS).toHaveLength(5);

    const sample = {
      names: [],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [
        {
          escapeKind: "as-any",
          locationHint: "src/x.ts::foo",
          reason: "test",
          codeShape: "x as any",
          alternativeConsidered: "narrow",
        },
      ],
    };

    const validated = validateIntent(sample);

    expect(validated.ok).toBe(true);
    expect(
      PLANNED_ESCAPE_ALL_KEYS.every(
        (key) => key in validated.intent.plannedTypeEscapes[0],
      ),
    ).toBe(true);
  });
});

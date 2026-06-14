import {
  existsSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";

import {
  generateInvocationId,
  hashIntent,
  writeAdvisory,
} from "../_lib/pre-write-artifact.mjs";

function withTempDir(prefix, fn) {
  const dir = mkdtempSync(path.join(tmpdir(), prefix));
  try {
    return fn(dir);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

describe("pre-write advisory artifact identity and writes", () => {
  it("generates timestamped invocation ids with random suffixes", () => {
    const id = generateInvocationId();
    expect(id).toEqual(expect.any(String));
    expect(id).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}Z-[a-z0-9]{6}$/);

    expect(generateInvocationId()).not.toBe(generateInvocationId());
  });

  it("hashes intent deterministically after recursive key normalization", () => {
    const intent = {
      names: ["formatDate"],
      shapes: [],
      files: [],
      dependencies: [],
      plannedTypeEscapes: [],
    };
    const h1 = hashIntent(intent);
    expect(hashIntent(intent)).toBe(h1);
    expect(h1).toMatch(/^[a-f0-9]{64}$/);

    expect(
      hashIntent({
        names: ["x"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      }),
    ).toBe(
      hashIntent({
        plannedTypeEscapes: [],
        dependencies: [],
        files: [],
        shapes: [],
        names: ["x"],
      }),
    );

    expect(
      hashIntent({
        names: [],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [
          { escapeKind: "as-any", locationHint: "x", reason: "y" },
        ],
      }),
    ).toBe(
      hashIntent({
        names: [],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [
          { reason: "y", locationHint: "x", escapeKind: "as-any" },
        ],
      }),
    );

    expect(
      hashIntent({
        names: ["foo"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      }),
    ).not.toBe(
      hashIntent({
        names: ["bar"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      }),
    );
  });

  it("writes latest and invocation-specific advisory files with identical bytes", () => {
    withTempDir("pw-artifact-", (dir) => {
      const invocationId = "2026-04-20T12-30-00Z-abc123";
      writeAdvisory(dir, {
        invocationId,
        intentHash: "dummy-hash",
        intent: {
          names: ["x"],
          shapes: [],
          files: [],
          dependencies: [],
          plannedTypeEscapes: [],
        },
        lookups: [],
        boundaryChecks: [],
        drift: [],
        capabilities: {
          anyContamination: false,
          identityFanIn: true,
          reExportRecords: "file-level",
        },
        failures: [],
      });

      const latest = path.join(dir, "pre-write-advisory.latest.json");
      const specific = path.join(
        dir,
        `pre-write-advisory.${invocationId}.json`,
      );
      expect(existsSync(latest)).toBe(true);
      expect(existsSync(specific)).toBe(true);

      const latestText = readFileSync(latest, "utf8");
      expect(readFileSync(specific, "utf8")).toBe(latestText);

      const parsed = JSON.parse(latestText);
      expect(parsed.invocationId).toBe(invocationId);
      expect(parsed.capabilities.identityFanIn).toBe(true);
    });
  });

  it("leaves no temp files and preserves invocation-specific files across reruns", () => {
    withTempDir("pw-atomic-", (dir) => {
      writeAdvisory(dir, {
        invocationId: "2026-04-20T12-31-00Z-def456",
        intentHash: "h",
        intent: {},
        lookups: [],
        boundaryChecks: [],
        drift: [],
        capabilities: null,
        failures: [],
      });

      const names = readdirSync(dir);
      expect(
        names.filter(
          (name) =>
            name.startsWith(".") ||
            name.endsWith(".tmp") ||
            name.includes(".tmp."),
        ),
      ).toEqual([]);
      expect(
        names.filter((name) => name.startsWith("pre-write-advisory.")),
      ).toHaveLength(2);
    });

    withTempDir("pw-multi-", (dir) => {
      const id1 = "2026-04-20T12-32-00Z-aaa111";
      const id2 = "2026-04-20T12-33-00Z-bbb222";
      writeAdvisory(dir, {
        invocationId: id1,
        intentHash: "h1",
        intent: { names: ["first"] },
        lookups: [],
        boundaryChecks: [],
        drift: [],
        capabilities: null,
        failures: [],
      });
      writeAdvisory(dir, {
        invocationId: id2,
        intentHash: "h2",
        intent: { names: ["second"] },
        lookups: [],
        boundaryChecks: [],
        drift: [],
        capabilities: null,
        failures: [],
      });

      const latest = JSON.parse(
        readFileSync(path.join(dir, "pre-write-advisory.latest.json"), "utf8"),
      );
      expect(latest).toMatchObject({ invocationId: id2, intentHash: "h2" });
      expect(existsSync(path.join(dir, `pre-write-advisory.${id1}.json`))).toBe(
        true,
      );
      expect(existsSync(path.join(dir, `pre-write-advisory.${id2}.json`))).toBe(
        true,
      );
    });
  });

  it("round-trips capabilities-missing failures and the direct intent hash", () => {
    withTempDir("pw-caps-missing-", (dir) => {
      writeAdvisory(dir, {
        invocationId: "2026-04-20T12-34-00Z-ccc333",
        intentHash: "h",
        intent: {},
        lookups: [],
        boundaryChecks: [],
        drift: [],
        capabilities: null,
        failures: [
          {
            kind: "capabilities-missing",
            reason: "symbols.meta.supports not found in symbols.json",
          },
        ],
      });
      const parsed = JSON.parse(
        readFileSync(path.join(dir, "pre-write-advisory.latest.json"), "utf8"),
      );
      expect(parsed.capabilities).toBeNull();
      expect(parsed.failures).toContainEqual(
        expect.objectContaining({ kind: "capabilities-missing" }),
      );
    });

    withTempDir("pw-hash-roundtrip-", (dir) => {
      const intent = {
        names: ["formatDate"],
        shapes: [],
        files: [],
        dependencies: [],
        plannedTypeEscapes: [],
      };
      const intentHash = hashIntent(intent);
      writeAdvisory(dir, {
        invocationId: "2026-04-20T12-35-00Z-ddd444",
        intentHash,
        intent,
        lookups: [],
        boundaryChecks: [],
        drift: [],
        capabilities: null,
        failures: [],
      });
      const parsed = JSON.parse(
        readFileSync(path.join(dir, "pre-write-advisory.latest.json"), "utf8"),
      );
      expect(parsed.intentHash).toBe(intentHash);
      expect(hashIntent(parsed.intent)).toBe(intentHash);
    });
  });
});

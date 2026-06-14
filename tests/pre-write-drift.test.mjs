import { readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import { computeDrift } from "../_lib/pre-write-drift.mjs";

function canonicalClaim({
  name,
  ownerFile,
  line = 42,
  file = "canonical/type-ownership.md",
  section = "Single owner (strong)",
} = {}) {
  return { name, ownerFile, line, file, section };
}

function nameLookup({
  intentName,
  canonicalAstStatus,
  identities = [],
  canonicalClaim = null,
}) {
  return {
    kind: "name",
    intentName,
    result: "EXISTS",
    identities,
    canonicalClaim,
    canonicalAstStatus,
    nearNames: [],
    citations: [],
  };
}

function identity(ownerFile, exportedName) {
  return {
    identity: `${ownerFile}::${exportedName}`,
    ownerFile,
    exportedName,
    fanIn: 1,
    fanInConfidence: "grounded",
    anyContamination: { state: "clean" },
    resolverConfidence: "high",
    citations: [],
  };
}

describe("pre-write canonical drift projection", () => {
  it("returns empty drift for empty, aligned, not-consulted, extra-owner, and non-name states", () => {
    expect(computeDrift({ canonicalClaims: [], lookups: [] })).toEqual([]);

    const alignedClaim = canonicalClaim({
      name: "SessionId",
      ownerFile: "src/protocol/ids.ts",
    });
    expect(
      computeDrift({
        canonicalClaims: [alignedClaim],
        lookups: [
          nameLookup({
            intentName: "SessionId",
            canonicalAstStatus: "aligned",
            identities: [identity("src/protocol/ids.ts", "SessionId")],
            canonicalClaim: alignedClaim,
          }),
        ],
      }),
    ).toEqual([]);

    expect(
      computeDrift({
        canonicalClaims: [],
        lookups: [
          nameLookup({
            intentName: "anything",
            canonicalAstStatus: "not-consulted",
            identities: [identity("src/x.ts", "anything")],
          }),
        ],
      }),
    ).toEqual([]);

    const userClaim = canonicalClaim({
      name: "User",
      ownerFile: "src/models/User.ts",
    });
    expect(
      computeDrift({
        canonicalClaims: [userClaim],
        lookups: [
          nameLookup({
            intentName: "User",
            canonicalAstStatus: "aligned",
            identities: [
              identity("src/models/User.ts", "User"),
              identity("apps/legacy/user.ts", "User"),
            ],
            canonicalClaim: userClaim,
          }),
        ],
      }),
    ).toEqual([]);

    expect(
      computeDrift({
        canonicalClaims: [canonicalClaim({ name: "X", ownerFile: "src/x.ts" })],
        lookups: [
          { kind: "file", intentFile: "src/y.ts", result: "NEW_FILE" },
          { kind: "dependency", depName: "dayjs", result: "NEW_PACKAGE" },
          { kind: "shape", shape: { fields: ["a"] }, result: "UNAVAILABLE" },
        ],
      }),
    ).toEqual([]);
  });

  it("records owner-disagrees drift with canonical and AST owners", () => {
    const claim = canonicalClaim({
      name: "User",
      ownerFile: "src/models/User.ts",
      line: 42,
    });
    const drift = computeDrift({
      canonicalClaims: [claim],
      lookups: [
        nameLookup({
          intentName: "User",
          canonicalAstStatus: "owner-disagrees",
          identities: [identity("apps/legacy/user.ts", "User")],
          canonicalClaim: claim,
        }),
      ],
    });

    expect(drift).toHaveLength(1);
    expect(drift[0]).toMatchObject({
      kind: "owner-disagrees",
      canonicalOwner: "src/models/User.ts",
      canonicalLine: 42,
      intentName: "User",
      canonicalFile: "canonical/type-ownership.md",
    });
    expect(drift[0].astOwners).toEqual(["apps/legacy/user.ts"]);
  });

  it("records ast-absent drift with an empty AST owner list", () => {
    const claim = canonicalClaim({
      name: "GoneType",
      ownerFile: "src/types/gone.ts",
      line: 7,
    });
    const drift = computeDrift({
      canonicalClaims: [claim],
      lookups: [
        nameLookup({
          intentName: "GoneType",
          canonicalAstStatus: "ast-absent",
          identities: [],
          canonicalClaim: claim,
        }),
      ],
    });

    expect(drift).toHaveLength(1);
    expect(drift[0]).toMatchObject({
      kind: "ast-absent",
      canonicalOwner: "src/types/gone.ts",
    });
    expect(drift[0].astOwners).toEqual([]);
  });

  it("emits one entry per disagreeing intent and keeps aligned names out", () => {
    const claims = [
      canonicalClaim({ name: "Aligned", ownerFile: "src/aligned.ts" }),
      canonicalClaim({ name: "Disagree", ownerFile: "src/disagree.ts" }),
      canonicalClaim({ name: "Absent", ownerFile: "src/absent.ts" }),
    ];
    const drift = computeDrift({
      canonicalClaims: claims,
      lookups: [
        nameLookup({
          intentName: "Aligned",
          canonicalAstStatus: "aligned",
          identities: [identity("src/aligned.ts", "Aligned")],
          canonicalClaim: claims[0],
        }),
        nameLookup({
          intentName: "Disagree",
          canonicalAstStatus: "owner-disagrees",
          identities: [identity("src/other.ts", "Disagree")],
          canonicalClaim: claims[1],
        }),
        nameLookup({
          intentName: "Absent",
          canonicalAstStatus: "ast-absent",
          identities: [],
          canonicalClaim: claims[2],
        }),
      ],
    });

    expect(drift.map((entry) => entry.kind).sort()).toEqual([
      "ast-absent",
      "owner-disagrees",
    ]);
    expect(drift).not.toEqual(
      expect.arrayContaining([
        expect.objectContaining({ intentName: "Aligned" }),
      ]),
    );
  });

  it("does not duplicate drift for multiple disagreeing AST owners", () => {
    const claim = canonicalClaim({ name: "X", ownerFile: "src/x.ts" });
    const drift = computeDrift({
      canonicalClaims: [claim],
      lookups: [
        nameLookup({
          intentName: "X",
          canonicalAstStatus: "owner-disagrees",
          identities: [
            identity("apps/x1.ts", "X"),
            identity("apps/x2.ts", "X"),
          ],
          canonicalClaim: claim,
        }),
      ],
    });

    expect(drift).toHaveLength(1);
    expect(drift[0].astOwners).toEqual(["apps/x1.ts", "apps/x2.ts"]);
  });

  it("stays a read-only projection module", () => {
    const src = readFileSync(
      path.join(import.meta.dirname, "..", "_lib", "pre-write-drift.mjs"),
      "utf8",
    );

    expect(src).not.toMatch(/from\s+['"][^'"]*pre-write-canonical-parser/);
    expect(src).not.toMatch(/from\s+['"][^'"]*pre-write-lookup-name/);
    expect(src).not.toMatch(/\breadFileSync\b|\bexistsSync\b|\breadSync\b/);
  });
});

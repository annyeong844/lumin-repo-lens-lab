import { describe, expect, it } from "vitest";

import { lookupName } from "../_lib/pre-write-lookup-name.mjs";

function buildSymbols({
  identitiesByFile = {},
  fanInByIdentity = {},
  fanInByIdentitySpace = {},
  topSymbolFanIn = [],
  supports = {
    anyContamination: true,
    identityFanIn: true,
    identityFanInSpace: true,
    reExportRecords: "file-level",
  },
  unresolvedInternalSpecifiers = [],
  filesWithParseErrors = [],
  defAnyContamination = {},
  defKindsByIdentity = {},
} = {}) {
  const defIndex = {};
  for (const [file, names] of Object.entries(identitiesByFile)) {
    defIndex[file] = {};
    for (const name of names) {
      const key = `${file}::${name}`;
      const defInfo = {
        kind: defKindsByIdentity[key] ?? "TSTypeAliasDeclaration",
        line: 1,
      };
      if (defAnyContamination[key]) {
        defInfo.anyContamination = defAnyContamination[key];
      }
      defIndex[file][name] = defInfo;
    }
  }
  return {
    meta: { schemaVersion: 3, supports },
    defIndex,
    fanInByIdentity,
    fanInByIdentitySpace,
    topSymbolFanIn,
    unresolvedInternalSpecifiers,
    filesWithParseErrors,
  };
}

const USER_SERVICE_SYMBOLS = buildSymbols({
  identitiesByFile: {
    "src/services/user.ts": ["fetchUser"],
    "src/services/post.ts": ["fetchPost"],
  },
  defKindsByIdentity: {
    "src/services/user.ts::fetchUser": "FunctionDeclaration",
    "src/services/post.ts::fetchPost": "FunctionDeclaration",
  },
  fanInByIdentity: {},
});

describe("pre-write lookupName exact identity and capability evidence", () => {
  it("resolves a single exact identity with identity fan-in and fan-in-space evidence", () => {
    const symbols = buildSymbols({
      identitiesByFile: { "src/utils/date.ts": ["formatDate"] },
      fanInByIdentity: { "src/utils/date.ts::formatDate": 8 },
      fanInByIdentitySpace: {
        "src/utils/date.ts::formatDate": { value: 7, type: 1, broad: 0 },
      },
    });

    const result = lookupName("formatDate", { symbols, canonicalClaims: [] });

    expect(result.result).toBe("EXISTS");
    expect(result.identities).toHaveLength(1);
    expect(result.identities[0]).toMatchObject({
      identity: "src/utils/date.ts::formatDate",
      ownerFile: "src/utils/date.ts",
      fanIn: 8,
      fanInConfidence: "grounded",
      fanInSpace: { value: 7, type: 1, broad: 0 },
      fanInSpaceConfidence: "grounded",
    });
    expect(result.canonicalAstStatus).toBe("not-consulted");
    expect(result.identities[0].anyContamination.state).toBe("clean");
  });

  it("preserves all owners for duplicate exported names instead of picking one", () => {
    const symbols = buildSymbols({
      identitiesByFile: {
        "apps/admin/types.ts": ["User"],
        "apps/blog/types.ts": ["User"],
      },
      fanInByIdentity: {
        "apps/admin/types.ts::User": 5,
        "apps/blog/types.ts::User": 2,
      },
    });

    const result = lookupName("User", { symbols, canonicalClaims: [] });

    expect(result.result).toBe("EXISTS_MULTIPLE");
    expect(
      result.identities.map((identity) => identity.ownerFile).sort(),
    ).toEqual(["apps/admin/types.ts", "apps/blog/types.ts"]);
    expect(
      result.identities.find(
        (identity) => identity.ownerFile === "apps/admin/types.ts",
      ).fanIn,
    ).toBe(5);
    expect(
      result.identities.find(
        (identity) => identity.ownerFile === "apps/blog/types.ts",
      ).fanIn,
    ).toBe(2);
  });

  it("does not use topSymbolFanIn as a substitute for identity-keyed fan-in", () => {
    const symbols = buildSymbols({
      identitiesByFile: { "src/a.ts": ["formatDate"] },
      fanInByIdentity: {},
      topSymbolFanIn: [
        {
          defFile: "src/a.ts",
          symbol: "formatDate",
          count: 999,
          kind: "const",
        },
      ],
    });

    const result = lookupName("formatDate", { symbols, canonicalClaims: [] });

    expect(result.identities[0].fanIn).toBeNull();
    expect(result.identities[0].fanInConfidence).toBe("unavailable");
    expect(result.identities[0].citations.join(" ")).toContain("확인 불가");
  });

  it("keeps fan-in-space unavailable when the producer capability is absent", () => {
    const symbols = buildSymbols({
      identitiesByFile: { "src/a.ts": ["formatDate"] },
      fanInByIdentity: { "src/a.ts::formatDate": 8 },
      fanInByIdentitySpace: {
        "src/a.ts::formatDate": { value: 8, type: 0, broad: 0 },
      },
      supports: {
        anyContamination: true,
        identityFanIn: true,
        identityFanInSpace: false,
        reExportRecords: "file-level",
      },
    });

    const result = lookupName("formatDate", { symbols, canonicalClaims: [] });

    expect(result.identities[0].fanIn).toBe(8);
    expect(result.identities[0].fanInConfidence).toBe("grounded");
    expect(result.identities[0].fanInSpace).toBeNull();
    expect(result.identities[0].fanInSpaceConfidence).toBe("unavailable");
  });
});

describe("pre-write lookupName canonical-first states", () => {
  it("marks canonical and AST agreement as CANONICAL_EXISTS_AND_EXISTS", () => {
    const symbols = buildSymbols({
      identitiesByFile: { "src/protocol/ids.ts": ["SessionId"] },
      fanInByIdentity: { "src/protocol/ids.ts::SessionId": 8 },
    });
    const canonicalClaims = [
      {
        name: "SessionId",
        ownerFile: "src/protocol/ids.ts",
        line: 42,
        file: "canonical/type-ownership.md",
        section: "Single owner (strong)",
      },
    ];

    const result = lookupName("SessionId", { symbols, canonicalClaims });

    expect(result.result).toBe("CANONICAL_EXISTS_AND_EXISTS");
    expect(result.canonicalClaim.line).toBe(42);
    expect(result.canonicalAstStatus).toBe("aligned");
  });

  it("keeps canonical-only owners distinct from AST observed identities", () => {
    const symbols = buildSymbols({
      identitiesByFile: {},
      fanInByIdentity: {},
    });
    const canonicalClaims = [
      {
        name: "TokenKind",
        ownerFile: "src/auth/token.ts",
        line: 7,
        file: "canonical/type-ownership.md",
        section: "Single owner (strong)",
      },
    ];

    const result = lookupName("TokenKind", { symbols, canonicalClaims });

    expect(result.result).toBe("CANONICAL_EXISTS_AST_ABSENT");
    expect(result.canonicalAstStatus).toBe("ast-absent");
    expect(result.identities).toHaveLength(0);
  });

  it("preserves AST identities and structured owner disagreement without drift prose", () => {
    const symbols = buildSymbols({
      identitiesByFile: { "src/other/path.ts": ["User"] },
      fanInByIdentity: { "src/other/path.ts::User": 3 },
    });
    const canonicalClaims = [
      {
        name: "User",
        ownerFile: "src/models/User.ts",
        line: 3,
        file: "canonical/type-ownership.md",
        section: "Single owner (strong)",
      },
    ];

    const result = lookupName("User", { symbols, canonicalClaims });

    expect(result.result).toBe("CANONICAL_EXISTS_AST_DISAGREE");
    expect(result.canonicalAstStatus).toBe("owner-disagrees");
    expect(result.identities.map((identity) => identity.ownerFile)).toEqual([
      "src/other/path.ts",
    ]);
    expect(JSON.stringify(result)).not.toContain("CANONICAL DRIFT:");
  });

  it("aligns with one canonical owner while preserving extra AST owners", () => {
    const symbols = buildSymbols({
      identitiesByFile: {
        "src/models/User.ts": ["User"],
        "apps/legacy/user.ts": ["User"],
      },
      fanInByIdentity: {
        "src/models/User.ts::User": 5,
        "apps/legacy/user.ts::User": 1,
      },
    });
    const canonicalClaims = [
      {
        name: "User",
        ownerFile: "src/models/User.ts",
        line: 3,
        file: "canonical/type-ownership.md",
        section: "Single owner (strong)",
      },
    ];

    const result = lookupName("User", { symbols, canonicalClaims });

    expect(result.result).toBe("CANONICAL_EXISTS_AND_EXISTS");
    expect(result.canonicalAstStatus).toBe("aligned");
    expect(result.identities).toHaveLength(2);
  });
});

describe("pre-write lookupName search hints and suppressed diagnostics", () => {
  it("keeps near-name matches as diagnostic search hints", () => {
    const symbols = buildSymbols({
      identitiesByFile: {
        "src/utils/date.ts": ["formatDate", "formatDateTime", "formatTimeAgo"],
      },
      fanInByIdentity: {},
    });

    const result = lookupName("formatTimestamp", {
      symbols,
      canonicalClaims: [],
    });

    expect(result.result).toBe("NOT_OBSERVED");
    expect(result.nearNames.length).toBeGreaterThan(0);
    expect(result.nearNames.length).toBeLessThanOrEqual(5);
    expect(result.nearNames.some((hint) => hint.name === "formatDate")).toBe(
      true,
    );
  });

  it("uses intent words for semantic search hints without morphology-only promotion", () => {
    const symbols = buildSymbols({
      identitiesByFile: {
        "_lib/artifacts.mjs": ["loadIfExists", "readJsonFile"],
        "_lib/check-canon-artifact.mjs": ["loadHelperRegistryCanon"],
      },
      fanInByIdentity: {},
    });

    const result = lookupName("loadArtifactJson", {
      symbols,
      canonicalClaims: [],
      intentDeclaration: {
        name: "loadArtifactJson",
        kind: "function",
        why: "load a JSON artifact file with existence check",
      },
    });

    expect(result.result).toBe("NOT_OBSERVED");
    expect(
      result.semanticHints.some((hint) => hint.name === "loadIfExists"),
    ).toBe(true);
    expect(
      result.semanticHints.some((hint) => hint.name === "readJsonFile"),
    ).toBe(true);
    expect(
      result.semanticHints.some(
        (hint) => hint.name === "loadHelperRegistryCanon",
      ),
    ).toBe(false);
  });

  it("keeps create-only token overlap suppressed instead of formal semantic hints", () => {
    const symbols = buildSymbols({
      identitiesByFile: {
        "src/store.ts": ["createStore"],
        "src/storage.ts": ["createJSONStorage"],
      },
      fanInByIdentity: {},
    });

    const result = lookupName("createLogger", {
      symbols,
      canonicalClaims: [],
      intentDeclaration: {
        name: "createLogger",
        kind: "function",
        why: "create a logger helper",
      },
    });

    expect(result.semanticHints).toHaveLength(0);
    expect(result.nearNames).toHaveLength(0);
    expect(result.suppressedSemanticHints).toHaveLength(2);
    expect(
      result.suppressedSemanticHints.every(
        (hint) => hint.reason === "domain-token-overlap",
      ),
    ).toBe(true);
  });

  it("records why fetchUser fell below formal thresholds for searchUser", () => {
    const symbols = buildSymbols({
      identitiesByFile: {
        "src/services/user.ts": ["fetchUser"],
        "src/services/post.ts": ["fetchPost"],
        "src/utils/format.ts": ["formatTimestamp"],
      },
      fanInByIdentity: {},
    });

    const result = lookupName("searchUser", {
      symbols,
      canonicalClaims: [],
      intentDeclaration: {
        name: "searchUser",
        kind: "function",
        why: "search user data",
        ownerFile: "src/services/user-search.ts",
      },
    });

    expect(result.intentTokens).toEqual(
      expect.arrayContaining(["search", "user"]),
    );
    expect(result.semanticHints.some((hint) => hint.name === "fetchUser")).toBe(
      false,
    );
    expect(result.nearNames.some((hint) => hint.name === "fetchUser")).toBe(
      false,
    );
    expect(result.suppressedSemanticHints).toContainEqual(
      expect.objectContaining({
        name: "fetchUser",
        reason: "single-non-weak-token-only",
        score: 1,
        matchedTokens: expect.arrayContaining(["user"]),
        locality: expect.objectContaining({ sameDir: true, sameFile: false }),
      }),
    );
    expect(result.suppressedNearNames).toContainEqual(
      expect.objectContaining({
        name: "fetchUser",
        reason: "near-distance-exceeded",
        locality: expect.objectContaining({ sameDir: true, sameFile: false }),
      }),
    );
    expect(result.suppressedSemanticHintCount).toBeGreaterThanOrEqual(
      result.suppressedSemanticHints.length,
    );
    expect(result.suppressedNearNameCount).toBeGreaterThanOrEqual(
      result.suppressedNearNames.length,
    );
  });
});

describe("pre-write lookupName service-operation sibling policy", () => {
  it("promotes searchUser to fetchUser only inside versioned policy evidence", () => {
    const result = lookupName("searchUser", {
      symbols: USER_SERVICE_SYMBOLS,
      canonicalClaims: [],
      intentDeclaration: {
        name: "searchUser",
        kind: "function",
        why: "search user data",
        ownerFile: "src/services/user-search.ts",
      },
    });
    const policy = result.serviceOperationSiblingPolicy;
    const promoted = policy.promoted.find((hint) => hint.name === "fetchUser");

    expect(policy.policyId).toBe("prewrite-service-operation-sibling-cue");
    expect(policy.policyVersion).toBe(
      "prewrite-service-operation-sibling-cue-v1",
    );
    expect(promoted).toMatchObject({
      identity: "src/services/user.ts::fetchUser",
      ownerFile: "src/services/user.ts",
      operationFamily: "read-query",
      locality: { sameDir: true, sameFile: false },
      signatureSupport: {
        status: "unavailable",
        reason: "no-signature-facts",
      },
    });
    expect(promoted.sharedDomainTokens).toContain("user");
    expect(promoted.supportingReasons).toEqual(
      expect.arrayContaining([
        "single-non-weak-token-only",
        "near-distance-exceeded",
      ]),
    );
    expect(result.semanticHints.some((hint) => hint.name === "fetchUser")).toBe(
      false,
    );
    expect(result.nearNames.some((hint) => hint.name === "fetchUser")).toBe(
      false,
    );
  });

  it("keeps operation-family and domain mismatches muted instead of promoted", () => {
    const createResult = lookupName("createUser", {
      symbols: USER_SERVICE_SYMBOLS,
      canonicalClaims: [],
      intentDeclaration: {
        name: "createUser",
        kind: "function",
        why: "create user data",
        ownerFile: "src/services/user-create.ts",
      },
    });
    const postResult = lookupName("searchPost", {
      symbols: buildSymbols({
        identitiesByFile: {
          "src/services/user.ts": ["fetchUser"],
        },
        defKindsByIdentity: {
          "src/services/user.ts::fetchUser": "FunctionDeclaration",
        },
        fanInByIdentity: {},
      }),
      canonicalClaims: [],
      intentDeclaration: {
        name: "searchPost",
        kind: "function",
        why: "search user data while writing the post search flow",
        ownerFile: "src/services/post-search.ts",
      },
    });

    expect(
      createResult.serviceOperationSiblingPolicy.promoted.some(
        (hint) => hint.name === "fetchUser",
      ),
    ).toBe(false);
    expect(createResult.serviceOperationSiblingPolicy.muted).toContainEqual(
      expect.objectContaining({
        name: "fetchUser",
        reason: "service-sibling-operation-family-mismatch",
      }),
    );
    expect(
      postResult.serviceOperationSiblingPolicy.promoted.some(
        (hint) => hint.name === "fetchUser",
      ),
    ).toBe(false);
    expect(postResult.serviceOperationSiblingPolicy.muted).toContainEqual(
      expect.objectContaining({
        name: "fetchUser",
        reason: "service-sibling-domain-mismatch",
      }),
    );
  });

  it("mutes type-like service candidates instead of promoting them as operations", () => {
    const result = lookupName("queryLibraryDoc", {
      symbols: buildSymbols({
        identitiesByFile: {
          "apps/server/src/repository.ts": [
            "ListLibraryDocsOptions",
            "listLibraryDocs",
          ],
        },
        defKindsByIdentity: {
          "apps/server/src/repository.ts::ListLibraryDocsOptions":
            "TSInterfaceDeclaration",
          "apps/server/src/repository.ts::listLibraryDocs":
            "FunctionDeclaration",
        },
      }),
      canonicalClaims: [],
      intentDeclaration: {
        name: "queryLibraryDoc",
        kind: "function",
        why: "query library docs from the repository",
        ownerFile: "apps/server/src/repository.ts",
      },
    });

    expect(result.serviceOperationSiblingPolicy.promoted).toContainEqual(
      expect.objectContaining({ name: "listLibraryDocs" }),
    );
    expect(result.serviceOperationSiblingPolicy.promoted).not.toContainEqual(
      expect.objectContaining({ name: "ListLibraryDocsOptions" }),
    );
    expect(result.serviceOperationSiblingPolicy.muted).toContainEqual(
      expect.objectContaining({
        name: "ListLibraryDocsOptions",
        reason: "service-sibling-non-callable-definition",
      }),
    );
  });

  it("keeps unrelated intents at the noise floor", () => {
    const result = lookupName("xyzzy", {
      symbols: USER_SERVICE_SYMBOLS,
      canonicalClaims: [],
      intentDeclaration: {
        name: "xyzzy",
        kind: "function",
        why: "unrelated marker",
        ownerFile: "src/services/xyzzy.ts",
      },
    });

    expect(result.suppressedNearNames).toHaveLength(0);
    expect(result.suppressedSemanticHints).toHaveLength(0);
    expect(result.serviceOperationSiblingPolicy).toMatchObject({
      evaluatedCandidateCount: 0,
      promotedCandidateCount: 0,
      mutedCandidateCount: 0,
    });
  });
});

describe("pre-write lookupName confidence and contamination diagnostics", () => {
  it("keeps any-contamination labels and measurements as review diagnostics", () => {
    const symbols = buildSymbols({
      identitiesByFile: { "src/t.ts": ["VeryDirty"] },
      fanInByIdentity: { "src/t.ts::VeryDirty": 1 },
      defAnyContamination: {
        "src/t.ts::VeryDirty": {
          label: "severely-any-contaminated",
          labels: ["has-any", "any-contaminated", "severely-any-contaminated"],
          measurements: {
            totalFields: 7,
            anyFields: 6,
            unknownFields: 0,
            anyFieldRatio: 0.85,
            indexSignatureAny: false,
          },
        },
      },
    });

    const result = lookupName("VeryDirty", { symbols, canonicalClaims: [] });

    expect(result.identities[0].anyContamination.state).toBe(
      "severely-any-contaminated",
    );
    expect(
      result.identities[0].anyContamination.measurements.anyFieldRatio,
    ).toBe(0.85);
    expect(result.identities[0].anyContamination.recommendation.action).toBe(
      "warn-on-reuse",
    );
    expect(
      result.identities[0].citations.some((citation) =>
        citation.includes(
          "[grounded, anyContamination.label = 'severely-any-contaminated'",
        ),
      ),
    ).toBe(true);
  });

  it("demotes resolver confidence for unresolved specifier overlap and parse errors", () => {
    const unresolvedSymbols = buildSymbols({
      identitiesByFile: {
        "apps/other/components/authControl.tsx": ["AuthControl"],
      },
      fanInByIdentity: {
        "apps/other/components/authControl.tsx::AuthControl": 0,
      },
      unresolvedInternalSpecifiers: ["@/components/authControl"],
    });
    const parseErrorSymbols = buildSymbols({
      identitiesByFile: { "src/broken.ts": ["BrokenSym"] },
      fanInByIdentity: { "src/broken.ts::BrokenSym": 0 },
      filesWithParseErrors: ["src/broken.ts"],
    });

    const unresolved = lookupName("AuthControl", {
      symbols: unresolvedSymbols,
      canonicalClaims: [],
    });
    const parseError = lookupName("BrokenSym", {
      symbols: parseErrorSymbols,
      canonicalClaims: [],
    });

    expect(["medium", "low"]).toContain(
      unresolved.identities[0].resolverConfidence,
    );
    expect(parseError.identities[0].resolverConfidence).not.toBe("high");
  });
});

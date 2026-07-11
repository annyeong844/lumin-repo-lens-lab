import {
  isGeneratedVirtualResolution,
  isNonSourceAssetResolution,
} from "./resolver-core.mjs";

export function isImportedNamespaceAliasUse(use) {
  return (
    use?.kind === "imported-namespace-member" ||
    use?.kind === "imported-namespace-escape"
  );
}

export function isRustResolvedRelativeUse(use) {
  return (
    typeof use === "object" &&
    use?.resolverStage === "relative" &&
    typeof use.resolvedFile === "string" &&
    use.resolvedFile.length > 0
  );
}

export function isRelativeSourceUse(use) {
  return (
    typeof use === "object" &&
    typeof use?.fromSpec === "string" &&
    (use.fromSpec.startsWith("./") || use.fromSpec.startsWith("../"))
  );
}

function stripResourceQuery(spec) {
  const query = spec.indexOf("?");
  const hash = spec.indexOf("#");
  const cuts = [query, hash].filter((index) => index >= 0);
  return cuts.length > 0 ? spec.slice(0, Math.min(...cuts)) : spec;
}

export function looksLikeNonSourceAssetSpecifier(spec) {
  if (typeof spec !== "string") return false;
  const stripped = stripResourceQuery(spec);
  const fileName = stripped.split("/").at(-1) ?? stripped;
  const dot = fileName.lastIndexOf(".");
  if (dot <= 0 || dot === fileName.length - 1) return false;
  return !/\.(?:ts|tsx|js|jsx|mjs|cjs|mts|cts|d\.ts|d\.mts|d\.cts)$/i.test(
    stripped,
  );
}

export function sourceUseRequiresSymbolName(kind) {
  return ![
    "cjs-side-effect-only",
    "import-side-effect",
    "reExportNamespace",
    "sfc-script-src",
    "namespace",
    "reExportAll",
    "dynamic",
    "import-meta-glob",
    "dynamic-import-meta-glob",
    "cjs-namespace-escape",
    "cjs-reexport-broad",
  ].includes(kind);
}

export function isInlineSourceUseCandidate(use) {
  if (!isRelativeSourceUse(use) && typeof use?.resolvedFile !== "string") {
    return false;
  }
  if (
    !isRustResolvedRelativeUse(use) &&
    use?.resolverStage !== "resolved-internal"
  ) {
    return false;
  }
  if (looksLikeNonSourceAssetSpecifier(use.fromSpec)) return false;
  const kind = use.kind ?? "import";
  return !(
    sourceUseRequiresSymbolName(kind) &&
    (typeof use.name !== "string" || use.name.length === 0)
  );
}

export function isResolvableRelativeSourceUseCandidate(use) {
  if (!isRelativeSourceUse(use)) return false;
  if (typeof use?.resolvedFile === "string" && use.resolvedFile.length > 0) {
    return false;
  }
  if (use?.kind === "import-meta-glob") return false;
  if (looksLikeNonSourceAssetSpecifier(use.fromSpec)) return false;
  const kind = use.kind ?? "import";
  return !(
    sourceUseRequiresSymbolName(kind) &&
    (typeof use.name !== "string" || use.name.length === 0)
  );
}

export function sourceUseRecordFailureReason(use) {
  if (!use || typeof use !== "object") return "non-object-use";
  if (typeof use.fromSpec !== "string" || use.fromSpec.length === 0) {
    return "missing-specifier";
  }
  if (looksLikeNonSourceAssetSpecifier(use.fromSpec)) {
    return "non-source-asset-specifier";
  }
  const kind = use.kind ?? "import";
  if (
    sourceUseRequiresSymbolName(kind) &&
    (typeof use.name !== "string" || use.name.length === 0)
  ) {
    return "missing-symbol-name";
  }
  if (!isRelativeSourceUse(use) && typeof use.resolvedFile !== "string") {
    return "non-relative-requires-js-resolver";
  }
  if (
    !isRustResolvedRelativeUse(use) &&
    use.resolverStage !== "resolved-internal"
  ) {
    return "missing-rust-resolved-stage";
  }
  return "record-build-failed";
}

export function createSourceUseRecordBuilder({
  normalizePath,
  unresolvedEvidence,
  existingRelativeTarget,
}) {
  const kind = (value) =>
    typeof value === "string" && value !== "import" ? value : undefined;
  const typeOnly = (value) => value === true;
  const resolverStage = (stage, resolvedFile) => {
    if (
      stage === "resolved-internal" &&
      typeof resolvedFile === "string" &&
      resolvedFile.length > 0
    ) {
      return undefined;
    }
    if (typeof stage === "string" && stage.length > 0) return stage;
    return typeof resolvedFile === "string" && resolvedFile.length > 0
      ? "resolved-internal"
      : undefined;
  };
  const consumerSource = (value) =>
    typeof value === "string" && value !== "source-import" ? value : undefined;

  function record(recordId, consumerFile, use) {
    if (!use || typeof use.fromSpec !== "string" || use.fromSpec.length === 0) {
      return null;
    }
    return {
      recordId,
      consumerFile: normalizePath(consumerFile),
      fromSpec: use.fromSpec,
      name: use.name,
      memberName: use.memberName,
      kind: kind(use.kind),
      typeOnly: typeOnly(use.typeOnly),
      typeOnlyPresent: typeof use.typeOnly === "boolean",
      line: Number.isFinite(use.line) ? use.line : undefined,
      sfcLanguage: use.sfcLanguage,
      resolvedFile: normalizePath(use.resolvedFile),
      resolverStage: resolverStage(use.resolverStage, use.resolvedFile),
      consumerSource: consumerSource(use.consumerSource),
      unresolvedEvidence: use.unresolvedEvidence,
      generatedVirtualSurface: use.generatedVirtualSurface,
    };
  }

  function externalRecord(recordId, consumerFile, use, source) {
    return record(recordId, consumerFile, {
      ...use,
      resolverStage: "external",
      consumerSource: source,
    });
  }

  function unresolvedRecord(recordId, consumerFile, use, stage) {
    return record(recordId, consumerFile, {
      ...use,
      resolverStage: stage,
      unresolvedEvidence:
        use?.unresolvedEvidence ?? unresolvedEvidence(consumerFile, use),
    });
  }

  function generatedVirtualCanResolve(surface, use) {
    if (use?.kind === "import-side-effect") return false;
    if (use?.kind === "namespace") return true;
    const name = use?.name;
    if (typeof name !== "string" || name.length === 0 || name === "*") {
      return false;
    }
    const wantedSpace = use?.typeOnly === true ? "type" : "value";
    return (surface?.exports ?? []).some(
      (entry) =>
        entry?.name === name &&
        Array.isArray(entry.spaces) &&
        entry.spaces.includes(wantedSpace),
    );
  }

  function generatedVirtualRecord(recordId, consumerFile, use, surface) {
    if (!isGeneratedVirtualResolution(surface)) return null;
    return record(recordId, consumerFile, {
      ...use,
      resolverStage: "generated-virtual",
      generatedVirtualSurface: surface,
      ...(!generatedVirtualCanResolve(surface, use)
        ? { unresolvedEvidence: unresolvedEvidence(consumerFile, use) }
        : {}),
    });
  }

  function nonSourceAssetRecord(recordId, consumerFile, use) {
    return record(recordId, consumerFile, {
      ...use,
      resolverStage: "non-source-asset",
    });
  }

  function terminalRecord(recordId, consumerFile, use, target, source) {
    const recordUse = source ? { ...use, consumerSource: source } : use;
    if (target === "EXTERNAL") {
      return {
        branch: "external",
        outcome: "external",
        record: externalRecord(
          recordId,
          consumerFile,
          recordUse,
          source ?? "source-import",
        ),
      };
    }
    if (isNonSourceAssetResolution(target)) {
      return {
        branch: "asset",
        outcome: "non-source-asset",
        record: nonSourceAssetRecord(recordId, consumerFile, {
          ...recordUse,
          resolvedFile: existingRelativeTarget(
            consumerFile,
            recordUse.fromSpec,
          ),
        }),
      };
    }
    if (target === "UNRESOLVED_INTERNAL" || !target) {
      const stage =
        target === "UNRESOLVED_INTERNAL"
          ? "unresolved-internal"
          : "unresolved-relative";
      return {
        branch: "unresolved",
        outcome: stage,
        record: unresolvedRecord(recordId, consumerFile, recordUse, stage),
      };
    }
    if (isGeneratedVirtualResolution(target)) {
      return {
        branch: "generatedVirtual",
        outcome: "generated-virtual",
        record: generatedVirtualRecord(
          recordId,
          consumerFile,
          recordUse,
          target,
        ),
      };
    }
    return {
      branch: "resolvedInternal",
      outcome: "resolved-internal",
      record: record(recordId, consumerFile, {
        ...recordUse,
        resolvedFile: target,
        resolverStage: "resolved-internal",
      }),
    };
  }

  return {
    externalRecord,
    generatedVirtualRecord,
    nonSourceAssetRecord,
    record,
    terminalRecord,
    unresolvedRecord,
  };
}

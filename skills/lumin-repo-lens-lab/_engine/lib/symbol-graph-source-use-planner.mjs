import { relPath } from "./paths.mjs";
import { sourceUseProjectionRecordId } from "./source-use-assembly-request.mjs";
import {
  isImportedNamespaceAliasUse,
  isInlineSourceUseCandidate,
  isRelativeSourceUse,
  isResolvableRelativeSourceUseCandidate,
  looksLikeNonSourceAssetSpecifier,
  sourceUseRecordFailureReason,
  sourceUseRequiresSymbolName,
} from "./source-use-record-builder.mjs";

function requireRecord(record, recordId, outcome) {
  if (record) return record;
  throw new Error(`source-use assembly refused ${outcome} record ${recordId}`);
}

function inlineRecordId(root, consumerFile, index) {
  return `${relPath(root, consumerFile)}#${index}`;
}

function isOutOfBandCandidate(use) {
  if (!isRelativeSourceUse(use)) return false;
  if (looksLikeNonSourceAssetSpecifier(use.fromSpec)) return false;
  const kind = use.kind ?? "import";
  return !(
    sourceUseRequiresSymbolName(kind) &&
    (typeof use.name !== "string" || use.name.length === 0)
  );
}

export function planInlineSourceUses({
  root,
  fileData,
  recordBuilder,
  canFastPathExternal,
  existingRelativeNonSourceAssetTarget,
}) {
  const records = [];
  const requiresRustResolution = [];
  const requiresJsResolution = [];
  let namespaceReExportCandidateCount = 0;

  for (const [consumerFile, info] of fileData) {
    for (let index = 0; index < info.uses.length; index++) {
      const use = info.uses[index];
      const recordId = inlineRecordId(root, consumerFile, index);
      if (consumerFile.endsWith(".py") || consumerFile.endsWith(".go")) {
        requiresJsResolution.push({ consumerFile, useIndex: index, use });
        continue;
      }
      if (canFastPathExternal(consumerFile, use)) {
        const record = recordBuilder.externalRecord(
          recordId,
          consumerFile,
          use,
          "source-import",
        );
        if (record) {
          records.push(record);
          continue;
        }
      }
      if (use?.kind === "import-meta-glob") {
        records.push(
          requireRecord(
            recordBuilder.record(recordId, consumerFile, {
              ...use,
              resolverStage: "relative",
            }),
            recordId,
            "import.meta.glob",
          ),
        );
        continue;
      }
      if (existingRelativeNonSourceAssetTarget(consumerFile, use?.fromSpec)) {
        const record = recordBuilder.nonSourceAssetRecord(
          recordId,
          consumerFile,
          use,
        );
        if (record) {
          records.push(record);
          continue;
        }
      }
      if (!isInlineSourceUseCandidate(use)) {
        if (isResolvableRelativeSourceUseCandidate(use)) {
          const record = recordBuilder.record(recordId, consumerFile, {
            ...use,
            resolverStage: "relative",
          });
          if (record) {
            requiresRustResolution.push(record);
            continue;
          }
        }
        const reason = sourceUseRecordFailureReason(use);
        if (reason !== "non-relative-requires-js-resolver") {
          throw new Error(
            `source-use assembly refused ${reason} record ${recordId}`,
          );
        }
        requiresJsResolution.push({ consumerFile, useIndex: index, use });
        continue;
      }
      const record = requireRecord(
        recordBuilder.record(recordId, consumerFile, use),
        recordId,
        "inline",
      );
      if (isImportedNamespaceAliasUse(use)) {
        namespaceReExportCandidateCount++;
      }
      records.push(record);
    }
  }

  return {
    records,
    requiresRustResolution,
    requiresJsResolution,
    namespaceReExportCandidateCount,
  };
}

export function planOutOfBandSourceUses({
  root,
  consumers,
  source,
  recordBuilder,
  canFastPathExternal,
  existingRelativeNonSourceAssetTarget,
}) {
  const records = [];
  for (let index = 0; index < consumers.length; index++) {
    const use = consumers[index];
    const recordId = sourceUseProjectionRecordId(root, source, index, use);
    if (canFastPathExternal(use.consumerFile, use)) {
      const record = recordBuilder.externalRecord(
        recordId,
        use.consumerFile,
        use,
        source,
      );
      if (record) records.push(record);
      continue;
    }
    if (existingRelativeNonSourceAssetTarget(use.consumerFile, use.fromSpec)) {
      const record = recordBuilder.nonSourceAssetRecord(
        recordId,
        use.consumerFile,
        use,
      );
      if (record) records.push(record);
      continue;
    }
    if (!isOutOfBandCandidate(use)) continue;
    const recordUse =
      source === "sfc-script-src"
        ? {
            ...use,
            kind: "sfc-script-src",
            typeOnly: false,
            consumerSource: source,
            resolverStage: "relative",
            unresolvedEvidence: {
              reason: "sfc-script-src-unresolved",
              resolverStage: "sfc-script-src",
              outputLevel: "unsupported",
              unsupportedFamily: "sfc-script-src",
              hint: "sfc-script-src-reachability",
            },
          }
        : { ...use, consumerSource: source, resolverStage: "relative" };
    const record = recordBuilder.record(recordId, use.consumerFile, recordUse);
    if (record) records.push(record);
  }
  return records;
}

export function planSfcComponentSourceUses({
  root,
  consumers,
  source,
  consumerFileForUse,
  fromSpecForUse,
  kind,
  recordBuilder,
  canFastPathExternal,
  existingRelativeNonSourceAssetTarget,
  existingExtensionlessRelativeRawTarget,
  resolve,
}) {
  const records = [];
  for (let index = 0; index < consumers.length; index++) {
    const use = consumers[index];
    const consumerFile = consumerFileForUse(use);
    const fromSpec = fromSpecForUse(use);
    if (
      typeof consumerFile !== "string" ||
      consumerFile.length === 0 ||
      typeof fromSpec !== "string" ||
      fromSpec.length === 0
    ) {
      continue;
    }
    const recordId = sourceUseProjectionRecordId(root, source, index, {
      consumerFile,
      fromSpec,
    });
    const recordUse = {
      fromSpec,
      kind,
      name: "*",
      typeOnly: false,
      consumerSource: source,
    };
    const nonSourceTarget =
      existingExtensionlessRelativeRawTarget(consumerFile, fromSpec) ??
      existingRelativeNonSourceAssetTarget(consumerFile, fromSpec);
    if (nonSourceTarget) {
      records.push(
        requireRecord(
          recordBuilder.nonSourceAssetRecord(recordId, consumerFile, {
            ...recordUse,
            resolvedFile: nonSourceTarget,
          }),
          recordId,
          "non-source-asset",
        ),
      );
      continue;
    }
    if (canFastPathExternal(consumerFile, recordUse)) {
      records.push(
        requireRecord(
          recordBuilder.externalRecord(
            recordId,
            consumerFile,
            recordUse,
            source,
          ),
          recordId,
          "external",
        ),
      );
      continue;
    }
    if (isRelativeSourceUse(recordUse)) {
      records.push(
        requireRecord(
          recordBuilder.record(recordId, consumerFile, {
            ...recordUse,
            resolverStage: "relative",
          }),
          recordId,
          "relative",
        ),
      );
      continue;
    }
    const target = resolve(
      consumerFile,
      recordUse,
      `${source}-projection-input`,
    );
    const terminal = recordBuilder.terminalRecord(
      recordId,
      consumerFile,
      recordUse,
      target,
      source,
    );
    records.push(requireRecord(terminal.record, recordId, terminal.outcome));
  }
  return records;
}

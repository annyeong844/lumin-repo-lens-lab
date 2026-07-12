import { relPath } from "./paths.mjs";

function fileDataRecord(filePath, info) {
  const reExports = info.reExports ?? [];
  const classMethods = info.classMethods ?? [];
  const localOperations = info.localOperations ?? [];
  const typeEscapes = info.typeEscapes ?? [];
  const dynamicImportOpacity = info.dynamicImportOpacity ?? [];
  const cjsRequireOpacity = info.cjsRequireOpacity ?? [];
  const hasData =
    reExports.length > 0 ||
    classMethods.length > 0 ||
    localOperations.length > 0 ||
    typeEscapes.length > 0 ||
    dynamicImportOpacity.length > 0 ||
    cjsRequireOpacity.length > 0 ||
    info.cjsExportSurface != null ||
    info.pyDunderAll !== undefined;
  if (!hasData) return null;
  return {
    filePath,
    pyDunderAll: info.pyDunderAll ?? null,
    reExports,
    classMethods,
    localOperations,
    typeEscapes,
    dynamicImportOpacity,
    cjsExportSurface: info.cjsExportSurface ?? null,
    cjsRequireOpacity,
  };
}

function parseErrorFiles(entries) {
  return Object.entries(entries)
    .filter(([, entry]) => entry?.parseError === true)
    .map(([filePath]) => filePath);
}

function compactFanInPaths(root, fanIn) {
  return {
    ...fanIn,
    consumerEntries: fanIn.consumerEntries.map((entry) => ({
      ...entry,
      defFile: relPath(root, entry.defFile),
      consumerFile: relPath(root, entry.consumerFile),
    })),
    namespaceUserEntries: fanIn.namespaceUserEntries.map((entry) => ({
      ...entry,
      defFile: relPath(root, entry.defFile),
      consumerFile: relPath(root, entry.consumerFile),
    })),
  };
}

function compactDeadCandidatePaths(root, deadCandidates) {
  return {
    ...deadCandidates,
    barrelFiles: deadCandidates.barrelFiles.map((file) => relPath(root, file)),
    testLikeFiles: deadCandidates.testLikeFiles.map((file) =>
      relPath(root, file),
    ),
  };
}

function compactSourceUsePaths(sourceUseAssembly, pathId) {
  if (!Array.isArray(sourceUseAssembly.pathTable)) return sourceUseAssembly;

  const pathIdRemap = sourceUseAssembly.pathTable.map(pathId);
  const remapPathId = (id) =>
    Number.isInteger(id) && pathIdRemap[id] != null ? pathIdRemap[id] : id;
  const remapRecordRows = (fields, rows) => {
    if (!Array.isArray(fields) || !Array.isArray(rows)) return rows;
    const consumerFileIdIndex = fields.indexOf("consumerFileId");
    const resolvedFileIdIndex = fields.indexOf("resolvedFileId");
    if (consumerFileIdIndex < 0 && resolvedFileIdIndex < 0) return rows;
    return rows.map((row) => {
      if (!Array.isArray(row)) return row;
      const next = [...row];
      if (
        consumerFileIdIndex >= 0 &&
        Number.isInteger(next[consumerFileIdIndex])
      ) {
        next[consumerFileIdIndex] = remapPathId(next[consumerFileIdIndex]);
      }
      if (
        resolvedFileIdIndex >= 0 &&
        Number.isInteger(next[resolvedFileIdIndex])
      ) {
        next[resolvedFileIdIndex] = remapPathId(next[resolvedFileIdIndex]);
      }
      return next;
    });
  };
  const { pathTable: _pathTable, ...rest } = sourceUseAssembly;
  return {
    ...rest,
    ...(Array.isArray(sourceUseAssembly.sourceFileIds)
      ? { sourceFileIds: sourceUseAssembly.sourceFileIds.map(remapPathId) }
      : {}),
    records: Array.isArray(sourceUseAssembly.records)
      ? sourceUseAssembly.records.map((record) => ({
          ...record,
          ...(Number.isInteger(record.consumerFileId)
            ? { consumerFileId: remapPathId(record.consumerFileId) }
            : {}),
          ...(Number.isInteger(record.resolvedFileId)
            ? { resolvedFileId: remapPathId(record.resolvedFileId) }
            : {}),
        }))
      : sourceUseAssembly.records,
    ...(Array.isArray(sourceUseAssembly.recordRows)
      ? {
          recordRows: remapRecordRows(
            sourceUseAssembly.recordRowFields,
            sourceUseAssembly.recordRows,
          ),
        }
      : {}),
  };
}

function compactRequestPaths(root, request) {
  const pathTable = [];
  const pathIds = new Map();
  const pathId = (value) => {
    if (typeof value !== "string" || value.length === 0) return null;
    const normalized = relPath(root, value);
    const existing = pathIds.get(normalized);
    if (existing !== undefined) return existing;
    const id = pathTable.length;
    pathTable.push(normalized);
    pathIds.set(normalized, id);
    return id;
  };
  const withFilePathId = ({ filePath, ...entry }) => {
    const filePathId = pathId(filePath);
    return {
      ...entry,
      ...(filePathId !== null ? { filePathId } : { filePath }),
    };
  };

  return {
    schemaVersion: request.schemaVersion,
    context: request.context,
    extraction: {
      pathTable,
      fileIds: request.extraction.files.map(pathId).filter((id) => id !== null),
      defIndex: request.extraction.defIndex.map(withFilePathId),
      fileData: request.extraction.fileData.map(withFilePathId),
      parseErrorFileIds: request.extraction.parseErrorFiles
        .map(pathId)
        .filter((id) => id !== null),
    },
    sourceUseAssembly: compactSourceUsePaths(request.sourceUseAssembly, pathId),
    graph: {
      fanIn: compactFanInPaths(root, request.graph.fanIn),
      deadCandidates: compactDeadCandidatePaths(
        root,
        request.graph.deadCandidates,
      ),
      sfc: request.graph.sfc,
    },
  };
}

export function buildSymbolGraphArtifactRequest({
  root,
  context,
  files,
  defIndex,
  fileData,
  cacheEntries,
  sourceUseAssembly,
  graph,
  compactPaths,
}) {
  const artifactParseErrorFiles = parseErrorFiles(cacheEntries);
  let request = {
    schemaVersion: "lumin-symbol-graph-producer-request.v2",
    context,
    extraction: {
      files,
      defIndex: [...defIndex.entries()].map(([filePath, definitions]) => ({
        filePath,
        definitions: Object.fromEntries(definitions),
      })),
      fileData: [...fileData.entries()]
        .map(([filePath, info]) => fileDataRecord(filePath, info))
        .filter((record) => record !== null),
      parseErrorFiles: artifactParseErrorFiles,
    },
    sourceUseAssembly,
    graph,
  };
  if (compactPaths) request = compactRequestPaths(root, request);
  return { request, artifactParseErrorFiles };
}

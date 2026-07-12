import { relPath } from "./paths.mjs";

export function sourceUseProjectionRecordId(root, source, index, use) {
  const consumerFile = relPath(root, use?.consumerFile ?? "");
  const fromSpec = use?.fromSpec ?? "";
  return `${source}:${index}:${consumerFile}:${fromSpec}`;
}

export function sourceUseAssemblyNeedsSourceFiles(records) {
  return records.some(
    (record) =>
      record?.resolverStage === "relative" &&
      typeof record.resolvedFile !== "string",
  );
}

export function sourceUseRecordIdRemap(records) {
  const remap = new Map();
  for (let index = 0; index < records.length; index++) {
    const recordId = records[index]?.recordId;
    if (typeof recordId === "string" && recordId.length > 0) {
      remap.set(recordId, `r${index}`);
    }
  }
  return remap;
}

export function remapSourceUseRecordIdInputs(inputs, remap) {
  if (!Array.isArray(inputs) || remap.size === 0) return inputs;
  return inputs.map((input) => {
    const sourceUseRecordId = input?.sourceUseRecordId;
    if (
      typeof sourceUseRecordId !== "string" ||
      sourceUseRecordId.length === 0
    ) {
      return input;
    }
    const remapped = remap.get(sourceUseRecordId);
    return typeof remapped === "string" && remapped.length > 0
      ? { ...input, sourceUseRecordId: remapped }
      : input;
  });
}

function compactRecordIds(records) {
  return records.map((record, index) => ({
    ...record,
    recordId: `r${index}`,
  }));
}

function compactRecordPaths(root, records, sourceFiles = []) {
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
  const sourceFileIds = sourceFiles.map(pathId).filter((id) => id !== null);
  return {
    pathTable,
    sourceFiles: sourceFileIds.length === sourceFiles.length ? [] : sourceFiles,
    ...(sourceFileIds.length === sourceFiles.length ? { sourceFileIds } : {}),
    records: records.map((record) => {
      const { consumerFile, resolvedFile, ...rest } = record;
      const consumerFileId = pathId(consumerFile);
      const resolvedFileId = pathId(resolvedFile);
      return {
        ...rest,
        ...(consumerFileId !== null ? { consumerFileId } : {}),
        ...(resolvedFileId !== null ? { resolvedFileId } : {}),
      };
    }),
  };
}

function compactRecordEnums(records) {
  const kindTable = [];
  const resolverStageTable = [];
  const consumerSourceTable = [];
  const tableId = (table, ids, value) => {
    if (typeof value !== "string" || value.length === 0) return null;
    const existing = ids.get(value);
    if (existing !== undefined) return existing;
    const id = table.length;
    table.push(value);
    ids.set(value, id);
    return id;
  };
  const kindIds = new Map();
  const resolverStageIds = new Map();
  const consumerSourceIds = new Map();
  return {
    kindTable,
    resolverStageTable,
    consumerSourceTable,
    records: records.map((record) => {
      const { kind, resolverStage, consumerSource, ...rest } = record;
      const kindId = tableId(kindTable, kindIds, kind);
      const resolverStageId = tableId(
        resolverStageTable,
        resolverStageIds,
        resolverStage,
      );
      const consumerSourceId = tableId(
        consumerSourceTable,
        consumerSourceIds,
        consumerSource,
      );
      return {
        ...rest,
        ...(kindId !== null ? { kindId } : {}),
        ...(resolverStageId !== null ? { resolverStageId } : {}),
        ...(consumerSourceId !== null ? { consumerSourceId } : {}),
      };
    }),
  };
}

function compactRecordSpecifiers(records) {
  const specifierTable = [];
  const specifierIds = new Map();
  const specifierId = (value) => {
    if (typeof value !== "string" || value.length === 0) return null;
    const existing = specifierIds.get(value);
    if (existing !== undefined) return existing;
    const id = specifierTable.length;
    specifierTable.push(value);
    specifierIds.set(value, id);
    return id;
  };
  return {
    specifierTable,
    records: records.map((record) => {
      const { fromSpec, ...rest } = record;
      const fromSpecId = specifierId(fromSpec);
      return {
        ...rest,
        ...(fromSpecId !== null ? { fromSpecId } : {}),
      };
    }),
  };
}

function compactRecordNames(records) {
  const nameTable = [];
  const nameIds = new Map();
  const nameId = (value) => {
    if (typeof value !== "string" || value.length === 0) return null;
    const existing = nameIds.get(value);
    if (existing !== undefined) return existing;
    const id = nameTable.length;
    nameTable.push(value);
    nameIds.set(value, id);
    return id;
  };
  return {
    nameTable,
    records: records.map((record) => {
      const { name, memberName, ...rest } = record;
      const compactedNameId = nameId(name);
      const compactedMemberNameId = nameId(memberName);
      return {
        ...rest,
        ...(compactedNameId !== null ? { nameId: compactedNameId } : {}),
        ...(compactedMemberNameId !== null
          ? { memberNameId: compactedMemberNameId }
          : {}),
      };
    }),
  };
}

function recordRowFields({ compactNames, compactTypeOnly }) {
  return [
    "consumerFileId",
    "resolvedFileId",
    "fromSpecId",
    compactNames ? "nameId" : "name",
    compactNames ? "memberNameId" : "memberName",
    "kindId",
    ...(compactTypeOnly ? ["typeOnlyState"] : ["typeOnly", "typeOnlyPresent"]),
    "line",
    "sfcLanguage",
    "resolverStageId",
    "consumerSourceId",
    "unresolvedEvidence",
    "generatedVirtualSurface",
  ];
}

function recordRowValue(record, field) {
  if (field === "typeOnlyState") {
    if (record.typeOnly === true) return 2;
    return record.typeOnlyPresent === true ? 1 : null;
  }
  const value = record[field];
  if (value === undefined || value === null) return null;
  if (typeof value === "string" && value.length === 0) return null;
  return value;
}

function compactRecordRows(records, { compactNames, compactTypeOnly }) {
  const candidateFields = recordRowFields({ compactNames, compactTypeOnly });
  const candidateRows = records.map((record) =>
    candidateFields.map((field) => recordRowValue(record, field)),
  );
  const retainedFieldIndexes = candidateFields
    .map((field, index) => ({ field, index }))
    .filter(({ index }) => candidateRows.some((row) => row[index] !== null));
  const fields = retainedFieldIndexes.map(({ field }) => field);
  const rows = candidateRows.map((candidateRow) => {
    const row = retainedFieldIndexes.map(({ index }) => candidateRow[index]);
    while (row.length > 0 && row[row.length - 1] === null) row.pop();
    return row;
  });
  return { fields, rows };
}

export function buildSourceUseAssemblyRequest({
  root,
  sourceFiles: inputSourceFiles,
  importMetaGlobCap,
  records,
  includeSourceFiles = true,
  compactRecordIds: useCompactRecordIds = false,
  omitRecordIds = false,
  compactPaths = false,
  compactEnums = false,
  compactSpecifiers = false,
  compactNames = false,
  compactTypeOnly = false,
  compactRows = false,
}) {
  let sourceFiles = includeSourceFiles
    ? [...inputSourceFiles].map((file) => relPath(root, file))
    : [];
  let sourceFileIds = [];
  let outputRecords = useCompactRecordIds ? compactRecordIds(records) : records;
  let pathTable = [];
  if (compactPaths) {
    const compacted = compactRecordPaths(root, outputRecords, sourceFiles);
    outputRecords = compacted.records;
    pathTable = compacted.pathTable;
    sourceFiles = compacted.sourceFiles;
    sourceFileIds = compacted.sourceFileIds ?? [];
  }
  let kindTable = [];
  let resolverStageTable = [];
  let consumerSourceTable = [];
  if (compactEnums) {
    const compacted = compactRecordEnums(outputRecords);
    outputRecords = compacted.records;
    kindTable = compacted.kindTable;
    resolverStageTable = compacted.resolverStageTable;
    consumerSourceTable = compacted.consumerSourceTable;
  }
  let specifierTable = [];
  if (compactSpecifiers) {
    const compacted = compactRecordSpecifiers(outputRecords);
    outputRecords = compacted.records;
    specifierTable = compacted.specifierTable;
  }
  let nameTable = [];
  if (compactNames) {
    const compacted = compactRecordNames(outputRecords);
    outputRecords = compacted.records;
    nameTable = compacted.nameTable;
  }
  if (omitRecordIds) {
    outputRecords = outputRecords.map((record) => {
      const { recordId: _recordId, ...rest } = record;
      return rest;
    });
  }
  const compactedRows = compactRows
    ? compactRecordRows(outputRecords, { compactNames, compactTypeOnly })
    : null;
  return {
    schemaVersion: "lumin-source-use-assembly-request.v1",
    root,
    ...(importMetaGlobCap !== 64 ? { importMetaGlobCap } : {}),
    ...(sourceFiles.length > 0 ? { sourceFiles } : {}),
    ...(sourceFileIds.length > 0 ? { sourceFileIds } : {}),
    ...(pathTable.length > 0 ? { pathTable } : {}),
    ...(kindTable.length > 0 ? { kindTable } : {}),
    ...(resolverStageTable.length > 0 ? { resolverStageTable } : {}),
    ...(consumerSourceTable.length > 0 ? { consumerSourceTable } : {}),
    ...(specifierTable.length > 0 ? { specifierTable } : {}),
    ...(nameTable.length > 0 ? { nameTable } : {}),
    ...(compactedRows
      ? {
          recordRowFields: compactedRows.fields,
          recordRows: compactedRows.rows,
        }
      : { records: outputRecords }),
  };
}

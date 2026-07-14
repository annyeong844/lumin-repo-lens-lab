import { statSync } from 'node:fs';

import { runAuditCoreJsonResultFile } from './audit-core.mjs';
import { relPath } from './paths.mjs';

export const RUST_JS_EXTRACTOR_POLICY_VERSION = 'rust-js-extract-hybrid.v14';
const REQUEST_SCHEMA_VERSION = 'lumin-js-ts-extract-request.v1';
const MAX_BATCH_SOURCE_BYTES = 16 * 1024 * 1024;

function emptyReasonCounts() {
  return Object.create(null);
}

function pushRequestFile(batch, root, filePath) {
  batch.push({
    filePath,
    artifactFilePath: relPath(root, filePath),
  });
}

function sourceByteEstimate(filePath, fileSizes) {
  const knownSize = fileSizes?.get?.(filePath);
  if (Number.isFinite(knownSize) && knownSize >= 0) return knownSize;
  try {
    return statSync(filePath).size;
  } catch {
    return 0;
  }
}

function runRustJsFactsRequest(
  files,
  sourceFiles,
  label,
  requestText = null,
) {
  if (!Array.isArray(files) || files.length === 0) return [];
  const expectedPaths = files.map((file) => file?.filePath);
  if (
    expectedPaths.some(
      (filePath) => typeof filePath !== 'string' || filePath.length === 0,
    )
  ) {
    throw new TypeError('js-ts-extract-artifact: every file requires filePath');
  }
  if (new Set(expectedPaths).size !== expectedPaths.length) {
    throw new Error('js-ts-extract-artifact: duplicate filePath');
  }
  const response = runAuditCoreJsonResultFile(
    ['js-ts-extract-artifact', '--input', '-'],
    label,
    {
      input:
        requestText ??
        JSON.stringify({
          schemaVersion: REQUEST_SCHEMA_VERSION,
          sourceFiles: [...sourceFiles],
          files,
        }),
    },
  );
  if (
    response?.schemaVersion !== 'lumin-js-ts-extract-response.v1' ||
    !Array.isArray(response.files) ||
    response.files.length !== files.length
  ) {
    throw new Error('js-ts-extract-artifact: malformed result');
  }
  const byPath = new Map(response.files.map((file) => [file?.filePath, file]));
  return expectedPaths.map((filePath) => {
    const result = byPath.get(filePath);
    if (!result) {
      throw new Error(`js-ts-extract-artifact: missing result for ${filePath}`);
    }
    return result;
  });
}

export function extractRustJsFactsForSources(files, { sourceFiles = [] } = {}) {
  return runRustJsFactsRequest(
    files,
    sourceFiles,
    'js-ts-extract-artifact',
  );
}

export function extractRustJsHybridBatch({
  root,
  files,
  fileSizes,
  sourceFiles = files,
}) {
  const results = new Map();
  const warnings = [];
  const fallbackByReason = emptyReasonCounts();
  const summary = {
    policyVersion: RUST_JS_EXTRACTOR_POLICY_VERSION,
    maxBatchSourceBytes: MAX_BATCH_SOURCE_BYTES,
    candidateFiles: files.length,
    eligibleFiles: 0,
    fallbackFiles: 0,
    rustExtractedFiles: 0,
    rustResolvedRelativeUses: 0,
    rustParseErrorFiles: 0,
    readErrorFiles: 0,
    commandFailedFiles: 0,
    batchCount: 0,
    inputBytes: 0,
    sourceBytes: 0,
    fallbackByReason,
  };

  let batch = [];
  let batchBytes = 0;

  function flushBatch() {
    if (batch.length === 0) return;
    const request = {
      schemaVersion: REQUEST_SCHEMA_VERSION,
      sourceFiles: [...sourceFiles],
      files: batch,
    };
    const requestText = JSON.stringify(request);
    const currentBatch = batch;
    batch = [];
    batchBytes = 0;
    summary.batchCount++;
    summary.inputBytes += Buffer.byteLength(requestText, 'utf8');

    let responseFiles;
    try {
      responseFiles = runRustJsFactsRequest(
        currentBatch,
        request.sourceFiles,
        'symbols rust-js extractor',
        requestText,
      );
    } catch (error) {
      const reason = error?.message ?? 'unknown audit-core failure';
      throw new Error(`symbols rust-js extractor failed: ${reason}`, {
        cause: error,
      });
    }

    const byFile = new Map(
      responseFiles.map((file) => [file.filePath, file]),
    );
    for (const file of currentBatch) {
      const result = byFile.get(file.filePath);
      if (!result) {
        throw new Error(
          `symbols rust-js extractor omitted ${relPath(root, file.filePath)}`,
        );
      }
      results.set(file.filePath, result);
      if (result.error) summary.rustParseErrorFiles++;
      else {
        summary.rustExtractedFiles++;
        summary.rustResolvedRelativeUses += (result.uses ?? []).filter(
          (use) => use?.resolverStage === 'relative' && typeof use.resolvedFile === 'string',
        ).length;
      }
    }
  }

  for (const filePath of files) {
    const sourceBytes = sourceByteEstimate(filePath, fileSizes);

    summary.eligibleFiles++;
    if (batch.length > 0 && batchBytes + sourceBytes > MAX_BATCH_SOURCE_BYTES) {
      flushBatch();
    }
    summary.sourceBytes += sourceBytes;
    pushRequestFile(batch, root, filePath);
    batchBytes += sourceBytes;
  }

  flushBatch();
  return { results, summary, warnings };
}

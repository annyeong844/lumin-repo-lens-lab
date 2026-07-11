import { statSync } from 'node:fs';

import { runAuditCoreJsonResultFile } from './audit-core.mjs';
import { relPath } from './paths.mjs';

export const RUST_JS_EXTRACTOR_POLICY_VERSION = 'rust-js-extract-hybrid.v13';
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

    let response;
    try {
      response = runAuditCoreJsonResultFile(
        ['js-ts-extract-artifact', '--input', '-'],
        'symbols rust-js extractor',
        { input: requestText },
      );
    } catch (error) {
      const reason = error?.message ?? 'unknown audit-core failure';
      throw new Error(`symbols rust-js extractor failed: ${reason}`, {
        cause: error,
      });
    }

    if (!Array.isArray(response?.files)) {
      throw new Error('symbols rust-js extractor returned malformed files');
    }

    const byFile = new Map(
      response.files.map((file) => [file.filePath, file]),
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
